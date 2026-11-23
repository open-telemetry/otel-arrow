// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::filelog_receiver::discovery::reconcile::PublishedReaderBinding;
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
) -> (usize, bool) {
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
    (resumes, summary.complete)
}

/// Scenario: After reading "hel", eviction and move/create stop the reader until its rotated file has a published binding.
/// Guarantees: Only a current accepted rebind resumes the original partial line, once, with its lease and progress intact; a stale key cannot redirect the replacement.
#[test]
fn validated_rename_resumes_partial_line_once() {
    for stop in [Transition::Replaced, Transition::Ineligible] {
        exercise_recovery(Recovery::Rename(stop));
    }
}

/// Scenario: The original file returns to app.log after a replacement parks its evicted reader.
/// Guarantees: Full evidence at the existing binding resumes the leased partial line once; weak and duplicate observations cannot schedule extra reads or mix in replacement bytes.
#[test]
fn validated_original_path_resumes_partial_line_once() {
    exercise_recovery(Recovery::Original);
}

#[derive(Clone, Copy)]
enum Recovery {
    Rename(Transition),
    Original,
    ChangedParent,
    RetryMissing,
    ParkAfterScan,
    OpenBeforeRebind,
}

/// Scenario: Discovery installs the rotated path after reopen but before the replacement handle's locator check.
/// Guarantees: The replacement supplies no bytes and the original partial line completes exactly once without another scan.
#[test]
fn published_rename_during_provisional_reopen() {
    exercise_recovery(Recovery::OpenBeforeRebind);
}

/// Scenario: An evicted reader sees ENOENT, then discovery publishes its renamed path.
/// Guarantees: The opener receives the path once and the existing retry finishes the original line without an extra ready entry.
#[test]
fn published_rename_reaches_running_retry() {
    exercise_recovery(Recovery::RetryMissing);
}

/// Scenario: An evicted reader parks after scan completion but before its published binding arrives.
/// Guarantees: The same publication still installs the new path and queues one read that finishes the original line.
#[test]
fn published_rename_reaches_reader_parked_after_scan() {
    exercise_recovery(Recovery::ParkAfterScan);
}

/// Scenario: The original locator returns to its live path under a replacement parent directory.
/// Guarantees: Positive evidence resumes the original partial line while the parent latch still blocks a complete pass and binding publication.
#[test]
fn validated_original_path_resumes_with_parent_change_latched() {
    exercise_recovery(Recovery::ChangedParent);
}

fn exercise_recovery(recovery: Recovery) {
    let restore_original = matches!(recovery, Recovery::Original | Recovery::ChangedParent);
    let stop = match recovery {
        Recovery::Rename(reason) => reason,
        _ => Transition::Replaced,
    };
    let f = Fixture::new();
    let inputs = f.inputs(|c| {
        c.limits.max_open_files = 1;
        if matches!(recovery, Recovery::RetryMissing) {
            // Let the retry fire before the peer's next EOF probe.
            c.reader.eof_reprobe_interval = Duration::from_secs(1);
        }
    });
    let mut owner = f.owner(&inputs);
    let now = Instant::now();
    let paths = Rc::new(RefCell::new(HashMap::<ReaderKey, PathBuf>::new()));
    let names = Rc::clone(&paths);
    let pause_after_open = Rc::new(Cell::new(false));
    let pause = Rc::clone(&pause_after_open);
    let io = LinuxIo(move |key: ReaderKey, _: &mut dyn FnMut() -> bool| {
        let handle = open(&names.borrow()[&key])?;
        pause.set(true);
        Ok(handle)
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
    // Keep the peer asleep across the provisional handle check.
    let peer_at = if matches!(recovery, Recovery::OpenBeforeRebind) {
        now + Duration::from_millis(1)
    } else {
        now
    };
    read_to_eof(&mut scheduler, &mut sink, peer_key, peer_at);
    let rotated = app.with_extension("log.1");
    fs::rename(&app, &rotated).unwrap();
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
    let at = scheduler.next_deadline(READ, &sink).unwrap();
    if matches!(
        recovery,
        Recovery::RetryMissing | Recovery::ParkAfterScan | Recovery::OpenBeforeRebind
    ) {
        let running = matches!(recovery, Recovery::RetryMissing);
        if running {
            let turn = scheduler.turn(at, READ, &mut sink, Waker::noop(), || false);
            assert_eq!(turn.key, Some(key));
            assert!(
                matches!(turn.reason, Reason::Retry(error) if error.kind() == std::io::ErrorKind::NotFound)
            );
            read_to_eof(&mut scheduler, &mut sink, peer_key, at);
        }
        assert_eq!(
            scan_directory(&mut scan, &mut scheduler, key, f.source.path(), &app, 1),
            (0, true)
        );
        let published = publish_rename(&mut scan, 1, &mut owner, id, &rotated);
        let provisional = matches!(recovery, Recovery::OpenBeforeRebind);
        if provisional {
            fs::write(&app, b"replacement bytes\n").unwrap();
            pause_after_open.set(false);
            let turn = scheduler.turn(at, READ, &mut sink, Waker::noop(), || {
                pause_after_open.get()
            });
            assert_eq!(turn.key, Some(key));
            assert!(matches!(turn.reason, Reason::Cancelled));
            assert_eq!(turn.bytes, 0);
            assert_eq!(scheduler.pin(key), Err(LifecycleRefusal::UnvalidatedHandle));
        } else if !running {
            fs::write(&app, b"replacement bytes\n").unwrap();
            let turn = scheduler.turn(at, READ, &mut sink, Waker::noop(), || false);
            assert_eq!(turn.key, Some(key));
            assert!(matches!(
                turn.reason,
                Reason::Transition(Transition::Replaced)
            ));
            read_to_eof(&mut scheduler, &mut sink, peer_key, at);
        }
        let deadline = scheduler.next_deadline(READ, &sink);
        assert_eq!(
            scheduler
                .resume_after_rebind(key, &published, |key, path| {
                    path.clone_into(paths.borrow_mut().get_mut(&key).unwrap());
                    true
                })
                .unwrap(),
            !running && !provisional
        );
        assert_eq!(paths.borrow()[&key], rotated);
        assert!(
            !scheduler
                .resume_after_rebind(key, &published, |_, _| panic!("already installed"))
                .unwrap()
        );
        append_suffix(&rotated);
        if provisional {
            let turn = scheduler.turn(at, READ, &mut sink, Waker::noop(), || false);
            assert_eq!(turn.key, Some(key));
            assert!(matches!(
                turn.reason,
                Reason::Transition(Transition::Replaced)
            ));
            assert_eq!(turn.bytes, 0);
        }
        let read_at = if running {
            assert_eq!(scheduler.next_deadline(READ, &sink), deadline);
            assert!(matches!(
                scheduler
                    .turn(at, READ, &mut sink, Waker::noop(), || false)
                    .reason,
                Reason::Idle
            ));
            deadline.unwrap()
        } else {
            at
        };
        read_to_eof(&mut scheduler, &mut sink, key, read_at);
        assert!(matches!(
            scheduler
                .turn(read_at, READ, &mut sink, Waker::noop(), || false)
                .reason,
            Reason::Idle
        ));
        assert_hello(&mut sink, id, epoch);
        return;
    }
    if stop == Transition::Ineligible {
        fs::create_dir(&app).unwrap();
    } else {
        fs::write(&app, b"different file\n").unwrap();
    }
    let turn = scheduler.turn(at, READ, &mut sink, Waker::noop(), || false);
    assert_eq!(turn.key, Some(key));
    assert!(matches!(turn.reason, Reason::Transition(actual) if actual == stop));
    read_to_eof(&mut scheduler, &mut sink, peer_key, at);

    if restore_original {
        fs::rename(&rotated, &app).unwrap();
    }
    scan.begin(1, Duration::ZERO, 0).unwrap();
    if matches!(recovery, Recovery::ChangedParent) {
        let directory = Observation {
            path: f.source.path().into(),
            locator: Some(FileLocator::from_metadata(
                &fs::metadata(f.source.path()).unwrap(),
            )),
            size: None,
            modified: None,
            fingerprint: vec![],
            reason: Some(ScanReason::Directory),
            errno: None,
        };
        scan.observe(1, &directory, Duration::ZERO, |_| {}).unwrap();
    }
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
    let _ = finish_pass(
        &mut scan,
        &mut scheduler,
        key,
        1,
        if matches!(recovery, Recovery::ChangedParent) {
            2
        } else {
            1
        },
        true,
    );
    assert!(
        scan.rebind_for_reader(1, TRACKED, |_| panic!("weak evidence reached commit"))
            .is_err()
    );
    if restore_original {
        let archived_parent = tempfile::tempdir().unwrap();
        let complete = !matches!(recovery, Recovery::ChangedParent);
        if !complete {
            let saved = archived_parent.path().join("old");
            fs::rename(f.source.path(), &saved).unwrap();
            fs::create_dir(f.source.path()).unwrap();
            fs::rename(saved.join("app.log"), &app).unwrap();
        }
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
            (1, complete)
        );
        assert_eq!(
            scan_directory(&mut scan, &mut scheduler, key, f.source.path(), &app, 3),
            (0, complete)
        );
        if !complete {
            assert!(
                scan.rebind_for_reader(3, TRACKED, |_| panic!("parent latch allows publication"))
                    .is_err()
            );
        }
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
        (0, true)
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
    let published = publish_rename(&mut scan, 2, &mut owner, id, &rotated);
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
        (0, true)
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

fn publish_rename(
    scan: &mut Reconciler,
    generation: u64,
    owner: &mut IdentityOwner,
    id: cp::FileId,
    rotated: &Path,
) -> PublishedReaderBinding {
    scan.rebind_for_reader(generation, TRACKED, |path| {
        assert_eq!(path, rotated);
        let record = owner.state().unwrap().get(&id).unwrap();
        let update = cp::UpdateMetadata {
            file_id: id,
            expected_prior_state: cp::LifecycleState::Active,
            expected_file_epoch: record.file_epoch,
            last_seen_time_unix_nano: record.last_seen_time_unix_nano,
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
    .unwrap()
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
