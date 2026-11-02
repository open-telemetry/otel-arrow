// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Complete-file publication, serialized by the store's exclusive mutable borrow.

use std::collections::HashSet;

use super::*;
use crate::{FileId, LifecycleState};

/// Durable publication of existing applied state; never source-progress authorization.
/// All progress applied before this compaction is durable. In the new generation,
/// `synced_sequence()` is zero and the next append uses sequence one. Compare
/// append sequences only within the same generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactionReceipt {
    /// Published snapshot generation. Its first subsequent WAL sequence is one.
    pub generation: u64,
    /// Entire vetted set omitted from the published snapshot.
    pub removed_records: usize,
}

pub(super) struct CompactionAttempt {
    generation: u64,
    removals: HashSet<FileId>,
    replacement_succeeded: bool,
}

impl CompactionAttempt {
    fn receipt(&self) -> CompactionReceipt {
        CompactionReceipt {
            generation: self.generation,
            removed_records: self.removals.len(),
        }
    }

    // Check before recovery barriers/repair/cleanup. A live attempt cannot adopt
    // unrelated valid state or silently roll back previously applied progress.
    pub(super) fn validate_recovery(
        &self,
        previous: &Recovered,
        state: &ReplayState,
        boundary: u64,
        end: u64,
        length: u64,
    ) -> Result<(), StoreError> {
        let published = state.generation() == self.generation;
        let expected_count =
            previous.state.records().len() - if published { self.removals.len() } else { 0 };
        let metadata_matches = if published {
            state.last_sequence() == 0 && end == boundary && length == boundary
        } else {
            !self.replacement_succeeded
                && state.generation() == previous.state.generation()
                && state.last_sequence() == previous.state.last_sequence()
                && boundary == previous.wal_offset
                && end == previous.end
                && length == previous.end
        };
        if !metadata_matches
            || state.records().len() != expected_count
            || previous.state.records().any(|record| {
                if published && self.removals.contains(&record.file_id) {
                    state.get(&record.file_id).is_some()
                } else {
                    state.get(&record.file_id) != Some(record)
                }
            })
        {
            return Err(StoreError::AuthorityChanged);
        }
        Ok(())
    }
}

impl CheckpointStore {
    /// Whether a compaction attempt still needs resolution, including when poisoned.
    #[must_use]
    pub fn has_unresolved_compaction(&self) -> bool {
        self.compaction.is_some()
    }

    /// Whether guarded compaction recovery is available under the held lock.
    /// The caller still classifies the original error and bounds retries.
    #[must_use]
    pub fn can_recover_compaction(&self) -> bool {
        self.compaction.is_some() && self.usable().is_ok()
    }

    /// Publishes all applied state with a fresh WAL, even when no records are removed.
    /// The worker retains any Ack-authorized batch until its later append succeeds.
    /// On failure, call `recover_compaction` if recovery is required; never retry
    /// a failed sync or open a new store to bypass terminal poisoning.
    pub fn compact(&mut self) -> Result<CompactionReceipt, StoreError> {
        self.compact_inner(&[])
    }

    /// Atomically removes the entire caller-vetted set through compaction.
    ///
    /// The caller must revalidate continuous absence and every runtime veto and
    /// serialize those decisions with this call. Storage does not infer eligibility
    /// from age, timestamps, lifecycle alone, or available capacity. Unknown,
    /// duplicate, and quarantined IDs reject the whole set before writing.
    /// An empty set performs no I/O and publishes no generation.
    pub fn compact_with_retention(
        &mut self,
        vetted_removals: &[FileId],
    ) -> Result<Option<CompactionReceipt>, StoreError> {
        self.ready()?;
        if vetted_removals.is_empty() {
            return Ok(None);
        }
        self.compact_inner(vetted_removals).map(Some)
    }

    fn compact_inner(&mut self, ids: &[FileId]) -> Result<CompactionReceipt, StoreError> {
        self.ready()?;
        let generation = self
            .recovered
            .state
            .generation()
            .checked_add(1)
            .ok_or(StoreError::GenerationExhausted)?;
        if ids.len() > self.recovered.state.records().len() {
            return Err(StoreError::InvalidRetention(
                "set exceeds stored population",
            ));
        }
        let mut removals = HashSet::with_capacity(ids.len());
        for id in ids {
            let record = self
                .recovered
                .state
                .get(id)
                .ok_or(StoreError::InvalidRetention("unknown file ID"))?;
            if record.lifecycle_state == LifecycleState::Quarantined {
                return Err(StoreError::InvalidRetention("quarantined record"));
            }
            if !removals.insert(*id) {
                return Err(StoreError::InvalidRetention("duplicate file ID"));
            }
        }
        // Measure using the existing codec and one bounded record's codec scratch
        // before cloning records or allocating the complete encoded snapshot.
        let mut snapshot_bytes = (SNAPSHOT_HEADER_BYTES + SNAPSHOT_FOOTER_BYTES) as u64;
        for record in self
            .recovered
            .state
            .records()
            .filter(|record| !removals.contains(&record.file_id))
        {
            snapshot_bytes = snapshot_bytes
                .checked_add(record.encode_frame()?.len() as u64)
                .ok_or(StoreError::RecoveryLimit("compaction snapshot arithmetic"))?;
            if snapshot_bytes > self.config.replay.max_snapshot_bytes {
                return Err(StoreError::RecoveryLimit("compaction snapshot bytes"));
            }
        }
        let records: Vec<_> = self
            .recovered
            .state
            .records()
            .filter(|record| !removals.contains(&record.file_id))
            .cloned()
            .collect();
        let mut prefix = encode_checkpoint(generation, &self.checkpoint_id, &records)?;
        drop(records);
        let _ = ReplayState::from_checkpoint(&prefix, &self.checkpoint_id, self.config.replay)?;
        self.verify_authority()?;
        self.verify_length()?;
        let _ = self.namespace.inventory()?;
        // Any subsequent failure must finish cleanup/recovery before publication.
        self.compaction = Some(CompactionAttempt {
            generation,
            removals,
            replacement_succeeded: false,
        });
        let mut temp = self.namespace.create(COMPACT)?;
        self.io.write_all(&mut temp, &prefix)?;
        self.io.step("validate compaction temporary")?;
        if temp
            .metadata()
            .map_err(|e| io("stat compaction temporary", e))?
            .len()
            != prefix.len() as u64
        {
            return Err(invalid_staging("unexpected file length"));
        }
        let _ = temp
            .seek(SeekFrom::Start(0))
            .map_err(|e| io("seek compaction temporary", e))?;
        temp.read_exact(&mut prefix).map_err(|error| {
            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                invalid_staging("incomplete file")
            } else {
                io("read compaction temporary", error)
            }
        })?;
        let (staged, boundary) =
            ReplayState::from_checkpoint(&prefix, &self.checkpoint_id, self.config.replay)
                .map_err(|source| StoreError::CompactionStagingInvalid {
                    reason: "snapshot validation failed",
                    source: Some(source),
                })?;
        if staged.generation() != generation {
            return Err(invalid_staging("unexpected generation"));
        }
        let attempt = self
            .compaction
            .as_ref()
            .expect("installed compaction attempt");
        attempt
            .validate_recovery(
                &self.recovered,
                &staged,
                boundary as u64,
                prefix.len() as u64,
                prefix.len() as u64,
            )
            .map_err(|_| invalid_staging("state differs from the intended snapshot"))?;
        drop(staged);
        drop(prefix);
        self.io.sync(&temp, "compaction file sync")?;
        self.namespace.verify(COMPACT, &temp)?;
        drop(temp);
        self.verify_authority()?;
        self.verify_length()?;
        // Preserve the old description's deferred-write error history before
        // closing it. A fresh description need not report the same error.
        self.recovered
            .file()
            .sync(&mut self.io, "pre-compaction file sync")?;
        drop(self.recovered.file.take());
        self.namespace.replace_database(&mut self.io)?;
        self.compaction
            .as_mut()
            .expect("installed compaction attempt")
            .replacement_succeeded = true;
        self.namespace
            .sync_directory(&mut self.io, "compaction directory sync")?;
        self.io.step("reopen compacted checkpoint")?;
        self.recover_compaction()?
            .ok_or(StoreError::AuthorityChanged)
    }

    /// Resolves a failed compaction under the retained namespace lock.
    ///
    /// `Some` means the complete proposed generation is durably published; `None`
    /// means the old generation remains authoritative. Revalidate removal vetoes
    /// before a new attempt. Pending state remains blocked until all recovery and
    /// cleanup succeeds. Sync poisoning always takes precedence and is never cleared.
    pub fn recover_compaction(&mut self) -> Result<Option<CompactionReceipt>, StoreError> {
        self.usable()?;
        if self.compaction.is_none() {
            return Err(StoreError::InvalidConfig("no pending compaction"));
        }
        // A pre-replacement failure still owns the old handle. Never bypass its
        // writeback history on a retry, including after temporary write failure.
        if self.recovered.file.is_some() {
            self.verify_authority()?;
            self.verify_length()?;
            self.recovered
                .file()
                .sync(&mut self.io, "pre-compaction recovery file sync")?;
        }
        let attempt = self
            .compaction
            .as_ref()
            .expect("checked compaction attempt");
        let recovery = recover(
            &self.namespace,
            &self.checkpoint_id,
            self.config,
            &mut self.io,
            None,
            Some((&self.recovered, attempt)),
        );
        if matches!(
            &recovery,
            Err(StoreError::AuthorityChanged | StoreError::Untrusted(_))
        ) {
            self.authority_failed = true;
        }
        let recovered = recovery?;
        let receipt =
            (recovered.state.generation() == attempt.generation).then(|| attempt.receipt());
        self.synced_sequence = recovered.state.last_sequence();
        self.recovered = recovered;
        self.compaction = None;
        Ok(receipt)
    }
}

fn invalid_staging(reason: &'static str) -> StoreError {
    StoreError::CompactionStagingInvalid {
        reason,
        source: None,
    }
}
