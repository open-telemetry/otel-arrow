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
use crate::self_tracing::encoder::DirectFieldVisitor;
use crate::self_tracing::{LOG_ARGUMENTS_ENCODE_INLINE, LogRecord};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::otlp::common::{BoundedBuf, ProtoBuffer};
use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogRecord;
use otel_arrow_dfe_pdata_views::views::common::AnyValueView;
use otel_arrow_dfe_pdata_views::views::logs::LogRecordView;
use std::fmt;
use std::marker::PhantomData;
use std::time::{Duration, Instant, SystemTime};

/// Minimum spacing between summaries of an ongoing episode.
pub const SUMMARY_INTERVAL: Duration = Duration::from_secs(60);
/// Failure-free interval required before fresh success clears an episode.
pub const RECOVERY_INTERVAL: Duration = Duration::from_secs(30);
/// Additional bounded space reserved for report counters appended to a
/// retained failure record.
///
/// These reports occur at most once per summary interval, so growing this
/// temporary heap buffer is preferable to dropping the diagnostic counters.
const REPORT_ATTRIBUTES_LIMIT: usize = 2 * 1024;
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

/// Snapshot to emit through `otel_diagnostic_report!` at the component callsite.
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
    /// Representative failure detail, captured only when reporting.
    pub detail: LogRecord,
    /// Age of the representative detail; success-triggered reports reuse it.
    pub detail_age: Duration,
}

impl<E> DiagnosticReport<E> {
    /// Decode `detail`'s body as text, via the same OTLP bytes view/decoder
    /// path used to render any other `LogRecord`.
    ///
    /// Returns an owned `String` because [`LogRecordView::body`]'s borrow is
    /// tied to a view constructed from `self.detail`, not to `self` itself.
    #[must_use]
    pub fn detail_str(&self) -> String {
        let raw = RawLogRecord::new(&self.detail.body_attrs_bytes);
        if let Some(body) = raw.body()
            && let Some(bytes) = body.as_string()
            && let Ok(text) = std::str::from_utf8(bytes)
        {
            return text.to_owned();
        }
        String::new()
    }
}

#[derive(Debug)]
struct Episode<E> {
    started_at: Instant,
    last_failure: Instant,
    last_report: Instant,
    interval: Counts<E>,
    total: Counts<E>,
    detail: Option<LogRecord>,
    detail_at: Instant,
}

impl<E: AttributeEnum> Episode<E> {
    fn report(&mut self, kind: ReportKind, now: Instant) -> DiagnosticReport<E> {
        let report = DiagnosticReport {
            kind,
            episode_duration: now.saturating_duration_since(self.started_at),
            interval_duration: now.saturating_duration_since(self.last_report),
            interval: self.interval.clone(),
            total: self.total.clone(),
            detail: self
                .detail
                .clone()
                .expect("an episode always has a detail by its first report"),
            detail_age: now.saturating_duration_since(self.detail_at),
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
    /// Observe a failure. `detail` is evaluated only when a report is due and
    /// returns the ordinary log record captured under the active tracing
    /// filter/context, or `None` when that callsite is disabled.
    /// Callers retain responsibility for redacting secrets from diagnostics.
    pub fn failure(
        &mut self,
        now: Instant,
        category: E,
        detail: impl FnOnce() -> Option<LogRecord>,
    ) -> Option<DiagnosticReport<E>> {
        let episode = self.episode.get_or_insert_with(|| Episode {
            started_at: now,
            last_failure: now,
            last_report: now,
            interval: Counts::default(),
            total: Counts::default(),
            detail: None,
            detail_at: now,
        });
        let first_report = episode.detail.is_none();
        let report_due =
            first_report || now.saturating_duration_since(episode.last_report) >= SUMMARY_INTERVAL;
        let selected_detail = report_due.then(detail).flatten();
        let emit = selected_detail.is_some();
        episode.last_failure = now;
        episode.interval.failure(category, !emit);
        episode.total.failure(category, !emit);
        if !emit {
            return None;
        }
        episode.detail = selected_detail;
        episode.detail_at = now;
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
            let report = episode
                .detail
                .is_some()
                .then(|| episode.report(ReportKind::Recovered, now));
            self.episode = None;
            return report;
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

/// Capture an ordinary WARN record at the true failure callsite, for use as a
/// [`DiagnosticTracker::failure`] `detail` closure's return value.
///
/// This uses the active tracing dispatch's normal filtering decision, entity
/// context, callsite metadata, and `LogRecord` encoding. It differs from
/// [`crate::otel_warn!`] only in withholding delivery so the diagnostic
/// tracker can apply suppression and summary policy first.
#[macro_export]
macro_rules! otel_diagnostic_warn {
    (target: $target:expr, $name:literal $(, $($fields:tt)*)?) => {{
        use $crate::_private::Callsite;

        const _: () = $crate::_private::validate_event_name($name);

        static __CALLSITE: $crate::_private::DefaultCallsite = $crate::_private::callsite2! {
            name: $name,
            kind: $crate::_private::Kind::EVENT,
            target: $target,
            level: $crate::_private::Level::WARN,
            fields: $($($fields)*)?
        };

        let meta = __CALLSITE.metadata();

        (|valueset: $crate::_private::ValueSet<'_>| {
            let event = $crate::_private::Event::new(meta, &valueset);
            $crate::tracing_init::capture_current_event(&event)
        })($crate::_private::valueset!(meta.fields(), $($($fields)*)?))
    }};
    ($name:literal $(, $($fields:tt)*)?) => {{
        $crate::otel_diagnostic_warn!(
            target: env!("CARGO_PKG_NAME"),
            $name
            $(, $($fields)*)?
        )
    }};
}

/// Splice additional pre-encoded attributes onto a copy of `detail`'s body and
/// attribute bytes, keeping `detail`'s own callsite (so the re-delivered
/// record still identifies the true failure site's name/level/file/line).
///
/// Protobuf's repeated fields are just concatenated entries, so appending
/// more `KeyValue` attributes after an existing, already-encoded body is
/// valid without decoding anything. The existing bytes are copied first, so
/// the representative detail is never at risk from this call's own
/// truncation; only the newly appended attributes can be truncated or
/// dropped if they overflow their own budget.
fn append_attrs(detail: &LogRecord, attrs: &tracing::Event<'_>) -> LogRecord {
    let existing = detail.body_attrs_bytes.len();
    let mut buf = ProtoBuffer::with_capacity_and_limit(
        existing + LOG_ARGUMENTS_ENCODE_INLINE,
        existing + REPORT_ATTRIBUTES_LIMIT,
    );
    let _ = buf.extend_from_slice(&detail.body_attrs_bytes);
    let mut dropped = u32::from(detail.dropped_attributes_count);
    {
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        attrs.record(&mut visitor);
        dropped += visitor.dropped_count();
    }
    LogRecord {
        callsite_id: detail.callsite_id.clone(),
        body_attrs_bytes: buf.into_bytes(),
        dropped_attributes_count: dropped.min(u32::from(u16::MAX)) as u16,
        context: detail.context.clone(),
    }
}

/// Re-deliver a diagnostic's representative detail directly, with report
/// counters appended, instead of decoding it to text and re-logging it as a
/// field of a brand-new event.
///
/// Filtering and entity context were captured with `detail` at the original
/// failure site. They are not re-evaluated here: this re-delivers an already
/// accepted occurrence, not a new one. `time` is always "now" (the delivery
/// moment); the original capture age is represented by the
/// `error_sample_age_seconds` attribute already included by the caller.
///
/// Only meaningful for [`ReportKind::Degraded`] and [`ReportKind::Summary`]:
/// recovery is a distinct, new occurrence (and is usually logged at a lower
/// severity), so it still uses [`otel_diagnostic_report!`] to build its own
/// event referencing the retained sample as supporting text.
///
pub fn emit_diagnostic_summary(detail: &LogRecord, attrs: &tracing::Event<'_>) {
    crate::tracing_init::deliver_current_record(SystemTime::now(), append_attrs(detail, attrs));
}

/// Build the report-counter fields for [`emit_diagnostic_summary`] and pass
/// them, together with the report's retained detail, to it.
///
/// The `report:` form re-delivers `report.detail` using its own captured
/// callsite, so only caller fields specific to the report's kind (e.g.
/// `retryable`, `diagnostic_kind`) need to be passed; `error`/`message`
/// duplicating the detail's body are neither needed nor accepted.
///
/// The `target:`/`emit:`/`name:` form emits a distinct ordinary event, used
/// for recovery reports.
#[macro_export]
macro_rules! otel_diagnostic_report {
    (report: $report:expr, $($fields:tt)+) => {{
        use $crate::_private::Callsite;

        let diagnostic_report = $report;

        static __CALLSITE: $crate::_private::DefaultCallsite = $crate::_private::callsite2! {
            name: "diagnostic.report.attrs",
            kind: $crate::_private::Kind::EVENT,
            target: "",
            level: $crate::_private::Level::TRACE,
            fields: $($fields)+,
                episode_seconds,
                interval_seconds,
                successful_attempts,
                failed_attempts,
                suppressed_diagnostics,
                total_successful_attempts,
                total_failed_attempts,
                total_suppressed_diagnostics,
                error_counts,
                total_error_counts,
                error_sample_age_seconds
        };
        let meta = __CALLSITE.metadata();

        // The IIFE keeps `valueset!`'s field temporaries (e.g. `f64`/`Counts`
        // by-reference values) alive across `Event::new` and the emit call,
        // matching the pattern `otel_diagnostic_warn!` uses for the same
        // reason: a separate `let valueset = ...;` statement would drop them
        // too early.
        (|valueset: $crate::_private::ValueSet<'_>| {
            let attrs_event = $crate::_private::Event::new(meta, &valueset);
            $crate::diagnostics::emit_diagnostic_summary(
                &diagnostic_report.detail,
                &attrs_event,
            );
        })($crate::_private::valueset!(meta.fields(),
            $($fields)+,
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
            error_sample_age_seconds = diagnostic_report.detail_age.as_secs_f64()
        ));
    }};
    (target: $target:expr, emit: $emit:ident, name: $name:literal, report: $report:expr, $($fields:tt)+) => {{
        let diagnostic_report = $report;
        $crate::$emit!(target: $target, $name,
            $($fields)+,
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
            error_sample_age_seconds = diagnostic_report.detail_age.as_secs_f64()
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
                    otel_diagnostic_warn!(
                        "test.diagnostic.detail",
                        message = "connection reset by peer"
                    )
                })
                .unwrap();
            otel_diagnostic_report!(
                report: &first,
                signal = "logs", retryable = true, diagnostic_kind = "first_failure"
            );

            let later = start + SUMMARY_INTERVAL;
            let summary = tracker
                .failure(later, DiagnosticErrorKind::Transport, || {
                    otel_diagnostic_warn!("test.diagnostic.detail", message = "unreachable")
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
