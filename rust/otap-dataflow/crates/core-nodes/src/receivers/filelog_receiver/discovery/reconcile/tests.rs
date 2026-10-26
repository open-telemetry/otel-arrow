// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn limits() -> ReconcileLimits {
    ReconcileLimits {
        tracked: 8,
        pending: 4,
        offers: 2,
        path_bytes: 1024,
        fingerprint_bytes: 16,
    }
}
fn locator(inode: u64) -> FileLocator {
    FileLocator { device: 1, inode }
}
fn file(id: u64, path: &str) -> TrackedFile {
    TrackedFile {
        id: TrackedId(id),
        locator: locator(id),
        binding: Some(path.into()),
        role: BindingRole::Live,
        active: true,
        resident: true,
    }
}
fn observation(inode: u64, path: &str) -> Observation {
    Observation {
        path: path.into(),
        locator: Some(locator(inode)),
        size: Some(3),
        modified: None,
        fingerprint: b"abc".to_vec(),
        reason: None,
        errno: None,
    }
}
fn evidence(inode: Option<u64>, path: &str, reason: ScanReason) -> Observation {
    Observation {
        path: path.into(),
        locator: inode.map(locator),
        size: None,
        modified: None,
        fingerprint: Vec::new(),
        reason: Some(reason),
        errno: None,
    }
}
fn start(r: &mut Reconciler, generation: u64) {
    r.begin(generation, Duration::from_secs(generation * 10), generation)
        .unwrap();
}
fn finish(r: &mut Reconciler, complete: bool, events: &mut Vec<String>) -> PassSummary {
    let pass = r.pass.as_ref().unwrap();
    let completion = ScanCompletion {
        generation: pass.generation,
        complete,
        cancelled: false,
        observations: pass.count,
    };
    let time = pass.started + Duration::from_secs(1);
    r.finish(completion, time, |event| record(event, events))
        .unwrap()
}
fn record(event: Evidence<'_>, events: &mut Vec<String>) {
    events.push(match event {
        Evidence::Present(id) => format!("present:{}", id.0),
        Evidence::Validated(id, _) => format!("validated:{}", id.0),
        Evidence::Excluded(id) => format!("excluded:{}", id.0),
        Evidence::Disappeared(id) => format!("missing:{}", id.0),
        Evidence::CoverageLost(_) => "coverage".into(),
        Evidence::ParentChanged(id) => format!("parent:{}", id.0),
        Evidence::RootChanged(path) => format!("root:{}", path.display()),
        Evidence::Completed(summary) => format!("complete:{}", summary.complete),
    });
}
fn feed(r: &mut Reconciler, observation: &Observation, events: &mut Vec<String>) {
    let generation = r.pass.as_ref().unwrap().generation;
    r.observe(
        generation,
        observation,
        r.pass.as_ref().unwrap().started,
        |event| record(event, events),
    )
    .unwrap();
}

/// Scenario: A complete pass misses an active tracked source.
/// Guarantees: Reasoned disappearance precedes completion and never removes the tracked identity.
#[test]
fn complete_miss_is_evidence_not_removal() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/app.log")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    assert!(finish(&mut r, true, &mut events).complete);
    assert_eq!(events, ["missing:1", "complete:true"]);
    assert!(r.tracked(TrackedId(1)).is_some());
}

/// Scenario: A failure and unreadable locator presence arrive before a pass completes.
/// Guarantees: Failure/presence are delivered immediately and no disappearance follows incomplete coverage.
#[test]
fn unfinished_failure_and_presence_are_immediate() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/app.log")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(
        &mut r,
        &evidence(None, "/", ScanReason::Coverage),
        &mut events,
    );
    feed(
        &mut r,
        &evidence(Some(1), "/app.log", ScanReason::Content),
        &mut events,
    );
    assert_eq!(events, ["coverage", "present:1"]);
    assert!(!finish(&mut r, true, &mut events).complete);
    assert!(!events.iter().any(|event| event.starts_with("missing")));
    assert!(!events.iter().any(|event| event.starts_with("validated")));
}

/// Scenario: Every scanner reason accompanies an otherwise completed pass.
/// Guarantees: Coverage classification matches the scanner and positive observations cannot restore a failed pass.
#[test]
fn every_reason_preserves_scanner_coverage_contract() {
    use ScanReason::*;
    for reason in [
        Directory,
        PresenceOnly,
        VerificationUnavailable,
        Missing,
        LeafLoop,
        Unsupported,
        RootUnavailable,
        RootChanged,
        Coverage,
        Resumption,
        UnqualifiedResumption,
        Cycle,
        Limit,
        ExclusionUnavailable,
        Policy,
        Excluded,
        Content,
        Unstable,
        ProbeBusy,
        DescriptorExhausted,
        Cancelled,
    ] {
        let mut r = Reconciler::new(limits()).unwrap();
        start(&mut r, 1);
        let mut events = Vec::new();
        feed(&mut r, &evidence(Some(10), "/x", reason), &mut events);
        feed(&mut r, &observation(2, "/app.log"), &mut events);
        assert_eq!(
            finish(&mut r, true, &mut events).complete,
            !reason.invalidates_coverage(),
            "{reason:?}"
        );
    }
}

/// Scenario: Caller supplies an early completion, stale observation, overlap and repeated generation.
/// Guarantees: Protocol errors cannot produce complete negative evidence or start overlapping passes.
#[test]
fn ordering_errors_fail_closed() {
    let mut r = Reconciler::new(limits()).unwrap();
    start(&mut r, 1);
    assert_eq!(
        r.begin(2, Duration::from_secs(20), 2),
        Err(ReconcileError::Phase)
    );
    let mut events = Vec::new();
    feed(&mut r, &observation(2, "/app.log"), &mut events);
    assert_eq!(
        r.finish(
            ScanCompletion {
                generation: 1,
                complete: true,
                cancelled: false,
                observations: 0
            },
            Duration::from_secs(11),
            |_| {}
        ),
        Err(ReconcileError::Count)
    );
    assert_eq!(
        r.observe(
            0,
            &observation(2, "/app.log"),
            Duration::from_secs(10),
            |_| {}
        ),
        Err(ReconcileError::Generation)
    );
    assert!(!finish(&mut r, true, &mut events).complete);
    assert_eq!(
        r.begin(1, Duration::from_secs(12), 1),
        Err(ReconcileError::Generation)
    );
    assert_eq!(
        r.observe(
            1,
            &observation(2, "/app.log"),
            Duration::from_secs(10),
            |_| {}
        ),
        Err(ReconcileError::Phase)
    );
}

/// Scenario: A cancelled pass has emitted no explicit scanner failure observation.
/// Guarantees: Cancellation alone prevents negative transitions and preserves the completion interval.
#[test]
fn cancellation_invalidates_completion() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/a")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    let summary = r
        .finish(
            ScanCompletion {
                generation: 1,
                complete: true,
                cancelled: true,
                observations: 0,
            },
            Duration::from_secs(12),
            |event| record(event, &mut events),
        )
        .unwrap();
    assert!(!summary.complete);
    assert_eq!(summary.started, Duration::from_secs(10));
    assert_eq!(summary.finished, Duration::from_secs(12));
    assert_eq!(events, ["coverage", "complete:false"]);
}

/// Scenario: Pending candidates disappear from an incomplete pass, then a complete pass.
/// Guarantees: Unknown candidates retain age but are not offered stale; only complete absence drops pending evidence.
#[test]
fn incomplete_scan_keeps_unseen_pending_but_never_offers_it() {
    let mut r = Reconciler::new(limits()).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(1, "/a"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    start(&mut r, 2);
    let _ = finish(&mut r, false, &mut events);
    assert_eq!(r.stats(Duration::from_secs(30)).pending, 1);
    assert_eq!(
        r.stats(Duration::from_secs(30)).oldest_pending_age,
        Duration::from_secs(20)
    );
    assert_eq!(r.offers().unwrap().count(), 0);
    start(&mut r, 3);
    let _ = finish(&mut r, true, &mut events);
    assert_eq!(r.stats(Duration::from_secs(40)).pending, 0);
}

/// Scenario: Candidate aliases and later passes refresh retained observations in reverse order.
/// Guarantees: First discovery order survives refresh and aliases produce one candidate per locator.
#[test]
fn candidates_keep_oldest_discovery_order() {
    let mut r = Reconciler::new(limits()).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(2, "/z"), &mut events);
    feed(&mut r, &observation(1, "/b"), &mut events);
    feed(&mut r, &observation(2, "/a"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    assert_eq!(
        r.offers()
            .unwrap()
            .map(|o| o.candidate.locator.inode)
            .collect::<Vec<_>>(),
        [2, 1]
    );
    start(&mut r, 2);
    feed(&mut r, &observation(1, "/b"), &mut events);
    feed(&mut r, &observation(2, "/z"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    let offers: Vec<_> = r.offers().unwrap().collect();
    assert_eq!(offers[0].candidate.locator.inode, 2);
    assert_eq!(offers[0].candidate.first_seen, Duration::from_secs(10));
}

/// Scenario: Tracked and pending capacity remain full with one identity offer per pass.
/// Guarantees: Reconnection candidates are still offered, overflow gets recurring opportunities, and population stays bounded.
#[test]
fn full_capacity_and_single_offer_do_not_suppress_overflow() {
    let mut r = Reconciler::new(ReconcileLimits {
        tracked: 1,
        pending: 1,
        offers: 1,
        ..limits()
    })
    .unwrap();
    r.track(file(99, "/old")).unwrap();
    let mut events = Vec::new();
    let mut offered = Vec::new();
    for generation in 1..=8 {
        start(&mut r, generation);
        for inode in 1..=20 {
            feed(
                &mut r,
                &observation(inode, &format!("/{inode}")),
                &mut events,
            );
        }
        assert!(finish(&mut r, true, &mut events).complete);
        let offers: Vec<_> = r.offers().unwrap().collect();
        assert_eq!(offers.len(), 1);
        offered.push(offers[0].candidate.locator.inode);
        assert_eq!(r.stats(Duration::from_secs(100)).pending, 1);
    }
    assert!(offered.contains(&1));
    assert!(offered.iter().any(|inode| *inode != 1));
    assert_eq!(r.stats(Duration::from_secs(100)).overflowing_passes, 8);
}

/// Scenario: The identity owner acknowledges a deferred offer twice and returns feedback after the next scan.
/// Guarantees: Duplicate and stale tokens cannot discard a current candidate or advance admission accounting.
#[test]
fn stale_and_duplicate_feedback_are_rejected() {
    let mut r = Reconciler::new(limits()).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(1, "/a"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    let token = r.offers().unwrap().next().unwrap().token;
    r.acknowledge(token, IdentityFeedback::Deferred, Duration::from_secs(22))
        .unwrap();
    assert_eq!(
        r.acknowledge(token, IdentityFeedback::Admitted, Duration::from_secs(22)),
        Err(ReconcileError::Feedback)
    );
    start(&mut r, 2);
    feed(&mut r, &observation(1, "/a"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    assert_eq!(
        r.acknowledge(token, IdentityFeedback::Admitted, Duration::from_secs(22)),
        Err(ReconcileError::Feedback)
    );
    assert_eq!(r.stats(Duration::from_secs(22)).pending, 1);
    assert!(r.stats(Duration::from_secs(22)).last_admission.is_none());
}

/// Scenario: Stable observations disagree on a retained candidate's fingerprint during one pass.
/// Guarantees: Contradictory evidence is never offered as stable and invalidates completeness.
#[test]
fn conflicting_candidate_evidence_is_not_offered() {
    let mut r = Reconciler::new(limits()).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(1, "/a"), &mut events);
    let mut changed = observation(1, "/a");
    changed.fingerprint = b"xyz".to_vec();
    feed(&mut r, &changed, &mut events);
    assert!(!finish(&mut r, true, &mut events).complete);
    assert_eq!(r.offers().unwrap().count(), 0);
}

/// Scenario: An excluded alias coexists with a fresh eligible distinguished binding.
/// Guarantees: The alias does not revoke the eligible reader, independent of observation order.
#[test]
fn excluded_alias_does_not_revoke_eligible_binding() {
    for reversed in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/a")).unwrap();
        start(&mut r, 1);
        let mut events = Vec::new();
        let excluded = evidence(Some(1), "/secret", ScanReason::Excluded);
        let eligible = observation(1, "/a");
        let observations = if reversed {
            [&eligible, &excluded]
        } else {
            [&excluded, &eligible]
        };
        for observation in observations {
            feed(&mut r, observation, &mut events);
        }
        let _ = finish(&mut r, true, &mut events);
        assert!(!events.iter().any(|event| event.starts_with("excluded")));
    }
}

/// Scenario: A tracked binding is positively excluded while unrelated traversal is incomplete.
/// Guarantees: Binding-check exclusion is prompt, reasoned, and never reported as disappearance.
#[test]
fn direct_exclusion_does_not_wait_for_complete_coverage() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/a")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(
        &mut r,
        &evidence(None, "/other", ScanReason::Coverage),
        &mut events,
    );
    feed(
        &mut r,
        &evidence(Some(1), "/a", ScanReason::Excluded),
        &mut events,
    );
    assert!(events.contains(&"excluded:1".into()));
    let _ = finish(&mut r, false, &mut events);
    assert!(!events.iter().any(|event| event.starts_with("missing")));
}

/// Scenario: Only an excluded alias is seen and the saved binding cannot be validated.
/// Guarantees: Incomplete coverage preserves the reader; complete fallback produces exclusion rather than disappearance.
#[test]
fn uncertain_exclusion_waits_for_complete_fallback() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/a")).unwrap();
    for (generation, complete) in [(1, false), (2, true)] {
        start(&mut r, generation);
        let mut events = Vec::new();
        feed(
            &mut r,
            &evidence(Some(1), "/secret", ScanReason::Excluded),
            &mut events,
        );
        let _ = finish(&mut r, complete, &mut events);
        assert_eq!(events.contains(&"excluded:1".into()), complete);
    }
}

/// Scenario: A tracked parent is replaced, its record becomes inactive, then the parent is unavailable and restored.
/// Guarantees: Replacement identity is never adopted; negative conclusions resume only after the original parent returns.
#[test]
fn parent_replacement_block_survives_pass_boundaries() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/logs/a")).unwrap();
    let mut events = Vec::new();
    for (generation, parent, complete) in [
        (1, Some(10), true),
        (2, Some(20), false),
        (3, None, false),
        (4, Some(10), true),
    ] {
        start(&mut r, generation);
        if let Some(parent) = parent {
            feed(
                &mut r,
                &evidence(Some(parent), "/logs", ScanReason::Directory),
                &mut events,
            );
        }
        assert_eq!(finish(&mut r, true, &mut events).complete, complete);
        if generation == 2 {
            r.update_reader_state(generation, TrackedId(1), false, false)
                .unwrap();
        }
    }
    assert!(events.contains(&"parent:1".into()));
}

/// Scenario: A smaller alias appears while the distinguished binding remains eligible.
/// Guarantees: Metadata and deterministic alias minimum do not replace an established live binding.
#[test]
fn established_binding_wins_over_smaller_alias() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/z")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(1, "/a"), &mut events);
    feed(&mut r, &observation(1, "/z"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    assert_eq!(
        r.binding_proposal(1, TrackedId(1)).unwrap(),
        Path::new("/z")
    );
    assert_eq!(
        r.tracked(TrackedId(1)).unwrap().binding.as_deref(),
        Some(Path::new("/z"))
    );
}

/// Scenario: A new locator replaces app.log during an incomplete scan while its predecessor moves.
/// Guarantees: Replacement context reaches identity, predecessor history survives, and binding reselection remains blocked.
#[test]
fn incomplete_rotation_preserves_replacement_context_and_history() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/app.log")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(1, "/app.log.1"), &mut events);
    feed(&mut r, &observation(2, "/app.log"), &mut events);
    let _ = finish(&mut r, false, &mut events);
    assert_eq!(
        r.offers().unwrap().next().unwrap().candidate.replacement,
        Replacement::Possible(TrackedId(1))
    );
    r.supersede(1, TrackedId(1), locator(2)).unwrap();
    let old = r.tracked(TrackedId(1)).unwrap();
    assert_eq!(old.role, BindingRole::Superseded);
    assert_eq!(old.binding.as_deref(), Some(Path::new("/app.log")));
    assert_eq!(
        r.binding_proposal(1, TrackedId(1)),
        Err(ReconcileError::Feedback)
    );
}

/// Scenario: A complete rotation has not yet received identity feedback for the rebound.
/// Guarantees: New binding publication waits for feedback, and a refused durable commit leaves the old binding unchanged.
#[test]
fn binding_publication_waits_for_identity_and_commit() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/app.log")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(1, "/app.log.1"), &mut events);
    feed(&mut r, &observation(2, "/app.log"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    assert_eq!(
        r.binding_proposal(1, TrackedId(1)),
        Err(ReconcileError::Feedback)
    );
    r.resolve_rebound(1, TrackedId(1), locator(2)).unwrap();
    assert!(!r.rebind(1, TrackedId(1), |_| false).unwrap());
    assert_eq!(
        r.tracked(TrackedId(1)).unwrap().binding.as_deref(),
        Some(Path::new("/app.log"))
    );
    assert!(
        r.rebind(1, TrackedId(1), |path| {
            assert_eq!(path, Path::new("/app.log.1"));
            true
        })
        .unwrap()
    );
    assert_eq!(
        r.tracked(TrackedId(1)).unwrap().binding.as_deref(),
        Some(Path::new("/app.log.1"))
    );
}

/// Scenario: Bad sizes, paths, timestamps and allocation arithmetic cross API boundaries.
/// Guarantees: Invalid input is rejected before retaining unbounded evidence or authorizing a complete pass.
#[test]
fn invalid_inputs_and_limits_fail_closed() {
    assert!(
        Reconciler::new(ReconcileLimits {
            tracked: usize::MAX,
            ..limits()
        })
        .is_err()
    );
    assert!(
        Reconciler::new(ReconcileLimits {
            offers: 0,
            ..limits()
        })
        .is_err()
    );
    let mut r = Reconciler::new(limits()).unwrap();
    start(&mut r, 1);
    let mut invalid = observation(1, "/a");
    invalid.path = "relative".into();
    assert_eq!(
        r.observe(1, &invalid, Duration::from_secs(10), |_| {}),
        Err(ReconcileError::Observation)
    );
    assert!(!finish(&mut r, true, &mut Vec::new()).complete);
    assert_eq!(r.begin(2, Duration::ZERO, 2), Err(ReconcileError::Time));
}

/// Scenario: Delay sampling uses zero jitter and both ends of the allowed jitter interval.
/// Guarantees: Completion-based delay is exact at zero jitter and checked bounds include both endpoints.
#[test]
fn completion_delay_has_checked_inclusive_bounds() {
    assert_eq!(
        reconcile_delay(Duration::from_secs(5), 0, u64::MAX).unwrap(),
        Duration::from_secs(5)
    );
    assert_eq!(
        reconcile_delay(Duration::from_secs(1), 25, 0).unwrap(),
        Duration::from_millis(750)
    );
    assert_eq!(
        reconcile_delay(Duration::from_secs(1), 25, 500_000_000).unwrap(),
        Duration::from_millis(1250)
    );
    assert!(reconcile_delay(Duration::MAX, 10, 0).is_err());
    assert!(reconcile_delay(Duration::ZERO, 0, 0).is_err());
}

/// Scenario: Thousands of passes repeat the same bounded tracked and pending population.
/// Guarantees: Warm retained capacity remains constant and does not grow with pass count.
#[test]
fn steady_state_capacity_does_not_grow() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/a")).unwrap();
    let tracked = observation(1, "/a");
    let pending = observation(2, "/b");
    let mut baseline = 0;
    for generation in 1..=1000 {
        start(&mut r, generation);
        r.observe(
            generation,
            &tracked,
            Duration::from_secs(generation * 10),
            |_| {},
        )
        .unwrap();
        r.observe(
            generation,
            &pending,
            Duration::from_secs(generation * 10),
            |_| {},
        )
        .unwrap();
        let _ = r
            .finish(
                ScanCompletion {
                    generation,
                    complete: true,
                    cancelled: false,
                    observations: 2,
                },
                Duration::from_secs(generation * 10 + 1),
                |_| {},
            )
            .unwrap();
        if generation == 1 {
            baseline = r.retained_bytes();
        }
        assert_eq!(r.retained_bytes(), baseline);
    }
}

/// Scenario: Two tracked live paths exchange locators during a complete traversal.
/// Guarantees: Swaps are incomplete reconciliation and neither binding can be published or superseded.
#[test]
fn tracked_swap_is_deferred_without_reassigning_paths() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/a")).unwrap();
    r.track(file(2, "/b")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(2, "/a"), &mut events);
    feed(&mut r, &observation(1, "/b"), &mut events);
    assert!(!finish(&mut r, true, &mut events).complete);
    assert_eq!(
        r.binding_proposal(1, TrackedId(1)),
        Err(ReconcileError::Feedback)
    );
    assert_eq!(
        r.supersede(1, TrackedId(1), locator(2)),
        Err(ReconcileError::Feedback)
    );
    assert_eq!(
        r.tracked(TrackedId(1)).unwrap().binding.as_deref(),
        Some(Path::new("/a"))
    );
}

/// Scenario: An unavailable saved path has narrowly associated replacement evidence from identity matching.
/// Guarantees: Only the associated candidate gets replacement context; unrelated first admissions keep their normal start policy.
#[test]
fn unavailable_binding_does_not_mark_every_candidate_as_replacement() {
    let mut r = Reconciler::new(limits()).unwrap();
    let mut old = file(1, "/a");
    old.binding = None;
    old.role = BindingRole::Unverified;
    r.track(old).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(2, "/a"), &mut events);
    feed(&mut r, &observation(3, "/unrelated"), &mut events);
    r.associate_replacement(1, TrackedId(1), locator(2), |_| {})
        .unwrap();
    let _ = finish(&mut r, false, &mut events);
    let offers: Vec<_> = r.offers().unwrap().collect();
    assert_eq!(
        offers[0].candidate.replacement,
        Replacement::Possible(TrackedId(1))
    );
    assert_eq!(offers[1].candidate.replacement, Replacement::Unrelated);
}

/// Scenario: Two native aliases differ only by a dot component in their spelling.
/// Guarantees: Initial binding selection orders complete native bytes, not normalized Path components.
#[test]
fn native_binding_minimum_preserves_dot_components() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/old")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(1, "/a/z"), &mut events);
    feed(&mut r, &observation(1, "/a/./z"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    assert_eq!(
        r.binding_proposal(1, TrackedId(1)).unwrap().as_os_str(),
        OsStr::new("/a/./z")
    );
}

/// Scenario: An observation arrives long after scan start and the next timestamp goes backwards.
/// Guarantees: Pending age starts at observation time and nonmonotonic input invalidates the pass.
#[test]
fn discovery_age_uses_observation_time() {
    let mut r = Reconciler::new(limits()).unwrap();
    start(&mut r, 1);
    r.observe(1, &observation(1, "/a"), Duration::from_secs(15), |_| {})
        .unwrap();
    assert_eq!(
        r.stats(Duration::from_secs(20)).oldest_pending_age,
        Duration::from_secs(5)
    );
    assert_eq!(
        r.observe(1, &observation(2, "/b"), Duration::from_secs(14), |_| {}),
        Err(ReconcileError::Time)
    );
    assert!(!r.abort(Duration::from_secs(16), |_| {}).unwrap().complete);
}

struct Policy;
impl super::super::MatchPolicy for Policy {
    fn includes(&self, _: &Path) -> bool {
        true
    }
    fn may_descend(&self, _: &Path) -> bool {
        true
    }
    fn excludes(&self, path: &Path) -> bool {
        path.extension().is_some_and(|ext| ext == "secret")
    }
    fn excludes_subtree(&self, _: &Path) -> bool {
        false
    }
    fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
        false
    }
}
fn scanner(root: &Path) -> super::super::Scanner<Policy> {
    use super::super::*;
    Scanner::new(
        ScanConfig {
            include_roots: vec![root.to_owned()],
            exclusion_roots: vec![],
            symlinks: SymlinkPolicy::Reject,
            checkpoint_namespace: locator(u64::MAX),
            limits: ScanLimits {
                fingerprint_bytes: 16,
                ..ScanLimits::default()
            },
        },
        Policy,
    )
    .unwrap()
}

/// Scenario: A real scanner delivers files, directory evidence and its exact completion to reconciliation.
/// Guarantees: Callback ordering/counts compose, candidates remain descriptor-free, and the transient budget is free at delivery.
#[test]
fn scanner_callbacks_compose_with_reconciliation() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("a.log"), b"abc").unwrap();
    std::fs::write(directory.path().join("b.log"), b"abc").unwrap();
    let mut scanner = scanner(directory.path());
    let mut r = Reconciler::new(limits()).unwrap();
    let budget = super::super::ProbeBudget::default();
    start(&mut r, 1);
    let completion = scanner.scan(
        1,
        &budget,
        || false,
        |batch| {
            assert!(budget.try_reserve().is_some());
            for observation in batch {
                r.observe(1, observation, Duration::from_secs(10), |_| {})
                    .unwrap();
            }
            true
        },
    );
    let summary = r
        .finish(completion, Duration::from_secs(11), |_| {})
        .unwrap();
    assert_eq!(summary.observations, completion.observations);
    assert_eq!(r.offers().unwrap().count(), 2);
}

/// Scenario: A tracked file moves to an excluded name while unrelated coverage fails.
/// Guarantees: A targeted missing-binding check with unchanged parent authorizes exclusion without manufacturing complete coverage.
#[test]
fn targeted_binding_miss_allows_exclusion_during_incomplete_pass() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("a.log");
    let excluded = directory.path().join("a.secret");
    std::fs::write(&path, b"abc").unwrap();
    let actual = FileLocator::from_metadata(&std::fs::metadata(&path).unwrap());
    let parent = FileLocator::from_metadata(&std::fs::metadata(directory.path()).unwrap());
    let mut tracked = file(1, path.to_str().unwrap());
    tracked.locator = actual;
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(tracked).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    let mut directory_observation = evidence(
        None,
        directory.path().to_str().unwrap(),
        ScanReason::Directory,
    );
    directory_observation.locator = Some(parent);
    feed(&mut r, &directory_observation, &mut events);
    std::fs::rename(&path, &excluded).unwrap();
    let mut excluded_observation = evidence(None, excluded.to_str().unwrap(), ScanReason::Excluded);
    excluded_observation.locator = Some(actual);
    feed(&mut r, &excluded_observation, &mut events);
    feed(
        &mut r,
        &evidence(None, "/other", ScanReason::Coverage),
        &mut events,
    );
    assert_eq!(r.binding_checks().count(), 1);
    let mut scanner = scanner(directory.path());
    let budget = super::super::ProbeBudget::default();
    scanner
        .check_binding(
            &path,
            Some(parent),
            &budget,
            || false,
            |check| {
                assert_eq!(check.observation.reason, Some(ScanReason::Missing));
                assert!(check.context_valid);
                assert!(budget.try_reserve().is_some());
                r.apply_binding_check(1, TrackedId(1), check, Duration::from_secs(10), |event| {
                    record(event, &mut events)
                })
                .unwrap();
            },
        )
        .unwrap();
    assert!(events.contains(&"excluded:1".into()));
    assert!(!finish(&mut r, false, &mut events).complete);
}

/// Scenario: A saved binding's parent is replaced before a targeted check.
/// Guarantees: The replacement directory cannot supply a contextual miss or eligible candidate under the old baseline.
#[test]
fn targeted_check_rejects_changed_parent() {
    let directory = tempfile::tempdir().unwrap();
    let parent_path = directory.path().join("logs");
    std::fs::create_dir(&parent_path).unwrap();
    let parent = FileLocator::from_metadata(&std::fs::metadata(&parent_path).unwrap());
    let mut scanner = scanner(directory.path());
    std::fs::rename(&parent_path, directory.path().join("old")).unwrap();
    std::fs::create_dir(&parent_path).unwrap();
    scanner
        .check_binding(
            &parent_path.join("a.log"),
            Some(parent),
            &super::super::ProbeBudget::default(),
            || false,
            |check| {
                assert_eq!(check.observation.reason, Some(ScanReason::RootChanged));
                assert!(!check.context_valid);
                assert!(check.observation.size.is_none());
            },
        )
        .unwrap();
}

/// Scenario: A rejected descendant symlink replaces the parent of a saved binding.
/// Guarantees: Targeted checks preserve follow_symlinks=false and never bypass traversal policy by canonicalizing the parent first.
#[test]
fn targeted_check_rejects_descendant_symlink_parent() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("target");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("a.log"), b"abc").unwrap();
    std::os::unix::fs::symlink(&target, directory.path().join("alias")).unwrap();
    let mut scanner = scanner(directory.path());
    scanner
        .check_binding(
            &directory.path().join("alias/a.log"),
            None,
            &super::super::ProbeBudget::default(),
            || false,
            |check| {
                assert_eq!(check.observation.reason, Some(ScanReason::Coverage));
                assert!(check.observation.size.is_none());
            },
        )
        .unwrap();
}

/// Scenario: Two incomplete move/create rotations leave historical paths equal to the latest live binding.
/// Guarantees: A later complete scan resolves historical rebounds without false swap conflicts or replacement re-registration.
#[test]
fn repeated_incomplete_rotation_reconciles_historical_bindings() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/app.log")).unwrap();
    let mut events = Vec::new();
    for generation in 1..=2 {
        start(&mut r, generation);
        feed(
            &mut r,
            &observation(generation + 1, "/app.log"),
            &mut events,
        );
        let _ = finish(&mut r, false, &mut events);
        r.supersede(generation, TrackedId(generation), locator(generation + 1))
            .unwrap();
        r.track(file(generation + 1, "/app.log")).unwrap();
    }
    start(&mut r, 3);
    feed(&mut r, &observation(1, "/app.log.2"), &mut events);
    feed(&mut r, &observation(2, "/app.log.1"), &mut events);
    feed(&mut r, &observation(3, "/app.log"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    assert_eq!(r.offers().unwrap().count(), 0);
    // Explicit admission of C also resolves A's established successor chain.
    assert_eq!(r.rebounds().count(), 0);
    assert!(
        r.rebind(3, TrackedId(1), |path| path == Path::new("/app.log.2"))
            .unwrap()
    );
    assert!(
        r.rebind(3, TrackedId(2), |path| path == Path::new("/app.log.1"))
            .unwrap()
    );
    assert_eq!(
        r.tracked(TrackedId(3)).unwrap().binding.as_deref(),
        Some(Path::new("/app.log"))
    );
    assert_eq!(r.stats(Duration::from_secs(32)).tracked, 3);
}

/// Scenario: An identity callback could perform durable work for an expired candidate token.
/// Guarantees: Stale feedback is rejected before the callback executes; fresh admission consumes exactly one current offer.
#[test]
fn identity_callback_is_not_invoked_for_stale_offer() {
    let mut r = Reconciler::new(limits()).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(1, "/a"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    let stale = r.offers().unwrap().next().unwrap().token;
    start(&mut r, 2);
    feed(&mut r, &observation(1, "/a"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    assert_eq!(
        r.resolve_offer(stale, |_| panic!("stale identity mutation")),
        Err(ReconcileError::Feedback)
    );
    let current = r.offers().unwrap().next().unwrap().token;
    let mut calls = 0;
    assert_eq!(
        r.resolve_offer(current, |offer| {
            calls += 1;
            assert_eq!(offer.candidate.locator, locator(1));
            (IdentityFeedback::Admitted, Duration::from_secs(22))
        })
        .unwrap(),
        IdentityFeedback::Admitted
    );
    assert_eq!(calls, 1);
    assert_eq!(r.stats(Duration::from_secs(22)).pending, 0);
    assert_eq!(
        r.resolve_offer(current, |_| panic!("duplicate identity mutation")),
        Err(ReconcileError::Feedback)
    );
}

/// Scenario: The oldest pending candidate needs a new slot while a later retained candidate can reconnect a full table.
/// Guarantees: Only proven new identities wait for capacity; unresolved exact-locator candidates still reach identity resolution.
#[test]
fn capacity_feedback_leaves_room_for_retained_reconnection() {
    let mut r = Reconciler::new(ReconcileLimits {
        tracked: 1,
        pending: 2,
        offers: 1,
        ..limits()
    })
    .unwrap();
    let mut tracked = file(9, "/existing");
    tracked.resident = false;
    r.track(tracked).unwrap();
    let mut events = Vec::new();
    for generation in 1..=2 {
        start(&mut r, generation);
        feed(&mut r, &observation(1, "/new"), &mut events);
        feed(&mut r, &observation(9, "/existing"), &mut events);
        let _ = finish(&mut r, true, &mut events);
        let offer = r.offers().unwrap().next().unwrap();
        let token = offer.token;
        assert_eq!(
            offer.candidate.locator,
            locator(if generation == 1 { 1 } else { 9 })
        );
        r.acknowledge(
            token,
            if generation == 1 {
                IdentityFeedback::NeedsCapacity
            } else {
                IdentityFeedback::Admitted
            },
            Duration::from_secs(generation * 10 + 2),
        )
        .unwrap();
    }
    assert_eq!(r.stats(Duration::from_secs(22)).pending, 1);
    r.update_reader_state(2, TrackedId(9), true, true).unwrap();
    assert!(r.tracked(TrackedId(9)).unwrap().resident);
}

/// Scenario: Identity policy rejects a fresh candidate without admitting any record.
/// Guarantees: Rejection releases pending storage without advancing successful-admission diagnostics.
#[test]
fn rejection_does_not_count_as_admission() {
    let mut r = Reconciler::new(limits()).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(1, "/old"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    let token = r.offers().unwrap().next().unwrap().token;
    r.acknowledge(token, IdentityFeedback::Rejected, Duration::from_secs(22))
        .unwrap();
    let stats = r.stats(Duration::from_secs(12));
    assert_eq!(stats.pending, 0);
    assert_eq!(stats.last_admission, None);
}

/// Scenario: A targeted check cannot find a deleted old parent beneath valid include roots.
/// Guarantees: Local uncertainty permits complete-pass exclusion fallback and unrelated disappearance.
#[test]
fn missing_parent_does_not_poison_complete_scan() {
    use super::super::{BindingScope, ProbeBudget};
    let dir = tempfile::tempdir().unwrap();
    let parent = dir.path().join("old");
    std::fs::create_dir(&parent).unwrap();
    let path = parent.join("a.log");
    std::fs::write(&path, b"abc").unwrap();
    let actual = FileLocator::from_metadata(&std::fs::metadata(&path).unwrap());
    let mut old = file(1, path.to_str().unwrap());
    old.locator = actual;
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(old).unwrap();
    r.track(file(2, dir.path().join("missing.log").to_str().unwrap()))
        .unwrap();
    std::fs::rename(&path, dir.path().join("a.secret")).unwrap();
    std::fs::remove_dir(&parent).unwrap();
    let mut scanner = scanner(dir.path());
    let budget = ProbeBudget::default();
    for generation in 1..=2 {
        start(&mut r, generation);
        let mut events = Vec::new();
        let completion = scanner.scan(
            generation,
            &budget,
            || false,
            |batch| {
                for observation in batch {
                    feed(&mut r, observation, &mut events);
                }
                true
            },
        );
        assert!(completion.complete);
        let request = r.binding_request(TrackedId(1)).unwrap();
        let mut batch = scanner.binding_batch(&budget, || false).unwrap();
        let result = batch.check(request.path, request.parent).unwrap();
        assert_eq!(result.scope, BindingScope::Binding);
        assert_eq!(result.observation.reason, Some(ScanReason::Missing));
        assert!(!result.context_valid);
        r.apply_binding_check(
            generation,
            TrackedId(1),
            result,
            Duration::from_secs(generation * 10),
            |e| record(e, &mut events),
        )
        .unwrap();
        assert!(r.binding_request(TrackedId(1)).is_none());
        assert!(
            r.finish(completion, Duration::from_secs(generation * 10 + 1), |e| {
                record(e, &mut events)
            })
            .unwrap()
            .complete
        );
        assert!(events.contains(&"excluded:1".into()));
        assert!(events.contains(&"missing:2".into()));
    }
}

/// Scenario: An include root changes during a targeted check, while the saved parent baseline remains intact.
/// Guarantees: The diagnostic identifies the root and does not incorrectly latch a tracked-parent replacement.
#[test]
fn targeted_root_change_is_not_a_parent_change() {
    use super::super::ProbeBudget;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    std::fs::create_dir(&root).unwrap();
    let path = root.join("a.log");
    let mut scanner = scanner(&root);
    let budget = ProbeBudget::default();
    let _ = scanner.scan(0, &budget, || false, |_| true);
    std::fs::rename(&root, dir.path().join("old")).unwrap();
    std::fs::create_dir(&root).unwrap();
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, path.to_str().unwrap())).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    scanner
        .check_binding(
            &path,
            None,
            &budget,
            || false,
            |check| {
                assert_eq!(check.diagnostic_path, root);
                r.apply_binding_check(1, TrackedId(1), check, Duration::from_secs(10), |e| {
                    record(e, &mut events)
                })
                .unwrap();
            },
        )
        .unwrap();
    assert!(!r.tracked[&TrackedId(1)].parent_changed);
    assert!(events.contains(&format!("root:{}", root.display())));
    assert!(!finish(&mut r, true, &mut events).complete);
}

/// Scenario: A targeted check finds a different directory at the distinguished parent path.
/// Guarantees: ParentChanged identifies the tracked binding and the receiver-wide block remains until recovery.
#[test]
fn targeted_parent_change_emits_parent_diagnostic() {
    use super::super::ProbeBudget;
    let dir = tempfile::tempdir().unwrap();
    let parent_path = dir.path().join("logs");
    std::fs::create_dir(&parent_path).unwrap();
    let expected = FileLocator::from_metadata(&std::fs::metadata(&parent_path).unwrap());
    let path = parent_path.join("a.log");
    let mut scanner = scanner(dir.path());
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, path.to_str().unwrap())).unwrap();
    r.tracked.get_mut(&TrackedId(1)).unwrap().parent = Some(expected);
    std::fs::rename(&parent_path, dir.path().join("old")).unwrap();
    std::fs::create_dir(&parent_path).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    scanner
        .check_binding(
            &path,
            Some(expected),
            &ProbeBudget::default(),
            || false,
            |check| {
                assert_eq!(check.diagnostic_path, parent_path);
                r.apply_binding_check(1, TrackedId(1), check, Duration::from_secs(10), |e| {
                    record(e, &mut events)
                })
                .unwrap();
            },
        )
        .unwrap();
    assert!(events.contains(&"parent:1".into()));
    assert!(r.tracked[&TrackedId(1)].parent_changed);
    assert!(!finish(&mut r, true, &mut events).complete);
}

/// Scenario: A current path becomes a hardlink to another tracked file whose own binding remains valid.
/// Guarantees: A one-way rebound stays complete and exposes identity work instead of manufacturing a swap.
#[test]
fn one_way_hardlink_rebound_is_actionable() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/a")).unwrap();
    r.track(file(2, "/b")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(1, "/a.old"), &mut events);
    feed(&mut r, &observation(2, "/a"), &mut events);
    feed(&mut r, &observation(2, "/b"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    assert_eq!(r.offers().unwrap().count(), 0);
    assert_eq!(
        r.rebounds().collect::<Vec<_>>(),
        vec![Rebound {
            predecessor: TrackedId(1),
            replacement: locator(2)
        }]
    );
    r.resolve_rebound(1, TrackedId(1), locator(2)).unwrap();
    assert_eq!(
        r.binding_proposal(1, TrackedId(1)).unwrap(),
        Path::new("/a.old")
    );
}

/// Scenario: Three tracked bindings form a replacement cycle.
/// Guarantees: Genuine cycles still invalidate reconciliation and cannot publish any new binding.
#[test]
fn three_way_binding_cycle_is_incomplete() {
    let mut r = Reconciler::new(limits()).unwrap();
    for (id, path) in [(1, "/a"), (2, "/b"), (3, "/c")] {
        r.track(file(id, path)).unwrap();
    }
    start(&mut r, 1);
    let mut events = Vec::new();
    for (id, path) in [(2, "/a"), (3, "/b"), (1, "/c")] {
        feed(&mut r, &observation(id, path), &mut events);
    }
    assert!(!finish(&mut r, true, &mut events).complete);
    for id in 1..=3 {
        assert!(r.binding_proposal(1, TrackedId(id)).is_err());
    }
}

/// Scenario: Policy excludes an ancestor subtree but not individual leaf names.
/// Guarantees: Targeted checks apply the same subtree policy as traversal and return no candidate content.
#[test]
fn targeted_check_honors_ancestor_subtree_policy() {
    use super::super::{MatchPolicy, ProbeBudget, ScanConfig, ScanLimits, Scanner};
    struct Pruned;
    impl MatchPolicy for Pruned {
        fn includes(&self, _: &Path) -> bool {
            true
        }
        fn may_descend(&self, _: &Path) -> bool {
            true
        }
        fn excludes(&self, _: &Path) -> bool {
            false
        }
        fn excludes_subtree(&self, path: &Path) -> bool {
            path.file_name().is_some_and(|n| n == "private")
        }
        fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
            false
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let private = dir.path().join("private");
    std::fs::create_dir(&private).unwrap();
    let path = private.join("a.log");
    std::fs::write(&path, b"must not be admitted").unwrap();
    let mut scanner = Scanner::new(
        ScanConfig {
            include_roots: vec![dir.path().to_owned()],
            exclusion_roots: vec![],
            symlinks: super::super::SymlinkPolicy::Reject,
            checkpoint_namespace: locator(u64::MAX),
            limits: ScanLimits::default(),
        },
        Pruned,
    )
    .unwrap();
    scanner
        .check_binding(
            &path,
            None,
            &ProbeBudget::default(),
            || false,
            |check| {
                assert_eq!(check.observation.reason, Some(ScanReason::Excluded));
                assert!(check.observation.fingerprint.is_empty());
                assert!(check.observation.size.is_none());
            },
        )
        .unwrap();
}

/// Scenario: An earlier-pass alternate alias is the only possible evidence against a newly excluded binding.
/// Guarantees: Cached eligibility never prevents revocation by itself; fresh alias validation does.
#[test]
fn earlier_alias_is_revalidated_before_fast_exclusion() {
    use super::super::{BindingCheck, BindingScope};
    for same_locator in [true, false] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/a")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(&mut r, &observation(1, "/a"), &mut events);
        feed(&mut r, &observation(1, "/z"), &mut events);
        let _ = finish(&mut r, true, &mut events);
        events.clear();
        start(&mut r, 2);
        feed(
            &mut r,
            &evidence(Some(1), "/a", ScanReason::Excluded),
            &mut events,
        );
        assert!(!events.contains(&"excluded:1".into()));
        assert_eq!(
            r.binding_request(TrackedId(1)).unwrap().path,
            Path::new("/z")
        );
        let alias = observation(if same_locator { 1 } else { 2 }, "/z");
        r.apply_binding_check(
            2,
            TrackedId(1),
            BindingCheck {
                observation: &alias,
                parent: None,
                context_valid: false,
                scope: BindingScope::Binding,
                diagnostic_path: &alias.path,
            },
            Duration::from_secs(20),
            |e| record(e, &mut events),
        )
        .unwrap();
        assert_eq!(events.contains(&"excluded:1".into()), !same_locator);
        let _ = finish(&mut r, false, &mut events);
    }
}

/// Scenario: One eligible offer group is smaller than its reserved share while the other has spare candidates.
/// Guarantees: All available offer slots are filled in either direction without duplicates.
#[test]
fn offer_budget_backfills_both_groups() {
    for (pending, total) in [(2, 30), (16, 18)] {
        let mut r = Reconciler::new(ReconcileLimits {
            pending,
            offers: 16,
            ..limits()
        })
        .unwrap();
        start(&mut r, 1);
        let mut events = Vec::new();
        for id in 1..=total {
            feed(&mut r, &observation(id, &format!("/{id}")), &mut events);
        }
        let _ = finish(&mut r, true, &mut events);
        let offers: Vec<_> = r
            .offers()
            .unwrap()
            .map(|o| o.candidate.locator.inode)
            .collect();
        assert_eq!(offers.len(), 16);
        let distinct: std::collections::HashSet<_> = offers.iter().collect();
        assert_eq!(distinct.len(), 16);
        let retained: Vec<_> = offers
            .into_iter()
            .filter(|id| *id <= pending as u64)
            .collect();
        assert!(retained.windows(2).all(|pair| pair[0] < pair[1]));
    }
}

/// Scenario: A complete native binding fills the configured byte limit exactly.
/// Guarantees: Reconciliation and targeted scanning reject the same boundary before any I/O.
#[test]
fn binding_path_limit_is_shared_and_strict() {
    let dir = tempfile::tempdir().unwrap();
    let mut scanner = scanner(dir.path());
    let path = format!("/{}", "x".repeat(4095));
    let mut r = Reconciler::new(ReconcileLimits {
        path_bytes: 4096,
        ..limits()
    })
    .unwrap();
    assert_eq!(r.track(file(1, &path)), Err(ReconcileError::Tracked));
    assert!(
        scanner
            .check_binding(
                Path::new(&path),
                None,
                &super::super::ProbeBudget::default(),
                || false,
                |_| panic!("invalid path checked")
            )
            .is_err()
    );
}

/// Scenario: A replacement admission succeeds, fails preflight, or is deferred by the identity owner.
/// Guarantees: No external callback runs after failed preflight, and successful publication updates all local state together.
#[test]
fn replacement_admission_is_preflighted_and_atomic() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/a")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(2, "/a"), &mut events);
    let _ = finish(&mut r, false, &mut events);
    let token = r.offers().unwrap().next().unwrap().token;
    assert!(
        r.admit(token, file(2, "/wrong"), |_| panic!(
            "invalid admission mutated state"
        ))
        .is_err()
    );
    assert!(!r.admit(token, file(2, "/a"), |_| None).unwrap());
    assert_eq!(r.tracked(TrackedId(1)).unwrap().role, BindingRole::Live);
    assert_eq!(r.stats(Duration::from_secs(12)).pending, 1);
    assert!(
        r.admit(token, file(2, "/a"), |_| Some(Duration::from_secs(19)))
            .unwrap()
    );
    assert_eq!(
        r.tracked(TrackedId(1)).unwrap().role,
        BindingRole::Superseded
    );
    assert_eq!(r.tracked(TrackedId(2)).unwrap().role, BindingRole::Live);
    assert_eq!(
        r.stats(Duration::from_secs(20)).last_admission,
        Some(Duration::from_secs(19))
    );
    assert_eq!(r.stats(Duration::from_secs(20)).pending, 0);
    assert!(
        r.admit(token, file(2, "/a"), |_| panic!("duplicate admission"))
            .is_err()
    );
    start(&mut r, 2);
    feed(&mut r, &observation(2, "/a"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    assert_eq!(r.rebounds().count(), 0);
}

/// Scenario: A full tracked table receives a new identity and a reconnectable existing identity.
/// Guarantees: Admission checks capacity before external mutation and permits reconnection without a new slot.
#[test]
fn atomic_admission_preserves_full_table_reconnection() {
    let mut r = Reconciler::new(ReconcileLimits {
        tracked: 1,
        ..limits()
    })
    .unwrap();
    let mut old = file(1, "/a");
    old.resident = false;
    r.track(old).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(2, "/b"), &mut events);
    feed(&mut r, &observation(1, "/a"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    let tokens: Vec<_> = r.offers().unwrap().map(|o| o.token).collect();
    assert_eq!(
        r.admit(tokens[0], file(2, "/b"), |_| panic!("capacity preflight")),
        Err(ReconcileError::Capacity)
    );
    assert!(
        r.admit(tokens[1], file(1, "/a"), |_| Some(Duration::from_secs(12)))
            .unwrap()
    );
    assert_eq!(r.stats(Duration::from_secs(13)).tracked, 1);
}

/// Scenario: An inactive live record is reactivated before or after another record claims its path.
/// Guarantees: Deactivation preserves role, and reactivation cannot introduce two live owners.
#[test]
fn inactive_role_is_preserved_without_duplicate_live_claims() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/a")).unwrap();
    start(&mut r, 1);
    let _ = finish(&mut r, true, &mut Vec::new());
    r.update_reader_state(1, TrackedId(1), false, false)
        .unwrap();
    assert_eq!(r.tracked(TrackedId(1)).unwrap().role, BindingRole::Live);
    r.update_reader_state(1, TrackedId(1), true, true).unwrap();
    r.update_reader_state(1, TrackedId(1), false, false)
        .unwrap();
    r.track(file(2, "/a")).unwrap();
    assert_eq!(
        r.update_reader_state(1, TrackedId(1), true, true),
        Err(ReconcileError::Tracked)
    );
    assert!(!r.tracked(TrackedId(1)).unwrap().active);
}

/// Scenario: Configuration chooses intervals outside the receiver's current defaults.
/// Guarantees: Scheduling enforces arithmetic safety without duplicating configuration policy bounds.
#[test]
fn delay_and_evidence_limits_belong_to_their_owners() {
    let scan = super::super::ScanLimits::default();
    let derived = ReconcileLimits::from_scan(&scan, 1, 2, 3);
    assert_eq!(derived.path_bytes, scan.path_bytes);
    assert_eq!(derived.fingerprint_bytes, scan.fingerprint_bytes);
    assert_eq!(
        reconcile_delay(Duration::from_millis(1), 0, 0),
        Ok(Duration::from_millis(1))
    );
    assert_eq!(
        reconcile_delay(Duration::from_secs(100_000), 0, 0),
        Ok(Duration::from_secs(100_000))
    );
    assert_eq!(
        reconcile_delay(Duration::from_secs(1), 101, 0),
        Err(ReconcileError::Delay)
    );
}

/// Scenario: An existing nonresident identity appears at another identity's live binding during an incomplete pass.
/// Guarantees: Reconnection preserves the predecessor's live claim and unresolved rebound; competing publication fails before commit.
#[test]
fn reconnect_replacement_preserves_predecessor_and_rejects_competing_claim() {
    for competing in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/a")).unwrap();
        let mut prior = file(2, if competing { "/a" } else { "/x.old" });
        prior.resident = false;
        if competing {
            prior.role = BindingRole::Superseded;
        }
        r.track(prior).unwrap();
        start(&mut r, 1);
        let mut events = Vec::new();
        feed(&mut r, &observation(2, "/a"), &mut events);
        let _ = finish(&mut r, false, &mut events);
        let token = r.offers().unwrap().next().unwrap().token;
        let reconnect = file(2, if competing { "/a" } else { "/x.old" });
        let called = std::cell::Cell::new(false);
        let result = r.admit(token, reconnect, |_| {
            called.set(true);
            Some(Duration::from_secs(12))
        });
        assert_eq!(called.get(), !competing);
        assert_eq!(
            result,
            if competing {
                Err(ReconcileError::Tracked)
            } else {
                Ok(true)
            }
        );
        assert_eq!(r.tracked(TrackedId(1)).unwrap().role, BindingRole::Live);
        assert_eq!(
            r.rebounds().collect::<Vec<_>>(),
            vec![Rebound {
                predecessor: TrackedId(1),
                replacement: locator(2),
            }]
        );
        assert_eq!(r.unresolved_rebounds, 1);
    }
}

/// Scenario: Three new-identity rotations use atomic admission or legacy supersede/track while readers are nonresident.
/// Guarantees: All established successor links advance together, and no resolved ancestor repeatedly blocks reselection.
#[test]
fn successor_chains_survive_eviction_through_both_admission_paths() {
    for atomic in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/a")).unwrap();
        let mut events = Vec::new();
        for generation in 1..=3 {
            start(&mut r, generation);
            feed(&mut r, &observation(generation + 1, "/a"), &mut events);
            let _ = finish(&mut r, false, &mut events);
            let mut successor = file(generation + 1, "/a");
            successor.resident = false;
            if atomic {
                let token = r
                    .offers()
                    .unwrap()
                    .find(|o| o.candidate.locator == successor.locator)
                    .unwrap()
                    .token;
                assert!(
                    r.admit(token, successor, |_| Some(Duration::from_secs(
                        generation * 10 + 2
                    )))
                    .unwrap()
                );
            } else {
                r.supersede(generation, TrackedId(generation), successor.locator)
                    .unwrap();
                assert!(r.tracked[&TrackedId(generation)].successor.is_none());
                r.track(successor).unwrap();
            }
            assert_eq!(r.unresolved_rebounds, 0);
            for id in 1..=generation {
                assert_eq!(
                    r.tracked[&TrackedId(id)].successor,
                    Some((TrackedId(generation + 1), locator(generation + 1)))
                );
            }
        }
        start(&mut r, 4);
        feed(&mut r, &observation(4, "/a"), &mut events);
        feed(&mut r, &observation(1, "/a.old"), &mut events);
        assert!(finish(&mut r, true, &mut events).complete);
        assert_eq!(r.rebounds().count(), 0);
        assert!(r.rebind(4, TrackedId(1), |_| true).unwrap());
        assert!(r.tracked[&TrackedId(1)].successor.is_none());
    }
}

/// Scenario: A successor is removed and another runtime identity reuses its locator.
/// Guarantees: Locator reuse alone cannot resolve an earlier identity's rebound memo.
#[test]
fn successor_memo_requires_runtime_identity_not_just_locator() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/a")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(2, "/a"), &mut events);
    let _ = finish(&mut r, false, &mut events);
    let token = r.offers().unwrap().next().unwrap().token;
    assert!(
        r.admit(token, file(2, "/a"), |_| Some(Duration::from_secs(12)))
            .unwrap()
    );
    drop(r.untrack(TrackedId(2)).unwrap());
    let mut reused = file(3, "/a");
    reused.locator = locator(2);
    r.track(reused).unwrap();
    start(&mut r, 2);
    feed(&mut r, &observation(2, "/a"), &mut events);
    let _ = finish(&mut r, true, &mut events);
    assert_eq!(r.rebounds().count(), 1);
    assert_eq!(r.unresolved_rebounds, 1);
}

/// Scenario: Traversal detects a changed include-root identity before scan completion.
/// Guarantees: The directory path diagnostic precedes coverage loss and incomplete completion.
#[test]
fn traversal_root_change_identifies_the_directory_immediately() {
    let mut r = Reconciler::new(limits()).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(
        &mut r,
        &evidence(None, "/logs", ScanReason::RootChanged),
        &mut events,
    );
    assert_eq!(events, ["root:/logs", "coverage"]);
    assert!(!finish(&mut r, true, &mut events).complete);
}

/// Scenario: An earlier eligible alias is missing, replaced, or temporarily inaccessible during an exclusion check.
/// Guarantees: Dead aliases expire, transient uncertainty preserves the hint, and queued work remains bounded across passes.
#[test]
fn binding_worklist_expires_dead_aliases_but_preserves_uncertain_hints() {
    use super::super::{BindingCheck, BindingScope};
    for reason in [Some(ScanReason::Missing), None, Some(ScanReason::ProbeBusy)] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/a")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(&mut r, &observation(1, "/z"), &mut events);
        let _ = finish(&mut r, true, &mut events);
        start(&mut r, 2);
        for _ in 0..8 {
            feed(
                &mut r,
                &evidence(Some(1), "/a", ScanReason::Excluded),
                &mut events,
            );
        }
        assert_eq!(r.binding_work.len(), 1);
        assert_eq!(r.next_binding_check(), Some(TrackedId(1)));
        let alias = reason.map_or_else(
            || observation(2, "/z"),
            |reason| evidence(None, "/z", reason),
        );
        r.apply_binding_check(
            2,
            TrackedId(1),
            BindingCheck {
                observation: &alias,
                parent: None,
                context_valid: false,
                scope: BindingScope::Binding,
                diagnostic_path: &alias.path,
            },
            Duration::from_secs(20),
            |_| {},
        )
        .unwrap();
        assert_eq!(r.next_binding_check(), None);
        let _ = finish(&mut r, false, &mut events);
        start(&mut r, 3);
        feed(
            &mut r,
            &evidence(Some(1), "/a", ScanReason::Excluded),
            &mut events,
        );
        assert_eq!(
            r.next_binding_check().is_some(),
            reason == Some(ScanReason::ProbeBusy)
        );
    }
}

/// Scenario: A superseded predecessor regains its unchanged historical name after the successor is retired.
/// Guarantees: Same-path binding publication restores Live and clears stale successor evidence.
#[test]
fn same_path_rebind_clears_successor_memo() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/a")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(2, "/a"), &mut events);
    let _ = finish(&mut r, false, &mut events);
    let token = r.offers().unwrap().next().unwrap().token;
    assert!(
        r.admit(token, file(2, "/a"), |_| Some(Duration::from_secs(12)))
            .unwrap()
    );
    drop(r.untrack(TrackedId(2)).unwrap());
    start(&mut r, 2);
    feed(&mut r, &observation(1, "/a"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(r.rebind(2, TrackedId(1), |_| true).unwrap());
    assert_eq!(r.tracked(TrackedId(1)).unwrap().role, BindingRole::Live);
    assert!(r.tracked[&TrackedId(1)].successor.is_none());
}

/// Scenario: Rebinding one of two historical path claims changes its parent, then removal reuses its dense slot.
/// Guarantees: ID-only indexes retain the other claim, notify only the correct parents, and never expose a recycled slot as its old identity.
#[test]
fn compact_indexes_preserve_shared_keys_across_rebind_and_slot_reuse() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/shared/a")).unwrap();
    let mut historical = file(2, "/shared/a");
    historical.role = BindingRole::Superseded;
    r.track(historical).unwrap();
    r.track(file(3, "/shared/b")).unwrap();
    r.track(file(4, "/other/c")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(10), "/shared", ScanReason::Directory),
        &mut events,
    );
    feed(&mut r, &observation(1, "/new/a"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(r.rebind(1, TrackedId(1), |_| true).unwrap());
    assert_eq!(
        r.bindings
            .get(OsStr::new("/shared/a"), &r.tracked)
            .collect::<Vec<_>>(),
        vec![TrackedId(2)]
    );
    start(&mut r, 2);
    events.clear();
    feed(
        &mut r,
        &evidence(Some(11), "/shared", ScanReason::Directory),
        &mut events,
    );
    assert!(events.contains(&"parent:2".into()));
    assert!(events.contains(&"parent:3".into()));
    assert!(!events.contains(&"parent:1".into()));
    assert!(!events.contains(&"parent:4".into()));
    let _ = finish(&mut r, false, &mut events);
    drop(r.untrack(TrackedId(1)).unwrap());
    r.track(file(5, "/fresh/d")).unwrap();
    for id in 2..=5 {
        assert_eq!(r.tracked(TrackedId(id)).unwrap().locator, locator(id));
    }
    assert_eq!(r.bindings.get(OsStr::new("/new/a"), &r.tracked).count(), 0);
    assert_eq!(r.parents.get(OsStr::new("/new"), &r.tracked).count(), 0);
    assert_eq!(r.locators.get(&locator(1), &r.tracked).count(), 0);
    assert_eq!(
        r.bindings
            .get(OsStr::new("/fresh/d"), &r.tracked)
            .collect::<Vec<_>>(),
        vec![TrackedId(5)]
    );
}

/// Scenario: Near-limit aliases grow twice while callers supply bindings with excessive spare capacity.
/// Guarantees: Each retained path buffer is bounded by L, bindings own exact-length storage, and indexes own no path copies.
#[test]
fn compact_paths_bound_growth_without_charging_caller_spare_capacity() {
    let mut r = Reconciler::new(limits()).unwrap();
    let mut binding = PathBuf::with_capacity(100_000);
    binding.push("/binding");
    let mut tracked = file(1, "/binding");
    tracked.binding = Some(binding);
    r.track(tracked).unwrap();
    assert_eq!(
        r.tracked(TrackedId(1))
            .unwrap()
            .binding
            .as_ref()
            .unwrap()
            .capacity(),
        "/binding".len()
    );
    let mut events = Vec::new();
    for generation in 1..=2 {
        start(&mut r, generation);
        feed(
            &mut r,
            &observation(1, &format!("/z{}", "x".repeat(1020))),
            &mut events,
        );
        feed(
            &mut r,
            &observation(1, &format!("/a{}", "x".repeat(1021))),
            &mut events,
        );
        let _ = finish(&mut r, true, &mut events);
        let tracked = &r.tracked[&TrackedId(1)];
        assert!(tracked.minimum.capacity() <= limits().path_bytes);
        assert!(tracked.previous_alias.capacity() <= limits().path_bytes);
    }
}

/// Scenario: Populated indexes cross hash-capacity boundaries and repeatedly remove/reinsert short-path records.
/// Guarantees: Metadata and native path storage fit the compact bound without index growth under churn.
#[test]
fn compact_storage_bound_covers_populated_index_capacity_boundaries() {
    for count in [
        1, 2, 3, 7, 8, 15, 16, 31, 32, 63, 64, 111, 112, 113, 127, 128, 129, 255, 256, 257, 895,
        896, 897,
    ] {
        let mut r = Reconciler::new(ReconcileLimits {
            tracked: count,
            pending: 1,
            offers: 1,
            path_bytes: 32,
            fingerprint_bytes: 16,
        })
        .unwrap();
        for id in 1..=count as u64 {
            r.track(file(id, &format!("/d{id:08}/f"))).unwrap();
        }
        let retained = r.retained_bytes();
        let charge = 4096 + 32 + 16 + count * (3 * 32 + 512) + 2 * (2 * (32 + 16) + 512);
        assert!(
            retained <= charge,
            "population {count}: {retained} exceeds {charge}"
        );
        for id in 1..=4 * count as u64 {
            drop(r.untrack(TrackedId(id)).unwrap());
            let new = id + count as u64;
            r.track(file(new, &format!("/d{new:08}/f"))).unwrap();
        }
        assert_eq!(
            r.retained_bytes(),
            retained,
            "population {count} grew after churn"
        );
    }
}

/// Scenario: A record layout or index representation changes on the pinned 64-bit platform.
/// Guarantees: The per-record metadata charge covers packed records, worklists and maximum rounded ID-table payloads.
#[test]
fn compact_metadata_charge_covers_structural_upper_bound() {
    // Reserving 2N entries at 7/8 occupancy rounds to fewer than 32N/7 buckets.
    // Each of five tables stores an eight-byte ID/slot and a control byte.
    // Fixed table padding and owner fields belong to the separate 4096-byte charge.
    let indexes = (5 * (size_of::<TrackedId>() + 1) * 32).div_ceil(7);
    let per_record = size_of::<Tracked>() + 3 * size_of::<TrackedId>() + indexes;
    assert!(
        per_record <= 512,
        "metadata layout {per_record} exceeds its charge"
    );
}

/// Scenario: An earlier alias loses required coverage before the binding is proved missing or excluded.
/// Guarantees: Later local evidence cannot revoke the reader in that pass; a fresh pass can establish exclusion.
#[test]
fn alias_coverage_failure_stays_latched_until_next_pass() {
    use super::super::{BindingCheck, BindingScope};
    for failure in [
        ScanReason::Coverage,
        ScanReason::Policy,
        ScanReason::RootUnavailable,
    ] {
        for binding_reason in [ScanReason::Missing, ScanReason::Excluded] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/a")).unwrap();
            r.track(file(2, "/unseen")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(
                &mut r,
                &evidence(Some(99), "/", ScanReason::Directory),
                &mut events,
            );
            feed(&mut r, &observation(1, "/a"), &mut events);
            feed(&mut r, &observation(1, "/z"), &mut events);
            let _ = finish(&mut r, true, &mut events);
            for generation in [2, 3] {
                events.clear();
                start(&mut r, generation);
                feed(
                    &mut r,
                    &evidence(Some(1), "/excluded-alias", ScanReason::Excluded),
                    &mut events,
                );
                assert_eq!(r.next_binding_check(), Some(TrackedId(1)));
                assert_eq!(
                    r.binding_request(TrackedId(1)).unwrap().path,
                    Path::new("/z")
                );
                let alias = evidence(
                    None,
                    "/z",
                    if generation == 2 {
                        failure
                    } else {
                        ScanReason::Missing
                    },
                );
                r.apply_binding_check(
                    generation,
                    TrackedId(1),
                    BindingCheck {
                        observation: &alias,
                        parent: None,
                        context_valid: false,
                        scope: if generation == 2 {
                            BindingScope::Coverage
                        } else {
                            BindingScope::Binding
                        },
                        diagnostic_path: &alias.path,
                    },
                    Duration::from_secs(generation * 10),
                    |e| record(e, &mut events),
                )
                .unwrap();
                assert_eq!(r.next_binding_check(), Some(TrackedId(1)));
                assert_eq!(
                    r.binding_request(TrackedId(1)).unwrap().path,
                    Path::new("/a")
                );
                let binding = evidence(None, "/a", binding_reason);
                r.apply_binding_check(
                    generation,
                    TrackedId(1),
                    BindingCheck {
                        observation: &binding,
                        parent: Some(locator(99)),
                        context_valid: true,
                        scope: BindingScope::Binding,
                        diagnostic_path: &binding.path,
                    },
                    Duration::from_secs(generation * 10),
                    |e| record(e, &mut events),
                )
                .unwrap();
                assert_eq!(r.next_binding_check(), None);
                assert_eq!(events.contains(&"excluded:1".into()), generation == 3);
                assert_eq!(finish(&mut r, true, &mut events).complete, generation == 3);
                assert_eq!(events.contains(&"missing:2".into()), generation == 3);
            }
        }
    }
}

/// Scenario: Recovery supplies directory spellings rather than complete filenames as bindings.
/// Guarantees: Tracking and targeted checks reject the same spellings, while directory observations remain valid.
#[test]
fn recovered_bindings_share_scanner_filename_validation() {
    let root = tempfile::tempdir().unwrap();
    let mut scanner = scanner(root.path());
    for path in ["/", "/.", "/var/log/app.log/", "/var/log/app.log/."] {
        let mut r = Reconciler::new(limits()).unwrap();
        assert_eq!(r.track(file(1, path)), Err(ReconcileError::Tracked));
        assert!(
            scanner
                .check_binding(
                    Path::new(path),
                    None,
                    &super::super::ProbeBudget::default(),
                    || false,
                    |_| panic!("invalid filename produced evidence")
                )
                .is_err()
        );
        start(&mut r, 1);
        let mut events = Vec::new();
        feed(
            &mut r,
            &evidence(Some(99), path, ScanReason::Directory),
            &mut events,
        );
        assert_eq!(
            r.observe(1, &observation(1, path), Duration::from_secs(10), |_| {}),
            Err(ReconcileError::Observation)
        );
        assert!(!finish(&mut r, true, &mut events).complete);
    }
}

/// Scenario: A scanner with a smaller path bound rejects a valid reconciler request before returning evidence.
/// Guarantees: Explicit failure completion advances the cursor without exclusion or disappearance, and rejects stale skips.
#[test]
fn rejected_binding_request_can_advance_without_negative_evidence() {
    use super::super::{ProbeBudget, ScanConfig, ScanLimits, Scanner, SymlinkPolicy};
    let root = tempfile::tempdir().unwrap();
    let mut scanner = Scanner::new(
        ScanConfig {
            include_roots: vec![root.path().to_owned()],
            exclusion_roots: vec![],
            symlinks: SymlinkPolicy::Reject,
            checkpoint_namespace: locator(u64::MAX),
            limits: ScanLimits {
                path_bytes: 256,
                ..ScanLimits::default()
            },
        },
        Policy,
    )
    .unwrap();
    let mut r = Reconciler::new(limits()).unwrap();
    let long = format!("/{}", "a".repeat(300));
    r.track(file(1, &long)).unwrap();
    r.track(file(2, "/b")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    for id in [1, 2] {
        feed(
            &mut r,
            &evidence(Some(id), "/excluded", ScanReason::Excluded),
            &mut events,
        );
    }
    let budget = ProbeBudget::default();
    let mut batch = scanner.binding_batch(&budget, || false).unwrap();
    assert_eq!(r.next_binding_check(), Some(TrackedId(1)));
    let request = r.binding_request(TrackedId(1)).unwrap();
    assert!(batch.check(request.path, request.parent).is_err());
    assert_eq!(
        r.skip_binding_checks(0, TrackedId(1), |_| {}),
        Err(ReconcileError::Generation)
    );
    r.skip_binding_checks(1, TrackedId(1), |e| record(e, &mut events))
        .unwrap();
    assert_eq!(
        r.skip_binding_checks(1, TrackedId(1), |_| {}),
        Err(ReconcileError::Feedback)
    );
    assert_eq!(r.next_binding_check(), Some(TrackedId(2)));
    let request = r.binding_request(TrackedId(2)).unwrap();
    let check = batch.check(request.path, request.parent).unwrap();
    r.apply_binding_check(1, TrackedId(2), check, Duration::from_secs(10), |e| {
        record(e, &mut events)
    })
    .unwrap();
    assert_eq!(r.next_binding_check(), None);
    assert!(!finish(&mut r, true, &mut events).complete);
    assert!(!events.contains(&"excluded:1".into()));
    assert!(!events.iter().any(|e| e.starts_with("missing:")));
}

/// Scenario: A full tracked table defers a replacement, then explicit retirement frees its predecessor's slot.
/// Guarantees: Fresh admission retains historical start policy without requiring the retired predecessor to exist.
#[test]
fn deferred_replacement_survives_predecessor_retirement() {
    let mut r = Reconciler::new(ReconcileLimits {
        tracked: 1,
        ..limits()
    })
    .unwrap();
    r.track(file(1, "/a")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(2, "/a"), &mut events);
    let _ = finish(&mut r, false, &mut events);
    let token = r.offers().unwrap().next().unwrap().token;
    r.acknowledge(
        token,
        IdentityFeedback::NeedsCapacity,
        Duration::from_secs(11),
    )
    .unwrap();
    drop(r.untrack(TrackedId(1)).unwrap());
    start(&mut r, 2);
    feed(&mut r, &observation(2, "/a"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    let offer = r.offers().unwrap().next().unwrap();
    assert_eq!(offer.candidate.first_seen, Duration::from_secs(10));
    let token = offer.token;
    assert!(
        r.admit(token, file(2, "/a"), |candidate| {
            assert_eq!(candidate.replacement, Replacement::Possible(TrackedId(1)));
            Some(Duration::from_secs(22))
        })
        .unwrap()
    );
    assert!(r.tracked(TrackedId(1)).is_none());
    assert_eq!(r.tracked(TrackedId(2)).unwrap().role, BindingRole::Live);
}

/// Scenario: A deferred replacement moves to an archive while its predecessor's path changes again, possibly unstably.
/// Guarantees: Historical context neither supersedes nor blocks on the unrelated current rebound; stable archived evidence remains admissible.
#[test]
fn deferred_replacement_survives_further_rotation() {
    for unstable_current_binding in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/a")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(&mut r, &observation(2, "/a"), &mut events);
        let _ = finish(&mut r, false, &mut events);
        let token = r.offers().unwrap().next().unwrap().token;
        r.acknowledge(token, IdentityFeedback::Deferred, Duration::from_secs(11))
            .unwrap();
        start(&mut r, 2);
        feed(&mut r, &observation(2, "/archive"), &mut events);
        feed(&mut r, &observation(3, "/a"), &mut events);
        if unstable_current_binding {
            feed(&mut r, &observation(4, "/a"), &mut events);
        }
        let _ = finish(&mut r, false, &mut events);
        let offer = r
            .offers()
            .unwrap()
            .find(|o| o.candidate.locator == locator(2))
            .unwrap();
        let token = offer.token;
        assert!(
            r.admit(token, file(2, "/archive"), |candidate| {
                assert_eq!(candidate.replacement, Replacement::Possible(TrackedId(1)));
                Some(Duration::from_secs(22))
            })
            .unwrap()
        );
        assert_eq!(r.tracked(TrackedId(1)).unwrap().role, BindingRole::Live);
        assert_eq!(
            r.rebounds().collect::<Vec<_>>(),
            vec![Rebound {
                predecessor: TrackedId(1),
                replacement: locator(if unstable_current_binding { 4 } else { 3 }),
            }]
        );
        if !unstable_current_binding {
            let token = r.offers().unwrap().next().unwrap().token;
            assert!(
                r.admit(token, file(3, "/a"), |_| Some(Duration::from_secs(23)))
                    .unwrap()
            );
            assert_eq!(
                r.tracked[&TrackedId(1)].successor,
                Some((TrackedId(3), locator(3)))
            );
        }
    }
}

/// Scenario: Two tracked paths become stable hardlinks to a new candidate, in either traversal order.
/// Guarantees: Admission publishes the minimum path, supersedes only that path's owner, and retains other rebound work.
#[test]
fn hardlink_admission_uses_selected_path_owner() {
    for reverse in [false, true] {
        for complete in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/a")).unwrap();
            r.track(file(3, "/b")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(&mut r, &observation(1, "/a.old"), &mut events);
            feed(&mut r, &observation(3, "/b.old"), &mut events);
            for path in if reverse { ["/b", "/a"] } else { ["/a", "/b"] } {
                feed(&mut r, &observation(2, path), &mut events);
            }
            assert_eq!(finish(&mut r, complete, &mut events).complete, complete);
            let offer = r.offers().unwrap().next().unwrap();
            assert_eq!(offer.candidate.path, Path::new("/a"));
            assert_eq!(
                offer.candidate.replacement,
                Replacement::Possible(TrackedId(1))
            );
            let token = offer.token;
            assert!(
                r.admit(token, file(2, "/a"), |_| Some(Duration::from_secs(12)))
                    .unwrap()
            );
            assert_eq!(
                r.tracked(TrackedId(1)).unwrap().role,
                BindingRole::Superseded
            );
            assert_eq!(r.tracked(TrackedId(3)).unwrap().role, BindingRole::Live);
            assert_eq!(
                r.rebounds().collect::<Vec<_>>(),
                vec![Rebound {
                    predecessor: TrackedId(3),
                    replacement: locator(2)
                }]
            );
            assert_eq!(
                r.binding_proposal(1, TrackedId(3)),
                Err(ReconcileError::Feedback)
            );
            r.resolve_rebound(1, TrackedId(3), locator(2)).unwrap();
            if complete {
                assert!(r.rebind(1, TrackedId(3), |_| true).unwrap());
                assert_eq!(
                    r.tracked(TrackedId(3)).unwrap().binding.as_deref(),
                    Some(Path::new("/b.old"))
                );
            }
        }
    }
}

/// Scenario: An overflow sample becomes inconsistent, then a better-ranked locator evicts that sample.
/// Guarantees: Eviction cannot restore completeness or emit disappearance; the next stable pass resets the latch.
#[test]
fn evicted_overflow_conflict_keeps_pass_incomplete() {
    for shrink in [false, true] {
        let mut r = Reconciler::new(ReconcileLimits {
            pending: 1,
            offers: 1,
            ..limits()
        })
        .unwrap();
        r.track(file(2000, "/unseen")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(&mut r, &observation(1, "/pending"), &mut events);
        feed(&mut r, &observation(2, "/overflow"), &mut events);
        let mut changed = observation(2, "/overflow");
        if shrink {
            changed.size = Some(2);
            changed.fingerprint = b"ab".to_vec();
        } else {
            changed.fingerprint = b"xyz".to_vec();
        }
        feed(&mut r, &changed, &mut events);
        // Fixed generation, seed and inventory force replacement of the initial
        // sample. Assert the eviction below instead of assuming it happened.
        for inode in 3..=1024 {
            feed(
                &mut r,
                &observation(inode, &format!("/other/{inode}")),
                &mut events,
            );
        }
        assert!(!finish(&mut r, true, &mut events).complete);
        assert!(!events.contains(&"missing:2000".into()));
        let offered = r.offers().unwrap().next().unwrap().candidate.locator;
        assert_ne!(offered, locator(1));
        assert_ne!(offered, locator(2));
        events.clear();
        start(&mut r, 2);
        assert!(finish(&mut r, true, &mut events).complete);
        assert!(events.contains(&"missing:2000".into()));
    }
}

/// Scenario: A 10,000-offer batch receives forward or reverse feedback, mixing deferral and rejection.
/// Guarantees: Every selected token resolves once, duplicate/stale callbacks never execute, and exhausted batches expose no offers.
#[test]
fn large_offer_batch_preserves_token_lifecycle() {
    for reverse in [false, true] {
        let mut r = Reconciler::new(ReconcileLimits {
            pending: 10000,
            offers: 10000,
            ..limits()
        })
        .unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        for inode in 1..=10000 {
            feed(
                &mut r,
                &observation(inode, &format!("/candidate/{inode}")),
                &mut events,
            );
        }
        assert!(finish(&mut r, true, &mut events).complete);
        let tokens: Vec<_> = r.offers().unwrap().map(|o| o.token).collect();
        assert_eq!(tokens.len(), 10000);
        for index in 0..tokens.len() {
            let token = if reverse {
                tokens[tokens.len() - index - 1]
            } else {
                r.offers().unwrap().next().unwrap().token
            };
            let expected = if index % 2 == 0 {
                IdentityFeedback::Deferred
            } else {
                IdentityFeedback::Rejected
            };
            assert_eq!(
                r.resolve_offer(token, |_| (expected, Duration::from_secs(12)))
                    .unwrap(),
                expected
            );
            assert_eq!(
                r.resolve_offer(token, |_| panic!("duplicate token callback")),
                Err(ReconcileError::Feedback)
            );
        }
        assert_eq!(r.offers().unwrap().count(), 0);
        start(&mut r, 2);
        feed(&mut r, &observation(10001, "/fresh"), &mut events);
        let _ = finish(&mut r, true, &mut events);
        assert_eq!(
            r.resolve_offer(tokens[0], |_| panic!("stale token callback")),
            Err(ReconcileError::Feedback)
        );
        let fresh = r.offers().unwrap().next().unwrap().token;
        assert_eq!(
            r.resolve_offer(fresh, |_| (
                IdentityFeedback::Deferred,
                Duration::from_secs(22)
            ))
            .unwrap(),
            IdentityFeedback::Deferred
        );
    }
}

/// Scenario: A scan detects candidate shrink or changed fingerprint after earlier passes supplied disappearance evidence.
/// Guarantees: The consumer receives coverage loss before observe returns, even if the new pass never finishes.
#[test]
fn candidate_conflicts_are_delivered_before_completion() {
    for shrink in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(99, "/missing")).unwrap();
        let mut events = Vec::new();
        for generation in [1, 2] {
            start(&mut r, generation);
            assert!(finish(&mut r, true, &mut events).complete);
        }
        events.clear();
        start(&mut r, 3);
        feed(&mut r, &observation(1, "/new"), &mut events);
        assert!(events.is_empty());
        let mut changed = observation(1, "/new");
        if shrink {
            changed.size = Some(2);
            changed.fingerprint = b"ab".to_vec();
        } else {
            changed.fingerprint = b"xyz".to_vec();
        }
        r.observe(3, &changed, Duration::from_secs(30), |e| {
            assert!(matches!(e, Evidence::CoverageLost(ScanReason::Unstable)));
            record(e, &mut events);
        })
        .unwrap();
        assert_eq!(events, ["coverage"]);
        assert!(!r.pass.as_ref().unwrap().complete);
        // Deliberately do not finish: the worker may stall after this observation.
    }
}

/// Scenario: A tracked binding observes its owner and replacement in either order, or two different replacements.
/// Guarantees: Contradictions immediately invalidate coverage; returning owner evidence cannot precede the failure notification.
#[test]
fn tracked_binding_conflicts_are_delivered_before_completion() {
    for (first, second) in [(1, 2), (2, 1), (2, 3)] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/bound")).unwrap();
        start(&mut r, 1);
        let mut events = Vec::new();
        feed(&mut r, &observation(first, "/bound"), &mut events);
        assert!(!events.contains(&"coverage".into()));
        events.clear();
        feed(&mut r, &observation(second, "/bound"), &mut events);
        let failure = events
            .iter()
            .position(|e| e == "coverage")
            .expect("immediate conflict evidence");
        if second == 1 {
            assert!(failure < events.iter().position(|e| e == "validated:1").unwrap());
        }
        assert!(!r.pass.as_ref().unwrap().complete);
    }
}

/// Scenario: Identity feedback associates two fresh candidates with the same predecessor during an unfinished pass.
/// Guarantees: An owner-supplied contradiction also emits coverage loss immediately, without waiting for finish.
#[test]
fn associated_replacement_conflicts_are_delivered_before_completion() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/bound")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(2, "/b"), &mut events);
    feed(&mut r, &observation(3, "/c"), &mut events);
    r.associate_replacement(1, TrackedId(1), locator(2), |e| record(e, &mut events))
        .unwrap();
    assert!(events.is_empty());
    r.associate_replacement(1, TrackedId(1), locator(3), |e| record(e, &mut events))
        .unwrap();
    assert_eq!(events, ["coverage"]);
    assert!(!r.pass.as_ref().unwrap().complete);
}

/// Scenario: Stable probes of one untracked native path report different locators in either order.
/// Guarantees: Both candidates are withheld, failure is immediate, and completion cannot emit disappearance.
#[test]
fn conflicting_candidate_paths_invalidate_inventory() {
    use std::os::unix::ffi::OsStrExt;
    for path in [
        PathBuf::from("/new"),
        PathBuf::from(OsStr::from_bytes(b"/logs/\xff")),
    ] {
        for reverse in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(99, "/missing")).unwrap();
            start(&mut r, 1);
            let mut events = Vec::new();
            for inode in if reverse { [2, 1] } else { [1, 2] } {
                let mut seen = observation(inode, "/unused");
                seen.path = path.clone();
                feed(&mut r, &seen, &mut events);
            }
            assert_eq!(events, ["coverage"]);
            assert!(!r.pass.as_ref().unwrap().complete);
            assert!(!finish(&mut r, true, &mut events).complete);
            assert_eq!(r.offers().unwrap().count(), 0);
            assert!(!events.contains(&"missing:99".into()));
        }
    }
}

/// Scenario: A candidate's selected path conflicts with another candidate or a resident alias, in either order.
/// Guarantees: Retained alias evidence makes failure immediate in both orders and prevents unrelated disappearance.
#[test]
fn selected_path_changes_and_resident_aliases_check_candidates() {
    for resident in [false, true] {
        for resident_first in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(99, "/missing")).unwrap();
            if resident {
                r.track(file(2, "/resident")).unwrap();
            }
            start(&mut r, 1);
            let mut events = Vec::new();
            if resident_first {
                feed(&mut r, &observation(2, "/a"), &mut events);
            }
            feed(&mut r, &observation(1, "/z"), &mut events);
            assert!(!events.contains(&"coverage".into()));
            feed(&mut r, &observation(1, "/a"), &mut events);
            if !resident_first {
                assert!(!events.contains(&"coverage".into()));
                feed(&mut r, &observation(2, "/a"), &mut events);
            }
            assert!(events.contains(&"coverage".into()));
            assert!(!r.pass.as_ref().unwrap().complete);
            assert!(!finish(&mut r, true, &mut events).complete);
            assert_eq!(r.offers().unwrap().count(), 0);
            assert!(!events.contains(&"missing:99".into()));
        }
    }
}

/// Scenario: A retained path acquires a new locator in a later pass, and different non-UTF-8 names coexist.
/// Guarantees: Historical paths are not current evidence, and native names are never compared through lossy text.
#[test]
fn path_conflict_checks_use_fresh_native_evidence() {
    use std::os::unix::ffi::OsStrExt;
    let mut r = Reconciler::new(limits()).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(1, "/same"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    start(&mut r, 2);
    events.clear();
    feed(&mut r, &observation(2, "/same"), &mut events);
    for (inode, path) in [(3, b"/logs/\xff"), (4, b"/logs/\xfe")] {
        let mut seen = observation(inode, "/unused");
        seen.path = PathBuf::from(OsStr::from_bytes(path));
        feed(&mut r, &seen, &mut events);
    }
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(!events.contains(&"coverage".into()));
}

/// Scenario: Contradictory overflow paths are followed by enough better-ranked candidates to replace the sample.
/// Guarantees: Path-conflict failure survives sample eviction and cannot produce disappearance evidence.
#[test]
fn evicted_overflow_path_conflict_stays_incomplete() {
    let mut r = Reconciler::new(ReconcileLimits {
        pending: 1,
        offers: 1,
        ..limits()
    })
    .unwrap();
    r.track(file(2000, "/missing")).unwrap();
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(1, "/pending"), &mut events);
    feed(&mut r, &observation(2, "/same"), &mut events);
    feed(&mut r, &observation(3, "/same"), &mut events);
    assert_eq!(events, ["coverage"]);
    for inode in 4..=1024 {
        feed(
            &mut r,
            &observation(inode, &format!("/fresh/{inode}")),
            &mut events,
        );
    }
    assert!(!finish(&mut r, true, &mut events).complete);
    assert!(!events.contains(&"missing:2000".into()));
    assert!(r.offers().unwrap().next().unwrap().candidate.locator.inode > 3);
}

/// Scenario: Candidate locator/path indexes cross capacity boundaries and recycle slots across passes and acknowledgements.
/// Guarantees: Owned storage fits the existing candidate allowance through path-index insertion, removal and overflow churn.
#[test]
fn candidate_path_index_fits_admission_under_churn() {
    for count in [1, 7, 8, 111, 112, 113, 895, 896, 897] {
        let mut r = Reconciler::new(ReconcileLimits {
            tracked: 1,
            pending: count,
            offers: count,
            path_bytes: 32,
            fingerprint_bytes: 16,
        })
        .unwrap();
        let charge = 4096 + 32 + 16 + (3 * 32 + 512) + 2 * count * (2 * (32 + 16) + 512);
        let mut events = Vec::new();
        for generation in 1..=4 {
            start(&mut r, generation);
            for id in 1..=3 * count as u64 {
                let inode = generation * 10000 + id;
                feed(
                    &mut r,
                    &observation(inode, &format!("/p{inode:08}")),
                    &mut events,
                );
            }
            assert!(finish(&mut r, true, &mut events).complete);
            let tokens: Vec<_> = r.offers().unwrap().map(|o| o.token).collect();
            for token in tokens {
                r.acknowledge(
                    token,
                    IdentityFeedback::Rejected,
                    Duration::from_secs(generation * 10 + 1),
                )
                .unwrap();
            }
            assert!(
                r.retained_bytes() <= charge,
                "{count}: {} > {charge}",
                r.retained_bytes()
            );
        }
    }
}

/// Scenario: Two resident records are observed at the same new alias in either order.
/// Guarantees: Coverage loss precedes validation of the second locator, and neither stale binding nor disappearance can be published.
#[test]
fn resident_alias_conflicts_are_order_independent() {
    for reverse in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/resident1")).unwrap();
        r.track(file(2, "/resident2")).unwrap();
        r.track(file(99, "/missing")).unwrap();
        start(&mut r, 1);
        let mut events = Vec::new();
        let [first, second] = if reverse { [2, 1] } else { [1, 2] };
        feed(&mut r, &observation(first, "/shared"), &mut events);
        assert!(!events.contains(&"coverage".into()));
        events.clear();
        feed(&mut r, &observation(second, "/shared"), &mut events);
        assert_eq!(events.first().map(String::as_str), Some("coverage"));
        assert!(!r.pass.as_ref().unwrap().complete);
        assert!(!finish(&mut r, true, &mut events).complete);
        assert!(!events.contains(&"missing:99".into()));
        for id in [TrackedId(1), TrackedId(2)] {
            assert_eq!(
                r.rebind(1, id, |_| panic!("conflicting durable binding")),
                Err(ReconcileError::Feedback)
            );
        }
    }
}

/// Scenario: A selected resident alias changes, its owner is rebound and retired, and another pass reuses a prior alias.
/// Guarantees: The ID-only alias index follows minimum changes and dense-slot removal, and old-pass aliases cannot manufacture conflicts.
#[test]
fn resident_alias_index_follows_rebind_retirement_and_pass_reset() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/resident1")).unwrap();
    r.track(file(2, "/resident2")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(1, "/z"), &mut events);
    feed(&mut r, &observation(1, "/a"), &mut events);
    feed(&mut r, &observation(2, "/b"), &mut events);
    assert_eq!(r.aliases.get(OsStr::new("/z"), &r.tracked).count(), 0);
    assert_eq!(
        r.aliases
            .get(OsStr::new("/a"), &r.tracked)
            .collect::<Vec<_>>(),
        vec![TrackedId(1)]
    );
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(r.rebind(1, TrackedId(1), |_| true).unwrap());
    drop(r.untrack(TrackedId(1)).unwrap());
    assert_eq!(r.aliases.get(OsStr::new("/a"), &r.tracked).count(), 0);
    assert_eq!(
        r.aliases
            .get(OsStr::new("/b"), &r.tracked)
            .collect::<Vec<_>>(),
        vec![TrackedId(2)]
    );
    start(&mut r, 2);
    events.clear();
    feed(&mut r, &observation(3, "/b"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(!events.contains(&"coverage".into()));
}

/// Scenario: Root refresh fails after successful traversal, but no tracked file needs a targeted check.
/// Guarantees: Lost include roots, unverifiable exclusion aliases and cancellation invalidate completion; healthy or resolvably absent exclusion roots remain complete.
#[test]
fn empty_binding_batch_delivers_root_refresh_failures() {
    use super::super::{ProbeBudget, ScanConfig, ScanLimits, Scanner, SymlinkPolicy};
    for case in [
        "healthy",
        "missing",
        "replaced",
        "exclusion",
        "cancelled",
        "absent_exclusion",
    ] {
        let healthy = matches!(case, "healthy" | "absent_exclusion");
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("logs");
        let excluded = dir.path().join("private");
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(&excluded).unwrap();
        let mut scanner = Scanner::new(
            ScanConfig {
                include_roots: vec![root.clone()],
                exclusion_roots: vec![excluded.clone()],
                symlinks: SymlinkPolicy::Reject,
                checkpoint_namespace: locator(u64::MAX),
                limits: ScanLimits::default(),
            },
            Policy,
        )
        .unwrap();
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, root.join("missing.log").to_str().unwrap()))
            .unwrap();
        let budget = ProbeBudget::default();
        let mut events = Vec::new();
        start(&mut r, 1);
        let completion = scanner.scan(
            1,
            &budget,
            || false,
            |batch| {
                for observation in batch {
                    feed(&mut r, observation, &mut events);
                }
                true
            },
        );
        assert!(completion.complete);
        assert_eq!(r.next_binding_check(), None);
        match case {
            "missing" | "replaced" => {
                std::fs::rename(&root, dir.path().join("old")).unwrap();
                if case == "replaced" {
                    std::fs::create_dir(&root).unwrap();
                }
            }
            "exclusion" | "absent_exclusion" => {
                std::fs::remove_dir(&excluded).unwrap();
                if case == "exclusion" {
                    std::os::unix::fs::symlink(dir.path().join("unavailable"), &excluded).unwrap();
                }
            }
            _ => (),
        }
        match scanner.binding_batch(&budget, || case == "cancelled") {
            Ok(_) => assert!(healthy, "{case}"),
            Err(error) => {
                assert!(!healthy, "{case}");
                assert!(error.reason.invalidates_coverage());
                if case == "replaced" {
                    assert_eq!(error.reason, ScanReason::RootChanged);
                    assert_eq!(error.root, Some(root.as_path()));
                }
                r.apply_binding_batch_error(1, &error, |e| record(e, &mut events))
                    .unwrap();
                assert!(events.contains(&"coverage".into()));
                assert!(!r.pass.as_ref().unwrap().complete);
                assert!(!events.contains(&"parent:1".into()));
            }
        }
        let summary = r
            .finish(completion, Duration::from_secs(11), |e| {
                record(e, &mut events)
            })
            .unwrap();
        assert_eq!(summary.complete, healthy);
        assert_eq!(events.contains(&"missing:1".into()), healthy);
    }
}

/// Scenario: The final edge of a two-way swap or three-cycle arrives before scan completion, in either traversal order.
/// Guarantees: The newly closed cycle emits failure immediately for resident and nonresident records, without waiting for finish.
#[test]
fn binding_cycles_are_reported_when_the_last_edge_arrives() {
    for size in [2, 3] {
        for reverse in [false, true] {
            for resident in [false, true] {
                let mut r = Reconciler::new(limits()).unwrap();
                for id in 1..=size {
                    let mut tracked = file(id, &format!("/file{id}"));
                    tracked.resident = resident;
                    r.track(tracked).unwrap();
                }
                r.track(file(99, "/missing")).unwrap();
                start(&mut r, 1);
                let mut events = Vec::new();
                let mut ids: Vec<_> = (1..=size).collect();
                if reverse {
                    ids.reverse();
                }
                for (index, id) in ids.into_iter().enumerate() {
                    feed(
                        &mut r,
                        &observation(id % size + 1, &format!("/file{id}")),
                        &mut events,
                    );
                    assert_eq!(
                        events.contains(&"coverage".into()),
                        index + 1 == size as usize
                    );
                }
                assert!(!r.pass.as_ref().unwrap().complete);
                assert!(!finish(&mut r, true, &mut events).complete);
                assert!(!events.contains(&"missing:99".into()));
                assert_eq!(r.offers().unwrap().count(), 0);
            }
        }
    }
}

/// Scenario: Owner-supplied evidence closes a rebound cycle between two records with unavailable bindings.
/// Guarantees: Association uses the same prompt cycle check as traversal and immediately invalidates coverage.
#[test]
fn associated_rebound_cycle_is_reported_before_finish() {
    let mut r = Reconciler::new(limits()).unwrap();
    for id in [1, 2] {
        let mut tracked = file(id, "/unused");
        tracked.binding = None;
        tracked.resident = false;
        r.track(tracked).unwrap();
    }
    start(&mut r, 1);
    let mut events = Vec::new();
    feed(&mut r, &observation(1, "/a"), &mut events);
    feed(&mut r, &observation(2, "/b"), &mut events);
    events.clear();
    r.associate_replacement(1, TrackedId(1), locator(2), |e| record(e, &mut events))
        .unwrap();
    assert!(events.is_empty());
    r.associate_replacement(1, TrackedId(2), locator(1), |e| record(e, &mut events))
        .unwrap();
    assert_eq!(events, ["coverage"]);
    assert!(!r.pass.as_ref().unwrap().complete);
}

/// Scenario: A reverse-ordered acyclic chain grows across many components, then one final edge closes it.
/// Guarantees: Connectivity bookkeeping never flags the open chain, reports closure immediately, and resets for a fresh pass.
#[test]
fn growing_rebound_chain_reports_only_cycle_closure() {
    const COUNT: u64 = 128;
    let mut r = Reconciler::new(ReconcileLimits {
        tracked: COUNT as usize,
        ..limits()
    })
    .unwrap();
    for id in 0..COUNT {
        r.track(file(id, &format!("/file{id}"))).unwrap();
    }
    let mut events = Vec::new();
    for generation in [1, 2] {
        start(&mut r, generation);
        events.clear();
        for id in (0..COUNT - 1).rev() {
            feed(
                &mut r,
                &observation(id + 1, &format!("/file{id}")),
                &mut events,
            );
        }
        assert!(!events.contains(&"coverage".into()));
        feed(
            &mut r,
            &observation(0, &format!("/file{}", COUNT - 1)),
            &mut events,
        );
        assert!(events.contains(&"coverage".into()));
        assert!(!r.pass.as_ref().unwrap().complete);
        assert!(!finish(&mut r, true, &mut events).complete);
    }
}

/// Scenario: A retained replacement is followed by explicit instability, with or without locator evidence.
/// Guarantees: Affected admissions are withheld, predecessors stay Live, and unrelated candidates remain usable.
#[test]
fn explicit_instability_revokes_retained_replacement_evidence() {
    for failed_locator in [None, Some(2)] {
        for targeted in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/a")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(&mut r, &observation(2, "/a"), &mut events);
            if targeted {
                // The rebound must also revoke a candidate whose selected path moved.
                feed(&mut r, &observation(2, "/0"), &mut events);
            }
            feed(&mut r, &observation(3, "/unrelated"), &mut events);
            let failure = evidence(failed_locator, "/a", ScanReason::Unstable);
            events.clear();
            if targeted {
                apply_test_check(&mut r, 1, &failure, None, false, &mut events);
            } else {
                feed(&mut r, &failure, &mut events);
            }
            assert_eq!(events.first().map(String::as_str), Some("coverage"));
            assert!(r.tracked[&TrackedId(1)].pass.conflict);
            assert!(!finish(&mut r, true, &mut events).complete);
            let offered: Vec<_> = r
                .offers()
                .unwrap()
                .map(|offer| offer.candidate.locator)
                .collect();
            assert_eq!(offered, [locator(3)]);
            assert_eq!(r.tracked(TrackedId(1)).unwrap().role, BindingRole::Live);
        }
    }
}

fn apply_test_check(
    r: &mut Reconciler,
    id: u64,
    observation: &Observation,
    parent: Option<FileLocator>,
    context_valid: bool,
    events: &mut Vec<String>,
) {
    let pass = r.pass.as_ref().unwrap();
    r.apply_binding_check(
        pass.generation,
        TrackedId(id),
        super::super::BindingCheck {
            observation,
            parent,
            context_valid,
            scope: super::super::BindingScope::Binding,
            diagnostic_path: &observation.path,
        },
        pass.observed_at,
        |e| record(e, events),
    )
    .unwrap();
}

/// Scenario: A persistent parent change, root change, or failed batch precedes a contextual binding miss.
/// Guarantees: Traversal and targeted misses cannot revoke another reader during an environment block or successful retry.
#[test]
fn environment_failure_blocks_contextual_misses() {
    for failure in 0..3 {
        for targeted in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/logs/a")).unwrap();
            r.track(file(2, "/other/b")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(
                &mut r,
                &evidence(Some(20), "/other", ScanReason::Directory),
                &mut events,
            );
            if failure == 0 {
                feed(
                    &mut r,
                    &evidence(Some(21), "/other", ScanReason::Directory),
                    &mut events,
                );
            }
            assert!(!finish(&mut r, false, &mut events).complete);
            start(&mut r, 2);
            feed(
                &mut r,
                &evidence(Some(10), "/logs", ScanReason::Directory),
                &mut events,
            );
            feed(
                &mut r,
                &evidence(Some(1), "/logs/hidden", ScanReason::Excluded),
                &mut events,
            );
            match failure {
                0 => {}
                1 => feed(
                    &mut r,
                    &evidence(None, "/", ScanReason::RootChanged),
                    &mut events,
                ),
                _ => r
                    .apply_binding_batch_error(
                        2,
                        &super::super::BindingBatchError {
                            reason: ScanReason::RootUnavailable,
                            errno: None,
                            root: Some(Path::new("/")),
                        },
                        |e| record(e, &mut events),
                    )
                    .unwrap(),
            }
            let missing = evidence(None, "/logs/a", ScanReason::Missing);
            if targeted {
                apply_test_check(&mut r, 1, &missing, Some(locator(10)), true, &mut events);
            } else {
                feed(&mut r, &missing, &mut events);
            }
            assert!(
                !events.contains(&"excluded:1".into()),
                "failure={failure}, targeted={targeted}"
            );
            assert!(!finish(&mut r, true, &mut events).complete);
        }
    }
}

/// Scenario: A smaller alias is published, then disappears while an excluded name and unrelated coverage failure are observed.
/// Guarantees: Repeat publication stays nonempty and idempotent; the next pass retains parent context for prompt exclusion.
#[test]
fn rebind_preserves_parent_context_without_duplicate_alias_hint() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/logs/z")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    feed(&mut r, &observation(1, "/logs/a"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(r.rebind(1, TrackedId(1), |_| true).unwrap());
    assert_eq!(
        r.binding_proposal(1, TrackedId(1)).unwrap(),
        Path::new("/logs/a")
    );
    let mut publications = 0;
    assert!(
        r.rebind(1, TrackedId(1), |path| {
            assert_eq!(path, Path::new("/logs/a"));
            publications += 1;
            true
        })
        .unwrap()
    );
    assert_eq!(publications, 1);
    start(&mut r, 2);
    feed(
        &mut r,
        &evidence(Some(1), "/logs/hidden", ScanReason::Excluded),
        &mut events,
    );
    feed(
        &mut r,
        &evidence(None, "/unrelated", ScanReason::Coverage),
        &mut events,
    );
    let request = r.binding_request(TrackedId(1)).unwrap();
    assert_eq!(request.path, Path::new("/logs/a"));
    assert_eq!(request.parent, Some(locator(10)));
    assert!(
        r.tracked[&TrackedId(1)]
            .previous_alias
            .as_os_str()
            .is_empty()
    );
    let missing = evidence(None, "/logs/a", ScanReason::Missing);
    apply_test_check(&mut r, 1, &missing, Some(locator(10)), true, &mut events);
    assert!(events.contains(&"excluded:1".into()));
    assert!(!finish(&mut r, false, &mut events).complete);
}

/// Scenario: A new tracked binding shares an established parent, whose previous owner is retired.
/// Guarantees: The known baseline survives ownership changes and detects directory replacement on the next pass.
#[test]
fn new_binding_inherits_known_parent_baseline() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/logs/a")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    assert!(finish(&mut r, true, &mut events).complete);
    r.track(file(2, "/logs/b")).unwrap();
    drop(r.untrack(TrackedId(1)).unwrap());
    assert_eq!(r.tracked[&TrackedId(2)].parent, Some(locator(10)));
    start(&mut r, 2);
    feed(
        &mut r,
        &evidence(Some(11), "/logs", ScanReason::Directory),
        &mut events,
    );
    assert!(events.contains(&"parent:2".into()));
    assert!(!finish(&mut r, true, &mut events).complete);
}

/// Scenario: Historical and Live owners share replacement paths visited in either order.
/// Guarantees: Start-policy hints choose the smallest runtime ID independently of hash order and Live ownership.
#[test]
fn replacement_hints_use_one_deterministic_rule() {
    for reverse in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        for id in [4, 2, 6] {
            let mut owner = file(id, "/a");
            if id != 6 {
                owner.role = BindingRole::Superseded;
            }
            r.track(owner).unwrap();
        }
        r.track(file(1, "/b")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        let paths = if reverse { ["/b", "/a"] } else { ["/a", "/b"] };
        for path in paths {
            feed(&mut r, &observation(8, path), &mut events);
        }
        assert!(!finish(&mut r, false, &mut events).complete);
        assert_eq!(
            r.offers().unwrap().next().unwrap().candidate.replacement,
            Replacement::Possible(TrackedId(1))
        );
    }
}

/// Scenario: Jitter passes conversion and width validation but the final sampled sum exceeds u64 nanoseconds.
/// Guarantees: Overflow returns Delay while the adjacent representable endpoint succeeds.
#[test]
fn jitter_final_sum_is_checked() {
    let base = u64::MAX;
    let spread = base / 100;
    assert_eq!(
        reconcile_delay(Duration::from_nanos(base), 1, 2 * spread),
        Err(ReconcileError::Delay)
    );
    assert_eq!(
        reconcile_delay(Duration::from_nanos(base), 1, spread),
        Ok(Duration::from_nanos(base))
    );
}

/// Scenario: Identity-owner associations arrive in either order for one fresh candidate.
/// Guarantees: Both orders retain the smallest predecessor ID without changing supersession ownership.
#[test]
fn owner_associations_merge_replacement_hints() {
    for ids in [[1, 2], [2, 1]] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/one")).unwrap();
        r.track(file(2, "/two")).unwrap();
        start(&mut r, 1);
        let mut events = Vec::new();
        feed(&mut r, &observation(3, "/candidate"), &mut events);
        for id in ids {
            r.associate_replacement(1, TrackedId(id), locator(3), |e| record(e, &mut events))
                .unwrap();
        }
        assert!(!finish(&mut r, false, &mut events).complete);
        assert_eq!(
            r.offers().unwrap().next().unwrap().candidate.replacement,
            Replacement::Possible(TrackedId(1))
        );
        assert_eq!(r.tracked(TrackedId(1)).unwrap().role, BindingRole::Live);
        assert_eq!(r.tracked(TrackedId(2)).unwrap().role, BindingRole::Live);
    }
}

/// Scenario: A replacement appears beneath a changed binding parent while another candidate remains stable.
/// Guarantees: No affected offer or explicit supersession is allowed until the expected parent returns; unrelated work remains usable.
#[test]
fn changed_parent_blocks_replacement_publication_until_recovery() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/logs/app")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    feed(&mut r, &observation(1, "/logs/app"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    start(&mut r, 2);
    feed(
        &mut r,
        &evidence(Some(11), "/logs", ScanReason::Directory),
        &mut events,
    );
    feed(&mut r, &observation(2, "/logs/app"), &mut events);
    feed(&mut r, &observation(3, "/other/file"), &mut events);
    assert!(!finish(&mut r, true, &mut events).complete);
    assert_eq!(
        r.offers()
            .unwrap()
            .map(|o| o.candidate.locator)
            .collect::<Vec<_>>(),
        [locator(3)]
    );
    assert_eq!(
        r.supersede(2, TrackedId(1), locator(2)),
        Err(ReconcileError::Feedback)
    );
    assert_eq!(r.tracked(TrackedId(1)).unwrap().role, BindingRole::Live);
    start(&mut r, 3);
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    feed(&mut r, &observation(2, "/logs/app"), &mut events);
    assert!(!finish(&mut r, false, &mut events).complete);
    let token = r.offers().unwrap().next().unwrap().token;
    assert!(
        r.admit(token, file(2, "/logs/app"), |_| Some(Duration::from_secs(
            32
        )))
        .unwrap()
    );
    assert_eq!(
        r.tracked(TrackedId(1)).unwrap().role,
        BindingRole::Superseded
    );
}

/// Scenario: A neighbour's expected parent is established before a second record sees the replacement directory.
/// Guarantees: Traversal and targeted checks preserve the old expected identity, and later tracking inherits that latched baseline deterministically.
#[test]
fn parent_baselines_do_not_adopt_a_neighbours_replacement() {
    for targeted in [false, true] {
        for ids in [[1, 2], [2, 1]] {
            let mut r = Reconciler::new(limits()).unwrap();
            for id in ids {
                r.track(file(id, &format!("/logs/{id}"))).unwrap();
            }
            let mut events = Vec::new();
            start(&mut r, 1);
            // A targeted check establishes the expected directory identity.
            apply_test_check(
                &mut r,
                1,
                &observation(1, "/logs/1"),
                Some(locator(10)),
                true,
                &mut events,
            );
            if targeted {
                apply_test_check(
                    &mut r,
                    2,
                    &observation(2, "/logs/2"),
                    Some(locator(11)),
                    true,
                    &mut events,
                );
            } else {
                feed(
                    &mut r,
                    &evidence(Some(11), "/logs", ScanReason::Directory),
                    &mut events,
                );
            }
            assert_eq!(r.tracked[&TrackedId(2)].parent, Some(locator(10)));
            assert!(r.tracked[&TrackedId(2)].parent_changed);
            assert!(!finish(&mut r, true, &mut events).complete);
            r.track(file(3, "/logs/3")).unwrap();
            assert_eq!(r.tracked[&TrackedId(3)].parent, Some(locator(10)));
            assert!(r.tracked[&TrackedId(3)].parent_changed);
        }
    }
}

/// Scenario: Unrelated policy or root failures precede fresh positive exclusion evidence.
/// Guarantees: A positively excluded or rebound binding can revoke independently, while pass completeness remains false.
#[test]
fn unrelated_failures_do_not_block_positive_exclusion() {
    for reason in [
        ScanReason::Policy,
        ScanReason::VerificationUnavailable,
        ScanReason::RootUnavailable,
    ] {
        for replacement in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/logs/app")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(&mut r, &evidence(None, "/other", reason), &mut events);
            feed(
                &mut r,
                &evidence(Some(1), "/hidden", ScanReason::Excluded),
                &mut events,
            );
            if replacement {
                feed(&mut r, &observation(2, "/logs/app"), &mut events);
            } else {
                feed(
                    &mut r,
                    &evidence(Some(1), "/logs/app", ScanReason::Excluded),
                    &mut events,
                );
            }
            assert!(events.contains(&"excluded:1".into()));
            assert!(!finish(&mut r, true, &mut events).complete);
        }
    }
}

/// Scenario: A contextual miss is retained before a root failure and a later excluded-alias observation.
/// Guarantees: Old missing-path evidence cannot bypass the environment gate, but a fresh positive exclusion can.
#[test]
fn environment_failure_invalidates_earlier_miss_authority() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/logs/app")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    feed(
        &mut r,
        &evidence(None, "/logs/app", ScanReason::Missing),
        &mut events,
    );
    feed(
        &mut r,
        &evidence(None, "/other", ScanReason::RootUnavailable),
        &mut events,
    );
    feed(
        &mut r,
        &evidence(Some(1), "/hidden", ScanReason::Excluded),
        &mut events,
    );
    assert!(!events.contains(&"excluded:1".into()));
    feed(
        &mut r,
        &evidence(Some(1), "/logs/app", ScanReason::Excluded),
        &mut events,
    );
    assert!(events.contains(&"excluded:1".into()));
    assert!(!finish(&mut r, true, &mut events).complete);
}

/// Scenario: A policy failure follows qualified evidence at one binding, while another binding is positively excluded.
/// Guarantees: Dependent missing/replacement authority is revoked without withholding the unrelated exclusion.
#[test]
fn policy_failure_invalidates_only_dependent_binding_evidence() {
    for reason in [ScanReason::Policy, ScanReason::VerificationUnavailable] {
        for replacement in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/logs/app")).unwrap();
            r.track(file(2, "/other/app")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(
                &mut r,
                &evidence(Some(10), "/logs", ScanReason::Directory),
                &mut events,
            );
            if replacement {
                feed(&mut r, &observation(3, "/logs/app"), &mut events);
            } else {
                feed(
                    &mut r,
                    &evidence(None, "/logs/app", ScanReason::Missing),
                    &mut events,
                );
            }
            feed(&mut r, &evidence(None, "/logs/app", reason), &mut events);
            feed(
                &mut r,
                &evidence(Some(1), "/hidden", ScanReason::Excluded),
                &mut events,
            );
            feed(
                &mut r,
                &evidence(Some(2), "/other/app", ScanReason::Excluded),
                &mut events,
            );
            assert!(!events.contains(&"excluded:1".into()));
            assert!(events.contains(&"excluded:2".into()));
            assert!(!finish(&mut r, true, &mut events).complete);
            assert!(r.offers().unwrap().next().is_none());
        }
    }
}

/// Scenario: A nonresident file is found through an eligible alias after its distinguished parent is replaced.
/// Guarantees: Exact-locator reconnection remains offered and admissible without changing the binding or clearing the parent latch.
#[test]
fn changed_parent_does_not_block_reconnection_through_another_alias() {
    let mut r = Reconciler::new(limits()).unwrap();
    let mut tracked = file(1, "/logs/app");
    tracked.resident = false;
    r.track(tracked).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    assert!(finish(&mut r, true, &mut events).complete);
    for generation in [2, 3] {
        start(&mut r, generation);
        if generation == 2 {
            feed(
                &mut r,
                &evidence(Some(11), "/logs", ScanReason::Directory),
                &mut events,
            );
        }
        feed(&mut r, &observation(1, "/logs.old/app"), &mut events);
        feed(&mut r, &observation(2, "/logs/app"), &mut events);
        assert!(!finish(&mut r, true, &mut events).complete);
        let offers: Vec<_> = r
            .offers()
            .unwrap()
            .map(|o| (o.token, o.candidate.locator))
            .collect();
        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].1, locator(1));
        assert!(
            r.admit(offers[0].0, file(1, "/logs/app"), |_| Some(
                Duration::from_secs(generation * 10 + 2)
            ))
            .unwrap()
        );
        assert_eq!(
            r.tracked(TrackedId(1)).unwrap().binding.as_deref(),
            Some(Path::new("/logs/app"))
        );
        assert!(r.tracked[&TrackedId(1)].parent_changed);
        r.update_reader_state(generation, TrackedId(1), true, false)
            .unwrap();
    }
}

/// Scenario: Replacement evidence is collected under a changed parent, then the expected parent returns in the same pass.
/// Guarantees: Neither traversal nor targeted restoration revives stale exclusion or replacement authority; a later fresh pass recovers.
#[test]
fn same_pass_parent_recovery_does_not_revive_binding_evidence() {
    for targeted in [false, true] {
        for inherited in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/logs/app")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(
                &mut r,
                &evidence(Some(10), "/logs", ScanReason::Directory),
                &mut events,
            );
            if inherited {
                feed(
                    &mut r,
                    &evidence(Some(11), "/logs", ScanReason::Directory),
                    &mut events,
                );
            }
            let _ = finish(&mut r, !inherited, &mut events);
            start(&mut r, 2);
            if !inherited {
                feed(
                    &mut r,
                    &evidence(Some(11), "/logs", ScanReason::Directory),
                    &mut events,
                );
            }
            feed(&mut r, &observation(2, "/logs/app"), &mut events);
            if targeted {
                apply_test_check(
                    &mut r,
                    1,
                    &evidence(None, "/logs/app", ScanReason::Missing),
                    Some(locator(10)),
                    true,
                    &mut events,
                );
            } else {
                feed(
                    &mut r,
                    &evidence(Some(10), "/logs", ScanReason::Directory),
                    &mut events,
                );
            }
            assert!(!r.tracked[&TrackedId(1)].parent_changed);
            feed(
                &mut r,
                &evidence(Some(1), "/hidden", ScanReason::Excluded),
                &mut events,
            );
            feed(&mut r, &observation(3, "/independent"), &mut events);
            assert!(!events.contains(&"excluded:1".into()));
            assert!(!finish(&mut r, true, &mut events).complete);
            assert_eq!(
                r.offers()
                    .unwrap()
                    .map(|o| o.candidate.locator)
                    .collect::<Vec<_>>(),
                [locator(3)]
            );
            assert_eq!(
                r.supersede(2, TrackedId(1), locator(2)),
                Err(ReconcileError::Feedback)
            );
            start(&mut r, 3);
            feed(
                &mut r,
                &evidence(Some(10), "/logs", ScanReason::Directory),
                &mut events,
            );
            feed(&mut r, &observation(2, "/logs/app"), &mut events);
            assert!(!finish(&mut r, false, &mut events).complete);
            let token = r.offers().unwrap().next().unwrap().token;
            assert!(
                r.admit(token, file(2, "/logs/app"), |_| Some(Duration::from_secs(
                    32
                )))
                .unwrap()
            );
        }
    }
}

/// Scenario: The owner associates replacement evidence while a carried-over parent change is latched, before same-pass restoration.
/// Guarantees: The association cannot authorize supersession after restoration, but its independently validated candidate remains available.
#[test]
fn parent_recovery_does_not_revive_owner_associated_binding_work() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/logs/app")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    feed(
        &mut r,
        &evidence(Some(11), "/logs", ScanReason::Directory),
        &mut events,
    );
    assert!(!finish(&mut r, true, &mut events).complete);
    start(&mut r, 2);
    feed(&mut r, &observation(2, "/independent"), &mut events);
    r.associate_replacement(2, TrackedId(1), locator(2), |e| record(e, &mut events))
        .unwrap();
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    assert!(!finish(&mut r, true, &mut events).complete);
    assert_eq!(
        r.supersede(2, TrackedId(1), locator(2)),
        Err(ReconcileError::Feedback)
    );
    assert_eq!(
        r.offers()
            .unwrap()
            .map(|o| o.candidate.locator)
            .collect::<Vec<_>>(),
        [locator(2)]
    );
}

/// Scenario: A nonresident record's own locator is validated at its blocked binding, alone or with smaller/larger aliases in either order.
/// Guarantees: Reconnection preserves the binding and parent block regardless of selected-path order; new identity and binding-change attempts fail before their callbacks.
#[test]
fn blocked_binding_allows_own_locator_reconnection_in_all_path_orders() {
    for paths in [
        &["/logs/app"][..],
        &["/logs/app", "/zz/app"][..],
        &["/zz/app", "/logs/app"][..],
        &["/logs/app", "/logs.old/app"][..],
        &["/logs.old/app", "/logs/app"][..],
    ] {
        let mut r = Reconciler::new(limits()).unwrap();
        let mut tracked = file(1, "/logs/app");
        tracked.resident = false;
        r.track(tracked).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(
            &mut r,
            &evidence(Some(10), "/logs", ScanReason::Directory),
            &mut events,
        );
        assert!(finish(&mut r, true, &mut events).complete);
        for generation in [2, 3] {
            start(&mut r, generation);
            if generation == 2 {
                feed(
                    &mut r,
                    &evidence(Some(11), "/logs", ScanReason::Directory),
                    &mut events,
                );
            }
            for path in paths {
                feed(&mut r, &observation(1, path), &mut events);
            }
            assert!(!finish(&mut r, true, &mut events).complete);
            let token = {
                let mut offers = r.offers().unwrap();
                let offer = offers.next().expect("own locator remains reconnectable");
                assert_eq!(offer.candidate.locator, locator(1));
                assert_eq!(offer.candidate.path, Path::new(paths.iter().min().unwrap()));
                let token = offer.token;
                assert!(offers.next().is_none());
                token
            };
            assert!(
                r.admit(token, file(1, "/changed"), |_| panic!(
                    "binding change reached durable owner"
                ))
                .is_err()
            );
            let mut new_identity = file(2, paths.iter().min().unwrap());
            new_identity.locator = locator(1);
            assert_eq!(
                r.admit(token, new_identity, |_| panic!(
                    "new identity reached durable owner"
                )),
                Err(ReconcileError::Feedback)
            );
            assert!(
                r.admit(token, file(1, "/logs/app"), |_| Some(Duration::from_secs(
                    generation * 10 + 2
                )))
                .unwrap()
            );
            let retained = &r.tracked[&TrackedId(1)];
            assert_eq!(
                retained.file.binding.as_deref(),
                Some(Path::new("/logs/app"))
            );
            assert_eq!(retained.file.role, BindingRole::Live);
            assert!(retained.file.resident);
            assert!(retained.parent_changed);
            assert!(retained.binding_blocked());
            r.update_reader_state(generation, TrackedId(1), true, false)
                .unwrap();
        }
    }
}

/// Scenario: A blocked Live, Superseded or Unverified binding exposes an offer for its own locator.
/// Guarantees: A new runtime identity is rejected before durability regardless of binding role, while same-ID reconnection preserves the role and block.
#[test]
fn blocked_historical_bindings_allow_only_reconnection() {
    for role in [
        BindingRole::Live,
        BindingRole::Superseded,
        BindingRole::Unverified,
    ] {
        let mut r = Reconciler::new(limits()).unwrap();
        let mut tracked = file(1, "/logs/app");
        tracked.role = role;
        tracked.resident = false;
        r.track(tracked).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(
            &mut r,
            &evidence(Some(10), "/logs", ScanReason::Directory),
            &mut events,
        );
        assert!(finish(&mut r, true, &mut events).complete);
        start(&mut r, 2);
        feed(
            &mut r,
            &evidence(Some(11), "/logs", ScanReason::Directory),
            &mut events,
        );
        feed(&mut r, &observation(1, "/logs/app"), &mut events);
        assert!(!finish(&mut r, true, &mut events).complete);
        let token = r
            .offers()
            .unwrap()
            .next()
            .expect("reconnection offer")
            .token;
        let mut new_identity = file(2, "/logs/app");
        new_identity.locator = locator(1);
        assert_eq!(
            r.admit(token, new_identity, |_| panic!(
                "new identity reached durable owner"
            )),
            Err(ReconcileError::Feedback)
        );
        let mut reconnect = file(1, "/logs/app");
        reconnect.role = role;
        assert!(
            r.admit(token, reconnect, |_| Some(Duration::from_secs(22)))
                .unwrap()
        );
        let retained = &r.tracked[&TrackedId(1)];
        assert_eq!(retained.file.role, role);
        assert_eq!(
            retained.file.binding.as_deref(),
            Some(Path::new("/logs/app"))
        );
        assert!(retained.file.resident);
        assert!(retained.binding_blocked());
    }
}

/// Scenario: An active Superseded predecessor shares a blocked path with a nonresident Live successor after rotation.
/// Guarantees: The predecessor cannot veto unchanged successor reconnection, but new identity publication stays blocked before durability.
#[test]
fn blocked_historical_predecessor_does_not_veto_live_reconnection() {
    for ids in [[1, 2], [2, 1]] {
        let mut r = Reconciler::new(limits()).unwrap();
        for id in ids {
            let mut tracked = file(id, "/logs/app");
            if id == 1 {
                tracked.role = BindingRole::Superseded;
            } else {
                tracked.resident = false;
            }
            r.track(tracked).unwrap();
        }
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(
            &mut r,
            &evidence(Some(10), "/logs", ScanReason::Directory),
            &mut events,
        );
        assert!(finish(&mut r, true, &mut events).complete);
        start(&mut r, 2);
        feed(
            &mut r,
            &evidence(Some(11), "/logs", ScanReason::Directory),
            &mut events,
        );
        feed(&mut r, &observation(2, "/logs/app"), &mut events);
        assert!(!finish(&mut r, true, &mut events).complete);
        let offers: Vec<_> = r
            .offers()
            .unwrap()
            .map(|o| (o.token, o.candidate.locator))
            .collect();
        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].1, locator(2));
        let mut new_identity = file(3, "/logs/app");
        new_identity.locator = locator(2);
        assert_eq!(
            r.admit(offers[0].0, new_identity, |_| panic!(
                "new identity reached durable owner"
            )),
            Err(ReconcileError::Feedback)
        );
        assert!(
            r.admit(offers[0].0, file(2, "/logs/app"), |_| Some(
                Duration::from_secs(22)
            ))
            .unwrap()
        );
        assert_eq!(r.tracked[&TrackedId(1)].file.role, BindingRole::Superseded);
        assert_eq!(r.tracked[&TrackedId(2)].file.role, BindingRole::Live);
        assert!(r.tracked[&TrackedId(2)].file.resident);
        for id in [1, 2] {
            assert!(r.tracked[&TrackedId(id)].binding_blocked());
            assert_eq!(
                r.tracked[&TrackedId(id)].file.binding.as_deref(),
                Some(Path::new("/logs/app"))
            );
        }
    }
}

/// Scenario: Recovered or superseded records reconnect with eligible binding evidence under persistent or same-pass parent blocks.
/// Guarantees: Promotion to Live is refused before durability, same-role reconnection remains possible, and a fresh recovered pass permits promotion.
#[test]
fn blocked_reconnection_cannot_promote_a_binding_to_live() {
    for role in [BindingRole::Unverified, BindingRole::Superseded] {
        for restore_in_pass in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            let mut tracked = file(1, "/logs/app");
            tracked.role = role;
            tracked.resident = false;
            r.track(tracked).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(
                &mut r,
                &evidence(Some(10), "/logs", ScanReason::Directory),
                &mut events,
            );
            assert!(finish(&mut r, true, &mut events).complete);
            start(&mut r, 2);
            feed(
                &mut r,
                &evidence(Some(11), "/logs", ScanReason::Directory),
                &mut events,
            );
            feed(&mut r, &observation(1, "/logs/app"), &mut events);
            if restore_in_pass {
                feed(
                    &mut r,
                    &evidence(Some(10), "/logs", ScanReason::Directory),
                    &mut events,
                );
            }
            assert!(!finish(&mut r, true, &mut events).complete);
            let token = r
                .offers()
                .unwrap()
                .next()
                .expect("same-role reconnection offer")
                .token;
            assert_eq!(
                r.admit(token, file(1, "/logs/app"), |_| panic!(
                    "blocked promotion reached durable owner"
                )),
                Err(ReconcileError::Tracked)
            );
            assert_eq!(r.tracked[&TrackedId(1)].file.role, role);
            let mut reconnect = file(1, "/logs/app");
            reconnect.role = role;
            assert!(
                r.admit(token, reconnect, |_| Some(Duration::from_secs(22)))
                    .unwrap()
            );
            assert_eq!(r.tracked[&TrackedId(1)].file.role, role);
            assert!(r.tracked[&TrackedId(1)].binding_blocked());
            assert_eq!(r.tracked[&TrackedId(1)].parent_changed, !restore_in_pass);
            r.update_reader_state(2, TrackedId(1), true, false).unwrap();
            start(&mut r, 3);
            feed(
                &mut r,
                &evidence(Some(10), "/logs", ScanReason::Directory),
                &mut events,
            );
            feed(&mut r, &observation(1, "/logs/app"), &mut events);
            assert!(finish(&mut r, true, &mut events).complete);
            let token = r
                .offers()
                .unwrap()
                .next()
                .expect("fresh eligible offer")
                .token;
            if role == BindingRole::Unverified {
                assert!(
                    r.admit(token, file(1, "/logs/app"), |_| Some(Duration::from_secs(
                        32
                    )))
                    .unwrap()
                );
            } else {
                let mut reconnect = file(1, "/logs/app");
                reconnect.role = role;
                assert!(
                    r.admit(token, reconnect, |_| Some(Duration::from_secs(32)))
                        .unwrap()
                );
                assert!(r.rebind(3, TrackedId(1), |_| true).unwrap());
            }
            assert_eq!(r.tracked[&TrackedId(1)].file.role, BindingRole::Live);
            assert!(!r.tracked[&TrackedId(1)].binding_blocked());
            assert_eq!(
                r.tracked[&TrackedId(1)].file.binding.as_deref(),
                Some(Path::new("/logs/app"))
            );
        }
    }
}

fn changed_parent_evidence(r: &mut Reconciler, targeted: bool, events: &mut Vec<String>) {
    if targeted {
        let failure = evidence(None, "/logs/a", ScanReason::RootChanged);
        let pass = r.pass.as_ref().unwrap();
        r.apply_binding_check(
            pass.generation,
            TrackedId(1),
            super::super::BindingCheck {
                observation: &failure,
                parent: Some(locator(11)),
                context_valid: false,
                scope: super::super::BindingScope::ParentChanged,
                diagnostic_path: Path::new("/logs"),
            },
            pass.observed_at,
            |e| record(e, events),
        )
        .unwrap();
    } else {
        feed(
            r,
            &evidence(Some(11), "/logs", ScanReason::Directory),
            events,
        );
    }
}

/// Scenario: Traversal or a targeted check detects a replaced parent before or after sibling replacement evidence.
/// Guarantees: Both routes preserve sibling baselines, withhold affected replacement work, and allow independent admission.
#[test]
fn parent_evidence_routes_apply_the_same_sibling_publication_block() {
    for targeted in [false, true] {
        for change_first in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/logs/a")).unwrap();
            r.track(file(2, "/logs/b")).unwrap();
            let mut inactive = file(5, "/logs/retired");
            inactive.active = false;
            r.track(inactive).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(
                &mut r,
                &evidence(Some(10), "/logs", ScanReason::Directory),
                &mut events,
            );
            assert!(finish(&mut r, true, &mut events).complete);
            start(&mut r, 2);
            if change_first {
                changed_parent_evidence(&mut r, targeted, &mut events);
            }
            feed(&mut r, &observation(3, "/logs/b"), &mut events);
            if !change_first {
                changed_parent_evidence(&mut r, targeted, &mut events);
            }
            for id in [1, 2, 5] {
                assert_eq!(r.tracked[&TrackedId(id)].parent, Some(locator(10)));
                assert!(r.tracked[&TrackedId(id)].parent_changed);
                assert!(r.tracked[&TrackedId(id)].pass.parent_unstable);
                assert!(events.contains(&format!("parent:{id}")));
            }
            // A repeated targeted result observes the already-propagated state.
            if targeted {
                changed_parent_evidence(&mut r, true, &mut events);
            }
            feed(&mut r, &observation(4, "/independent"), &mut events);
            assert!(!finish(&mut r, true, &mut events).complete);
            assert_eq!(
                r.supersede(2, TrackedId(2), locator(3)),
                Err(ReconcileError::Feedback)
            );
            let offers: Vec<_> = r
                .offers()
                .unwrap()
                .map(|o| (o.token, o.candidate.locator))
                .collect();
            assert_eq!(offers.len(), 1);
            assert_eq!(offers[0].1, locator(4));
            assert!(
                r.admit(offers[0].0, file(4, "/independent"), |_| Some(
                    Duration::from_secs(22)
                ))
                .unwrap()
            );
            assert_eq!(r.tracked[&TrackedId(2)].file.role, BindingRole::Live);
        }
    }
}

/// Scenario: A targeted check discovers parent replacement after a nonresident sibling's eligible evidence, optionally followed by restoration.
/// Guarantees: Sibling promotion and new-ID callbacks are withheld, but unchanged-role reconnection and unrelated admission remain usable.
#[test]
fn targeted_parent_changes_guard_sibling_admission_before_durability() {
    for role in [BindingRole::Unverified, BindingRole::Superseded] {
        for restore in [0, 1, 2] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/logs/a")).unwrap();
            let mut sibling = file(2, "/logs/b");
            sibling.role = role;
            sibling.resident = false;
            r.track(sibling).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(
                &mut r,
                &evidence(Some(10), "/logs", ScanReason::Directory),
                &mut events,
            );
            assert!(finish(&mut r, true, &mut events).complete);
            start(&mut r, 2);
            feed(&mut r, &observation(2, "/logs/b"), &mut events);
            changed_parent_evidence(&mut r, true, &mut events);
            if restore == 1 {
                feed(
                    &mut r,
                    &evidence(Some(10), "/logs", ScanReason::Directory),
                    &mut events,
                );
            } else if restore == 2 {
                apply_test_check(
                    &mut r,
                    1,
                    &evidence(None, "/logs/a", ScanReason::Missing),
                    Some(locator(10)),
                    true,
                    &mut events,
                );
            }
            feed(&mut r, &observation(4, "/independent"), &mut events);
            assert!(!finish(&mut r, true, &mut events).complete);
            assert!(r.tracked[&TrackedId(2)].pass.parent_unstable);
            assert_eq!(r.tracked[&TrackedId(2)].parent_changed, restore == 0);
            let offers: Vec<_> = r
                .offers()
                .unwrap()
                .map(|o| (o.token, o.candidate.locator))
                .collect();
            let token = offers.iter().find(|(_, loc)| *loc == locator(2)).unwrap().0;
            assert_eq!(
                r.admit(token, file(2, "/logs/b"), |_| panic!(
                    "sibling promotion reached durability"
                )),
                Err(ReconcileError::Tracked)
            );
            let mut new_identity = file(3, "/logs/b");
            new_identity.locator = locator(2);
            assert_eq!(
                r.admit(token, new_identity, |_| panic!(
                    "sibling new identity reached durability"
                )),
                Err(ReconcileError::Feedback)
            );
            let mut reconnect = file(2, "/logs/b");
            reconnect.role = role;
            assert!(
                r.admit(token, reconnect, |_| Some(Duration::from_secs(22)))
                    .unwrap()
            );
            assert_eq!(r.tracked[&TrackedId(2)].file.role, role);
            assert!(r.tracked[&TrackedId(2)].binding_blocked());
            let independent = offers.iter().find(|(_, loc)| *loc == locator(4)).unwrap().0;
            assert!(
                r.admit(independent, file(4, "/independent"), |_| Some(
                    Duration::from_secs(22)
                ))
                .unwrap()
            );
        }
    }
}

/// Scenario: A superseded predecessor reappears after its successor retires, first on an incomplete pass and then behind another unresolved rebound.
/// Guarantees: Same-role reconnection works, but only rebind after completeness and rebound resolution restores Live and clears the successor memo.
#[test]
fn superseded_publication_requires_rebind_and_the_rebound_barrier() {
    let mut r = Reconciler::new(limits()).unwrap();
    let mut old = file(1, "/s");
    old.resident = false;
    r.track(old).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(2, "/s"), &mut events);
    assert!(!finish(&mut r, false, &mut events).complete);
    let token = r.offers().unwrap().next().unwrap().token;
    assert!(
        r.admit(token, file(2, "/s"), |_| Some(Duration::from_secs(12)))
            .unwrap()
    );
    drop(r.untrack(TrackedId(2)).unwrap());
    assert_eq!(
        r.tracked[&TrackedId(1)].successor,
        Some((TrackedId(2), locator(2)))
    );
    r.track(file(3, "/other")).unwrap();
    start(&mut r, 2);
    feed(&mut r, &observation(1, "/s"), &mut events);
    assert!(!finish(&mut r, false, &mut events).complete);
    let token = r.offers().unwrap().next().unwrap().token;
    assert_eq!(
        r.admit(token, file(1, "/s"), |_| panic!(
            "incomplete promotion reached durability"
        )),
        Err(ReconcileError::Tracked)
    );
    let mut reconnect = file(1, "/s");
    reconnect.role = BindingRole::Superseded;
    assert!(
        r.admit(token, reconnect, |_| Some(Duration::from_secs(22)))
            .unwrap()
    );
    assert_eq!(
        r.rebind(2, TrackedId(1), |_| panic!(
            "incomplete rebind reached durability"
        )),
        Err(ReconcileError::Feedback)
    );
    r.update_reader_state(2, TrackedId(1), true, false).unwrap();
    start(&mut r, 3);
    feed(&mut r, &observation(1, "/s"), &mut events);
    feed(&mut r, &observation(4, "/other"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    let token = r
        .offers()
        .unwrap()
        .find(|o| o.candidate.locator == locator(1))
        .unwrap()
        .token;
    assert_eq!(
        r.admit(token, file(1, "/s"), |_| panic!(
            "superseded promotion bypassed rebind"
        )),
        Err(ReconcileError::Tracked)
    );
    assert_eq!(
        r.rebind(3, TrackedId(1), |_| panic!(
            "unresolved rebound bypassed barrier"
        )),
        Err(ReconcileError::Feedback)
    );
    r.resolve_rebound(3, TrackedId(3), locator(4)).unwrap();
    assert!(
        r.rebind(3, TrackedId(1), |path| {
            assert_eq!(path, Path::new("/s"));
            true
        })
        .unwrap()
    );
    assert_eq!(r.tracked[&TrackedId(1)].file.role, BindingRole::Live);
    assert!(r.tracked[&TrackedId(1)].successor.is_none());
}

fn assert_parent_families_uniform(r: &Reconciler) {
    let mut families = std::collections::HashMap::new();
    for tracked in r.tracked.values() {
        if let Some(parent) = tracked.file.binding.as_deref().and_then(Path::parent) {
            let state = (tracked.parent, tracked.parent_changed);
            if let Some(previous) = families.insert(parent.as_os_str(), state) {
                assert_eq!(previous, state, "parent family {}", parent.display());
            }
        }
    }
}

/// Scenario: Parent families undergo initial targeted validation, change, restoration, insertion, removal and cross-directory rebinding.
/// Guarantees: Every member shares persistent parent state, so even an unknown baseline can be read from one member safely.
#[test]
fn parent_families_remain_uniform_through_lifecycle() {
    let mut r = Reconciler::new(limits()).unwrap();
    for (id, path) in [(1, "/logs/a"), (2, "/logs/b"), (3, "/other/c")] {
        r.track(file(id, path)).unwrap();
        assert_parent_families_uniform(&r);
    }
    let mut events = Vec::new();
    start(&mut r, 1);
    apply_test_check(
        &mut r,
        1,
        &observation(1, "/logs/a"),
        Some(locator(10)),
        false,
        &mut events,
    );
    assert_parent_families_uniform(&r);
    assert_eq!(r.tracked[&TrackedId(2)].parent, Some(locator(10)));
    assert!(finish(&mut r, true, &mut events).complete);
    r.track(file(4, "/logs/d")).unwrap();
    r.track(file(5, "/unknown/e")).unwrap();
    assert_parent_families_uniform(&r);
    drop(r.untrack(TrackedId(2)).unwrap());
    assert_parent_families_uniform(&r);
    start(&mut r, 2);
    feed(
        &mut r,
        &evidence(Some(11), "/logs", ScanReason::Directory),
        &mut events,
    );
    assert_parent_families_uniform(&r);
    assert!(r.tracked[&TrackedId(4)].parent_changed);
    apply_test_check(
        &mut r,
        1,
        &evidence(None, "/logs/a", ScanReason::Missing),
        Some(locator(10)),
        true,
        &mut events,
    );
    assert_parent_families_uniform(&r);
    assert!(!r.tracked[&TrackedId(4)].parent_changed);
    assert!(!finish(&mut r, true, &mut events).complete);
    r.track(file(6, "/logs/f")).unwrap();
    assert_parent_families_uniform(&r);
    start(&mut r, 3);
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    feed(&mut r, &observation(3, "/logs/0"), &mut events);
    feed(&mut r, &observation(1, "/new/0"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(r.rebind(3, TrackedId(3), |_| true).unwrap());
    assert_eq!(r.tracked[&TrackedId(3)].parent, Some(locator(10)));
    assert_parent_families_uniform(&r);
    assert!(r.rebind(3, TrackedId(1), |_| true).unwrap());
    assert_eq!(r.tracked[&TrackedId(1)].parent, None);
    assert_parent_families_uniform(&r);
    drop(r.untrack(TrackedId(4)).unwrap());
    assert_parent_families_uniform(&r);
}

/// Scenario: A parent-change result arrives before a baseline is known and another record is registered afterward.
/// Guarantees: Unknown parent state retains its change latch uniformly instead of losing it during admission lookup.
#[test]
fn unknown_parent_change_is_preserved_by_new_family_members() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/unknown/a")).unwrap();
    r.track(file(2, "/unknown/b")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    let failure = evidence(None, "/unknown/a", ScanReason::RootChanged);
    r.apply_binding_check(
        1,
        TrackedId(1),
        super::super::BindingCheck {
            observation: &failure,
            parent: None,
            context_valid: false,
            scope: super::super::BindingScope::ParentChanged,
            diagnostic_path: Path::new("/unknown"),
        },
        Duration::from_secs(10),
        |e| record(e, &mut events),
    )
    .unwrap();
    assert_parent_families_uniform(&r);
    assert!(!finish(&mut r, true, &mut events).complete);
    r.track(file(3, "/unknown/c")).unwrap();
    assert_eq!(r.tracked[&TrackedId(3)].parent, None);
    assert!(r.tracked[&TrackedId(3)].parent_changed);
    assert_parent_families_uniform(&r);
    start(&mut r, 2);
    feed(
        &mut r,
        &evidence(Some(10), "/unknown", ScanReason::Directory),
        &mut events,
    );
    assert_parent_families_uniform(&r);
    assert_eq!(r.tracked[&TrackedId(3)].parent, Some(locator(10)));
    assert!(finish(&mut r, true, &mut events).complete);
}

/// Scenario: An earlier alias returns a local coverage-invalidating failure after the binding is positively excluded.
/// Guarantees: Required alias uncertainty emits failure immediately, prevents exclusion and prevents complete-pass fallback.
#[test]
fn local_alias_failures_block_positive_exclusion() {
    for reason in [
        ScanReason::Coverage,
        ScanReason::ProbeBusy,
        ScanReason::Limit,
        ScanReason::DescriptorExhausted,
        ScanReason::Unstable,
    ] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/binding")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(&mut r, &observation(1, "/alias"), &mut events);
        assert!(finish(&mut r, true, &mut events).complete);
        start(&mut r, 2);
        feed(
            &mut r,
            &evidence(Some(1), "/binding", ScanReason::Excluded),
            &mut events,
        );
        assert!(!events.contains(&"excluded:1".into()));
        apply_test_check(
            &mut r,
            1,
            &evidence(None, "/alias", reason),
            None,
            false,
            &mut events,
        );
        assert!(events.contains(&"coverage".into()));
        assert!(!events.contains(&"excluded:1".into()));
        assert!(!finish(&mut r, true, &mut events).complete);
        assert!(!events.contains(&"excluded:1".into()));
    }
}

/// Scenario: Real scanner alias checks encounter an occupied probe budget or an inaccessible ancestor after positive binding exclusion.
/// Guarantees: Local scanner failures block revocation, preserve the alias hint and recover on a fresh pass; privileged processes skip only the permission subcase.
#[test]
fn scanner_alias_uncertainty_blocks_exclusion() {
    use std::os::unix::fs::PermissionsExt;
    struct RestorePermissions(PathBuf, std::fs::Permissions);
    impl Drop for RestorePermissions {
        fn drop(&mut self) {
            let _ = std::fs::set_permissions(&self.0, self.1.clone());
        }
    }
    for inaccessible in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let protected = dir.path().join("aliases");
        std::fs::create_dir_all(protected.join("nested")).unwrap();
        let binding = dir.path().join("binding.secret");
        let alias = protected.join("nested/file.log");
        std::fs::write(&binding, b"abc").unwrap();
        std::fs::hard_link(&binding, &alias).unwrap();
        let actual = FileLocator::from_metadata(&std::fs::metadata(&alias).unwrap());
        let mut tracked = file(1, binding.to_str().unwrap());
        tracked.locator = actual;
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(tracked).unwrap();
        let mut scanner = scanner(dir.path());
        let budget = super::super::ProbeBudget::default();
        let mut events = Vec::new();
        start(&mut r, 1);
        scanner
            .check_binding(
                &alias,
                None,
                &budget,
                || false,
                |check| {
                    assert!(check.observation.reason.is_none());
                    feed(&mut r, check.observation, &mut events);
                },
            )
            .unwrap();
        assert!(finish(&mut r, true, &mut events).complete);
        start(&mut r, 2);
        scanner
            .check_binding(
                &binding,
                None,
                &budget,
                || false,
                |check| {
                    assert_eq!(check.observation.reason, Some(ScanReason::Excluded));
                    assert_eq!(check.observation.locator, Some(actual));
                    feed(&mut r, check.observation, &mut events);
                },
            )
            .unwrap();
        let restore = if inaccessible {
            let guard = RestorePermissions(
                protected.clone(),
                std::fs::metadata(&protected).unwrap().permissions(),
            );
            std::fs::set_permissions(&protected, std::fs::Permissions::from_mode(0o0)).unwrap();
            match std::fs::metadata(&alias) {
                Ok(_) => {
                    eprintln!("permission subcase skipped: process bypasses directory permissions");
                    continue;
                }
                Err(e) => assert_eq!(e.raw_os_error(), Some(libc::EACCES)),
            }
            Some(guard)
        } else {
            None
        };
        let reservation = if inaccessible {
            None
        } else {
            Some(budget.try_reserve().unwrap())
        };
        let expected = if inaccessible {
            ScanReason::Coverage
        } else {
            ScanReason::ProbeBusy
        };
        {
            let mut batch = scanner.binding_batch(&budget, || false).unwrap();
            let request = r.binding_request(TrackedId(1)).unwrap();
            assert_eq!(request.path, alias);
            let check = batch.check(request.path, request.parent).unwrap();
            assert_eq!(check.scope, super::super::BindingScope::Binding);
            assert_eq!(check.observation.reason, Some(expected));
            if inaccessible {
                assert_eq!(check.observation.errno, Some(libc::EACCES));
            }
            r.apply_binding_check(2, TrackedId(1), check, Duration::from_secs(20), |e| {
                record(e, &mut events)
            })
            .unwrap();
            assert!(!events.contains(&"excluded:1".into()));
            assert!(!finish(&mut r, true, &mut events).complete);
            assert!(!events.contains(&"excluded:1".into()));
        }
        drop(reservation);
        drop(restore);
        start(&mut r, 3);
        scanner
            .check_binding(
                &binding,
                None,
                &budget,
                || false,
                |check| feed(&mut r, check.observation, &mut events),
            )
            .unwrap();
        let request = r.binding_request(TrackedId(1)).unwrap();
        let mut batch = scanner.binding_batch(&budget, || false).unwrap();
        let check = batch.check(request.path, request.parent).unwrap();
        assert!(check.observation.reason.is_none());
        r.apply_binding_check(3, TrackedId(1), check, Duration::from_secs(30), |e| {
            record(e, &mut events)
        })
        .unwrap();
        assert!(finish(&mut r, true, &mut events).complete);
        assert!(!events.contains(&"excluded:1".into()));
        eprintln!("scanner alias failure verified: {expected:?}");
    }
}

fn named_observation(id: u64, path: &str, reason: Option<ScanReason>) -> Observation {
    reason.map_or_else(
        || observation(id, path),
        |reason| evidence(Some(id), path, reason),
    )
}

/// Scenario: Full or policy-qualified binding evidence contradicts traversal or owner-supplied rebounds in either order.
/// Guarantees: Failure is immediate, supersession is refused, and a fresh pass without naming evidence still permits legitimate replacement.
#[test]
fn binding_names_and_rebounds_conflict_at_every_evidence_strength() {
    for reason in [
        None,
        Some(ScanReason::PresenceOnly),
        Some(ScanReason::Content),
    ] {
        for association in [false, true] {
            for rebound_first in [false, true] {
                for targeted in [false, true] {
                    let mut r = Reconciler::new(limits()).unwrap();
                    r.track(file(1, "/binding")).unwrap();
                    let mut events = Vec::new();
                    start(&mut r, 1);
                    feed(&mut r, &observation(2, "/candidate"), &mut events);
                    for rebound in [rebound_first, !rebound_first] {
                        if rebound {
                            if association {
                                r.associate_replacement(1, TrackedId(1), locator(2), |e| {
                                    record(e, &mut events)
                                })
                                .unwrap();
                            } else {
                                feed(&mut r, &observation(2, "/binding"), &mut events);
                            }
                        } else {
                            let named = named_observation(1, "/binding", reason);
                            if targeted {
                                apply_test_check(&mut r, 1, &named, None, false, &mut events);
                            } else {
                                feed(&mut r, &named, &mut events);
                            }
                        }
                    }
                    assert!(events.contains(&"coverage".into()));
                    assert!(!r.pass.as_ref().unwrap().complete);
                    assert!(!finish(&mut r, true, &mut events).complete);
                    assert_eq!(
                        r.supersede(1, TrackedId(1), locator(2)),
                        Err(ReconcileError::Feedback)
                    );
                    assert_eq!(r.tracked[&TrackedId(1)].file.role, BindingRole::Live);
                    start(&mut r, 2);
                    feed(&mut r, &observation(2, "/candidate"), &mut events);
                    r.associate_replacement(2, TrackedId(1), locator(2), |e| {
                        record(e, &mut events)
                    })
                    .unwrap();
                    assert!(!finish(&mut r, false, &mut events).complete);
                    r.supersede(2, TrackedId(1), locator(2)).unwrap();
                }
            }
        }
    }
}

/// Scenario: A policy-qualified name accompanies an excluded alias, before or after it, with complete or incomplete traversal.
/// Guarantees: Both exclusion paths are vetoed without granting full validation or blocking unrelated complete-pass disappearance.
#[test]
fn policy_names_veto_exclusion_without_granting_authority() {
    for reason in [ScanReason::Content, ScanReason::PresenceOnly] {
        for path in ["/binding", "/allowed"] {
            for named_first in [false, true] {
                for complete in [false, true] {
                    let mut r = Reconciler::new(limits()).unwrap();
                    r.track(file(1, "/binding")).unwrap();
                    r.track(file(2, "/unseen")).unwrap();
                    let mut events = Vec::new();
                    start(&mut r, 1);
                    feed(
                        &mut r,
                        &evidence(Some(99), "/", ScanReason::Directory),
                        &mut events,
                    );
                    for named in [named_first, !named_first] {
                        feed(
                            &mut r,
                            &if named {
                                evidence(Some(1), path, reason)
                            } else {
                                evidence(Some(1), "/excluded", ScanReason::Excluded)
                            },
                            &mut events,
                        );
                    }
                    assert!(r.binding_request(TrackedId(1)).is_none());
                    apply_test_check(
                        &mut r,
                        1,
                        &evidence(None, "/binding", ScanReason::Missing),
                        Some(locator(99)),
                        true,
                        &mut events,
                    );
                    assert!(!events.contains(&"coverage".into()));
                    assert!(!events.contains(&"excluded:1".into()));
                    assert!(!events.contains(&"validated:1".into()));
                    assert_eq!(finish(&mut r, complete, &mut events).complete, complete);
                    assert!(!events.contains(&"excluded:1".into()));
                    assert_eq!(events.contains(&"missing:2".into()), complete);
                    assert!(r.offers().unwrap().next().is_none());
                    assert!(r.binding_proposal(1, TrackedId(1)).is_err());
                }
            }
        }
    }
}

/// Scenario: A binding check cannot identify any locator, or identifies a different locator through Content evidence.
/// Guarantees: Neither protects the old reader; complete fallback still excludes it without treating Content as receiver-wide failure.
#[test]
fn policy_evidence_for_another_locator_does_not_veto_fallback() {
    for observed in [None, Some(3)] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/binding")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(
            &mut r,
            &evidence(Some(1), "/excluded", ScanReason::Excluded),
            &mut events,
        );
        let check = evidence(
            observed,
            "/binding",
            if observed.is_some() {
                ScanReason::Content
            } else {
                ScanReason::Coverage
            },
        );
        apply_test_check(&mut r, 1, &check, None, false, &mut events);
        assert!(!events.contains(&"excluded:1".into()));
        assert!(finish(&mut r, true, &mut events).complete);
        assert!(events.contains(&"excluded:1".into()));
        assert!(r.offers().unwrap().next().is_none());
    }
}

/// Scenario: A name seen only through policy-qualified evidence is needed as an earlier-alias hint in the next pass.
/// Guarantees: Exclusion waits for revalidation; clean/Content results protect the reader while Missing/different-locator results expire the hint.
#[test]
fn policy_alias_hints_survive_without_carrying_full_authority() {
    for reason in [ScanReason::Content, ScanReason::PresenceOnly] {
        for result in 0..4 {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/binding")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(&mut r, &evidence(Some(1), "/allowed", reason), &mut events);
            assert!(finish(&mut r, true, &mut events).complete);
            assert!(r.binding_proposal(1, TrackedId(1)).is_err());
            start(&mut r, 2);
            events.clear();
            assert_eq!(
                r.tracked[&TrackedId(1)].pass.minimum_evidence,
                NameEvidence::None
            );
            feed(
                &mut r,
                &evidence(Some(1), "/binding", ScanReason::Excluded),
                &mut events,
            );
            assert!(!events.contains(&"excluded:1".into()));
            assert_eq!(
                r.binding_request(TrackedId(1)).unwrap().path,
                Path::new("/allowed")
            );
            let alias = match result {
                0 => observation(1, "/allowed"),
                1 => evidence(Some(1), "/allowed", ScanReason::Content),
                2 => evidence(None, "/allowed", ScanReason::Missing),
                _ => observation(2, "/allowed"),
            };
            apply_test_check(&mut r, 1, &alias, None, false, &mut events);
            assert_eq!(
                r.tracked[&TrackedId(1)]
                    .previous_alias
                    .as_os_str()
                    .is_empty(),
                result >= 2
            );
            assert_eq!(events.contains(&"excluded:1".into()), result >= 2);
            assert!(finish(&mut r, true, &mut events).complete);
            assert_eq!(events.contains(&"excluded:1".into()), result >= 2);
        }
    }
}

/// Scenario: Weak and full aliases arrive in either order, at the same path or with the full path sorting later.
/// Guarantees: Full evidence wins, equal-path upgrades do not duplicate indexes, and replaced keys leave no stale entries after publication/removal.
#[test]
fn alias_strength_upgrades_preserve_index_and_publication_rules() {
    for reason in [ScanReason::Content, ScanReason::PresenceOnly] {
        for full_path in ["/a", "/z"] {
            for full_first in [false, true] {
                let mut r = Reconciler::new(limits()).unwrap();
                r.track(file(1, "/binding")).unwrap();
                let mut events = Vec::new();
                start(&mut r, 1);
                for full in [full_first, !full_first] {
                    feed(
                        &mut r,
                        &if full {
                            observation(1, full_path)
                        } else {
                            evidence(Some(1), "/a", reason)
                        },
                        &mut events,
                    );
                }
                assert_eq!(r.tracked[&TrackedId(1)].minimum, Path::new(full_path));
                assert_eq!(
                    r.tracked[&TrackedId(1)].pass.minimum_evidence,
                    NameEvidence::Full
                );
                assert_eq!(
                    r.aliases
                        .get(Path::new(full_path).as_os_str(), &r.tracked)
                        .collect::<Vec<_>>(),
                    [TrackedId(1)]
                );
                if full_path != "/a" {
                    assert!(
                        r.aliases
                            .get(Path::new("/a").as_os_str(), &r.tracked)
                            .next()
                            .is_none()
                    );
                    // Discarded aliases are not an unbounded conflict history.
                    feed(&mut r, &observation(2, "/a"), &mut events);
                }
                assert!(finish(&mut r, true, &mut events).complete);
                assert_eq!(
                    r.binding_proposal(1, TrackedId(1)).unwrap(),
                    Path::new(full_path)
                );
                assert!(r.rebind(1, TrackedId(1), |_| true).unwrap());
                assert_eq!(
                    r.tracked[&TrackedId(1)].pass.minimum_evidence,
                    NameEvidence::None
                );
                drop(r.untrack(TrackedId(1)).unwrap());
                assert!(
                    r.aliases
                        .get(Path::new(full_path).as_os_str(), &r.tracked)
                        .next()
                        .is_none()
                );
            }
        }
    }
}

/// Scenario: An established binding and an alias have different evidence strengths in either observation order.
/// Guarantees: A weakly named Live binding is preserved without publication authority; a full binding remains usable despite a weak minimum.
#[test]
fn established_binding_precedence_respects_evidence_strength() {
    for role in [
        BindingRole::Live,
        BindingRole::Unverified,
        BindingRole::Superseded,
    ] {
        for reason in [ScanReason::Content, ScanReason::PresenceOnly] {
            for binding_full in [false, true] {
                for binding_first in [false, true] {
                    let mut r = Reconciler::new(limits()).unwrap();
                    let mut tracked = file(1, "/z");
                    tracked.role = role;
                    r.track(tracked).unwrap();
                    let mut events = Vec::new();
                    start(&mut r, 1);
                    for binding in [binding_first, !binding_first] {
                        let path = if binding { "/z" } else { "/a" };
                        feed(
                            &mut r,
                            &named_observation(
                                1,
                                path,
                                if binding == binding_full {
                                    None
                                } else {
                                    Some(reason)
                                },
                            ),
                            &mut events,
                        );
                    }
                    assert!(finish(&mut r, true, &mut events).complete);
                    if binding_full {
                        assert_eq!(
                            r.binding_proposal(1, TrackedId(1)).unwrap(),
                            Path::new("/z")
                        );
                    } else if role != BindingRole::Live {
                        assert_eq!(
                            r.binding_proposal(1, TrackedId(1)).unwrap(),
                            Path::new("/a")
                        );
                    } else {
                        assert_eq!(
                            r.rebind(1, TrackedId(1), |_| panic!(
                                "weak binding authorized publication"
                            )),
                            Err(ReconcileError::Feedback)
                        );
                    }
                    assert_eq!(
                        r.tracked[&TrackedId(1)].file.binding.as_deref(),
                        Some(Path::new("/z"))
                    );
                }
            }
        }
    }
}

/// Scenario: Policy-qualified aliases conflict with another tracked alias or an untracked full candidate in either order.
/// Guarantees: Retained-path contradictions invalidate coverage immediately and survive without granting weak publication authority.
#[test]
fn policy_alias_conflicts_are_order_independent() {
    for reason in [ScanReason::Content, ScanReason::PresenceOnly] {
        for other_tracked in [false, true] {
            for weak_first in [false, true] {
                let mut r = Reconciler::new(limits()).unwrap();
                r.track(file(1, "/one")).unwrap();
                if other_tracked {
                    r.track(file(2, "/two")).unwrap();
                }
                let mut events = Vec::new();
                start(&mut r, 1);
                for weak in [weak_first, !weak_first] {
                    feed(
                        &mut r,
                        &if weak {
                            evidence(Some(1), "/shared", reason)
                        } else {
                            observation(2, "/shared")
                        },
                        &mut events,
                    );
                }
                assert!(events.contains(&"coverage".into()));
                assert!(!finish(&mut r, true, &mut events).complete);
                assert!(r.offers().unwrap().next().is_none());
            }
        }
    }
}

/// Scenario: A prior full offer is followed by weak-only evidence for nonresident and untracked locators in a fresh pass.
/// Guarantees: Old full authority and tokens are not reused; weak names emit presence but no offer, validation or binding proposal for recovered/historical roles.
#[test]
fn weak_only_passes_cannot_reuse_identity_authority() {
    for role in [BindingRole::Unverified, BindingRole::Superseded] {
        for reason in [ScanReason::Content, ScanReason::PresenceOnly] {
            let mut r = Reconciler::new(limits()).unwrap();
            let mut tracked = file(1, "/binding");
            tracked.role = role;
            tracked.resident = false;
            r.track(tracked).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(&mut r, &observation(1, "/allowed"), &mut events);
            assert!(finish(&mut r, true, &mut events).complete);
            let old = r.offers().unwrap().next().unwrap().token;
            r.acknowledge(old, IdentityFeedback::Deferred, Duration::from_secs(12))
                .unwrap();
            start(&mut r, 2);
            events.clear();
            feed(&mut r, &evidence(Some(1), "/allowed", reason), &mut events);
            feed(
                &mut r,
                &evidence(Some(3), "/untracked", reason),
                &mut events,
            );
            assert!(finish(&mut r, true, &mut events).complete);
            assert!(events.contains(&"present:1".into()));
            assert!(!events.contains(&"validated:1".into()));
            assert!(!events.contains(&"missing:1".into()));
            assert!(r.offers().unwrap().next().is_none());
            assert!(r.binding_proposal(2, TrackedId(1)).is_err());
            assert_eq!(
                r.admit(old, file(1, "/binding"), |_| panic!(
                    "stale authority reached identity owner"
                )),
                Err(ReconcileError::Feedback)
            );
            assert_eq!(r.tracked[&TrackedId(1)].file.role, role);
        }
    }
}

/// Scenario: Descriptor exhaustion or cancellation retains a locator without certifying policy.
/// Guarantees: The affected reader is not excluded, but no policy/full name, candidate or publication authority is manufactured.
#[test]
fn uncertain_locator_failures_veto_exclusion_without_eligibility() {
    for reason in [ScanReason::DescriptorExhausted, ScanReason::Cancelled] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/binding")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(&mut r, &evidence(Some(1), "/maybe", reason), &mut events);
        feed(
            &mut r,
            &evidence(Some(1), "/binding", ScanReason::Excluded),
            &mut events,
        );
        assert_eq!(
            r.tracked[&TrackedId(1)].pass.name_evidence(),
            NameEvidence::None
        );
        assert_eq!(
            r.tracked[&TrackedId(1)].pass.binding_evidence,
            NameEvidence::None
        );
        assert!(r.tracked[&TrackedId(1)].minimum.as_os_str().is_empty());
        assert!(!events.contains(&"excluded:1".into()));
        assert!(!finish(&mut r, true, &mut events).complete);
        assert!(!events.contains(&"excluded:1".into()));
        assert!(r.offers().unwrap().next().is_none());
    }
}

/// Scenario: Real scanner probes produce Content after file read-open denial or PresenceOnly for an expected locator.
/// Guarantees: Both protect an existing reader from exclusion without validation or publication authority, and content failure leaves unrelated coverage complete.
#[test]
fn scanner_policy_names_protect_existing_readers() {
    use std::{io::Read, os::unix::fs::PermissionsExt};
    struct Restore(PathBuf, std::fs::Permissions);
    impl Drop for Restore {
        fn drop(&mut self) {
            let _ = std::fs::set_permissions(&self.0, self.1.clone());
        }
    }
    for presence_only in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let allowed = dir.path().join("allowed.log");
        let binding = dir.path().join("binding.secret");
        std::fs::write(&allowed, b"abc").unwrap();
        std::fs::hard_link(&allowed, &binding).unwrap();
        let actual = FileLocator::from_metadata(&std::fs::metadata(&allowed).unwrap());
        let mut reader = std::fs::File::open(&allowed).unwrap();
        let _restore = Restore(
            allowed.clone(),
            std::fs::metadata(&allowed).unwrap().permissions(),
        );
        std::fs::set_permissions(&allowed, std::fs::Permissions::from_mode(0o000)).unwrap();
        if !presence_only {
            match std::fs::File::open(&allowed) {
                Ok(_) => {
                    eprintln!("Content permission subcase skipped: privileged process");
                    continue;
                }
                Err(error) => assert_eq!(error.raw_os_error(), Some(libc::EACCES)),
            }
        }
        let mut scanner = scanner(dir.path());
        let budget = super::super::ProbeBudget::default();
        let mut name = None;
        let completion = scanner.scan_with_requests(
            1,
            &budget,
            |_| {
                if presence_only {
                    super::super::ProbeRequest::PresenceOnly { expected: actual }
                } else {
                    super::super::ProbeRequest::Full
                }
            },
            || false,
            |batch| {
                for o in batch {
                    if o.path == allowed {
                        name = Some(Observation {
                            path: o.path.clone(),
                            locator: o.locator,
                            size: o.size,
                            modified: o.modified,
                            fingerprint: o.fingerprint.clone(),
                            reason: o.reason,
                            errno: o.errno,
                        });
                    }
                }
                true
            },
        );
        assert!(completion.complete);
        let name = name.expect("allowed name was probed");
        assert_eq!(
            name.reason,
            Some(if presence_only {
                ScanReason::PresenceOnly
            } else {
                ScanReason::Content
            })
        );
        assert_eq!(name.locator, Some(actual));
        if !presence_only {
            assert_eq!(name.errno, Some(libc::EACCES));
        }
        let mut r = Reconciler::new(limits()).unwrap();
        let mut tracked = file(1, binding.to_str().unwrap());
        tracked.locator = actual;
        r.track(tracked).unwrap();
        r.track(file(2, "/unrelated-missing")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(&mut r, &name, &mut events);
        scanner
            .check_binding(
                &binding,
                None,
                &budget,
                || false,
                |check| {
                    assert_eq!(check.observation.reason, Some(ScanReason::Excluded));
                    feed(&mut r, check.observation, &mut events);
                },
            )
            .unwrap();
        // An occupied budget cannot poison the pass through an unnecessary
        // exclusion-only probe once a policy-qualified name protects the reader.
        let reservation = budget.try_reserve().unwrap();
        assert!(r.next_binding_check().is_none());
        assert!(finish(&mut r, true, &mut events).complete);
        assert!(!events.contains(&"excluded:1".into()));
        assert!(!events.contains(&"validated:1".into()));
        assert!(events.contains(&"missing:2".into()));
        assert!(r.offers().unwrap().next().is_none());
        assert!(r.binding_proposal(1, TrackedId(1)).is_err());
        drop(reservation);
        let mut bytes = Vec::new();
        assert_eq!(reader.read_to_end(&mut bytes).unwrap(), 3);
        assert_eq!(bytes, b"abc");
        eprintln!(
            "policy-qualified scanner result verified: {:?}",
            name.reason
        );
    }
}

/// Scenario: A caller supplies policy-qualified evidence without a complete native filename.
/// Guarantees: Invalid weak names cannot become retained hints that make targeted checking fail repeatedly.
#[test]
fn policy_name_hints_require_complete_filenames() {
    for reason in [ScanReason::Content, ScanReason::PresenceOnly] {
        for path in ["/", "/logs/", "/logs/."] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/binding")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            assert_eq!(
                r.observe(
                    1,
                    &evidence(Some(1), path, reason),
                    Duration::from_secs(10),
                    |e| record(e, &mut events)
                ),
                Err(ReconcileError::Observation)
            );
            assert!(r.tracked[&TrackedId(1)].minimum.as_os_str().is_empty());
            assert!(!finish(&mut r, true, &mut events).complete);
        }
    }
}

/// Scenario: A weak eligible name protects one reader while another binding is positively excluded and batch roots later fail.
/// Guarantees: Eligibility is record-local, exclusion-only work can stop, and the independent empty-worklist root failure still invalidates completion.
#[test]
fn policy_eligibility_is_local_and_does_not_skip_root_failures() {
    for reason in [ScanReason::Content, ScanReason::PresenceOnly] {
        for weak_first in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/binding")).unwrap();
            r.track(file(2, "/other")).unwrap();
            r.track(file(3, "/missing")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            for weak in [weak_first, !weak_first] {
                feed(
                    &mut r,
                    &if weak {
                        evidence(Some(1), "/allowed", reason)
                    } else {
                        evidence(Some(2), "/other", ScanReason::Excluded)
                    },
                    &mut events,
                );
            }
            feed(
                &mut r,
                &evidence(Some(1), "/binding", ScanReason::Excluded),
                &mut events,
            );
            assert!(events.contains(&"excluded:2".into()));
            assert!(!events.contains(&"excluded:1".into()));
            assert!(!events.contains(&"coverage".into()));
            assert!(r.next_binding_check().is_none());
            r.apply_binding_batch_error(
                1,
                &super::super::BindingBatchError {
                    reason: ScanReason::RootUnavailable,
                    errno: None,
                    root: Some(Path::new("/")),
                },
                |e| record(e, &mut events),
            )
            .unwrap();
            assert!(!finish(&mut r, true, &mut events).complete);
            assert!(!events.contains(&"missing:3".into()));
        }
    }
}

/// Scenario: A binding-local descriptor failure retains the file locator after otherwise complete traversal.
/// Guarantees: Its uncertainty vetoes fallback exclusion without manufacturing policy evidence or blocking unrelated disappearance.
#[test]
fn targeted_descriptor_uncertainty_survives_complete_fallback() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/binding")).unwrap();
    r.track(file(2, "/unseen")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(1), "/excluded", ScanReason::Excluded),
        &mut events,
    );
    apply_test_check(
        &mut r,
        1,
        &evidence(Some(1), "/binding", ScanReason::DescriptorExhausted),
        Some(locator(99)),
        false,
        &mut events,
    );
    assert_eq!(
        r.tracked[&TrackedId(1)].pass.name_evidence(),
        NameEvidence::None
    );
    assert_eq!(
        r.tracked[&TrackedId(1)].pass.binding_evidence,
        NameEvidence::None
    );
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(!events.contains(&"excluded:1".into()));
    assert!(events.contains(&"missing:2".into()));
}

/// Scenario: An untracked foreign locator weakly names a retained binding before or after that binding names its owner.
/// Guarantees: Both orders fail immediately, withhold unrelated disappearance, and recover only on a fresh contradiction-free pass.
#[test]
fn weak_foreign_binding_observations_are_order_independent() {
    for foreign_reason in [ScanReason::Content, ScanReason::PresenceOnly] {
        for own_reason in [
            None,
            Some(ScanReason::Content),
            Some(ScanReason::PresenceOnly),
        ] {
            for foreign_first in [false, true] {
                for targeted in [false, true] {
                    let mut r = Reconciler::new(limits()).unwrap();
                    r.track(file(1, "/binding")).unwrap();
                    r.track(file(3, "/unseen")).unwrap();
                    let mut events = Vec::new();
                    start(&mut r, 1);
                    for foreign in [foreign_first, !foreign_first] {
                        let o = if foreign {
                            evidence(Some(2), "/binding", foreign_reason)
                        } else {
                            named_observation(1, "/binding", own_reason)
                        };
                        if targeted {
                            apply_test_check(&mut r, 1, &o, None, false, &mut events);
                        } else {
                            feed(&mut r, &o, &mut events);
                        }
                    }
                    assert!(events.contains(&"coverage".into()));
                    assert!(!r.pass.as_ref().unwrap().complete);
                    assert!(!finish(&mut r, true, &mut events).complete);
                    assert!(!events.contains(&"missing:3".into()));
                    assert!(r.rebounds().next().is_none());
                    assert_eq!(r.tracked[&TrackedId(1)].file.role, BindingRole::Live);
                    start(&mut r, 2);
                    events.clear();
                    feed(&mut r, &observation(1, "/binding"), &mut events);
                    assert!(finish(&mut r, true, &mut events).complete);
                    assert!(events.contains(&"missing:3".into()));
                }
            }
        }
    }
}

/// Scenario: Two different foreign locators name a retained binding with weak or full evidence.
/// Guarantees: Contradictions among foreign files are detected too, rather than only contradictions with the original owner.
#[test]
fn retained_foreign_locator_conflicts_cover_both_strengths() {
    for second_full in [false, true] {
        for reverse in [false, true] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/binding")).unwrap();
            r.track(file(4, "/unseen")).unwrap();
            let weak = evidence(Some(2), "/binding", ScanReason::Content);
            let other = named_observation(
                3,
                "/binding",
                if second_full {
                    None
                } else {
                    Some(ScanReason::PresenceOnly)
                },
            );
            let mut events = Vec::new();
            start(&mut r, 1);
            for o in if reverse {
                [&other, &weak]
            } else {
                [&weak, &other]
            } {
                feed(&mut r, o, &mut events);
            }
            assert!(events.contains(&"coverage".into()));
            assert!(!finish(&mut r, true, &mut events).complete);
            assert!(!events.contains(&"missing:4".into()));
            assert!(r.offers().unwrap().next().is_none());
        }
    }
}

/// Scenario: One foreign Content observation has no contradictory current-pass observation.
/// Guarantees: It grants no offer, actionable rebound, resolution or supersession, and does not invalidate unrelated coverage or consume the rebound counter.
#[test]
fn weak_foreign_target_is_observation_not_replacement_authority() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/binding")).unwrap();
    r.track(file(3, "/unseen")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(2), "/binding", ScanReason::Content),
        &mut events,
    );
    assert!(!events.contains(&"coverage".into()));
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(events.contains(&"missing:3".into()));
    assert!(r.offers().unwrap().next().is_none());
    assert!(r.rebounds().next().is_none());
    assert_eq!(
        r.resolve_rebound(1, TrackedId(1), locator(2)),
        Err(ReconcileError::Feedback)
    );
    assert_eq!(
        r.supersede(1, TrackedId(1), locator(2)),
        Err(ReconcileError::Feedback)
    );
    assert_eq!(r.unresolved_rebounds, 0);
    drop(r.untrack(TrackedId(1)).unwrap());
    assert_eq!(r.unresolved_rebounds, 0);
}

/// Scenario: A weak foreign observation is upgraded by full traversal or owner association without changing its locator.
/// Guarantees: Only qualification exposes identity work; resolution and untracking preserve the pending counter.
#[test]
fn weak_target_upgrade_creates_actionable_rebound_once() {
    for associated in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/binding")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(
            &mut r,
            &evidence(Some(2), "/binding", ScanReason::Content),
            &mut events,
        );
        assert!(r.rebounds().next().is_none());
        if associated {
            feed(&mut r, &observation(2, "/candidate"), &mut events);
            r.associate_replacement(1, TrackedId(1), locator(2), |e| record(e, &mut events))
                .unwrap();
        } else {
            feed(&mut r, &observation(2, "/binding"), &mut events);
        }
        // A later weak result must not downgrade the qualified target.
        feed(
            &mut r,
            &evidence(Some(2), "/binding", ScanReason::Content),
            &mut events,
        );
        assert!(!events.contains(&"coverage".into()));
        assert!(!finish(&mut r, false, &mut events).complete);
        assert_eq!(r.unresolved_rebounds, 1);
        assert_eq!(
            r.rebounds().collect::<Vec<_>>(),
            [Rebound {
                predecessor: TrackedId(1),
                replacement: locator(2)
            }]
        );
        r.resolve_rebound(1, TrackedId(1), locator(2)).unwrap();
        assert_eq!(r.unresolved_rebounds, 0);
        r.supersede(1, TrackedId(1), locator(2)).unwrap();
        assert_eq!(r.unresolved_rebounds, 0);
        drop(r.untrack(TrackedId(1)).unwrap());
        assert_eq!(r.unresolved_rebounds, 0);
    }
}

/// Scenario: A weak target becomes the qualified edge that closes a two-record binding cycle.
/// Guarantees: Strength upgrade triggers incremental cycle detection immediately even though the locator itself did not change.
#[test]
fn target_strength_upgrade_checks_cycles_before_completion() {
    for weak_first in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/a")).unwrap();
        r.track(file(2, "/b")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        for weak in [weak_first, !weak_first] {
            feed(
                &mut r,
                &if weak {
                    evidence(Some(2), "/a", ScanReason::Content)
                } else {
                    observation(1, "/b")
                },
                &mut events,
            );
        }
        assert!(!events.contains(&"coverage".into()));
        feed(&mut r, &observation(2, "/a"), &mut events);
        assert!(events.contains(&"coverage".into()));
        assert!(!r.pass.as_ref().unwrap().complete);
        assert!(!finish(&mut r, true, &mut events).complete);
    }
}

/// Scenario: A qualified successor was memo-resolved and is then observed weakly at the historical binding.
/// Guarantees: Weak observations neither downgrade resolved identity work nor reintroduce pending counter work.
#[test]
fn weak_observation_preserves_resolved_successor_work() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/binding")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(&mut r, &observation(2, "/binding"), &mut events);
    assert!(!finish(&mut r, false, &mut events).complete);
    let token = r.offers().unwrap().next().unwrap().token;
    assert!(
        r.admit(token, file(2, "/binding"), |_| Some(Duration::from_secs(
            12
        )))
        .unwrap()
    );
    start(&mut r, 2);
    feed(&mut r, &observation(2, "/binding"), &mut events);
    feed(
        &mut r,
        &evidence(Some(2), "/binding", ScanReason::Content),
        &mut events,
    );
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(r.rebounds().next().is_none());
    assert_eq!(r.unresolved_rebounds, 0);
    drop(r.untrack(TrackedId(1)).unwrap());
    assert_eq!(r.unresolved_rebounds, 0);
}

/// Scenario: Weak and qualified binding targets share one bounded locator slot.
/// Guarantees: The tagged target adds no storage over the previous optional locator, and aggregate name strength needs no stored field.
#[test]
fn binding_target_layout_preserves_compact_storage() {
    assert!(size_of::<BindingTarget>() <= size_of::<Option<FileLocator>>());
    assert!(size_of::<TrackedPass>() <= 48);
}

/// Scenario: A weak target is seen under an inherited parent block, then qualified after same-pass restoration.
/// Guarantees: Qualification cannot revive tainted binding work; a later fresh pass can admit the replacement normally.
#[test]
fn weak_target_upgrade_preserves_the_parent_evidence_barrier() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/logs/app")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    feed(
        &mut r,
        &evidence(Some(11), "/logs", ScanReason::Directory),
        &mut events,
    );
    assert!(!finish(&mut r, true, &mut events).complete);
    start(&mut r, 2);
    feed(
        &mut r,
        &evidence(Some(2), "/logs/app", ScanReason::Content),
        &mut events,
    );
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    feed(&mut r, &observation(2, "/logs/app"), &mut events);
    assert!(!finish(&mut r, true, &mut events).complete);
    assert!(r.offers().unwrap().next().is_none());
    assert_eq!(
        r.supersede(2, TrackedId(1), locator(2)),
        Err(ReconcileError::Feedback)
    );
    start(&mut r, 3);
    feed(
        &mut r,
        &evidence(Some(10), "/logs", ScanReason::Directory),
        &mut events,
    );
    feed(&mut r, &observation(2, "/logs/app"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    let token = r.offers().unwrap().next().unwrap().token;
    assert!(
        r.admit(token, file(2, "/logs/app"), |_| Some(Duration::from_secs(
            32
        )))
        .unwrap()
    );
}

/// Scenario: A weak foreign binding observation disagrees with an owner association in either order.
/// Guarantees: The contradiction is delivered immediately, blocks supersession and prevents unrelated disappearance.
#[test]
fn weak_target_and_different_association_conflict_in_both_orders() {
    for associated_first in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/binding")).unwrap();
        r.track(file(4, "/unseen")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        feed(&mut r, &observation(3, "/candidate"), &mut events);
        for associated in [associated_first, !associated_first] {
            if associated {
                r.associate_replacement(1, TrackedId(1), locator(3), |e| record(e, &mut events))
                    .unwrap();
            } else {
                feed(
                    &mut r,
                    &evidence(Some(2), "/binding", ScanReason::Content),
                    &mut events,
                );
            }
        }
        assert!(events.contains(&"coverage".into()));
        assert!(!r.pass.as_ref().unwrap().complete);
        assert!(!finish(&mut r, true, &mut events).complete);
        assert!(!events.contains(&"missing:4".into()));
        assert_eq!(
            r.supersede(1, TrackedId(1), locator(3)),
            Err(ReconcileError::Feedback)
        );
    }
}

/// Scenario: A predecessor has a full alternate name while its old binding weakly names a replacement.
/// Guarantees: Reselection preserves replacement context until qualification, without withholding unrelated publication.
#[test]
fn weak_target_preserves_replacement_context_until_qualified() {
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(file(1, "/binding")).unwrap();
    r.track(file(3, "/other")).unwrap();
    let mut events = Vec::new();
    start(&mut r, 1);
    feed(
        &mut r,
        &evidence(Some(2), "/binding", ScanReason::Content),
        &mut events,
    );
    feed(&mut r, &observation(1, "/alias"), &mut events);
    feed(&mut r, &observation(3, "/other"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    assert!(r.rebounds().next().is_none());
    assert_eq!(
        r.binding_proposal(1, TrackedId(1)),
        Err(ReconcileError::Feedback)
    );
    assert_eq!(
        r.binding_proposal(1, TrackedId(3)).unwrap(),
        Path::new("/other")
    );
    start(&mut r, 2);
    feed(&mut r, &observation(2, "/binding"), &mut events);
    feed(&mut r, &observation(1, "/alias"), &mut events);
    assert!(finish(&mut r, true, &mut events).complete);
    let offer = r.offers().unwrap().next().unwrap();
    assert_eq!(
        offer.candidate.replacement,
        Replacement::Possible(TrackedId(1))
    );
    r.resolve_rebound(2, TrackedId(1), locator(2)).unwrap();
    assert_eq!(
        r.binding_proposal(2, TrackedId(1)).unwrap(),
        Path::new("/alias")
    );
}

/// Scenario: A qualified replacement has a smaller alias and its binding names the original owner or another replacement in either order.
/// Guarantees: Every current-pass conflicting replacement is withheld, including aliases observed after the conflict; independent work and fresh-pass recovery remain available.
#[test]
fn current_binding_conflicts_withhold_candidates_across_alias_orders() {
    for other in [1, 3] {
        for replacement_first in [false, true] {
            for alias_first in [false, true] {
                let mut r = Reconciler::new(limits()).unwrap();
                r.track(file(1, "/bound")).unwrap();
                let mut events = Vec::new();
                start(&mut r, 1);
                if alias_first {
                    feed(&mut r, &observation(2, "/alias"), &mut events);
                }
                for inode in if replacement_first {
                    [2, other]
                } else {
                    [other, 2]
                } {
                    feed(&mut r, &observation(inode, "/bound"), &mut events);
                }
                if !alias_first {
                    feed(&mut r, &observation(2, "/alias"), &mut events);
                }
                feed(&mut r, &observation(99, "/independent"), &mut events);
                assert!(!finish(&mut r, true, &mut events).complete);
                assert_eq!(
                    r.offers()
                        .unwrap()
                        .map(|o| o.candidate.locator)
                        .collect::<Vec<_>>(),
                    [locator(99)]
                );
                start(&mut r, 2);
                feed(&mut r, &observation(2, "/alias"), &mut events);
                assert!(finish(&mut r, true, &mut events).complete);
                assert_eq!(
                    r.offers()
                        .unwrap()
                        .map(|o| o.candidate.locator)
                        .collect::<Vec<_>>(),
                    [locator(2)]
                );
            }
        }
    }
}

/// Scenario: Two owner-associated candidates contradict the same predecessor in either order.
/// Guarantees: Both the overwritten and retained qualified targets are withheld while an unrelated candidate remains offered.
#[test]
fn conflicting_associations_withhold_both_current_targets() {
    for reverse in [false, true] {
        let mut r = Reconciler::new(limits()).unwrap();
        r.track(file(1, "/bound")).unwrap();
        let mut events = Vec::new();
        start(&mut r, 1);
        for (inode, path) in [(2, "/b"), (3, "/c"), (99, "/independent")] {
            feed(&mut r, &observation(inode, path), &mut events);
        }
        for inode in if reverse { [3, 2] } else { [2, 3] } {
            r.associate_replacement(1, TrackedId(1), locator(inode), |e| record(e, &mut events))
                .unwrap();
        }
        assert!(!finish(&mut r, true, &mut events).complete);
        assert_eq!(
            r.offers()
                .unwrap()
                .map(|o| o.candidate.locator)
                .collect::<Vec<_>>(),
            [locator(99)]
        );
    }
}

/// Scenario: A locator-less binding failure precedes or follows qualification, with the candidate's smaller alias arriving first or last.
/// Guarantees: Final filtering blocks current qualified targets even when their candidate arrives after the failure.
#[test]
fn binding_failure_blocks_qualified_targets_in_all_orders() {
    for reason in [
        ScanReason::Unstable,
        ScanReason::Policy,
        ScanReason::VerificationUnavailable,
    ] {
        for failure_first in [false, true] {
            for alias_first in [false, true] {
                let mut r = Reconciler::new(limits()).unwrap();
                r.track(file(1, "/bound")).unwrap();
                let mut events = Vec::new();
                start(&mut r, 1);
                if alias_first {
                    feed(&mut r, &observation(2, "/alias"), &mut events);
                }
                for failure in [failure_first, !failure_first] {
                    let o = if failure {
                        evidence(None, "/bound", reason)
                    } else {
                        observation(2, "/bound")
                    };
                    feed(&mut r, &o, &mut events);
                }
                if !alias_first {
                    feed(&mut r, &observation(2, "/alias"), &mut events);
                }
                assert!(!finish(&mut r, true, &mut events).complete);
                assert!(r.offers().unwrap().next().is_none());
            }
        }
    }
}

/// Scenario: A weak foreign name at a failed binding also has an independent full alias, with or without subsequent binding qualification.
/// Guarantees: Weak-only targets do not suppress independent work; upgrading the target makes it subject to the current binding conflict.
#[test]
fn weak_target_only_blocks_independent_alias_after_qualification() {
    for upgrade in [false, true] {
        for weak_reason in [ScanReason::Content, ScanReason::PresenceOnly] {
            let mut r = Reconciler::new(limits()).unwrap();
            r.track(file(1, "/bound")).unwrap();
            let mut events = Vec::new();
            start(&mut r, 1);
            feed(
                &mut r,
                &evidence(Some(2), "/bound", weak_reason),
                &mut events,
            );
            feed(
                &mut r,
                &evidence(None, "/bound", ScanReason::Unstable),
                &mut events,
            );
            feed(&mut r, &observation(2, "/alias"), &mut events);
            if upgrade {
                feed(&mut r, &observation(2, "/bound"), &mut events);
            }
            assert!(!finish(&mut r, true, &mut events).complete);
            assert_eq!(r.offers().unwrap().count(), usize::from(!upgrade));
        }
    }
}

/// Scenario: An earlier alias is probed successfully, then its parent is renamed at the pre-recheck cancellation boundary.
/// Guarantees: The real scanner reports instability rather than a definite miss, preserving the alias hint and preventing prompt or complete-pass exclusion.
#[test]
fn alias_parent_disappearance_after_probe_preserves_uncertainty() {
    use super::super::{BindingScope, MatchPolicy, ProbeBudget, ScanConfig, ScanLimits, Scanner};
    use std::cell::Cell;
    struct ProbeObserved<'a> {
        budget: &'a ProbeBudget,
        observed: &'a Cell<bool>,
    }
    impl MatchPolicy for ProbeObserved<'_> {
        fn includes(&self, _: &Path) -> bool {
            true
        }
        fn may_descend(&self, _: &Path) -> bool {
            true
        }
        fn excludes(&self, _: &Path) -> bool {
            // Policy evaluation while the source-probe reservation is held proves
            // that the probe reached a pinned source, not just lexical filtering.
            if self.budget.try_reserve().is_none() {
                self.observed.set(true);
            }
            false
        }
        fn excludes_subtree(&self, _: &Path) -> bool {
            false
        }
        fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
            false
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let parent = dir.path().join("aliases");
    let moved = dir.path().join("moved");
    std::fs::create_dir(&parent).unwrap();
    let alias = parent.join("a.log");
    let binding = dir.path().join("a.secret");
    std::fs::write(&alias, b"abc").unwrap();
    std::fs::hard_link(&alias, &binding).unwrap();
    let actual = FileLocator::from_metadata(&std::fs::metadata(&alias).unwrap());
    let mut tracked = file(1, binding.to_str().unwrap());
    tracked.locator = actual;
    let mut r = Reconciler::new(limits()).unwrap();
    r.track(tracked).unwrap();
    let budget = ProbeBudget::default();
    let mut scanner = scanner(dir.path());
    let mut events = Vec::new();
    start(&mut r, 1);
    scanner
        .check_binding(
            &alias,
            None,
            &budget,
            || false,
            |check| {
                assert!(check.observation.reason.is_none());
                feed(&mut r, check.observation, &mut events);
            },
        )
        .unwrap();
    assert!(finish(&mut r, true, &mut events).complete);
    start(&mut r, 2);
    scanner
        .check_binding(
            &binding,
            None,
            &budget,
            || false,
            |check| {
                assert_eq!(check.observation.reason, Some(ScanReason::Excluded));
                feed(&mut r, check.observation, &mut events);
            },
        )
        .unwrap();
    assert!(!events.contains(&"excluded:1".into()));
    let observed = Cell::new(false);
    let renamed = Cell::new(false);
    let mut scanner = Scanner::new(
        ScanConfig {
            include_roots: vec![dir.path().to_owned()],
            exclusion_roots: vec![],
            symlinks: super::super::super::source::SymlinkPolicy::Reject,
            limits: ScanLimits {
                fingerprint_bytes: 16,
                ..ScanLimits::default()
            },
            checkpoint_namespace: locator(u64::MAX),
        },
        ProbeObserved {
            budget: &budget,
            observed: &observed,
        },
    )
    .unwrap();
    let mut batch = scanner
        .binding_batch(&budget, || {
            // The reservation is released when probe returns. Rename at this exact
            // boundary, before the scanner reopens the parent, without a timing race.
            if observed.get() && !renamed.get() && budget.try_reserve().is_some() {
                std::fs::rename(&parent, &moved).unwrap();
                renamed.set(true);
            }
            false
        })
        .unwrap();
    let request = r.binding_request(TrackedId(1)).unwrap();
    assert_eq!(request.path, alias);
    let check = batch.check(request.path, request.parent).unwrap();
    assert!(observed.get() && renamed.get());
    assert_eq!(check.scope, BindingScope::Binding);
    assert_eq!(check.observation.reason, Some(ScanReason::Unstable));
    assert_eq!(check.observation.errno, Some(libc::ENOENT));
    r.apply_binding_check(2, TrackedId(1), check, Duration::from_secs(20), |e| {
        record(e, &mut events)
    })
    .unwrap();
    assert!(events.contains(&"coverage".into()));
    assert!(!events.contains(&"excluded:1".into()));
    assert!(!finish(&mut r, true, &mut events).complete);
    assert!(!events.contains(&"excluded:1".into()));
    assert_eq!(r.tracked[&TrackedId(1)].previous_alias, alias);
    assert!(budget.try_reserve().is_some());
}
