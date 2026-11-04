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
    unresolved: BTreeSet<(u64, u64)>,
    preflights: usize,
}
impl BatchSink for Sink {
    fn preflight(&mut self, _: ReaderKey, _: u64) -> Flow {
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
    fn unresolved(&self, key: ReaderKey, epoch: u64) -> bool {
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
    for (id, _) in files.iter().enumerate() {
        s.readers.push(Some(Reader {
            key: ReaderKey(id as u64),
            epoch: 7,
            evidence: Evidence {
                locator: s.io.files[id].locator,
                fingerprint: Vec::new(),
                applied_offset: 0,
                applied_guard: CommittedFrontierGuard::empty(),
            },
            handle: None,
            validation: None,
            partial: None,
            window: Window::empty(),
            initial: true,
            needs_input: false,
            check_size: false,
            service: 0,
            record_start: 0,
            failures: 0,
            last_service: 0,
            pinned: false,
            halted: false,
            lease: None,
        }));
        s.ready.push_back(id);
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
/// Guarantees: Both source reads and zero-consumption processing yield at their independent bounds.
#[test]
fn source_and_processing_budgets_include_empty_replay() {
    let mut c = line();
    c.encoding = Encoding::Utf16Le;
    let input = [0xff, 0xfe, b'a', 0, b'\n', 0, b'b', 0, b'\n', 0];
    let mut s = scheduler(&[&input], 3, 3, 1, 1, FramingConfig::Line(c));
    let mut sink = Sink::default();
    let now = Instant::now();
    let mut zero_byte_work = false;
    for _ in 0..100 {
        let t = tick(&mut s, &mut sink, now);
        zero_byte_work |= t.bytes == 0 && t.steps == 3;
        if sink.frames.len() == 2 {
            break;
        }
    }
    assert_eq!(
        sink.frames.iter().map(body).collect::<Vec<_>>(),
        [b"a", b"b"]
    );
    assert!(zero_byte_work);
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
    let deadline = s.next_deadline(permission()).unwrap();
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
    assert_eq!(s.next_deadline(paused), None);
    s.io.read_error = Some(11);
    assert!(matches!(
        tick(&mut s, &mut sink, deadline).reason,
        Reason::Retry(_)
    ));
    assert_eq!(s.deadlines.len(), 1);
    let retry = s.next_deadline(permission()).unwrap();
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
    assert_eq!(s.slots, 2);
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
    struct Lease(Rc<Cell<usize>>);
    impl Drop for Lease {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let dropped = Rc::new(Cell::new(0));
    let mut s = scheduler(
        &[b"long partial", b"b\n"],
        4,
        32,
        2,
        1,
        FramingConfig::Line(line()),
    );
    s.readers[0].as_mut().unwrap().lease = Some(Box::new(Lease(Rc::clone(&dropped))));
    let mut sink = Sink {
        stop_at: Some(1),
        ..Sink::default()
    };
    frames(&mut s, &mut sink, 1, Instant::now());
    assert_eq!(dropped.get(), 0);
    let r = s.readers[0].as_ref().unwrap();
    assert_eq!(r.key, ReaderKey(0));
    assert_eq!(r.epoch, 7);
    assert_eq!(r.evidence.applied_offset, 0);
    drop(s);
    assert_eq!(dropped.get(), 1);
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
        let deadline = s.next_deadline(permission()).unwrap();
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
        applied_offset: 7,
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
    let deadline = s.next_deadline(permission()).unwrap();
    assert!(matches!(
        tick(&mut s, &mut sink, deadline).reason,
        Reason::Eof
    ));
    assert_eq!(&s.io.calls.borrow()[count..], &[(0, 7, 32)]);
    s.io.files[0].bytes.truncate(2);
    let deadline = s.next_deadline(permission()).unwrap();
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
