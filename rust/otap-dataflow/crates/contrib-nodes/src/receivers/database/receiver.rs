// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Small shared polling receiver core.

use super::driver::{DriverAdapter, QueryResult};
use super::otlp::{rows_to_pdata, validate_mapping};
use super::query::CompiledQuery;
use super::scheduler::QueryScheduler;
use async_trait::async_trait;
use otel_arrow_dfe_engine::control::NodeControlMsg;
use otel_arrow_dfe_engine::error::{Error, ReceiverErrorKind};
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
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
        self.adapter.execute(&self.query).await
    }

    /// Waits until due, executes one poll, then schedules from completion.
    pub async fn next_poll(&mut self) -> Result<QueryResult, A::Error> {
        self.scheduler.wait().await;
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
        let columns = {
            let validation = self.adapter.validate_query(&self.query);
            tokio::pin!(validation);
            loop {
                tokio::select! {
                    control = ctrl_msg_recv.recv() => {
                        if let Some(terminal) = startup_control(control, &effect_handler).await? {
                            return Ok(terminal);
                        }
                    }
                    result = &mut validation => {
                        break result.map_err(|error| receiver_error(&effect_handler, error))?;
                    }
                }
            }
        };
        validate_mapping(&columns, self.query.output())
            .map_err(|error| receiver_error(&effect_handler, error))?;

        loop {
            let result = {
                // Keep this future pinned across non-terminal control messages.
                // Dropping it could detach its spawn_blocking Oracle worker and
                // let the next loop iteration start an overlapping query.
                let poll = self.next_poll();
                tokio::pin!(poll);
                loop {
                    tokio::select! {
                        control = ctrl_msg_recv.recv() => {
                            if let Some(terminal) =
                                startup_control(control, &effect_handler).await?
                            {
                                return Ok(terminal);
                            }
                        }
                        result = &mut poll => break result,
                    }
                }
            };

            match result {
                Ok(result) if !result.rows.is_empty() => {
                    let observed_time = observed_time_unix_nano()
                        .map_err(|error| receiver_error(&effect_handler, error))?;
                    match rows_to_pdata(
                        result,
                        self.adapter.system(),
                        &self.source_id,
                        self.query.output(),
                        observed_time,
                    ) {
                        Ok(pdata) => effect_handler.send_message(pdata).await?,
                        Err(error) => {
                            // The design defines fail-batch as the default but
                            // does not define a public policy field yet.
                            tracing::error!(
                                error = %error,
                                "database receiver discarded one failed conversion batch"
                            );
                        }
                    }
                }
                Ok(_) => {}
                Err(error) if self.adapter.is_batch_error(&error) => {
                    // Driver normalization errors follow the same fixed
                    // first-slice fail-batch behavior.
                    tracing::error!(
                        error = %error,
                        "database receiver discarded one failed batch"
                    );
                }
                Err(error) => {
                    return Err(receiver_error(&effect_handler, error));
                }
            }
        }
    }
}

async fn startup_control(
    control: Result<NodeControlMsg<OtapPdata>, otel_arrow_dfe_channel::error::RecvError>,
    effect_handler: &local::EffectHandler<OtapPdata>,
) -> Result<Option<TerminalState>, Error> {
    match control {
        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
            effect_handler.notify_receiver_drained().await?;
            Ok(Some(TerminalState::new::<[MetricSetSnapshot; 0]>(
                deadline,
                [],
            )))
        }
        Ok(NodeControlMsg::Shutdown { deadline, .. }) => Ok(Some(TerminalState::new::<
            [MetricSetSnapshot; 0],
        >(deadline, []))),
        Ok(_) => Ok(None),
        Err(error) => Err(Error::ChannelRecvError(error)),
    }
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
    error: impl std::fmt::Display,
) -> Error {
    Error::ReceiverError {
        receiver: effect_handler.receiver_id(),
        kind: ReceiverErrorKind::Other,
        error: error.to_string(),
        source_detail: String::new(),
    }
}
