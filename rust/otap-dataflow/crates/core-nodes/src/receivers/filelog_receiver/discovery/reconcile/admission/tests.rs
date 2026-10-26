// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::{cell::Cell, convert::Infallible};

fn locator(inode: u64) -> FileLocator {
    FileLocator { device: 1, inode }
}
fn file(id: u64, inode: u64, path: &str, resident: bool) -> TrackedFile {
    TrackedFile {
        id: TrackedId(id),
        locator: locator(inode),
        binding: Some(path.into()),
        role: BindingRole::Live,
        active: true,
        resident,
    }
}
fn reconciler(tracked: usize) -> Reconciler {
    Reconciler::new(ReconcileLimits {
        tracked,
        pending: 4,
        offers: 4,
        path_bytes: 1024,
        fingerprint_bytes: 16,
    })
    .unwrap()
}
fn begin(r: &mut Reconciler, generation: u64) {
    r.begin(generation, Duration::from_secs(generation * 10), generation)
        .unwrap();
}
fn observe(r: &mut Reconciler, inode: u64, path: &str, directory: bool) {
    let generation = r.pass.as_ref().unwrap().generation;
    let observation = Observation {
        path: path.into(),
        locator: Some(locator(inode)),
        size: (!directory).then_some(3),
        modified: None,
        fingerprint: if directory {
            Vec::new()
        } else {
            b"abc".to_vec()
        },
        reason: directory.then_some(ScanReason::Directory),
        errno: None,
    };
    r.observe(
        generation,
        &observation,
        Duration::from_secs(generation * 10),
        |_| {},
    )
    .unwrap();
}
fn finish(r: &mut Reconciler) -> PassSummary {
    let pass = r.pass.as_ref().unwrap();
    let generation = pass.generation;
    r.finish(
        ScanCompletion {
            generation,
            complete: true,
            cancelled: false,
            observations: pass.count,
        },
        Duration::from_secs(generation * 10 + 1),
        |_| {},
    )
    .unwrap()
}
fn token(r: &Reconciler, inode: u64) -> OfferToken {
    r.offers()
        .unwrap()
        .find(|offer| offer.candidate.locator == locator(inode))
        .unwrap()
        .token
}
fn success(_: &[&Candidate]) -> Result<Option<Duration>, Infallible> {
    Ok(Some(Duration::from_secs(12)))
}

/// Scenario: One member of a group fails preflight, followed by a valid group using those same offers.
/// Guarantees: No durable callback or tracking mutation occurs on rejection; success publishes and acknowledges all members together.
#[test]
fn batch_preflight_precedes_every_durable_effect() {
    let mut r = reconciler(4);
    begin(&mut r, 1);
    observe(&mut r, 1, "/a", false);
    observe(&mut r, 2, "/b", false);
    assert!(finish(&mut r).complete);
    let a = token(&r, 1);
    let b = token(&r, 2);
    let called = Cell::new(false);
    let bad = vec![
        AdmissionChange {
            token: a,
            file: file(1, 1, "/a", true),
            replace_id: None,
        },
        AdmissionChange {
            token: b,
            file: file(2, 2, "/wrong", true),
            replace_id: None,
        },
    ];
    let result = r.admit_batch(bad, |_| {
        called.set(true);
        success(&[])
    });
    assert!(matches!(
        result,
        Err(AdmissionError::Preflight(ReconcileError::Tracked))
    ));
    assert!(!called.get());
    assert_eq!(r.tracked.len(), 0);
    assert_eq!(r.stats(Duration::from_secs(12)).pending, 2);
    let good = vec![
        AdmissionChange {
            token: a,
            file: file(1, 1, "/a", true),
            replace_id: None,
        },
        AdmissionChange {
            token: b,
            file: file(2, 2, "/b", true),
            replace_id: None,
        },
    ];
    assert!(
        r.admit_batch(good, |candidates| {
            assert_eq!(candidates.len(), 2);
            success(candidates)
        })
        .unwrap()
    );
    assert_eq!(r.tracked.len(), 2);
    assert_eq!(r.stats(Duration::from_secs(12)).pending, 0);
    assert!(matches!(
        r.admit_batch(
            vec![AdmissionChange {
                token: a,
                file: file(1, 1, "/a", true),
                replace_id: None
            }],
            success
        ),
        Err(AdmissionError::Preflight(ReconcileError::Feedback))
    ));
}

/// Scenario: A full tracked table replaces one same-locator identity with a fresh runtime ID.
/// Guarantees: Deferral and commit error preserve the old view; success uses no spare slot and preserves the directory baseline.
#[test]
fn full_capacity_replacement_preserves_failure_atomicity_and_parent() {
    for outcome in 0..3 {
        let mut r = reconciler(1);
        r.track(file(1, 1, "/logs/a", false)).unwrap();
        begin(&mut r, 1);
        observe(&mut r, 9, "/logs", true);
        observe(&mut r, 1, "/logs/a", false);
        assert!(finish(&mut r).complete);
        let change = AdmissionChange {
            token: token(&r, 1),
            file: file(2, 1, "/logs/a", true),
            replace_id: Some(TrackedId(1)),
        };
        let result = r.admit_batch(vec![change], |_| match outcome {
            0 => Ok(None),
            1 => Err(std::io::Error::other("known rejected commit")),
            _ => Ok(Some(Duration::from_secs(12))),
        });
        assert_eq!(r.tracked.len(), 1);
        if outcome < 2 {
            if outcome == 0 {
                assert!(!result.unwrap());
            } else {
                assert!(matches!(result, Err(AdmissionError::Commit(_))));
            }
            assert!(r.tracked(TrackedId(1)).is_some());
            assert!(r.tracked(TrackedId(2)).is_none());
            assert_eq!(r.stats(Duration::from_secs(12)).pending, 1);
        } else {
            assert!(result.unwrap());
            assert!(r.tracked(TrackedId(1)).is_none());
            assert_eq!(r.tracked[&TrackedId(2)].parent, Some(locator(9)));
            begin(&mut r, 2);
            observe(&mut r, 10, "/logs", true);
            observe(&mut r, 1, "/logs/a", false);
            assert!(!finish(&mut r).complete);
            assert!(r.tracked[&TrackedId(2)].parent_changed);
        }
    }
}

/// Scenario: A net-zero replacement shares a group with more new files than the remaining tracked slots.
/// Guarantees: Capacity is checked across the group before its callback, and a fitting group reaches the full limit exactly.
#[test]
fn group_capacity_counts_net_growth() {
    let mut r = reconciler(2);
    r.track(file(1, 1, "/a", false)).unwrap();
    begin(&mut r, 1);
    for (inode, path) in [(1, "/a"), (2, "/b"), (3, "/c")] {
        observe(&mut r, inode, path, false);
    }
    assert!(finish(&mut r).complete);
    let make = |r: &Reconciler, inode| AdmissionChange {
        token: token(r, inode),
        file: file(
            if inode == 1 { 10 } else { inode },
            inode,
            ["", "/a", "/b", "/c"][inode as usize],
            true,
        ),
        replace_id: (inode == 1).then_some(TrackedId(1)),
    };
    let result = r.admit_batch(
        vec![make(&r, 1), make(&r, 2), make(&r, 3)],
        |_| -> Result<_, Infallible> { panic!("capacity reached callback") },
    );
    assert!(matches!(
        result,
        Err(AdmissionError::Preflight(ReconcileError::Capacity))
    ));
    assert_eq!(r.tracked.len(), 1);
    assert!(
        r.admit_batch(vec![make(&r, 1), make(&r, 2)], success)
            .unwrap()
    );
    assert_eq!(r.tracked.len(), 2);
    assert!(r.tracked(TrackedId(1)).is_none());
    assert!(r.tracked(TrackedId(10)).is_some());
}

/// Scenario: A replacement names the wrong locator or tries to retire a reader that became live after the offer.
/// Guarantees: Neither condition can execute durable work or remove the old tracked identity.
#[test]
fn replacement_cannot_retire_an_unrelated_or_live_reader() {
    for live in [false, true] {
        let mut r = reconciler(3);
        r.track(file(1, 1, "/a", false)).unwrap();
        begin(&mut r, 1);
        observe(&mut r, 1, "/a", false);
        observe(&mut r, 2, "/b", false);
        assert!(finish(&mut r).complete);
        if live {
            r.update_reader_state(1, TrackedId(1), true, true).unwrap();
        }
        let inode = if live { 1 } else { 2 };
        let path = if live { "/a" } else { "/b" };
        let change = AdmissionChange {
            token: token(&r, inode),
            file: file(3, inode, path, true),
            replace_id: Some(TrackedId(1)),
        };
        assert!(matches!(
            r.admit_batch(vec![change], |_| -> Result<_, Infallible> {
                panic!("invalid retirement reached callback")
            }),
            Err(AdmissionError::Preflight(ReconcileError::Tracked))
        ));
        assert!(r.tracked(TrackedId(1)).is_some());
        assert!(r.tracked(TrackedId(3)).is_none());
    }
}

/// Scenario: A group reconnects a rotated predecessor and admits its successor in either publication order.
/// Guarantees: Only the successor owns the old path after success; both independent identities remain tracked.
#[test]
fn grouped_rotation_publication_is_order_independent() {
    for reverse in [false, true] {
        let mut r = reconciler(3);
        r.track(file(1, 1, "/a", false)).unwrap();
        begin(&mut r, 1);
        observe(&mut r, 1, "/archive", false);
        observe(&mut r, 2, "/a", false);
        assert!(finish(&mut r).complete);
        let mut changes = vec![
            AdmissionChange {
                token: token(&r, 1),
                file: file(1, 1, "/a", true),
                replace_id: None,
            },
            AdmissionChange {
                token: token(&r, 2),
                file: file(2, 2, "/a", true),
                replace_id: None,
            },
        ];
        if reverse {
            changes.reverse();
        }
        assert!(r.admit_batch(changes, success).unwrap());
        assert_eq!(
            r.tracked(TrackedId(1)).unwrap().role,
            BindingRole::Superseded
        );
        assert_eq!(r.tracked(TrackedId(2)).unwrap().role, BindingRole::Live);
        assert_eq!(r.rebounds().count(), 0);
    }
}

/// Scenario: A group repeats an offer token or a runtime identity.
/// Guarantees: Duplicate members cannot cause double publication or invoke the durable callback twice.
#[test]
fn duplicate_group_members_fail_preflight() {
    let mut r = reconciler(4);
    begin(&mut r, 1);
    observe(&mut r, 1, "/a", false);
    assert!(finish(&mut r).complete);
    let token = token(&r, 1);
    let changes = (0..2)
        .map(|_| AdmissionChange {
            token,
            file: file(1, 1, "/a", true),
            replace_id: None,
        })
        .collect();
    assert!(matches!(
        r.admit_batch(changes, |_| -> Result<_, Infallible> {
            panic!("duplicate reached callback")
        }),
        Err(AdmissionError::Preflight(ReconcileError::Tracked))
    ));
    assert_eq!(r.tracked.len(), 0);
}

/// Scenario: A rotation successor is replaced at the same locator after its reader retires.
/// Guarantees: Failed commits preserve predecessor links; successful replacement keeps later passes free of spurious pending rebounds.
#[test]
fn same_locator_replacement_preserves_rotation_successors() {
    for outcome in 0..3 {
        let mut r = reconciler(2);
        r.track(file(1, 1, "/app", false)).unwrap();
        begin(&mut r, 1);
        observe(&mut r, 1, "/archive", false);
        observe(&mut r, 2, "/app", false);
        assert!(finish(&mut r).complete);
        assert!(
            r.admit(token(&r, 2), file(2, 2, "/app", false), |_| {
                Some(Duration::from_secs(12))
            })
            .unwrap()
        );
        assert_eq!(
            r.tracked[&TrackedId(1)].successor,
            Some((TrackedId(2), locator(2)))
        );

        begin(&mut r, 2);
        observe(&mut r, 1, "/archive", false);
        observe(&mut r, 2, "/app", false);
        assert!(finish(&mut r).complete);
        assert_eq!(r.rebounds().count(), 0);
        let change = AdmissionChange {
            token: token(&r, 2),
            file: file(3, 2, "/app", false),
            replace_id: Some(TrackedId(2)),
        };
        let result = r.admit_batch(vec![change], |_| match outcome {
            0 => Ok(None),
            1 => Err(std::io::Error::other("commit failed")),
            _ => Ok(Some(Duration::from_secs(22))),
        });
        match outcome {
            0 => assert!(!result.unwrap()),
            1 => assert!(matches!(result, Err(AdmissionError::Commit(_)))),
            _ => assert!(result.unwrap()),
        }
        let expected = if outcome == 2 {
            TrackedId(3)
        } else {
            TrackedId(2)
        };
        assert_eq!(r.tracked.len(), 2);
        assert_eq!(
            r.tracked[&TrackedId(1)].successor,
            Some((expected, locator(2)))
        );
        begin(&mut r, 3);
        observe(&mut r, 1, "/archive", false);
        observe(&mut r, 2, "/app", false);
        assert!(finish(&mut r).complete);
        assert_eq!(r.rebounds().count(), 0);
    }
}

/// Scenario: A new ID claims an active locator through an alias, with each binding role, an absent binding, and full or spare capacity.
/// Guarantees: Preflight rejects before durability or capacity errors; the offer remains available for explicit same-locator replacement.
#[test]
fn active_locator_requires_explicit_identity_replacement() {
    for capacity in [1, 2] {
        for role in [
            BindingRole::Live,
            BindingRole::Unverified,
            BindingRole::Superseded,
        ] {
            for has_binding in [false, true] {
                for reverse in [false, true] {
                    let mut r = reconciler(capacity);
                    let mut old = file(1, 1, "/logs/app", false);
                    old.role = role;
                    if !has_binding {
                        old.binding = None;
                    }
                    r.track(old).unwrap();
                    begin(&mut r, 1);
                    for path in if reverse {
                        ["/logs/app", "/logs.old/app"]
                    } else {
                        ["/logs.old/app", "/logs/app"]
                    } {
                        observe(&mut r, 1, path, false);
                    }
                    assert!(finish(&mut r).complete);
                    let token = token(&r, 1);
                    assert_eq!(
                        r.admit(token, file(2, 1, "/logs.old/app", true), |_| {
                            panic!("duplicate active locator reached durable callback")
                        }),
                        Err(ReconcileError::Feedback)
                    );
                    assert_eq!(r.tracked.len(), 1);
                    assert!(r.tracked(TrackedId(1)).is_some());
                    assert_eq!(r.offers().unwrap().count(), 1);
                    assert!(
                        r.admit_batch(
                            vec![AdmissionChange {
                                token,
                                file: file(2, 1, "/logs.old/app", true),
                                replace_id: Some(TrackedId(1)),
                            }],
                            success
                        )
                        .unwrap()
                    );
                    assert!(r.tracked(TrackedId(1)).is_none());
                    assert_eq!(r.tracked.len(), 1);
                }
            }
        }
    }
}

/// Scenario: A retired reader's inactive record retains a locator later used by a new identity.
/// Guarantees: Historical inactive state does not prevent a distinct active admission or get implicitly removed.
#[test]
fn inactive_locator_does_not_require_replacement() {
    let mut r = reconciler(2);
    let mut old = file(1, 1, "/old", false);
    old.active = false;
    r.track(old).unwrap();
    begin(&mut r, 1);
    observe(&mut r, 1, "/new", false);
    assert!(finish(&mut r).complete);
    let token = token(&r, 1);
    assert!(
        r.admit(token, file(2, 1, "/new", true), |_| Some(
            Duration::from_secs(12)
        ))
        .unwrap()
    );
    assert!(!r.tracked(TrackedId(1)).unwrap().active);
    assert!(r.tracked(TrackedId(2)).unwrap().active);
}
