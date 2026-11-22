// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Developer-only, single-file tailing through the source, framer and workers.
//!
//! This is not a registered OTAP receiver or a Phase 1 implementation. It has no
//! durable identity, checkpoint, OTAP topology, rotation or restart contract.
//! Complete raw LF-delimited records are copied from offset zero to a new file.
//! The file sink acknowledges each visible write before the next record proceeds.
//! An incomplete final record is never completed merely because EOF or shutdown
//! occurred. Cancellation may abandon an in-flight write.

use super::{
    decoder::{Encoding, OnDecodeError},
    framer::{LineBody, LineConfig, LineFailure, LineFramer, LineStart, OversizeBehavior},
    source::{FileAccessError, FileLocator, PinnedSource, SourceFile, SymlinkPolicy},
    workers::{self, Cancellation, ChannelError, Input, JoinReport, WorkerOutcome, Workers},
};
use std::{
    ffi::CString,
    fs::File,
    future::Future,
    io,
    num::NonZeroUsize,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::oneshot,
};

const POLL_INTERVAL: Duration = Duration::from_millis(100);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
const READ_BYTES: usize = 16 * 1024;
const MAX_LINE_BYTES: usize = 64 * 1024;

/// Failure of the experimental path; no failure authorizes checkpoint progress.
#[derive(Debug, thiserror::Error)]
pub enum TailError {
    /// Source, output or shutdown-signal I/O failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// The checked Linux source operation failed.
    #[error(transparent)]
    Source(FileAccessError),
    /// The raw line framer failed.
    #[error(transparent)]
    Framing(#[from] LineFailure),
    /// The input must name one regular file.
    #[error("input must name one file with a valid native filename")]
    InvalidInput,
    /// This experiment does not authorize stream replacement.
    #[error("input replacement or truncation is unsupported by this experiment")]
    InputChanged,
    /// The example fails rather than silently splitting or truncating a record.
    #[error("input line exceeds the experimental 64 KiB limit")]
    LineTooLong,
    /// A transport failed without delivering a record.
    #[error("worker transport failed: {0}")]
    Transport(String),
    /// Cooperative cancellation interrupted work.
    #[error("experimental tail cancelled")]
    Cancelled,
    /// Output acknowledgements must match exactly one published record.
    #[error("unexpected output acknowledgement")]
    UnexpectedAck,
    /// The worker failed or did not publish its result.
    #[error("{0} worker failed: {1}")]
    Worker(&'static str, String),
    /// The process must not report successful cleanup while a worker is alive.
    #[error("{0} worker is still running at the shutdown deadline")]
    JoinTimeout(&'static str),
}

impl From<FileAccessError> for TailError {
    fn from(error: FileAccessError) -> Self {
        match error {
            FileAccessError::Cancelled => Self::Cancelled,
            error => Self::Source(error),
        }
    }
}

impl From<ChannelError> for TailError {
    fn from(error: ChannelError) -> Self {
        match error {
            ChannelError::Cancelled => Self::Cancelled,
            error => Self::Transport(error.to_string()),
        }
    }
}

struct Batch {
    sequence: u64,
    payload: Payload,
}

enum Payload {
    Record(Vec<u8>),
    AtEof(u64),
}

struct Written(u64);
type Reader = workers::ReadCheckpoint<FileLocator, Batch, Written>;

fn input_entry(path: &Path) -> Result<(File, CString), TailError> {
    let name = path.file_name().ok_or(TailError::InvalidInput)?;
    let name = CString::new(name.as_bytes()).map_err(|_| TailError::InvalidInput)?;
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    Ok((File::open(parent.unwrap_or_else(|| Path::new(".")))?, name))
}

fn discover(
    input: &Path,
    expected: FileLocator,
    endpoint: workers::Discovery<FileLocator>,
    cancel: &Cancellation,
) -> Result<(), TailError> {
    let (directory, name) = input_entry(input)?;
    loop {
        // A pin establishes only this experiment's current locator, not durable
        // identity. Drop it before waiting on a bounded observation handoff.
        let locator = PinnedSource::open_at(
            &directory,
            &name,
            SymlinkPolicy::Reject,
            Some(expected),
            || cancel.is_cancelled(),
        )?
        .locator();
        endpoint.observe(locator)?;
        cancel.wait(POLL_INTERVAL)?;
    }
}

fn observe(input: Input<FileLocator, Written>, expected: FileLocator) -> Result<(), TailError> {
    match input {
        Input::Observation(locator) if locator == expected => Ok(()),
        Input::Observation(_) => Err(TailError::InputChanged),
        Input::Command(_) => Err(TailError::UnexpectedAck),
    }
}

fn publish(
    reader: &mut Reader,
    sequence: &mut u64,
    payload: Payload,
    locator: FileLocator,
) -> Result<(), TailError> {
    let mut unexpected = false;
    reader.publish(
        Batch {
            sequence: *sequence,
            payload,
        },
        |_| unexpected = true,
    )?;
    if unexpected {
        return Err(TailError::UnexpectedAck);
    }
    loop {
        match reader.next()? {
            Input::Command(Written(ack)) if ack == *sequence => {
                *sequence = sequence.checked_add(1).ok_or(TailError::UnexpectedAck)?;
                return Ok(());
            }
            input => observe(input, locator)?,
        }
    }
}

fn read(input: &Path, mut reader: Reader, cancel: &Cancellation) -> Result<(), TailError> {
    // Waiting for discovery first also prevents a source-open failure racing
    // the standalone owner's second worker startup.
    let locator = match reader.next()? {
        Input::Observation(locator) => locator,
        Input::Command(_) => return Err(TailError::UnexpectedAck),
    };
    let (directory, name) = input_entry(input)?;
    let source = SourceFile::open_at(
        &directory,
        &name,
        SymlinkPolicy::Reject,
        Some(locator),
        || cancel.is_cancelled(),
    )?;
    let mut framer = LineFramer::new(
        LineConfig {
            encoding: Encoding::Raw,
            on_decode_error: OnDecodeError::Fail,
            max_line_bytes: MAX_LINE_BYTES,
            max_record_bytes: MAX_LINE_BYTES,
            oversize: OversizeBehavior::Split,
        },
        LineStart::NewStream,
    )
    .map_err(|error| TailError::Worker("reader", error.to_string()))?;
    let mut buffer = vec![0; READ_BYTES];
    let mut offset = 0;
    let mut sequence = 0u64;
    loop {
        if source.metadata(|| cancel.is_cancelled())?.len() < offset {
            return Err(TailError::InputChanged);
        }
        let length = source.read_at(offset, &mut buffer, || cancel.is_cancelled())?;
        if length == 0 {
            publish(&mut reader, &mut sequence, Payload::AtEof(offset), locator)?;
            observe(reader.next()?, locator)?;
            continue;
        }
        let mut cursor = 0;
        loop {
            if cancel.is_cancelled() {
                return Err(TailError::Cancelled);
            }
            let step = framer.next(offset, &buffer[cursor..length])?;
            cursor += step.consumed;
            offset += step.consumed as u64;
            if let Some(frame) = step.frame {
                if frame.fragment.is_some() {
                    return Err(TailError::LineTooLong);
                }
                let mut bytes = match frame.body {
                    LineBody::Bytes(bytes) => bytes,
                    LineBody::Text(_) => {
                        return Err(TailError::Worker(
                            "reader",
                            "raw framer returned text".into(),
                        ));
                    }
                };
                bytes.push(b'\n');
                publish(&mut reader, &mut sequence, Payload::Record(bytes), locator)?;
            }
            if !step.advanced {
                break;
            }
        }
    }
}

fn check_joins(report: JoinReport) -> Result<(), TailError> {
    for (name, outcome) in [
        ("discovery", report.discovery),
        ("reader", report.read_checkpoint),
    ] {
        match outcome {
            Some(WorkerOutcome::Panicked(message)) => return Err(TailError::Worker(name, message)),
            Some(WorkerOutcome::StillRunning) => return Err(TailError::JoinTimeout(name)),
            Some(WorkerOutcome::Joined) | None => {}
        }
    }
    Ok(())
}

/// Tails one existing Linux file into a new output file until shutdown resolves.
///
/// The output must not already exist. Records are raw bytes terminated by LF,
/// at most 64 KiB excluding LF. Final-entry symlinks are rejected. The input must
/// remain the same append-only file; detectable replacement/truncation fails.
/// An optional new status file exposes the last observed temporary-EOF offset
/// as a fixed-width decimal line. It is a test handshake, never a checkpoint.
/// There are no checkpoints, replay, production receiver registration or durable
/// delivery claims. A successful shutdown means both workers joined, not that an
/// incomplete or in-flight record was drained.
pub async fn tail(
    input: PathBuf,
    output: PathBuf,
    status: Option<PathBuf>,
    shutdown: impl Future<Output = io::Result<()>>,
) -> Result<(), TailError> {
    let input_metadata = tokio::fs::symlink_metadata(&input).await?;
    if !input_metadata.is_file() {
        return Err(TailError::InvalidInput);
    }
    let expected = FileLocator::from_metadata(&input_metadata);
    let mut output = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .await?;
    let mut status = match status {
        Some(path) => Some(
            tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .await?,
        ),
        None => None,
    };
    let mut workers = Workers::new(SHUTDOWN_TIMEOUT)?;
    let cancel = workers.cancellation();
    let (discovery, reader, mut owner) = workers::channels(
        NonZeroUsize::new(1).expect("nonzero observation capacity"),
        NonZeroUsize::new(1).expect("nonzero command capacity"),
        cancel.clone(),
        cancel.clone(),
    );
    let (read_tx, read_rx) = oneshot::channel();
    let (discovery_tx, discovery_rx) = oneshot::channel();
    let read_input = input.clone();
    workers.start_read_checkpoint(move |cancel| {
        // The owner may itself have been dropped while awaiting async I/O.
        _ = read_tx.send(read(&read_input, reader, &cancel));
    })?;
    if let Err(error) = workers.start_discovery(move |cancel| {
        _ = discovery_tx.send(discover(&input, expected, discovery, &cancel));
    }) {
        check_joins(workers.join().await)?;
        return Err(error.into());
    }

    let mut shutdown = std::pin::pin!(shutdown);
    let operation = async {
        loop {
            let batch = tokio::select! {
                biased;
                result = &mut shutdown => {
                    return result.map_err(TailError::from);
                }
                result = owner.batch() => result?,
            };
            let write = async {
                match &batch.payload {
                    Payload::Record(bytes) => {
                        output.write_all(bytes).await?;
                        output.flush().await
                    }
                    Payload::AtEof(offset) => {
                        if let Some(status) = &mut status {
                            _ = status.rewind().await?;
                            status
                                .write_all(format!("{offset:020}\n").as_bytes())
                                .await?;
                            status.flush().await?;
                        }
                        Ok::<_, io::Error>(())
                    }
                }
            };
            tokio::select! {
                biased;
                result = &mut shutdown => {
                    return result.map_err(TailError::from);
                }
                _ = cancel.cancelled() => return Err(TailError::Cancelled),
                result = write => result?,
            }
            owner.command(Written(batch.sequence)).await?;
        }
    }
    .await;

    check_joins(workers.join().await)?;
    for (name, result) in [("discovery", discovery_rx), ("reader", read_rx)] {
        match result
            .await
            .map_err(|error| TailError::Worker(name, error.to_string()))?
        {
            Ok(()) | Err(TailError::Cancelled) => {}
            Err(error) => return Err(error),
        }
    }
    operation
}
