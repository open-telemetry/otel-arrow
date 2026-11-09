// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{DiagnosticReport, DiagnosticTracker, ReportKind, SignalSet};
use crate::attributes::AttributeEnum;
use crate::internal_events::{Logger, Sampler};
use crate::self_tracing::encoder::DirectFieldVisitor;
use crate::self_tracing::{LogContext, LogRecord};
use crate::tracing_init::StructuredLogEmitter;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::otlp::common::{BoundedBuf, ProtoBuffer};
use std::time::{Instant, SystemTime};
use tracing::{Dispatch, Event, Metadata};

/// Maximum encoded body/attributes of a diagnostic, including its nested sample.
pub const DIAGNOSTIC_RECORD_LIMIT: usize = 2048;

/// Periodic failure summaries, without retained records or recovery state.
#[derive(Debug)]
pub struct IntervalSampler<E> {
    tracker: DiagnosticTracker<E>,
    emitter: StructuredLogEmitter,
}

impl<E: AttributeEnum> IntervalSampler<E> {
    /// Creates a sampler using the engine's structured sink.
    #[must_use]
    pub fn new(emitter: StructuredLogEmitter) -> Self {
        Self {
            tracker: DiagnosticTracker::default(),
            emitter,
        }
    }

    /// Observes a failure and returns the sampler for its warning callsite.
    pub fn failure(&mut self, now: Instant, category: E) -> DiagnosticEmission<'_, E> {
        DiagnosticEmission {
            report: self.tracker.failure(now, category),
            sample: Sample::None,
            emitter: &self.emitter,
            signal: None,
        }
    }
}

impl<E: AttributeEnum> SignalSet<IntervalSampler<E>> {
    /// Observes a failure now and includes its signal in the selected record.
    pub fn logger(&mut self, signal: SignalType, category: E) -> DiagnosticEmission<'_, E> {
        self.signal(signal)
            .failure(Instant::now(), category)
            .with_signal(signal)
    }
}

/// Delivery episodes with independently filtered WARN and INFO records.
///
/// Call `observe` before either logging macro, including when INFO is disabled.
/// Only the episode's first failure event is retained if emitted. Recovery
/// releases it even when INFO is disabled.
#[derive(Debug)]
pub struct EpisodeSampler<E> {
    tracker: DiagnosticTracker<E>,
    start: Option<LogRecord>,
    emitter: StructuredLogEmitter,
}

impl<E: AttributeEnum> EpisodeSampler<E> {
    /// Creates an episode sampler using the engine's structured sink.
    #[must_use]
    pub fn new(emitter: StructuredLogEmitter) -> Self {
        Self {
            tracker: DiagnosticTracker::default(),
            start: None,
            emitter,
        }
    }

    /// Observes one completion independently of runtime log filtering.
    pub fn observe(
        &mut self,
        result: Result<(), E>,
        started_at: Instant,
        now: Instant,
    ) -> DiagnosticEmission<'_, E> {
        let report = match result {
            Ok(()) => self.tracker.success(started_at, now),
            Err(category) => self.tracker.failure(now, category),
        };
        let sample = match (result.is_err(), report.as_ref().map(|report| report.kind)) {
            (_, Some(ReportKind::Recovered)) => Sample::End(self.start.take()),
            (false, Some(ReportKind::Summary)) => Sample::Replay(self.start.as_ref()),
            (true, Some(ReportKind::Degraded)) => Sample::Start(&mut self.start),
            _ => Sample::None,
        };
        DiagnosticEmission {
            report,
            sample,
            emitter: &self.emitter,
            signal: None,
        }
    }
}

impl<E: AttributeEnum> SignalSet<EpisodeSampler<E>> {
    /// Observes a completion now, before either log call, and carries its signal.
    pub fn observe(
        &mut self,
        signal: SignalType,
        result: Result<(), E>,
        started_at: Instant,
    ) -> DiagnosticEmission<'_, E> {
        self.signal(signal)
            .observe(result, started_at, Instant::now())
            .with_signal(signal)
    }
}

// Own the closing sample inline instead of allocating another box on recovery.
#[allow(variant_size_differences)]
enum Sample<'a> {
    None,
    Start(&'a mut Option<LogRecord>),
    Replay(Option<&'a LogRecord>),
    End(Option<LogRecord>),
}

/// A precomputed observation handed to an ordinary scoped logging macro.
///
/// Dropping it when that macro is disabled does not undo the observation.
pub struct DiagnosticEmission<'a, E> {
    report: Option<DiagnosticReport<E>>,
    sample: Sample<'a>,
    emitter: &'a StructuredLogEmitter,
    signal: Option<SignalType>,
}

impl<E: AttributeEnum> DiagnosticEmission<'_, E> {
    fn with_signal(mut self, signal: SignalType) -> Self {
        self.signal = Some(signal);
        self
    }

    /// Counter snapshot selected by the observation, if any.
    pub const fn report(&self) -> Option<&DiagnosticReport<E>> {
        self.report.as_ref()
    }

    /// Selects the INFO callsite; other observations use the WARN callsite.
    pub fn is_recovery(&self) -> bool {
        self.report
            .as_ref()
            .is_some_and(|r| r.kind == ReportKind::Recovered)
    }

    /// Encodes one selected diagnostic. Exposed separately for sink-free benchmarks.
    ///
    /// The first half of the budget holds ordinary fields; the remainder holds
    /// counters and the optional nested sample. The saved prefix shares this
    /// allocation, and end records traverse it through borrowed pdata views.
    pub fn encode(&mut self, event: &Event<'_>, context: LogContext) -> LogRecord {
        self.encode_with(event, context, |visitor, saved| {
            visitor.record_log_record("episode.start_event", saved);
        })
    }

    /// Benchmark hook for comparing snapshot representations with identical episode work.
    #[cfg(feature = "testing")]
    pub fn encode_with_snapshot(
        &mut self,
        event: &Event<'_>,
        context: LogContext,
        snapshot: impl FnOnce(&mut DirectFieldVisitor<'_, ProtoBuffer>, &LogRecord),
    ) -> LogRecord {
        self.encode_with(event, context, snapshot)
    }

    fn encode_with(
        &mut self,
        event: &Event<'_>,
        context: LogContext,
        snapshot: impl FnOnce(&mut DirectFieldVisitor<'_, ProtoBuffer>, &LogRecord),
    ) -> LogRecord {
        let report = self
            .report
            .as_ref()
            .expect("only selected observations are encoded");
        let mut buf =
            ProtoBuffer::with_capacity_and_limit(DIAGNOSTIC_RECORD_LIMIT, DIAGNOSTIC_RECORD_LIMIT);
        let mut dropped = 0;
        match &self.sample {
            Sample::Replay(Some(saved)) => {
                buf.extend_from_slice(&saved.body_attrs_bytes)
                    .expect("retained prefix fits the diagnostic budget");
                dropped = u32::from(saved.dropped_attributes_count);
            }
            _ => {
                buf.with_max_remaining(DIAGNOSTIC_RECORD_LIMIT / 2, |buf| {
                    let mut visitor = DirectFieldVisitor::new(buf);
                    if let Some(signal) = self.signal {
                        visitor.write_str("signal", signal.as_str());
                    }
                    event.record(&mut visitor);
                    dropped = visitor.dropped_count();
                });
            }
        }
        let base_len = buf.len();
        let base_dropped = dropped;
        {
            let mut visitor = DirectFieldVisitor::new(&mut buf);
            visitor.write_str("diagnostic_kind", report.kind.as_str());
            visitor.write_f64("episode_seconds", report.episode_duration.as_secs_f64());
            visitor.write_f64("interval_seconds", report.interval_duration.as_secs_f64());
            visitor.write_u64("successful_attempts", report.interval.successes);
            visitor.write_u64("failed_attempts", report.interval.failures);
            visitor.write_u64("suppressed_diagnostics", report.interval.suppressed);
            visitor.write_u64("total_successful_attempts", report.total.successes);
            visitor.write_u64("total_failed_attempts", report.total.failures);
            visitor.write_u64("total_suppressed_diagnostics", report.total.suppressed);
            visitor.write_display("error_counts", &report.interval);
            visitor.write_display("total_error_counts", &report.total);
            if let Sample::End(Some(saved)) = &self.sample {
                snapshot(&mut visitor, saved);
            }
            dropped = dropped.saturating_add(visitor.dropped_count());
        }
        let bytes = buf.into_bytes();
        if let Sample::Start(saved) = &mut self.sample
            && saved.is_none()
        {
            **saved = Some(LogRecord {
                callsite_id: event.metadata().callsite(),
                body_attrs_bytes: bytes.slice(..base_len),
                dropped_attributes_count: base_dropped.min(u32::from(u16::MAX)) as u16,
                context: context.clone(),
            });
        }
        let (callsite_id, context) = match &self.sample {
            Sample::Replay(Some(saved)) => (saved.callsite_id.clone(), saved.context.clone()),
            _ => (event.metadata().callsite(), context),
        };
        LogRecord {
            callsite_id,
            body_attrs_bytes: bytes,
            dropped_attributes_count: dropped.min(u32::from(u16::MAX)) as u16,
            context,
        }
    }
}

impl<E: AttributeEnum> Sampler for DiagnosticEmission<'_, E> {
    fn should_sample(&mut self, _metadata: &Metadata<'_>) -> bool {
        self.report.is_some() && !matches!(self.sample, Sample::Replay(None))
    }
}

impl<E: AttributeEnum> Logger for DiagnosticEmission<'_, E> {
    fn emit(&mut self, event: &Event<'_>, _dispatch: &Dispatch) {
        let record = self.encode(event, self.emitter.context());
        self.emitter.deliver(SystemTime::now(), record);
    }
}
