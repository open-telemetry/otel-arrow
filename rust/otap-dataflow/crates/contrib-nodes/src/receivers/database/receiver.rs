// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Small shared polling receiver core.

use super::driver::{DriverAdapter, QueryResult};
use super::otlp::{rows_to_pdata, validate_mapping};
use super::query::CompiledQuery;
use super::scheduler::QueryScheduler;
use super::ErrorPolicy;
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
    query_name: String,
}

impl<A> DatabaseReceiver<A>
where
    A: DriverAdapter,
{
    /// Creates a receiver with delay-based, non-overlapping scheduling.
    #[must_use]
    pub fn new(
        adapter: A,
        query: CompiledQuery,
        source_id: String,
        query_name: String,
    ) -> Self {
        let scheduler = QueryScheduler::new(query.interval());
        Self {
            adapter,
            query,
            scheduler,
            source_id,
            query_name,
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
            tokio::select! {
                control = ctrl_msg_recv.recv() => {
                    match control {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Err(error) => return Err(Error::ChannelRecvError(error)),
                        _ => {}
                    }
                }
                result = self.next_poll() => {
                    match result {
                        Ok(result) if !result.rows.is_empty() => {
                            let observed_time = observed_time_unix_nano()
                                .map_err(|error| receiver_error(&effect_handler, error))?;
                            match rows_to_pdata(
                                result,
                                self.adapter.system(),
                                &self.source_id,
                                &self.query_name,
                                self.query.output(),
                                observed_time,
                            ) {
                                Ok(pdata) => effect_handler.send_message(pdata).await?,
                                Err(error) => {
                                    if let Some(terminal) = apply_error_policy(
                                        self.query.error_policy(),
                                        "OTLP conversion",
                                        error,
                                        &mut ctrl_msg_recv,
                                        &effect_handler,
                                    ).await? {
                                        return Ok(terminal);
                                    }
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(error) if self.adapter.is_batch_error(&error) => {
                            if let Some(terminal) = apply_error_policy(
                                self.query.error_policy(),
                                "database row conversion",
                                error,
                                &mut ctrl_msg_recv,
                                &effect_handler,
                            ).await? {
                                return Ok(terminal);
                            }
                        }
                        Err(error) => {
                            return Err(receiver_error(&effect_handler, error));
                        }
                    }
                }
            }
        }
    }
}

async fn apply_error_policy(
    policy: ErrorPolicy,
    operation: &'static str,
    error: impl std::fmt::Display,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
    effect_handler: &local::EffectHandler<OtapPdata>,
) -> Result<Option<TerminalState>, Error> {
    match policy {
        ErrorPolicy::FailBatch => {
            tracing::error!(
                operation,
                error = %error,
                "database receiver discarded one failed batch"
            );
            Ok(None)
        }
        ErrorPolicy::StopQuery => {
            tracing::error!(
                operation,
                error = %error,
                "database receiver stopped polling after a permanent query failure"
            );
            // This foundation has one query per receiver. Keeping the node
            // alive lets orchestration still drain or shut it down cleanly.
            loop {
                if let Some(terminal) =
                    startup_control(ctrl_msg_recv.recv().await, effect_handler).await?
                {
                    return Ok(Some(terminal));
                }
            }
        }
        ErrorPolicy::StopReceiver => Err(receiver_error(effect_handler, error)),
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

fn observed_time_unix_nano() -> Result<u64, std::time::SystemTimeError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().try_into().unwrap_or(u64::MAX))
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

#[cfg(test)]
mod tests {
    use super::DatabaseReceiver;
    use crate::receivers::database::{
        ColumnMetadata, CompiledQuery, DatabaseSystem, DriverAdapter, ErrorPolicy, OutputConfig,
        PollingConfig, QueryResult,
    };
    use async_trait::async_trait;
    use std::convert::Infallible;
    use std::time::Duration;

    struct FakeAdapter;

    #[async_trait(?Send)]
    impl DriverAdapter for FakeAdapter {
        type Error = Infallible;

        fn system(&self) -> DatabaseSystem {
            DatabaseSystem::Oracle
        }

        async fn validate_query(
            &mut self,
            _query: &CompiledQuery,
        ) -> Result<Vec<ColumnMetadata>, Self::Error> {
            Ok(Vec::new())
        }

        async fn execute(&mut self, _query: &CompiledQuery) -> Result<QueryResult, Self::Error> {
            Ok(QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                normalized_bytes: 0,
            })
        }
    }

    /// Scenario: A database-neutral adapter is connected to the shared receiver core.
    /// Guarantees: A requested poll executes successfully through the adapter contract.
    #[tokio::test]
    async fn poll_once_executes_adapter() {
        let query = CompiledQuery::compile(
            "SELECT 1".to_owned(),
            PollingConfig {
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(5),
                fetch_size: 1,
                max_rows_per_poll: 1,
            },
            OutputConfig::default(),
            ErrorPolicy::default(),
        )
        .expect("query should compile");
        let mut receiver = DatabaseReceiver::new(
            FakeAdapter,
            query,
            "test-source".to_owned(),
            "test-query".to_owned(),
        );

        let result = receiver.poll_once().await.expect("poll should succeed");
        assert!(result.rows.is_empty());
    }
}
