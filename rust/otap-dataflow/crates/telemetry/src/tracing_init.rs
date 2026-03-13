// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio tracing subscriber initialization.
//!
//! This module handles the setup of the global and per-thread tokio
//! tracing subscriber. The tracing subscriber determines how log and
//! trace events are captured and routed.

use crate::event::{LogEvent, ObservedEventReporter};
use crate::log_tap::InternalLogTapReporter;
use crate::self_tracing::{ConsoleWriter, LogContextFn, LogRecord};
use otap_df_config::settings::telemetry::logs::LogLevel;
use smallvec::SmallVec;
use std::time::SystemTime;
use tracing::{Dispatch, Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer as TracingLayer};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

/// Creates an `EnvFilter` for the given log level.
///
/// If the `RUST_LOG` environment variable is set, it takes precedence.
/// Otherwise, the level's [`RUST_LOG`-style directive string][env-filter] is
/// passed directly to [`EnvFilter`].
///
/// [env-filter]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives
#[must_use]
pub fn create_env_filter(level: &LogLevel) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.as_str()))
}

/// Combined tracing configuration for a thread.
///
/// This struct bundles the provider setup with the log level, allowing
/// the `InternalTelemetrySystem` to control all tracing configuration.
/// Future enhancements may include per-thread log level overrides.
#[derive(Clone)]
pub struct TracingSetup {
    /// The provider mode configuration.
    pub provider: ProviderSetup,
    /// The log level for filtering.
    pub log_level: LogLevel,
    /// Context function.
    pub context_fn: LogContextFn,
}

impl TracingSetup {
    /// Create a new tracing setup.
    #[must_use]
    pub fn new(provider: ProviderSetup, log_level: LogLevel, context_fn: LogContextFn) -> Self {
        Self {
            provider,
            log_level,
            context_fn,
        }
    }

    /// Initialize this setup as the global tracing subscriber.
    pub fn try_init_global(&self) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
        self.provider
            .try_init_global(&self.log_level, self.context_fn)
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    pub fn with_subscriber<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.provider
            .with_subscriber(&self.log_level, self.context_fn, f)
    }

    #[cfg(test)]
    pub(crate) fn with_subscriber_ignoring_env<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.provider
            .with_subscriber_ignoring_env(&self.log_level, self.context_fn, f)
    }
}

/// Provider configuration for setting up a tracing subscriber.
#[derive(Clone)]
pub enum ProviderSetup {
    /// No external logging sink.
    ///
    /// Events may still be forwarded to the internal log tap when configured.
    Noop {
        /// Optional internal log tap.
        tap: Option<InternalLogTapReporter>,
    },

    /// Synchronous console logging via `StructuredLoggingLayer`.
    ConsoleDirect {
        /// Optional internal log tap.
        tap: Option<InternalLogTapReporter>,
    },

    /// Asynchronous console logging via an observed event reporter which
    /// is either the admin component ("console_async") or an internal telemetry
    /// pipeline engine ("its").
    InternalAsync {
        /// Reporter to send log events through.
        reporter: ObservedEventReporter,
        /// Optional internal log tap.
        tap: Option<InternalLogTapReporter>,
    },
}

impl ProviderSetup {
    fn build_dispatch_with_filter(&self, filter: EnvFilter, context_fn: LogContextFn) -> Dispatch {
        match self {
            ProviderSetup::Noop { tap: None } => {
                Dispatch::new(tracing::subscriber::NoSubscriber::new())
            }

            ProviderSetup::Noop { tap } => {
                let layer =
                    StructuredLoggingLayer::new(None, sinks(None, tap.as_ref()), context_fn);
                Dispatch::new(Registry::default().with(filter).with(layer))
            }

            ProviderSetup::ConsoleDirect { tap } => {
                let layer = StructuredLoggingLayer::new(
                    Some(ConsoleWriter::color()),
                    sinks(None, tap.as_ref()),
                    context_fn,
                );
                Dispatch::new(Registry::default().with(filter).with(layer))
            }

            ProviderSetup::InternalAsync { reporter, tap } => {
                let layer = StructuredLoggingLayer::new(
                    None,
                    sinks(Some(reporter), tap.as_ref()),
                    context_fn,
                );
                Dispatch::new(Registry::default().with(filter).with(layer))
            }
        }
    }

    /// Build a `Dispatch` for this provider setup with the given log level.
    fn build_dispatch(&self, log_level: &LogLevel, context_fn: LogContextFn) -> Dispatch {
        self.build_dispatch_with_filter(create_env_filter(log_level), context_fn)
    }

    /// Initialize this setup as the global tracing subscriber.
    pub fn try_init_global(
        &self,
        log_level: &LogLevel,
        context_fn: LogContextFn,
    ) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
        let dispatch = self.build_dispatch(log_level, context_fn);
        tracing::dispatcher::set_global_default(dispatch)
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    pub fn with_subscriber<F, R>(&self, log_level: &LogLevel, context_fn: LogContextFn, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let dispatch = self.build_dispatch(log_level, context_fn);
        tracing::dispatcher::with_default(&dispatch, f)
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
        let dispatch =
            self.build_dispatch_with_filter(EnvFilter::new(log_level.as_str()), context_fn);
        tracing::dispatcher::with_default(&dispatch, f)
    }
}

#[derive(Clone)]
enum AsyncLogSink {
    Observed(ObservedEventReporter),
    Tap(InternalLogTapReporter),
}

impl AsyncLogSink {
    fn log(&self, event: LogEvent) {
        match self {
            Self::Observed(reporter) => reporter.log(event),
            Self::Tap(reporter) => reporter.log(event),
        }
    }
}

fn sinks(
    reporter: Option<&ObservedEventReporter>,
    tap: Option<&InternalLogTapReporter>,
) -> SmallVec<[AsyncLogSink; 2]> {
    let mut sinks = SmallVec::new();
    if let Some(reporter) = reporter {
        sinks.push(AsyncLogSink::Observed(reporter.clone()));
    }
    if let Some(tap) = tap {
        sinks.push(AsyncLogSink::Tap(tap.clone()));
    }
    sinks
}

/// A tracing layer that emits a single structured log record and fans it out.
pub struct StructuredLoggingLayer {
    writer: Option<ConsoleWriter>,
    sinks: SmallVec<[AsyncLogSink; 2]>,
    context_fn: LogContextFn,
}

impl StructuredLoggingLayer {
    /// Create a new structured logging layer.
    #[must_use]
    fn new(
        writer: Option<ConsoleWriter>,
        sinks: SmallVec<[AsyncLogSink; 2]>,
        context_fn: LogContextFn,
    ) -> Self {
        Self {
            writer,
            sinks,
            context_fn,
        }
    }
}

impl<S> TracingLayer<S> for StructuredLoggingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let time = SystemTime::now();
        let context = (self.context_fn)();
        let record = LogRecord::new(event, context);
        if let Some(writer) = self.writer {
            writer.print_log_record(time, &record, |w| {
                w.format_entity_suffix_without_registry(&record.context);
            });
        }
        if self.sinks.is_empty() {
            return;
        }
        let mut sinks = self.sinks.iter();
        let last = sinks.next_back().expect("non-empty sink list");
        for sink in sinks {
            sink.log(LogEvent {
                time,
                record: record.clone(),
            });
        }
        last.log(LogEvent { time, record });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::ObservedEvent;
    use crate::log_tap;
    use crate::self_tracing::LogContext;
    use crate::{otel_debug, otel_error, otel_info, otel_warn};
    use otap_df_config::observed_state::SendPolicy;
    use otap_df_config::settings::telemetry::logs::InternalLogTapConfig;

    fn test_reporter() -> (ObservedEventReporter, flume::Receiver<ObservedEvent>) {
        let (tx, rx) = flume::bounded(16);
        let reporter = ObservedEventReporter::new(SendPolicy::default(), tx);
        (reporter, rx)
    }

    fn noop_provider() -> ProviderSetup {
        ProviderSetup::Noop { tap: None }
    }

    fn console_direct_provider() -> ProviderSetup {
        ProviderSetup::ConsoleDirect { tap: None }
    }

    fn internal_async_provider(reporter: ObservedEventReporter) -> ProviderSetup {
        ProviderSetup::InternalAsync {
            reporter,
            tap: None,
        }
    }

    fn test_tap() -> (
        InternalLogTapReporter,
        log_tap::InternalLogTapHandle,
        log_tap::InternalLogTapRuntime,
    ) {
        let config = InternalLogTapConfig {
            enabled: true,
            ingest_channel_size: 16,
            max_entries: 16,
            max_bytes: 1024 * 1024,
        };
        log_tap::build(&config)
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

    #[test]
    fn noop_provider_runs() {
        let setup = test_setup(noop_provider(), level("info"));
        setup.with_subscriber_ignoring_env(|| {
            otel_info!("log_dropped");
        });
    }

    #[test]
    fn noop_provider_all_levels() {
        for l in all_simple_levels() {
            let setup = test_setup(noop_provider(), l);
            setup.with_subscriber_ignoring_env(|| {
                otel_debug!("debug", "debug message");
                otel_info!("info");
                otel_warn!("warn");
                otel_error!("error");
            });
        }
    }

    #[test]
    fn console_direct_provider_runs() {
        let setup = test_setup(console_direct_provider(), level("info"));
        setup.with_subscriber_ignoring_env(|| {
            otel_info!("console_log");
        });
    }

    #[test]
    fn console_direct_all_levels() {
        for l in all_simple_levels() {
            let setup = test_setup(console_direct_provider(), l);
            setup.with_subscriber_ignoring_env(|| {
                otel_debug!("debug", "debug message");
                otel_info!("info");
                otel_warn!("warn");
                otel_error!("error");
            });
        }
    }

    #[test]
    fn console_async_provider_sends_logs() {
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
    }

    #[test]
    fn console_async_all_levels() {
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
    }

    #[test]
    fn log_level_filters_debug() {
        let (reporter, receiver) = test_reporter();
        let setup = test_setup(internal_async_provider(reporter), level("info"));

        setup.with_subscriber_ignoring_env(|| {
            otel_debug!("filtered", "debug message filtered out");
        });

        assert!(
            receiver.try_recv().is_err(),
            "debug log should not be received at Info level"
        );
    }

    #[test]
    fn log_level_warn_filters_lower() {
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
    }

    #[test]
    fn log_level_error_filters_lower() {
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
    }

    #[test]
    fn log_level_off_filters_all() {
        let (reporter, receiver) = test_reporter();
        let setup = test_setup(internal_async_provider(reporter), level("off"));

        setup.with_subscriber_ignoring_env(|| {
            otel_debug!("filtered", "debug message filtered out");
            otel_info!("filtered");
            otel_warn!("filtered");
            otel_error!("filtered");
        });

        assert!(receiver.try_recv().is_err(), "all logs should be filtered");
    }

    #[test]
    fn log_level_debug_allows_all() {
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
    }

    #[test]
    fn console_async_layer_with_fields() {
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
    }

    #[test]
    fn provider_setup_with_subscriber_all_variants() {
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
    }

    #[test]
    fn its_provider_filters_correctly() {
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
    }

    #[test]
    fn nested_with_subscriber() {
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
    }

    #[test]
    fn noop_provider_can_feed_tap() {
        let (tap, handle, runtime) = test_tap();
        let setup = test_setup(ProviderSetup::Noop { tap: Some(tap) }, level("info"));

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async move {
            let cancel = tokio_util::sync::CancellationToken::new();
            let join = tokio::spawn(runtime.run(cancel.clone()));
            setup.with_subscriber_ignoring_env(|| {
                otel_info!("tapped_from_noop");
            });
            tokio::task::yield_now().await;
            cancel.cancel();
            join.await.expect("join").expect("runtime success");
        });

        let result = handle.query(log_tap::LogQuery {
            after: None,
            limit: 10,
        });
        assert_eq!(result.logs.len(), 1);
    }

    #[test]
    fn console_direct_provider_can_feed_tap() {
        let (tap, handle, runtime) = test_tap();
        let setup = test_setup(
            ProviderSetup::ConsoleDirect { tap: Some(tap) },
            level("info"),
        );

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async move {
            let cancel = tokio_util::sync::CancellationToken::new();
            let join = tokio::spawn(runtime.run(cancel.clone()));
            setup.with_subscriber_ignoring_env(|| {
                otel_info!("tapped_from_console");
            });
            tokio::task::yield_now().await;
            cancel.cancel();
            join.await.expect("join").expect("runtime success");
        });

        let result = handle.query(log_tap::LogQuery {
            after: None,
            limit: 10,
        });
        assert_eq!(result.logs.len(), 1);
    }
}
