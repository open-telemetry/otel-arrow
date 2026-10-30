// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Standalone admission/churn measurements, not integrated receiver qualification.
//! FILELOG_IDENTITY_MODE=bulk|churn, FILELOG_IDENTITY_FILES=10000,
//! FILELOG_IDENTITY_WIDTH=1000, FILELOG_IDENTITY_HEAP=1 (separate from timing).

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(not(target_os = "linux"))]
fn main() {}

#[cfg(target_os = "linux")]
fn main() {
    use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{
        config::Config,
        discovery::ProbeBudget,
        identity::{Admission, Candidate, IdentityOwner, decision::Context, lease::LeaseRegistry},
        source::{FileLocator, SourceFile, SymlinkPolicy},
    };
    use otel_arrow_dfe_filelog_checkpoint::{
        self as cp,
        store::{CheckpointStore, StateRoot},
    };
    use std::{
        ffi::CString,
        fs::{self, File},
        hint::black_box,
        io::Write,
        os::{
            fd::AsFd,
            unix::{ffi::OsStrExt, fs::PermissionsExt},
        },
        time::{Duration, Instant, SystemTime},
    };

    let count = std::env::var("FILELOG_IDENTITY_FILES")
        .map_or(10000, |v| v.parse().expect("integer benchmark parameter"));
    let width = std::env::var("FILELOG_IDENTITY_WIDTH")
        .map_or(1000, |v| v.parse().expect("integer benchmark parameter"));
    let mode = std::env::var("FILELOG_IDENTITY_MODE").unwrap_or_else(|_| "churn".into());
    assert!(matches!(mode.as_str(), "bulk" | "churn"));
    assert!((16..=65535).contains(&width));
    let heap = std::env::var_os("FILELOG_IDENTITY_HEAP").is_some();
    let dir = tempfile::tempdir().expect("temporary fixture directory");
    let source = tempfile::tempdir().expect("temporary fixture directory");
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700))
        .expect("benchmark setup or admission");
    let root = StateRoot::from_borrowed_fd(
        File::open(dir.path())
            .expect("benchmark setup or admission")
            .as_fd(),
    )
    .expect("benchmark setup or admission");
    let mut config = Config {
        include: vec![
            source
                .path()
                .join("*")
                .to_str()
                .expect("UTF-8 temporary path")
                .into(),
        ],
        ..Config::default()
    };
    config.identity.fingerprint_bytes = width as u64;
    config.limits.max_tracked_files = count as u64 + 100;
    let inputs = config
        .compile("g", "p", "identity-bench", source.path())
        .expect("benchmark setup or admission")
        .into_parts()
        .0;
    let store = CheckpointStore::open(
        &root,
        inputs.checkpoint_id(),
        inputs.store_config().expect("validated store settings"),
    )
    .expect("benchmark setup or admission");
    drop(store);
    // Late-diverging prefixes exercise full comparisons. All fixture allocation
    // and source creation precedes profiling. Store recovery and admission do not.
    let prefix = |id: usize| {
        let mut bytes = vec![b'a'; width];
        bytes[width - 8..].copy_from_slice(&(id as u64).to_be_bytes());
        bytes
    };
    if mode == "churn" {
        let records: Vec<_> = (0..count)
            .map(|id| cp::SnapshotRecord {
                file_id: cp::FileId::from_bytes((id as u128).to_be_bytes()),
                file_epoch: 1,
                committed_offset: 0,
                committed_frontier_guard: cp::CommittedFrontierGuard::empty(),
                fingerprint: prefix(id),
                ignored_header_bytes: 0,
                locator: cp::Locator::PosixDevIno {
                    dev: 0,
                    ino: id as u64,
                },
                framing_profile_version: cp::FRAMING_PROFILE_VERSION,
                framing_profile_digest: inputs.profile().digest().expect("profile digest"),
                framing_resume: cp::FramingResume::Clean,
                lifecycle_state: cp::LifecycleState::Active,
                quarantine_evidence: None,
                last_seen_time_unix_nano: 1,
                advisory_path: cp::AdvisoryPath::from_unix_bytes(format!("/saved/{id}").as_bytes())
                    .expect("benchmark setup or admission"),
            })
            .collect();
        let snapshot = cp::encode_checkpoint(1, inputs.checkpoint_id(), &records)
            .expect("benchmark setup or admission");
        fs::write(
            dir.path()
                .join("filelog/@v1")
                .join(inputs.namespace_component())
                .join("checkpoint.db"),
            snapshot,
        )
        .expect("benchmark setup or admission");
    }
    let attempts = if mode == "bulk" { count } else { 100 };
    let fixtures: Vec<_> = (0..attempts)
        .map(|id| {
            let path = source.path().join(format!("candidate-{id}"));
            let bytes = prefix(count + id);
            fs::write(&path, &bytes).expect("benchmark setup or admission");
            let locator =
                FileLocator::from_metadata(&fs::metadata(&path).expect("source metadata"));
            (path, bytes, locator)
        })
        .collect();
    let profiler = heap.then(|| dhat::Profiler::builder().testing().build());
    let store = CheckpointStore::open(
        &root,
        inputs.checkpoint_id(),
        inputs.store_config().expect("validated store settings"),
    )
    .expect("benchmark setup or admission");
    let leases = LeaseRegistry::process()
        .owner(
            "identity-bench",
            IdentityOwner::required_lease_capacity(&inputs).expect("benchmark setup or admission"),
        )
        .expect("benchmark setup or admission");
    let mut owner =
        IdentityOwner::new(store, leases, &inputs).expect("benchmark setup or admission");
    let budget = ProbeBudget::default();
    let mut plans = Vec::new();
    let mut readers = Vec::new();
    let mut visits = 0usize;
    let mut compared = 0usize;
    let mut steps = 0usize;
    let mut longest_step = Duration::ZERO;
    let mut longest_prepare = Duration::ZERO;
    let mut longest_finish = Duration::ZERO;
    let mut longest_commit = Duration::ZERO;
    let started = Instant::now();
    for (index, (path, bytes, locator)) in fixtures.iter().enumerate() {
        let open = || {
            let parent = File::open(path.parent().expect("candidate parent"))
                .expect("benchmark setup or admission");
            let name = CString::new(path.file_name().expect("candidate filename").as_bytes())
                .expect("benchmark setup or admission");
            SourceFile::open_at(
                &parent,
                &name,
                SymlinkPolicy::Reject,
                Some(*locator),
                || false,
            )
        };
        let before = Instant::now();
        let mut result = owner
            .prepare(
                Candidate {
                    path,
                    locator: *locator,
                    size: width as u64,
                    fingerprint: bytes,
                    context: Context::Unrelated,
                },
                &budget,
                SystemTime::now(),
                open,
                || false,
            )
            .expect("benchmark setup or admission");
        longest_prepare = longest_prepare.max(before.elapsed());
        if let Admission::Scanning(mut scan) = result {
            while !scan.is_complete() {
                let before = Instant::now();
                let work = owner
                    .advance_association(&mut scan, || false)
                    .expect("association step");
                longest_step = longest_step.max(before.elapsed());
                assert!(work.records <= 64 && work.bytes <= 64 * 1024);
                visits += work.records;
                compared += work.bytes;
                steps += 1;
                // Represents the caller's opportunity to service another command.
                // Actual progress interleaving is covered by deterministic unit tests.
                let _ = black_box(owner.synced_sequence());
                assert!(budget.try_reserve().is_some());
            }
            let before = Instant::now();
            result = owner
                .finish_prepare(scan, &budget, SystemTime::now(), open, || false)
                .expect("benchmark setup or admission");
            longest_finish = longest_finish.max(before.elapsed());
        }
        let Admission::Ready(plan) = result else {
            panic!("unexpected {result:?}")
        };
        if mode == "bulk" {
            plans.push(plan);
            if plans.len() == owner.admission_limit() || index + 1 == attempts {
                let selected = owner
                    .select_batch(std::mem::take(&mut plans))
                    .expect("group capacity");
                assert!(selected.needs_capacity.is_empty());
                let before = Instant::now();
                readers.extend(owner.commit_batch(selected.ready).expect("durable group"));
                longest_commit = longest_commit.max(before.elapsed());
            }
        }
    }
    let elapsed = started.elapsed();
    let stats = profiler.as_ref().map(|_| dhat::HeapStats::get());
    drop(readers);
    drop(owner);
    drop(profiler);
    let mut output = std::io::stdout().lock();
    writeln!(output,
        "mode={mode} files={count} width={width} heap={heap} attempts={attempts} elapsed_ms={} visits={visits} compared_bytes={compared} steps={steps} max_step_us={} max_prepare_us={} max_finish_us={} max_commit_us={}",
        elapsed.as_millis(),
        longest_step.as_micros(),
        longest_prepare.as_micros(),
        longest_finish.as_micros(),
        longest_commit.as_micros()
    ).expect("measurement output");
    if let Some(stats) = stats {
        writeln!(
            output,
            "retained_bytes={} peak_bytes={} allocated_bytes={} allocations={}",
            stats.curr_bytes, stats.max_bytes, stats.total_bytes, stats.total_blocks
        )
        .expect("heap output");
    }
}
