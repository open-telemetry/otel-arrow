// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Concrete Task 16 adapter; no delivery authorization or checkpoint writes.

use super::super::{
    batch::{BatchBuilder, BatchError, Record},
    config::{FileMetadata, ValidatedInputs},
    metadata::FileMetadataAttributes,
};
use super::{CommittedFrontierGuard, Frame, ReaderKey};
use hashbrown::HashMap;
use std::{rc::Rc, time::Instant};

pub(crate) use super::super::batch::Flow;

/// Owned framed output. Source ranges and boundary/fragment evidence stay intact.
#[derive(Debug)]
pub(crate) struct Output {
    pub file_id: super::cp::FileId,
    pub key: ReaderKey,
    pub epoch: u32,
    pub frame: Frame,
    pub guard: CommittedFrontierGuard,
}

/// Failure explicitly returns ownership. The scheduler retains this output and
/// stops; composition must dispose it with the reader's unresolved work.
pub(crate) struct Rejected {
    pub output: Output,
}

/// Task 16 review boundary. Preflight includes retained batch/carry-over state
/// and all known record/byte/progress-mark limits. It runs before *every* framing
/// step, including empty-input replay, and before every source operation.
pub(crate) trait BatchSink {
    fn preflight(&mut self, key: ReaderKey, epoch: u32) -> Flow;
    /// Composition supplies the monotonic turn clock before any work or deadline check.
    fn poll(&mut self, _now: Instant) -> Flow {
        Flow::Continue
    }
    /// Exposed even when all readers are idle or paused.
    fn next_deadline(&self) -> Option<Instant> {
        None
    }
    fn failed(&self) -> bool {
        false
    }
    /// Descriptor pressure must resolve an open batch before pinned handles can be reclaimed.
    fn flush(&mut self) -> Flow {
        Flow::Continue
    }
    /// Both success values consume ownership. Stop retains at most one carry-over
    /// and must prevent subsequent admission until composition resolves it.
    // Return the owned frame inline: failure must not allocate to preserve it.
    #[allow(clippy::result_large_err)]
    fn offer(&mut self, output: Output) -> Result<Flow, Rejected>;
    fn unresolved(&self, key: ReaderKey, epoch: u32) -> bool;
}

struct Source {
    file_id: super::cp::FileId,
    epoch: u32,
    next_offset: u64,
    resume: super::cp::FramingResume,
    metadata: Rc<FileMetadataAttributes>,
}

/// One worker's source context and concrete batch storage; no extra handoff queue.
pub(crate) struct Batcher {
    pub builder: BatchBuilder,
    sources: HashMap<ReaderKey, Source>,
    max_sources: usize,
    now: Instant,
    observed_time: fn() -> u64,
    metadata_policy: FileMetadata,
    failure: Option<BatchError>,
}

impl Batcher {
    pub(crate) fn new(inputs: &ValidatedInputs, now: Instant, observed_time: fn() -> u64) -> Self {
        Self {
            builder: BatchBuilder::new(inputs),
            sources: HashMap::new(),
            max_sources: inputs.settings().limits.max_tracked_files as usize,
            now,
            observed_time,
            metadata_policy: inputs.settings().include_file_metadata,
            failure: None,
        }
    }

    /// Register alongside Scheduler::admit, from the same authoritative seed.
    pub(crate) fn register(
        &mut self,
        key: ReaderKey,
        file_id: super::cp::FileId,
        epoch: u32,
        offset: u64,
        resume: super::cp::FramingResume,
        path: &super::cp::AdvisoryPath,
    ) -> Result<(), BatchError> {
        if self.sources.len() == self.max_sources
            || self.sources.contains_key(&key)
            || self
                .sources
                .values()
                .any(|source| source.file_id == file_id)
        {
            return Err(BatchError::Progress);
        }
        let _ = self.sources.insert(
            key,
            Source {
                file_id,
                epoch,
                next_offset: offset,
                resume,
                metadata: Rc::new(FileMetadataAttributes::project(self.metadata_policy, path)),
            },
        );
        Ok(())
    }

    /// Refresh bounded path evidence without rewriting any accepted output.
    pub(crate) fn update_metadata(
        &mut self,
        key: ReaderKey,
        path: &super::cp::AdvisoryPath,
    ) -> Result<(), BatchError> {
        self.sources
            .get_mut(&key)
            .ok_or(BatchError::Progress)?
            .metadata = Rc::new(FileMetadataAttributes::project(self.metadata_policy, path));
        Ok(())
    }

    pub(crate) fn failure(&self) -> Option<&BatchError> {
        self.failure.as_ref().or_else(|| self.builder.failure())
    }
}

impl BatchSink for Batcher {
    fn poll(&mut self, now: Instant) -> Flow {
        self.now = now;
        if self.failure().is_some() {
            return Flow::Stop;
        }
        self.builder.poll(now).unwrap_or(Flow::Stop)
    }

    fn next_deadline(&self) -> Option<Instant> {
        self.builder.next_deadline()
    }

    fn failed(&self) -> bool {
        self.failure().is_some()
    }

    fn preflight(&mut self, key: ReaderKey, epoch: u32) -> Flow {
        if self.failure().is_some() {
            return Flow::Stop;
        }
        let Some(source) = self
            .sources
            .get(&key)
            .filter(|source| source.epoch == epoch)
        else {
            self.failure = Some(BatchError::Progress);
            return Flow::Stop;
        };
        // Empty bodies are legal; configured metadata is the known minimum.
        let minimum = super::super::batch::RECORD_OVERHEAD
            + super::super::batch::metadata_size(&source.metadata);
        self.builder
            .preflight(source.file_id, epoch, minimum, self.now)
            .unwrap_or(Flow::Stop)
    }

    fn offer(&mut self, output: Output) -> Result<Flow, Rejected> {
        let valid = self.sources.get(&output.key).is_some_and(|source| {
            source.file_id == output.file_id
                && source.epoch == output.epoch
                && source.next_offset == output.frame.range().start
                && super::super::batch::follows(source.resume, &output.frame)
        });
        if !valid || self.failure().is_some() {
            self.failure = Some(BatchError::Progress);
            return Err(Rejected { output });
        }
        let source = self.sources.get_mut(&output.key).expect("validated source");
        let key = output.key;
        let end = output.frame.range().end;
        let resume = output.frame.resume();
        let record = Record {
            file_id: output.file_id,
            epoch: output.epoch,
            frame: output.frame,
            guard: output.guard,
            observed_time_unix_nano: (self.observed_time)(),
            metadata: Rc::clone(&source.metadata),
        };
        match self.builder.offer(record, self.now) {
            Ok(flow) => {
                source.next_offset = end;
                source.resume = resume;
                Ok(flow)
            }
            Err(rejected) => {
                self.failure = Some(rejected.error);
                let record = rejected.record;
                Err(Rejected {
                    output: Output {
                        file_id: record.file_id,
                        key,
                        epoch: record.epoch,
                        frame: record.frame,
                        guard: record.guard,
                    },
                })
            }
        }
    }

    fn flush(&mut self) -> Flow {
        if self.failure().is_some()
            || self.builder.seal().is_err()
            || self.builder.retained().is_some()
        {
            Flow::Stop
        } else {
            Flow::Continue
        }
    }

    fn unresolved(&self, key: ReaderKey, epoch: u32) -> bool {
        self.sources.get(&key).is_some_and(|source| {
            source.epoch == epoch && self.builder.unresolved(source.file_id, epoch)
        })
    }
}
