// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Blocking Linux checkpoint storage. Call only from the dedicated checkpoint
//! worker. The caller supplies an engine-validated state-directory descriptor,
//! source/Ack authorization, retry deadlines, and periodic/drain sync scheduling.
//! Compaction is a separate task; reaching admission returns `CompactionRequired`.

mod filesystem;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::fd::BorrowedFd;

use crate::{
    CHECKPOINT_HEADER_BYTES, DecodeError, EncodeError, Operation, ReplayConfig, ReplayError,
    ReplayState, ReplayStep, SNAPSHOT_FOOTER_BYTES, SNAPSHOT_HEADER_BYTES, TX_HEADER_BYTES,
    Transaction, TransactionScan, WAL_MAX_TX_FRAME_BYTES, decode_checkpoint_header,
    encode_checkpoint, encode_transaction, scan_next_transaction,
};
use filesystem::{Io, Namespace};

const DB: &str = "checkpoint.db";
const CREATE: &str = "checkpoint.db.create.tmp";
const COMPACT: &str = "checkpoint.db.compact.tmp";
const MAX_BYTES: u64 = 1 << 30;

/// Failures never authorize automatic salvage or source progress.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Invalid namespace ID or resource admission.
    #[error("invalid checkpoint store configuration: {0}")]
    InvalidConfig(&'static str),
    /// Resource admission failed; retain every contributing configured value.
    #[error(
        "invalid checkpoint limits: {reason}; max_tracked_files={max_tracked_files}, max_snapshot_bytes={max_snapshot_bytes}, max_wal_bytes={max_wal_bytes}, max_wal_transactions={max_wal_transactions}"
    )]
    InvalidLimits {
        /// Rejected bound or arithmetic operation.
        reason: &'static str,
        /// Configured record limit.
        max_tracked_files: u32,
        /// Configured snapshot byte budget.
        max_snapshot_bytes: u64,
        /// Configured append byte threshold.
        max_wal_bytes: u64,
        /// Configured append transaction threshold.
        max_wal_transactions: u64,
    },
    /// A failed durability barrier is terminal, even if a later fsync would succeed.
    #[error(
        "checkpoint {operation} failed; durability is unknown and sync-only retry is unsafe: {source}"
    )]
    SyncFailed {
        /// Failed barrier.
        operation: &'static str,
        /// Original error, which may be reported by the kernel only once.
        #[source]
        source: std::io::Error,
    },
    /// A prior failed barrier permanently prevents this store from certifying durability.
    #[error("checkpoint store poisoned by failed {operation}; storage remediation required")]
    SyncPoisoned {
        /// First failed barrier.
        operation: &'static str,
    },
    /// Another store owns this exact namespace, including in this process.
    #[error("checkpoint namespace is already locked")]
    Locked,
    /// A filesystem artifact violated the namespace trust contract.
    #[error("untrusted checkpoint artifact: {0}")]
    Untrusted(&'static str),
    /// No authority exists and this is not interrupted first publication.
    #[error("authoritative checkpoint generation is missing")]
    AuthoritativeGenerationMissing,
    /// The immutable prefix is physically incomplete; it cannot be repaired.
    #[error("authoritative checkpoint generation is incomplete")]
    AuthoritativeGenerationIncomplete,
    /// The authoritative file cannot be opened or read.
    #[error("authoritative checkpoint generation is unreadable: {0}")]
    AuthoritativeGenerationUnreadable(#[source] std::io::Error),
    /// Required filesystem work failed. An append may be pending resolution.
    #[error("checkpoint {operation}: {source}")]
    Io {
        /// Failed operation without path or source content.
        operation: &'static str,
        /// Original operating-system error.
        #[source]
        source: std::io::Error,
    },
    /// A wire validation failure is corruption, not repair permission.
    #[error(transparent)]
    Decode(#[from] DecodeError),
    /// A locally produced transaction could not be encoded.
    #[error(transparent)]
    Encode(#[from] EncodeError),
    /// Stored-state preconditions or replay failed.
    #[error(transparent)]
    Replay(#[from] ReplayError),
    /// WAL admission would be exceeded; no bytes or live state changed.
    #[error("checkpoint compaction required before append")]
    CompactionRequired,
    /// Hard file or configured replay admission was exceeded during recovery.
    #[error("checkpoint recovery exceeds {0}")]
    RecoveryLimit(&'static str),
    /// An uncertain append must be resolved before further use.
    #[error("checkpoint append requires recovery under the held namespace lock")]
    RecoveryRequired,
    /// The authoritative file changed unexpectedly under exclusive ownership.
    #[error("checkpoint authority changed under namespace ownership")]
    AuthorityChanged,
}

fn io(operation: &'static str, source: impl Into<std::io::Error>) -> StoreError {
    StoreError::Io {
        operation,
        source: source.into(),
    }
}

/// Store admission. The receiver must additionally admit its aggregate memory.
#[derive(Debug, Clone, Copy)]
pub struct StoreConfig {
    /// Record, snapshot, and framing-profile admission.
    pub replay: ReplayConfig,
    /// Maximum WAL bytes, excluding header and snapshot; at least one maximum frame.
    pub max_wal_bytes: u64,
    /// Maximum complete transactions in a generation; must be positive.
    pub max_wal_transactions: u64,
}

impl StoreConfig {
    fn invalid(self, reason: &'static str) -> StoreError {
        StoreError::InvalidLimits {
            reason,
            max_tracked_files: self.replay.max_tracked_files,
            max_snapshot_bytes: self.replay.max_snapshot_bytes,
            max_wal_bytes: self.max_wal_bytes,
            max_wal_transactions: self.max_wal_transactions,
        }
    }

    fn validate(self) -> Result<Self, StoreError> {
        if self.replay.max_tracked_files == 0
            || self.replay.max_snapshot_bytes
                < (SNAPSHOT_HEADER_BYTES + SNAPSHOT_FOOTER_BYTES) as u64
            || self.max_wal_bytes < WAL_MAX_TX_FRAME_BYTES
            || self.max_wal_transactions == 0
        {
            return Err(self.invalid(
                "nonzero records/count, complete snapshot, and maximum WAL frame required",
            ));
        }
        // The product can exceed u64 while the byte threshold still bounds
        // every reachable WAL. Saturation preserves the mathematical minimum.
        let wal_bound = self
            .max_wal_transactions
            .saturating_mul(WAL_MAX_TX_FRAME_BYTES)
            .min(self.max_wal_bytes);
        let snapshot = self.replay.max_snapshot_bytes;
        let bounded = |value: Option<u64>, reason| {
            value
                .filter(|v| *v <= MAX_BYTES)
                .ok_or_else(|| self.invalid(reason))
        };
        let _ = bounded(
            snapshot
                .checked_add(CHECKPOINT_HEADER_BYTES as u64)
                .and_then(|v| v.checked_add(wal_bound)),
            "checkpoint file bound exceeds 1 GiB or overflows",
        )?;
        let _ = bounded(
            snapshot
                .checked_mul(4)
                .and_then(|v| v.checked_add(CHECKPOINT_HEADER_BYTES as u64)),
            "snapshot recovery bound exceeds 1 GiB or overflows",
        )?;
        let _ = bounded(
            snapshot
                .checked_mul(3)
                .and_then(|v| v.checked_add(wal_bound))
                .and_then(|v| {
                    v.checked_add(4 * WAL_MAX_TX_FRAME_BYTES + CHECKPOINT_HEADER_BYTES as u64)
                }),
            "WAL recovery bound exceeds 1 GiB or overflows",
        )?;
        Ok(self)
    }
}

/// Ordinary progress durability requested by the worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    /// Append, apply, and synchronize before success (the default receiver policy).
    Sync,
    /// Ordinary progress may be applied before the worker's next sync deadline.
    Deferred,
}

/// Successful append receipt. No source/Ack authorization is inferred by storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppendReceipt {
    /// Sequence applied exactly once in this generation.
    pub sequence: u64,
    /// Whether this sequence has passed a file synchronization barrier.
    pub synced: bool,
}

/// Startup recovery facts for caller-owned health reporting.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryReport {
    /// An existing namespace lacked authority in a recognized initial state.
    pub interrupted_creation: bool,
    /// Proven incomplete final WAL bytes removed after the publication barrier.
    pub repaired_tail_bytes: u64,
    /// A recognized, non-authoritative temporary was removed.
    pub cleaned_temporary: bool,
}

struct Recovered {
    file: File,
    state: ReplayState,
    wal_offset: u64,
    end: u64,
    report: RecoveryReport,
}

struct Pending {
    frame: Vec<u8>,
    boundary: u64,
    sequence: u64,
}

/// Exclusive owner of one checkpoint namespace. Drop releases ownership but
/// does not imply a successful drain; callers must explicitly call `sync`.
/// A recoverable failed append retains its frame and lock until
/// `resolve_pending` succeeds or the store is dropped. Failed synchronization
/// permanently poisons this instance and requires explicit storage remediation.
/// Poison is not shared with newly opened instances; callers must enforce the
/// no-automatic-reopen policy. Never retry the logical transaction after
/// resolution returns its receipt: it has already been applied.
pub struct CheckpointStore {
    checkpoint_id: String,
    config: StoreConfig,
    recovered: Recovered,
    synced_sequence: u64,
    pending: Option<Pending>,
    authority_failed: bool,
    io: Io,
    // Rust drops fields in declaration order: ownership outlives every data handle.
    namespace: Namespace,
}

impl CheckpointStore {
    /// Opens relative to the engine's already provisioned, trusted state root.
    /// Passing an arbitrary directory does not fulfill engine provisioning.
    /// Lock contention returns immediately; the worker owns bounded retries.
    /// A known prior sync failure requires storage remediation, not automatic
    /// reopen: a new handle may not observe another process's writeback error.
    /// This method cannot check prior instances' failure history. The lifecycle
    /// caller must prevent retry/reconfiguration from bypassing a known failure,
    /// including SyncFailed returned before a store instance was constructed.
    pub fn open(
        root: BorrowedFd<'_>,
        checkpoint_id: &str,
        config: StoreConfig,
    ) -> Result<Self, StoreError> {
        Self::open_with_io(root, checkpoint_id, config, Io::default())
    }

    fn open_with_io(
        root: BorrowedFd<'_>,
        checkpoint_id: &str,
        config: StoreConfig,
        mut io: Io,
    ) -> Result<Self, StoreError> {
        let config = config.validate()?;
        let namespace = Namespace::open(root, checkpoint_id, &mut io)?;
        let artifacts = namespace.inventory()?;
        let mut interrupted_creation = false;
        let mut cleaned_creation = false;
        if !artifacts.database {
            if artifacts.compact {
                return Err(StoreError::AuthoritativeGenerationMissing);
            }
            interrupted_creation = !namespace.created;
            // Establish a namespace barrier before deleting an interrupted temp.
            io.sync(&namespace.directory, "initial recovery directory sync")?;
            if artifacts.create {
                namespace.remove(CREATE)?;
                io.sync(&namespace.directory, "initial cleanup directory sync")?;
                cleaned_creation = true;
            }
            let prefix = encode_checkpoint(0, checkpoint_id, &[])?;
            let _ = ReplayState::from_checkpoint(&prefix, checkpoint_id, config.replay)?;
            let mut temp = namespace.create(CREATE)?;
            io.write_all(&mut temp, &prefix)?;
            io.sync(&temp, "creation file sync")?;
            namespace.verify(CREATE, &temp)?;
            drop(temp);
            namespace.publish_initial()?;
            io.sync(&namespace.directory, "publication directory sync")?;
        }
        let mut recovered = recover(&namespace, checkpoint_id, config, &mut io, None)?;
        recovered.report.interrupted_creation = interrupted_creation;
        recovered.report.cleaned_temporary |= cleaned_creation;
        let synced_sequence = recovered.state.last_sequence();
        Ok(Self {
            namespace,
            checkpoint_id: checkpoint_id.to_owned(),
            config,
            recovered,
            synced_sequence,
            pending: None,
            authority_failed: false,
            io,
        })
    }

    /// Immutable applied state, unavailable while a write outcome is unresolved.
    pub fn state(&self) -> Result<&ReplayState, StoreError> {
        self.ready()?;
        Ok(&self.recovered.state)
    }

    /// Facts from the most recent successful recovery.
    #[must_use]
    pub fn recovery_report(&self) -> RecoveryReport {
        self.recovered.report
    }

    /// Last sequence covered by a successful file sync in this generation.
    /// Startup assumes the caller has excluded unresolved prior writeback errors;
    /// a new handle cannot establish that history from a successful fsync.
    #[must_use]
    pub fn synced_sequence(&self) -> u64 {
        self.synced_sequence
    }

    /// Applied WAL bytes, excluding the immutable prefix.
    #[must_use]
    pub fn wal_bytes(&self) -> u64 {
        self.recovered.end - self.recovered.wal_offset
    }

    fn usable(&self) -> Result<(), StoreError> {
        self.io.check_sync_health()?;
        if self.authority_failed {
            return Err(StoreError::AuthorityChanged);
        }
        Ok(())
    }

    // Conservatively latch a failed authority check, including unreadable metadata.
    // This instance must never resume after observing namespace substitution.
    fn verify_authority(&mut self) -> Result<(), StoreError> {
        self.authority_failed = true;
        self.namespace.verify(DB, &self.recovered.file)?;
        self.authority_failed = false;
        Ok(())
    }

    // Only used without a pending attempt: the accepted append boundary must
    // still be physical EOF before appending or certifying deferred progress.
    fn verify_length(&mut self) -> Result<(), StoreError> {
        self.authority_failed = true;
        let length = self
            .recovered
            .file
            .metadata()
            .map_err(|e| io("stat checkpoint length", e))?
            .len();
        if length != self.recovered.end {
            return Err(StoreError::AuthorityChanged);
        }
        self.authority_failed = false;
        Ok(())
    }

    fn ready(&self) -> Result<(), StoreError> {
        self.usable()?;
        if self.pending.is_some() {
            Err(StoreError::RecoveryRequired)
        } else {
            Ok(())
        }
    }

    /// Encodes and preflights the complete transaction before writing. All
    /// non-progress operations and rotation finalization force synchronization.
    /// On a recoverable write failure, resolve the retained attempt instead of
    /// submitting again. A failed synchronization barrier permanently poisons
    /// the store; neither resolution nor another sync can clear that condition.
    pub fn append(
        &mut self,
        operations: Vec<Operation>,
        durability: Durability,
    ) -> Result<AppendReceipt, StoreError> {
        self.ready()?;
        let sequence = self
            .recovered
            .state
            .last_sequence()
            .checked_add(1)
            .ok_or(StoreError::CompactionRequired)?;
        let must_sync = durability == Durability::Sync
            || operations
                .iter()
                .any(|op| !matches!(op, Operation::UpdateProgress(progress) if !progress.finalize));
        let frame = encode_transaction(&Transaction {
            sequence,
            operations,
        })?;
        let bytes = self
            .wal_bytes()
            .checked_add(frame.len() as u64)
            .ok_or(StoreError::CompactionRequired)?;
        if bytes > self.config.max_wal_bytes || sequence > self.config.max_wal_transactions {
            return Err(StoreError::CompactionRequired);
        }
        let Some(TransactionScan::Complete { transaction, .. }) =
            scan_next_transaction(&frame, sequence)?
        else {
            unreachable!("encoder produced a complete transaction");
        };
        let prepared = self.recovered.state.prepare(transaction)?;
        self.verify_authority()?;
        self.verify_length()?;
        // Install the retained attempt before any fallible write. A returned
        // write/sync error cannot permit another logical append to pass it.
        self.pending = Some(Pending {
            frame,
            boundary: self.recovered.end,
            sequence,
        });
        let pending = self.pending.as_ref().expect("installed pending append");
        self.io
            .write_all(&mut self.recovered.file, &pending.frame)?;
        self.recovered.state.commit(prepared);
        self.recovered.end += pending.frame.len() as u64;
        if must_sync {
            self.io.sync(&self.recovered.file, "append file sync")?;
            self.synced_sequence = sequence;
        }
        self.pending = None;
        Ok(AppendReceipt {
            sequence,
            synced: must_sync,
        })
    }

    /// Syncs all applied progress, including while sources are idle and on drain.
    /// Any failed barrier permanently poisons the store and blocks release.
    /// A later fsync success cannot prove persistence of previously failed writes.
    pub fn sync(&mut self) -> Result<(), StoreError> {
        self.ready()?;
        self.verify_authority()?;
        self.verify_length()?;
        self.io.sync(&self.recovered.file, "explicit file sync")?;
        self.synced_sequence = self.recovered.state.last_sequence();
        Ok(())
    }

    /// Reopens under the retained lock and resolves precisely the failed attempt.
    /// First syncs the original handle after checking its identity and size.
    /// A complete matching frame is synced without duplication; a proven torn
    /// suffix is repaired, then the retained transaction is appended once.
    /// Any failure keeps the store unavailable and the frame retained. This is
    /// never available after a failed sync; reopening is not a writeback repair.
    pub fn resolve_pending(&mut self) -> Result<AppendReceipt, StoreError> {
        self.usable()?;
        if self.pending.is_none() {
            return Err(StoreError::InvalidConfig("no pending append"));
        }
        self.verify_authority()?;
        let pending = self.pending.as_ref().expect("pending attempt was checked");
        self.authority_failed = true;
        // Validate before repair that disk cannot extend beyond this attempt.
        let size = self
            .recovered
            .file
            .metadata()
            .map_err(|e| io("stat pending append", e))?
            .len();
        if size < pending.boundary || size > pending.boundary + pending.frame.len() as u64 {
            return Err(StoreError::AuthorityChanged);
        }
        self.authority_failed = false;
        // Preserve the original open description's writeback-error history.
        // A newly opened handle need not report an already-observed error.
        // This covers both earlier deferred progress and this failed attempt.
        self.io
            .sync(&self.recovered.file, "pre-resolution file sync")?;
        let recovery = recover(
            &self.namespace,
            &self.checkpoint_id,
            self.config,
            &mut self.io,
            Some((
                pending,
                self.recovered.state.generation(),
                self.recovered.wal_offset,
            )),
        );
        if matches!(
            &recovery,
            Err(StoreError::AuthorityChanged | StoreError::Untrusted(_))
        ) {
            self.authority_failed = true;
        }
        let mut recovered = recovery?;
        // Recovery checked generation, immutable boundary, and the exact
        // retained frame before it was allowed to repair or clean anything.
        if recovered.state.last_sequence() != pending.sequence {
            // Preflight again against recovered authority before retrying bytes.
            let Some(TransactionScan::Complete { transaction, .. }) =
                scan_next_transaction(&pending.frame, pending.sequence)?
            else {
                unreachable!("retained frame is complete");
            };
            let prepared = recovered.state.prepare(transaction)?;
            self.io.write_all(&mut recovered.file, &pending.frame)?;
            recovered.state.commit(prepared);
            recovered.end += pending.frame.len() as u64;
        }
        self.io.sync(&recovered.file, "resolved append sync")?;
        let receipt = AppendReceipt {
            sequence: pending.sequence,
            synced: true,
        };
        self.synced_sequence = pending.sequence;
        self.recovered = recovered;
        self.pending = None;
        Ok(receipt)
    }
}

// Shared by startup and live resolution of a failed append:
// - Startup passes no pending guard and reconstructs state from validated disk
//   contents, repairing only a proven incomplete final WAL transaction.
// - Live resolution first syncs the original handle, then supplies the retained
//   attempt, generation, and WAL offset. Recovery checks these before repair or
//   cleanup so it cannot roll back accepted progress or accept a different
//   attempt. The caller re-appends the retained transaction if it is absent.
// Both paths run under the namespace lock and require the recovery barriers
// before returning state. Neither path remediates a known sync failure.
fn recover(
    namespace: &Namespace,
    checkpoint_id: &str,
    config: StoreConfig,
    io_ops: &mut Io,
    pending_guard: Option<(&Pending, u64, u64)>,
) -> Result<Recovered, StoreError> {
    let artifacts = namespace.inventory()?;
    if !artifacts.database {
        return Err(StoreError::AuthoritativeGenerationMissing);
    }
    let mut file = namespace.open_database()?;
    let length = file
        .metadata()
        .map_err(StoreError::AuthoritativeGenerationUnreadable)?
        .len();
    if length > MAX_BYTES {
        return Err(StoreError::RecoveryLimit("1 GiB checkpoint file ceiling"));
    }
    let mut header = [0; CHECKPOINT_HEADER_BYTES];
    read_prefix(&mut file, &mut header)?;
    let boundary = decode_checkpoint_header(&header, config.replay.max_snapshot_bytes)?.wal_offset;
    if length < boundary {
        return Err(StoreError::AuthoritativeGenerationIncomplete);
    }
    let prefix_len = usize::try_from(boundary)
        .map_err(|_| StoreError::RecoveryLimit("platform snapshot size"))?;
    let mut prefix = vec![0; prefix_len];
    prefix[..CHECKPOINT_HEADER_BYTES].copy_from_slice(&header);
    read_prefix(&mut file, &mut prefix[CHECKPOINT_HEADER_BYTES..])?;
    let (mut state, _) = ReplayState::from_checkpoint(&prefix, checkpoint_id, config.replay)?;
    drop(prefix);
    let mut end = boundary;
    let mut frame = Vec::new();
    while end < length {
        let sequence = state
            .last_sequence()
            .checked_add(1)
            .ok_or(ReplayError::SequenceExhausted)?;
        let header_len = (length - end).min(TX_HEADER_BYTES as u64) as usize;
        frame.resize(header_len, 0);
        file.read_exact(&mut frame)
            .map_err(StoreError::AuthoritativeGenerationUnreadable)?;
        let Some(TransactionScan::Incomplete { total_len, .. }) =
            scan_next_transaction(&frame, sequence)?
        else {
            unreachable!("header alone cannot contain a complete frame");
        };
        let Some(total) = total_len else {
            break;
        };
        let available = (length - end).min(total as u64) as usize;
        frame.resize(available, 0);
        file.read_exact(&mut frame[header_len..])
            .map_err(StoreError::AuthoritativeGenerationUnreadable)?;
        if available < total {
            break;
        }
        match state.replay_next(&frame)? {
            ReplayStep::Applied { consumed, .. } => end += consumed as u64,
            _ => unreachable!("complete validated frame must apply or fail"),
        }
    }
    drop(frame);
    // Live repair must first validate the entire previously accepted prefix.
    // Never let a newly incomplete earlier frame move the repair boundary back.
    if let Some((pending, generation, wal_offset)) = pending_guard {
        if state.generation() != generation || boundary != wal_offset {
            return Err(StoreError::AuthorityChanged);
        }
        if state.last_sequence() == pending.sequence {
            if end != pending.boundary + pending.frame.len() as u64 || end != length {
                return Err(StoreError::AuthorityChanged);
            }
            let mut actual = vec![0; pending.frame.len()];
            let _ = file
                .seek(SeekFrom::Start(pending.boundary))
                .map_err(|e| io("seek pending validation", e))?;
            file.read_exact(&mut actual)
                .map_err(|e| io("read pending validation", e))?;
            if actual != pending.frame {
                return Err(StoreError::AuthorityChanged);
            }
        } else {
            if state.last_sequence().checked_add(1) != Some(pending.sequence)
                || end != pending.boundary
            {
                return Err(StoreError::AuthorityChanged);
            }
            // A mechanically torn suffix must also be this exact retained attempt.
            let suffix_len =
                usize::try_from(length - end).map_err(|_| StoreError::AuthorityChanged)?;
            let expected = pending
                .frame
                .get(..suffix_len)
                .ok_or(StoreError::AuthorityChanged)?;
            let mut actual = vec![0; suffix_len];
            let _ = file
                .seek(SeekFrom::Start(end))
                .map_err(|e| io("seek torn attempt", e))?;
            file.read_exact(&mut actual)
                .map_err(|e| io("read torn attempt", e))?;
            if actual != expected {
                return Err(StoreError::AuthorityChanged);
            }
        }
    }
    // Recheck opened authority before any mutation; the directory barrier must
    // precede both torn-tail truncation and temporary cleanup.
    namespace.verify(DB, &file)?;
    if file
        .metadata()
        .map_err(StoreError::AuthoritativeGenerationUnreadable)?
        .len()
        != length
    {
        return Err(StoreError::AuthorityChanged);
    }
    io_ops.sync(&namespace.directory, "recovery directory sync")?;
    if end != length {
        file.set_len(end)
            .map_err(|e| io("truncate torn WAL tail", e))?;
    }
    // This barrier covers normal startup and truncation. It cannot certify bytes
    // affected by a previous process's writeback failure: a newly opened handle
    // may not report that failure. Callers must not automatically reopen after
    // SyncFailed/SyncPoisoned; see the storage remediation contract.
    io_ops.sync(&file, "recovery file sync")?;
    let mut cleaned = false;
    for (exists, name) in [(artifacts.create, CREATE), (artifacts.compact, COMPACT)] {
        if exists {
            namespace.remove(name)?;
            cleaned = true;
        }
    }
    if cleaned {
        io_ops.sync(&namespace.directory, "cleanup directory sync")?;
    }
    let _ = file
        .seek(SeekFrom::Start(end))
        .map_err(|e| io("seek append boundary", e))?;
    Ok(Recovered {
        file,
        state,
        wal_offset: boundary,
        end,
        report: RecoveryReport {
            repaired_tail_bytes: length - end,
            cleaned_temporary: cleaned,
            interrupted_creation: false,
        },
    })
}

fn read_prefix(file: &mut File, bytes: &mut [u8]) -> Result<(), StoreError> {
    file.read_exact(bytes).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            StoreError::AuthoritativeGenerationIncomplete
        } else {
            StoreError::AuthoritativeGenerationUnreadable(e)
        }
    })
}

#[cfg(test)]
mod tests;
