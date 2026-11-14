// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Worker-local OTAP construction. Delivery authorization and checkpoint I/O stay external.

use std::{rc::Rc, time::Instant};

use hashbrown::HashMap;
use otel_arrow_dfe_filelog_checkpoint as cp;
use otel_arrow_dfe_pdata::{
    encode::record::{
        attributes::{KnownKey, KnownKeyArrayBuilder, StrKeysAttributesRecordBatchBuilder},
        logs::LogsRecordBatchBuilder,
    },
    otap::{Logs, OtapArrowRecords},
    proto::opentelemetry::{arrow::v1::ArrowPayloadType, common::v1::any_value::Value},
};
use sha2::{Digest, Sha256};

use super::{
    config::ValidatedInputs,
    decoder::SourceRange,
    framer::{LineBody, LineEnding, LineFrame},
    metadata::FileMetadataAttributes,
    multiline::{RecordEnding, RecordFrame},
};

/// Conservative per-record overhead in the logical admission model, not Arrow bytes.
pub const RECORD_OVERHEAD: u64 = 128;
/// One progress-only WAL transaction must cover the entire batch.
pub const MAX_DISTINCT_FILES: usize = cp::WAL_MAX_OPS_PER_TX as usize;

const FRAGMENT_KEYS: [&str; 7] = [
    "otel.arrow.filelog.fragment.id",
    "otel.arrow.filelog.fragment.index",
    "otel.arrow.filelog.fragment.is_last",
    "otel.arrow.filelog.fragment.body.start",
    "otel.arrow.filelog.fragment.body.end",
    "otel.arrow.filelog.fragment.frame.start",
    "otel.arrow.filelog.fragment.frame.end",
];
const TERMINAL_KEY: &str = "otel.arrow.filelog.terminal_unterminated";
const ATTRIBUTE_KEYS: [&str; 14] = [
    "log.file.path",
    "log.file.name",
    "otel.arrow.filelog.path.kind",
    "otel.arrow.filelog.path.native",
    "otel.arrow.filelog.path.truncated",
    "otel.arrow.filelog.path.sha256",
    FRAGMENT_KEYS[0],
    FRAGMENT_KEYS[1],
    FRAGMENT_KEYS[2],
    FRAGMENT_KEYS[3],
    FRAGMENT_KEYS[4],
    FRAGMENT_KEYS[5],
    FRAGMENT_KEYS[6],
    TERMINAL_KEY,
];

/// Common sizing arithmetic for configuration admission and actual projection.
pub(super) fn logical_size(body_bytes: u64, attribute_bytes: u64) -> Option<u64> {
    body_bytes
        .checked_add(attribute_bytes)?
        .checked_add(RECORD_OVERHEAD)
}

/// Owned primitive output, without any checkpoint authorization.
#[derive(Debug)]
pub enum Frame {
    /// One physical line or fragment.
    Line(LineFrame),
    /// One grouped record or fragment.
    Multiline(RecordFrame),
}

impl Frame {
    /// Source bytes owned by this output, including delimiters and discarded tails.
    #[must_use]
    pub fn range(&self) -> SourceRange {
        match self {
            Self::Line(f) => f.frame_range,
            Self::Multiline(f) => f.frame_range,
        }
    }

    /// Whether this output finishes its logical record.
    #[must_use]
    pub fn complete_record(&self) -> bool {
        match self {
            Self::Line(f) => f.fragment.is_none_or(|f| f.is_last),
            Self::Multiline(f) => f.fragment.is_none_or(|f| f.is_last),
        }
    }

    /// Heap payload retained by the frame, including a preserve-raw shadow.
    #[must_use]
    pub fn retained_capacity(&self) -> usize {
        let source = match self {
            Self::Line(f) => f.source_body.as_ref().map_or(0, Vec::capacity),
            Self::Multiline(_) => 0,
        };
        source
            + match self.body() {
                LineBody::Text(s) => s.capacity(),
                LineBody::Bytes(b) => b.capacity(),
            }
    }

    fn body(&self) -> &LineBody {
        match self {
            Self::Line(f) => &f.body,
            Self::Multiline(f) => &f.body,
        }
    }

    fn body_range(&self) -> SourceRange {
        match self {
            Self::Line(f) => f.body_range,
            Self::Multiline(f) => f.body_range,
        }
    }

    fn fragment(&self) -> Option<(u64, u32, bool)> {
        match self {
            Self::Line(f) => f
                .fragment
                .map(|f| (f.record_start_offset, f.index, f.is_last)),
            Self::Multiline(f) => f
                .fragment
                .map(|f| (f.record_start_offset, f.index, f.is_last)),
        }
    }

    fn terminal(&self) -> bool {
        match self {
            Self::Line(f) => f.ending == LineEnding::PermanentEof,
            Self::Multiline(f) => f.ending == RecordEnding::PermanentEof,
        }
    }

    pub(super) fn resume(&self) -> cp::FramingResume {
        match self {
            Self::Line(f) => f.continuation.map_or(cp::FramingResume::Clean, |c| {
                cp::FramingResume::Continuation {
                    record_start_offset: c.record_start_offset,
                    record_end_offset: 0,
                    next_fragment_index: c.next_fragment_index,
                }
            }),
            Self::Multiline(f) => f.continuation.map_or(cp::FramingResume::Clean, |c| {
                cp::FramingResume::Continuation {
                    record_start_offset: c.record_start_offset,
                    record_end_offset: c.record_end_offset,
                    next_fragment_index: c.next_fragment_index,
                }
            }),
        }
    }
}

/// Exact framed contribution. Metadata is immutable and shared only within this worker.
#[derive(Debug)]
pub struct Record {
    /// Durable source identity, never inferred from exported attributes.
    pub file_id: cp::FileId,
    /// Source epoch supplied by identity admission.
    pub epoch: u32,
    /// Original primitive output, including ranges and continuation.
    pub frame: Frame,
    /// Evidence at the frame's resulting frontier.
    pub guard: cp::CommittedFrontierGuard,
    /// Wall-clock timestamp captured when this frame becomes ready.
    pub observed_time_unix_nano: u64,
    /// Selected metadata at acceptance; later path updates cannot rewrite it.
    pub metadata: Rc<FileMetadataAttributes>,
}

impl Record {
    /// Exact logical projected size, independent of Arrow allocation and wire size.
    pub fn logical_size(&self) -> Result<u64, BatchError> {
        let body = match self.frame.body() {
            LineBody::Text(s) => s.len(),
            LineBody::Bytes(b) => b.len(),
        };
        let mut attrs = metadata_size(&self.metadata);
        if self.frame.fragment().is_some() {
            for (key, bytes) in FRAGMENT_KEYS.iter().zip([64, 8, 1, 8, 8, 8, 8]) {
                attrs = attrs
                    .checked_add(key.len() as u64 + bytes)
                    .ok_or(BatchError::RecordSize)?;
            }
        }
        if self.frame.terminal() {
            attrs = attrs
                .checked_add(TERMINAL_KEY.len() as u64 + 1)
                .ok_or(BatchError::RecordSize)?;
        }
        logical_size(body as u64, attrs).ok_or(BatchError::RecordSize)
    }

    fn validate(&self) -> Result<(), BatchError> {
        let range = self.frame.range();
        let body = self.frame.body_range();
        if range.start >= range.end
            || body.start < range.start
            || body.end > range.end
            || body.start > body.end
            || self.observed_time_unix_nano > i64::MAX as u64
        {
            return Err(BatchError::Progress);
        }
        if self.frame.fragment().is_some() && range.end > i64::MAX as u64 {
            return Err(BatchError::Progress);
        }
        if let Some((start, index, _)) = self.frame.fragment()
            && (start > range.start || (index == 0) != (start == range.start))
        {
            return Err(BatchError::Progress);
        }
        match (self.frame.fragment(), self.frame.resume()) {
            (
                Some((start, index, false)),
                cp::FramingResume::Continuation {
                    record_start_offset,
                    record_end_offset,
                    next_fragment_index,
                },
            ) if start == record_start_offset
                && start <= range.start
                && index.checked_add(1) == Some(next_fragment_index)
                && (record_end_offset == 0 || range.end < record_end_offset) => {}
            (None | Some((_, _, true)), cp::FramingResume::Clean) => {}
            _ => return Err(BatchError::Progress),
        }
        Ok(())
    }
}

pub(super) fn follows(resume: cp::FramingResume, frame: &Frame) -> bool {
    match (resume, frame.fragment()) {
        (cp::FramingResume::Clean, None | Some((_, 0, _))) => true,
        (
            cp::FramingResume::Continuation {
                record_start_offset,
                record_end_offset,
                next_fragment_index,
            },
            Some((start, index, last)),
        ) => {
            start == record_start_offset
                && index == next_fragment_index
                && (record_end_offset == 0
                    || (frame.range().end <= record_end_offset
                        && last == (frame.range().end == record_end_offset)))
                && (last
                    || matches!(frame.resume(), cp::FramingResume::Continuation {
                    record_start_offset: next_start,
                    record_end_offset: next_end,
                    ..
                } if next_start == start && next_end == record_end_offset))
        }
        _ => false,
    }
}

pub(super) fn metadata_size(metadata: &FileMetadataAttributes) -> u64 {
    metadata
        .as_slice()
        .iter()
        .map(|(key, value)| {
            key.len() as u64
                + match value {
                    Value::StringValue(s) => s.len() as u64,
                    Value::BytesValue(b) => b.len() as u64,
                    Value::BoolValue(_) => 1,
                    _ => 8,
                }
        })
        .sum()
}

/// Accepted output may require the caller to stop immediately.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flow {
    /// More bounded work may be attempted.
    Continue,
    /// A sealed batch, carry-over, or terminal failure prevents further work.
    Stop,
}

/// Invariant violations fail closed, rather than silently dropping a record.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum BatchError {
    /// A record violates admitted size or arithmetic bounds.
    #[error("filelog record exceeds admitted bounds")]
    RecordSize,
    /// Ranges, epoch, continuation, or timestamp are inconsistent.
    #[error("filelog noncontiguous or inconsistent source progress")]
    Progress,
    /// A deadline cannot be represented.
    #[error("filelog batch deadline overflows")]
    Deadline,
    /// Work was offered while a batch was retained or after terminal failure.
    #[error("filelog batch admission is stopped")]
    Stopped,
    /// Release did not match the entire applied progress transaction.
    #[error("filelog batch release does not match applied progress")]
    Release,
    /// OTAP construction failed; no checkpoint progress is authorized.
    #[error("filelog OTAP construction failed: {0}")]
    Encoding(String),
}

/// A failed offer returns the exact owned record without reconstructing source bytes.
#[derive(Debug)]
pub struct RejectedRecord {
    /// Terminal reason.
    pub error: BatchError,
    /// Unaccepted source output.
    pub record: Record,
}

/// One immutable batch retained through delivery and checkpoint application.
pub struct Batch {
    /// OTAP payload; delivery may clone its shared Arrow buffers for an attempt.
    pub records: OtapArrowRecords,
    /// Exactly one update per represented file, with first expected and last resulting progress.
    pub progress: Vec<cp::UpdateProgress>,
    /// Source provenance in log-row order, independent of exported attributes.
    pub provenance: Vec<Provenance>,
    /// Number of log records, independent of distinct-file progress.
    pub record_count: usize,
    /// Logical size under the shared admission model.
    pub logical_bytes: u64,
}

/// Bounded internal evidence for one log row; never recovered from mutable attributes.
#[derive(Debug)]
pub struct Provenance {
    /// Durable source identity.
    pub file_id: cp::FileId,
    /// Source epoch.
    pub epoch: u32,
    /// Source bytes represented by the body.
    pub body_range: SourceRange,
    /// Complete source progress owned by this row.
    pub frame_range: SourceRange,
    /// Whether the configured oversize policy discarded body bytes.
    pub truncated: bool,
    /// Original source body bytes intentionally discarded.
    pub discarded_source_bytes: u64,
    /// Malformed source units, including a discarded tail.
    pub malformed_units: u64,
    /// Start-pattern fallback evidence, not reconstructed on recovery.
    pub pattern_not_matched: bool,
}

/// Explicit report when uncommitted in-memory capture is abandoned at shutdown.
#[derive(Debug, Eq, PartialEq)]
pub struct PendingCapture {
    /// Open/retained log records plus the unsent carry-over, if present.
    pub records: usize,
    /// Logical bytes not released by authorized checkpoint application.
    pub logical_bytes: u64,
}

/// One open OR retained batch and at most one exact, already-framed carry-over.
pub struct BatchBuilder {
    max_records: usize,
    max_bytes: u64,
    max_record_bytes: u64,
    max_body_bytes: usize,
    flush_period: std::time::Duration,
    logs: LogsRecordBatchBuilder,
    attrs: StrKeysAttributesRecordBatchBuilder<u16, KnownKeyArrayBuilder<14>>,
    progress: Vec<cp::UpdateProgress>,
    provenance: Vec<Provenance>,
    indices: HashMap<cp::FileId, usize>,
    count: usize,
    bytes: u64,
    deadline: Option<Instant>,
    retained: Option<Batch>,
    carry: Option<Record>,
    failure: Option<BatchError>,
}

impl BatchBuilder {
    /// Uses the same validated size model as configuration admission.
    #[must_use]
    pub fn new(inputs: &ValidatedInputs) -> Self {
        Self {
            max_records: inputs.settings().batch.max_records as usize,
            max_bytes: inputs.settings().batch.max_bytes,
            max_record_bytes: inputs.resources().record_bytes,
            max_body_bytes: if inputs.grouping().is_some() {
                inputs.line().max_record_bytes
            } else {
                inputs
                    .line()
                    .max_line_bytes
                    .min(inputs.line().max_record_bytes)
            },
            flush_period: inputs.settings().batch.max_flush_period,
            logs: LogsRecordBatchBuilder::new(),
            attrs: StrKeysAttributesRecordBatchBuilder::new_with_known_keys(&ATTRIBUTE_KEYS),
            progress: Vec::new(),
            provenance: Vec::new(),
            indices: HashMap::new(),
            count: 0,
            bytes: 0,
            deadline: None,
            retained: None,
            carry: None,
            failure: None,
        }
    }

    /// Nonempty sparse batches must be serviced at this deadline, even without ready readers.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Instant> {
        self.deadline
    }

    /// Terminal invariant failure, distinct from an ordinary delivery wait.
    #[must_use]
    pub fn failure(&self) -> Option<&BatchError> {
        self.failure.as_ref()
    }

    /// Borrows retained data without releasing ownership or authorizing another source turn.
    #[must_use]
    pub fn retained(&self) -> Option<&Batch> {
        self.retained.as_ref()
    }

    /// Includes open, retained and carry-over progress until release after application.
    #[must_use]
    pub fn unresolved(&self, file_id: cp::FileId, epoch: u32) -> bool {
        self.indices.get(&file_id).is_some_and(|&index| {
            let progress = self
                .retained
                .as_ref()
                .map_or(self.progress.as_slice(), |b| &b.progress);
            progress[index].expected_file_epoch == epoch
        }) || self
            .carry
            .as_ref()
            .is_some_and(|r| r.file_id == file_id && r.epoch == epoch)
    }

    /// Services the first-record deadline independently of source activity.
    pub fn poll(&mut self, now: Instant) -> Result<Flow, BatchError> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        if self.deadline.is_some_and(|at| at <= now) {
            self.seal()?;
        }
        Ok(if self.retained.is_some() {
            Flow::Stop
        } else {
            Flow::Continue
        })
    }

    /// Cheap known-bound check before reading or buffered framing.
    pub fn preflight(
        &mut self,
        file_id: cp::FileId,
        epoch: u32,
        minimum_logical_bytes: u64,
        now: Instant,
    ) -> Result<Flow, BatchError> {
        if self.poll(now)? == Flow::Stop {
            return Ok(Flow::Stop);
        }
        if minimum_logical_bytes > self.max_bytes {
            return self.fail(BatchError::RecordSize);
        }
        if let Some(&index) = self.indices.get(&file_id)
            && self.progress[index].expected_file_epoch != epoch
        {
            return self.fail(BatchError::Progress);
        }
        if self.count != 0
            && (self.count == self.max_records
                || (!self.indices.contains_key(&file_id)
                    && self.progress.len() == MAX_DISTINCT_FILES)
                || minimum_logical_bytes > self.max_bytes - self.bytes)
        {
            self.seal()?;
            return Ok(Flow::Stop);
        }
        Ok(Flow::Continue)
    }

    /// Accepts ownership on both success values, retaining at most one carry-over.
    #[allow(clippy::result_large_err)]
    pub fn offer(&mut self, record: Record, now: Instant) -> Result<Flow, RejectedRecord> {
        let result = (|| {
            if self.failure.is_some() || self.retained.is_some() || self.carry.is_some() {
                return Err(BatchError::Stopped);
            }
            record.validate()?;
            let size = record.logical_size()?;
            let body_len = match record.frame.body() {
                LineBody::Text(s) => s.len(),
                LineBody::Bytes(b) => b.len(),
            };
            if size > self.max_record_bytes
                || size > self.max_bytes
                || body_len > self.max_body_bytes
            {
                return Err(BatchError::RecordSize);
            }
            if let Some(&index) = self.indices.get(&record.file_id) {
                let p = &self.progress[index];
                if p.expected_file_epoch != record.epoch
                    || p.new_committed_offset != record.frame.range().start
                    || !follows(p.new_framing_resume, &record.frame)
                {
                    return Err(BatchError::Progress);
                }
            }
            // Validate the next first-record deadline before changing any builder.
            let _ = now
                .checked_add(self.flush_period)
                .ok_or(BatchError::Deadline)?;
            Ok(size)
        })();
        let size = match result {
            Ok(size) => size,
            Err(error) => {
                self.failure = Some(error.clone());
                return Err(RejectedRecord { error, record });
            }
        };
        let full = self.count != 0
            && (size > self.max_bytes - self.bytes
                || self.count == self.max_records
                || (!self.indices.contains_key(&record.file_id)
                    && self.progress.len() == MAX_DISTINCT_FILES)
                || self.deadline.is_some_and(|at| at <= now));
        if full {
            self.carry = Some(record);
            return self.seal().map(|()| Flow::Stop).map_err(|error| {
                // A failed seal is terminal, but the unappended output still belongs to the caller.
                RejectedRecord {
                    error,
                    record: self.carry.take().expect("owned carry-over"),
                }
            });
        }
        self.append(&record, size, now);
        if self.count == self.max_records || self.bytes == self.max_bytes {
            // The frame is already in Arrow storage, so a failed seal is reported via failure().
            let _ = self.seal();
            return Ok(Flow::Stop);
        }
        Ok(Flow::Continue)
    }

    /// Explicit nonempty flush, including descriptor pressure or normal drain.
    pub fn seal(&mut self) -> Result<(), BatchError> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        if self.retained.is_some() || self.count == 0 {
            return Ok(());
        }
        let result = self.finish_records();
        let records = match result {
            Ok(records) => records,
            Err(error) => return self.fail(BatchError::Encoding(error.to_string())),
        };
        self.retained = Some(Batch {
            records,
            progress: std::mem::take(&mut self.progress),
            provenance: std::mem::take(&mut self.provenance),
            record_count: self.count,
            logical_bytes: self.bytes,
        });
        self.count = 0;
        self.bytes = 0;
        self.deadline = None;
        Ok(())
    }

    /// Call ONLY after matching aggregate completion/loss policy and successful application
    /// of this entire progress transaction. This method performs no Ack authorization or I/O.
    /// Carry-over seeds the next open batch before the caller may resume the scheduler.
    pub fn release_applied(
        &mut self,
        applied: &[cp::UpdateProgress],
        now: Instant,
    ) -> Result<(), BatchError> {
        if self.failure.is_some() || self.retained.as_ref().is_none_or(|b| b.progress != applied) {
            return Err(BatchError::Release);
        }
        if self.carry.is_some() && now.checked_add(self.flush_period).is_none() {
            return self.fail(BatchError::Deadline);
        }
        let mut batch = self.retained.take().expect("validated retained batch");
        batch.progress.clear();
        self.progress = std::mem::take(&mut batch.progress);
        batch.provenance.clear();
        self.provenance = std::mem::take(&mut batch.provenance);
        self.indices.clear();
        drop(batch);
        if let Some(record) = self.carry.take() {
            let size = record.logical_size()?;
            self.append(&record, size, now);
            if self.count == self.max_records || self.bytes == self.max_bytes {
                self.seal()?;
            }
        }
        Ok(())
    }

    /// Drops uncommitted storage with an explicit pending-capture report, never progress.
    #[must_use]
    pub fn abort(self) -> PendingCapture {
        PendingCapture {
            records: self.count
                + self.retained.as_ref().map_or(0, |b| b.record_count)
                + usize::from(self.carry.is_some()),
            logical_bytes: self.bytes
                + self.retained.as_ref().map_or(0, |b| b.logical_bytes)
                + self
                    .carry
                    .as_ref()
                    .map_or(0, |r| r.logical_size().expect("validated carry-over")),
        }
    }

    fn fail<T>(&mut self, error: BatchError) -> Result<T, BatchError> {
        self.failure = Some(error.clone());
        Err(error)
    }

    fn append(&mut self, record: &Record, size: u64, now: Instant) {
        let id = self.count as u16;
        self.logs.append_id(Some(id));
        self.logs.append_time_unix_nano(0);
        self.logs
            .append_observed_time_unix_nano(record.observed_time_unix_nano as i64);
        self.logs.append_severity_number(None);
        self.logs.append_severity_text(None);
        match record.frame.body() {
            LineBody::Text(s) => self.logs.body.append_str(s.as_bytes()),
            LineBody::Bytes(b) => self.logs.body.append_bytes(b),
        }
        for (key, value) in record.metadata.as_slice() {
            self.attrs.append_parent_id(&id);
            let slot = ATTRIBUTE_KEYS[..6]
                .iter()
                .position(|name| name == key)
                .expect("metadata projector emits only registered keys");
            self.attrs.append_known_key(KnownKey::<14>::slots()[slot]);
            match value {
                Value::StringValue(s) => self.attrs.any_values_builder.append_str(s.as_bytes()),
                Value::BytesValue(b) => self.attrs.any_values_builder.append_bytes(b),
                Value::BoolValue(b) => self.attrs.any_values_builder.append_bool(*b),
                _ => unreachable!("metadata projector emits only strings, bytes and bools"),
            }
        }
        if let Some((start, index, last)) = record.frame.fragment() {
            let mut digest = Sha256::new();
            digest.update(b"otel-arrow-filelog-fragment-v1\0");
            digest.update(record.file_id.as_bytes());
            digest.update(record.epoch.to_be_bytes());
            digest.update(start.to_be_bytes());
            let mut hex = [0; 64];
            hex::encode_to_slice(digest.finalize(), &mut hex).expect("fixed digest output");
            for i in 0..FRAGMENT_KEYS.len() {
                self.attrs.append_parent_id(&id);
                self.attrs.append_known_key(KnownKey::<14>::slots()[6 + i]);
                match i {
                    0 => self.attrs.any_values_builder.append_str(&hex),
                    2 => self.attrs.any_values_builder.append_bool(last),
                    _ => self.attrs.any_values_builder.append_int(match i {
                        1 => i64::from(index),
                        3 => record.frame.body_range().start as i64,
                        4 => record.frame.body_range().end as i64,
                        5 => record.frame.range().start as i64,
                        _ => record.frame.range().end as i64,
                    }),
                }
            }
        }
        if record.frame.terminal() {
            self.attrs.append_parent_id(&id);
            self.attrs.append_known_key(KnownKey::<14>::slots()[13]);
            self.attrs.any_values_builder.append_bool(true);
        }
        let range = record.frame.range();
        let (truncated, discarded_source_bytes, malformed_units, pattern_not_matched) =
            match &record.frame {
                Frame::Line(f) => (
                    f.truncated,
                    f.discarded_source_bytes,
                    f.malformed_units,
                    false,
                ),
                Frame::Multiline(f) => (
                    f.truncated,
                    f.discarded_source_bytes,
                    f.malformed_units,
                    f.pattern_not_matched,
                ),
            };
        self.provenance.push(Provenance {
            file_id: record.file_id,
            epoch: record.epoch,
            body_range: record.frame.body_range(),
            frame_range: range,
            truncated,
            discarded_source_bytes,
            malformed_units,
            pattern_not_matched,
        });
        if let Some(&index) = self.indices.get(&record.file_id) {
            let p = &mut self.progress[index];
            p.new_committed_offset = range.end;
            p.new_committed_frontier_guard = record.guard;
            p.new_framing_resume = record.frame.resume();
            p.new_last_seen_time_unix_nano = p
                .new_last_seen_time_unix_nano
                .max(record.observed_time_unix_nano);
        } else {
            let _ = self.indices.insert(record.file_id, self.progress.len());
            self.progress.push(cp::UpdateProgress {
                file_id: record.file_id,
                expected_committed_offset: range.start,
                expected_file_epoch: record.epoch,
                new_committed_offset: range.end,
                new_committed_frontier_guard: record.guard,
                new_framing_resume: record.frame.resume(),
                new_last_seen_time_unix_nano: record.observed_time_unix_nano,
                finalize: false,
            });
        }
        if self.count == 0 {
            self.deadline = now.checked_add(self.flush_period);
        }
        self.count += 1;
        self.bytes += size;
    }

    fn finish_records(&mut self) -> otel_arrow_dfe_pdata::encode::Result<OtapArrowRecords> {
        let n = self.count;
        self.logs.resource.append_id_n(0, n);
        self.logs.resource.append_schema_url_n(None, n);
        self.logs.resource.append_dropped_attributes_count_n(0, n);
        self.logs.scope.append_id_n(0, n);
        self.logs
            .scope
            .append_name_n(Some(b"otel-arrow-dfe-core-nodes/filelog"), n);
        self.logs
            .scope
            .append_version_n(Some(env!("CARGO_PKG_VERSION").as_bytes()), n);
        self.logs.scope.append_dropped_attributes_count_n(0, n);
        self.logs.append_schema_url_n(None, n);
        self.logs.append_dropped_attributes_count_n(0, n);
        self.logs.append_flags_n(None, n);
        self.logs.append_trace_id_n(None, n)?;
        self.logs.append_span_id_n(None, n)?;
        let mut records = OtapArrowRecords::Logs(Logs::default());
        records.set(ArrowPayloadType::Logs, self.logs.finish()?)?;
        let attrs = self.attrs.finish()?;
        if attrs.num_rows() != 0 {
            records.set(ArrowPayloadType::LogAttrs, attrs)?;
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests;
