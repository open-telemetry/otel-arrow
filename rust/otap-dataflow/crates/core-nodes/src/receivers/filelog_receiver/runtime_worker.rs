// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    config::PathPolicy,
    delivery::{Attempt, CheckpointOutcome, Correlation, Delivery, SendOutcome},
    discovery::{Observation, ProbeBudget, ScanReason, Scanner, VerificationError},
    identity::{
        Admission, Candidate, CommittedAdmission, IdentityError, IdentityOwner, ReaderSeed,
        decision::Context, lease::LeaseRegistry,
    },
    scheduler::{Permission, Reason, Scheduler, batch::Batcher, io::LinuxIo},
    source::{FileAccessError, FileLocator, SourceFile, SymlinkPolicy},
    startup::WorkerInputs,
    workers::{Cancellation, Discovery, Input, ReadCheckpoint},
};
use otel_arrow_dfe_engine::control::CallData;
use std::{
    ffi::CString,
    fs::File,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    sync::Arc,
    task::Waker,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::oneshot;

pub(super) type Failure = Box<dyn std::error::Error + Send + Sync>;
const SCHEDULER_STEPS_PER_TURN: usize = 64;
const DISCOVERY_WARNING_INTERVAL: Duration = Duration::from_secs(60);

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = "urn:otel:receiver:filelog",
    target = "otel.receiver.filelog",
);

#[derive(PartialEq, Eq)]
pub(super) enum WorkerExit {
    Drained,
    Stopped,
}

pub(super) enum Command {
    Drain,
    #[cfg(test)]
    Inspect(flume::Sender<(u64, usize)>),
    Sent(Correlation, SendOutcome),
    Completion(CallData, bool),
}
pub(super) struct Source {
    path: PathBuf,
    locator: FileLocator,
    size: u64,
    fingerprint: Vec<u8>,
}

impl Source {
    fn open(&self, cancel: &Cancellation) -> Result<SourceFile, FileAccessError> {
        let parent = self.path.parent().ok_or(FileAccessError::InvalidName)?;
        let name = CString::new(
            self.path
                .file_name()
                .ok_or(FileAccessError::InvalidName)?
                .as_bytes(),
        )
        .map_err(|_| FileAccessError::InvalidName)?;
        use rustix::fs::{CWD, Mode, OFlags, ResolveFlags, openat2};
        let directory = openat2(
            CWD,
            parent,
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS,
        )
        .map_err(std::io::Error::from)?;
        SourceFile::open_at(
            &File::from(directory),
            &name,
            SymlinkPolicy::Reject,
            Some(self.locator),
            || cancel.is_cancelled(),
        )
    }
}

enum AdmissionReply {
    Rescan,
    Committed,
}

pub(super) struct Offer {
    source: Source,
    reply: oneshot::Sender<AdmissionReply>,
}

pub(super) fn discover(
    mut scanner: Scanner<PathPolicy>,
    channels: Discovery<Offer>,
    budget: Arc<ProbeBudget>,
    cancel: Cancellation,
    interval: Duration,
    file: &std::path::Path,
) {
    let mut generation = 0u64;
    let mut last_warning: Option<Instant> = None;
    while !cancel.is_cancelled() {
        generation += 1;
        let observed = lookup_literal(&mut scanner, &budget, &cancel, file);
        if cancel.is_cancelled() {
            return;
        }
        if let Ok(source) = observed {
            let (reply, result) = oneshot::channel();
            if let Err(error) = channels.observe(Offer { source, reply }) {
                if !cancel.is_cancelled() {
                    let detail = error.to_string();
                    otel_warn!(
                        "filelog_receiver.observation_channel_closed",
                        error = detail.as_str(),
                        message = "Filelog discovery could not hand off a verified observation"
                    );
                }
                return;
            }
            let decision = futures::executor::block_on(async {
                tokio::select! {
                    biased;
                    _ = cancel.cancelled() => None,
                    result = result => result.ok(),
                }
            });
            match decision {
                Some(AdmissionReply::Committed) => {
                    drop(scanner);
                    // Returning from either worker cancels its sibling.
                    futures::executor::block_on(cancel.cancelled());
                    return;
                }
                Some(AdmissionReply::Rescan) => {}
                None => return,
            }
        } else if let Err((reason, errno)) = observed
            && (reason.invalidates_coverage()
                || matches!(
                    reason,
                    ScanReason::Content | ScanReason::Unsupported | ScanReason::LeafLoop
                ))
            && reason != ScanReason::Cancelled
            && last_warning.is_none_or(|last| last.elapsed() >= DISCOVERY_WARNING_INTERVAL)
        {
            last_warning = Some(Instant::now());
            let detail = format!("{reason:?}");
            otel_warn!(
                "filelog_receiver.discovery_failed",
                generation = generation,
                reason = detail.as_str(),
                errno = i64::from(errno.unwrap_or(0)),
                message = "Filelog discovery could not verify or read its configured source"
            );
        }
        if cancel.wait(interval).is_err() {
            return;
        }
    }
}

// A literal include needs one verified lookup, regardless of sibling count or enumeration order.
fn lookup_literal(
    scanner: &mut Scanner<PathPolicy>,
    budget: &ProbeBudget,
    cancel: &Cancellation,
    file: &std::path::Path,
) -> Result<Source, (ScanReason, Option<i32>)> {
    scanner.check_verification().map_err(|e| {
        let reason = if matches!(e, VerificationError::Descriptors(..)) {
            ScanReason::DescriptorExhausted
        } else {
            ScanReason::VerificationUnavailable
        };
        (reason, e.source_error().raw_os_error())
    })?;
    let mut bindings = scanner
        .binding_batch(budget, || cancel.is_cancelled())
        .map_err(|e| (e.reason, e.errno))?;
    let check = bindings
        .check(file, None)
        .map_err(|_| (ScanReason::Limit, None))?;
    match check.observation {
        Observation {
            reason: None,
            path,
            locator: Some(locator),
            size: Some(size),
            fingerprint,
            ..
        } => Ok(Source {
            path: path.clone(),
            locator: *locator,
            size: *size,
            fingerprint: fingerprint.clone(),
        }),
        observation => Err((
            observation.reason.unwrap_or(ScanReason::Coverage),
            observation.errno,
        )),
    }
}

pub(super) type ScannerSetup = (Scanner<PathPolicy>, Duration);
pub(super) fn run(
    startup: WorkerInputs,
    mut endpoint: ReadCheckpoint<Offer, Attempt, Command>,
    scanner_tx: oneshot::Sender<ScannerSetup>,
    ready: oneshot::Sender<()>,
    budget: Arc<ProbeBudget>,
    cancel: Cancellation,
    #[cfg(test)] probe: Option<super::runtime_tests::Probe>,
) -> Result<WorkerExit, Failure> {
    let WorkerInputs {
        settings,
        store,
        scanner,
    } = startup;
    let leases = LeaseRegistry::process().owner(
        "filelog",
        IdentityOwner::required_lease_capacity(&settings)?,
    )?;
    let mut owner = IdentityOwner::new(store, leases, &settings)?;
    let batcher = Batcher::new(&settings, Instant::now, || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .min(u64::MAX.into()) as u64
    });
    let mut delivery = Delivery::new(batcher, &settings)?;
    let binding = std::cell::OnceCell::<Source>::new();
    let io = LinuxIo(|_, _: &mut dyn FnMut() -> bool| {
        binding
            .get()
            .ok_or(FileAccessError::InvalidName)?
            .open(&cancel)
            .map_err(Into::into)
    });
    let mut scheduler = Scheduler::from_inputs(
        io,
        &settings,
        SCHEDULER_STEPS_PER_TURN,
        Arc::clone(&budget),
        Instant::now(),
    )
    .map_err(|e| format!("scheduler setup: {e:?}"))?;
    let reconcile_interval = settings.settings().discovery.reconcile_interval;
    scanner_tx
        .send((scanner, reconcile_interval))
        .map_err(|_| "discovery worker stopped")?;
    ready.send(()).map_err(|_| "receiver stopped")?;
    let Some((source, seed)) = await_admission(
        &mut endpoint,
        &mut owner,
        &budget,
        &cancel,
        reconcile_interval,
        #[cfg(test)]
        probe.as_ref(),
    )?
    else {
        return Ok(WorkerExit::Drained);
    };
    let _ = binding.get_or_init(|| source);
    let id = seed.file_id;
    let record = owner.state()?.get(&id).ok_or("missing registered file")?;
    let key = scheduler
        .admit(seed)
        .map_err(|_| "reader admission failed")?;
    delivery
        .builder_mut(Instant::now())
        .ok_or("batcher unavailable")?
        .register(
            key,
            id,
            record.file_epoch,
            record.committed_offset,
            record.framing_resume,
            &record.advisory_path,
        )?;
    let permission = Permission {
        read: true,
        process: true,
        memory: true,
    };
    let mut draining = false;
    while !cancel.is_cancelled() {
        let now = Instant::now();
        let _ = delivery.sync(&mut owner, now)?;
        if let CheckpointOutcome::Applied(_) = delivery.checkpoint(&mut owner, now)? {
            scheduler
                .observe_applied(key, owner.state()?.get(&id).ok_or("missing checkpoint")?)
                .map_err(|e| format!("apply progress: {e:?}"))?;
            #[cfg(test)]
            if let Some(probe) = &probe {
                probe.applied(
                    owner
                        .state()?
                        .get(&id)
                        .ok_or("missing checkpoint")?
                        .committed_offset,
                );
            }
            scheduler.resume_sink();
        }
        let mut deadline = None;
        if let Some(sink) = delivery.builder_mut(now) {
            let turn = scheduler.turn(now, permission, sink, Waker::noop(), || {
                cancel.is_cancelled()
            });
            #[cfg(test)]
            if matches!(turn.reason, Reason::Eof)
                && let Some(probe) = &probe
            {
                probe.eof();
            }
            if draining && matches!(turn.reason, Reason::Idle | Reason::Paused) {
                sink.builder.seal()?;
                if sink.builder.retained().is_none() {
                    let _ = owner.sync_checkpoint()?;
                    return Ok(WorkerExit::Drained);
                }
            }
            deadline = match turn.reason {
                Reason::Progress | Reason::Eof => Some(now),
                Reason::Idle | Reason::Paused | Reason::Retry(_) | Reason::OpenBackoff => {
                    scheduler.next_deadline(permission, sink)
                }
                Reason::Cancelled => break,
                other => return Err(format!("stable file reader stopped: {other:?}").into()),
            };
        }
        // A drain never waits out a retry backoff to start another attempt.
        let retry_interrupted = draining && delivery.retry_deadline().is_some();
        if !retry_interrupted && let Some(attempt) = delivery.next_attempt(now)? {
            let mut command_error = None;
            endpoint.publish(attempt, |command| {
                if matches!(command, Command::Drain) {
                    scheduler.stop_source_reads();
                    draining = true;
                    #[cfg(test)]
                    if let Some(probe) = &probe {
                        probe.drain_event(super::runtime_tests::DrainEvent::IntakeStopped);
                    }
                } else if let Command::Completion(data, ack) = command {
                    if let Err(e) = delivery.complete(&data, ack, Instant::now()) {
                        command_error = Some(e);
                    }
                } else {
                    command_error = Some(super::delivery::DeliveryError::Invariant);
                }
            })?;
            if let Some(error) = command_error {
                return Err(error.into());
            }
        }
        let next = if retry_interrupted {
            #[cfg(test)]
            if let (Some(probe), Some(retry_at)) = (&probe, delivery.retry_deadline()) {
                probe.drain_event(super::runtime_tests::DrainEvent::RetryInterrupted {
                    retry_at,
                    checked_at: now,
                });
            }
            None
        } else {
            deadline.into_iter().chain(delivery.next_deadline()).min()
        };
        match endpoint.next_until(false, next) {
            Ok(Input::Command(Command::Sent(id, result))) => {
                delivery.sent(id, result, Instant::now())?
            }
            Ok(Input::Command(Command::Completion(data, ack))) => {
                let _ = delivery.complete(&data, ack, Instant::now())?;
            }
            #[cfg(test)]
            Ok(Input::Command(Command::Inspect(reply))) => {
                let record = owner.state()?.get(&id).ok_or("missing checkpoint")?;
                let _ = reply.try_send((record.committed_offset, delivery.builder().frames));
            }
            Ok(Input::Command(Command::Drain)) => {
                scheduler.stop_source_reads();
                draining = true;
                #[cfg(test)]
                if let Some(probe) = &probe {
                    probe.drain_event(super::runtime_tests::DrainEvent::IntakeStopped);
                }
            }
            Ok(Input::Deadline) => {}
            Ok(Input::Observation(_)) => unreachable!("intake is disabled"),
            Err(_) if cancel.is_cancelled() => break,
            Err(e) => return Err(e.into()),
        }
    }
    let _ = owner.sync_checkpoint()?;
    Ok(WorkerExit::Stopped)
}

/// Between preparation, each bounded association step, and commit, service worker control.
fn await_admission(
    endpoint: &mut ReadCheckpoint<Offer, Attempt, Command>,
    owner: &mut IdentityOwner,
    budget: &ProbeBudget,
    cancel: &Cancellation,
    reconcile_interval: Duration,
    #[cfg(test)] probe: Option<&super::runtime_tests::Probe>,
) -> Result<Option<(Source, ReaderSeed)>, Failure> {
    let mut pending: Option<(Instant, Offer, Admission)> = None;
    loop {
        let input = endpoint.next_until(pending.is_none(), pending.as_ref().map(|(at, ..)| *at))?;
        match input {
            Input::Command(Command::Drain) => return Ok(None),
            Input::Command(_) => return Err("unexpected delivery command before admission".into()),
            Input::Observation(offer) => {
                #[cfg(test)]
                if let Some(probe) = probe {
                    probe.before_admission();
                }
                let source = &offer.source;
                let candidate = Candidate {
                    path: &source.path,
                    locator: source.locator,
                    size: source.size,
                    fingerprint: &source.fingerprint,
                    context: Context::Unrelated,
                };
                let admission = retry_source_pressure(owner.prepare(
                    candidate,
                    budget,
                    SystemTime::now(),
                    || source.open(cancel),
                    || cancel.is_cancelled(),
                ))?;
                pending = Some((Instant::now(), offer, admission));
            }
            Input::Deadline => {
                let (_, offer, admission) = pending.take().ok_or("missing pending admission")?;
                match admission {
                    Admission::Scanning(mut scan) if !scan.is_complete() => {
                        let _ = owner.advance_association(&mut scan, || cancel.is_cancelled())?;
                        #[cfg(test)]
                        if !scan.is_complete()
                            && let Some(probe) = probe
                        {
                            probe.before_admission();
                        }
                        pending = Some((Instant::now(), offer, Admission::Scanning(scan)));
                    }
                    Admission::Scanning(scan) => {
                        let next = retry_source_pressure(owner.finish_prepare(
                            scan,
                            budget,
                            SystemTime::now(),
                            || offer.source.open(cancel),
                            || cancel.is_cancelled(),
                        ))?;
                        // A busy verification probe returns the completed cursor and its lease.
                        // Wait for the next retry or control message without rescanning the table.
                        let now = Instant::now();
                        let at = if matches!(next, Admission::Scanning(_)) {
                            now.checked_add(reconcile_interval)
                                .ok_or("admission probe retry deadline overflow")?
                        } else {
                            now
                        };
                        pending = Some((at, offer, next));
                    }
                    Admission::Ready(plan) => {
                        let CommittedAdmission::Reader(seed) = owner.commit(plan)? else {
                            return Err("file quarantined".into());
                        };
                        let _ = offer.reply.send(AdmissionReply::Committed);
                        return Ok(Some((offer.source, seed)));
                    }
                    Admission::Ignored | Admission::Deferred => {
                        let _ = offer.reply.send(AdmissionReply::Rescan);
                    }
                    other => return Err(format!("file admission: {other:?}").into()),
                }
            }
        }
    }
}

fn retry_source_pressure(
    result: Result<Admission, IdentityError>,
) -> Result<Admission, IdentityError> {
    match result {
        Err(IdentityError::Source(error))
            if matches!(
                error.os_error().and_then(std::io::Error::raw_os_error),
                Some(libc::EMFILE | libc::ENFILE)
            ) =>
        {
            Ok(Admission::Deferred)
        }
        other => other,
    }
}

/// Scenario: a literal include shares its directory with unrelated files and enumeration has one work unit.
/// Guarantees: targeted discovery verifies the selected file and its fingerprint even when a full traversal is incomplete.
#[cfg(test)]
#[test]
fn literal_lookup_does_not_consume_directory_enumeration_budget() {
    use super::{
        config::Config,
        discovery::{ScanConfig, ScanLimits},
        workers::Workers,
    };
    let dir = tempfile::tempdir().unwrap();
    for i in 0..8 {
        std::fs::write(dir.path().join(format!("other-{i}")), b"unrelated").unwrap();
    }
    let file = dir.path().join("app.log");
    std::fs::write(&file, b"hello\n").unwrap();
    let (_, paths) = Config {
        include: vec![file.to_str().unwrap().into()],
        ..Config::default()
    }
    .compile("group", "main", "logs", dir.path())
    .unwrap()
    .into_parts();
    let mut scanner = Scanner::new(
        ScanConfig {
            recursive: false,
            include_roots: paths.include_roots().to_vec(),
            exclusion_roots: Vec::new(),
            symlinks: SymlinkPolicy::Reject,
            checkpoint_namespace: FileLocator {
                device: u64::MAX,
                inode: u64::MAX,
            },
            limits: ScanLimits {
                work_units: 1,
                ..ScanLimits::default()
            },
        },
        paths,
    )
    .unwrap();
    let budget = Arc::new(ProbeBudget::default());
    let pass = scanner.scan(
        1,
        &budget,
        || false,
        |observations| {
            assert!(
                !observations
                    .iter()
                    .any(|obs| obs.path == file && obs.reason.is_none())
            );
            true
        },
    );
    assert!(!pass.complete);
    let workers = Workers::new(Duration::from_secs(5)).unwrap();
    let cancel = workers.cancellation();
    let (out, mut input, _pipeline) = super::workers::channels::<Offer, (), ()>(
        std::num::NonZeroUsize::MIN,
        std::num::NonZeroUsize::MIN,
        cancel.clone(),
        cancel.clone(),
    );
    std::thread::scope(|scope| {
        let stop = cancel.clone();
        let path = &file;
        let _worker =
            scope.spawn(move || discover(scanner, out, budget, stop, Duration::from_secs(1), path));
        let received = input.next_until(true, Some(Instant::now() + Duration::from_secs(60)));
        let _ = cancel.cancel();
        let Input::Observation(offer) = received.unwrap() else {
            panic!("expected verified literal file");
        };
        assert_eq!(
            offer.source.locator,
            FileLocator::from_metadata(&std::fs::metadata(&file).unwrap())
        );
        assert_eq!(offer.source.fingerprint, b"hello\n");
    });
}
