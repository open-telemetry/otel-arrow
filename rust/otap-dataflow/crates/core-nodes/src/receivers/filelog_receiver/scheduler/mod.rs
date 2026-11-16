// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Synchronous reader scheduling. See `../docs/scheduling.md`.
//!
//! `IdentityOwner` supplies readers after committed admission or validated recovery.
//! No method writes checkpoints or authorizes lifecycle transitions.

use super::discovery::{ProbeBudget, ReaderProbeRequest};
use super::identity::{ReaderSeed, lease::LeaseGuard};
use otel_arrow_dfe_filelog_checkpoint::{self as cp, CommittedFrontierGuard};
use std::collections::{BTreeSet, VecDeque};
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::Arc;
use std::task::Waker;
use std::time::{Duration, Instant};

pub(crate) mod batch;
#[cfg(test)]
use batch::Rejected;
pub(crate) use batch::{BatchSink, Flow, Output};
mod framing;
pub(crate) mod io;
use framing::Partial;
pub(crate) use framing::{Frame, FramingConfig, FramingFailure};
pub(crate) use io::{AccessFailure, ReaderIo};
use io::{Evidence, Validation};

/// Opaque runtime key allocated by the scheduler, not a durable file identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub(crate) struct ReaderKey(u64);

/// Independent authority from composition. Clearing one cause cannot clear others.
#[derive(Clone, Copy, Default)]
pub(crate) struct Permission {
    pub read: bool,
    pub process: bool,
    pub memory: bool,
}
impl Permission {
    fn allowed(self) -> bool {
        self.read && self.process && self.memory
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Transition {
    Replaced,
    Shrunk,
    Changed,
    Ineligible,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Fault {
    Configuration,
    DeadlineOverflow,
    Accounting,
    SinkRejected,
    SourceContract,
}
#[derive(Debug)]
pub(crate) enum Reason {
    Progress,
    Idle,
    Paused,
    Cancelled,
    Capacity,
    Descriptors,
    Probe,
    Eof,
    OpenBackoff,
    Retry(std::io::Error),
    Transition(Transition),
    Framing(FramingFailure),
    Source(super::source::FileAccessError),
    Terminal(Fault),
}
#[derive(Debug)]
pub(crate) struct Turn {
    pub key: Option<ReaderKey>,
    pub bytes: usize,
    pub steps: usize,
    pub reason: Reason,
}

struct Limits {
    readers: usize,
    descriptors: usize,
    read_bytes: usize,
    steps: usize,
    partial_bytes: usize,
    eof_interval: Duration,
}

#[derive(Clone, Debug)]
pub(crate) struct Window {
    end: u64,
    data: [u8; 64],
}
impl Window {
    fn empty() -> Self {
        Self {
            end: 0,
            data: [0; 64],
        }
    }
    fn bytes(&self) -> &[u8] {
        &self.data[..self.end.min(64) as usize]
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), Fault> {
        let end = self
            .end
            .checked_add(bytes.len() as u64)
            .ok_or(Fault::SourceContract)?;
        if bytes.len() >= 64 {
            self.data.copy_from_slice(&bytes[bytes.len() - 64..]);
        } else {
            let old = self.bytes().len();
            let keep = old.min(64 - bytes.len());
            self.data.copy_within(old - keep..old, 0);
            self.data[keep..keep + bytes.len()].copy_from_slice(bytes);
        }
        self.end = end;
        Ok(())
    }
    fn guard(&self) -> CommittedFrontierGuard {
        CommittedFrontierGuard::compute(self.end, self.bytes()).expect("exact frontier window")
    }
}

struct Reader<H> {
    file_id: cp::FileId,
    resume: cp::FramingResume,
    key: ReaderKey,
    epoch: u32,
    evidence: Evidence,
    handle: Option<H>,
    validation: Option<Validation>,
    partial: Option<Partial>,
    window: Window,
    initial: bool,
    needs_input: bool,
    check_size: bool,
    service: u64,
    record_start: u64,
    failures: u8,
    last_service: u64,
    pinned: bool,
    halted: bool,
    // A completed probe handoff is waiting for its first partial-state slot.
    awaiting_slot_after_open: bool,
    lease: LeaseGuard,
}

pub(crate) struct Scheduler<I: ReaderIo> {
    io: I,
    limits: Limits,
    framing: FramingConfig,
    matcher: Option<super::multiline_pattern::BoundaryMatcher>,
    readers: Vec<Option<Reader<I::Handle>>>,
    ready: VecDeque<usize>,
    waiting: VecDeque<usize>,
    blocked: VecDeque<usize>,
    probe_deferred: VecDeque<usize>,
    reactivate: usize,
    deadlines: BTreeSet<(Instant, usize)>,
    resident: BTreeSet<(u64, usize)>,
    clock: u64,
    slots: usize,
    grants: BTreeSet<usize>,
    slot_bytes: usize,
    buffer: Vec<u8>,
    probe_budget: Arc<ProbeBudget>,
    probe_waiter: Option<(usize, ReaderProbeRequest)>,
    probe_ready: bool,
    fingerprint_offset: u64,
    // An eviction walk can yield to worker control without rescanning protected
    // prefixes. No reader is serviced until this bounded walk completes.
    selecting: Option<(usize, Option<(u64, usize)>)>,
    open_retry: Option<(Instant, u8)>,
    sink_stopped: bool,
    rejected: Option<Output>,
    terminal: Option<Fault>,
    #[cfg(test)]
    partial_creations: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum SeedRefusal {
    Capacity,
    InvalidSeed,
    Stopped,
}

#[derive(Debug)]
pub(crate) struct RejectedSeed {
    pub reason: SeedRefusal,
    pub seed: ReaderSeed,
}

impl<I: ReaderIo> Scheduler<I> {
    /// Build on the blocking worker from the same validated inputs as `IdentityOwner`.
    pub(crate) fn from_inputs(
        io: I,
        inputs: &super::config::ValidatedInputs,
        steps: usize,
        probe_budget: Arc<ProbeBudget>,
        now: Instant,
    ) -> Result<Self, Fault> {
        let config = inputs.settings();
        let framing = match inputs.grouping() {
            Some(mode) => FramingConfig::Multiline(
                super::multiline::MultilineConfig {
                    line: inputs.line(),
                    max_lines: config.framing.max_multiline_lines,
                    mode,
                },
                inputs.boundary().ok_or(Fault::Configuration)?.clone(),
            ),
            None => FramingConfig::Line(inputs.line()),
        };
        let limits = Limits {
            readers: usize::try_from(config.limits.max_tracked_files)
                .map_err(|_| Fault::Configuration)?,
            descriptors: usize::try_from(config.limits.max_open_files)
                .map_err(|_| Fault::Configuration)?,
            read_bytes: usize::try_from(config.limits.max_read_bytes_per_turn)
                .map_err(|_| Fault::Configuration)?,
            steps,
            partial_bytes: usize::try_from(config.limits.max_partial_state_bytes)
                .map_err(|_| Fault::Configuration)?,
            eof_interval: config.reader.eof_reprobe_interval,
        };
        let mut scheduler = Self::new(io, limits, framing, probe_budget, now)?;
        scheduler.fingerprint_offset = config.identity.ignored_header_bytes;
        Ok(scheduler)
    }

    /// Consume the actual committed admission capability, retaining its lease.
    /// Refusal returns the intact seed; it does not release ownership or admit
    /// a replacement identity. No source open or checkpoint write occurs here.
    #[allow(clippy::result_large_err)]
    pub(crate) fn admit(&mut self, seed: ReaderSeed) -> Result<ReaderKey, RejectedSeed> {
        use super::identity::decision::{Anchor, Decision};
        let decision_ok = match seed.decision {
            Decision::Resume(id) => id == seed.file_id,
            Decision::Register {
                anchor: Anchor::Beginning | Anchor::End,
                ..
            } => true,
            _ => false,
        };
        let refusal = if self.terminal.is_some() {
            Some(SeedRefusal::Stopped)
        } else if self.readers.len() == self.limits.readers {
            Some(SeedRefusal::Capacity)
        } else if !decision_ok || seed.frontier.end_offset() != seed.offset {
            Some(SeedRefusal::InvalidSeed)
        } else {
            None
        };
        if let Some(reason) = refusal {
            return Err(RejectedSeed { reason, seed });
        }
        let id = self.readers.len();
        let key = ReaderKey(id as u64);
        let mut window = Window::empty();
        window.end = seed.offset;
        window.data[..seed.frontier.bytes().len()].copy_from_slice(seed.frontier.bytes());
        let evidence = Evidence {
            locator: seed.lease.locator(),
            fingerprint: seed.fingerprint,
            fingerprint_offset: self.fingerprint_offset,
            applied_offset: seed.offset,
            resume_end: match seed.resume {
                cp::FramingResume::Continuation {
                    record_end_offset, ..
                } => record_end_offset,
                _ => 0,
            },
            applied_guard: window.guard(),
        };
        self.readers.push(Some(Reader {
            key,
            file_id: seed.file_id,
            epoch: seed.epoch,
            resume: seed.resume,
            evidence,
            handle: None,
            validation: None,
            partial: None,
            window,
            initial: seed.offset == 0,
            needs_input: false,
            check_size: false,
            service: 0,
            record_start: seed.offset,
            failures: 0,
            last_service: 0,
            pinned: false,
            halted: false,
            awaiting_slot_after_open: false,
            lease: seed.lease,
        }));
        self.ready.push_back(id);
        Ok(key)
    }

    /// Refresh cached evidence from authoritative applied state after the owner
    /// authorizes progress. This does not apply or persist a checkpoint.
    pub(crate) fn observe_applied(
        &mut self,
        key: ReaderKey,
        record: &cp::SnapshotRecord,
    ) -> Result<(), Fault> {
        let r = self
            .readers
            .get_mut(key.0 as usize)
            .and_then(Option::as_mut)
            .ok_or(Fault::SourceContract)?;
        let locator = r.lease.locator();
        if record.file_id != r.file_id
            || record.file_epoch != r.epoch
            || record.locator
                != (cp::Locator::PosixDevIno {
                    dev: locator.device,
                    ino: locator.inode,
                })
            || record.committed_offset < r.evidence.applied_offset
            || record.committed_offset > r.window.end
            || record.lifecycle_state != cp::LifecycleState::Active
        {
            return Err(Fault::SourceContract);
        }
        if r.validation.is_some()
            && (r.evidence.applied_offset != record.committed_offset
                || r.evidence.applied_guard != record.committed_frontier_guard)
        {
            r.validation = Some(Validation::default());
        }
        r.evidence.applied_offset = record.committed_offset;
        r.evidence.applied_guard = record.committed_frontier_guard;
        Ok(())
    }

    fn new(
        io: I,
        limits: Limits,
        framing: FramingConfig,
        probe_budget: Arc<ProbeBudget>,
        now: Instant,
    ) -> Result<Self, Fault> {
        let slot_bytes = framing.slot_bytes(limits.read_bytes)?;
        if limits.readers == 0
            || limits.descriptors == 0
            || limits.read_bytes == 0
            || limits.steps < 2
            || limits.partial_bytes < slot_bytes
            || limits.eof_interval.is_zero()
            || now.checked_add(limits.eof_interval).is_none()
        {
            return Err(Fault::Configuration);
        }
        let matcher = framing.matcher();
        Ok(Self {
            buffer: vec![0; limits.read_bytes],
            ready: VecDeque::with_capacity(limits.readers),
            waiting: VecDeque::with_capacity(limits.readers),
            blocked: VecDeque::with_capacity(limits.readers),
            probe_deferred: VecDeque::with_capacity(limits.readers),
            readers: Vec::with_capacity(limits.readers),
            io,
            limits,
            framing,
            matcher,
            reactivate: 0,
            deadlines: BTreeSet::new(),
            resident: BTreeSet::new(),
            clock: 0,
            slots: 0,
            grants: BTreeSet::new(),
            slot_bytes,
            probe_budget,
            probe_waiter: None,
            probe_ready: false,
            fingerprint_offset: 0,
            selecting: None,
            open_retry: None,
            sink_stopped: false,
            rejected: None,
            terminal: None,
            #[cfg(test)]
            partial_creations: 0,
        })
    }

    /// Explicit composition notification after batch resolution. Independent
    /// pressure/lifecycle permission is still required on the next turn.
    pub(crate) fn resume_sink(&mut self) {
        if self.rejected.is_none() {
            self.sink_stopped = false;
        }
        self.resources_changed();
    }
    /// Called for probe-release notification, delta resolution or pin changes.
    pub(crate) fn resources_changed(&mut self) {
        self.reactivate = self.blocked.len();
        self.probe_available();
    }
    /// Probe release wakes only its owner, without rescanning descriptor waiters.
    pub(crate) fn probe_available(&mut self) {
        self.probe_ready = self.probe_waiter.is_some();
    }

    fn clear_probe(&mut self, active: Option<usize>) {
        if let Some((id, request)) = self.probe_waiter.take() {
            drop(request);
            if Some(id) != active && self.selecting.map(|s| s.0) != Some(id) {
                self.ready.push_front(id);
            }
        }
        self.probe_ready = false;
    }
    pub(crate) fn next_deadline(&self, permission: Permission) -> Option<Instant> {
        if !permission.allowed() || self.sink_stopped || self.terminal.is_some() {
            None
        } else {
            self.deadlines.first().map(|(at, _)| *at)
        }
    }
    fn park_until(&mut self, id: usize, now: Instant, delay: Duration) -> Result<(), Fault> {
        let at = now.checked_add(delay).ok_or(Fault::DeadlineOverflow)?;
        // A reader leaves its prior queue before reaching here: no stale timer
        // records, cancellation generations, or repeated heap insertions.
        let _ = self.deadlines.insert((at, id));
        Ok(())
    }
    fn reserve_slot(&mut self, id: usize, reader: &mut Reader<I::Handle>) -> bool {
        if reader.partial.is_some() || self.grants.contains(&id) {
            reader.awaiting_slot_after_open = false;
            return true;
        }
        if self.slots == self.limits.partial_bytes / self.slot_bytes || !self.waiting.is_empty() {
            return false;
        }
        let _ = self.grants.insert(id);
        self.slots += 1;
        reader.awaiting_slot_after_open = false;
        true
    }
    fn release_slot(&mut self, id: usize, reader: &mut Reader<I::Handle>) {
        let partial = reader.partial.take();
        let granted = self.grants.remove(&id);
        if partial.is_none() && !granted {
            return;
        }
        drop(partial);
        reader.initial &= reader.window.end == 0;
        if let Some(waiter) = self.waiting.pop_front() {
            let _ = self.grants.insert(waiter);
            self.ready.push_back(waiter);
        } else {
            self.slots -= 1;
        }
    }
    fn touch(&mut self, id: usize, reader: &mut Reader<I::Handle>) -> Result<(), Fault> {
        let _ = self.resident.remove(&(reader.last_service, id));
        self.clock = self.clock.checked_add(1).ok_or(Fault::SourceContract)?;
        reader.last_service = self.clock;
        let _ = self.resident.insert((self.clock, id));
        Ok(())
    }

    pub(crate) fn turn(
        &mut self,
        now: Instant,
        permission: Permission,
        sink: &mut impl BatchSink,
        wake: &Waker,
        mut cancel: impl FnMut() -> bool,
    ) -> Turn {
        let mut turn = Turn {
            key: None,
            bytes: 0,
            steps: 0,
            reason: Reason::Idle,
        };
        if let Some(fault) = self.terminal {
            self.clear_probe(None);
            turn.reason = Reason::Terminal(fault);
            return turn;
        }
        if cancel() {
            self.clear_probe(None);
            turn.reason = Reason::Cancelled;
            return turn;
        }
        if !permission.allowed() || self.sink_stopped {
            self.clear_probe(None);
            self.resources_changed();
            turn.reason = Reason::Paused;
            return turn;
        }
        while turn.steps < self.limits.steps && self.selecting.is_none() && !self.probe_ready {
            if self.probe_waiter.is_none() && !self.probe_deferred.is_empty() {
                let id = self
                    .probe_deferred
                    .pop_front()
                    .expect("deferred probe reader");
                self.ready.push_back(id);
                turn.steps += 1;
            } else if self.reactivate != 0 {
                self.reactivate -= 1;
                if let Some(id) = self.blocked.pop_front() {
                    self.ready.push_back(id);
                }
                turn.steps += 1;
            } else if self.deadlines.first().is_some_and(|(at, _)| *at <= now) {
                let (_, id) = self.deadlines.pop_first().expect("due deadline");
                self.ready.push_back(id);
                turn.steps += 1;
            } else {
                break;
            }
            if cancel() {
                self.clear_probe(None);
                turn.reason = Reason::Cancelled;
                return turn;
            }
        }
        if turn.steps == self.limits.steps {
            turn.reason = Reason::Progress;
            return turn;
        }
        let pending = if self.probe_ready {
            self.probe_ready = false;
            if let Some(previous) = self.selecting.take()
                && self.probe_waiter.as_ref().map(|p| p.0) != Some(previous.0)
            {
                self.ready.push_front(previous.0);
            }
            self.probe_waiter.as_ref().map(|p| (p.0, None))
        } else {
            self.selecting.take()
        };
        let (id, mut after) = match pending {
            Some(v) => v,
            None => match self.ready.pop_front() {
                Some(id) => (id, None),
                None => return turn,
            },
        };
        turn.steps += 1;
        let mut reader = self.readers[id].take().expect("queued reader");
        turn.key = Some(reader.key);
        let reason = self.serve(
            id,
            &mut reader,
            &mut after,
            now,
            sink,
            wake,
            &mut cancel,
            &mut turn,
        );
        let reason = match reason {
            Ok(r) => r,
            Err(f) => {
                self.terminal = Some(f);
                Reason::Terminal(f)
            }
        };
        // A capacity wait does not abandon another reader's probe handoff:
        // that reader can finish opening before reacquiring a partial slot.
        if matches!(
            reason,
            Reason::Paused | Reason::Cancelled | Reason::Terminal(_)
        ) {
            self.clear_probe(Some(id));
        }
        // Resource parking or a halt must not hold an empty payload slot.
        // Unfinished input keeps its reservation. Terminal faults (including
        // rejected output) do not release slots. Releasing empty capacity does
        // not release the lease or authorize retirement.
        if matches!(
            reason,
            Reason::Descriptors
                | Reason::Probe
                | Reason::OpenBackoff
                | Reason::Retry(_)
                | Reason::Transition(_)
                | Reason::Framing(_)
                | Reason::Source(_)
        ) && reader.partial.as_ref().is_none_or(|p| !p.unfinished())
        {
            self.release_slot(id, &mut reader);
        }
        match &reason {
            Reason::Progress | Reason::Cancelled => {
                if self.selecting.is_none() {
                    self.ready.push_back(id);
                }
            }
            Reason::Paused => {
                self.ready.push_front(id);
                self.sink_stopped = true;
            }
            Reason::Capacity => self.waiting.push_back(id),
            Reason::Descriptors => self.blocked.push_back(id),
            Reason::Probe => {
                if self.probe_waiter.as_ref().is_some_and(|p| p.0 != id) {
                    self.probe_deferred.push_back(id);
                }
            }
            Reason::Transition(_) | Reason::Framing(_) | Reason::Source(_) => {
                reader.halted = true;
            }
            _ => {}
        }
        self.readers[id] = Some(reader);
        turn.reason = reason;
        turn
    }

    #[allow(clippy::too_many_arguments)]
    fn serve(
        &mut self,
        id: usize,
        reader: &mut Reader<I::Handle>,
        after: &mut Option<(u64, usize)>,
        now: Instant,
        sink: &mut impl BatchSink,
        wake: &Waker,
        cancel: &mut dyn FnMut() -> bool,
        turn: &mut Turn,
    ) -> Result<Reason, Fault> {
        if sink.preflight(reader.key, reader.epoch) == Flow::Stop {
            return Ok(Reason::Paused);
        }
        if reader.handle.is_none() {
            if let Some((deadline, _)) = self.open_retry
                && now < deadline
            {
                self.park_until(id, now, deadline.duration_since(now))?;
                return Ok(Reason::OpenBackoff);
            }
            // Only the registered waiter may consume its vacant resident slot
            // and permit. Resident readers can still do useful work meanwhile.
            if self.probe_waiter.as_ref().is_some_and(|p| p.0 != id) {
                return Ok(Reason::Probe);
            }
        }
        // Only an already validated resident with no pending framing state can
        // observe unchanged EOF without reserving partial-state capacity.
        let empty_eof = reader.check_size
            && reader.handle.is_some()
            && reader.validation.is_none()
            && reader.partial.is_none()
            && reader.resume == cp::FramingResume::Clean;
        if empty_eof {
            if turn.steps == self.limits.steps {
                self.selecting = Some((id, None));
                return Ok(Reason::Progress);
            }
            if cancel() {
                return Ok(Reason::Cancelled);
            }
            // The serve-entry preflight precedes this source operation.
            turn.steps += 1;
            match self
                .io
                .metadata(reader.handle.as_ref().expect("resident"), cancel)
            {
                Ok((locator, size)) => {
                    if locator != reader.evidence.locator {
                        return Ok(Reason::Transition(Transition::Replaced));
                    }
                    if size < reader.window.end.max(reader.evidence.resume_end) {
                        return Ok(Reason::Transition(Transition::Shrunk));
                    }
                    if size == reader.window.end {
                        // Keep check_size armed and do not refresh last_service:
                        // idle readers remain preferred eviction candidates.
                        self.release_slot(id, reader);
                        self.park_until(id, now, self.limits.eof_interval)?;
                        reader.failures = 0;
                        return Ok(Reason::Eof);
                    }
                    reader.check_size = false;
                }
                Err(error) => return self.access_failure(id, reader, error, now),
            }
        }
        // A pending probe already earned an open opportunity. Complete that
        // handoff before reacquiring partial capacity, so a resident slot holder
        // cannot make us discard it repeatedly. Fresh opens still reserve first.
        let completing_probe =
            reader.handle.is_none() && self.probe_waiter.as_ref().is_some_and(|p| p.0 == id);
        if !completing_probe
            && reader.handle.is_none()
            && reader.partial.is_none()
            && self.grants.contains(&id)
            && self.resident.len() == self.limits.descriptors
            && self.waiting.front().is_some_and(|waiter| {
                self.readers[*waiter]
                    .as_ref()
                    .is_some_and(|r| r.handle.is_some() && r.awaiting_slot_after_open)
            })
        {
            // Avoid crossing an unused memory grant with the next FIFO waiter's
            // newly opened descriptor. Otherwise each can evict the other before
            // either validates or reads when both resource limits are tight.
            self.release_slot(id, reader);
            return Ok(Reason::Capacity);
        }
        if !completing_probe && !self.reserve_slot(id, reader) {
            // Capacity waiting can outlive the observed growth. Recheck the
            // resident source when this reader eventually gets a slot.
            reader.check_size |= empty_eof;
            return Ok(Reason::Capacity);
        }
        if reader.handle.is_none() {
            while self.resident.len() == self.limits.descriptors {
                if turn.steps == self.limits.steps {
                    self.selecting = Some((id, *after));
                    return Ok(Reason::Progress);
                }
                if cancel() {
                    return Ok(Reason::Cancelled);
                }
                turn.steps += 1;
                let victim = match *after {
                    Some(last) => self
                        .resident
                        .range((Excluded(last), Unbounded))
                        .next()
                        .copied(),
                    None => self.resident.first().copied(),
                };
                let Some((stamp, victim)) = victim else {
                    return Ok(Reason::Descriptors);
                };
                *after = Some((stamp, victim));
                let other = self.readers[victim].as_mut().expect("resident reader");
                if other.pinned || sink.unresolved(other.key, other.epoch) {
                    continue;
                }
                drop(other.handle.take());
                other.awaiting_slot_after_open = false;
                other.validation = Some(Validation::default());
                let _ = self.resident.remove(&(stamp, victim));
            }
            if turn.steps == self.limits.steps {
                self.selecting = Some((id, None));
                return Ok(Reason::Progress);
            }
            if cancel() {
                self.selecting = Some((id, None));
                return Ok(Reason::Cancelled);
            }
            let request = if let Some((owner, request)) = self.probe_waiter.take() {
                if owner != id {
                    return Err(Fault::SourceContract);
                }
                request
            } else {
                self.probe_budget
                    .request_reader(wake.clone())
                    .ok_or(Fault::SourceContract)?
            };
            request.register_waker(wake);
            let result = {
                let Some(_reservation) = request.try_reserve() else {
                    self.probe_waiter = Some((id, request));
                    return Ok(Reason::Probe);
                };
                turn.steps += 1;
                self.io.open(reader.key, cancel)
            };
            drop(request);
            match result {
                Ok(handle) => {
                    reader.handle = Some(handle);
                    reader.validation = Some(Validation::default());
                    self.open_retry = None;
                    self.touch(id, reader)?;
                }
                Err(e) => return self.access_failure(id, reader, e, now),
            }
        }
        if completing_probe && !self.reserve_slot(id, reader) {
            reader.awaiting_slot_after_open = true;
            // The probe request is released and the new descriptor is evictable.
            // Leave validation pending until FIFO partial capacity is granted;
            // no evidence or content reads occur while waiting for that slot.
            return Ok(Reason::Capacity);
        }
        while reader.validation.is_some() {
            if turn.steps == self.limits.steps || turn.bytes == self.limits.read_bytes {
                self.selecting = Some((id, None));
                return Ok(Reason::Progress);
            }
            if cancel() {
                return Ok(Reason::Cancelled);
            }
            if sink.preflight(reader.key, reader.epoch) == Flow::Stop {
                return Ok(Reason::Paused);
            }
            turn.steps += 1;
            let n = self.limits.read_bytes - turn.bytes;
            match reader.validation.as_mut().expect("validation").step(
                &mut self.io,
                reader.handle.as_ref().expect("opened"),
                &reader.evidence,
                &reader.window,
                &mut self.buffer[..n],
                cancel,
            ) {
                Ok((done, bytes)) => {
                    turn.bytes += bytes;
                    if done {
                        reader.validation = None;
                        reader.check_size = false;
                        reader.failures = 0;
                    }
                }
                Err(e) => {
                    reader.validation = Some(Validation::default());
                    return self.access_failure(id, reader, e, now);
                }
            }
        }
        if turn.steps == self.limits.steps || turn.bytes == self.limits.read_bytes {
            self.selecting = Some((id, None));
            return Ok(Reason::Progress);
        }
        if reader.check_size {
            if cancel() {
                return Ok(Reason::Cancelled);
            }
            if sink.preflight(reader.key, reader.epoch) == Flow::Stop {
                return Ok(Reason::Paused);
            }
            turn.steps += 1;
            match self
                .io
                .metadata(reader.handle.as_ref().expect("opened"), cancel)
            {
                Ok((locator, size)) => {
                    reader.check_size = false;
                    if locator != reader.evidence.locator {
                        return Ok(Reason::Transition(Transition::Replaced));
                    }
                    if size < reader.window.end.max(reader.evidence.resume_end) {
                        return Ok(Reason::Transition(Transition::Shrunk));
                    }
                }
                Err(error) => return self.access_failure(id, reader, error, now),
            }
        }
        if reader.partial.is_none() {
            #[cfg(test)]
            {
                self.partial_creations += 1;
            }
            reader.partial = Some(self.framing.create(
                reader.window.end,
                reader.initial,
                &reader.window,
                reader.resume,
            )?);
            let _ = self.grants.remove(&id);
            reader.service = 0;
            // A newly constructed framer has no replay to drain. Reading first
            // also ensures a two-step turn can progress after resource parking.
            reader.needs_input = true;
        }
        self.touch(id, reader)?;
        // Process owned remainder first. Moving the Vec avoids copying it or
        // borrowing reader storage across a frame/slot transfer.
        let mut owned = std::mem::take(&mut reader.partial.as_mut().expect("slot").input);
        let mut cursor = reader.partial.as_ref().expect("slot").cursor;
        let mut fresh_len = 0;
        let mut fresh = false;
        let mut read_once = false;
        let result = (|| loop {
            if turn.steps == self.limits.steps {
                break Ok(Reason::Progress);
            }
            if cancel() {
                break Ok(Reason::Cancelled);
            }
            if sink.preflight(reader.key, reader.epoch) == Flow::Stop {
                break Ok(Reason::Paused);
            }
            if !reader.needs_input {
                turn.steps += 1;
                let input = if fresh {
                    &self.buffer[cursor..fresh_len]
                } else {
                    &owned[cursor..]
                };
                let step = reader
                    .partial
                    .as_mut()
                    .expect("slot")
                    .next(input, &mut self.matcher);
                cursor += step.consumed;
                if let Some(error) = step.error {
                    break Ok(Reason::Framing(error));
                }
                if let Some(frame) = step.frame {
                    let end = frame.range().end;
                    let boundary = reader.partial.as_ref().expect("slot").boundary(end)?;
                    let complete = frame.complete_record();
                    if complete {
                        reader.service = reader
                            .service
                            .saturating_add(
                                end.checked_sub(reader.record_start)
                                    .ok_or(Fault::SourceContract)?,
                            )
                            .min(self.limits.read_bytes as u64);
                        reader.record_start = end;
                        reader.resume = cp::FramingResume::Clean;
                    }
                    let output = Output {
                        file_id: reader.file_id,
                        key: reader.key,
                        epoch: reader.epoch,
                        frame,
                        guard: boundary.guard(),
                    };
                    let offered = sink.offer(output);
                    let flow = match offered {
                        Ok(flow) => flow,
                        Err(rejected) => {
                            self.rejected = Some(rejected.output);
                            self.sink_stopped = true;
                            break Err(Fault::SinkRejected);
                        }
                    };
                    if complete
                        && reader.service >= self.limits.read_bytes as u64
                        && !self.waiting.is_empty()
                    {
                        // At a completed record, primitive replay/lookahead belongs
                        // exclusively to later work. There is no terminal error:
                        // this successful primitive step returned exactly one frame.
                        // Revalidation is mandatory even if the descriptor remains.
                        owned = Vec::new();
                        reader.window = boundary;
                        reader.initial = false;
                        reader.validation = Some(Validation::default());
                        self.release_slot(id, reader);
                        break Ok(if flow == Flow::Stop {
                            Reason::Paused
                        } else {
                            Reason::Capacity
                        });
                    }
                    if flow == Flow::Stop {
                        break Ok(Reason::Paused);
                    }
                }
                if step.advanced {
                    continue;
                }
                reader.needs_input = true;
            }
            if read_once || turn.bytes == self.limits.read_bytes {
                break Ok(Reason::Progress);
            }
            if cancel() {
                break Ok(Reason::Cancelled);
            }
            if sink.preflight(reader.key, reader.epoch) == Flow::Stop {
                break Ok(Reason::Paused);
            }
            if turn.steps == self.limits.steps {
                break Ok(Reason::Progress);
            }
            turn.steps += 1;
            let n = self.limits.read_bytes - turn.bytes;
            match self.io.read(
                reader.handle.as_ref().expect("opened"),
                reader.window.end,
                &mut self.buffer[..n],
                cancel,
            ) {
                Ok(0) => {
                    // Empty Vec capacities must actually be released before the
                    // reservation becomes available. No idle flush is authorized.
                    owned = Vec::new();
                    cursor = 0;
                    reader.partial.as_mut().expect("slot").cursor = 0;
                    if !reader.partial.as_ref().expect("slot").unfinished() {
                        self.release_slot(id, reader);
                    }
                    reader.check_size = true;
                    self.park_until(id, now, self.limits.eof_interval)?;
                    reader.failures = 0;
                    break Ok(Reason::Eof);
                }
                Ok(bytes) if bytes <= n => {
                    reader.window.append(&self.buffer[..bytes])?;
                    reader.failures = 0;
                    turn.bytes += bytes;
                    reader.needs_input = false;
                    fresh_len = bytes;
                    cursor = 0;
                    fresh = true;
                    read_once = true;
                }
                Ok(_) => break Err(Fault::SourceContract),
                Err(e) => break self.access_failure(id, reader, e, now),
            }
        })();
        if let Some(partial) = reader.partial.as_mut() {
            if fresh {
                owned.clear();
                if owned.capacity() < fresh_len - cursor {
                    owned
                        .try_reserve_exact(self.limits.read_bytes)
                        .map_err(|_| Fault::Accounting)?;
                }
                owned.extend_from_slice(&self.buffer[cursor..fresh_len]);
                cursor = 0;
            }
            partial.input = owned;
            partial.cursor = cursor;
            let returned = self
                .rejected
                .as_ref()
                .filter(|output| output.key == reader.key && output.epoch == reader.epoch)
                .map_or(0, |output| output.frame.retained_capacity());
            if partial
                .used()
                .checked_add(returned)
                .is_none_or(|used| used > self.slot_bytes)
            {
                return Err(Fault::Accounting);
            }
        }
        result
    }

    fn access_failure(
        &mut self,
        id: usize,
        reader: &mut Reader<I::Handle>,
        error: AccessFailure,
        now: Instant,
    ) -> Result<Reason, Fault> {
        match error {
            AccessFailure::Cancelled => Ok(Reason::Cancelled),
            AccessFailure::Transition(change) => Ok(Reason::Transition(change)),
            AccessFailure::Terminal(error) => Ok(Reason::Source(error)),
            AccessFailure::Environmental(error) => {
                reader.failures = reader.failures.saturating_add(1);
                let global = matches!(error.raw_os_error(), Some(libc::ENFILE | libc::EMFILE));
                let count = if global {
                    self.open_retry.map_or(1, |(_, n)| n.saturating_add(1))
                } else {
                    reader.failures
                };
                let delay =
                    Duration::from_millis((250u64 << count.saturating_sub(1).min(7)).min(30_000));
                if global {
                    self.open_retry = Some((
                        now.checked_add(delay).ok_or(Fault::DeadlineOverflow)?,
                        count,
                    ));
                }
                self.park_until(id, now, delay)?;
                Ok(Reason::Retry(error))
            }
        }
    }
}

#[cfg(test)]
mod tests;
