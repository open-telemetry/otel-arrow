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

#[cfg(test)]
mod tests {
    use super::DatabaseReceiver;
    use crate::receivers::database::{
        ColumnMetadata, CompiledQuery, DatabaseSystem, DriverAdapter, OutputConfig, PollingConfig,
        QueryResult,
    };
    use async_trait::async_trait;
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
    use otel_arrow_dfe_engine::testing::{receiver::TestRuntime, test_node};
    use otel_arrow_dfe_otap::pdata::OtapPdata;
    use std::cell::Cell;
    use std::convert::Infallible;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

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

    struct DelayedAdapter {
        started: Rc<Cell<usize>>,
        completed: Rc<Cell<usize>>,
    }

    #[async_trait(?Send)]
    impl DriverAdapter for DelayedAdapter {
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
            self.started.set(self.started.get() + 1);
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.completed.set(self.completed.get() + 1);
            Ok(QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                normalized_bytes: 0,
            })
        }
    }

    fn compiled_query(interval: Duration) -> CompiledQuery {
        CompiledQuery::compile(
            "SELECT 1".to_owned(),
            PollingConfig {
                interval,
                timeout: Duration::from_secs(5),
                fetch_size: 1,
                max_rows_per_poll: 1,
            },
            OutputConfig::default(),
        )
        .expect("query should compile")
    }

    /// Scenario: A database-neutral adapter is connected to the shared receiver core.
    /// Guarantees: A requested poll executes successfully through the adapter contract.
    #[tokio::test]
    async fn poll_once_executes_adapter() {
        let query = compiled_query(Duration::from_secs(30));
        let mut receiver = DatabaseReceiver::new(FakeAdapter, query, "test-source".to_owned());

        let result = receiver.poll_once().await.expect("poll should succeed");
        assert!(result.rows.is_empty());
    }

    /// Scenario: A timer control message arrives while a database poll is awaiting completion.
    /// Guarantees: Non-terminal control traffic does not cancel or overlap the in-flight query.
    #[test]
    fn control_message_does_not_restart_in_flight_poll() {
        let started = Rc::new(Cell::new(0));
        let completed = Rc::new(Cell::new(0));
        let receiver = DatabaseReceiver::new(
            DelayedAdapter {
                started: Rc::clone(&started),
                completed: Rc::clone(&completed),
            },
            compiled_query(Duration::from_secs(30)),
            "test-source".to_owned(),
        );
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            "test-database-receiver",
        ));
        let receiver_wrapper = ReceiverWrapper::local(
            receiver,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver_wrapper)
            .run_test(|ctx| async move {
                ctx.sleep(Duration::from_millis(20)).await;
                ctx.send_timer_tick()
                    .await
                    .expect("timer tick should enqueue");
                ctx.sleep(Duration::from_millis(120)).await;
                ctx.send_shutdown(Instant::now(), "test complete")
                    .await
                    .expect("shutdown should enqueue");
            })
            .run_validation(|_| async {});

        assert_eq!(started.get(), 1);
        assert_eq!(completed.get(), 1);
    }
}
