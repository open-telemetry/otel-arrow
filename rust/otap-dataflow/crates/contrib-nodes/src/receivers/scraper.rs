// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared lifecycle for simple pull-based receivers.

use async_trait::async_trait;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error as EngineError, ReceiverErrorKind, format_error_sources};
use otap_df_engine::local::receiver as local;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::pdata::OtapPdata;
use std::error::Error as StdError;
use std::time::Duration;
use tokio::time::{self, MissedTickBehavior};

/// Identifies which lifecycle operation produced a scraper error.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ScraperPhase {
    /// The scraper was opening or validating its source.
    Start,
    /// The scraper was collecting one batch.
    Scrape,
    /// The scraper was closing its source.
    Shutdown,
}

impl ScraperPhase {
    const fn action(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Scrape => "scrape",
            Self::Shutdown => "shut down",
        }
    }
}

/// Source-specific behavior used by the shared polling lifecycle.
///
/// Implementations own source connections, queries, and data conversion. The
/// receiver owns interval scheduling, non-overlap, downstream backpressure, and
/// lifecycle control.
#[async_trait(?Send)]
pub(crate) trait Scraper {
    /// Error returned by this scraper.
    type Error: StdError + 'static;

    /// Returns a stable source name for customer-facing errors.
    fn name(&self) -> &'static str;

    /// Classifies a source-specific error for the receiver error taxonomy.
    fn classify_error(&self, phase: ScraperPhase, error: &Self::Error) -> ReceiverErrorKind;

    /// Opens and validates resources required by the scraper.
    async fn start(&mut self) -> Result<(), Self::Error>;

    /// Collects at most one batch.
    async fn scrape(&mut self) -> Result<Option<OtapPdata>, Self::Error>;

    /// Closes resources owned by the scraper.
    async fn shutdown(&mut self) -> Result<(), Self::Error>;
}

/// Minimal reusable receiver for one non-overlapping scraper.
pub(crate) struct ScraperReceiver<S> {
    scraper: S,
    collection_interval: Duration,
}

impl<S> ScraperReceiver<S> {
    /// Creates a receiver that polls `scraper` at `collection_interval`.
    pub(crate) const fn new(scraper: S, collection_interval: Duration) -> Self {
        Self {
            scraper,
            collection_interval,
        }
    }
}

#[async_trait(?Send)]
impl<S> local::Receiver<OtapPdata> for ScraperReceiver<S>
where
    S: Scraper + 'static,
{
    async fn start(
        self: Box<Self>,
        mut ctrl_chan: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let mut receiver = *self;
        receiver.start_scraper(&effect_handler).await?;

        let mut interval = time::interval(receiver.collection_interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                biased;

                ctrl = ctrl_chan.recv() => {
                    match ctrl {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            receiver.shutdown_scraper(&effect_handler).await?;
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(empty_terminal_state(deadline));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            receiver.shutdown_scraper(&effect_handler).await?;
                            return Ok(empty_terminal_state(deadline));
                        }
                        Err(error) => return Err(EngineError::ChannelRecvError(error)),
                        _ => {}
                    }
                }

                _ = interval.tick() => {
                    let pdata = receiver.scraper.scrape().await.map_err(|error| {
                        scraper_error(
                            &effect_handler,
                            receiver.scraper.name(),
                            ScraperPhase::Scrape,
                            receiver.scraper.classify_error(ScraperPhase::Scrape, &error),
                            &error,
                        )
                    })?;

                    if let Some(pdata) = pdata {
                        effect_handler.send_message_with_source_node(pdata).await?;
                    }
                }
            }
        }
    }
}

impl<S> ScraperReceiver<S>
where
    S: Scraper,
{
    async fn start_scraper(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        self.scraper.start().await.map_err(|error| {
            scraper_error(
                effect_handler,
                self.scraper.name(),
                ScraperPhase::Start,
                self.scraper.classify_error(ScraperPhase::Start, &error),
                &error,
            )
        })
    }

    async fn shutdown_scraper(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        self.scraper.shutdown().await.map_err(|error| {
            scraper_error(
                effect_handler,
                self.scraper.name(),
                ScraperPhase::Shutdown,
                self.scraper.classify_error(ScraperPhase::Shutdown, &error),
                &error,
            )
        })
    }
}

fn empty_terminal_state(deadline: std::time::Instant) -> TerminalState {
    TerminalState::new::<[otap_df_telemetry::metrics::MetricSetSnapshot; 0]>(deadline, [])
}

fn scraper_error(
    effect_handler: &local::EffectHandler<OtapPdata>,
    scraper_name: &str,
    phase: ScraperPhase,
    kind: ReceiverErrorKind,
    error: &(dyn StdError + 'static),
) -> EngineError {
    EngineError::ReceiverError {
        receiver: effect_handler.receiver_id(),
        kind,
        error: format!(
            "{scraper_name} scraper failed to {}: {error}",
            phase.action()
        ),
        source_detail: format_error_sources(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{receiver::TestRuntime, test_node};
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::time::Instant;

    #[derive(Debug, thiserror::Error)]
    enum FakeScraperError {
        #[error("scrape called before start")]
        NotStarted,
    }

    struct FakeScraper {
        started: Rc<Cell<bool>>,
        scraped: Rc<Cell<bool>>,
        stopped: Rc<Cell<bool>>,
    }

    #[async_trait(?Send)]
    impl Scraper for FakeScraper {
        type Error = FakeScraperError;

        fn name(&self) -> &'static str {
            "fake"
        }

        fn classify_error(&self, _phase: ScraperPhase, _error: &Self::Error) -> ReceiverErrorKind {
            ReceiverErrorKind::Other
        }

        async fn start(&mut self) -> Result<(), Self::Error> {
            self.started.set(true);
            Ok(())
        }

        async fn scrape(&mut self) -> Result<Option<OtapPdata>, Self::Error> {
            if !self.started.get() {
                return Err(FakeScraperError::NotStarted);
            }
            if self.scraped.replace(true) {
                return Ok(None);
            }

            let logs = LogsData {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![LogRecord::default()],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            let payload = OtlpProtoMessage::Logs(logs)
                .try_into()
                .expect("test logs should encode");
            Ok(Some(OtapPdata::new_todo_context(payload)))
        }

        async fn shutdown(&mut self) -> Result<(), Self::Error> {
            self.stopped.set(true);
            Ok(())
        }
    }

    /// Scenario: a local scraper implements the shared lifecycle and emits one batch.
    /// Guarantees: the receiver calls start before scrape, forwards data, and calls shutdown.
    #[test]
    fn scraper_receiver_runs_the_trait_lifecycle() {
        let started = Rc::new(Cell::new(false));
        let scraped = Rc::new(Cell::new(false));
        let stopped = Rc::new(Cell::new(false));
        let scraper = FakeScraper {
            started: Rc::clone(&started),
            scraped: Rc::clone(&scraped),
            stopped: Rc::clone(&stopped),
        };
        let receiver = ScraperReceiver::new(scraper, Duration::from_millis(10));
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:test_scraper",
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
                ctx.sleep(Duration::from_millis(50)).await;
                ctx.send_shutdown(Instant::now(), "fake scraper test complete")
                    .await
                    .expect("shutdown should enqueue");
            })
            .run_validation(|mut ctx| async move {
                let pdata = ctx.recv().await.expect("scraper should emit pdata");
                assert_eq!(pdata.num_items(), 1);
            });

        assert!(started.get());
        assert!(scraped.get());
        assert!(stopped.get());
    }
}
