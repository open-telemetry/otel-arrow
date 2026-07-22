// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-EventName runtime filtering for internal telemetry logs.
//!
//! The internal telemetry macros (`otel_info!`, `otel_warn!`, ...) set the
//! tracing event's *metadata name* to the OpenTelemetry **EventName** (a short,
//! stable identifier such as `receiver.start` or `channel.full`). See
//! [`crate::internal_events`]. `tracing`'s built-in [`EnvFilter`] can only gate
//! on *target + level*, never on this EventName, so it cannot single out one
//! event (or one subsystem) at runtime.
//!
//! [`EventNameFilter`] closes that gap. It is a per-layer
//! [`Filter`](tracing_subscriber::layer::Filter) that matches
//! `metadata.name()` against a dynamically updatable set of EventName patterns,
//! enabling the "zoom in / zoom out" model at the granularity of a single event
//! or a whole subsystem.
//!
//! # Matching
//!
//! Each pattern is either:
//! - **Exact** — `receiver.start` matches only that EventName; or
//! - **Prefix** — a trailing `*` (`receiver.*`, or `receiver*`) matches every
//!   EventName starting with the stem. This maps naturally onto the dotted,
//!   hierarchical EventName convention ("zoom into a whole subsystem").
//!
//! Mid-string globs and regular expressions are intentionally *not* supported:
//! the matcher stays a hash lookup plus a short prefix scan, which is cheap on
//! the per-event hot path. The matcher can be swapped later without touching
//! the [`Filter`](tracing_subscriber::layer::Filter) / [`Interest`] plumbing.
//!
//! # Runtime dynamism (the important part)
//!
//! `tracing` caches per-callsite [`Interest`]. If [`callsite_enabled`] returned
//! [`Interest::always`] or [`Interest::never`], the decision would be cached
//! once per callsite and later changes to the pattern set would silently have
//! no effect. To keep the set changeable at runtime, [`callsite_enabled`]
//! returns [`Interest::sometimes`], which forces `enabled` to run on every
//! event. The active mode lives behind an [`ArcSwap`] so a control plane can
//! swap it in without blocking readers and without rebuilding the subscriber.
//!
//! [`EnvFilter`]: tracing_subscriber::EnvFilter
//! [`callsite_enabled`]: tracing_subscriber::layer::Filter::callsite_enabled

use std::collections::HashSet;
use std::sync::Arc;

use arc_swap::ArcSwap;
use tracing::subscriber::Interest;
use tracing::{Metadata, Subscriber};
use tracing_subscriber::layer::{Context, Filter};

/// A compiled set of EventName patterns split into exact matches and prefixes.
#[derive(Clone, Debug, Default)]
struct Patterns {
    /// Exact EventName matches; `O(1)` lookup on the hot path.
    exact: HashSet<Box<str>>,
    /// Prefix stems (from patterns ending in `*`); scanned linearly. Expected
    /// to be short, so `starts_with` over this list stays cheap.
    prefixes: Vec<Box<str>>,
}

impl Patterns {
    /// Compile pattern specs. A spec ending in `*` becomes a prefix match on
    /// the preceding stem; every other spec is an exact match.
    fn compile<I, S>(specs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut exact = HashSet::new();
        let mut prefixes = Vec::new();
        for spec in specs {
            let spec = spec.as_ref();
            if let Some(stem) = spec.strip_suffix('*') {
                prefixes.push(Box::from(stem));
            } else {
                let _ = exact.insert(Box::from(spec));
            }
        }
        Self { exact, prefixes }
    }

    /// Returns `true` if `name` matches any exact entry or prefix stem.
    fn matches(&self, name: &str) -> bool {
        if self.exact.contains(name) {
            return true;
        }
        self.prefixes.iter().any(|stem| name.starts_with(&**stem))
    }
}

/// The active matching mode.
#[derive(Clone, Debug, Default)]
enum Mode {
    /// Every EventName passes. This is the default and preserves the behavior
    /// of a pipeline with no EventName filtering configured.
    #[default]
    AllowAll,
    /// Only EventNames matching one of these patterns pass ("zoom in").
    Allow(Patterns),
    /// Every EventName passes *except* those matching one of these patterns
    /// ("zoom out but suppress known noise").
    Deny(Patterns),
}

impl Mode {
    /// Applies the mode to a single EventName.
    #[inline]
    fn allows(&self, name: &str) -> bool {
        match self {
            Mode::AllowAll => true,
            Mode::Allow(patterns) => patterns.matches(name),
            Mode::Deny(patterns) => !patterns.matches(name),
        }
    }
}

/// A per-layer [`Filter`](tracing_subscriber::layer::Filter) that gates events
/// by their OpenTelemetry EventName (the tracing metadata `name`).
///
/// Create one with [`EventNameFilter::new`], attach it to a layer with
/// [`Layer::with_filter`](tracing_subscriber::Layer::with_filter), and mutate it
/// at runtime through the paired [`EventNameFilterHandle`]. Defaults to
/// [`Mode::AllowAll`], so installing it is a no-op until the handle narrows it.
///
/// Cloning shares the same underlying mode, so the filter and its handle always
/// observe each other's updates.
#[derive(Clone)]
pub struct EventNameFilter {
    mode: Arc<ArcSwap<Mode>>,
}

/// A cheaply cloneable handle for updating an [`EventNameFilter`] at runtime.
///
/// Updates are lock-free swaps and take effect on the next event, without
/// rebuilding the tracing subscriber.
#[derive(Clone)]
pub struct EventNameFilterHandle {
    mode: Arc<ArcSwap<Mode>>,
}

impl EventNameFilter {
    /// Creates a filter (defaulting to [`Mode::AllowAll`]) and a handle that
    /// controls it.
    #[must_use]
    pub fn new() -> (Self, EventNameFilterHandle) {
        let mode = Arc::new(ArcSwap::from_pointee(Mode::AllowAll));
        (Self { mode: mode.clone() }, EventNameFilterHandle { mode })
    }

    /// Builds a filter fixed to `mode`, with no external handle.
    fn from_mode(mode: Mode) -> Self {
        Self {
            mode: Arc::new(ArcSwap::from_pointee(mode)),
        }
    }

    /// A static filter that lets every EventName pass. Installing it is a no-op
    /// relative to having no EventName filter at all.
    #[must_use]
    pub fn allow_all() -> Self {
        Self::from_mode(Mode::AllowAll)
    }

    /// A static filter that passes only EventNames matching one of `specs`.
    ///
    /// An empty set drops every internal-telemetry event. A spec ending in `*`
    /// is a prefix match.
    #[must_use]
    pub fn allowing<I, S>(specs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::from_mode(Mode::Allow(Patterns::compile(specs)))
    }

    /// A static filter that passes every EventName except those matching one of
    /// `specs`. A spec ending in `*` is a prefix match.
    #[must_use]
    pub fn denying<I, S>(specs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::from_mode(Mode::Deny(Patterns::compile(specs)))
    }
}

impl EventNameFilterHandle {
    /// Let every EventName pass (the default).
    pub fn allow_all(&self) {
        self.mode.store(Arc::new(Mode::AllowAll));
    }

    /// Pass only EventNames matching one of `specs`. An empty set drops every
    /// internal-telemetry event. A spec ending in `*` is a prefix match.
    pub fn allow<I, S>(&self, specs: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.mode
            .store(Arc::new(Mode::Allow(Patterns::compile(specs))));
    }

    /// Pass every EventName except those matching one of `specs`. A spec ending
    /// in `*` is a prefix match.
    pub fn deny<I, S>(&self, specs: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.mode
            .store(Arc::new(Mode::Deny(Patterns::compile(specs))));
    }
}

impl<S: Subscriber> Filter<S> for EventNameFilter {
    #[inline]
    fn enabled(&self, meta: &Metadata<'_>, _cx: &Context<'_, S>) -> bool {
        // `meta.name()` is the OpenTelemetry EventName set by the `otel_*!`
        // macros (e.g. "receiver.start").
        self.mode.load().allows(meta.name())
    }

    fn callsite_enabled(&self, _meta: &'static Metadata<'static>) -> Interest {
        // The decision depends on the mutable mode, so it must be re-evaluated
        // per event. Returning `sometimes()` (rather than `always`/`never`)
        // defeats `tracing`'s per-callsite interest cache and is what makes
        // runtime enable/disable actually take effect.
        Interest::sometimes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tracing::{Event, Subscriber};
    use tracing_subscriber::Registry;
    use tracing_subscriber::layer::{Context, Layer};
    use tracing_subscriber::prelude::*;

    #[test]
    fn patterns_exact_and_prefix() {
        let p = Patterns::compile(["receiver.start", "channel.*"]);
        assert!(p.matches("receiver.start"));
        assert!(!p.matches("receiver.stop"));
        assert!(p.matches("channel.full"));
        assert!(p.matches("channel.drop"));
        assert!(!p.matches("exporter.start"));
    }

    #[test]
    fn empty_allowlist_matches_nothing() {
        let p = Patterns::compile(Vec::<&str>::new());
        assert!(!p.matches("anything"));
    }

    /// A layer that counts events it receives (i.e. events that passed filters).
    struct CountingLayer(Arc<AtomicUsize>);
    impl<S: Subscriber> Layer<S> for CountingLayer {
        fn on_event(&self, _event: &Event<'_>, _cx: Context<'_, S>) {
            let _ = self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Exercises a *single* callsite repeatedly across mode changes. This is the
    /// core proof that runtime toggling works: because each `emit_start()` call
    /// hits the same callsite, a stale cached `Interest` would freeze the very
    /// first decision. The assertions below only hold because
    /// `callsite_enabled` returns `Interest::sometimes()`.
    #[test]
    fn same_callsite_re_evaluates_across_mode_changes() {
        let count = Arc::new(AtomicUsize::new(0));
        let (filter, handle) = EventNameFilter::new();
        let subscriber = Registry::default().with(CountingLayer(count.clone()).with_filter(filter));

        tracing::subscriber::with_default(subscriber, || {
            // Read and reset the counter.
            let delta = || count.swap(0, Ordering::SeqCst);
            // Every call below is the SAME callsite (one source line).
            let emit_start = || tracing::info!(name: "receiver.start", detail = "x");

            // Default AllowAll: passes.
            emit_start();
            assert_eq!(delta(), 1, "AllowAll should pass");

            // Narrow to an allowlist that excludes it: dropped.
            handle.allow(["exporter.start"]);
            emit_start();
            assert_eq!(delta(), 0, "excluded by allowlist");

            // Add it to the allowlist: passes again (same callsite re-evaluated).
            handle.allow(["receiver.start"]);
            emit_start();
            assert_eq!(delta(), 1, "included by allowlist");

            // Back to AllowAll: passes.
            handle.allow_all();
            emit_start();
            assert_eq!(delta(), 1, "AllowAll again");

            // Deny it: dropped.
            handle.deny(["receiver.*"]);
            emit_start();
            assert_eq!(delta(), 0, "excluded by denylist prefix");
        });
    }

    #[test]
    fn prefix_and_exact_distinguish_sibling_events() {
        let count = Arc::new(AtomicUsize::new(0));
        let (filter, handle) = EventNameFilter::new();
        let subscriber = Registry::default().with(CountingLayer(count.clone()).with_filter(filter));

        tracing::subscriber::with_default(subscriber, || {
            let delta = || count.swap(0, Ordering::SeqCst);

            // Exact allow: only receiver.start, not its sibling.
            handle.allow(["receiver.start"]);
            tracing::info!(name: "receiver.start", detail = "x");
            tracing::info!(name: "receiver.stop", detail = "x");
            assert_eq!(delta(), 1, "only the exact match passes");

            // Prefix allow: both receiver.* siblings, but not exporter.*.
            handle.allow(["receiver.*"]);
            tracing::info!(name: "receiver.start", detail = "x");
            tracing::info!(name: "receiver.stop", detail = "x");
            tracing::info!(name: "exporter.start", detail = "x");
            assert_eq!(delta(), 2, "both receiver.* pass, exporter.* does not");
        });
    }
}
