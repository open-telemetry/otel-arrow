// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime-reloadable filtering for internal telemetry logs.

use std::sync::{Arc, Mutex, Weak};

use arc_swap::ArcSwap;
use otap_df_config::settings::telemetry::logs::LogLevel;
use tracing::span::{Attributes, Id, Record};
use tracing::subscriber::Interest;
use tracing::{Metadata, Subscriber};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Context, Layer};

struct SharedState {
    configured_level: ArcSwap<LogLevel>,
    template: ArcSwap<EnvFilter>,
    layers: Mutex<Vec<Weak<ArcSwap<EnvFilter>>>>,
}

/// One dispatcher-local layer managed by a [`RuntimeLogFilter`].
pub struct RuntimeLogFilterLayer {
    filter: Arc<ArcSwap<EnvFilter>>,
}

/// Creates an `EnvFilter` from `RUST_LOG`, falling back to `level`.
///
/// `RUST_LOG` remains an explicit process-level override when runtime
/// configuration updates are applied.
#[must_use]
pub fn create_env_filter(level: &LogLevel) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.as_str()))
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

/// A cloneable handle for applying reconciled log-level directives.
#[derive(Clone)]
pub struct RuntimeLogFilterHandle {
    shared: Arc<SharedState>,
}

impl RuntimeLogFilter {
    /// Creates a filter and its update handle from the configured log level.
    #[must_use]
    pub fn new(level: &LogLevel) -> (Self, RuntimeLogFilterHandle) {
        let shared = Arc::new(SharedState {
            configured_level: ArcSwap::from_pointee(level.clone()),
            template: ArcSwap::from_pointee(create_env_filter(level)),
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
                configured_level: ArcSwap::from_pointee(level),
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
}

impl RuntimeLogFilterHandle {
    /// Replaces the active level/target directives and refreshes all callsites.
    pub fn apply(&self, level: &LogLevel) {
        if self.shared.configured_level.load().as_ref() == level {
            return;
        }
        let filter = create_env_filter(level);
        let mut layers = self
            .shared
            .layers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.shared.configured_level.store(Arc::new(level.clone()));
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

    /// Returns the most recently applied configuration value.
    #[must_use]
    pub fn configured_level(&self) -> LogLevel {
        self.shared.configured_level.load().as_ref().clone()
    }
}

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
            let (filter, handle) = RuntimeLogFilter::new(&level("warn"));
            let subscriber = Registry::default()
                .with(filter.layer())
                .with(CountingLayer(Arc::clone(&count)));

            tracing::subscriber::with_default(subscriber, || {
                let emit_info = || tracing::info!("runtime level test");

                emit_info();
                assert_eq!(count.swap(0, Ordering::SeqCst), 0);

                handle.apply(&level("info"));
                emit_info();
                assert_eq!(count.swap(0, Ordering::SeqCst), 1);

                handle.apply(&level("error"));
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
            let (filter, handle) = RuntimeLogFilter::new(&level("off"));
            let subscriber = Registry::default()
                .with(filter.layer())
                .with(CountingLayer(Arc::clone(&count)));

            tracing::subscriber::with_default(subscriber, || {
                tracing::info!(target: "runtime_allowed", "allowed target");
                assert_eq!(count.swap(0, Ordering::SeqCst), 0);

                handle.apply(&level("off,runtime_allowed=info"));
                tracing::info!(target: "runtime_allowed", "allowed target");
                tracing::info!(target: "runtime_blocked", "blocked target");
                assert_eq!(count.swap(0, Ordering::SeqCst), 1);
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
            let (filter, handle) = RuntimeLogFilter::new(&level("warn"));
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

            handle.apply(&level("info"));
            tracing::dispatcher::with_default(&first, emit_info);
            tracing::dispatcher::with_default(&second, emit_info);
            assert_eq!(first_count.load(Ordering::SeqCst), 1);
            assert_eq!(second_count.load(Ordering::SeqCst), 1);
        });
    }

    /// Scenario: a dispatcher-local layer is dropped before another is created.
    /// Guarantees: dead registrations are pruned without waiting for an update.
    #[test]
    fn layer_creation_prunes_dropped_dispatchers() {
        let (filter, _handle) = RuntimeLogFilter::new(&level("warn"));
        let first = filter.layer();
        assert_eq!(filter.shared.layers.lock().unwrap().len(), 1);
        drop(first);

        let _second = filter.layer();
        assert_eq!(filter.shared.layers.lock().unwrap().len(), 1);
    }

    /// Scenario: reconciliation reapplies the currently configured log level.
    /// Guarantees: an unchanged level preserves the existing dispatcher filter instance.
    #[test]
    fn unchanged_level_does_not_replace_filters() {
        let (filter, handle) = RuntimeLogFilter::new(&level("warn"));
        let layer = filter.layer();
        let before = layer.filter.load_full();

        handle.apply(&level("warn"));

        let after = layer.filter.load_full();
        assert!(Arc::ptr_eq(&before, &after));
    }
}
