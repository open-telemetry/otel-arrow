// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded diagnostics for repeated operation failures and confirmed recovery.
//!
//! [`EpisodeSampler::observe`] processes completions before independently filtered
//! WARN and INFO callsites. [`IntervalSampler`] only observes recurring failures
//! and never retains a record. Both use [`DiagnosticTracker`] for bounded counts.
//!
//! Integrations own event names, error classifications, and observation
//! boundaries. Keep distinct operations in separate trackers; a successful
//! enqueue, for example, cannot establish recovery of a failing storage write.
//! [`SignalSet`] optionally groups state by telemetry signal.
//!
//! A tracker has no timers, locks, or metric-interest dependency. Selected
//! observations snapshot counters; only emitted warnings encode failure fields.

use crate::attributes::AttributeEnum;
use otel_arrow_dfe_config::SignalType;
use std::fmt;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

/// Minimum spacing between summaries of an ongoing episode.
pub const SUMMARY_INTERVAL: Duration = Duration::from_secs(60);
/// Failure-free interval required before fresh success clears an episode.
pub const RECOVERY_INTERVAL: Duration = Duration::from_secs(30);

mod samplers;
pub use samplers::{DiagnosticEmission, EpisodeSampler, IntervalSampler};

/// Common operation failure classifications for callers without a more specific enum.
#[derive(Clone, Copy, Debug, otel_arrow_dfe_telemetry_macros::AttributeEnum)]
pub enum DiagnosticErrorKind {
    /// Network or stream failure.
    Transport,
    /// Authentication failed.
    Authentication,
    /// Authorization failed.
    Authorization,
    /// An operation timed out.
    Timeout,
    /// Resource or service capacity was exhausted.
    Throttled,
    /// A service reported an internal error.
    ServerError,
    /// An operation or its input was rejected.
    Rejected,
    /// An operation accepted only part of its input.
    PartialRejection,
    /// Local encoding or preparation failed.
    Preparation,
    /// Upstream Ack/Nack delivery failed.
    Notification,
    /// File or database operation failed.
    Io,
    /// The protocol response could not be correlated or consumed.
    Protocol,
    /// Failure without a more specific classification.
    Other,
}

/// The reason a report was produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportKind {
    /// First failure of an episode.
    Degraded,
    /// Further observations during an episode.
    Summary,
    /// Fresh operation success after the failure-free confirmation interval.
    Recovered,
}

impl ReportKind {
    /// Diagnostic annotation for this reporting decision.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Degraded => "first_failure",
            Self::Summary => "summary",
            Self::Recovered => "recovery",
        }
    }
}

/// Counts for an interval or a complete episode. Counts represent attempts at
/// the caller's observation stage, never unique batches or records lost.
#[derive(Clone, Debug)]
pub struct Counts<E> {
    /// Successful attempts at the observed stage.
    pub successes: u64,
    /// Failed attempts at the observed stage.
    pub failures: u64,
    /// Failure diagnostics not emitted individually.
    pub suppressed: u64,
    errors: Box<[u64]>,
    marker: PhantomData<E>,
}

impl<E: AttributeEnum> Default for Counts<E> {
    fn default() -> Self {
        Self {
            successes: 0,
            failures: 0,
            suppressed: 0,
            errors: vec![0; E::CARDINALITY].into_boxed_slice(),
            marker: PhantomData,
        }
    }
}

impl<E: AttributeEnum> Counts<E> {
    fn failure(&mut self, category: E, suppressed: bool) {
        self.failures = self.failures.saturating_add(1);
        self.suppressed = self.suppressed.saturating_add(u64::from(suppressed));
        let count = &mut self.errors[category.variant_index()];
        *count = count.saturating_add(1);
    }

    fn reset(&mut self) {
        self.successes = 0;
        self.failures = 0;
        self.suppressed = 0;
        self.errors.fill(0);
    }
}

impl<E: AttributeEnum> fmt::Display for Counts<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut separator = "";
        for (category, count) in E::VARIANTS.iter().zip(&self.errors) {
            if *count != 0 {
                write!(f, "{separator}{category}={count}")?;
                separator = ",";
            }
        }
        Ok(())
    }
}

/// Counter snapshot selected independently of log level/filter decisions.
#[derive(Debug)]
pub struct DiagnosticReport<E> {
    /// Transition or summary.
    pub kind: ReportKind,
    /// Time since the initial observed failure, not measured service downtime.
    pub episode_duration: Duration,
    /// Time covered by `interval`.
    pub interval_duration: Duration,
    /// Counts since the previous report, including this observation.
    pub interval: Counts<E>,
    /// Counts since the episode began, including this observation.
    pub total: Counts<E>,
}

#[derive(Debug)]
struct Episode<E> {
    started_at: Instant,
    last_failure: Instant,
    last_report: Instant,
    interval: Counts<E>,
    total: Counts<E>,
}

impl<E: AttributeEnum> Episode<E> {
    fn report(&mut self, kind: ReportKind, now: Instant) -> DiagnosticReport<E> {
        let report = DiagnosticReport {
            kind,
            episode_duration: now.saturating_duration_since(self.started_at),
            interval_duration: now.saturating_duration_since(self.last_report),
            interval: self.interval.clone(),
            total: self.total.clone(),
        };
        self.interval.reset();
        self.last_report = now;
        report
    }
}

/// One operation's observed failure episode in a caller-defined, bounded scope.
///
/// A first failure opens an episode; confirmed recovery clears it. Successful
/// operation before an episode is silent. Callers without meaningful recovery
/// evidence can observe failures only, retaining totals until the tracker is
/// dropped. Keep separate instances for distinct operations and scopes.
#[derive(Debug)]
pub struct DiagnosticTracker<E> {
    episode: Option<Episode<E>>,
}

impl<E> Default for DiagnosticTracker<E> {
    fn default() -> Self {
        Self { episode: None }
    }
}

impl<E: AttributeEnum> DiagnosticTracker<E> {
    /// Observe a failure and select a first warning or interval summary.
    pub fn failure(&mut self, now: Instant, category: E) -> Option<DiagnosticReport<E>> {
        let first = self.episode.is_none();
        let episode = self.episode.get_or_insert_with(|| Episode {
            started_at: now,
            last_failure: now,
            last_report: now,
            interval: Counts::default(),
            total: Counts::default(),
        });
        let emit = first || now.saturating_duration_since(episode.last_report) >= SUMMARY_INTERVAL;
        episode.last_failure = now;
        episode.interval.failure(category, !emit);
        episode.total.failure(category, !emit);
        if !emit {
            return None;
        }
        Some(episode.report(
            if first {
                ReportKind::Degraded
            } else {
                ReportKind::Summary
            },
            now,
        ))
    }

    /// Observe actual success of the same operation and scope as past failures.
    /// An in-flight attempt begun before the latest observed failure cannot
    /// confirm recovery, regardless of its age.
    pub fn success(&mut self, started_at: Instant, now: Instant) -> Option<DiagnosticReport<E>> {
        let episode = self.episode.as_mut()?;
        episode.interval.successes = episode.interval.successes.saturating_add(1);
        episode.total.successes = episode.total.successes.saturating_add(1);
        if started_at > episode.last_failure
            && now.saturating_duration_since(episode.last_failure) >= RECOVERY_INTERVAL
        {
            let report = episode.report(ReportKind::Recovered, now);
            self.episode = None;
            return Some(report);
        }
        // Do not claim continuing failures after an idle period or repeatedly
        // report successes from old work: a summary needs new failure evidence.
        if episode.interval.failures != 0
            && now.saturating_duration_since(episode.last_report) >= SUMMARY_INTERVAL
        {
            return Some(episode.report(ReportKind::Summary, now));
        }
        None
    }
}

/// Fixed, allocation-free state for the three telemetry signals.
///
/// This container keeps signal indexing consistent for component-specific state
/// as well as shared diagnostic trackers. It avoids dynamic keys while allowing
/// each integration to attach the metadata its operation requires.
#[derive(Debug)]
pub struct SignalSet<T> {
    signals: [T; 3],
}

impl<T: Default> Default for SignalSet<T> {
    fn default() -> Self {
        Self {
            signals: std::array::from_fn(|_| T::default()),
        }
    }
}

impl<T> SignalSet<T> {
    /// Initializes the fixed set without a dynamic map.
    pub fn new(mut init: impl FnMut() -> T) -> Self {
        Self {
            signals: std::array::from_fn(|_| init()),
        }
    }

    /// Selects one signal's state.
    pub fn signal(&mut self, signal: SignalType) -> &mut T {
        &mut self.signals[match signal {
            SignalType::Logs => 0,
            SignalType::Metrics => 1,
            SignalType::Traces => 2,
        }]
    }
}

/// Independent diagnostic trackers for the three telemetry signals.
///
/// Callers whose operation has no telemetry signal can use [`DiagnosticTracker`]
/// directly. This alias adds a fixed set of scopes without dynamic keys.
pub type SignalDiagnostics<E> = SignalSet<DiagnosticTracker<E>>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: A destination fails 100,000 times within each reporting window.
    /// Guarantees: Warnings and formatting are time bounded while all failures are counted.
    #[test]
    fn sustained_failure_is_bounded_and_counted() {
        let mut tracker = DiagnosticTracker::default();
        let start = Instant::now();
        let first = tracker
            .failure(start, DiagnosticErrorKind::Transport)
            .unwrap();
        assert_eq!(first.kind, ReportKind::Degraded);
        assert_eq!(first.total.failures, 1);
        for _ in 0..100_000 {
            assert!(
                tracker
                    .failure(
                        start + Duration::from_secs(1),
                        DiagnosticErrorKind::Transport,
                    )
                    .is_none()
            );
        }
        let summary = tracker
            .failure(start + SUMMARY_INTERVAL, DiagnosticErrorKind::Rejected)
            .unwrap();
        assert_eq!(summary.kind, ReportKind::Summary);
        assert_eq!(summary.interval.failures, 100_001);
        assert_eq!(summary.total.failures, 100_002);
        assert_eq!(summary.interval.suppressed, 100_000);
        assert_eq!(summary.interval.to_string(), "transport=100000,rejected=1");
        assert_eq!(summary.interval_duration, SUMMARY_INTERVAL);
    }

    /// Scenario: Concurrent old attempts complete successfully after a new failure.
    /// Guarantees: Recovery needs fresh delivery evidence and 30 failure-free seconds.
    #[test]
    fn stale_success_and_idle_time_do_not_confirm_recovery() {
        let mut tracker = DiagnosticTracker::default();
        let start = Instant::now();
        assert!(tracker.success(start, start).is_none());
        assert!(
            tracker
                .failure(start, DiagnosticErrorKind::Transport)
                .is_some()
        );
        assert!(
            tracker
                .success(start, start + Duration::from_secs(300))
                .is_none()
        );
        let fresh = start + Duration::from_secs(301);
        let recovered = tracker.success(fresh, fresh).unwrap();
        assert_eq!(recovered.kind, ReportKind::Recovered);
        assert_eq!(recovered.total.successes, 2);
        assert_eq!(recovered.total.failures, 1);
        assert_eq!(recovered.episode_duration, Duration::from_secs(301));
        assert!(tracker.success(fresh, fresh).is_none());
        assert_eq!(
            tracker
                .failure(fresh, DiagnosticErrorKind::Transport)
                .unwrap()
                .kind,
            ReportKind::Degraded
        );
    }

    /// Scenario: Successes and changing failure categories alternate at high frequency.
    /// Guarantees: Intermittent delivery stays degraded and summaries retain successes and causes.
    #[test]
    fn intermittent_delivery_does_not_flap() {
        let mut tracker = DiagnosticTracker::default();
        let start = Instant::now();
        let _ = tracker.failure(start, DiagnosticErrorKind::Transport);
        for second in 1..60 {
            let now = start + Duration::from_secs(second);
            assert!(tracker.success(now, now).is_none());
            assert!(
                tracker
                    .failure(now, DiagnosticErrorKind::Rejected)
                    .is_none()
            );
        }
        let summary = tracker.success(start, start + SUMMARY_INTERVAL).unwrap();
        assert_eq!(summary.kind, ReportKind::Summary);
        assert_eq!(summary.interval.successes, 60);
        assert_eq!(summary.interval.failures, 59);
        let last_failure = start + Duration::from_secs(59);
        assert!(
            tracker
                .success(
                    last_failure + Duration::from_secs(1),
                    last_failure + Duration::from_secs(29)
                )
                .is_none()
        );
        assert_eq!(
            tracker
                .success(
                    last_failure + Duration::from_secs(1),
                    last_failure + RECOVERY_INTERVAL
                )
                .unwrap()
                .kind,
            ReportKind::Recovered
        );
    }

    /// Scenario: Signals, destinations, and exporter instances observe independent failures.
    /// Guarantees: Success on another scope cannot suppress a first failure or clear an episode.
    #[test]
    fn scopes_are_independent() {
        let now = Instant::now();
        let mut instances: [SignalDiagnostics<DiagnosticErrorKind>; 2] =
            std::array::from_fn(|_| SignalDiagnostics::default());
        for instance in &mut instances {
            for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
                assert_eq!(
                    instance
                        .signal(signal)
                        .failure(now, DiagnosticErrorKind::Rejected)
                        .unwrap()
                        .kind,
                    ReportKind::Degraded
                );
            }
        }
        let later = now + SUMMARY_INTERVAL;
        assert_eq!(
            instances[0]
                .signal(SignalType::Logs)
                .success(later, later)
                .unwrap()
                .kind,
            ReportKind::Recovered
        );
        assert!(
            instances[1]
                .signal(SignalType::Logs)
                .failure(now, DiagnosticErrorKind::Rejected)
                .is_none()
        );
        assert!(
            instances[0]
                .signal(SignalType::Metrics)
                .failure(now, DiagnosticErrorKind::Rejected)
                .is_none()
        );
    }

    /// Scenario: A new failure occurs exactly when recovery could otherwise be confirmed.
    /// Guarantees: Every failure restarts the confirmation interval, including a new error cause.
    #[test]
    fn failure_restarts_recovery_confirmation() {
        let start = Instant::now();
        let mut tracker = DiagnosticTracker::default();
        let _ = tracker.failure(start, DiagnosticErrorKind::Transport);
        let later = start + RECOVERY_INTERVAL;
        assert!(
            tracker
                .failure(later, DiagnosticErrorKind::Rejected)
                .is_none()
        );
        assert!(
            tracker
                .success(later, later + RECOVERY_INTERVAL)
                .is_some_and(|r| r.kind == ReportKind::Summary)
        );
        assert_eq!(
            tracker
                .success(later + Duration::from_secs(1), later + RECOVERY_INTERVAL)
                .unwrap()
                .kind,
            ReportKind::Recovered
        );
    }
}
