// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Glue between incoming OTAP Arrow streams and the ACK registry.
//!
//! `StatusStream` fans batches from a gRPC request into the downstream pipeline and
//! fans ACK/NACK notifications back into `BatchStatus` responses. The bounded
//! registry supplies backpressure—when no tokens remain we immediately return an
//! overloaded status and stop enqueueing more work. Each stream also tracks how many
//! batches are currently `in_flight`, so a single connection can’t monopolize every
//! token. Both limits interact with the rest of the system by gating how much work
//! we hand the pipeline; excess demand is reflected back to the client. Everything
//! runs on the single-threaded runtime, so the implementation uses `Rc`/`RefCell` and futures
//! that never cross threads, operations such as enqueueing batches, polling inflight
//! futures, and reclaiming registry slots are all O(1), keeping the hot path
//! predictable under load.

use super::ack::{
    AckPollResult, AckRegistry, AckToken, nack_status, overloaded_status, success_status,
};
use super::grpc::RequestStream;
use crate::otel_receiver::status::Status;
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

/// Builds the stream of OTAP batch statuses for a single gRPC request.
pub(crate) fn stream_batch_statuses<S, T, F>(
    input_stream: S,
    effect_handler: local::EffectHandler<OtapPdata>,
    ack_registry: Option<AckRegistry>,
    otap_batch: F,
    max_in_flight_per_connection: usize,
) -> StatusStream<S, T, F>
where
    S: RequestStream + Unpin + 'static,
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

/// Drives an inbound OTAP stream while waiting for ACK or NACK outcomes.
///
/// Each `StatusStream` instance is bound to a single gRPC stream and lives
/// entirely on the local executor. It pushes decoded batches into the pipeline
/// and yields `BatchStatus` responses as soon as the corresponding ACK futures
/// complete.
pub(crate) struct StatusStream<S, T, F>
where
    S: RequestStream + Unpin + 'static,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    state: Option<StatusStreamState<S, T, F>>,
    pending: Option<LocalBoxFuture<'static, (StatusStreamState<S, T, F>, StreamStep)>>,
    finished: bool,
}

impl<S, T, F> StatusStream<S, T, F>
where
    S: RequestStream + Unpin + 'static,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    /// Wraps the prepared state in the facade consumed by the router.
    fn new(state: StatusStreamState<S, T, F>) -> Self {
        Self {
            state: Some(state),
            pending: None,
            finished: false,
        }
    }

    /// Drives the state machine until it produces the next `StreamStep`.
    ///
    /// Internally this means "fill" until we either enqueue more work or hit an
    /// error, and if that doesn't yield anything new we "drain" by awaiting the
    /// next inflight ACK future.
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
    S: RequestStream + Unpin + 'static,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    type Item = Result<BatchStatus, Status>;

    /// Implements the `Stream` contract by repeatedly driving the state machine.
    fn poll_next(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.finished {
            return Poll::Ready(None);
        }

        if this.pending.is_none() {
            // Lazily grab ownership of the state the first time we are polled. If it
            // is already `None` we know the stream is complete.
            let state = match this.state.take() {
                Some(state) => state,
                None => {
                    this.finished = true;
                    return Poll::Ready(None);
                }
            };
            // Kick off an async step that will either enqueue more work or drain an inflight future.
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
                        // Save the updated state and yield the status/error to the caller.
                        this.state = Some(state);
                        Poll::Ready(Some(item))
                    }
                    StreamStep::Done => {
                        // No more work; mark finished and drop the state.
                        this.finished = true;
                        this.state = None;
                        Poll::Ready(None)
                    }
                }
            }
        }
    }
}

/// Mutable state for a single `StatusStream`.
///
/// Holds:
/// - the decoded gRPC input stream,
/// - the local effect handler into the pipeline,
/// - an optional `AckRegistry` for wait for result mode,
/// - a bounded set of in flight ACK wait futures, and
/// - a per connection `max_in_flight` limit that prevents a single client
///   from monopolizing all wait slots.
struct StatusStreamState<S, T, F>
where
    S: RequestStream + Unpin + 'static,
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

/// What the stream should do next.
enum StreamStep {
    /// Emit a `BatchStatus` (or gRPC error) downstream.
    Yield(Result<BatchStatus, Status>),
    /// Tear down the stream.
    /// No more messages will be produced.
    Done,
}

/// Result of attempting to enqueue a batch into the pipeline.
enum PreparedBatch {
    /// The batch was queued and we should continue filling/draining.
    Enqueued,
    /// The batch triggered an immediate status (success/failure) or termination.
    Immediate(StreamStep),
}

impl<S, T, F> StatusStreamState<S, T, F>
where
    S: RequestStream + Unpin + 'static,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    /// Creates state for a single inbound connection/request.
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

    /// Pulls the next work item by either filling or draining the pipeline.
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

    /// Attempts to enqueue additional batches while respecting capacity limits.
    ///
    /// At most `max_in_flight` iterations and each operation is O(1), so the loop
    /// remains bounded even when the inbound stream is eager.
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

    /// Converts an incoming `BatchArrowRecords` into pipeline work plus wait token.
    ///
    /// Aside from the actual pipeline send (which is async), all bookkeeping here
    /// is O(1): decoding, registry allocation, and inflight bookkeeping simply
    /// index into fixed-size structures.
    async fn enqueue_batch(&mut self, mut batch: BatchArrowRecords) -> PreparedBatch {
        let batch_id = batch.batch_id;

        // Decode the batch. Because this receiver pulls everything onto a single
        // thread, there is no concurrent mutation of `batch` after this point.
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

        // Push the batch into the downstream pipeline. This is the only `.await`
        // in the method and will yield until the local channel accepts the data.
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

/// Bounded collection of ACK wait futures that enforces per stream inflight limits.
///
/// All operations are O(1) and polling is delegated to `FuturesUnordered`.
struct InFlightSet<F> {
    futures: FuturesUnordered<F>,
    capacity: usize,
}

impl<F> InFlightSet<F> {
    /// Creates a set that can hold up to `capacity` futures.
    fn with_capacity(capacity: usize) -> Self {
        Self {
            futures: FuturesUnordered::new(),
            capacity,
        }
    }

    /// Returns the number of currently tracked futures.
    fn len(&self) -> usize {
        self.futures.len()
    }

    /// Attempts to push a future, returning it back if the set is full.
    fn push(&mut self, future: F) -> Result<(), F> {
        if self.len() >= self.capacity {
            Err(future)
        } else {
            self.futures.push(future);
            Ok(())
        }
    }

    /// Polls the underlying futures, forwarding readiness to the caller.
    fn poll_next(&mut self, cx: &mut TaskContext<'_>) -> Poll<Option<<F as Future>::Output>>
    where
        F: Future + Unpin,
    {
        Pin::new(&mut self.futures).poll_next(cx)
    }
}

/// Future that resolves once a specific batch receives an ACK or NACK.
struct AckWaitFuture {
    batch_id: i64,
    token: AckToken,
    state: AckRegistry,
    completed: bool,
}

impl AckWaitFuture {
    /// Builds a wait future tied to the provided registry token.
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

    /// Resolves once the registry slot finishes with ACK/NACK/cancelled.
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
    /// Ensures the registry slot is released if the future is dropped early.
    fn drop(&mut self) {
        if !self.completed {
            self.state.cancel(self.token);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otap_mock::create_otap_batch;
    use crate::otel_receiver::ack::{AckRegistry, AckToken};
    use crate::otel_receiver::grpc::RequestStream;
    use crate::otel_receiver::status::Status;
    use crate::pdata::OtapPdata;
    use async_trait::async_trait;
    use futures::StreamExt;
    use otap_df_channel::mpsc;
    use otap_df_config::PortName;
    use otap_df_engine::control::pipeline_ctrl_msg_channel;
    use otap_df_engine::local::message::LocalSender;
    use otap_df_engine::local::receiver as local;
    use otap_df_engine::testing::test_node;
    use otap_df_pdata::Producer;
    use otap_df_pdata::otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces};
    use otap_df_pdata::proto::opentelemetry::arrow::v1::{
        ArrowPayloadType, BatchArrowRecords, StatusCode as ProtoStatusCode,
    };
    use otap_df_telemetry::reporter::MetricsReporter;
    use std::collections::{HashMap, HashSet, VecDeque};
    use tokio::task::yield_now;
    use tokio::time::Duration;

    struct FakeArrowStream {
        batches: VecDeque<BatchArrowRecords>,
    }

    impl FakeArrowStream {
        fn new(batches: VecDeque<BatchArrowRecords>) -> Self {
            Self { batches }
        }
    }

    #[async_trait(?Send)]
    impl RequestStream for FakeArrowStream {
        async fn next_message(&mut self) -> Result<Option<BatchArrowRecords>, Status> {
            Ok(self.batches.pop_front())
        }
    }

    fn build_test_effect_handler(
        channel_capacity: usize,
    ) -> (local::EffectHandler<OtapPdata>, mpsc::Receiver<OtapPdata>) {
        let (tx, rx) = mpsc::Channel::new(channel_capacity);
        let mut senders = HashMap::new();
        let default_port: PortName = PortName::from("default");
        let _ = senders.insert(default_port.clone(), LocalSender::MpscSender(tx));
        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let effect_handler = local::EffectHandler::new(
            test_node("otel_receiver_status_test"),
            senders,
            Some(default_port),
            ctrl_tx,
            metrics_reporter,
        );
        (effect_handler, rx)
    }

    fn arrow_batches(
        payload_type: ArrowPayloadType,
        batch_count: usize,
    ) -> VecDeque<BatchArrowRecords> {
        let mut queue = VecDeque::with_capacity(batch_count);
        let mut producer = Producer::new();
        for batch_index in 0..batch_count {
            let mut batch = create_otap_batch(batch_index as i64, payload_type);
            let bar = producer
                .produce_bar(&mut batch)
                .expect("failed to encode arrow batch");
            queue.push_back(bar);
        }
        queue
    }

    async fn drive_ack_pipeline(
        pdata_rx: mpsc::Receiver<OtapPdata>,
        ack_registry: AckRegistry,
        total_batches: usize,
    ) -> (usize, usize) {
        let mut success = 0;
        let mut failure = 0;
        for idx in 0..total_batches {
            let pdata = pdata_rx
                .recv()
                .await
                .expect("pdata channel closed unexpectedly");
            let calldata = pdata
                .current_calldata()
                .expect("missing calldata for wait_for_result");
            let token = AckToken::from_calldata(&calldata).expect("invalid ack token");

            if idx % 11 == 0 {
                tokio::time::sleep(Duration::from_micros(((idx % 5) + 1) as u64 * 20)).await;
                let _ = ack_registry.complete(token, Err(format!("failure #{idx}")));
                failure += 1;
            } else {
                if idx % 7 == 0 {
                    tokio::time::sleep(Duration::from_micros(((idx % 3) + 1) as u64 * 10)).await;
                } else if idx % 3 == 0 {
                    yield_now().await;
                }
                let _ = ack_registry.complete(token, Ok(()));
                success += 1;
            }
        }
        (success, failure)
    }

    async fn run_status_stream_load_test<T>(
        payload_type: ArrowPayloadType,
        otap_batch: fn(T) -> OtapArrowRecords,
    ) where
        T: OtapBatchStore + 'static,
    {
        const TOTAL_BATCHES: usize = 1024;
        const MAX_CONCURRENT_REQUESTS: usize = 256;
        const MAX_IN_FLIGHT: usize = 64;

        let stream = FakeArrowStream::new(arrow_batches(payload_type, TOTAL_BATCHES));
        let (effect_handler, pdata_rx) = build_test_effect_handler(TOTAL_BATCHES);
        let ack_registry = AckRegistry::new(MAX_CONCURRENT_REQUESTS);

        let mut status_stream = stream_batch_statuses::<_, T, _>(
            stream,
            effect_handler,
            Some(ack_registry.clone()),
            otap_batch,
            MAX_IN_FLIGHT,
        );

        let ack_task = drive_ack_pipeline(pdata_rx, ack_registry.clone(), TOTAL_BATCHES);

        let status_task = async {
            let mut successes = 0;
            let mut failures = 0;
            let mut ids = HashSet::with_capacity(TOTAL_BATCHES);
            while let Some(next) = status_stream.next().await {
                let status = next.expect("receiver should not emit tonic errors");
                assert!(
                    ids.insert(status.batch_id),
                    "duplicate status for batch {}",
                    status.batch_id
                );
                match status.status_code {
                    code if code == ProtoStatusCode::Ok as i32 => {
                        assert_eq!(status.status_message, "Successfully received");
                        successes += 1;
                    }
                    code if code == ProtoStatusCode::Unavailable as i32 => {
                        assert!(
                            status
                                .status_message
                                .starts_with("Pipeline processing failed:"),
                            "unexpected failure message {}",
                            status.status_message
                        );
                        failures += 1;
                    }
                    other => panic!("unexpected status code {other}"),
                }
            }
            assert_eq!(ids.len(), TOTAL_BATCHES);
            (successes, failures)
        };

        let ((expected_successes, expected_failures), (actual_successes, actual_failures)) =
            tokio::join!(ack_task, status_task);

        assert_eq!(actual_successes, expected_successes);
        assert_eq!(actual_failures, expected_failures);
        assert_eq!(actual_successes + actual_failures, TOTAL_BATCHES);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn stream_batch_statuses_handles_large_ack_nack_load() {
        run_status_stream_load_test::<Metrics>(
            ArrowPayloadType::MultivariateMetrics,
            OtapArrowRecords::Metrics,
        )
        .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn stream_batch_statuses_handles_large_logs_load() {
        run_status_stream_load_test::<Logs>(ArrowPayloadType::Logs, OtapArrowRecords::Logs).await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn stream_batch_statuses_handles_large_traces_load() {
        run_status_stream_load_test::<Traces>(ArrowPayloadType::Spans, OtapArrowRecords::Traces)
            .await;
    }
}
