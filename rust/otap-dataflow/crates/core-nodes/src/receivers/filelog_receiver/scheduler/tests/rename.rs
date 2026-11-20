// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_pdata::{
    otlp::{ProtoBuffer, ProtoBytesEncoder, logs::LogsProtoBytesEncoder},
    proto::opentelemetry::{
        collector::logs::v1::ExportLogsServiceRequest, common::v1::any_value::Value,
    },
};
use prost::Message;

fn scan_directory(
    scan: &mut Reconciler,
    scheduler: &mut Scheduler<impl ReaderIo>,
    key: ReaderKey,
    root: &Path,
    opener_path: &Path,
    generation: u64,
) -> usize {
    let at = Duration::from_secs(generation);
    scan.begin(generation, at, 0).unwrap();
    let mut resumes = 0;
    let completion = test_scanner(root).scan(
        generation,
        &ProbeBudget::default(),
        || false,
        |batch| {
            for observation in batch {
                scan.observe(generation, observation, at, |event| {
                    resumes += usize::from(
                        scheduler
                            .observe_discovery(key, TRACKED, &event, opener_path)
                            .unwrap(),
                    );
                })
                .unwrap();
            }
            true
        },
    );
    let summary = scan
        .finish(completion, at, |event| {
            assert!(
                !scheduler
                    .observe_discovery(key, TRACKED, &event, opener_path)
                    .unwrap()
            );
        })
        .unwrap();
    assert!(summary.complete);
    resumes
}

/// Scenario: After reading "hel", eviction and move/create stop the reader until its rotated file has a published binding.
/// Guarantees: Only a current accepted rebind resumes the original partial line, once, with its lease and progress intact; a stale key cannot redirect the replacement.
#[test]
fn validated_rename_resumes_partial_line_once() {
    for stop in [Transition::Replaced, Transition::Ineligible] {
        exercise_recovery(stop, false);
    }
}

/// Scenario: The original file returns to app.log after a replacement parks its evicted reader.
/// Guarantees: Full evidence at the existing binding resumes the leased partial line once; weak and duplicate observations cannot schedule extra reads or mix in replacement bytes.
#[test]
fn validated_original_path_resumes_partial_line_once() {
    exercise_recovery(Transition::Replaced, true);
}

fn exercise_recovery(stop: Transition, restore_original: bool) {
    let f = Fixture::new();
    let inputs = f.inputs(|c| c.limits.max_open_files = 1);
    let mut owner = f.owner(&inputs);
    let now = Instant::now();
    let paths = Rc::new(RefCell::new(HashMap::<ReaderKey, PathBuf>::new()));
    let names = Rc::clone(&paths);
    let io = LinuxIo(move |key: ReaderKey, _: &mut dyn FnMut() -> bool| {
        open(&names.borrow()[&key]).map_err(Into::into)
    });
    let mut scheduler = Scheduler::from_inputs(io, &inputs, 128, Arc::default(), now).unwrap();
    let mut sink = Batcher::new(&inputs, || now, || 1);
    let app = f.write("app.log", b"hel");
    let seed = commit(&mut owner, &app, Context::Unrelated);
    let (id, epoch, locator) = (seed.file_id, seed.epoch, seed.lease.locator());
    let key = admit(&mut scheduler, &mut sink, &paths, &app, seed);
    read_to_eof(&mut scheduler, &mut sink, key, now);
    let peer = f.write("peer.log", b"partial peer");
    let peer_seed = commit(&mut owner, &peer, Context::Unrelated);
    let peer_key = admit(&mut scheduler, &mut sink, &paths, &peer, peer_seed);
    read_to_eof(&mut scheduler, &mut sink, peer_key, now);
    let rotated = app.with_extension("log.1");
    fs::rename(&app, &rotated).unwrap();
    if stop == Transition::Ineligible {
        fs::create_dir(&app).unwrap();
    } else {
        fs::write(&app, b"different file\n").unwrap();
    }
    let at = scheduler.next_deadline(READ, &sink).unwrap();
    let turn = scheduler.turn(at, READ, &mut sink, Waker::noop(), || false);
    assert_eq!(turn.key, Some(key));
    assert!(matches!(turn.reason, Reason::Transition(actual) if actual == stop));
    read_to_eof(&mut scheduler, &mut sink, peer_key, at);

    let mut scan = Reconciler::new(ReconcileLimits {
        tracked: 3,
        pending: 3,
        offers: 3,
        path_bytes: 1024,
        fingerprint_bytes: 16,
    })
    .unwrap();
    scan.track(TrackedFile {
        id: TRACKED,
        locator,
        binding: Some(app.clone()),
        role: BindingRole::Live,
        active: true,
        resident: true,
    })
    .unwrap();
    if restore_original {
        fs::rename(&rotated, &app).unwrap();
    }
    scan.begin(1, Duration::ZERO, 0).unwrap();
    let weak = Observation {
        path: if restore_original {
            app.clone()
        } else {
            rotated.clone()
        },
        locator: Some(locator),
        size: None,
        modified: None,
        fingerprint: vec![],
        reason: Some(ScanReason::PresenceOnly),
        errno: None,
    };
    scan.observe(1, &weak, Duration::ZERO, |event| {
        assert!(
            !scheduler
                .observe_discovery(key, TRACKED, &event, &app)
                .unwrap()
        );
    })
    .unwrap();
    let _ = finish_pass(&mut scan, &mut scheduler, key, 1, 1, true);
    assert!(
        scan.rebind_for_reader(1, TRACKED, |_| panic!("weak evidence reached commit"))
            .is_err()
    );
    if restore_original {
        let wal = owner.wal_bytes();
        let before = owner.state().unwrap().get(&id).unwrap().clone();
        assert!(matches!(
            scheduler
                .turn(at, READ, &mut sink, Waker::noop(), || false)
                .reason,
            Reason::Idle
        ));
        assert_eq!(
            scan_directory(&mut scan, &mut scheduler, key, f.source.path(), &app, 2),
            1
        );
        assert_eq!(
            scan_directory(&mut scan, &mut scheduler, key, f.source.path(), &app, 3),
            0
        );
        append_suffix(&app);
        read_to_eof(&mut scheduler, &mut sink, key, at);
        assert!(matches!(
            scheduler
                .turn(at, READ, &mut sink, Waker::noop(), || false)
                .reason,
            Reason::Idle
        ));
        assert_hello(&mut sink, id, epoch);
        assert_eq!(owner.wal_bytes(), wal);
        assert_eq!(owner.state().unwrap().get(&id), Some(&before));
        assert_eq!(f.registry.len().unwrap(), 2);
        return;
    }
    append_suffix(&rotated);
    if stop == Transition::Ineligible {
        fs::remove_dir(&app).unwrap();
        fs::write(&app, b"different file\n").unwrap();
    }
    assert_eq!(
        scan_directory(&mut scan, &mut scheduler, key, f.source.path(), &app, 2),
        0
    );

    // B's admission resolves the old path rebound independently of A.
    let b = scan
        .offers()
        .unwrap()
        .find(|offer| offer.candidate.path == app)
        .unwrap();
    assert_eq!(b.candidate.replacement, Replacement::Possible(TRACKED));
    let plan = ready(prepare(&mut owner, &b.candidate.path, Context::Replacement).unwrap());
    let mut replacement = None;
    assert!(
        scan.admit(
            b.token,
            TrackedFile {
                id: TrackedId(2),
                locator: b.candidate.locator,
                binding: Some(app.clone()),
                role: BindingRole::Live,
                active: true,
                resident: true
            },
            |_| {
                let CommittedAdmission::Reader(seed) = owner.commit(plan).unwrap() else {
                    panic!("expected replacement admission")
                };
                assert_ne!(seed.file_id, id);
                replacement = Some(seed);
                Some(Duration::from_secs(2))
            }
        )
        .unwrap()
    );
    let before = owner.state().unwrap().get(&id).unwrap().clone();
    assert!(
        scan.rebind_for_reader(2, TRACKED, |_| false)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        scheduler
            .turn(at, READ, &mut sink, Waker::noop(), || false)
            .reason,
        Reason::Idle
    ));

    let expected_path = cp::AdvisoryPath::from_unix_bytes(rotated.as_os_str().as_bytes()).unwrap();
    let published = scan
        .rebind_for_reader(2, TRACKED, |path| {
            assert_eq!(path, rotated);
            let update = cp::UpdateMetadata {
                file_id: id,
                expected_prior_state: cp::LifecycleState::Active,
                expected_file_epoch: epoch,
                last_seen_time_unix_nano: before.last_seen_time_unix_nano,
                advisory_path: Some(
                    cp::AdvisoryPath::from_unix_bytes(path.as_os_str().as_bytes()).unwrap(),
                ),
            };
            let _ = owner
                .append_checkpoint(
                    vec![cp::Operation::UpdateMetadata(update)],
                    Durability::Sync,
                )
                .unwrap();
            true
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        scan.tracked(TRACKED).unwrap().binding.as_ref(),
        Some(&rotated)
    );
    let mut expected_record = before;
    expected_record.advisory_path = expected_path;
    assert_eq!(owner.state().unwrap().get(&id), Some(&expected_record));
    let wal = owner.wal_bytes();
    assert_eq!(
        scheduler.resume_after_rebind(key, &published, |_, _| false),
        Err(LifecycleRefusal::BindingNotInstalled)
    );
    assert_eq!(paths.borrow()[&key], app);

    // A later completed pass supersedes a delayed notification, even for A.
    assert_eq!(
        scan_directory(&mut scan, &mut scheduler, key, f.source.path(), &app, 3),
        0
    );
    assert!(
        !scheduler
            .resume_after_rebind(key, &published, |_, _| panic!("stale publication"))
            .unwrap()
    );
    let published = scan
        .rebind_for_reader(3, TRACKED, |path| {
            // The same binding already has a durable update.
            cp::AdvisoryPath::from_unix_bytes(path.as_os_str().as_bytes()).unwrap()
                == expected_record.advisory_path
        })
        .unwrap()
        .unwrap();
    assert!(
        scheduler
            .resume_after_rebind(key, &published, |key, path| {
                sink.update_metadata(key, &expected_record.advisory_path)
                    .unwrap();
                path.clone_into(paths.borrow_mut().get_mut(&key).unwrap());
                true
            })
            .unwrap()
    );
    assert!(
        !scheduler
            .resume_after_rebind(key, &published, |_, _| panic!("duplicate publication"))
            .unwrap()
    );
    read_to_eof(&mut scheduler, &mut sink, key, at);
    assert!(matches!(
        scheduler
            .turn(at, READ, &mut sink, Waker::noop(), || false)
            .reason,
        Reason::Idle
    ));
    assert_hello(&mut sink, id, epoch);
    assert_eq!(owner.wal_bytes(), wal);
    assert_eq!(owner.state().unwrap().get(&id), Some(&expected_record));
    assert_eq!(f.registry.len().unwrap(), 3);

    let mut delivery = Delivery::new(sink, &inputs).unwrap();
    acknowledge(&mut delivery, at);
    assert!(matches!(
        delivery.checkpoint(&mut owner, at).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    scheduler
        .observe_applied(key, owner.state().unwrap().get(&id).unwrap())
        .unwrap();
    let sink = delivery.builder_mut(at).unwrap();
    scheduler.retire(key, sink).unwrap().release().unwrap();
    let new_key = admit(&mut scheduler, sink, &paths, &app, replacement.unwrap());
    assert_ne!(new_key, key);
    assert_eq!(
        scheduler.resume_after_rebind(key, &published, |_, _| panic!("stale reader key")),
        Err(LifecycleRefusal::UnknownReader)
    );
}

fn append_suffix(path: &Path) {
    OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap()
        .write_all(b"lo\n")
        .unwrap();
}

fn assert_hello(sink: &mut Batcher<impl Fn() -> Instant>, id: cp::FileId, epoch: u32) {
    sink.builder.seal().unwrap();
    let batch = sink.builder.retained().unwrap();
    assert_eq!(batch.record_count, 1);
    assert_eq!(batch.progress.len(), 1);
    assert_eq!(batch.progress[0].file_id, id);
    assert_eq!(batch.progress[0].expected_file_epoch, epoch);
    assert_eq!(batch.progress[0].expected_committed_offset, 0);
    assert_eq!(batch.progress[0].new_committed_offset, 6);
    let mut bytes = ProtoBuffer::default();
    LogsProtoBytesEncoder::new()
        .encode(&mut batch.records.clone(), &mut bytes)
        .unwrap();
    let request = ExportLogsServiceRequest::decode(bytes.as_ref()).unwrap();
    let logs = &request.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(logs.len(), 1);
    assert_eq!(
        logs[0].body.as_ref().unwrap().value,
        Some(Value::StringValue("hello".into()))
    );
}
