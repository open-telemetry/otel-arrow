// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio tracing subscriber initialization.
//!
//! This module handles the setup of the global and per-thread tokio
//! tracing subscriber. The tracing subscriber determines how log and
//! trace events are captured and routed.

use crate::event::{LogEvent, ObservedEventReporter};
use crate::log_filter::RuntimeLogFilter;
use crate::self_tracing::{ConsoleWriter, LogContextFn, LogRecord, StackLogRecord};
use otel_arrow_dfe_config::settings::telemetry::logs::LogLevel;
use std::cell::RefCell;
use std::sync::OnceLock;
use std::time::SystemTime;
use tracing::{Dispatch, Event, Subscriber};
#[cfg(test)]
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::{Context, Layer as TracingLayer};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Registry, layer::SubscriberExt};

/// Combined tracing configuration for a thread.
///
/// This struct bundles the provider setup with the shared runtime filter, allowing
/// the `InternalTelemetrySystem` to control all tracing configuration.
/// Future enhancements may include per-thread log level overrides.
#[derive(Clone)]
pub struct TracingSetup {
    /// The provider mode configuration.
    pub provider: ProviderSetup,
    /// Context function.
    pub context_fn: LogContextFn,
    /// Shared runtime filter.
    log_filter: RuntimeLogFilter,
}

impl TracingSetup {
    /// Creates a standalone tracing setup with no externally retained update handle.
    ///
    /// Use [`Self::with_log_filter`] to attach a shared runtime filter managed by
    /// the internal telemetry system.
    #[must_use]
    pub fn new(provider: ProviderSetup, log_level: LogLevel, context_fn: LogContextFn) -> Self {
        let (log_filter, _handle) = RuntimeLogFilter::new_configured(&log_level);
        Self::from_log_filter(provider, log_filter, context_fn)
    }

    pub(crate) fn from_log_filter(
        provider: ProviderSetup,
        log_filter: RuntimeLogFilter,
        context_fn: LogContextFn,
    ) -> Self {
        Self {
            provider,
            context_fn,
            log_filter,
        }
    }

    /// Replaces the setup's private filter with a shared runtime filter.
    #[must_use]
    pub fn with_log_filter(mut self, log_filter: RuntimeLogFilter) -> Self {
        self.log_filter = log_filter;
        self
    }

    /// Initialize this setup as the global tracing subscriber.
    pub fn try_init_global(&self) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
        self.provider
            .try_init_global(self.context_fn, &self.log_filter)
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    pub fn with_subscriber<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.provider
            .with_subscriber(self.context_fn, &self.log_filter, f)
    }

    #[cfg(test)]
    pub(crate) fn with_subscriber_ignoring_env<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let log_level = self.log_filter.effective_level();
        self.provider
            .with_subscriber_ignoring_env(&log_level, self.context_fn, f)
    }
}

/// Provider configuration for setting up a tracing subscriber.
#[derive(Clone)]
pub enum ProviderSetup {
    /// Logs are silently dropped.
    Noop,

    /// Synchronous console logging via `StructuredLoggingLayer`.
    ConsoleDirect,

    /// Asynchronous console logging via an observed event reporter which
    /// is either the admin component ("console_async") or an internal telemetry
    /// pipeline engine ("its").
    InternalAsync {
        /// Reporter to send log events through.
        reporter: ObservedEventReporter,
    },
}

/// The capture and delivery path paired with one tracing dispatch.
///
/// Normal events and statefully delivered diagnostics both use this type, so
/// delayed delivery changes scheduling, not record construction or routing.
#[derive(Clone)]
struct LogPipeline {
    writer: Option<ConsoleWriter>,
    reporter: Option<ObservedEventReporter>,
    context_fn: LogContextFn,
}

impl LogPipeline {
    fn capture(&self, event: &Event<'_>) -> LogRecord {
        LogRecord::new(event, (self.context_fn)())
    }

    fn deliver(&self, time: SystemTime, record: LogRecord) {
        if let Some(writer) = self.writer {
            writer.print_log_record(time, &record.as_view(), |w| {
                w.format_entity_suffix_without_registry(&record.context);
            });
        }
        if let Some(reporter) = &self.reporter {
            reporter.log(LogEvent { time, record });
        }
    }

    fn capture_and_deliver(&self, event: &Event<'_>) {
        self.deliver(SystemTime::now(), self.capture(event));
    }
}

thread_local! {
    /// Thread-scoped capture/delivery path installed alongside its `Dispatch`.
    static CURRENT_LOG_PIPELINE: RefCell<Option<LogPipeline>> = const { RefCell::new(None) };
}

/// Process-wide capture/delivery path installed with the global dispatch.
static GLOBAL_LOG_PIPELINE: OnceLock<LogPipeline> = OnceLock::new();

fn current_log_pipeline() -> Option<LogPipeline> {
    CURRENT_LOG_PIPELINE
        .with(|cell| cell.borrow().clone())
        .or_else(|| GLOBAL_LOG_PIPELINE.get().cloned())
}

/// Applies the active tracing filter and entity context to a record constructed
/// directly at its ordinary callsite.
#[doc(hidden)]
#[must_use]
pub fn capture_current_record(record: StackLogRecord) -> Option<LogRecord> {
    let enabled = tracing::dispatcher::get_default(|dispatch| dispatch.enabled(record.metadata()));
    if !enabled {
        return None;
    }
    let Some(pipeline) = current_log_pipeline() else {
        crate::raw_error!(
            "diagnostic.capture.missing_pipeline",
            event_name = record.metadata().name()
        );
        return None;
    };
    Some(record.into_record((pipeline.context_fn)()))
}

/// Delivers an already accepted [`LogRecord`] through the same sink as an
/// ordinary event under the active tracing setup, without re-filtering it.
#[doc(hidden)]
pub fn deliver_current_record(time: SystemTime, record: LogRecord) {
    let Some(pipeline) = current_log_pipeline() else {
        crate::raw_error!(
            "diagnostic.delivery.missing_pipeline",
            event_name = record.callsite().name()
        );
        return;
    };
    pipeline.deliver(time, record);
}

/// Runs `f` with `pipeline` paired with the thread-scoped tracing dispatch,
/// restoring the previous path afterward even on panic/unwind.
fn scoped_log_pipeline<F, R>(pipeline: LogPipeline, f: F) -> R
where
    F: FnOnce() -> R,
{
    struct RestoreGuard(Option<LogPipeline>);
    impl Drop for RestoreGuard {
        fn drop(&mut self) {
            CURRENT_LOG_PIPELINE.with(|cell| *cell.borrow_mut() = self.0.take());
        }
    }

    let previous = CURRENT_LOG_PIPELINE.with(|cell| cell.replace(Some(pipeline)));
    let _guard = RestoreGuard(previous);
    f()
}

impl ProviderSetup {
    fn log_pipeline(&self, context_fn: LogContextFn) -> LogPipeline {
        match self {
            ProviderSetup::Noop => LogPipeline {
                writer: None,
                reporter: None,
                context_fn,
            },
            ProviderSetup::ConsoleDirect => LogPipeline {
                writer: Some(ConsoleWriter::color()),
                reporter: None,
                context_fn,
            },
            ProviderSetup::InternalAsync { reporter } => LogPipeline {
                writer: None,
                reporter: Some(reporter.clone()),
                context_fn,
            },
        }
    }

    fn build_dispatch_with_filter(
        &self,
        filter: &RuntimeLogFilter,
        context_fn: LogContextFn,
    ) -> Dispatch {
        match self {
            ProviderSetup::Noop => Dispatch::new(tracing::subscriber::NoSubscriber::new()),
            ProviderSetup::ConsoleDirect | ProviderSetup::InternalAsync { .. } => {
                let layer = StructuredLoggingLayer::new(self.log_pipeline(context_fn));
                Dispatch::new(Registry::default().with(filter.layer()).with(layer))
            }
        }
    }

    /// Build a `Dispatch` for this provider setup with the given log level.
    fn build_dispatch(&self, context_fn: LogContextFn, filter: &RuntimeLogFilter) -> Dispatch {
        self.build_dispatch_with_filter(filter, context_fn)
    }

    /// Initialize this setup as the global tracing subscriber.
    pub fn try_init_global(
        &self,
        context_fn: LogContextFn,
        filter: &RuntimeLogFilter,
    ) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
        let dispatch = self.build_dispatch(context_fn, filter);
        tracing::dispatcher::set_global_default(dispatch)?;
        let _ = GLOBAL_LOG_PIPELINE.set(self.log_pipeline(context_fn));
        Ok(())
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    pub fn with_subscriber<F, R>(
        &self,
        context_fn: LogContextFn,
        filter: &RuntimeLogFilter,
        f: F,
    ) -> R
    where
        F: FnOnce() -> R,
    {
        let dispatch = self.build_dispatch(context_fn, filter);
        scoped_log_pipeline(self.log_pipeline(context_fn), || {
            tracing::dispatcher::with_default(&dispatch, f)
        })
    }

    #[cfg(test)]
    fn with_subscriber_ignoring_env<F, R>(
        &self,
        log_level: &LogLevel,
        context_fn: LogContextFn,
        f: F,
    ) -> R
    where
        F: FnOnce() -> R,
    {
        let filter =
            RuntimeLogFilter::from_filter(log_level.clone(), EnvFilter::new(log_level.as_str()));
        let dispatch = self.build_dispatch_with_filter(&filter, context_fn);
        scoped_log_pipeline(self.log_pipeline(context_fn), || {
            tracing::dispatcher::with_default(&dispatch, f)
        })
    }
}

/// A tracing layer that emits a structured log record to either console or an async sink.
pub struct StructuredLoggingLayer {
    pipeline: LogPipeline,
}

impl StructuredLoggingLayer {
    /// Create a new structured logging layer.
    #[must_use]
    fn new(pipeline: LogPipeline) -> Self {
        Self { pipeline }
    }
}

impl<S> TracingLayer<S> for StructuredLoggingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        self.pipeline.capture_and_deliver(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{DiagnosticErrorKind, DiagnosticTracker};
    use crate::event::ObservedEvent;
    use crate::log_filter::{RuntimeLogFilter, create_env_filter};
    use crate::self_tracing::LogContext;
    use crate::testing::EmptyAttributes;
    use crate::{otel_debug, otel_error, otel_info, otel_warn};
    use otel_arrow_dfe_config::observed_state::SendPolicy;
    use std::time::Instant;

    thread_local! {
        static TEST_LOG_CONTEXT: RefCell<LogContext> = const { RefCell::new(LogContext::new_const()) };
    }

    fn test_log_context() -> LogContext {
        TEST_LOG_CONTEXT.with(|context| context.borrow().clone())
    }

    fn test_reporter() -> (ObservedEventReporter, flume::Receiver<ObservedEvent>) {
        let (tx, rx) = flume::bounded(16);
        let reporter = ObservedEventReporter::new(SendPolicy::default(), tx);
        (reporter, rx)
    }

    fn noop_provider() -> ProviderSetup {
        ProviderSetup::Noop
    }

    fn console_direct_provider() -> ProviderSetup {
        ProviderSetup::ConsoleDirect
    }

    fn internal_async_provider(reporter: ObservedEventReporter) -> ProviderSetup {
        ProviderSetup::InternalAsync { reporter }
    }

    fn test_setup(p: ProviderSetup, l: LogLevel) -> TracingSetup {
        TracingSetup::new(p, l, LogContext::new)
    }

    fn level(s: &str) -> LogLevel {
        serde_yaml::from_str(&format!("\"{s}\"")).unwrap()
    }

    fn all_simple_levels() -> Vec<LogLevel> {
        vec![
            level("off"),
            level("debug"),
            level("info"),
            level("warn"),
            level("error"),
        ]
    }

    /// Scenario: each supported simple log level is used to construct an environment filter.
    /// Guarantees: filter construction accepts every configured level without panicking.
    #[test]
    fn create_env_filter_parses_for_all_levels() {
        crate::with_cleared_rust_log(|| {
            for l in all_simple_levels() {
                let _ = create_env_filter(&l);
            }
        });
    }

    /// Scenario: one info callsite emits through an async provider across warn and info updates.
    /// Guarantees: an installed tracing setup changes live without rebuilding its subscriber.
    #[test]
    fn internal_async_provider_applies_runtime_level_updates() {
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let (filter, handle) = RuntimeLogFilter::new(Some(&level("warn")));
            let setup = test_setup(internal_async_provider(reporter), level("warn"))
                .with_log_filter(filter);

            setup.with_subscriber(|| {
                let emit_info = || otel_info!("runtime.level.info");

                emit_info();
                assert!(receiver.try_recv().is_err());

                handle.apply(Some(&level("info")));
                emit_info();
                assert!(matches!(receiver.try_recv(), Ok(ObservedEvent::Log(_))));

                handle.apply(Some(&level("warn")));
                emit_info();
                assert!(receiver.try_recv().is_err());
            });
        });
    }

    /// Scenario: an ordinary warning and a statefully delivered diagnostic
    /// are captured under the same async tracing setup.
    /// Guarantees: both records inherit the active pipeline/node entity
    /// context from the setup's canonical capture path.
    #[test]
    fn diagnostic_capture_uses_ordinary_log_context() {
        crate::with_cleared_rust_log(|| {
            let registry = crate::registry::TelemetryRegistryHandle::new();
            let entity = registry.register_entity(EmptyAttributes());
            TEST_LOG_CONTEXT.with(|context| {
                *context.borrow_mut() = LogContext::from_buf([entity]);
            });

            let (reporter, receiver) = test_reporter();
            let setup = TracingSetup::new(
                internal_async_provider(reporter),
                level("warn"),
                test_log_context,
            );
            setup.with_subscriber(|| {
                otel_warn!("test.ordinary.warning", message = "ordinary");

                let mut tracker = DiagnosticTracker::default();
                let report = tracker
                    .failure(Instant::now(), DiagnosticErrorKind::Transport, || {
                        capture_current_record(crate::__log_record_impl!(
                            crate::Level::WARN,
                            "test.diagnostic.warning",
                            message = "diagnostic"
                        ))
                    })
                    .expect("enabled first failure should produce a report");
                crate::otel_diagnostic_report!(
                    report: &report,
                    diagnostic_kind = "first_failure"
                );
            });

            for expected_name in ["test.ordinary.warning", "test.diagnostic.warning"] {
                let ObservedEvent::Log(log) = receiver.try_recv().expect("log should be delivered")
                else {
                    panic!("expected log event");
                };
                assert_eq!(log.record.callsite().name(), expected_name);
                assert_eq!(log.record.context.as_slice(), &[entity]);
            }
        });
    }

    /// Scenario: a diagnostic callsite is initially disabled, then captured
    /// after a live filter update, and the filter is disabled again before
    /// delayed delivery.
    /// Guarantees: disabled occurrences are counted but not retained, while
    /// an accepted record is delivered later without re-evaluating its filter.
    #[test]
    fn diagnostic_delivery_preserves_capture_filter_decision() {
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let (filter, handle) = RuntimeLogFilter::new(Some(&level("error")));
            let setup = test_setup(internal_async_provider(reporter), level("error"))
                .with_log_filter(filter);

            setup.with_subscriber(|| {
                let start = Instant::now();
                let mut tracker = DiagnosticTracker::default();
                assert!(
                    tracker
                        .failure(start, DiagnosticErrorKind::Transport, || {
                            capture_current_record(crate::__log_record_impl!(
                                crate::Level::WARN,
                                "test.diagnostic.filtered",
                                message = "disabled"
                            ))
                        })
                        .is_none()
                );
                assert!(receiver.try_recv().is_err());

                handle.apply(Some(&level("warn")));
                let report = tracker
                    .failure(
                        start + std::time::Duration::from_secs(1),
                        DiagnosticErrorKind::Transport,
                        || {
                            capture_current_record(crate::__log_record_impl!(
                                crate::Level::WARN,
                                "test.diagnostic.filtered",
                                message = "accepted"
                            ))
                        },
                    )
                    .expect("enabled occurrence should produce the first report");
                assert_eq!(report.total.failures, 2);
                assert_eq!(report.total.suppressed, 1);

                handle.apply(Some(&level("error")));
                crate::otel_diagnostic_report!(
                    report: &report,
                    diagnostic_kind = "first_failure"
                );
            });

            let ObservedEvent::Log(log) = receiver
                .try_recv()
                .expect("accepted record should be delivered")
            else {
                panic!("expected log event");
            };
            assert_eq!(log.record.callsite().name(), "test.diagnostic.filtered");
            assert!(receiver.try_recv().is_err());
        });
    }

    /// Scenario: an info event is emitted through the no-op provider.
    /// Guarantees: the no-op subscriber accepts the event without failing.
    #[test]
    fn noop_provider_runs() {
        crate::with_cleared_rust_log(|| {
            let setup = test_setup(noop_provider(), level("info"));
            setup.with_subscriber_ignoring_env(|| {
                otel_info!("log_dropped");
            });
        });
    }

    /// Scenario: every log severity is emitted under each supported no-op provider level.
    /// Guarantees: the no-op provider remains usable for every level and event severity.
    #[test]
    fn noop_provider_all_levels() {
        crate::with_cleared_rust_log(|| {
            for l in all_simple_levels() {
                let setup = test_setup(noop_provider(), l);
                setup.with_subscriber_ignoring_env(|| {
                    otel_debug!("debug", "debug message");
                    otel_info!("info");
                    otel_warn!("warn");
                    otel_error!("error");
                });
            }
        });
    }

    /// Scenario: an info event is emitted through the direct console provider.
    /// Guarantees: direct console subscriber setup and event handling complete successfully.
    #[test]
    fn console_direct_provider_runs() {
        crate::with_cleared_rust_log(|| {
            let setup = test_setup(console_direct_provider(), level("info"));
            setup.with_subscriber_ignoring_env(|| {
                otel_info!("console_log");
            });
        });
    }

    /// Scenario: every log severity is emitted under each direct console provider level.
    /// Guarantees: direct console logging remains usable across all supported levels.
    #[test]
    fn console_direct_all_levels() {
        crate::with_cleared_rust_log(|| {
            for l in all_simple_levels() {
                let setup = test_setup(console_direct_provider(), l);
                setup.with_subscriber_ignoring_env(|| {
                    otel_debug!("debug", "debug message");
                    otel_info!("info");
                    otel_warn!("warn");
                    otel_error!("error");
                });
            }
        });
    }

    /// Scenario: an info event is emitted through the asynchronous internal provider.
    /// Guarantees: the provider forwards the event to its observed-event channel as a log.
    #[test]
    fn console_async_provider_sends_logs() {
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("info"));

            setup.with_subscriber_ignoring_env(|| {
                otel_info!("async_log");
            });

            // Verify the log was sent through the channel
            let event = receiver.try_recv().expect("should receive log event");
            assert!(
                matches!(event, ObservedEvent::Log(_)),
                "event should be a log"
            );
        });
    }

    /// Scenario: four event severities are emitted at each asynchronous provider level.
    /// Guarantees: the channel receives exactly the number of events allowed by each level.
    #[test]
    fn console_async_all_levels() {
        crate::with_cleared_rust_log(|| {
            for l in all_simple_levels() {
                let (reporter, receiver) = test_reporter();
                let setup = test_setup(internal_async_provider(reporter), l.clone());
                setup.with_subscriber_ignoring_env(|| {
                    otel_debug!("debug", "debug message");
                    otel_info!("info");
                    otel_warn!("warn");
                    otel_error!("error");
                });
                drop(setup);

                let cnt = receiver.into_iter().count();
                let expect = match l.as_str() {
                    "off" => 0,
                    "debug" => 4,
                    "info" => 3,
                    "warn" => 2,
                    "error" => 1,
                    _ => unreachable!(),
                };
                assert_eq!(cnt, expect);
            }
        });
    }

    /// Scenario: a debug event is emitted while the asynchronous provider is set to info.
    /// Guarantees: debug events are excluded from the provider's output channel.
    #[test]
    fn log_level_filters_debug() {
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("info"));

            setup.with_subscriber_ignoring_env(|| {
                otel_debug!("filtered", "debug message filtered out");
            });

            assert!(
                receiver.try_recv().is_err(),
                "debug log should not be received at Info level"
            );
        });
    }

    /// Scenario: debug, info, and warn events are emitted at the warn level.
    /// Guarantees: only the warn event reaches the asynchronous provider's channel.
    #[test]
    fn log_level_warn_filters_lower() {
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("warn"));

            setup.with_subscriber_ignoring_env(|| {
                otel_debug!("filtered", "debug message filtered out");
                otel_info!("filtered");
                otel_warn!("not_filtered");
            });

            // Should only receive the warn
            let event = receiver.try_recv().expect("should receive warn");
            assert!(matches!(event, ObservedEvent::Log(_)));
            assert!(receiver.try_recv().is_err(), "should only have one event");
        });
    }

    /// Scenario: debug through error events are emitted at the error level.
    /// Guarantees: only the error event reaches the asynchronous provider's channel.
    #[test]
    fn log_level_error_filters_lower() {
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("error"));

            setup.with_subscriber_ignoring_env(|| {
                otel_debug!("filtered", "debug message filtered out");
                otel_info!("filtered");
                otel_warn!("filtered");
                otel_error!("not_filtered");
            });

            let event = receiver.try_recv().expect("should receive error");
            assert!(matches!(event, ObservedEvent::Log(_)));
            assert!(receiver.try_recv().is_err(), "should only have one event");
        });
    }

    /// Scenario: events of every severity are emitted while logging is off.
    /// Guarantees: the asynchronous provider emits no events at the off level.
    #[test]
    fn log_level_off_filters_all() {
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("off"));

            setup.with_subscriber_ignoring_env(|| {
                otel_debug!("filtered", "debug message filtered out");
                otel_info!("filtered");
                otel_warn!("filtered");
                otel_error!("filtered");
            });

            assert!(receiver.try_recv().is_err(), "all logs should be filtered");
        });
    }

    /// Scenario: events of every severity are emitted at the debug level.
    /// Guarantees: all four events reach the asynchronous provider's channel.
    #[test]
    fn log_level_debug_allows_all() {
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("debug"));

            setup.with_subscriber_ignoring_env(|| {
                otel_debug!("d", "debug message");
                otel_info!("i");
                otel_warn!("w");
                otel_error!("e");
            });

            // Should receive all 4
            for _ in 0..4 {
                let _ = receiver.try_recv().expect("should receive log");
            }
        });
    }

    /// Scenario: oversized structured attributes overflow the inline log encoding buffer.
    /// Guarantees: the dropped-attribute count survives ITS encoding and OTLP parsing.
    #[test]
    fn dropped_attributes_count_propagates() {
        // Regression test: when too many attributes are passed to overflow
        // the inline encoding buffer, the visitor's dropped_attributes_count
        // must be preserved end-to-end through the ITS encode path
        // (encode_export_logs_request) and parsed back via the same
        // RawLogsData view used by the console exporter.
        //
        // Historically, a partial body write left an unpatched length
        // placeholder + trailing garbage bytes in the inline buffer, which
        // corrupted subsequent fields appended by encode_log_record (notably
        // dropped_attributes_count itself). encode_body_string is now wrapped
        // in try_encode to roll back partial bytes on overflow.
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("info"));

            // Use enough long-string attributes to overflow any reasonable
            // LOG_ARGUMENTS_ENCODE_INLINE (well above 256 bytes worth of payload).
            setup.with_subscriber_ignoring_env(|| {
                otel_info!(
                    "overflow.test",
                    a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    b = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    c = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                    d = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
                    e = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                    f = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                    g = "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
                    message = "Body that itself is fairly long and may not fit alongside the attributes above"
                );
            });

            let event = receiver.try_recv().expect("should receive log");
            let log_event = match event {
                ObservedEvent::Log(le) => le,
                _ => panic!("expected log"),
            };
            let visitor_dropped = log_event.record.dropped_attributes_count;
            assert!(
                visitor_dropped > 0,
                "expected visitor to drop attrs, got {visitor_dropped}"
            );

            // Encode through the full ITS path and parse via RawLogsData
            // (the same path used by internal_telemetry_receiver -> console
            // exporter).
            use crate::self_tracing::{ScopeToBytesMap, encode_export_logs_request};
            use bytes::Bytes;
            use otel_arrow_dfe_pdata::otlp::ProtoBuffer;
            use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogsData;
            use otel_arrow_dfe_pdata_views::views::logs::{
                LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
            };

            let resource_bytes = Bytes::new();
            let registry = crate::registry::TelemetryRegistryHandle::new();
            let mut scope_cache = ScopeToBytesMap::new(registry);
            let mut buf = ProtoBuffer::default();
            encode_export_logs_request(
                &mut buf,
                &mut [log_event],
                &resource_bytes,
                &mut scope_cache,
            )
            .unwrap();
            let bytes_vec = buf.into_bytes();

            let raw = RawLogsData::new(bytes_vec.as_ref());
            let mut parsed_dropped = None;
            for rl in raw.resources() {
                for sl in rl.scopes() {
                    for lr in sl.log_records() {
                        parsed_dropped = Some(lr.dropped_attributes_count());
                    }
                }
            }
            assert_eq!(
                parsed_dropped,
                Some(visitor_dropped as u32),
                "dropped_attributes_count must round-trip through encode/parse"
            );
        });
    }

    /// Scenario: an asynchronous log event contains string and numeric fields.
    /// Guarantees: the channel event preserves the event name and both structured fields.
    #[test]
    fn console_async_layer_with_fields() {
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("info"));

            setup.with_subscriber_ignoring_env(|| {
                otel_info!("structured", key = "value", number = 42);
            });

            let event = receiver.try_recv().expect("should receive log");
            assert!(matches!(event, ObservedEvent::Log(_)));
            let text = event.to_string();
            assert!(text.contains("key=value"), "text is {}", text);
            assert!(text.contains("number=42"), "text is {}", text);
            assert!(text.contains("structured"), "text is {}", text);
        });
    }

    /// Scenario: each provider variant installs a scoped subscriber and emits an event.
    /// Guarantees: no-op, direct console, and asynchronous providers all support scoped use.
    #[test]
    fn provider_setup_with_subscriber_all_variants() {
        crate::with_cleared_rust_log(|| {
            let info = level("info");
            noop_provider().with_subscriber_ignoring_env(&info, LogContext::new, || {
                otel_info!("noop");
            });

            console_direct_provider().with_subscriber_ignoring_env(&info, LogContext::new, || {
                otel_info!("console_direct");
            });

            let (reporter, _rx) = test_reporter();
            internal_async_provider(reporter).with_subscriber_ignoring_env(
                &info,
                LogContext::new,
                || {
                    otel_info!("console_async");
                },
            );
        });
    }

    /// Scenario: debug through error events are emitted through ITS at the warn level.
    /// Guarantees: ITS forwards exactly the warn and error events.
    #[test]
    fn its_provider_filters_correctly() {
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("warn"));

            setup.with_subscriber_ignoring_env(|| {
                otel_debug!("filtered", "debug message filtered out");
                otel_info!("filtered");
                otel_warn!("not_filtered");
                otel_error!("not_filtered");
            });
            drop(setup);

            assert_eq!(receiver.into_iter().count(), 2);
        });
    }

    /// Scenario: one asynchronous subscriber is temporarily nested inside another.
    /// Guarantees: inner events remain isolated while outer events return to the outer channel.
    #[test]
    fn nested_with_subscriber() {
        crate::with_cleared_rust_log(|| {
            let (reporter1, receiver1) = test_reporter();
            let (reporter2, receiver2) = test_reporter();

            let setup1 = test_setup(internal_async_provider(reporter1), level("info"));
            let setup2 = test_setup(internal_async_provider(reporter2), level("info"));

            let result = setup1.with_subscriber_ignoring_env(|| {
                otel_info!("outer");
                setup2.with_subscriber_ignoring_env(|| {
                    otel_info!("inner");
                });
                otel_info!("outer_again");
                100
            });

            assert_eq!(result, 100);

            // Outer should receive 2, inner should receive 1 and no more.
            assert!(receiver1.try_recv().is_ok());
            assert!(receiver2.try_recv().is_ok());
            assert!(receiver1.try_recv().is_ok());

            assert!(receiver1.try_recv().is_err());
            assert!(receiver2.try_recv().is_err());
        });
    }
}
