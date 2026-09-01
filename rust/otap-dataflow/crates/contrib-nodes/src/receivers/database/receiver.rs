// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Small shared polling receiver core.

use super::driver::{DriverAdapter, DriverCancellation, QueryResult};
use super::otlp::{rows_to_pdata, validate_mapping};
use super::query::CompiledQuery;
use super::scheduler::QueryScheduler;
use async_trait::async_trait;
use otel_arrow_dfe_channel::error::SendError;
use otel_arrow_dfe_engine::control::NodeControlMsg;
use otel_arrow_dfe_engine::error::{Error, ReceiverErrorKind, TypedError, format_error_sources};
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
use otel_arrow_dfe_telemetry::{otel_error, otel_warn};
use std::future::Future;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

/// Executes one compiled query through a database-specific adapter.
pub struct DatabaseReceiver<A> {
    adapter: A,
    query: CompiledQuery,
    scheduler: QueryScheduler,
    source_id: String,
}

impl<A> DatabaseReceiver<A>
where
    A: DriverAdapter,
{
    /// Creates a receiver with delay-based, non-overlapping scheduling.
    #[must_use]
    pub fn new(adapter: A, query: CompiledQuery, source_id: String) -> Self {
        let scheduler = QueryScheduler::new(query.interval());
        Self {
            adapter,
            query,
            scheduler,
            source_id,
        }
    }

    /// Executes immediately, primarily for startup validation and tests.
    pub async fn poll_once(&mut self) -> Result<QueryResult, A::Error> {
        _ = self.adapter.begin_operation()?;
        self.adapter.execute(&self.query).await
    }

    /// Waits until due, executes one poll, then schedules from completion.
    pub async fn next_poll(&mut self) -> Result<QueryResult, A::Error> {
        self.scheduler.wait().await;
        _ = self.adapter.begin_operation()?;
        let result = self.adapter.execute(&self.query).await;
        self.scheduler.complete();
        result
    }
}

#[async_trait(?Send)]
impl<A> local::Receiver<OtapPdata> for DatabaseReceiver<A>
where
    A: DriverAdapter + 'static,
{
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        // Prepare the live query before entering the ingestion loop. Selecting
        // over control messages keeps slow database startup drainable.
        let cancellation = self
            .adapter
            .begin_operation()
            .map_err(|error| receiver_error(&effect_handler, A::classify_error(&error), error))?;
        let columns = match await_database_operation_or_stop(
            self.adapter.validate_query(&self.query),
            cancellation,
            &mut ctrl_msg_recv,
        )
        .await?
        {
            OperationOutcome::Completed(result) => result.map_err(|error| {
                receiver_error(&effect_handler, A::classify_error(&error), error)
            })?,
            OperationOutcome::Stopped(stop) => {
                return finish_stop(stop, &effect_handler).await;
            }
        };
        validate_mapping(&columns, self.query.output()).map_err(|error| {
            receiver_error(&effect_handler, ReceiverErrorKind::Configuration, error)
        })?;

        loop {
            if let Some(stop) =
                wait_for_schedule_or_stop(self.scheduler.wait(), &mut ctrl_msg_recv).await?
            {
                return finish_stop(stop, &effect_handler).await;
            }
            let cancellation = self.adapter.begin_operation().map_err(|error| {
                receiver_error(&effect_handler, A::classify_error(&error), error)
            })?;
            let result = match await_database_operation_or_stop(
                self.adapter.execute(&self.query),
                cancellation,
                &mut ctrl_msg_recv,
            )
            .await?
            {
                OperationOutcome::Completed(result) => {
                    self.scheduler.complete();
                    result
                }
                OperationOutcome::Stopped(stop) => {
                    return finish_stop(stop, &effect_handler).await;
                }
            };

            match result {
                Ok(result) if !result.rows.is_empty() => {
                    let observed_time = observed_time_unix_nano().map_err(|error| {
                        receiver_error(&effect_handler, ReceiverErrorKind::Other, error)
                    })?;
                    match rows_to_pdata(
                        result,
                        self.adapter.system(),
                        &self.source_id,
                        self.query.output(),
                        observed_time,
                    ) {
                        Ok(pdata) => {
                            if let Some(terminal) =
                                send_or_stop(pdata, &mut ctrl_msg_recv, &effect_handler).await?
                            {
                                return Ok(terminal);
                            }
                        }
                        Err(error) => {
                            // The design defines fail-batch as the default but
                            // does not define a public policy field yet.
                            otel_error!(
                                "database_receiver.batch_conversion_failed",
                                error = %error,
                                message = "Database receiver discarded one failed conversion batch"
                            );
                        }
                    }
                }
                Ok(_) => {}
                Err(error) if self.adapter.is_batch_error(&error) => {
                    // Driver normalization errors follow the same fixed
                    // first-slice fail-batch behavior.
                    otel_error!(
                        "database_receiver.batch_normalization_failed",
                        error = %error,
                        message = "Database receiver discarded one failed batch"
                    );
                }
                Err(error) => {
                    return Err(receiver_error(
                        &effect_handler,
                        A::classify_error(&error),
                        error,
                    ));
                }
            }
        }
    }
}

enum OperationOutcome<T> {
    Completed(T),
    Stopped(StopRequest),
}

#[derive(Clone, Copy)]
enum StopRequest {
    Drain(std::time::Instant),
    Shutdown(std::time::Instant),
}

async fn wait_for_schedule_or_stop<F>(
    wait: F,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
) -> Result<Option<StopRequest>, Error>
where
    F: Future<Output = ()>,
{
    tokio::pin!(wait);
    loop {
        tokio::select! {
            biased;

            control = ctrl_msg_recv.recv() => {
                let control = control.map_err(Error::ChannelRecvError)?;
                if let Some(stop) = stop_request(control) {
                    return Ok(Some(stop));
                }
            }
            () = &mut wait => return Ok(None),
        }
    }
}

async fn await_database_operation_or_stop<F, T, C>(
    operation: F,
    cancellation: C,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
) -> Result<OperationOutcome<T>, Error>
where
    F: Future<Output = T>,
    C: DriverCancellation,
{
    tokio::pin!(operation);
    loop {
        tokio::select! {
            biased;

            control = ctrl_msg_recv.recv() => {
                match control {
                    Ok(control) => {
                        if let Some(stop) = stop_request(control) {
                            cancel_and_join(operation.as_mut(), &cancellation).await;
                            return Ok(OperationOutcome::Stopped(stop));
                        }
                    }
                    Err(error) => {
                        cancel_and_join(operation.as_mut(), &cancellation).await;
                        return Err(Error::ChannelRecvError(error));
                    }
                }
            }
            result = &mut operation => return Ok(OperationOutcome::Completed(result)),
        }
    }
}

async fn cancel_and_join<F, C>(mut operation: Pin<&mut F>, cancellation: &C)
where
    F: Future,
    C: DriverCancellation,
{
    if let Err(error) = cancellation.cancel().await {
        otel_warn!(
            "database_receiver.cancellation_failed",
            error = %error,
            message = "Database receiver could not interrupt its active operation"
        );
    }
    // Join blocking work even when cancellation itself fails, so a
    // replacement receiver cannot overlap this operation.
    _ = operation.as_mut().await;
}

async fn send_or_stop(
    pdata: OtapPdata,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
    effect_handler: &local::EffectHandler<OtapPdata>,
) -> Result<Option<TerminalState>, Error> {
    let pdata = match effect_handler.try_send_message(pdata) {
        Ok(()) => return Ok(None),
        Err(TypedError::ChannelSendError(SendError::Full(pdata))) => pdata,
        Err(error) => return Err(error.into()),
    };
    let send = effect_handler.send_message(pdata);
    tokio::pin!(send);
    let mut drain_deadline = None;

    loop {
        tokio::select! {
            biased;

            _ = async {
                if let Some(deadline) = drain_deadline {
                    tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
                }
            }, if drain_deadline.is_some() => {
                let Some(deadline) = drain_deadline else {
                    continue;
                };
                otel_warn!(
                    "database_receiver.drain_deadline_reached",
                    message = "Database receiver drain deadline reached while sending downstream"
                );
                return finish_stop(StopRequest::Drain(deadline), effect_handler)
                    .await
                    .map(Some);
            }
            control = ctrl_msg_recv.recv() => {
                let control = control.map_err(Error::ChannelRecvError)?;
                match stop_request(control) {
                    Some(StopRequest::Drain(deadline)) => {
                        drain_deadline = Some(deadline);
                    }
                    Some(stop @ StopRequest::Shutdown(_)) => {
                        return finish_stop(stop, effect_handler).await.map(Some);
                    }
                    None => {}
                }
            }
            result = &mut send => {
                result.map_err(Error::from)?;
                return match drain_deadline {
                    Some(deadline) => finish_stop(StopRequest::Drain(deadline), effect_handler)
                        .await
                        .map(Some),
                    None => Ok(None),
                };
            }
        }
    }
}

fn stop_request(control: NodeControlMsg<OtapPdata>) -> Option<StopRequest> {
    match control {
        NodeControlMsg::DrainIngress { deadline, .. } => Some(StopRequest::Drain(deadline)),
        NodeControlMsg::Shutdown { deadline, .. } => Some(StopRequest::Shutdown(deadline)),
        _ => None,
    }
}

async fn finish_stop(
    stop: StopRequest,
    effect_handler: &local::EffectHandler<OtapPdata>,
) -> Result<TerminalState, Error> {
    let deadline = match stop {
        StopRequest::Drain(deadline) => {
            effect_handler.notify_receiver_drained().await?;
            deadline
        }
        StopRequest::Shutdown(deadline) => deadline,
    };
    Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []))
}

fn observed_time_unix_nano() -> Result<u64, ObservationTimeError> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
    // OTLP uses u64 nanoseconds. Reject an unrepresentable clock value rather
    // than silently saturating and emitting a misleading timestamp.
    Ok(duration.as_nanos().try_into()?)
}

#[derive(Debug, thiserror::Error)]
enum ObservationTimeError {
    /// The system clock is earlier than the Unix epoch.
    #[error("system clock is earlier than the Unix epoch")]
    BeforeUnixEpoch(#[from] std::time::SystemTimeError),
    /// Nanoseconds since the epoch do not fit OTLP's unsigned 64-bit field.
    #[error("observation time is outside the supported OTLP range")]
    OutOfRange(#[from] std::num::TryFromIntError),
}

fn receiver_error(
    effect_handler: &local::EffectHandler<OtapPdata>,
    kind: ReceiverErrorKind,
    error: impl std::error::Error + 'static,
) -> Error {
    let source_detail = format_error_sources(&error);
    Error::ReceiverError {
        receiver: effect_handler.receiver_id(),
        kind,
        error: error.to_string(),
        source_detail,
    }
}

#[cfg(test)]
#[path = "receiver_tests.rs"]
mod tests;
