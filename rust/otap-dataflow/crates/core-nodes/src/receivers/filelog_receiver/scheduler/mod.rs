// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Task 15 synchronous scheduling core. See `../docs/scheduling.md`.
//!
//! Admission is deliberately unavailable in production until ReaderSeed lands.
//! No method writes checkpoints or authorizes lifecycle transitions.

use super::discovery::{ProbeBudget, ReaderProbeRequest};
use otel_arrow_dfe_filelog_checkpoint::CommittedFrontierGuard;
use std::collections::{BTreeSet, VecDeque};
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::Arc;
use std::task::Waker;
use std::time::{Duration, Instant};

mod batch;
#[cfg(test)]
use batch::Rejected;
use batch::{BatchSink, Flow, Output};
mod framing;
mod io;
use framing::{Frame, FramingConfig, FramingFailure, Partial};
use io::{AccessFailure, Evidence, ReaderIo, Validation};

/// Opaque runtime key supplied by identity/composition, not a durable ID generator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReaderKey(u64);

/// Independent authority from composition. Clearing one cause cannot clear others.
#[derive(Clone, Copy, Default)]
struct Permission {
    read: bool,
    process: bool,
    memory: bool,
}
impl Permission {
    fn allowed(self) -> bool {
        self.read && self.process && self.memory
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Transition {
    Replaced,
    Shrunk,
    Changed,
    Ineligible,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fault {
    Configuration,
    DeadlineOverflow,
    Accounting,
    SinkRejected,
    SourceContract,
}
#[derive(Debug)]
enum Reason {
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
struct Turn {
    key: Option<ReaderKey>,
    bytes: usize,
    steps: usize,
    reason: Reason,
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
struct Window {
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
    key: ReaderKey,
    epoch: u64,
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
    // Task 12 integration will add ReaderSeed's actual non-cloneable lease here.
    // Test construction cannot be used by production composition.
    #[cfg(test)]
    lease: Option<Box<dyn std::any::Any>>,
}

struct Scheduler<I: ReaderIo> {
    io: I,
    limits: Limits,
    framing: FramingConfig,
    matcher: Option<super::multiline_pattern::BoundaryMatcher>,
    readers: Vec<Option<Reader<I::Handle>>>,
    ready: VecDeque<usize>,
    waiting: VecDeque<usize>,
    blocked: VecDeque<usize>,
    reactivate: usize,
    deadlines: BTreeSet<(Instant, usize)>,
    resident: BTreeSet<(u64, usize)>,
    clock: u64,
    slots: usize,
    grants: BTreeSet<usize>,
    slot_bytes: usize,
    buffer: Vec<u8>,
    probe_budget: Arc<ProbeBudget>,
    probe_request: Option<ReaderProbeRequest>,
    // An eviction walk can yield to worker control without rescanning protected
    // prefixes. No reader is serviced until this bounded walk completes.
    selecting: Option<(usize, Option<(u64, usize)>)>,
    open_retry: Option<(Instant, u8)>,
    sink_stopped: bool,
    rejected: Option<Output>,
    terminal: Option<Fault>,
}

impl<I: ReaderIo> Scheduler<I> {
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
            probe_request: None,
            selecting: None,
            open_retry: None,
            sink_stopped: false,
            rejected: None,
            terminal: None,
        })
    }

    /// Explicit composition notification after batch resolution. Independent
    /// pressure/lifecycle permission is still required on the next turn.
    fn resume_sink(&mut self) {
        if self.rejected.is_none() {
            self.sink_stopped = false;
        }
        self.resources_changed();
    }
    /// Called for probe-release notification, delta resolution or pin changes.
    fn resources_changed(&mut self) {
        self.reactivate = self.blocked.len();
    }
    fn next_deadline(&self, permission: Permission) -> Option<Instant> {
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
    fn release_slot(&mut self, reader: &mut Reader<I::Handle>) {
        drop(reader.partial.take());
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

    fn turn(
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
            turn.reason = Reason::Terminal(fault);
            return turn;
        }
        if cancel() {
            self.probe_request = None;
            turn.reason = Reason::Cancelled;
            return turn;
        }
        if !permission.allowed() || self.sink_stopped {
            self.probe_request = None;
            self.resources_changed();
            turn.reason = Reason::Paused;
            return turn;
        }
        while turn.steps < self.limits.steps && self.selecting.is_none() {
            if self.reactivate != 0 {
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
                turn.reason = Reason::Cancelled;
                return turn;
            }
        }
        if turn.steps == self.limits.steps {
            turn.reason = Reason::Progress;
            return turn;
        }
        let (id, mut after) = match self.selecting.take() {
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
            Reason::Descriptors | Reason::Probe => self.blocked.push_back(id),
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
        if reader.partial.is_none() {
            // Existing waiters must never be bypassed by a returning holder.
            let granted = self.grants.remove(&id);
            if !granted
                && (self.slots == self.limits.partial_bytes / self.slot_bytes
                    || !self.waiting.is_empty())
            {
                return Ok(Reason::Capacity);
            }
            reader.partial = Some(self.framing.create(
                reader.window.end,
                reader.initial,
                &reader.window,
                self.limits.read_bytes,
            )?);
            reader.service = 0;
            reader.needs_input = false;
            if !granted {
                self.slots += 1;
            }
        }
        if reader.handle.is_none() {
            if let Some((deadline, _)) = self.open_retry
                && now < deadline
            {
                self.park_until(id, now, deadline.duration_since(now))?;
                return Ok(Reason::OpenBackoff);
            }
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
                other.validation = Some(Validation::default());
                let _ = self.resident.remove(&(stamp, victim));
            }
            if turn.steps == self.limits.steps {
                return Ok(Reason::Progress);
            }
            if cancel() {
                self.probe_request = None;
                return Ok(Reason::Cancelled);
            }
            if self.probe_request.is_none() {
                self.probe_request = self.probe_budget.request_reader(wake.clone());
            }
            let Some(request) = self.probe_request.take() else {
                return Err(Fault::SourceContract);
            };
            let result = {
                let Some(_reservation) = request.try_reserve() else {
                    self.probe_request = Some(request);
                    return Ok(Reason::Probe);
                };
                turn.steps += 1;
                self.io.open(reader.key, cancel)
                // On success, the handle is already charged to the vacant
                // resident slot before the transient reservation is released.
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
                    if size < reader.window.end {
                        return Ok(Reason::Transition(Transition::Shrunk));
                    }
                }
                Err(error) => return self.access_failure(id, reader, error, now),
            }
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
                    }
                    let output = Output {
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
                        self.release_slot(reader);
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
                        self.release_slot(reader);
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
                let global = matches!(error.raw_os_error(), Some(23 | 24)); // ENFILE / EMFILE on Linux.
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
