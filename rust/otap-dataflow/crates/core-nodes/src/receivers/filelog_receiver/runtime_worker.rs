// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    config::PathPolicy,
    delivery::{Attempt, CheckpointOutcome, Correlation, Delivery, SendOutcome},
    discovery::{Observation, ProbeBudget, ScanReason, Scanner},
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
    let mut generation = 0;
    let mut last_warning: Option<Instant> = None;
    while !cancel.is_cancelled() {
        generation += 1;
        let mut candidate = None;
        let mut failure = None;
        let pass = scanner.scan(
            generation,
            &budget,
            || cancel.is_cancelled(),
            |observations| {
                for obs in observations {
                    if let Some(reason) = obs.reason {
                        if reason != ScanReason::Cancelled
                            && (reason.invalidates_coverage()
                                || (obs.path == file
                                    && matches!(
                                        reason,
                                        ScanReason::Content
                                            | ScanReason::Unsupported
                                            | ScanReason::LeafLoop
                                    )))
                        {
                            let _ = failure.get_or_insert((reason, obs.errno));
                        }
                        continue;
                    }
                    if let Observation {
                        path,
                        locator: Some(locator),
                        size: Some(size),
                        fingerprint,
                        ..
                    } = obs
                        && path == file
                    {
                        candidate = Some(Source {
                            path: path.clone(),
                            locator: *locator,
                            size: *size,
                            fingerprint: fingerprint.clone(),
                        });
                        return false;
                    }
                }
                true
            },
        );
        if cancel.is_cancelled() {
            return;
        }
        if let Some(source) = candidate {
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
        } else if let Some((reason, errno)) = failure
            .or_else(|| (!pass.complete && !pass.cancelled).then_some((ScanReason::Coverage, None)))
            && last_warning.is_none_or(|last| last.elapsed() >= DISCOVERY_WARNING_INTERVAL)
        {
            last_warning = Some(Instant::now());
            let detail = format!("{reason:?}");
            otel_warn!(
                "filelog_receiver.discovery_failed",
                generation = pass.generation,
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
    scanner_tx
        .send((scanner, settings.settings().discovery.reconcile_interval))
        .map_err(|_| "discovery worker stopped")?;
    ready.send(()).map_err(|_| "receiver stopped")?;
    let seed = loop {
        let offer = match endpoint.next()? {
            Input::Observation(offer) => offer,
            Input::Command(Command::Drain) => return Ok(WorkerExit::Drained),
            _ => return Err("unexpected command before discovery".into()),
        };
        #[cfg(test)]
        if let Some(probe) = &probe {
            probe.before_admission();
        }
        if let Some(seed) = admit(&offer.source, &mut owner, &budget, &cancel)? {
            let _ = binding.get_or_init(|| offer.source);
            let _ = offer.reply.send(AdmissionReply::Committed);
            break seed;
        }
        let _ = offer.reply.send(AdmissionReply::Rescan);
    };
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
        let retry_interrupted = draining && delivery.retry_pending();
        if !retry_interrupted && let Some(attempt) = delivery.next_attempt(now)? {
            let mut command_error = None;
            endpoint.publish(attempt, |command| {
                if matches!(command, Command::Drain) {
                    scheduler.stop_source_reads();
                    draining = true;
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
                    probe.draining();
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

fn admit(
    source: &Source,
    owner: &mut IdentityOwner,
    budget: &ProbeBudget,
    cancel: &Cancellation,
) -> Result<Option<ReaderSeed>, Failure> {
    let admission = match prepare(source, owner, budget, cancel) {
        Err(IdentityError::Source(error))
            if matches!(
                error.os_error().and_then(std::io::Error::raw_os_error),
                Some(libc::EMFILE | libc::ENFILE)
            ) =>
        {
            return Ok(None);
        }
        result => result?,
    };
    let Admission::Ready(plan) = admission else {
        return match admission {
            Admission::Deferred | Admission::Ignored | Admission::Scanning(_) => Ok(None),
            other => Err(format!("file admission: {other:?}").into()),
        };
    };
    let CommittedAdmission::Reader(seed) = owner.commit(plan)? else {
        return Err("file quarantined".into());
    };
    Ok(Some(seed))
}

fn prepare(
    source: &Source,
    owner: &mut IdentityOwner,
    budget: &ProbeBudget,
    cancel: &Cancellation,
) -> Result<Admission, IdentityError> {
    let candidate = Candidate {
        path: &source.path,
        locator: source.locator,
        size: source.size,
        fingerprint: &source.fingerprint,
        context: Context::Unrelated,
    };
    let mut admission = owner.prepare(
        candidate,
        budget,
        SystemTime::now(),
        || source.open(cancel),
        || cancel.is_cancelled(),
    )?;
    if let Admission::Scanning(mut scan) = admission {
        while !owner
            .advance_association(&mut scan, || cancel.is_cancelled())?
            .complete
        {}
        admission = owner.finish_prepare(
            scan,
            budget,
            SystemTime::now(),
            || source.open(cancel),
            || cancel.is_cancelled(),
        )?;
    }
    Ok(admission)
}
