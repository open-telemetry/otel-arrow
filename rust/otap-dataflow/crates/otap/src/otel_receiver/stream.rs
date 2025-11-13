// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::ack::{
    AckPollResult, AckRegistry, AckToken, nack_status, overloaded_status, success_status,
};
use crate::otap_grpc::ArrowRequestStream;
use crate::pdata::{Context, OtapPdata};
use futures::Stream;
use futures::future::{LocalBoxFuture, poll_fn};
use futures::stream::FuturesUnordered;
use log::error;
use otap_df_engine::local::receiver as local;
use otap_df_engine::{Interests, ProducerEffectHandlerExtension};
use otap_df_pdata::Consumer;
use otap_df_pdata::otap::{OtapArrowRecords, OtapBatchStore, from_record_messages};
use otap_df_pdata::proto::opentelemetry::arrow::v1::{BatchArrowRecords, BatchStatus};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use tonic::Status;

pub(crate) fn stream_batch_statuses<S, T, F>(
    input_stream: S,
    effect_handler: local::EffectHandler<OtapPdata>,
    ack_registry: Option<AckRegistry>,
    otap_batch: F,
    max_in_flight_per_connection: usize,
) -> StatusStream<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    let state = StatusStreamState::new(
        input_stream,
        effect_handler,
        ack_registry,
        otap_batch,
        max_in_flight_per_connection,
    );
    StatusStream::new(state)
}

pub(crate) struct StatusStream<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    state: Option<StatusStreamState<S, T, F>>,
    pending: Option<LocalBoxFuture<'static, (StatusStreamState<S, T, F>, StreamStep)>>,
    finished: bool,
}

impl<S, T, F> StatusStream<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    fn new(state: StatusStreamState<S, T, F>) -> Self {
        Self {
            state: Some(state),
            pending: None,
            finished: false,
        }
    }

    fn drive_next(
        state: StatusStreamState<S, T, F>,
    ) -> LocalBoxFuture<'static, (StatusStreamState<S, T, F>, StreamStep)> {
        Box::pin(async move {
            let mut state = state;
            let step = state.next_item().await;
            (state, step)
        })
    }
}

impl<S, T, F> Stream for StatusStream<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    type Item = Result<BatchStatus, Status>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.finished {
            return Poll::Ready(None);
        }

        if this.pending.is_none() {
            let state = match this.state.take() {
                Some(state) => state,
                None => {
                    this.finished = true;
                    return Poll::Ready(None);
                }
            };
            this.pending = Some(Self::drive_next(state));
        }

        match this
            .pending
            .as_mut()
            .expect("pending future must exist")
            .as_mut()
            .poll(cx)
        {
            Poll::Pending => Poll::Pending,
            Poll::Ready((state, step)) => {
                this.pending = None;
                match step {
                    StreamStep::Yield(item) => {
                        this.state = Some(state);
                        Poll::Ready(Some(item))
                    }
                    StreamStep::Done => {
                        this.finished = true;
                        this.state = None;
                        Poll::Ready(None)
                    }
                }
            }
        }
    }
}

struct StatusStreamState<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    input_stream: S,
    consumer: Consumer,
    effect_handler: local::EffectHandler<OtapPdata>,
    state: Option<AckRegistry>,
    otap_batch: F,
    in_flight: InFlightSet<AckWaitFuture>,
    max_in_flight: usize,
    finished: bool,
    _marker: PhantomData<fn() -> T>,
}

enum StreamStep {
    Yield(Result<BatchStatus, Status>),
    Done,
}

enum PreparedBatch {
    Enqueued,
    Immediate(StreamStep),
}

impl<S, T, F> StatusStreamState<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    fn new(
        input_stream: S,
        effect_handler: local::EffectHandler<OtapPdata>,
        state: Option<AckRegistry>,
        otap_batch: F,
        max_in_flight_per_connection: usize,
    ) -> Self {
        Self {
            input_stream,
            consumer: Consumer::default(),
            effect_handler,
            state,
            otap_batch,
            in_flight: InFlightSet::with_capacity(max_in_flight_per_connection.max(1)),
            max_in_flight: max_in_flight_per_connection.max(1),
            finished: false,
            _marker: PhantomData,
        }
    }

    async fn next_item(&mut self) -> StreamStep {
        if let Some(step) = self.fill_inflight().await {
            return step;
        }

        match poll_fn(|cx| self.in_flight.poll_next(cx)).await {
            Some(step) => {
                if matches!(step, StreamStep::Done) {
                    self.finished = true;
                }
                step
            }
            None => StreamStep::Done,
        }
    }

    async fn fill_inflight(&mut self) -> Option<StreamStep> {
        while !self.finished && self.in_flight.len() < self.max_in_flight {
            match self.input_stream.next_message().await {
                Ok(Some(batch)) => match self.enqueue_batch(batch).await {
                    PreparedBatch::Enqueued => continue,
                    PreparedBatch::Immediate(step) => return Some(step),
                },
                Ok(None) => {
                    self.finished = true;
                    break;
                }
                Err(status) => {
                    self.finished = true;
                    return Some(StreamStep::Yield(Err(status)));
                }
            }
        }
        None
    }

    async fn enqueue_batch(&mut self, mut batch: BatchArrowRecords) -> PreparedBatch {
        let batch_id = batch.batch_id;

        let batch = match self.consumer.consume_bar(&mut batch) {
            Ok(batch) => batch,
            Err(e) => {
                error!("Error decoding OTAP Batch: {e:?}. Closing stream");
                self.finished = true;
                return PreparedBatch::Immediate(StreamStep::Done);
            }
        };

        let batch = from_record_messages::<T>(batch);
        let otap_batch_as_otap_arrow_records = (self.otap_batch)(batch);
        let mut otap_pdata =
            OtapPdata::new(Context::default(), otap_batch_as_otap_arrow_records.into());

        let wait_token = if let Some(state) = self.state.clone() {
            match state.allocate() {
                None => {
                    error!("Too many concurrent requests");
                    return PreparedBatch::Immediate(StreamStep::Yield(Ok(overloaded_status(
                        batch_id,
                    ))));
                }
                Some(token) => {
                    self.effect_handler.subscribe_to(
                        Interests::ACKS | Interests::NACKS,
                        token.to_calldata(),
                        &mut otap_pdata,
                    );
                    Some((state, token))
                }
            }
        } else {
            None
        };

        if let Err(e) = self.effect_handler.send_message(otap_pdata).await {
            error!("Failed to send to pipeline: {e}");
            self.finished = true;
            return PreparedBatch::Immediate(StreamStep::Done);
        };

        if let Some((state, token)) = wait_token {
            if let Err(_future) = self
                .in_flight
                .push(AckWaitFuture::new(batch_id, token, state))
            {
                error!("In-flight future set unexpectedly full");
                return PreparedBatch::Immediate(StreamStep::Yield(Ok(overloaded_status(
                    batch_id,
                ))));
            }
            PreparedBatch::Enqueued
        } else {
            PreparedBatch::Immediate(StreamStep::Yield(Ok(success_status(batch_id))))
        }
    }
}

struct InFlightSet<F> {
    futures: FuturesUnordered<F>,
    capacity: usize,
}

impl<F> InFlightSet<F> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            futures: FuturesUnordered::new(),
            capacity,
        }
    }

    fn len(&self) -> usize {
        self.futures.len()
    }

    fn push(&mut self, future: F) -> Result<(), F> {
        if self.len() >= self.capacity {
            Err(future)
        } else {
            self.futures.push(future);
            Ok(())
        }
    }

    fn poll_next(&mut self, cx: &mut TaskContext<'_>) -> Poll<Option<<F as Future>::Output>>
    where
        F: Future + Unpin,
    {
        Pin::new(&mut self.futures).poll_next(cx)
    }
}

struct AckWaitFuture {
    batch_id: i64,
    token: AckToken,
    state: AckRegistry,
    completed: bool,
}

impl AckWaitFuture {
    fn new(batch_id: i64, token: AckToken, state: AckRegistry) -> Self {
        Self {
            batch_id,
            token,
            state,
            completed: false,
        }
    }
}

impl Future for AckWaitFuture {
    type Output = StreamStep;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.state.poll_slot(this.token, cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(AckPollResult::Ack) => {
                this.completed = true;
                Poll::Ready(StreamStep::Yield(Ok(success_status(this.batch_id))))
            }
            Poll::Ready(AckPollResult::Nack(reason)) => {
                this.completed = true;
                Poll::Ready(StreamStep::Yield(Ok(nack_status(this.batch_id, reason))))
            }
            Poll::Ready(AckPollResult::Cancelled) => {
                this.completed = true;
                Poll::Ready(StreamStep::Done)
            }
        }
    }
}

impl Drop for AckWaitFuture {
    fn drop(&mut self) {
        if !self.completed {
            self.state.cancel(self.token);
        }
    }
}
