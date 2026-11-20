// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::filelog_receiver::{
    config::Rotation,
    discovery::{
        Observation, ScanCompletion, ScanReason,
        reconcile::{BindingRole, ReconcileLimits, Reconciler, TrackedFile, TrackedId},
    },
};

const ID: TrackedId = TrackedId(7);

struct Fixture {
    readers: Scheduler<FakeIo>,
    sink: Sink,
    scan: Reconciler,
    generation: u64,
    observations: u64,
    clock: Rc<Cell<Instant>>,
    rotation: Rotation,
}

impl Fixture {
    fn at_eof() -> Self {
        Self::with_file(b"", Duration::from_millis(250))
    }

    fn with_file(bytes: &[u8], reprobe_interval: Duration) -> Self {
        let mut readers = scheduler(&[bytes], 64, 32, 1, 1, FramingConfig::Line(line()));
        readers.limits.eof_interval = reprobe_interval;
        let mut sink = Sink::default();
        let now = Instant::now();
        let clock = Rc::new(Cell::new(now));
        let eof_clock = Rc::clone(&clock);
        readers.eof_clock = Box::new(move || eof_clock.get());
        park_at_eof(&mut readers, &mut sink, 1, now);
        let mut scan = Reconciler::new(ReconcileLimits {
            tracked: 1,
            pending: 1,
            offers: 1,
            path_bytes: 1024,
            fingerprint_bytes: 16,
        })
        .unwrap();
        scan.track(TrackedFile {
            id: ID,
            locator: readers.io.files[0].locator,
            binding: Some("/logs/app.log".into()),
            role: BindingRole::Live,
            active: true,
            resident: true,
        })
        .unwrap();
        Self {
            readers,
            sink,
            scan,
            generation: 0,
            observations: 0,
            clock,
            rotation: Rotation::default(),
        }
    }

    fn begin(&mut self, start: u64) {
        self.generation += 1;
        self.observations = 0;
        self.scan
            .begin(self.generation, Duration::from_millis(start), 0)
            .unwrap();
    }

    fn finish(&mut self, end: u64) {
        let reader = self
            .readers
            .resident_disappearance(ReaderKey(0, 0))
            .unwrap();
        let _ = self
            .scan
            .finish(
                ScanCompletion {
                    generation: self.generation,
                    observations: self.observations,
                    complete: true,
                    cancelled: false,
                },
                Duration::from_millis(end),
                |event| reader.observe(ID, &event),
            )
            .unwrap();
    }

    fn miss(&mut self, start: u64, end: u64) {
        self.begin(start);
        self.finish(end);
    }

    fn observe(&mut self, at: u64, reason: Option<ScanReason>) {
        let file = &self.readers.io.files[0];
        self.feed(
            at,
            Observation {
                path: "/logs/app.log".into(),
                locator: Some(file.locator),
                size: reason.is_none().then_some(file.bytes.len() as u64),
                modified: None,
                fingerprint: Vec::new(),
                reason,
                errno: None,
            },
        );
    }

    fn feed(&mut self, at: u64, observation: Observation) {
        let reader = self
            .readers
            .resident_disappearance(ReaderKey(0, 0))
            .unwrap();
        self.scan
            .observe(
                self.generation,
                &observation,
                Duration::from_millis(at),
                |event| reader.observe(ID, &event),
            )
            .unwrap();
        self.observations += 1;
    }

    fn turn_at(&mut self, at: Instant) -> Reason {
        self.clock.set(at);
        tick(&mut self.readers, &mut self.sink, at).reason
    }

    fn met(&mut self) -> bool {
        self.readers
            .resident_disappearance(ReaderKey(0, 0))
            .unwrap()
            .discovery_and_eof_inactivity_met(&self.rotation)
    }
}

/// Scenario: EOF inactivity expires with only one miss or a pass that started exactly at its finish.
/// Guarantees: Only a complete miss started strictly later satisfies the outstanding discovery condition.
#[test]
fn disappearance_requires_a_later_pass_start() {
    let mut f = Fixture::at_eof();
    let expired = f.clock.get() + f.rotation.rotate_wait;
    assert!(matches!(f.turn_at(expired), Reason::Eof));
    f.miss(0, 10);
    assert!(!f.met());
    f.miss(10, 20);
    assert!(!f.met());
    f.miss(21, 30);
    assert!(f.met());
}

/// Scenario: Discovery confirms before the deadline, then the resident file grows just before expiry.
/// Guarantees: The appended line is delivered to the sink and only a full interval after the new EOF satisfies inactivity.
#[test]
fn disappearance_late_line_restarts_only_eof_inactivity() {
    let mut f = Fixture::at_eof();
    let expiry = f.clock.get() + f.rotation.rotate_wait;
    let append = expiry - Duration::from_millis(1);
    f.miss(0, 1);
    f.miss(2, 3);
    assert!(!f.met());
    f.clock.set(append);
    f.readers.io.files[0].bytes.extend_from_slice(b"late\n");

    f.sink.blocked = true;
    assert!(matches!(f.turn_at(expiry), Reason::Paused));
    assert!(!f.met());
    f.sink.blocked = false;
    f.readers.resume_sink();
    frames(&mut f.readers, &mut f.sink, 1, expiry);
    assert_eq!(body(&f.sink.frames[0]), b"late");
    assert!(!f.met());

    // The first EOF read completes after its turn started.
    let new_eof = expiry + Duration::from_secs(1);
    f.clock.set(new_eof);
    park_at_eof(&mut f.readers, &mut f.sink, 1, expiry);
    let new_expiry = new_eof + f.rotation.rotate_wait;
    let early = new_expiry - f.readers.limits.eof_interval;
    assert!(matches!(f.turn_at(early), Reason::Eof));
    assert!(!f.met());
    assert!(matches!(f.turn_at(new_expiry), Reason::Eof));
    assert!(f.met());
}

/// Scenario: An unfinished pass loses policy coverage after two complete misses.
/// Guarantees: The failure vetoes immediately and restoration needs one miss started after the failed pass finished.
#[test]
fn disappearance_unfinished_failure_invalidates_confirmation() {
    let mut f = Fixture::at_eof();
    let expired = f.clock.get() + f.rotation.rotate_wait;
    assert!(matches!(f.turn_at(expired), Reason::Eof));
    f.miss(0, 1);
    f.miss(2, 3);
    assert!(f.met());
    f.begin(4);
    // Failure is receiver-wide, even when the failed name has no locator.
    f.feed(
        5,
        Observation {
            path: "/other".into(),
            locator: None,
            size: None,
            modified: None,
            fingerprint: Vec::new(),
            reason: Some(ScanReason::Policy),
            errno: None,
        },
    );
    assert!(!f.met());
    f.finish(6);
    f.miss(6, 7);
    assert!(!f.met());
    f.miss(8, 9);
    assert!(f.met());
}

/// Scenario: A missing locator returns first with presence alone and then under a validated name of unchanged size.
/// Guarantees: Presence blocks until a fresh miss; validated reappearance requires two misses and preserves EOF inactivity.
#[test]
fn disappearance_presence_and_validated_reappearance() {
    let mut f = Fixture::with_file(b"line\n", Duration::from_millis(250));
    let expiry = f.clock.get() + f.rotation.rotate_wait;
    assert!(matches!(f.turn_at(expiry), Reason::Eof));
    f.miss(0, 1);
    f.miss(2, 3);
    f.begin(4);
    f.observe(5, Some(ScanReason::PresenceOnly));
    assert!(!f.met());
    f.finish(6);
    f.miss(7, 8);
    assert!(f.met());
    f.begin(9);
    f.observe(10, None);
    f.finish(11);
    f.miss(12, 13);
    assert!(!f.met());
    f.miss(14, 15);
    assert!(f.met());
}

/// Scenario: Discovery validates a larger file before two fresh misses, with no intervening reader turn.
/// Guarantees: Known unread bytes invalidate the old EOF interval until read and followed by a new inactivity interval.
#[test]
fn disappearance_discovery_growth_invalidates_eof() {
    let mut f = Fixture::at_eof();
    let expiry = f.clock.get() + f.rotation.rotate_wait;
    assert!(matches!(f.turn_at(expiry), Reason::Eof));
    f.miss(0, 1);
    f.miss(2, 3);
    assert!(f.met());
    f.readers.io.files[0].bytes.extend_from_slice(b"late\n");
    f.begin(4);
    f.observe(5, None);
    f.finish(6);
    f.miss(7, 8);
    f.miss(9, 10);
    assert!(!f.met());

    let read_at = expiry + f.readers.limits.eof_interval;
    f.clock.set(read_at);
    frames(&mut f.readers, &mut f.sink, 1, read_at);
    assert_eq!(body(&f.sink.frames[0]), b"late");
    park_at_eof(&mut f.readers, &mut f.sink, 1, read_at);
    assert!(!f.met());
    assert!(matches!(
        f.turn_at(read_at + f.rotation.rotate_wait),
        Reason::Eof
    ));
    assert!(f.met());
}

/// Scenario: An authorized exclusion arrives while the validated handle remains open.
/// Guarantees: Further complete misses cannot qualify the excluded handle for rotation.
#[test]
fn disappearance_exclusion_supersedes_missing_sequence() {
    let mut f = Fixture::at_eof();
    assert!(matches!(
        f.turn_at(f.clock.get() + f.rotation.rotate_wait),
        Reason::Eof
    ));
    f.miss(0, 1);
    f.miss(2, 3);
    assert!(f.met());
    f.begin(4);
    f.observe(5, Some(ScanReason::Excluded));
    assert!(!f.met());
    f.finish(6);
    f.miss(7, 8);
    f.miss(9, 10);
    assert!(!f.met());
}

/// Scenario: An established include root or binding parent changes identity and later recovers.
/// Guarantees: The old misses cannot confirm disappearance; two complete misses after recovery can.
#[test]
fn disappearance_directory_change_requires_two_fresh_misses() {
    let directory = |inode, reason| Observation {
        path: "/logs".into(),
        locator: Some(FileLocator { device: 1, inode }),
        size: None,
        modified: None,
        fingerprint: Vec::new(),
        reason: Some(reason),
        errno: None,
    };
    for reason in [ScanReason::Directory, ScanReason::RootChanged] {
        let mut f = Fixture::at_eof();
        assert!(matches!(
            f.turn_at(f.clock.get() + f.rotation.rotate_wait),
            Reason::Eof
        ));
        f.begin(0);
        f.feed(1, directory(10, ScanReason::Directory));
        f.finish(2);
        f.miss(3, 4);
        assert!(f.met());
        f.begin(5);
        f.feed(6, directory(11, reason));
        assert!(!f.met());
        f.finish(7);
        f.begin(8);
        f.feed(9, directory(10, ScanReason::Directory));
        f.finish(10);
        assert!(!f.met());
        f.miss(11, 12);
        assert!(f.met());
    }
}

/// Scenario: The next resident EOF probe is scheduled later than rotate_wait.
/// Guarantees: Expiry while idle cannot satisfy inactivity until the scheduled probe succeeds.
#[test]
fn disappearance_waits_for_a_long_reprobe_interval() {
    let interval = Duration::from_secs(10);
    let mut f = Fixture::with_file(b"", interval);
    let first = f.clock.get();
    f.miss(0, 1);
    f.miss(2, 3);
    assert!(matches!(
        f.turn_at(first + f.rotation.rotate_wait),
        Reason::Idle
    ));
    assert!(!f.met());
    assert!(matches!(f.turn_at(first + interval), Reason::Eof));
    assert!(f.met());
}

/// Scenario: An unfinished line needs a content reprobe and its read fails at the deadline.
/// Guarantees: The failure cannot confirm EOF; a successful retry on the same handle can.
#[test]
fn disappearance_waits_for_successful_eof_after_read_error() {
    let mut f = Fixture::with_file(b"partial", Duration::from_millis(250));
    let expiry = f.clock.get() + f.rotation.rotate_wait;
    f.miss(0, 1);
    f.miss(2, 3);
    f.readers.io.read_error = Some(libc::EAGAIN);
    assert!(matches!(f.turn_at(expiry), Reason::Retry(_)));
    assert!(!f.met());
    let retry = f.readers.next_deadline(permission(), &f.sink).unwrap();
    assert!(matches!(f.turn_at(retry), Reason::Eof));
    assert!(f.met());
}
