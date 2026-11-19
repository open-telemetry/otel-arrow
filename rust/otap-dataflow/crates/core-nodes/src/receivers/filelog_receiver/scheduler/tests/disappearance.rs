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
use std::sync::Mutex;

const ID: TrackedId = TrackedId(7);

struct Fixture {
    readers: Scheduler<FakeIo>,
    sink: Sink,
    scan: Reconciler,
    generation: u64,
    observations: u64,
    clock: Arc<Mutex<Instant>>,
    rotation: Rotation,
}

impl Fixture {
    fn at_eof() -> Self {
        let mut readers = scheduler(&[b""], 64, 32, 1, 1, FramingConfig::Line(line()));
        let mut sink = Sink::default();
        let now = Instant::now();
        let clock = Arc::new(Mutex::new(now));
        let eof_clock = Arc::clone(&clock);
        readers.eof_clock = Box::new(move || *eof_clock.lock().unwrap());
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
            binding: Some("/app.log".into()),
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
        let reader = self.readers.resident_disappearance(ReaderKey(0)).unwrap();
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
        let observation = Observation {
            path: "/app.log".into(),
            locator: Some(self.readers.io.files[0].locator),
            size: reason.is_none().then_some(3),
            modified: None,
            fingerprint: if reason.is_none() {
                b"abc".to_vec()
            } else {
                Vec::new()
            },
            reason,
            errno: None,
        };
        let reader = self.readers.resident_disappearance(ReaderKey(0)).unwrap();
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

    fn met(&mut self, at: Instant) -> bool {
        self.readers
            .resident_disappearance(ReaderKey(0))
            .unwrap()
            .discovery_and_eof_inactivity_met(at, &self.rotation)
    }
}

/// Scenario: EOF inactivity expires with only one miss or a pass that started exactly at its finish.
/// Guarantees: Only a complete miss started strictly later satisfies the outstanding discovery condition.
#[test]
fn disappearance_requires_a_later_pass_start() {
    let mut f = Fixture::at_eof();
    let expired = *f.clock.lock().unwrap() + f.rotation.rotate_wait;
    f.miss(0, 10);
    assert!(!f.met(expired));
    f.miss(10, 20);
    assert!(!f.met(expired));
    f.miss(21, 30);
    assert!(f.met(expired));
}

/// Scenario: Discovery confirms before the deadline, then the resident file grows just before expiry.
/// Guarantees: The appended line is delivered to the sink and only a full interval after the new EOF satisfies inactivity.
#[test]
fn disappearance_late_line_restarts_only_eof_inactivity() {
    let mut f = Fixture::at_eof();
    let expiry = *f.clock.lock().unwrap() + f.rotation.rotate_wait;
    let append = expiry - Duration::from_millis(1);
    f.miss(0, 1);
    f.miss(2, 3);
    assert!(!f.met(append));
    f.readers.io.files[0].bytes.extend_from_slice(b"late\n");
    *f.clock.lock().unwrap() = append;
    frames(&mut f.readers, &mut f.sink, 1, append);
    assert_eq!(body(&f.sink.frames[0]), b"late");
    assert!(!f.met(expiry));
    // A blocking read can finish after turn entry.
    let new_eof = expiry + Duration::from_secs(1);
    *f.clock.lock().unwrap() = new_eof;
    park_at_eof(&mut f.readers, &mut f.sink, 1, append);
    let new_expiry = new_eof + f.rotation.rotate_wait;
    assert!(!f.met(new_expiry - Duration::from_millis(1)));
    assert!(f.met(new_expiry));
}

/// Scenario: An unfinished pass loses policy coverage after two complete misses.
/// Guarantees: The failure vetoes immediately and restoration needs one miss started after the failed pass finished.
#[test]
fn disappearance_unfinished_failure_invalidates_confirmation() {
    let mut f = Fixture::at_eof();
    let expired = *f.clock.lock().unwrap() + f.rotation.rotate_wait;
    f.miss(0, 1);
    f.miss(2, 3);
    assert!(f.met(expired));
    f.begin(4);
    let reader = f.readers.resident_disappearance(ReaderKey(0)).unwrap();
    // Failure is receiver-wide, even when the failed name has no locator.
    f.scan
        .observe(
            f.generation,
            &Observation {
                path: "/other".into(),
                locator: None,
                size: None,
                modified: None,
                fingerprint: Vec::new(),
                reason: Some(ScanReason::Policy),
                errno: None,
            },
            Duration::from_millis(5),
            |event| reader.observe(ID, &event),
        )
        .unwrap();
    f.observations += 1;
    assert!(!f.met(expired));
    f.finish(6);
    f.miss(6, 7);
    assert!(!f.met(expired));
    f.miss(8, 9);
    assert!(f.met(expired));
}

/// Scenario: A missing reader's locator returns with weak or excluded name evidence before returning under an eligible name.
/// Guarantees: Each sighting blocks immediately. Only identity-validated reappearance makes two fresh misses necessary.
#[test]
fn disappearance_presence_and_validated_reappearance() {
    for reason in [Some(ScanReason::PresenceOnly), Some(ScanReason::Excluded)] {
        let mut f = Fixture::at_eof();
        let expired = *f.clock.lock().unwrap() + f.rotation.rotate_wait;
        f.miss(0, 1);
        f.miss(2, 3);
        f.begin(4);
        f.observe(5, reason);
        assert!(!f.met(expired));
        f.finish(6);
        assert!(!f.met(expired));
        f.miss(7, 8);
        assert!(f.met(expired));

        f.begin(9);
        f.observe(10, None);
        f.finish(11);
        f.miss(12, 13);
        assert!(!f.met(expired));
        f.miss(14, 15);
        assert!(f.met(expired));
    }
}
