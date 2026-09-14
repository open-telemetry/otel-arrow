// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime-reloadable filtering for internal telemetry logs.

use std::env;
use std::sync::{Arc, Mutex, Weak};

use arc_swap::ArcSwap;
use otel_arrow_dfe_config::settings::telemetry::logs::LogLevel;
use tracing::span::{Attributes, Id, Record};
use tracing::subscriber::Interest;
use tracing::{Metadata, Subscriber};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Context, Layer};

struct SharedState {
    // Captured once so removing an explicit setting has deterministic behavior.
    fallback_level: LogLevel,
    // The directive currently installed in all dispatcher-local filters.
    effective_level: ArcSwap<LogLevel>,
    template: ArcSwap<EnvFilter>,
    layers: Mutex<Vec<Weak<ArcSwap<EnvFilter>>>>,
}

/// One dispatcher-local layer managed by a [`RuntimeLogFilter`].
pub struct RuntimeLogFilterLayer {
    filter: Arc<ArcSwap<EnvFilter>>,
}

/// Creates an `EnvFilter` from a validated log level.
#[cfg(test)]
#[must_use]
pub(crate) fn create_env_filter(level: &LogLevel) -> EnvFilter {
    env_filter(level)
}

fn env_filter(level: &LogLevel) -> EnvFilter {
    EnvFilter::try_new(level.as_str()).expect("log level must be validated before use")
}

fn capture_fallback_level() -> LogLevel {
    env::var("RUST_LOG")
        .ok()
        .and_then(|value| LogLevel::try_from(value).ok())
        .unwrap_or_default()
}

/// A shared factory for runtime-reloadable `EnvFilter` layers.
///
/// Every dispatcher receives its own `EnvFilter` instance so span state and
/// locks are never shared across independent registries. Clones share the
/// update registry and current configuration.
#[derive(Clone)]
pub struct RuntimeLogFilter {
    shared: Arc<SharedState>,
}

/// A cloneable handle for applying configured log-level directives.
#[derive(Clone)]
pub struct RuntimeLogFilterHandle {
    shared: Arc<SharedState>,
}

impl RuntimeLogFilter {
    /// Creates a filter and its update handle from optional configuration.
    ///
    /// An explicit configured level takes precedence. When configuration omits
    /// the level, a valid `RUST_LOG` value captured here is used, followed by
    /// the built-in [`LogLevel::default`].
    #[must_use]
    pub fn new(configured_level: Option<&LogLevel>) -> (Self, RuntimeLogFilterHandle) {
        let fallback_level = capture_fallback_level();
        let effective_level = configured_level
            .cloned()
            .unwrap_or_else(|| fallback_level.clone());
        Self::from_levels(effective_level, fallback_level)
    }

    /// Creates a filter from the configured level without consulting `RUST_LOG`.
    ///
    /// This is intended for callers such as benchmarks that need deterministic
    /// configured-filter behavior independent of the process environment.
    #[must_use]
    pub fn new_configured(level: &LogLevel) -> (Self, RuntimeLogFilterHandle) {
        Self::from_levels(level.clone(), LogLevel::default())
    }

    fn from_levels(
        effective_level: LogLevel,
        fallback_level: LogLevel,
    ) -> (Self, RuntimeLogFilterHandle) {
        let filter = env_filter(&effective_level);
        let shared = Arc::new(SharedState {
            fallback_level,
            effective_level: ArcSwap::from_pointee(effective_level),
            template: ArcSwap::from_pointee(filter),
            layers: Mutex::new(Vec::new()),
        });
        (
            Self {
                shared: Arc::clone(&shared),
            },
            RuntimeLogFilterHandle { shared },
        )
    }

    #[cfg(test)]
    pub(crate) fn from_filter(level: LogLevel, filter: EnvFilter) -> Self {
        Self {
            shared: Arc::new(SharedState {
                fallback_level: level.clone(),
                effective_level: ArcSwap::from_pointee(level),
                template: ArcSwap::from_pointee(filter),
                layers: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Creates and registers a dispatcher-local filter layer.
    #[must_use]
    pub fn layer(&self) -> RuntimeLogFilterLayer {
        let mut layers = self
            .shared
            .layers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        layers.retain(|layer| layer.strong_count() > 0);
        let filter = Arc::new(ArcSwap::from_pointee(
            self.shared.template.load().as_ref().clone(),
        ));
        layers.push(Arc::downgrade(&filter));
        RuntimeLogFilterLayer { filter }
    }

    /// Returns the effective level currently installed in tracing filters.
    #[must_use]
    pub fn effective_level(&self) -> LogLevel {
        self.shared.effective_level.load().as_ref().clone()
    }
}

impl RuntimeLogFilterHandle {
    /// Replaces the active level/target directives and refreshes all callsites.
    ///
    /// Severity and target directives take effect immediately. Span-scoped
    /// directives such as `[pipeline_thread]=debug` do not apply to spans that
    /// were entered before this call: the replacement `EnvFilter` never
    /// observed their `on_new_span`/`on_enter` callbacks, so its scope stack
    /// stays empty for them. Such directives still work when supplied at
    /// startup. See the crate README for operator-facing details.
    pub fn apply(&self, configured_level: Option<&LogLevel>) {
        let level = configured_level
            .cloned()
            .unwrap_or_else(|| self.shared.fallback_level.clone());
        if self.shared.effective_level.load().as_ref() == &level {
            return;
        }
        let filter = env_filter(&level);
        let mut layers = self
            .shared
            .layers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.shared.effective_level.store(Arc::new(level));
        self.shared.template.store(Arc::new(filter.clone()));
        layers.retain(|layer| {
            if let Some(layer) = layer.upgrade() {
                layer.store(Arc::new(filter.clone()));
                true
            } else {
                false
            }
        });
        drop(layers);
        tracing::callsite::rebuild_interest_cache();
    }

    /// Returns the effective level currently installed in tracing filters.
    #[must_use]
    pub fn effective_level(&self) -> LogLevel {
        self.shared.effective_level.load().as_ref().clone()
    }
}

// Delegate to the dispatcher-local EnvFilter currently selected by ArcSwap.
impl<S: Subscriber> Layer<S> for RuntimeLogFilterLayer {
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        let filter = self.filter.load();
        <EnvFilter as Layer<S>>::register_callsite(&filter, metadata)
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        let filter = self.filter.load();
        <EnvFilter as Layer<S>>::max_level_hint(&filter)
    }

    fn enabled(&self, metadata: &Metadata<'_>, context: Context<'_, S>) -> bool {
        let filter = self.filter.load();
        <EnvFilter as Layer<S>>::enabled(&filter, metadata, context)
    }

    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        let filter = self.filter.load();
        <EnvFilter as Layer<S>>::on_new_span(&filter, attrs, id, context);
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, context: Context<'_, S>) {
        let filter = self.filter.load();
        <EnvFilter as Layer<S>>::on_record(&filter, id, values, context);
    }

    fn on_enter(&self, id: &Id, context: Context<'_, S>) {
        let filter = self.filter.load();
        <EnvFilter as Layer<S>>::on_enter(&filter, id, context);
    }

    fn on_exit(&self, id: &Id, context: Context<'_, S>) {
        let filter = self.filter.load();
        <EnvFilter as Layer<S>>::on_exit(&filter, id, context);
    }

    fn on_close(&self, id: Id, context: Context<'_, S>) {
        let filter = self.filter.load();
        <EnvFilter as Layer<S>>::on_close(&filter, id, context);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tracing::Event;
    use tracing_subscriber::Registry;
    use tracing_subscriber::layer::SubscriberExt;

    use super::*;

    struct CountingLayer(Arc<AtomicUsize>);

    impl<S: Subscriber> Layer<S> for CountingLayer {
        fn on_event(&self, _event: &Event<'_>, _context: Context<'_, S>) {
            _ = self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn level(value: &str) -> LogLevel {
        serde_yaml::from_str(value).expect("log level should parse")
    }

    /// Scenario: one info callsite is evaluated across warn, info, and error updates.
    /// Guarantees: rebuilding callsite interest enables and disables that same callsite.
    #[test]
    fn same_callsite_tracks_runtime_level_updates() {
        crate::with_cleared_rust_log(|| {
            let count = Arc::new(AtomicUsize::new(0));
            let (filter, handle) = RuntimeLogFilter::new(Some(&level("warn")));
            let subscriber = Registry::default()
                .with(filter.layer())
                .with(CountingLayer(Arc::clone(&count)));

            tracing::subscriber::with_default(subscriber, || {
                let emit_info = || tracing::info!("runtime level test");

                emit_info();
                assert_eq!(count.swap(0, Ordering::SeqCst), 0);

                handle.apply(Some(&level("info")));
                emit_info();
                assert_eq!(count.swap(0, Ordering::SeqCst), 1);

                handle.apply(Some(&level("error")));
                emit_info();
                assert_eq!(count.swap(0, Ordering::SeqCst), 0);
            });
        });
    }

    /// Scenario: target-specific directives are replaced while the subscriber remains active.
    /// Guarantees: runtime updates retain full EnvFilter target directive semantics.
    #[test]
    fn runtime_update_preserves_target_directives() {
        crate::with_cleared_rust_log(|| {
            let count = Arc::new(AtomicUsize::new(0));
            let (filter, handle) = RuntimeLogFilter::new(Some(&level("off")));
            let subscriber = Registry::default()
                .with(filter.layer())
                .with(CountingLayer(Arc::clone(&count)));

            tracing::subscriber::with_default(subscriber, || {
                tracing::info!(target: "runtime_allowed", "allowed target");
                assert_eq!(count.swap(0, Ordering::SeqCst), 0);

                handle.apply(Some(&level("off,runtime_allowed=info")));
                tracing::info!(target: "runtime_allowed", "allowed target");
                tracing::info!(target: "runtime_blocked", "blocked target");
                assert_eq!(count.swap(0, Ordering::SeqCst), 1);
            });
        });
    }

    /// Scenario: a span-scoped directive changes while a matching span is entered.
    /// Guarantees: the existing span falls back to the new base directive, while a new span uses the new span directive.
    #[test]
    fn runtime_update_applies_span_directive_only_to_new_spans() {
        crate::with_cleared_rust_log(|| {
            let count = Arc::new(AtomicUsize::new(0));
            let (filter, handle) = RuntimeLogFilter::new(Some(&level("warn,[reload_span]=debug")));
            let subscriber = Registry::default()
                .with(filter.layer())
                .with(CountingLayer(Arc::clone(&count)));

            tracing::subscriber::with_default(subscriber, || {
                let span = tracing::info_span!("reload_span");
                let entered = span.enter();
                tracing::debug!("old span directive permits this event");
                assert_eq!(count.swap(0, Ordering::SeqCst), 1);

                handle.apply(Some(&level("warn,[reload_span]=trace")));
                tracing::debug!("existing span falls back to warn");
                assert_eq!(count.swap(0, Ordering::SeqCst), 0);
                drop(entered);
                drop(span);

                let span = tracing::info_span!("reload_span");
                let _entered = span.enter();
                tracing::trace!("new span directive permits this event");
                assert_eq!(count.load(Ordering::SeqCst), 1);
            });
        });
    }

    /// Scenario: two independent dispatchers are created from one runtime filter.
    /// Guarantees: one update refreshes both dispatcher-local EnvFilter instances.
    #[test]
    fn runtime_update_reaches_multiple_dispatchers() {
        crate::with_cleared_rust_log(|| {
            let first_count = Arc::new(AtomicUsize::new(0));
            let second_count = Arc::new(AtomicUsize::new(0));
            let (filter, handle) = RuntimeLogFilter::new(Some(&level("warn")));
            let first = tracing::Dispatch::new(
                Registry::default()
                    .with(filter.layer())
                    .with(CountingLayer(Arc::clone(&first_count))),
            );
            let second = tracing::Dispatch::new(
                Registry::default()
                    .with(filter.layer())
                    .with(CountingLayer(Arc::clone(&second_count))),
            );
            let emit_info = || tracing::info!("multi-dispatch runtime level test");

            tracing::dispatcher::with_default(&first, emit_info);
            tracing::dispatcher::with_default(&second, emit_info);
            assert_eq!(first_count.load(Ordering::SeqCst), 0);
            assert_eq!(second_count.load(Ordering::SeqCst), 0);

            handle.apply(Some(&level("info")));
            tracing::dispatcher::with_default(&first, emit_info);
            tracing::dispatcher::with_default(&second, emit_info);
            assert_eq!(first_count.load(Ordering::SeqCst), 1);
            assert_eq!(second_count.load(Ordering::SeqCst), 1);
        });
    }

    /// Scenario: a dispatcher is created after the runtime log level changes.
    /// Guarantees: new dispatcher-local layers start with the latest filter template.
    #[test]
    fn dispatcher_created_after_update_uses_latest_filter() {
        crate::with_cleared_rust_log(|| {
            let count = Arc::new(AtomicUsize::new(0));
            let (filter, handle) = RuntimeLogFilter::new(Some(&level("warn")));

            handle.apply(Some(&level("info")));
            let dispatch = tracing::Dispatch::new(
                Registry::default()
                    .with(filter.layer())
                    .with(CountingLayer(Arc::clone(&count))),
            );

            tracing::dispatcher::with_default(&dispatch, || {
                tracing::info!("post-update dispatcher test");
            });
            assert_eq!(count.load(Ordering::SeqCst), 1);
        });
    }

    /// Scenario: RUST_LOG is set while logs.level is explicitly configured.
    /// Guarantees: the explicit configuration controls the initial filter without an activation update.
    #[test]
    fn explicit_config_overrides_rust_log_from_initial_filter() {
        crate::with_rust_log(Some("error"), || {
            let count = Arc::new(AtomicUsize::new(0));
            let (filter, handle) = RuntimeLogFilter::new(Some(&level("info")));
            let subscriber = Registry::default()
                .with(filter.layer())
                .with(CountingLayer(Arc::clone(&count)));

            tracing::subscriber::with_default(subscriber, || {
                tracing::info!("explicit logs.level permits this initial event");
                assert_eq!(count.swap(0, Ordering::SeqCst), 1);
            });

            assert_eq!(handle.effective_level().as_str(), "info");
        });
    }

    /// Scenario: logs.level is omitted while RUST_LOG supplies the startup fallback.
    /// Guarantees: an explicit runtime override can be applied and later removed to restore RUST_LOG.
    #[test]
    fn omitted_config_uses_and_restores_captured_rust_log() {
        crate::with_rust_log(Some("error"), || {
            let count = Arc::new(AtomicUsize::new(0));
            let (filter, handle) = RuntimeLogFilter::new(None);
            let subscriber = Registry::default()
                .with(filter.layer())
                .with(CountingLayer(Arc::clone(&count)));

            tracing::subscriber::with_default(subscriber, || {
                tracing::info!("RUST_LOG blocks the initial event");
                assert_eq!(count.swap(0, Ordering::SeqCst), 0);

                handle.apply(Some(&level("info")));
                tracing::info!("explicit logs.level permits the event");
                assert_eq!(count.swap(0, Ordering::SeqCst), 1);

                handle.apply(None);
                tracing::info!("restored RUST_LOG blocks the event again");
                assert_eq!(count.swap(0, Ordering::SeqCst), 0);
            });

            assert_eq!(handle.effective_level().as_str(), "error");
        });
    }

    /// Scenario: logs.level is omitted and RUST_LOG contains an invalid directive.
    /// Guarantees: the initial filter safely uses the built-in default directive.
    #[test]
    fn invalid_rust_log_uses_builtin_fallback() {
        crate::with_rust_log(Some("info,["), || {
            let (filter, handle) = RuntimeLogFilter::new(None);
            let count = Arc::new(AtomicUsize::new(0));
            let subscriber = Registry::default()
                .with(filter.layer())
                .with(CountingLayer(Arc::clone(&count)));

            tracing::subscriber::with_default(subscriber, || {
                tracing::info!("built-in info fallback permits this event");
            });

            assert_eq!(count.load(Ordering::SeqCst), 1);
            assert_eq!(handle.effective_level(), LogLevel::default());
        });
    }

    /// Scenario: a deterministic configured filter is created while RUST_LOG is set.
    /// Guarantees: new_configured uses logs.level without consulting the environment.
    #[test]
    fn configured_filter_ignores_rust_log() {
        crate::with_rust_log(Some("error"), || {
            let count = Arc::new(AtomicUsize::new(0));
            let (filter, _handle) = RuntimeLogFilter::new_configured(&level("info"));
            let subscriber = Registry::default()
                .with(filter.layer())
                .with(CountingLayer(Arc::clone(&count)));

            tracing::subscriber::with_default(subscriber, || {
                tracing::info!("configured filter ignores RUST_LOG");
            });

            assert_eq!(count.load(Ordering::SeqCst), 1);
        });
    }

    /// Scenario: a dispatcher-local layer is dropped before another is created.
    /// Guarantees: dead registrations are pruned without waiting for an update.
    #[test]
    fn layer_creation_prunes_dropped_dispatchers() {
        crate::with_cleared_rust_log(|| {
            let (filter, _handle) = RuntimeLogFilter::new(Some(&level("warn")));
            let first = filter.layer();
            assert_eq!(filter.shared.layers.lock().unwrap().len(), 1);
            drop(first);

            let _second = filter.layer();
            assert_eq!(filter.shared.layers.lock().unwrap().len(), 1);
        });
    }

    /// Scenario: reconciliation reapplies the currently configured log level.
    /// Guarantees: an unchanged level preserves the existing dispatcher filter instance.
    #[test]
    fn unchanged_level_does_not_replace_filters() {
        crate::with_cleared_rust_log(|| {
            let (filter, handle) = RuntimeLogFilter::new(Some(&level("warn")));
            let layer = filter.layer();
            let before = layer.filter.load_full();

            handle.apply(Some(&level("warn")));

            let after = layer.filter.load_full();
            assert!(Arc::ptr_eq(&before, &after));
        });
    }
}
