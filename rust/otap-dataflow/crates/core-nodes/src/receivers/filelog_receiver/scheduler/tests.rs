// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::{
    decoder::{Encoding, OnDecodeError},
    framer::{LineBody, LineConfig, OversizeBehavior},
    multiline::{GroupingMode, MultilineConfig},
    multiline_pattern::{BoundaryPattern, PatternMode},
    source::FileLocator,
};
use super::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::Wake;

#[derive(Default)]
struct WakeCount(AtomicUsize);
impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        let _ = self.0.fetch_add(1, Ordering::Relaxed);
    }
}
struct FileData {
    bytes: Vec<u8>,
    locator: FileLocator,
}
struct Handle {
    id: usize,
    open: Rc<Cell<usize>>,
}
impl Drop for Handle {
    fn drop(&mut self) {
        self.open.set(self.open.get() - 1);
    }
}
struct FakeIo {
    files: Vec<FileData>,
    opens: Rc<Cell<usize>>,
    peak: usize,
    reads: usize,
    open_error: Option<i32>,
    read_error: Option<i32>,
    calls: Rc<RefCell<Vec<(usize, u64, usize)>>>,
}
impl ReaderIo for FakeIo {
    type Handle = Handle;
    fn open(
        &mut self,
        key: ReaderKey,
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<Handle, AccessFailure> {
        if cancel() {
            return Err(AccessFailure::Cancelled);
        }
        if let Some(errno) = self.open_error.take() {
            return Err(AccessFailure::Environmental(
                std::io::Error::from_raw_os_error(errno),
            ));
        }
        self.opens.set(self.opens.get() + 1);
        self.peak = self.peak.max(self.opens.get());
        Ok(Handle {
            id: key.0 as usize,
            open: Rc::clone(&self.opens),
        })
    }
    fn metadata(
        &mut self,
        h: &Handle,
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<(FileLocator, u64), AccessFailure> {
        if cancel() {
            return Err(AccessFailure::Cancelled);
        }
        let f = &self.files[h.id];
        Ok((f.locator, f.bytes.len() as u64))
    }
    fn read(
        &mut self,
        h: &Handle,
        offset: u64,
        bytes: &mut [u8],
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<usize, AccessFailure> {
        if cancel() {
            return Err(AccessFailure::Cancelled);
        }
        if let Some(errno) = self.read_error.take() {
            return Err(AccessFailure::Environmental(
                std::io::Error::from_raw_os_error(errno),
            ));
        }
        self.reads += 1;
        self.calls.borrow_mut().push((h.id, offset, bytes.len()));
        let f = &self.files[h.id].bytes;
        let offset = (offset as usize).min(f.len());
        let n = bytes.len().min(f.len() - offset);
        bytes[..n].copy_from_slice(&f[offset..offset + n]);
        Ok(n)
    }
}
#[derive(Default)]
struct Sink {
    frames: Vec<Output>,
    blocked: bool,
    stop_at: Option<usize>,
    reject: bool,
    hold: bool,
    unresolved: BTreeSet<(u64, u32)>,
    preflights: usize,
}
impl BatchSink for Sink {
    fn preflight(&mut self, _: ReaderKey, _: u32) -> Flow {
        self.preflights += 1;
        if self.blocked {
            Flow::Stop
        } else {
            Flow::Continue
        }
    }
    fn offer(&mut self, output: Output) -> Result<Flow, Rejected> {
        assert!(!self.blocked, "no second carry-over");
        if self.reject {
            return Err(Rejected { output });
        }
        if self.hold {
            let _ = self.unresolved.insert((output.key.0, output.epoch));
        }
        self.frames.push(output);
        if self.stop_at == Some(self.frames.len()) {
            self.blocked = true;
            Ok(Flow::Stop)
        } else {
            Ok(Flow::Continue)
        }
    }
    fn unresolved(&self, key: ReaderKey, epoch: u32) -> bool {
        self.unresolved.contains(&(key.0, epoch))
    }
}
fn line() -> LineConfig {
    LineConfig {
        encoding: Encoding::Utf8,
        on_decode_error: OnDecodeError::PreserveRaw,
        max_line_bytes: 128,
        max_record_bytes: 128,
        oversize: OversizeBehavior::Split,
    }
}
fn permission() -> Permission {
    Permission {
        read: true,
        process: true,
        memory: true,
    }
}
fn scheduler(
    files: &[&[u8]],
    bytes: usize,
    steps: usize,
    slots: usize,
    descriptors: usize,
    config: FramingConfig,
) -> Scheduler<FakeIo> {
    let io = FakeIo {
        files: files
            .iter()
            .enumerate()
            .map(|(n, bytes)| FileData {
                bytes: bytes.to_vec(),
                locator: FileLocator {
                    device: 1,
                    inode: n as u64 + 1,
                },
            })
            .collect(),
        opens: Rc::new(Cell::new(0)),
        peak: 0,
        reads: 0,
        open_error: None,
        read_error: None,
        calls: Rc::new(RefCell::new(Vec::new())),
    };
    let budget = config.slot_bytes(bytes).unwrap();
    let limits = Limits {
        readers: files.len(),
        descriptors,
        read_bytes: bytes,
        steps,
        partial_bytes: budget * slots,
        eof_interval: Duration::from_millis(250),
    };
    let mut s = Scheduler::new(
        io,
        limits,
        config,
        Arc::new(ProbeBudget::default()),
        Instant::now(),
    )
    .unwrap_or_else(|e| panic!("{e:?}"));
    let registry = Arc::new(super::super::identity::lease::LeaseRegistry::default());
    let owner = registry.owner("scheduler-test", files.len()).unwrap();
    for (id, _) in files.iter().enumerate() {
        let seed = ReaderSeed {
            decision: super::super::identity::decision::Decision::Register {
                anchor: super::super::identity::decision::Anchor::Beginning,
                superseded: None,
                reset: false,
            },
            file_id: cp::FileId::from_bytes((id as u128 + 1).to_le_bytes()),
            epoch: 7,
            offset: 0,
            resume: cp::FramingResume::Clean,
            frontier: cp::CommittedFrontierWindow::empty(),
            fingerprint: Vec::new(),
            lease: owner.acquire(s.io.files[id].locator).unwrap(),
        };
        assert_eq!(s.admit(seed).unwrap(), ReaderKey(id as u64));
    }
    s
}
fn tick(s: &mut Scheduler<FakeIo>, sink: &mut Sink, now: Instant) -> Turn {
    let t = s.turn(now, permission(), sink, Waker::noop(), || false);
    assert!(
        t.bytes <= s.limits.read_bytes && t.steps <= s.limits.steps,
        "{t:?}"
    );
    assert!(s.io.opens.get() <= s.limits.descriptors);
    assert!(s.deadlines.len() <= s.readers.len());
    assert!(s.slots * s.slot_bytes <= s.limits.partial_bytes);
    assert!(!matches!(t.reason, Reason::Terminal(_)), "{t:?}");
    t
}
fn frames(s: &mut Scheduler<FakeIo>, sink: &mut Sink, count: usize, now: Instant) {
    for _ in 0..2000 {
        if sink.frames.len() >= count {
            return;
        }
        let t = tick(s, sink, now);
        assert!(
            !matches!(
                t.reason,
                Reason::Transition(_) | Reason::Framing(_) | Reason::Source(_)
            ),
            "{t:?}"
        );
    }
    panic!("wanted {count} frames, got {}", sink.frames.len());
}
fn body(output: &Output) -> &[u8] {
    let b = match &output.frame {
        Frame::Line(f) => &f.body,
        Frame::Multiline(f) => &f.body,
    };
    match b {
        LineBody::Text(s) => s.as_bytes(),
        LineBody::Bytes(b) => b,
    }
}

/// Scenario: A continuously ready file runs beside two short files.
/// Guarantees: Round-robin grants recurring turns without waiting for hot-file EOF.
#[test]
fn hot_reader_cannot_starve_ready_readers() {
    let hot = b"hot\n".repeat(1000);
    let mut s = scheduler(
        &[&hot, b"one\n", b"two\n"],
        8,
        16,
        3,
        3,
        FramingConfig::Line(line()),
    );
    let mut sink = Sink::default();
    let now = Instant::now();
    frames(&mut s, &mut sink, 10, now);
    assert!(sink.frames.iter().take(10).any(|f| f.key == ReaderKey(1)));
    assert!(sink.frames.iter().take(10).any(|f| f.key == ReaderKey(2)));
    assert!(s.io.files[0].bytes.len() as u64 > s.readers[0].as_ref().unwrap().window.end);
}

/// Scenario: UTF-16 decoding, long lines and BOM replay share tiny turn limits.
/// Guarantees: Small byte and processing turns preserve decoded bodies and original source ranges.
#[test]
fn utf16_source_and_step_budgets_preserve_ranges() {
    let mut c = line();
    c.encoding = Encoding::Utf16Le;
    let input = [0xff, 0xfe, b'a', 0, b'\n', 0, b'b', 0, b'\n', 0];
    let mut s = scheduler(&[&input], 3, 3, 1, 1, FramingConfig::Line(c));
    let mut sink = Sink::default();
    let now = Instant::now();

    for _ in 0..100 {
        let _ = tick(&mut s, &mut sink, now);

        if sink.frames.len() == 2 {
            break;
        }
    }
    assert_eq!(
        sink.frames.iter().map(body).collect::<Vec<_>>(),
        [b"a", b"b"]
    );

    assert_eq!(sink.frames[0].frame.range().start, 0);
    assert_eq!(sink.frames[0].frame.range().end, 6);
}

/// Scenario: EOF and environmental retry expire while read permission is absent.
/// Guarantees: Pauses perform no I/O, expose no spinning deadline, and retain one timer per reader.
#[test]
fn deadlines_survive_pause_without_spinning() {
    let mut s = scheduler(&[b""], 8, 32, 1, 1, FramingConfig::Line(line()));
    let mut sink = Sink::default();
    let now = Instant::now();
    assert!(matches!(tick(&mut s, &mut sink, now).reason, Reason::Eof));
    let deadline = s.next_deadline(permission(), &sink).unwrap();
    let reads = s.io.reads;
    let paused = Permission {
        read: false,
        ..permission()
    };
    for _ in 0..20 {
        assert!(matches!(
            s.turn(deadline, paused, &mut sink, Waker::noop(), || false)
                .reason,
            Reason::Paused
        ));
    }
    assert_eq!(s.io.reads, reads);
    assert_eq!(s.deadlines.len(), 1);
    assert_eq!(s.next_deadline(paused, &sink), None);
    s.io.read_error = Some(11);
    assert!(matches!(
        tick(&mut s, &mut sink, deadline).reason,
        Reason::Retry(_)
    ));
    assert_eq!(s.deadlines.len(), 1);
    let retry = s.next_deadline(permission(), &sink).unwrap();
    assert!(matches!(
        s.turn(retry, paused, &mut sink, Waker::noop(), || false)
            .reason,
        Reason::Paused
    ));
    assert_eq!(s.deadlines.len(), 1);
    assert!(matches!(tick(&mut s, &mut sink, retry).reason, Reason::Eof));
}

/// Scenario: Two long partial records alternate through one resident descriptor.
/// Guarantees: Eviction preserves payload, source frontier and exact emitted bodies.
#[test]
fn descriptor_turnover_preserves_partial_progress() {
    let a = [b'a'; 100];
    let b = [b'b'; 100];
    let mut aa = a.to_vec();
    aa.push(b'\n');
    let mut bb = b.to_vec();
    bb.push(b'\n');
    let mut s = scheduler(&[&aa, &bb], 16, 32, 2, 1, FramingConfig::Line(line()));
    let mut sink = Sink::default();
    frames(&mut s, &mut sink, 2, Instant::now());
    assert_eq!(s.io.peak, 1);
    assert!(sink.frames.iter().any(|f| body(f) == a));
    assert!(sink.frames.iter().any(|f| body(f) == b));
}

/// Scenario: Reopening a partially consumed source observes replacement, shrink or rewrite.
/// Guarantees: Typed transition outcomes stop that reader before mixing retained bytes with new input.
#[test]
fn reopen_validates_locator_size_and_provisional_guard() {
    for change in [
        Transition::Replaced,
        Transition::Shrunk,
        Transition::Changed,
    ] {
        let mut s = scheduler(
            &[b"abcdef\n", b"other\n"],
            3,
            32,
            2,
            1,
            FramingConfig::Line(line()),
        );
        let mut sink = Sink::default();
        let now = Instant::now();
        let _ = tick(&mut s, &mut sink, now);
        let _ = tick(&mut s, &mut sink, now);
        match change {
            Transition::Replaced => s.io.files[0].locator.inode += 10,
            Transition::Shrunk => s.io.files[0].bytes.clear(),
            Transition::Changed => s.io.files[0].bytes[0] = b'x',
            _ => unreachable!(),
        }
        let mut found = false;
        for _ in 0..30 {
            let t = tick(&mut s, &mut sink, now);
            if matches!(t.reason, Reason::Transition(c) if c == change) {
                found = true;
                break;
            }
        }
        assert!(found, "{change:?}");
        assert!(sink.frames.iter().all(|f| f.key != ReaderKey(0)));
        assert_eq!(s.readers[0].as_ref().unwrap().evidence.applied_offset, 0);
        assert_eq!(s.readers[0].as_ref().unwrap().epoch, 7);
    }
}

/// Scenario: Open descriptors are unavailable at the OS level.
/// Guarantees: EMFILE backs off new opens globally, retains slots and never quarantines identities.
#[test]
fn descriptor_exhaustion_uses_bounded_global_backoff() {
    let mut s = scheduler(&[b"a\n", b"b\n"], 8, 32, 2, 2, FramingConfig::Line(line()));
    s.io.open_error = Some(24);
    let mut sink = Sink::default();
    let now = Instant::now();
    assert!(matches!(
        tick(&mut s, &mut sink, now).reason,
        Reason::Retry(_)
    ));
    assert!(matches!(
        tick(&mut s, &mut sink, now).reason,
        Reason::OpenBackoff
    ));
    assert_eq!(s.io.opens.get(), 0);
    assert_eq!(s.deadlines.len(), 2);
    assert_eq!(s.slots, 0);
    frames(&mut s, &mut sink, 2, now + Duration::from_millis(250));
    assert!(s.readers.iter().flatten().all(|r| !r.halted));
}

/// Scenario: One partial slot serves multiple continuously backlogged readers.
/// Guarantees: Completed source-byte allowance yields capacity FIFO without needing EOF.
#[test]
fn partial_capacity_yields_at_clean_record_boundaries() {
    let hot = b"a\n".repeat(100);
    let mut s = scheduler(
        &[&hot, b"one\n", b"two\n"],
        8,
        16,
        1,
        3,
        FramingConfig::Line(line()),
    );
    let mut sink = Sink::default();
    frames(&mut s, &mut sink, 12, Instant::now());
    let one = sink
        .frames
        .iter()
        .position(|f| f.key == ReaderKey(1))
        .unwrap();
    let two = sink
        .frames
        .iter()
        .position(|f| f.key == ReaderKey(2))
        .unwrap();
    assert!(one < two && two < 12);
    assert!(s.slots <= 1);
    for f in sink.frames.iter().filter(|f| f.key == ReaderKey(0)) {
        assert_eq!(body(f), b"a");
    }
}

/// Scenario: A slot holder reaches EOF with an unfinished record and idle flush disabled.
/// Guarantees: Other readers park without reads or extra opens; the existing payload stays charged.
#[test]
fn unfinished_eof_retains_capacity_without_reading_waiters() {
    let mut s = scheduler(
        &[b"partial", b"ready\n"],
        16,
        64,
        1,
        2,
        FramingConfig::Line(line()),
    );
    let mut sink = Sink::default();
    let now = Instant::now();
    for _ in 0..10 {
        let _ = tick(&mut s, &mut sink, now);
    }
    assert!(sink.frames.is_empty());
    assert_eq!(s.slots, 1);
    assert_eq!(s.waiting.len(), 1);
    assert!(s.io.calls.borrow().iter().all(|(id, _, _)| *id == 0));
    assert_eq!(s.io.opens.get(), 1);
    assert!(
        s.readers[0]
            .as_ref()
            .unwrap()
            .partial
            .as_ref()
            .unwrap()
            .used()
            <= s.slot_bytes
    );
}

/// Scenario: Multiline start-pattern lookahead has consumed the next header when capacity yields.
/// Guarantees: Replay resumes at the completed boundary with exact ranges and guard evidence.
#[test]
fn multiline_lookahead_yield_replays_only_later_work() {
    let c = FramingConfig::Multiline(
        MultilineConfig {
            line: line(),
            max_lines: 100,
            mode: GroupingMode::Start,
        },
        BoundaryPattern::compile("^H", PatternMode::Text).unwrap(),
    );
    let mut s = scheduler(
        &[b"H1\nbody\nH2\nend\nH3\n", b"HotherXX\nHnext\n"],
        8,
        16,
        1,
        2,
        c,
    );
    let mut sink = Sink::default();
    frames(&mut s, &mut sink, 3, Instant::now());
    let first = sink.frames.iter().find(|f| f.key == ReaderKey(0)).unwrap();
    assert_eq!(body(first), b"H1\nbody");
    assert_eq!(first.frame.range().end, 8);
    assert_eq!(
        first.guard,
        CommittedFrontierGuard::compute(8, b"H1\nbody\n").unwrap()
    );
    assert!(sink.frames.iter().any(|f| f.key == ReaderKey(1)));
    assert!(sink.frames.iter().any(|f| body(f) == b"H2\nend"));
}

/// Scenario: Discovery holds both transient descriptors when a reader requests reopening.
/// Guarantees: Release wakes the reader and discovery cannot reacquire ahead of its registered request.
#[test]
fn probe_contention_has_notification_and_reader_handoff() {
    let mut s = scheduler(&[b"a\n"], 8, 32, 1, 1, FramingConfig::Line(line()));
    let budget = Arc::clone(&s.probe_budget);
    let discovery = budget.try_reserve().unwrap();
    let count = Arc::new(WakeCount::default());
    let wake = Waker::from(Arc::clone(&count));
    let mut sink = Sink::default();
    let now = Instant::now();
    assert!(matches!(
        s.turn(now, permission(), &mut sink, &wake, || false).reason,
        Reason::Probe
    ));
    assert_eq!(s.io.opens.get(), 0);
    drop(discovery);
    assert_eq!(count.0.load(Ordering::Relaxed), 1);
    assert!(budget.try_reserve().is_none());
    s.resources_changed();
    frames(&mut s, &mut sink, 1, now);
    assert!(budget.try_reserve().is_some());
}

/// Scenario: Cancellation arrives with a pending reader probe request.
/// Guarantees: Priority is relinquished and no source read or descriptor acquisition occurs.
#[test]
fn cancellation_releases_probe_priority() {
    let mut s = scheduler(&[b"a\n"], 8, 32, 1, 1, FramingConfig::Line(line()));
    let budget = Arc::clone(&s.probe_budget);
    let discovery = budget.try_reserve().unwrap();
    let mut sink = Sink::default();
    let now = Instant::now();
    assert!(matches!(tick(&mut s, &mut sink, now).reason, Reason::Probe));
    drop(discovery);
    assert!(matches!(
        s.turn(now, permission(), &mut sink, Waker::noop(), || true)
            .reason,
        Reason::Cancelled
    ));
    assert!(budget.try_reserve().is_some());
    assert_eq!(s.io.reads, 0);
}

/// Scenario: Batch preflight refuses work before a reader has opened or framed anything.
/// Guarantees: Known batch limits prevent source operations and partial payload allocation.
#[test]
fn preflight_runs_before_source_and_replay() {
    let mut s = scheduler(&[b"a\nb\n"], 8, 3, 1, 1, FramingConfig::Line(line()));
    let mut sink = Sink {
        blocked: true,
        ..Sink::default()
    };
    let now = Instant::now();
    assert!(matches!(
        tick(&mut s, &mut sink, now).reason,
        Reason::Paused
    ));
    assert_eq!(s.io.reads, 0);
    assert_eq!(s.slots, 0);
    sink.blocked = false;
    s.resume_sink();
    frames(&mut s, &mut sink, 1, now);
    sink.blocked = true;
    let read = s.io.reads;
    let end = s.readers[0].as_ref().unwrap().window.end;
    assert!(matches!(
        tick(&mut s, &mut sink, now).reason,
        Reason::Paused
    ));
    assert_eq!(sink.frames.len(), 1);
    assert_eq!(s.io.reads, read);
    assert_eq!(s.readers[0].as_ref().unwrap().window.end, end);
}

/// Scenario: The sink owns one carry-over and other pause reasons remain active after its resolution.
/// Guarantees: Stop consumes exactly one output and resuming the sink does not grant read or pressure permission.
#[test]
fn stop_ownership_and_independent_pause_reasons() {
    let mut s = scheduler(&[b"a\nb\nc\n"], 8, 64, 1, 1, FramingConfig::Line(line()));
    let mut sink = Sink {
        stop_at: Some(1),
        ..Sink::default()
    };
    let now = Instant::now();
    frames(&mut s, &mut sink, 1, now);
    let reads = s.io.reads;
    for _ in 0..4 {
        assert!(matches!(
            tick(&mut s, &mut sink, now).reason,
            Reason::Paused
        ));
    }
    assert_eq!(sink.frames.len(), 1);
    assert_eq!(s.io.reads, reads);
    assert_eq!(s.slots, 1);
    sink.blocked = false;
    s.resume_sink();
    for paused in [
        Permission {
            read: false,
            ..permission()
        },
        Permission {
            process: false,
            ..permission()
        },
        Permission {
            memory: false,
            ..permission()
        },
    ] {
        assert!(matches!(
            s.turn(now, paused, &mut sink, Waker::noop(), || false)
                .reason,
            Reason::Paused
        ));
    }
    frames(&mut s, &mut sink, 3, now);
    assert_eq!(
        sink.frames.iter().map(body).collect::<Vec<_>>(),
        [b"a", b"b", b"c"]
    );
}

/// Scenario: A sink unexpectedly rejects an owned frame.
/// Guarantees: The exact output and reservation remain retained with a sticky terminal disposition.
#[test]
fn unexpected_sink_failure_preserves_output_and_accounting() {
    let mut s = scheduler(&[b"a\nb\n"], 8, 64, 1, 1, FramingConfig::Line(line()));
    let mut sink = Sink {
        reject: true,
        ..Sink::default()
    };
    let now = Instant::now();
    let t = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    assert!(matches!(t.reason, Reason::Terminal(Fault::SinkRejected)));
    assert_eq!(body(s.rejected.as_ref().unwrap()), b"a");
    assert_eq!(s.slots, 1);
    assert!(sink.frames.is_empty());
    let read = s.io.reads;
    s.resume_sink();
    assert!(matches!(
        s.turn(now, permission(), &mut sink, Waker::noop(), || false)
            .reason,
        Reason::Terminal(Fault::SinkRejected)
    ));
    assert_eq!(s.io.reads, read);
    assert_eq!(s.readers[0].as_ref().unwrap().evidence.applied_offset, 0);
}

/// Scenario: The least-recently-served handle owns unresolved batch progress.
/// Guarantees: Descriptor pressure parks work until composition resolves the delta and notifies the scheduler.
#[test]
fn unresolved_delta_vetoes_eviction() {
    let mut s = scheduler(&[b"a\n", b"b\n"], 8, 64, 2, 1, FramingConfig::Line(line()));
    let mut sink = Sink {
        hold: true,
        ..Sink::default()
    };
    let now = Instant::now();
    frames(&mut s, &mut sink, 1, now);
    assert!(matches!(
        tick(&mut s, &mut sink, now).reason,
        Reason::Descriptors
    ));
    assert!(s.readers[0].as_ref().unwrap().handle.is_some());
    assert_eq!(sink.frames.len(), 1);
    sink.unresolved.clear();
    s.resources_changed();
    frames(&mut s, &mut sink, 2, now);
    assert_eq!(sink.frames[1].key, ReaderKey(1));
}

/// Scenario: A live ownership token spans eviction, a retained batch and a deadline wait.
/// Guarantees: None of these scheduling operations releases ownership or changes supplied identity state.
#[test]
fn lease_and_identity_survive_eviction_and_ack_waits() {
    let registry = Arc::new(super::super::identity::lease::LeaseRegistry::default());
    let owner = registry.owner("lease-test", 1).unwrap();
    let competing = registry.owner("competitor", 1).unwrap();
    let mut s = scheduler(
        &[b"long partial", b"b\n"],
        4,
        32,
        2,
        1,
        FramingConfig::Line(line()),
    );
    let locator = s.readers[0].as_ref().unwrap().lease.locator();
    s.readers[0].as_mut().unwrap().lease = owner.acquire(locator).unwrap();
    let mut sink = Sink {
        stop_at: Some(1),
        ..Sink::default()
    };
    frames(&mut s, &mut sink, 1, Instant::now());
    assert!(competing.acquire(locator).is_err());
    let r = s.readers[0].as_ref().unwrap();
    assert_eq!(r.key, ReaderKey(0));
    assert_eq!(r.epoch, 7);
    assert_eq!(r.evidence.applied_offset, 0);
    drop(s);
    assert!(competing.acquire(locator).is_ok());
}

/// Scenario: Cancellation becomes visible immediately after a completed source read.
/// Guarantees: The bytes remain owned and the next authorized turn emits each frame exactly once.
#[test]
fn cancellation_between_read_and_framing_retains_remainder() {
    let mut s = scheduler(&[b"a\nb\n"], 8, 64, 1, 1, FramingConfig::Line(line()));
    let calls = Rc::clone(&s.io.calls);
    let mut sink = Sink::default();
    let now = Instant::now();
    let t = s.turn(now, permission(), &mut sink, Waker::noop(), || {
        !calls.borrow().is_empty()
    });
    assert!(matches!(t.reason, Reason::Cancelled));
    assert!(sink.frames.is_empty());
    assert_eq!(
        s.readers[0]
            .as_ref()
            .unwrap()
            .partial
            .as_ref()
            .unwrap()
            .input,
        b"a\nb\n"
    );
    frames(&mut s, &mut sink, 2, now);
    assert_eq!(
        sink.frames.iter().map(body).collect::<Vec<_>>(),
        [b"a", b"b"]
    );
}

/// Scenario: A resident source is rewritten after its reader yields its partial slot.
/// Guarantees: Revalidation occurs despite residency and rejects changed boundary evidence before replay.
#[test]
fn resident_capacity_yield_requires_revalidation() {
    let mut s = scheduler(
        &[b"aa\nbb\ncc\n", b"other\n"],
        3,
        12,
        1,
        2,
        FramingConfig::Line(line()),
    );
    let mut sink = Sink::default();
    let now = Instant::now();
    for _ in 0..100 {
        let _ = tick(&mut s, &mut sink, now);
        let r = s.readers[0].as_ref().unwrap();
        if r.partial.is_none() && r.window.end != 0 {
            break;
        }
    }
    assert!(s.readers[0].as_ref().unwrap().partial.is_none());
    assert!(s.readers[0].as_ref().unwrap().handle.is_some());
    s.io.files[0].bytes[0] = b'x';
    let mut changed = false;
    for _ in 0..200 {
        let t = tick(&mut s, &mut sink, now);
        if matches!(t.reason, Reason::Transition(Transition::Changed)) {
            changed = true;
            break;
        }
    }
    assert!(changed);
    assert!(
        sink.frames
            .iter()
            .filter(|f| f.key == ReaderKey(0))
            .all(|f| f.frame.range().end <= 6)
    );
}

/// Scenario: A long split record spans several service allowances while another reader waits.
/// Guarantees: Nonfinal fragments keep their reservation and fragment identity until the logical record completes.
#[test]
fn split_fragments_do_not_release_partial_slot_early() {
    let mut c = line();
    c.max_line_bytes = 4;
    c.max_record_bytes = 4;
    let mut s = scheduler(
        &[b"abcdefghijkl\n", b"z\n"],
        4,
        12,
        1,
        2,
        FramingConfig::Line(c),
    );
    let mut sink = Sink::default();
    frames(&mut s, &mut sink, 1, Instant::now());
    assert!(!sink.frames[0].frame.complete_record());
    assert_eq!(s.slots, 1);
    assert!(s.readers[0].as_ref().unwrap().partial.is_some());
    frames(&mut s, &mut sink, 4, Instant::now());
    let first_other = sink
        .frames
        .iter()
        .position(|f| f.key == ReaderKey(1))
        .unwrap();
    assert!(sink.frames[first_other - 1].frame.complete_record());
    assert!(
        sink.frames[..first_other]
            .iter()
            .all(|f| f.key == ReaderKey(0) && f.epoch == 7)
    );
}

/// Scenario: Repeated environmental reads fail for longer than the exponential ramp.
/// Guarantees: Delay saturates at 30 seconds with one timer and no identity or checkpoint mutation.
#[test]
fn environmental_retry_saturates_without_quarantine() {
    let mut s = scheduler(&[b"a\n"], 8, 64, 1, 1, FramingConfig::Line(line()));
    let mut sink = Sink::default();
    let mut now = Instant::now();
    // First error is the content read after empty initial validation.
    for attempt in 0..12 {
        s.io.read_error = Some(5);
        let t = tick(&mut s, &mut sink, now);
        assert!(matches!(t.reason, Reason::Retry(_)), "{t:?}");
        let deadline = s.next_deadline(permission(), &sink).unwrap();
        assert_eq!(
            deadline.duration_since(now),
            Duration::from_millis((250u64 << attempt.min(7)).min(30_000))
        );
        assert_eq!(s.deadlines.len(), 1);
        now = deadline;
    }
    let r = s.readers[0].as_ref().unwrap();
    assert!(!r.halted);
    assert_eq!(r.evidence.applied_offset, 0);
    assert_eq!(r.epoch, 7);
}

/// Scenario: An empty EOF reader had retained reusable framing allocations after a raw-body output.
/// Guarantees: EOF drops actual owned capacity before releasing the slot to a waiting reader.
#[test]
fn empty_eof_releases_payload_capacity() {
    let mut s = scheduler(&[b"abc\n"], 8, 64, 1, 1, FramingConfig::Line(line()));
    let mut sink = Sink::default();
    let now = Instant::now();
    frames(&mut s, &mut sink, 1, now);
    for _ in 0..10 {
        if matches!(tick(&mut s, &mut sink, now).reason, Reason::Eof) {
            break;
        }
    }
    assert_eq!(s.slots, 0);
    assert!(s.readers[0].as_ref().unwrap().partial.is_none());
    s.io.files[0].bytes.extend_from_slice(b"next\n");
    frames(&mut s, &mut sink, 2, now + Duration::from_millis(250));
    assert_eq!(body(&sink.frames[1]), b"next");
    assert_eq!(sink.frames[1].frame.range().start, 4);
}

/// Scenario: An invalid deadline cannot be represented by the scheduling clock.
/// Guarantees: Configuration fails before any source activity and timer insertion cannot wrap.
#[test]
fn deadline_overflow_is_explicit() {
    let mut s = scheduler(&[b""], 8, 32, 1, 1, FramingConfig::Line(line()));
    assert_eq!(
        s.park_until(0, Instant::now(), Duration::MAX),
        Err(Fault::DeadlineOverflow)
    );
    assert!(s.deadlines.is_empty());
    assert_eq!(s.io.reads, 0);
}

/// Scenario: Several protected residents precede an eligible descriptor under a small step budget.
/// Guarantees: Eviction scanning resumes across control turns and selects the least-recently-served eligible handle.
#[test]
fn eviction_walk_is_bounded_and_resumable() {
    let mut s = scheduler(
        &[b"a\n", b"b\n", b"c\n", b"d\n"],
        8,
        64,
        4,
        3,
        FramingConfig::Line(line()),
    );
    let mut sink = Sink::default();
    let now = Instant::now();
    frames(&mut s, &mut sink, 3, now);
    for id in 0..2 {
        s.readers[id].as_mut().unwrap().pinned = true;
    }
    s.limits.steps = 2;
    frames(&mut s, &mut sink, 4, now);
    assert!(s.readers[0].as_ref().unwrap().handle.is_some());
    assert!(s.readers[1].as_ref().unwrap().handle.is_some());
    assert!(s.readers[2].as_ref().unwrap().handle.is_none());
    assert_eq!(s.io.peak, 3);
}

/// Scenario: A real Linux source is reopened after retaining a prefix and a committed guard.
/// Guarantees: The production read adapter validates exact evidence and reports rewritten bytes as a transition.
#[test]
fn linux_adapter_checks_real_source_evidence() {
    use super::super::source::{SourceFile, SymlinkPolicy};
    use super::io::LinuxIo;
    use std::fs::{File, OpenOptions};
    use std::os::unix::fs::FileExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("log");
    std::fs::write(&path, b"prefix\nline\n").unwrap();
    let parent = File::open(dir.path()).unwrap();
    let source =
        SourceFile::open_at(&parent, c"log", SymlinkPolicy::Reject, None, || false).unwrap();
    let evidence = Evidence {
        locator: source.locator(),
        fingerprint: b"prefix".to_vec(),
        fingerprint_offset: 0,
        applied_offset: 7,
        resume_end: 0,
        applied_guard: CommittedFrontierGuard::compute(7, b"prefix\n").unwrap(),
    };
    drop(source);
    // The test owns a private directory with one known allowed file. Production
    // supplies its existing pin-bound path/target/namespace policy at this seam.
    let mut io = LinuxIo(|_: ReaderKey, cancel: &mut dyn FnMut() -> bool| {
        SourceFile::open_at(
            &parent,
            c"log",
            SymlinkPolicy::Reject,
            Some(evidence.locator),
            cancel,
        )
        .map_err(AccessFailure::from)
    });
    let handle = io.open(ReaderKey(4), &mut || false).unwrap();
    let mut window = Window::empty();
    window.append(b"prefix\nline").unwrap();
    let mut validate = Validation::default();
    let mut scratch = [0; 3];
    let mut complete = false;
    for _ in 0..32 {
        if validate
            .step(
                &mut io,
                &handle,
                &evidence,
                &window,
                &mut scratch,
                &mut || false,
            )
            .unwrap()
            .0
        {
            complete = true;
            break;
        }
    }
    assert!(complete);
    OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .write_all_at(b"X", 8)
        .unwrap();
    let mut validate = Validation::default();
    let mut changed = false;
    for _ in 0..32 {
        match validate.step(
            &mut io,
            &handle,
            &evidence,
            &window,
            &mut scratch,
            &mut || false,
        ) {
            Err(AccessFailure::Transition(Transition::Changed)) => {
                changed = true;
                break;
            }
            Ok((false, _)) => {}
            result => panic!("{result:?}"),
        }
    }
    assert!(changed);
}

/// Scenario: Capacity yielding resumes text at a nonzero boundary containing a BOM-shaped scalar.
/// Guarantees: Re-admission never restarts BOM stripping across UTF-8 or either UTF-16 encoding.
#[test]
fn capacity_replay_preserves_noninitial_bom_and_encoding() {
    for encoding in [Encoding::Utf8, Encoding::Utf16Le, Encoding::Utf16Be] {
        let encode = |text: &str| -> Vec<u8> {
            match encoding {
                Encoding::Utf8 => text.as_bytes().to_vec(),
                Encoding::Utf16Le => text.encode_utf16().flat_map(u16::to_le_bytes).collect(),
                Encoding::Utf16Be => text.encode_utf16().flat_map(u16::to_be_bytes).collect(),
                _ => unreachable!(),
            }
        };
        let data = encode("a\n\u{feff}B\nend\n");
        let other = encode("other\n");
        let mut c = line();
        c.encoding = encoding;
        let bytes = if encoding == Encoding::Utf8 { 2 } else { 4 };
        let mut s = scheduler(&[&data, &other], bytes, 8, 1, 2, FramingConfig::Line(c));
        let mut sink = Sink::default();
        frames(&mut s, &mut sink, 3, Instant::now());
        let outputs: Vec<_> = sink
            .frames
            .iter()
            .filter(|f| f.key == ReaderKey(0))
            .collect();
        assert_eq!(body(outputs[0]), b"a");
        assert_eq!(body(outputs[1]), "\u{feff}B".as_bytes());
        assert_eq!(outputs[0].frame.range().end, outputs[1].frame.range().start);
    }
}

/// Scenario: Malformed data occurs in a discarded truncate tail after an earlier complete frame.
/// Guarantees: The scheduler returns a typed failure, keeps earlier owned output, and never invents progress or a truncated frame.
#[test]
fn truncate_decode_failure_preserves_ordered_disposition() {
    let mut c = line();
    c.max_line_bytes = 4;
    c.max_record_bytes = 4;
    c.oversize = OversizeBehavior::Truncate;
    c.on_decode_error = OnDecodeError::Fail;
    let mut s = scheduler(&[b"ok\nlonglong\xff\n"], 4, 8, 1, 1, FramingConfig::Line(c));
    let mut sink = Sink {
        hold: true,
        ..Sink::default()
    };
    let now = Instant::now();
    let mut failed = false;
    for _ in 0..100 {
        let t = tick(&mut s, &mut sink, now);
        if matches!(t.reason, Reason::Framing(_)) {
            failed = true;
            break;
        }
    }
    assert!(failed);
    assert_eq!(sink.frames.len(), 1);
    assert_eq!(body(&sink.frames[0]), b"ok");
    assert!(sink.unresolved(ReaderKey(0), 7));
    assert_eq!(s.slots, 1);
    assert_eq!(s.readers[0].as_ref().unwrap().evidence.applied_offset, 0);
}

/// Scenario: Discovery releases its conversion on another thread after reader registration.
/// Guarantees: The registered wake is delivered and the reader owns the next acquisition without polling.
#[test]
fn cross_thread_probe_release_preserves_reader_priority() {
    use std::sync::mpsc::sync_channel;
    let budget = Arc::new(ProbeBudget::default());
    let (ready_tx, ready_rx) = sync_channel(1);
    let (release_tx, release_rx) = sync_channel(1);
    let count = Arc::new(WakeCount::default());
    std::thread::scope(|scope| {
        let discovery_budget = Arc::clone(&budget);
        let discovery = scope.spawn(move || {
            let held = discovery_budget.try_reserve().unwrap();
            ready_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(30)).unwrap();
            drop(held);
            assert!(discovery_budget.try_reserve().is_none());
        });
        ready_rx.recv_timeout(Duration::from_secs(30)).unwrap();
        let request = budget
            .request_reader(Waker::from(Arc::clone(&count)))
            .unwrap();
        assert!(request.try_reserve().is_none());
        release_tx.send(()).unwrap();
        discovery.join().unwrap();
        assert_eq!(count.0.load(Ordering::Relaxed), 1);
        let held = request.try_reserve().unwrap();
        drop(held);
        drop(request);
    });
    assert!(budget.try_reserve().is_some());
}

/// Scenario: A validated resident reader remains at EOF across a reprobe, then shrinks.
/// Guarantees: Reprobes avoid redundant fingerprint reads while metadata still detects same-handle shrink.
#[test]
fn resident_eof_reprobe_avoids_redundant_evidence_reads() {
    let mut s = scheduler(&[b"prefix\n"], 32, 64, 1, 1, FramingConfig::Line(line()));
    s.readers[0].as_mut().unwrap().evidence.fingerprint = b"prefix".to_vec();
    let mut sink = Sink::default();
    let now = Instant::now();
    frames(&mut s, &mut sink, 1, now);
    assert!(matches!(tick(&mut s, &mut sink, now).reason, Reason::Eof));
    let count = s.io.calls.borrow().len();
    let deadline = s.next_deadline(permission(), &sink).unwrap();
    assert!(matches!(
        tick(&mut s, &mut sink, deadline).reason,
        Reason::Eof
    ));
    assert_eq!(&s.io.calls.borrow()[count..], &[(0, 7, 32)]);
    s.io.files[0].bytes.truncate(2);
    let deadline = s.next_deadline(permission(), &sink).unwrap();
    assert!(matches!(
        tick(&mut s, &mut sink, deadline).reason,
        Reason::Transition(Transition::Shrunk)
    ));
    assert_eq!(sink.frames.len(), 1);
}

/// Scenario: A controlled reader starts at applied offset 100 and is evicted midway through its next record.
/// Guarantees: Reopen preserves provisional progress and rolling evidence without replaying excluded source bytes or changing applied progress.
#[test]
fn nonzero_start_and_short_reads_preserve_both_frontiers() {
    let mut source = vec![b'z'; 100];
    source.extend_from_slice(&[b'a'; 50]);
    source.push(b'\n');
    let mut s = scheduler(
        &[&source, b"other\n"],
        16,
        32,
        2,
        1,
        FramingConfig::Line(line()),
    );
    let reader = s.readers[0].as_mut().unwrap();
    reader.evidence.fingerprint = source[..8].to_vec();
    reader.evidence.applied_offset = 100;
    reader.evidence.applied_guard = CommittedFrontierGuard::compute(100, &source[36..100]).unwrap();
    reader.window = Window {
        end: 100,
        data: source[36..100].try_into().unwrap(),
    };
    reader.initial = false;
    reader.record_start = 100;
    let mut sink = Sink::default();
    frames(&mut s, &mut sink, 2, Instant::now());
    let output = sink.frames.iter().find(|f| f.key == ReaderKey(0)).unwrap();
    assert_eq!(body(output), [b'a'; 50]);
    assert_eq!(output.frame.range().start, 100);
    assert_eq!(output.frame.range().end, 151);
    assert_eq!(
        output.guard,
        CommittedFrontierGuard::compute(151, &source[87..151]).unwrap()
    );
    assert_eq!(s.readers[0].as_ref().unwrap().evidence.applied_offset, 100);
    assert_eq!(s.readers[0].as_ref().unwrap().epoch, 7);
    assert_eq!(s.io.peak, 1);
}

/// Scenario: Two backlogged readers share one descriptor with the minimum two-step budget.
/// Guarantees: A reader that spends its last step evicting retains the next open opportunity; both emit before EOF.
#[test]
fn minimum_budget_eviction_cannot_starve_competing_reader() {
    let hot = b"a\n".repeat(1000);
    let mut s = scheduler(&[&hot, &hot], 8, 2, 2, 1, FramingConfig::Line(line()));
    let mut sink = Sink::default();
    let now = Instant::now();
    frames(&mut s, &mut sink, 12, now);
    for key in [ReaderKey(0), ReaderKey(1)] {
        assert!(sink.frames.iter().take(12).any(|f| f.key == key));
        assert!(s.readers[key.0 as usize].as_ref().unwrap().window.end < hot.len() as u64);
    }
}

/// Scenario: A split frame leaves an owned overflow unit and no fresh input to consume.
/// Guarantees: One remaining processing step advances that replay only; uncharged replay would read or emit later work in the same turn.
#[test]
fn pending_empty_input_replay_spends_the_processing_budget() {
    let mut config = line();
    config.max_line_bytes = 4;
    config.max_record_bytes = 4;
    let mut s = scheduler(&[b"abcdX\n"], 5, 64, 1, 1, FramingConfig::Line(config));
    let mut sink = Sink {
        stop_at: Some(1),
        ..Sink::default()
    };
    let now = Instant::now();
    frames(&mut s, &mut sink, 1, now);
    let partial = s.readers[0].as_ref().unwrap().partial.as_ref().unwrap();
    assert!(partial.input.is_empty());
    assert!(!sink.frames[0].frame.complete_record());
    let reads = s.io.reads;
    sink.blocked = false;
    s.resume_sink();
    s.limits.steps = 2;
    let turn = tick(&mut s, &mut sink, now);
    assert_eq!(turn.bytes, 0);
    assert_eq!(turn.steps, 2);
    assert_eq!(s.io.reads, reads);
    assert_eq!(sink.frames.len(), 1);
    frames(&mut s, &mut sink, 2, now);
    assert_eq!(body(&sink.frames[1]), b"X");
}

/// Scenario: A probe waiter is notified with several other ready readers and a vacant resident slot.
/// Guarantees: Only that waiter consumes the handoff, and probe notification does not reactivate descriptor waiters.
#[test]
fn notified_probe_belongs_to_its_waiting_reader() {
    let mut s = scheduler(
        &[b"a\n", b"b\n", b"c\n"],
        8,
        32,
        3,
        1,
        FramingConfig::Line(line()),
    );
    let budget = Arc::clone(&s.probe_budget);
    let held = budget.try_reserve().unwrap();
    let mut sink = Sink::default();
    let now = Instant::now();
    assert!(matches!(tick(&mut s, &mut sink, now).reason, Reason::Probe));
    assert_eq!(s.slots, 0);
    assert!(matches!(
        tick(&mut s, &mut sink, now).reason,
        Reason::Descriptors
    ));
    assert_eq!(s.slots, 0);
    assert_eq!(s.blocked.len(), 1);
    drop(held);
    s.probe_available();
    let t = tick(&mut s, &mut sink, now);
    assert_eq!(t.key, Some(ReaderKey(0)));
    assert_eq!(sink.frames[0].key, ReaderKey(0));
    assert_eq!(s.blocked.len(), 1);
    assert!(budget.try_reserve().is_some());
}

/// Scenario: One reader waits for a probe while a resident reader stops or rejects output.
/// Guarantees: The stopping turn releases the pending request immediately while preserving retained frame ownership and leases.
#[test]
fn sink_stop_and_terminal_fault_release_other_readers_probe_request() {
    for reject in [false, true] {
        let mut s = scheduler(
            &[b"a\nb\n", b"c\n"],
            2,
            32,
            2,
            2,
            FramingConfig::Line(line()),
        );
        let mut sink = Sink::default();
        let now = Instant::now();
        frames(&mut s, &mut sink, 1, now);
        let budget = Arc::clone(&s.probe_budget);
        let held = budget.try_reserve().unwrap();
        assert!(matches!(tick(&mut s, &mut sink, now).reason, Reason::Probe));
        drop(held);
        assert!(budget.try_reserve().is_none());
        // Do not deliver the wake yet: resident work reaches its sink first.
        sink.reject = reject;
        sink.stop_at = Some(2);
        for _ in 0..16 {
            let t = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
            if matches!(
                t.reason,
                Reason::Paused | Reason::Terminal(Fault::SinkRejected)
            ) {
                break;
            }
        }
        assert!(s.sink_stopped);
        assert!(s.probe_waiter.is_none());
        assert!(budget.try_reserve().is_some());
        if reject {
            assert_eq!(body(s.rejected.as_ref().unwrap()), b"b");
        }
    }
}

/// Scenario: Cancellation is first observed while draining a resource notification.
/// Guarantees: It releases a separately pending probe request on that return, without needing another turn.
#[test]
fn cancellation_during_reactivation_releases_probe_request() {
    let mut s = scheduler(&[b"a\n", b"b\n"], 8, 32, 2, 1, FramingConfig::Line(line()));
    let budget = Arc::clone(&s.probe_budget);
    let held = budget.try_reserve().unwrap();
    let mut sink = Sink::default();
    let now = Instant::now();
    assert!(matches!(tick(&mut s, &mut sink, now).reason, Reason::Probe));
    assert!(matches!(
        tick(&mut s, &mut sink, now).reason,
        Reason::Descriptors
    ));
    drop(held);
    s.reactivate = s.blocked.len();
    let mut checks = 0;
    let t = s.turn(now, permission(), &mut sink, Waker::noop(), || {
        checks += 1;
        checks == 2
    });
    assert!(matches!(t.reason, Reason::Cancelled));
    assert!(budget.try_reserve().is_some());
}

/// Scenario: A nonresident file cannot evict a pinned resident that has released its empty EOF slot.
/// Guarantees: Descriptor waiting holds no empty reservation, allowing the pinned reader to read appended data.
#[test]
fn descriptor_wait_does_not_pin_empty_partial_capacity() {
    let mut s = scheduler(&[b"", b"b\n"], 8, 32, 1, 1, FramingConfig::Line(line()));
    let mut sink = Sink::default();
    let now = Instant::now();
    assert!(matches!(tick(&mut s, &mut sink, now).reason, Reason::Eof));
    s.readers[0].as_mut().unwrap().pinned = true;
    assert!(matches!(
        tick(&mut s, &mut sink, now).reason,
        Reason::Descriptors
    ));
    assert_eq!(s.slots, 0);
    s.io.files[0].bytes.extend_from_slice(b"new\n");
    frames(&mut s, &mut sink, 1, now + Duration::from_millis(250));
    assert_eq!(body(&sink.frames[0]), b"new");
}

/// Scenario: A partial slot is reserved without any source remainder or fresh decoded bytes.
/// Guarantees: Payload capacity is lazy; admission itself does not allocate the maximum read buffer or history ceiling.
#[test]
fn partial_payload_is_lazy_until_used() {
    let config = FramingConfig::Line(line());
    let partial = config
        .create(0, true, &Window::empty(), cp::FramingResume::Clean)
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(partial.input.capacity(), 0);
    assert_eq!(partial.used(), size_of::<Partial>());
}

/// Scenario: Task 12 commits admission, the scheduler emits data, and authorized progress is applied by the identity owner.
/// Guarantees: ReaderSeed supplies the real ID/epoch/lease and ignored-header evidence; the scheduler neither writes progress nor releases ownership during Stop.
#[test]
fn committed_reader_seed_and_applied_evidence_integrate_with_scheduler() {
    use super::super::{
        config::{Config, StartAt},
        identity::{
            Admission, Candidate, CommittedAdmission, IdentityOwner, decision::Context,
            lease::LeaseRegistry,
        },
        source::{SourceFile, SymlinkPolicy},
    };
    use super::io::LinuxIo;
    use std::{
        fs::File,
        os::{fd::AsFd, unix::fs::PermissionsExt},
        time::SystemTime,
    };
    let source = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = source.path().join("log");
    let bytes = b"header\nbody\nnext\n";
    std::fs::write(&path, bytes).unwrap();
    let parent = File::open(source.path()).unwrap();
    let open = || SourceFile::open_at(&parent, c"log", SymlinkPolicy::Reject, None, || false);
    let locator = open().unwrap().locator();
    let mut config = Config {
        include: vec![path.to_str().unwrap().into()],
        start_at: StartAt::Beginning,
        ..Config::default()
    };
    config.identity.ignored_header_bytes = 7;
    config.identity.fingerprint_bytes = 16;
    config.limits.max_tracked_files = 4;
    config.limits.max_open_files = 1;
    let (inputs, _) = config
        .compile("g", "p", "filelog", source.path())
        .unwrap()
        .into_parts();
    let root =
        cp::store::StateRoot::from_borrowed_fd(File::open(state.path()).unwrap().as_fd()).unwrap();
    let registry = Arc::new(LeaseRegistry::default());
    let other = registry.owner("other", 1).unwrap();
    let store = cp::store::CheckpointStore::open(
        &root,
        inputs.checkpoint_id(),
        inputs.store_config().unwrap(),
    )
    .unwrap();
    let mut owner = IdentityOwner::new(
        store,
        registry
            .owner(
                "owner",
                IdentityOwner::required_lease_capacity(&inputs).unwrap(),
            )
            .unwrap(),
        &inputs,
    )
    .unwrap();
    let budget = Arc::new(ProbeBudget::default());
    // An outstanding scheduler request deliberately takes precedence over a
    // new opportunistic identity preparation on the same read worker.
    let request = budget.request_reader(Waker::noop().clone()).unwrap();
    let candidate = || Candidate {
        path: &path,
        locator,
        size: bytes.len() as u64,
        fingerprint: &bytes[7..],
        context: Context::Unrelated,
    };
    assert!(matches!(
        owner
            .prepare(candidate(), &budget, SystemTime::now(), open, || false)
            .unwrap(),
        Admission::Deferred
    ));
    drop(request);
    let mut admission = owner
        .prepare(candidate(), &budget, SystemTime::now(), open, || false)
        .unwrap();
    for _ in 0..32 {
        let Admission::Scanning(mut scan) = admission else {
            break;
        };
        if owner
            .advance_association(&mut scan, || false)
            .unwrap()
            .complete
        {
            admission = owner
                .finish_prepare(scan, &budget, SystemTime::now(), open, || false)
                .unwrap();
        } else {
            admission = Admission::Scanning(scan);
        }
    }
    let plan = match admission {
        Admission::Ready(plan) => plan,
        other => panic!("{other:?}"),
    };
    let seed = match owner.commit(plan).unwrap() {
        CommittedAdmission::Reader(seed) => seed,
        other => panic!("{other:?}"),
    };
    let file_id = seed.file_id;
    let epoch = seed.epoch;
    let io = LinuxIo(|_: ReaderKey, cancel: &mut dyn FnMut() -> bool| {
        SourceFile::open_at(
            &parent,
            c"log",
            SymlinkPolicy::Reject,
            Some(locator),
            cancel,
        )
        .map_err(AccessFailure::from)
    });
    let mut scheduler = Scheduler::from_inputs(io, &inputs, 64, budget, Instant::now())
        .unwrap_or_else(|e| panic!("{e:?}"));
    let key = scheduler.admit(seed).unwrap();
    let mut sink = Sink {
        stop_at: Some(1),
        ..Sink::default()
    };
    for _ in 0..20 {
        let _ = scheduler.turn(
            Instant::now(),
            permission(),
            &mut sink,
            Waker::noop(),
            || false,
        );
        if !sink.frames.is_empty() {
            break;
        }
    }
    assert_eq!(body(&sink.frames[0]), b"header");
    assert_eq!(sink.frames[0].file_id, file_id);
    assert_eq!(sink.frames[0].epoch, epoch);
    assert!(other.acquire(locator).is_err());
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&file_id)
            .unwrap()
            .committed_offset,
        0
    );
    let _ = owner
        .append_checkpoint(
            vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id,
                expected_committed_offset: 0,
                expected_file_epoch: epoch,
                new_committed_offset: 7,
                new_committed_frontier_guard: sink.frames[0].guard,
                new_framing_resume: cp::FramingResume::Clean,
                new_last_seen_time_unix_nano: 1,
                finalize: false,
            })],
            cp::store::Durability::Sync,
        )
        .unwrap();
    let record = owner.state().unwrap().get(&file_id).unwrap();
    scheduler.observe_applied(key, record).unwrap();
    assert_eq!(
        scheduler.readers[0]
            .as_ref()
            .unwrap()
            .evidence
            .applied_offset,
        7
    );
    let mut stale = record.clone();
    stale.file_epoch += 1;
    assert_eq!(
        scheduler.observe_applied(key, &stale),
        Err(Fault::SourceContract)
    );
    drop(scheduler);
    assert!(other.acquire(locator).is_ok());
}

/// Scenario: An admitted ReaderSeed carries a recovered split continuation.
/// Guarantees: The scheduler preserves its original fragment index, file ID, epoch and starting frontier instead of beginning a new record.
#[test]
fn reader_seed_preserves_recovered_split_continuation() {
    use super::super::identity::{decision::Decision, lease::LeaseRegistry};
    let mut config = line();
    config.max_line_bytes = 4;
    config.max_record_bytes = 4;
    let mut s = scheduler(&[b"abcdef\n"], 8, 32, 1, 1, FramingConfig::Line(config));
    s.readers.clear();
    s.ready.clear();
    let registry = Arc::new(LeaseRegistry::default());
    let owner = registry.owner("recovery", 1).unwrap();
    let file_id = cp::FileId::from_bytes([42; 16]);
    let seed = ReaderSeed {
        decision: Decision::Resume(file_id),
        file_id,
        epoch: 9,
        offset: 4,
        resume: cp::FramingResume::Continuation {
            record_start_offset: 0,
            record_end_offset: 0,
            next_fragment_index: 1,
        },
        frontier: cp::CommittedFrontierWindow::new(4, b"abcd").unwrap(),
        fingerprint: b"abcd".to_vec(),
        lease: owner.acquire(s.io.files[0].locator).unwrap(),
    };
    let _ = s.admit(seed).unwrap();
    let mut sink = Sink::default();
    frames(&mut s, &mut sink, 1, Instant::now());
    let output = &sink.frames[0];
    assert_eq!(body(output), b"ef");
    assert_eq!(output.file_id, file_id);
    assert_eq!(output.epoch, 9);
    assert_eq!(output.frame.range().start, 4);
    let Frame::Line(frame) = &output.frame else {
        panic!("expected line");
    };
    let fragment = frame.fragment.unwrap();
    assert_eq!(fragment.index, 1);
    assert_eq!(fragment.record_start_offset, 0);
    assert!(fragment.is_last);
    assert_eq!(s.readers[0].as_ref().unwrap().evidence.applied_offset, 4);
}

/// Scenario: A recovered multiline continuation has a known end beyond the file's current size.
/// Guarantees: Reopen reports shrink before reading or emitting any part of that continuation.
#[test]
fn reader_seed_known_end_is_revalidated_before_continuation() {
    use super::super::identity::{decision::Decision, lease::LeaseRegistry};
    let config = FramingConfig::Multiline(
        MultilineConfig {
            line: line(),
            max_lines: 10,
            mode: GroupingMode::Start,
        },
        BoundaryPattern::compile("^H", PatternMode::Text).unwrap(),
    );
    let mut s = scheduler(&[b"abcdef"], 8, 32, 1, 1, config);
    s.readers.clear();
    s.ready.clear();
    let registry = Arc::new(LeaseRegistry::default());
    let owner = registry.owner("known-end", 1).unwrap();
    let file_id = cp::FileId::from_bytes([44; 16]);
    let seed = ReaderSeed {
        decision: Decision::Resume(file_id),
        file_id,
        epoch: 2,
        offset: 4,
        resume: cp::FramingResume::Continuation {
            record_start_offset: 0,
            record_end_offset: 10,
            next_fragment_index: 1,
        },
        frontier: cp::CommittedFrontierWindow::new(4, b"abcd").unwrap(),
        fingerprint: b"abcd".to_vec(),
        lease: owner.acquire(s.io.files[0].locator).unwrap(),
    };
    let _ = s.admit(seed).unwrap();
    let mut sink = Sink::default();
    let t = tick(&mut s, &mut sink, Instant::now());
    assert!(matches!(t.reason, Reason::Transition(Transition::Shrunk)));
    assert!(sink.frames.is_empty());
    assert_eq!(s.io.reads, 0);
}

/// Scenario: A committed seed cannot fit the scheduler's logical-reader capacity.
/// Guarantees: Refusal returns the actual seed and its lease, without source I/O or a terminal scheduler fault.
#[test]
fn admission_refusal_returns_seed_ownership() {
    use super::super::identity::{decision::Decision, lease::LeaseRegistry};
    let mut s = scheduler(&[b"a\n"], 8, 32, 1, 1, FramingConfig::Line(line()));
    let registry = Arc::new(LeaseRegistry::default());
    let owner = registry.owner("refused", 1).unwrap();
    let other = registry.owner("other", 1).unwrap();
    let locator = FileLocator {
        device: 2,
        inode: 2,
    };
    let file_id = cp::FileId::from_bytes([45; 16]);
    let seed = ReaderSeed {
        decision: Decision::Resume(file_id),
        file_id,
        epoch: 3,
        offset: 0,
        resume: cp::FramingResume::Clean,
        frontier: cp::CommittedFrontierWindow::empty(),
        fingerprint: Vec::new(),
        lease: owner.acquire(locator).unwrap(),
    };
    let rejected = s.admit(seed).unwrap_err();
    assert_eq!(rejected.reason, SeedRefusal::Capacity);
    assert_eq!(rejected.seed.file_id, file_id);
    assert!(other.acquire(locator).is_err());
    assert_eq!(s.io.reads, 0);
    assert!(s.terminal.is_none());
    drop(rejected);
    assert!(other.acquire(locator).is_ok());
}

/// Scenario: The read worker acquires and releases its own probe conversion.
/// Guarantees: Release does not generate a spurious wake for that same worker.
#[test]
fn reader_conversion_does_not_wake_itself() {
    let budget = Arc::new(ProbeBudget::default());
    let count = Arc::new(WakeCount::default());
    let request = budget
        .request_reader(Waker::from(Arc::clone(&count)))
        .unwrap();
    let held = request.try_reserve().unwrap();
    drop(held);
    drop(request);
    assert_eq!(count.0.load(Ordering::Relaxed), 0);
    assert!(budget.try_reserve().is_some());
}

/// Scenario: Both hot readers encounter probe contention on every reopen with one descriptor and two processing steps.
/// Guarantees: Dropping empty reservations does not repeatedly restart setup without reaching source reads; both readers emit.
#[test]
fn contended_minimum_budget_opens_reach_source_reads() {
    let hot = b"hot\n".repeat(1000);
    let mut s = scheduler(&[&hot, &hot], 8, 2, 2, 1, FramingConfig::Line(line()));
    let budget = Arc::clone(&s.probe_budget);
    let mut sink = Sink::default();
    let now = Instant::now();
    for _ in 0..1000 {
        // Discovery holds the allowance until the scheduler has returned at a
        // controlled operation boundary. A pending reader owns the next turn.
        let held = budget.try_reserve();
        let _ = tick(&mut s, &mut sink, now);
        drop(held);
        s.probe_available();
        if [ReaderKey(0), ReaderKey(1)]
            .iter()
            .all(|key| sink.frames.iter().any(|f| &f.key == key))
        {
            return;
        }
    }
    panic!("both readers must emit under sustained contention");
}

fn batcher(
    s: &Scheduler<FakeIo>,
    now: Instant,
    max_bytes: u64,
    metadata: super::super::config::FileMetadata,
) -> batch::Batcher {
    use super::super::{config::Config, framer::OversizeBehavior};
    let mut config = Config {
        include: vec!["/logs/*.log".into()],
        include_file_metadata: metadata,
        ..Config::default()
    };
    config.framing.max_line_bytes = 128;
    config.framing.max_record_bytes = 128;
    config.framing.max_log_size_behavior = OversizeBehavior::Truncate;
    config.batch.max_bytes = max_bytes;
    let inputs = config
        .compile("g", "p", "filelog", std::path::Path::new("/"))
        .unwrap()
        .into_parts()
        .0;
    let mut sink = batch::Batcher::new(&inputs, now, || 123);
    for r in s.readers.iter().flatten() {
        sink.register(
            r.key,
            r.file_id,
            r.epoch,
            r.evidence.applied_offset,
            r.resume,
            &cp::AdvisoryPath::from_unix_bytes(b"/logs/old").unwrap(),
        )
        .unwrap();
    }
    sink
}

/// Scenario: A reader is idle or independently paused while a nonempty OTAP batch approaches expiry.
/// Guarantees: The worker sees the batch deadline and polling seals it at expiry without reading the source.
#[test]
fn concrete_batch_deadline_flushes_without_ready_reader_work() {
    let mut s = scheduler(&[b"a\n"], 8, 64, 1, 1, FramingConfig::Line(line()));
    let now = Instant::now();
    let mut sink = batcher(&s, now, 1024, super::super::config::FileMetadata::None);
    let first = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    assert!(!matches!(first.reason, Reason::Terminal(_)));
    let at = sink.builder.next_deadline().unwrap();
    let paused = Permission {
        read: false,
        ..permission()
    };
    assert_eq!(s.next_deadline(paused, &sink), Some(at));
    let reads = s.io.reads;
    let turn = s.turn(at, paused, &mut sink, Waker::noop(), || false);
    assert!(matches!(turn.reason, Reason::Paused));
    assert_eq!(s.io.reads, reads);
    assert_eq!(sink.builder.retained().unwrap().record_count, 1);
    assert!(sink.failure().is_none());
}

/// Scenario: A variable-size output becomes carry-over, then its mutable source disappears during delivery.
/// Guarantees: No reads happen while retained, release promotes the exact owned carry-over, and checkpoint application cannot rewind the reader's provisional frontier.
#[test]
fn concrete_carry_over_survives_source_change_and_delivery_wait() {
    let mut source = b"0123456789\n".to_vec();
    source.extend_from_slice(&[b'x'; 128]);
    source.extend_from_slice(b"\ntail\n");
    let mut s = scheduler(&[&source], 32, 64, 1, 1, FramingConfig::Line(line()));
    let now = Instant::now();
    let mut sink = batcher(&s, now, 300, super::super::config::FileMetadata::None);
    for _ in 0..100 {
        let t = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
        assert!(!matches!(t.reason, Reason::Terminal(_)));
        if sink.builder.retained().is_some() {
            break;
        }
    }
    assert_eq!(sink.builder.retained().unwrap().record_count, 1);
    let applied = sink.builder.retained().unwrap().progress.clone();
    assert_eq!(applied[0].new_committed_offset, 11);
    let reads = s.io.reads;
    let provisional = s.readers[0].as_ref().unwrap().window.end;
    s.io.files[0].bytes.clear();
    for _ in 0..4 {
        assert!(matches!(
            s.turn(now, permission(), &mut sink, Waker::noop(), || false)
                .reason,
            Reason::Paused
        ));
    }
    assert_eq!(s.io.reads, reads);
    sink.builder.release_applied(&applied, now).unwrap();
    assert!(sink.unresolved(ReaderKey(0), 7));
    sink.builder.seal().unwrap();
    let records = sink.builder.retained().unwrap();
    assert_eq!(records.progress[0].expected_committed_offset, 11);
    assert_eq!(records.progress[0].new_committed_offset, 140);
    assert_eq!(
        records.provenance[0].body_range,
        super::super::decoder::SourceRange {
            start: 11,
            end: 139
        }
    );
    assert_eq!(s.readers[0].as_ref().unwrap().window.end, provisional);
    assert_eq!(s.io.reads, reads);
    assert!(sink.failure().is_none());
}

/// Scenario: Every resident descriptor owns progress in a nonempty open batch.
/// Guarantees: Descriptor pressure seals that batch rather than parking forever, and the waiting reader proceeds only after matching application and explicit resume.
#[test]
fn concrete_batch_flush_breaks_unresolved_descriptor_pressure() {
    let mut s = scheduler(&[b"a\n", b"b\n"], 8, 64, 2, 1, FramingConfig::Line(line()));
    let now = Instant::now();
    let mut sink = batcher(&s, now, 1024, super::super::config::FileMetadata::None);
    let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    assert!(sink.builder.retained().is_none());
    let t = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    assert!(matches!(t.reason, Reason::Paused));
    let applied = sink.builder.retained().unwrap().progress.clone();
    assert!(sink.unresolved(ReaderKey(0), 7));
    sink.builder.release_applied(&applied, now).unwrap();
    assert!(!sink.unresolved(ReaderKey(0), 7));
    s.resume_sink();
    for _ in 0..100 {
        let t = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
        assert!(!matches!(t.reason, Reason::Terminal(_)));
        if sink.unresolved(ReaderKey(1), 7) {
            break;
        }
    }
    assert!(sink.unresolved(ReaderKey(1), 7));
    assert!(sink.failure().is_none());
}

/// Scenario: File-name evidence changes after a record is accepted as carry-over.
/// Guarantees: Promotion preserves the old metadata and observation time instead of reprojecting from mutable reader context.
#[test]
fn concrete_carry_over_freezes_projected_metadata() {
    use otel_arrow_dfe_pdata::{
        otlp::{ProtoBuffer, ProtoBytesEncoder, logs::LogsProtoBytesEncoder},
        proto::opentelemetry::{
            collector::logs::v1::ExportLogsServiceRequest, common::v1::any_value::Value,
        },
    };
    use prost::Message;
    let source = [vec![b'x'; 128], vec![b'\n']].concat().repeat(40);
    let mut s = scheduler(&[&source], 32, 64, 1, 1, FramingConfig::Line(line()));
    let now = Instant::now();
    let mut sink = batcher(&s, now, 6500, super::super::config::FileMetadata::Name);
    for _ in 0..2000 {
        let t = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
        assert!(!matches!(t.reason, Reason::Terminal(_)));
        if sink.builder.retained().is_some() {
            break;
        }
    }
    let applied = sink.builder.retained().unwrap().progress.clone();
    sink.update_metadata(
        ReaderKey(0),
        &cp::AdvisoryPath::from_unix_bytes(b"/logs/new").unwrap(),
    )
    .unwrap();
    sink.builder.release_applied(&applied, now).unwrap();
    sink.builder.seal().unwrap();
    let batch = sink.builder.retained().unwrap();
    assert_eq!(batch.record_count, 1);
    let mut records = batch.records.clone();
    let mut bytes = ProtoBuffer::default();
    LogsProtoBytesEncoder::new()
        .encode(&mut records, &mut bytes)
        .unwrap();
    let request = ExportLogsServiceRequest::decode(bytes.as_ref()).unwrap();
    let log = &request.resource_logs[0].scope_logs[0].log_records[0];
    assert_eq!(log.observed_time_unix_nano, 123);
    assert_eq!(log.attributes[0].key, "log.file.name");
    assert_eq!(
        log.attributes[0].value.as_ref().unwrap().value,
        Some(Value::StringValue("old".into()))
    );
}
