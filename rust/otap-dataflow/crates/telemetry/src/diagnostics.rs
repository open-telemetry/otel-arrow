// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded diagnostics for repeated operation failures and confirmed recovery.
//!
//! Each tracker observes one operation in a bounded scope local to a node/core.
//! Call [`DiagnosticTracker::failure`] and [`DiagnosticTracker::success`] when
//! successful completions provide meaningful recovery evidence for that scope.
//! Call only `failure` for bounded summaries without recovery, as for payload
//! rejection or notification errors. These observers retain their episode
//! totals for the lifetime of the tracker.
//!
//! Integrations own event names, error classifications, and observation
//! boundaries. Keep distinct operations in separate trackers; a successful
//! enqueue, for example, cannot establish recovery of a failing storage write.
//! [`SignalSet`] optionally groups state by telemetry signal, and
//! [`SignalDiagnostics`] specializes it for diagnostic trackers.
//!
//! A tracker has no timers, locks, or metric-interest dependency. Only emitted
//! reports allocate or format diagnostic text after an episode has started.

use crate::attributes::AttributeEnum;
use otel_arrow_dfe_config::SignalType;
use std::fmt;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

/// Minimum spacing between summaries of an ongoing episode.
pub const SUMMARY_INTERVAL: Duration = Duration::from_secs(60);
/// Failure-free interval required before fresh success clears an episode.
pub const RECOVERY_INTERVAL: Duration = Duration::from_secs(30);
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
    /// Stable log attribute value for this sampling decision.
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

/// Snapshot selected by the diagnostic sampler for an ordinary log event.
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
    reported: bool,
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
        self.reported = true;
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
    /// Observe a failure and return a report only when an ordinary warning
    /// should be emitted.
    pub fn failure(&mut self, now: Instant, category: E) -> Option<DiagnosticReport<E>> {
        let episode = self.episode.get_or_insert_with(|| Episode {
            started_at: now,
            last_failure: now,
            last_report: now,
            interval: Counts::default(),
            total: Counts::default(),
            reported: false,
        });
        let first_report = !episode.reported;
        let emit =
            first_report || now.saturating_duration_since(episode.last_report) >= SUMMARY_INTERVAL;
        episode.last_failure = now;
        episode.interval.failure(category, !emit);
        episode.total.failure(category, !emit);
        if !emit {
            return None;
        }
        Some(episode.report(
            if first_report {
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
            let report = Some(episode.report(ReportKind::Recovered, now));
            self.episode = None;
            return report;
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

/// Sample repeated failures and emit one ordinary warning when selected.
///
/// The tracker decides before the tracing event is constructed. The selected
/// event follows the normal logging path, with diagnostic counters inserted
/// before the caller's ordinary tracing fields and message.
#[macro_export]
macro_rules! otel_summary_warn {
    (target: $target:expr, $tracker:expr, $signal:expr, $category:expr, $name:literal, $($fields:tt)+) => {{
        $crate::otel_summary_warn!(
            target: $target,
            at: std::time::Instant::now(),
            $tracker,
            $signal,
            $category,
            $name,
            $($fields)+
        );
    }};
    (target: $target:expr, at: $now:expr, $tracker:expr, $signal:expr, $category:expr, $name:literal, $($fields:tt)+) => {{
        let diagnostic_signal = $signal;
        if let Some(diagnostic_report) = ($tracker).failure($now, $category) {
            $crate::otel_warn!(target: $target, $name,
                signal = diagnostic_signal.as_str(),
                diagnostic_kind = diagnostic_report.kind.as_str(),
                episode_seconds = diagnostic_report.episode_duration.as_secs_f64(),
                interval_seconds = diagnostic_report.interval_duration.as_secs_f64(),
                successful_attempts = diagnostic_report.interval.successes,
                failed_attempts = diagnostic_report.interval.failures,
                suppressed_diagnostics = diagnostic_report.interval.suppressed,
                total_successful_attempts = diagnostic_report.total.successes,
                total_failed_attempts = diagnostic_report.total.failures,
                total_suppressed_diagnostics = diagnostic_report.total.suppressed,
                error_counts = %diagnostic_report.interval,
                total_error_counts = %diagnostic_report.total,
                $($fields)+
            );
        }
    }};
}

/// Add diagnostic episode counters to a distinct ordinary event, such as a
/// confirmed recovery notification.
#[macro_export]
macro_rules! otel_diagnostic_report {
    (target: $target:expr, emit: $emit:ident, name: $name:literal, report: $report:expr, $($fields:tt)+) => {{
        let diagnostic_report = $report;
        $crate::$emit!(target: $target, $name,
            diagnostic_kind = diagnostic_report.kind.as_str(),
            episode_seconds = diagnostic_report.episode_duration.as_secs_f64(),
            interval_seconds = diagnostic_report.interval_duration.as_secs_f64(),
            successful_attempts = diagnostic_report.interval.successes,
            failed_attempts = diagnostic_report.interval.failures,
            suppressed_diagnostics = diagnostic_report.interval.suppressed,
            total_successful_attempts = diagnostic_report.total.successes,
            total_failed_attempts = diagnostic_report.total.failures,
            total_suppressed_diagnostics = diagnostic_report.total.suppressed,
            error_counts = %diagnostic_report.interval,
            total_error_counts = %diagnostic_report.total,
            $($fields)+
        );
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a test detail `LogRecord` whose body is `message`.
    fn detail(message: &str) -> Option<LogRecord> {
        Some(
            crate::__log_record_impl!(
                crate::_private::Level::WARN,
                "test.diagnostic.detail",
                message = %message
            )
            .into_record(crate::self_tracing::LogContext::new()),
        )
    }

    /// Scenario: A destination fails 100,000 times within each reporting window.
    /// Guarantees: Warnings and formatting are time bounded while all failures are counted.
    #[test]
    fn sustained_failure_is_bounded_and_counted() {
        let mut tracker = DiagnosticTracker::default();
        let start = Instant::now();
        let first = tracker
            .failure(start, DiagnosticErrorKind::Transport, || {
                detail("DNS unavailable")
            })
            .unwrap();
        assert_eq!(first.kind, ReportKind::Degraded);
        assert_eq!(first.total.failures, 1);
        for _ in 0..100_000 {
            assert!(
                tracker
                    .failure(
                        start + Duration::from_secs(1),
                        DiagnosticErrorKind::Transport,
                        || -> Option<LogRecord> {
                            panic!("suppressed diagnostics must not be formatted")
                        }
                    )
                    .is_none()
            );
        }
        let summary = tracker
            .failure(
                start + SUMMARY_INTERVAL,
                DiagnosticErrorKind::Rejected,
                || detail("503"),
            )
            .unwrap();
        assert_eq!(summary.kind, ReportKind::Summary);
        assert_eq!(summary.interval.failures, 100_001);
        assert_eq!(summary.total.failures, 100_002);
        assert_eq!(summary.interval.suppressed, 100_000);
        assert_eq!(summary.interval.to_string(), "transport=100000,rejected=1");
        assert_eq!(summary.detail_str(), "503");
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
                .failure(start, DiagnosticErrorKind::Transport, || detail("offline"))
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
                .failure(fresh, DiagnosticErrorKind::Transport, || {
                    detail("offline again")
                })
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
        let _ = tracker.failure(start, DiagnosticErrorKind::Transport, || detail("offline"));
        for second in 1..60 {
            let now = start + Duration::from_secs(second);
            assert!(tracker.success(now, now).is_none());
            assert!(
                tracker
                    .failure(now, DiagnosticErrorKind::Rejected, || detail("rejected"))
                    .is_none()
            );
        }
        let summary = tracker.success(start, start + SUMMARY_INTERVAL).unwrap();
        assert_eq!(summary.kind, ReportKind::Summary);
        assert_eq!(summary.interval.successes, 60);
        assert_eq!(summary.interval.failures, 59);
        assert_eq!(summary.detail_age, SUMMARY_INTERVAL);
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
                        .failure(now, DiagnosticErrorKind::Rejected, || detail("denied"))
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
                .failure(now, DiagnosticErrorKind::Rejected, || detail("denied"))
                .is_none()
        );
        assert!(
            instances[0]
                .signal(SignalType::Metrics)
                .failure(now, DiagnosticErrorKind::Rejected, || detail("denied"))
                .is_none()
        );
    }

    /// Scenario: A failure supplies detail text longer than the encode growth limit.
    /// Guarantees: The retained detail is safely truncated with a `[...]` suffix, not silently dropped.
    #[test]
    fn representative_details_are_bounded() {
        let mut tracker = DiagnosticTracker::default();
        let text = "x".repeat(LOG_ARGUMENTS_ENCODE_INLINE * 2);
        let report = tracker
            .failure(Instant::now(), DiagnosticErrorKind::Other, || detail(&text))
            .unwrap();
        assert!(report.detail_str().len() < text.len());
        assert!(report.detail_str().ends_with("[...]"));
    }

    /// Scenario: A new failure occurs exactly when recovery could otherwise be confirmed.
    /// Guarantees: Every failure restarts the confirmation interval, including a new error cause.
    #[test]
    fn failure_restarts_recovery_confirmation() {
        let start = Instant::now();
        let mut tracker = DiagnosticTracker::default();
        let _ = tracker.failure(start, DiagnosticErrorKind::Transport, || detail("offline"));
        let later = start + RECOVERY_INTERVAL;
        assert!(
            tracker
                .failure(later, DiagnosticErrorKind::Rejected, || detail("denied"))
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

    /// Scenario: A first failure and a later summary are delivered via
    /// `otel_diagnostic_report!`, which re-dispatches the retained detail
    /// directly (splicing on report counters) instead of re-logging decoded
    /// text as a field of a new event.
    /// Guarantees: Both deliveries render through the same console path as
    /// any other log record, preserving the original failure's file/line and
    /// the `error_sample_age_seconds` relative age, never the stale capture
    /// time as the event's own timestamp.
    #[test]
    fn example_episode_start_and_summary_messages() {
        use crate::event::{LogEvent, ObservedEvent, ObservedEventReporter};
        use crate::self_tracing::format_log_record_to_string;
        use crate::tracing_init::{ProviderSetup, TracingSetup};
        use otel_arrow_dfe_config::observed_state::SendPolicy;
        use otel_arrow_dfe_config::settings::telemetry::logs::LogLevel;

        let (sender, receiver) = flume::unbounded();
        let reporter = ObservedEventReporter::new(SendPolicy::default(), sender);
        let setup = TracingSetup::new(
            ProviderSetup::InternalAsync { reporter },
            LogLevel::default(),
            crate::self_tracing::LogContext::new,
        );

        let rendered: Vec<String> = setup.with_subscriber(|| {
            let mut tracker = DiagnosticTracker::default();
            let start = Instant::now();
            let first = tracker
                .failure(start, DiagnosticErrorKind::Transport, || {
                    crate::tracing_init::capture_current_record(crate::__log_record_impl!(
                        crate::Level::WARN,
                        "test.diagnostic.detail",
                        message = "connection reset by peer"
                    ))
                })
                .unwrap();
            otel_diagnostic_report!(
                report: &first,
                signal = "logs", retryable = true, diagnostic_kind = "first_failure"
            );

            let later = start + SUMMARY_INTERVAL;
            let summary = tracker
                .failure(later, DiagnosticErrorKind::Transport, || {
                    crate::tracing_init::capture_current_record(crate::__log_record_impl!(
                        crate::Level::WARN,
                        "test.diagnostic.detail",
                        message = "unreachable"
                    ))
                })
                .unwrap();
            otel_diagnostic_report!(
                report: &summary,
                signal = "logs", retryable = true, diagnostic_kind = "summary"
            );

            receiver
                .drain()
                .map(|event| match event {
                    ObservedEvent::Log(LogEvent { time, record }) => {
                        format_log_record_to_string(Some(time), &record)
                    }
                    ObservedEvent::Engine(_) => {
                        unreachable!("only log events are emitted here")
                    }
                })
                .collect()
        });
        assert_eq!(rendered.len(), 2);
        for line in &rendered {
            eprintln!("{line}");
        }
    }
}
