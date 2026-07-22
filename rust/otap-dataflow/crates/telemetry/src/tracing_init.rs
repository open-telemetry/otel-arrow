// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio tracing subscriber initialization.
//!
//! This module handles the setup of the global and per-thread tokio
//! tracing subscriber. The tracing subscriber determines how log and
//! trace events are captured and routed.

use crate::event::{LogEvent, ObservedEventReporter};
use crate::eventname_filter::EventNameFilter;
use crate::self_tracing::{ConsoleWriter, LogContextFn, LogRecord};
use otap_df_config::settings::telemetry::logs::LogLevel;
use std::time::SystemTime;
use tracing::{Dispatch, Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer as TracingLayer};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

/// Creates an `EnvFilter` for the given log level.
///
/// The base filter comes from the `RUST_LOG` environment variable if set;
/// otherwise it falls back to the level's
/// [`RUST_LOG`-style directive string][env-filter]. The suppression directive
/// described below is then appended unconditionally on top of that base, so it
/// can override a conflicting user-supplied directive for the
/// `opentelemetry-prometheus[{metric_description}]` target.
///
/// In all cases the filter suppresses one specific benign per-scrape warning
/// from the `opentelemetry-prometheus` crate: the
/// `MetricValidationFailed` event carrying a `metric_description` field. The
/// Prometheus exporter flattens OpenTelemetry instrumentation scopes into a
/// single namespace keyed by metric name (scope is exposed only as the
/// `otel_scope_name` label, per the OTel/Prometheus interop spec). When two
/// scopes emit the same metric name with different descriptions, the exporter
/// keeps the first `# HELP` and logs this warning on every scrape. No data is
/// lost (each scope remains a distinct time series), so it is pure noise.
///
/// The directive is field-scoped, so it is surgical: it caps *only* the
/// description-conflict warning (which carries `metric_description`) at `ERROR`.
/// The sibling type-conflict warning (which carries `metric_type` and *does*
/// drop data) lacks that field, so it is left untouched and remains visible at
/// `WARN`, as do all other diagnostics from the crate.
/// See https://github.com/open-telemetry/otel-arrow/issues/2734 for more
/// details.
///
/// [env-filter]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives
#[must_use]
pub fn create_env_filter(level: &LogLevel) -> EnvFilter {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.as_str()));
    // Target matches `env!("CARGO_PKG_NAME")` set by opentelemetry's `otel_warn!`.
    // The `[{metric_description}]` field filter matches only the benign
    // description-conflict `MetricValidationFailed` event, leaving the
    // data-dropping type-conflict variant (field `metric_type`) visible.
    filter.add_directive(
        "opentelemetry-prometheus[{metric_description}]=error"
            .parse()
            .expect("valid tracing directive"),
    )
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
    /// EventName-based filter, composed on top of the level/target `EnvFilter`.
    /// Defaults to allow-all (no EventName filtering).
    event_filter: EventNameFilter,
}

impl TracingSetup {
    /// Create a new tracing setup.
    #[must_use]
    pub fn new(provider: ProviderSetup, log_level: LogLevel, context_fn: LogContextFn) -> Self {
        Self {
            provider,
            log_level,
            context_fn,
            event_filter: EventNameFilter::allow_all(),
        }
    }

    /// Sets the EventName filter applied on top of the level/target filter.
    #[must_use]
    pub fn with_event_filter(mut self, event_filter: EventNameFilter) -> Self {
        self.event_filter = event_filter;
        self
    }

    /// Initialize this setup as the global tracing subscriber.
    pub fn try_init_global(&self) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
        self.provider
            .try_init_global(&self.log_level, self.context_fn, &self.event_filter)
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    pub fn with_subscriber<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.provider
            .with_subscriber(&self.log_level, self.context_fn, &self.event_filter, f)
    }

    #[cfg(test)]
    pub(crate) fn with_subscriber_ignoring_env<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.provider.with_subscriber_ignoring_env(
            &self.log_level,
            self.context_fn,
            &self.event_filter,
            f,
        )
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

impl ProviderSetup {
    fn build_dispatch_with_filter(
        &self,
        filter: EnvFilter,
        context_fn: LogContextFn,
        event_filter: &EventNameFilter,
    ) -> Dispatch {
        match self {
            ProviderSetup::Noop => Dispatch::new(tracing::subscriber::NoSubscriber::new()),

            ProviderSetup::ConsoleDirect => {
                let layer =
                    StructuredLoggingLayer::new(Some(ConsoleWriter::color()), None, context_fn);
                Dispatch::new(
                    Registry::default()
                        .with(filter)
                        .with(layer.with_filter(event_filter.clone())),
                )
            }

            ProviderSetup::InternalAsync { reporter } => {
                let layer = StructuredLoggingLayer::new(None, Some(reporter.clone()), context_fn);
                Dispatch::new(
                    Registry::default()
                        .with(filter)
                        .with(layer.with_filter(event_filter.clone())),
                )
            }
        }
    }

    /// Build a `Dispatch` for this provider setup with the given log level.
    fn build_dispatch(
        &self,
        log_level: &LogLevel,
        context_fn: LogContextFn,
        event_filter: &EventNameFilter,
    ) -> Dispatch {
        self.build_dispatch_with_filter(create_env_filter(log_level), context_fn, event_filter)
    }

    /// Initialize this setup as the global tracing subscriber.
    pub fn try_init_global(
        &self,
        log_level: &LogLevel,
        context_fn: LogContextFn,
        event_filter: &EventNameFilter,
    ) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
        let dispatch = self.build_dispatch(log_level, context_fn, event_filter);
        tracing::dispatcher::set_global_default(dispatch)
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    pub fn with_subscriber<F, R>(
        &self,
        log_level: &LogLevel,
        context_fn: LogContextFn,
        event_filter: &EventNameFilter,
        f: F,
    ) -> R
    where
        F: FnOnce() -> R,
    {
        let dispatch = self.build_dispatch(log_level, context_fn, event_filter);
        tracing::dispatcher::with_default(&dispatch, f)
    }

    #[cfg(test)]
    fn with_subscriber_ignoring_env<F, R>(
        &self,
        log_level: &LogLevel,
        context_fn: LogContextFn,
        event_filter: &EventNameFilter,
        f: F,
    ) -> R
    where
        F: FnOnce() -> R,
    {
        let dispatch = self.build_dispatch_with_filter(
            EnvFilter::new(log_level.as_str()),
            context_fn,
            event_filter,
        );
        tracing::dispatcher::with_default(&dispatch, f)
    }
}

/// A tracing layer that emits a structured log record to either console or an async sink.
pub struct StructuredLoggingLayer {
    writer: Option<ConsoleWriter>,
    reporter: Option<ObservedEventReporter>,
    context_fn: LogContextFn,
}

impl StructuredLoggingLayer {
    /// Create a new structured logging layer.
    #[must_use]
    fn new(
        writer: Option<ConsoleWriter>,
        reporter: Option<ObservedEventReporter>,
        context_fn: LogContextFn,
    ) -> Self {
        Self {
            writer,
            reporter,
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
            writer.print_log_record(time, &record.as_view(), |w| {
                w.format_entity_suffix_without_registry(&record.context);
            });
        }
        if let Some(reporter) = &self.reporter {
            reporter.log(LogEvent { time, record });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::ObservedEvent;
    use crate::self_tracing::LogContext;
    use crate::{otel_debug, otel_error, otel_info, otel_warn};
    use otap_df_config::observed_state::SendPolicy;

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

    /// Counts how many events reach the subscriber after filtering.
    fn count_events_through_filter<F>(emit: F) -> usize
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct CountingLayer(Arc<AtomicUsize>);
        impl<S: Subscriber> TracingLayer<S> for CountingLayer {
            fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>) {
                let _ = self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let count = Arc::new(AtomicUsize::new(0));
        crate::with_cleared_rust_log(|| {
            let subscriber = Registry::default()
                .with(create_env_filter(&level("info")))
                .with(CountingLayer(count.clone()));
            tracing::subscriber::with_default(subscriber, emit);
        });
        count.load(Ordering::SeqCst)
    }

    #[test]
    fn create_env_filter_parses_for_all_levels() {
        // The embedded `opentelemetry-prometheus[{metric_description}]=error`
        // directive must parse for every level (otherwise `create_env_filter`
        // panics at startup via its `.expect`).
        crate::with_cleared_rust_log(|| {
            for l in all_simple_levels() {
                let _ = create_env_filter(&l);
            }
        });
    }

    #[test]
    fn prometheus_description_conflict_warning_is_suppressed() {
        // Benign description-conflict warning (field `metric_description`): the
        // field-scoped directive caps it at ERROR, so a WARN is dropped.
        let count = count_events_through_filter(|| {
            tracing::warn!(
                target: "opentelemetry-prometheus",
                metric_description = "conflict",
                "Instrument description conflict, using existing"
            );
        });
        assert_eq!(
            count, 0,
            "benign description-conflict warning should be suppressed"
        );
    }

    #[test]
    fn prometheus_type_conflict_warning_remains_visible() {
        // Data-dropping type-conflict warning (field `metric_type`, not
        // `metric_description`): the directive does not match it, so it stays
        // visible at WARN.
        let count = count_events_through_filter(|| {
            tracing::warn!(
                target: "opentelemetry-prometheus",
                metric_type = "conflict",
                "Instrument type conflict, using existing type definition"
            );
        });
        assert_eq!(
            count, 1,
            "data-dropping type-conflict warning should remain visible"
        );
    }

    #[test]
    fn noop_provider_runs() {
        crate::with_cleared_rust_log(|| {
            let setup = test_setup(noop_provider(), level("info"));
            setup.with_subscriber_ignoring_env(|| {
                otel_info!("log_dropped");
            });
        });
    }

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

    #[test]
    fn console_direct_provider_runs() {
        crate::with_cleared_rust_log(|| {
            let setup = test_setup(console_direct_provider(), level("info"));
            setup.with_subscriber_ignoring_env(|| {
                otel_info!("console_log");
            });
        });
    }

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
            // (the same path used by internal_telemetry_receiver → console
            // exporter).
            use crate::self_tracing::{ScopeToBytesMap, encode_export_logs_request};
            use bytes::Bytes;
            use otap_df_pdata::otlp::ProtoBuffer;
            use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
            use otap_df_pdata_views::views::logs::{
                LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
            };

            let resource_bytes = Bytes::new();
            let registry = crate::registry::TelemetryRegistryHandle::new();
            let mut scope_cache = ScopeToBytesMap::new(registry);
            let mut buf = ProtoBuffer::default();
            encode_export_logs_request(&mut buf, &log_event, &resource_bytes, &mut scope_cache);
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

    #[test]
    fn provider_setup_with_subscriber_all_variants() {
        crate::with_cleared_rust_log(|| {
            let info = level("info");
            let allow_all = EventNameFilter::allow_all();
            noop_provider().with_subscriber_ignoring_env(
                &info,
                LogContext::new,
                &allow_all,
                || {
                    otel_info!("noop");
                },
            );

            console_direct_provider().with_subscriber_ignoring_env(
                &info,
                LogContext::new,
                &allow_all,
                || {
                    otel_info!("console_direct");
                },
            );

            let (reporter, _rx) = test_reporter();
            internal_async_provider(reporter).with_subscriber_ignoring_env(
                &info,
                LogContext::new,
                &allow_all,
                || {
                    otel_info!("console_async");
                },
            );
        });
    }

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

    #[test]
    fn event_name_filter_composes_with_level_filter() {
        use crate::eventname_filter::EventNameFilter;

        // Allowlist by EventName, layered on top of the level filter. Only
        // events whose EventName passes *both* filters should be reported.
        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("info"))
                .with_event_filter(EventNameFilter::allowing(["receiver.start", "channel.*"]));

            setup.with_subscriber_ignoring_env(|| {
                otel_info!("receiver.start"); // allowed (exact)
                otel_info!("receiver.stop"); // dropped (not in allowlist)
                otel_warn!("channel.full"); // allowed (prefix)
                otel_error!("exporter.fail"); // dropped (not in allowlist)
            });
            drop(setup);

            assert_eq!(
                receiver.into_iter().count(),
                2,
                "only receiver.start and channel.full pass the EventName allowlist"
            );
        });
    }

    #[test]
    fn event_name_deny_filter_suppresses_matches() {
        use crate::eventname_filter::EventNameFilter;

        crate::with_cleared_rust_log(|| {
            let (reporter, receiver) = test_reporter();
            let setup = test_setup(internal_async_provider(reporter), level("info"))
                .with_event_filter(EventNameFilter::denying(["noisy.*"]));

            setup.with_subscriber_ignoring_env(|| {
                otel_info!("noisy.tick"); // dropped (denied prefix)
                otel_info!("noisy.tock"); // dropped (denied prefix)
                otel_warn!("important.event"); // passes
            });
            drop(setup);

            assert_eq!(
                receiver.into_iter().count(),
                1,
                "only the non-denied event passes"
            );
        });
    }

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
