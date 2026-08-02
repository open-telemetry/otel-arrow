// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Journald receiver.
//!
//! This module implements the Linux-only journald source described in
//! [`docs/journald-receiver.md`](../../../../../../docs/journald-receiver.md).
//!
//! Most runtime code is Linux-only (`#[cfg(target_os = "linux")]`); on other
//! platforms the factory rejects construction with a clear error.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

#[cfg(target_os = "linux")]
use async_trait::async_trait;
use linkme::distributed_slice;
#[cfg(target_os = "linux")]
use otap_df_channel::error::SendError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
#[cfg(target_os = "linux")]
use otap_df_engine::control::{CallData, Context8u8, NodeControlMsg};
#[cfg(target_os = "linux")]
use otap_df_engine::error::{Error, ReceiverErrorKind, TypedError};
#[cfg(target_os = "linux")]
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
#[cfg(target_os = "linux")]
use otap_df_engine::terminal_state::TerminalState;
#[cfg(target_os = "linux")]
use otap_df_engine::{
    Interests, MessageSourceLocalEffectHandlerExtension, ProducerEffectHandlerExtension,
};
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
#[cfg(target_os = "linux")]
use otap_df_otap::pdata::Context;
use otap_df_otap::pdata::OtapPdata;
#[cfg(target_os = "linux")]
use otap_df_pdata::OtapPayload;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
#[cfg(target_os = "linux")]
use otap_df_telemetry::metrics::MetricSetSnapshot;
#[cfg(target_os = "linux")]
use otap_df_telemetry::{otel_debug, otel_info, otel_warn};
use otap_df_telemetry_macros::metric_set;
use serde_json::Value;
#[cfg(any(target_os = "linux", test))]
use std::collections::BTreeMap;
use std::collections::HashSet;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::{LazyLock, Mutex};
#[cfg(target_os = "linux")]
use std::time::Instant as StdInstant;

mod arrow_records_encoder;
mod checkpoint;
mod config;
#[cfg(any(target_os = "linux", test, feature = "bench"))]
mod decode;
mod journal;

/// Re-exported only for the out-of-crate `journald_decode` benchmark. Not part
/// of the public API; may change or disappear without notice.
#[cfg(feature = "bench")]
#[doc(hidden)]
pub use decode::bench_reference_decode;

use config::RuntimeConfig;
pub use config::{
    BatchConfig, CheckpointConfig, Config, DEFAULT_JOURNAL_ROOT_PATH, DEFAULT_SOURCE_ID,
    ExtractionConfig, JournalConfig, LargeFieldPolicy, MaxPriority, OnNack, StartAt,
    severity_number_from_priority,
};

/// URN for the journald receiver.
pub const JOURNALD_RECEIVER_URN: &str = "urn:otel:receiver:journald";

/// Telemetry metrics for the journald receiver.
///
/// Tracks lifecycle transitions, downstream delivery, Ack/Nack handling, and
/// durable cursor checkpoint progress.
#[metric_set(name = "receiver.journald")]
#[derive(Debug, Default, Clone)]
pub struct JournaldReceiverMetrics {
    /// Number of times the receiver was started.
    #[metric(unit = "{start}")]
    pub starts: Counter<u64>,
    /// Number of clean drain transitions.
    #[metric(unit = "{drain}")]
    pub drains: Counter<u64>,
    /// Number of clean shutdown transitions.
    #[metric(unit = "{shutdown}")]
    pub shutdowns: Counter<u64>,
    /// Number of log batches emitted downstream.
    #[metric(unit = "{batch}")]
    pub batches_sent: Counter<u64>,
    /// Number of log records emitted downstream.
    #[metric(unit = "{record}")]
    pub records_sent: Counter<u64>,
    /// Number of downstream Acks observed.
    #[metric(unit = "{ack}")]
    pub acks: Counter<u64>,
    /// Number of downstream Nacks observed.
    #[metric(unit = "{nack}")]
    pub nacks: Counter<u64>,
    /// Number of durable cursor commits completed.
    #[metric(unit = "{commit}")]
    pub cursor_commits: Counter<u64>,
    /// Number of durable cursor commit failures.
    #[metric(unit = "{failure}")]
    pub checkpoint_failures: Counter<u64>,
    /// Number of source read failures reported by the worker.
    #[metric(unit = "{failure}")]
    pub source_failures: Counter<u64>,
    /// Number of journald fields dropped by extraction safety limits.
    #[metric(unit = "{field}")]
    pub source_dropped_fields: Counter<u64>,
    /// Number of times the worker was asked to rewind after a Nack.
    #[metric(unit = "{rewind}")]
    pub rewinds: Counter<u64>,
}

/// Journald receiver instance.
pub struct JournaldReceiver {
    #[allow(dead_code)]
    config: RuntimeConfig,
    #[cfg(target_os = "linux")]
    checkpoint_path: PathBuf,
    _lease: SourceLease,
    metrics: Option<MetricSet<JournaldReceiverMetrics>>,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
/// Declares the journald receiver as a local receiver factory.
pub static JOURNALD_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: JOURNALD_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        create_journald_receiver(pipeline, node, node_config, receiver_config)
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: validate_journald_config,
};

#[cfg(target_os = "linux")]
fn create_journald_receiver(
    pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    receiver_config: &ReceiverConfig,
) -> Result<ReceiverWrapper<OtapPdata>, otap_df_config::error::Error> {
    if pipeline.num_cores() > 1 {
        return Err(otap_df_config::error::Error::InvalidUserConfig {
            error: "journald must run in a one-core source pipeline; use \
                 receiver:journald -> exporter:topic and fan out downstream"
                .to_owned(),
        });
    }
    let mut receiver = JournaldReceiver::from_config(&node_config.config)?;
    receiver.checkpoint_path = checkpoint::checkpoint_path(
        &receiver.config.checkpoint.directory,
        pipeline.pipeline_group_id().as_ref(),
        pipeline.pipeline_id().as_ref(),
        receiver_config.name.as_ref(),
        &receiver.config.source_id,
    );
    receiver.metrics = Some(pipeline.register_metrics::<JournaldReceiverMetrics>());
    Ok(ReceiverWrapper::local(
        receiver,
        node,
        node_config,
        receiver_config,
    ))
}

#[cfg(not(target_os = "linux"))]
fn create_journald_receiver(
    _pipeline: PipelineContext,
    _node: NodeId,
    _node_config: Arc<NodeUserConfig>,
    _receiver_config: &ReceiverConfig,
) -> Result<ReceiverWrapper<OtapPdata>, otap_df_config::error::Error> {
    Err(unsupported_platform_error())
}

#[cfg(target_os = "linux")]
fn validate_journald_config(config: &Value) -> Result<(), otap_df_config::error::Error> {
    let parsed: Config = serde_json::from_value(config.clone()).map_err(|e| {
        otap_df_config::error::Error::InvalidUserConfig {
            error: e.to_string(),
        }
    })?;
    RuntimeConfig::try_from(parsed).map(|_| ())
}

#[cfg(not(target_os = "linux"))]
fn validate_journald_config(_config: &Value) -> Result<(), otap_df_config::error::Error> {
    Err(unsupported_platform_error())
}

#[cfg(not(target_os = "linux"))]
fn unsupported_platform_error() -> otap_df_config::error::Error {
    otap_df_config::error::Error::InvalidUserConfig {
        error: "journald receiver is supported only on Linux".to_owned(),
    }
}

impl JournaldReceiver {
    /// Builds a receiver from a JSON config value.
    fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let parsed: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Self::new(parsed)
    }

    /// Builds a receiver from an already-deserialized `Config`.
    fn new(config: Config) -> Result<Self, otap_df_config::error::Error> {
        let runtime = RuntimeConfig::try_from(config)?;
        let lease = SourceLease::acquire(&runtime.lease_key)?;
        Ok(Self {
            config: runtime,
            #[cfg(target_os = "linux")]
            checkpoint_path: PathBuf::new(),
            _lease: lease,
            metrics: None,
        })
    }
}

#[cfg(target_os = "linux")]
fn terminal_state(
    deadline: std::time::Instant,
    metrics: &Option<MetricSet<JournaldReceiverMetrics>>,
) -> TerminalState {
    if let Some(metrics) = metrics {
        TerminalState::new(deadline, [metrics.snapshot()])
    } else {
        TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, [])
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct WorkerBatch {
    id: u64,
    first_cursor: String,
    last_cursor: String,
    records: otap_df_pdata::otap::OtapArrowRecords,
    record_count: usize,
    dropped_fields: u64,
}

#[cfg(target_os = "linux")]
enum WorkerEvent {
    Batch(WorkerBatch),
    CommitResult {
        batch_id: u64,
        cursor: String,
        result: Result<(), checkpoint::CheckpointError>,
    },
    Failed(WorkerError),
    Stopped,
}

#[cfg(target_os = "linux")]
#[derive(Debug, thiserror::Error)]
enum WorkerError {
    #[error(transparent)]
    Checkpoint(#[from] checkpoint::CheckpointError),
    #[error(transparent)]
    Journal(#[from] journal::JournalError),
    #[error("failed to encode journald batch: {source}")]
    Encode {
        source: otap_df_pdata::encode::Error,
    },
    #[error("journald receiver event channel closed")]
    EventChannelClosed,
    #[error("journald worker received an unexpected command while handing off a batch")]
    UnexpectedCommand,
    #[error("journald cannot rewind before the first checkpoint is committed")]
    RewindBeforeCheckpoint,
}

#[cfg(any(target_os = "linux", test))]
#[derive(Debug, Eq, PartialEq)]
enum WorkerCommand {
    Commit { batch_id: u64, cursor: String },
    Rewind,
    Drain,
    Shutdown,
}

#[cfg(target_os = "linux")]
enum BatchHandoff {
    Sent { drain_requested: bool },
    Shutdown,
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug)]
struct PendingBatch {
    last_cursor: String,
    decision: PendingDecision,
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingDecision {
    Pending,
    CommitSent,
    RewindSent,
    FailSent,
}

#[cfg(any(target_os = "linux", test))]
#[derive(Debug, Eq, PartialEq)]
struct PendingControlEffect {
    command: Option<WorkerCommand>,
    fail_nack: bool,
    record_ack: bool,
    record_nack: bool,
    record_rewind: bool,
}

#[cfg(target_os = "linux")]
struct WorkerHandle {
    cmd_tx: std::sync::mpsc::SyncSender<WorkerCommand>,
    join: std::thread::JoinHandle<()>,
}

#[cfg(target_os = "linux")]
const WORKER_COMMAND_CHANNEL_CAPACITY: usize = 8;
#[cfg(target_os = "linux")]
const WORKER_EVENT_CONTROL_SLOTS: usize = 4;
#[cfg(target_os = "linux")]
const WORKER_COMMAND_RETRY_LIMIT: usize = 200;
#[cfg(target_os = "linux")]
const WORKER_COMMAND_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(10);

#[cfg(target_os = "linux")]
fn batch_id_from_call_data(call_data: &CallData) -> Option<u64> {
    call_data.first().copied().map(u64::from)
}

#[cfg(any(target_os = "linux", test))]
fn apply_pending_ack(
    pending: &mut BTreeMap<u64, PendingBatch>,
    batch_id: u64,
) -> Option<PendingControlEffect> {
    let pending_batch = pending.get_mut(&batch_id)?;
    if pending_batch.decision != PendingDecision::Pending {
        return None;
    }
    pending_batch.decision = PendingDecision::CommitSent;
    Some(PendingControlEffect {
        command: Some(WorkerCommand::Commit {
            batch_id,
            cursor: pending_batch.last_cursor.clone(),
        }),
        fail_nack: false,
        record_ack: true,
        record_nack: false,
        record_rewind: false,
    })
}

#[cfg(any(target_os = "linux", test))]
fn apply_pending_nack(
    pending: &mut BTreeMap<u64, PendingBatch>,
    batch_id: u64,
    on_nack: OnNack,
) -> Option<PendingControlEffect> {
    let pending_batch = pending.get_mut(&batch_id)?;
    if pending_batch.decision != PendingDecision::Pending {
        return None;
    }

    match on_nack {
        OnNack::Rewind => {
            pending_batch.decision = PendingDecision::RewindSent;
            pending.clear();
            Some(PendingControlEffect {
                command: Some(WorkerCommand::Rewind),
                fail_nack: false,
                record_ack: false,
                record_nack: true,
                record_rewind: true,
            })
        }
        OnNack::Fail => {
            pending_batch.decision = PendingDecision::FailSent;
            Some(PendingControlEffect {
                command: None,
                fail_nack: true,
                record_ack: false,
                record_nack: true,
                record_rewind: false,
            })
        }
    }
}

#[cfg(target_os = "linux")]
fn receiver_error(receiver: NodeId, error: impl Into<String>) -> Error {
    Error::ReceiverError {
        receiver,
        kind: ReceiverErrorKind::Other,
        error: error.into(),
        source_detail: String::new(),
    }
}

#[cfg(target_os = "linux")]
fn terminal_error(
    effect_handler: &local::EffectHandler<OtapPdata>,
    error: impl Into<String>,
) -> Error {
    receiver_error(effect_handler.receiver_id(), error)
}

#[cfg(target_os = "linux")]
async fn send_worker_command(
    tx: &std::sync::mpsc::SyncSender<WorkerCommand>,
    cmd: WorkerCommand,
    effect_handler: &local::EffectHandler<OtapPdata>,
) -> Result<(), Error> {
    let mut cmd = Some(cmd);
    for _ in 0..WORKER_COMMAND_RETRY_LIMIT {
        match tx.try_send(cmd.take().expect("command must be present")) {
            Ok(()) => return Ok(()),
            Err(std::sync::mpsc::TrySendError::Full(returned)) => {
                cmd = Some(returned);
                tokio::time::sleep(WORKER_COMMAND_RETRY_DELAY).await;
            }
            Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                return Err(terminal_error(
                    effect_handler,
                    "journald worker command channel disconnected",
                ));
            }
        }
    }
    Err(terminal_error(
        effect_handler,
        "journald worker command channel saturated",
    ))
}

#[cfg(target_os = "linux")]
fn spawn_worker(
    config: RuntimeConfig,
    checkpoint_path: PathBuf,
    event_tx: tokio::sync::mpsc::Sender<WorkerEvent>,
) -> Result<WorkerHandle, String> {
    // Keep blocking libsystemd reads and checkpoint fsyncs off the dfengine
    // async runtime thread. As with host_metrics, a dedicated source worker
    // caps the blocking surface at one OS thread for this receiver.
    let (cmd_tx, cmd_rx) = std::sync::mpsc::sync_channel(WORKER_COMMAND_CHANNEL_CAPACITY);
    let join = std::thread::Builder::new()
        .name(format!("otap-journald-{}", config.source_id))
        .spawn(move || worker_loop(config, checkpoint_path, event_tx, cmd_rx))
        .map_err(|err| format!("failed to spawn journald worker thread: {err}"))?;
    Ok(WorkerHandle { cmd_tx, join })
}

#[cfg(target_os = "linux")]
async fn join_worker(
    worker: WorkerHandle,
    effect_handler: &local::EffectHandler<OtapPdata>,
) -> Result<(), Error> {
    let receiver_id = effect_handler.receiver_id();
    drop(worker.cmd_tx);
    tokio::task::spawn_blocking(move || worker.join.join())
        .await
        .map_err(|err| {
            receiver_error(
                receiver_id.clone(),
                format!("journald worker join failed: {err}"),
            )
        })?
        .map_err(|_| receiver_error(receiver_id, "journald worker thread panicked"))
}

#[cfg(target_os = "linux")]
fn worker_loop(
    config: RuntimeConfig,
    checkpoint_path: PathBuf,
    event_tx: tokio::sync::mpsc::Sender<WorkerEvent>,
    cmd_rx: std::sync::mpsc::Receiver<WorkerCommand>,
) {
    let result = worker_loop_inner(config, checkpoint_path, &event_tx, &cmd_rx);
    if let Err(err) = result {
        let _ = event_tx.blocking_send(WorkerEvent::Failed(err));
    }
    let _ = event_tx.blocking_send(WorkerEvent::Stopped);
}

#[cfg(target_os = "linux")]
fn worker_loop_inner(
    config: RuntimeConfig,
    checkpoint_path: PathBuf,
    event_tx: &tokio::sync::mpsc::Sender<WorkerEvent>,
    cmd_rx: &std::sync::mpsc::Receiver<WorkerCommand>,
) -> Result<(), WorkerError> {
    let mut committed_cursor = checkpoint::read_cursor(&checkpoint_path)?;
    let mut reader = journal::SdJournalReader::open(&config, committed_cursor.as_deref())?;
    let mut next_batch_id = 1u64;
    let mut builder = arrow_records_encoder::JournaldArrowRecordsBuilder::new();
    let mut first_cursor = String::new();
    let mut last_cursor = String::new();
    let mut dropped_fields = 0u64;
    let mut first_record_at = StdInstant::now();
    let mut in_flight = None;
    let mut draining = false;

    loop {
        if in_flight.is_some() {
            match cmd_rx.recv() {
                Ok(WorkerCommand::Commit { batch_id, cursor }) => {
                    let result = checkpoint::write_cursor(&checkpoint_path, &cursor);
                    if result.is_ok() {
                        committed_cursor = Some(cursor.clone());
                        in_flight = None;
                    }
                    // On failure, keep the batch in flight and report the result.
                    // The async task owns retry counting and resends Commit until
                    // the write succeeds or max_consecutive_failures is reached.
                    let _ = event_tx.blocking_send(WorkerEvent::CommitResult {
                        batch_id,
                        cursor,
                        result,
                    });
                    if draining && in_flight.is_none() {
                        return Ok(());
                    }
                }
                Ok(WorkerCommand::Rewind) => {
                    if let Some(cursor) = committed_cursor.as_deref() {
                        builder = arrow_records_encoder::JournaldArrowRecordsBuilder::new();
                        first_cursor.clear();
                        last_cursor.clear();
                        dropped_fields = 0;
                        reader = journal::SdJournalReader::open_for_rewind(&config, Some(cursor))?;
                        in_flight = None;
                    } else if let Some(batch) = in_flight.clone() {
                        // No cursor has been committed yet. Re-opening from start_at=end
                        // would skip the uncheckpointed batch, so retry the retained batch.
                        match send_batch_or_observe_shutdown(event_tx, cmd_rx, batch)? {
                            BatchHandoff::Sent { drain_requested } => {
                                draining |= drain_requested;
                            }
                            BatchHandoff::Shutdown => return Ok(()),
                        }
                    }
                }
                Ok(WorkerCommand::Drain) => {
                    draining = true;
                }
                Ok(WorkerCommand::Shutdown) | Err(_) => return Ok(()),
            }
            continue;
        }

        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                WorkerCommand::Commit { batch_id, cursor } => {
                    let result = checkpoint::write_cursor(&checkpoint_path, &cursor);
                    if result.is_ok() {
                        committed_cursor = Some(cursor.clone());
                    }
                    let _ = event_tx.blocking_send(WorkerEvent::CommitResult {
                        batch_id,
                        cursor,
                        result,
                    });
                    if draining {
                        return Ok(());
                    }
                }
                WorkerCommand::Rewind => {
                    let Some(cursor) = committed_cursor.as_deref() else {
                        // Defense-in-depth: with max_in_flight=1, a valid Nack rewind
                        // before the first commit is handled by the in-flight arm above.
                        return Err(WorkerError::RewindBeforeCheckpoint);
                    };
                    builder = arrow_records_encoder::JournaldArrowRecordsBuilder::new();
                    first_cursor.clear();
                    last_cursor.clear();
                    dropped_fields = 0;
                    reader = journal::SdJournalReader::open_for_rewind(&config, Some(cursor))?;
                }
                WorkerCommand::Drain => {
                    draining = true;
                }
                WorkerCommand::Shutdown => return Ok(()),
            }
        }

        if draining && builder.len() == 0 {
            return Ok(());
        }

        if !draining {
            let read_timeout = if builder.len() == 0 {
                Some(config.wait_timeout)
            } else {
                let elapsed = first_record_at.elapsed();
                config
                    .batch
                    .max_flush_period
                    .checked_sub(elapsed)
                    .filter(|remaining| !remaining.is_zero())
                    .map(|remaining| remaining.min(config.wait_timeout))
            };
            if let Some(timeout) = read_timeout {
                if let Some(entry) = reader.next_entry_with_wait_timeout(timeout)? {
                    if builder.len() == 0 {
                        first_cursor = entry.cursor.clone();
                        first_record_at = StdInstant::now();
                    }
                    last_cursor = entry.cursor.clone();
                    dropped_fields = dropped_fields.saturating_add(entry.dropped_fields);
                    builder.append(&entry);
                }
            }
        }

        let should_flush = builder.len() as usize >= config.batch.max_records
            || (draining && builder.len() > 0)
            || (builder.len() > 0 && first_record_at.elapsed() >= config.batch.max_flush_period);
        if should_flush {
            let record_count = usize::from(builder.len());
            let records = std::mem::replace(
                &mut builder,
                arrow_records_encoder::JournaldArrowRecordsBuilder::new(),
            )
            .build()
            .map_err(|source| WorkerError::Encode { source })?;
            let batch = WorkerBatch {
                id: next_batch_id,
                first_cursor: std::mem::take(&mut first_cursor),
                last_cursor: std::mem::take(&mut last_cursor),
                records,
                record_count,
                dropped_fields: std::mem::take(&mut dropped_fields),
            };
            let retained_batch = batch.clone();
            next_batch_id = next_batch_id.saturating_add(1);
            match send_batch_or_observe_shutdown(event_tx, cmd_rx, batch)? {
                BatchHandoff::Sent { drain_requested } => {
                    draining |= drain_requested;
                }
                BatchHandoff::Shutdown => {
                    return Ok(());
                }
            }
            in_flight = Some(retained_batch);
        }
    }
}

#[cfg(target_os = "linux")]
fn send_batch_or_observe_shutdown(
    event_tx: &tokio::sync::mpsc::Sender<WorkerEvent>,
    cmd_rx: &std::sync::mpsc::Receiver<WorkerCommand>,
    batch: WorkerBatch,
) -> Result<BatchHandoff, WorkerError> {
    let mut event = WorkerEvent::Batch(batch);
    let mut drain_requested = false;
    loop {
        match event_tx.try_send(event) {
            Ok(()) => return Ok(BatchHandoff::Sent { drain_requested }),
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                return Err(WorkerError::EventChannelClosed);
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(returned)) => {
                event = returned;
                match cmd_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                    Ok(WorkerCommand::Drain) => {
                        drain_requested = true;
                    }
                    Ok(WorkerCommand::Shutdown)
                    | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        return Ok(BatchHandoff::Shutdown);
                    }
                    Ok(WorkerCommand::Commit { .. } | WorkerCommand::Rewind) => {
                        return Err(WorkerError::UnexpectedCommand);
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for JournaldReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let JournaldReceiver {
            config,
            checkpoint_path,
            _lease,
            mut metrics,
        } = *self;

        if let Some(metrics) = metrics.as_mut() {
            metrics.starts.add(1);
        }

        otel_info!(
            "journald_receiver.start",
            source_id = config.source_id.as_str(),
            journal_root_path = config.journal.root_path.display().to_string()
        );

        // Periodic telemetry collection mirrors host_metrics_receiver.
        let _ = effect_handler
            .start_periodic_telemetry(std::time::Duration::from_secs(1))
            .await?;

        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<WorkerEvent>(
            config
                .checkpoint
                .max_in_flight_batches
                .saturating_add(WORKER_EVENT_CONTROL_SLOTS),
        );
        let worker = spawn_worker(config.clone(), checkpoint_path, event_tx)
            .map_err(|err| terminal_error(&effect_handler, err))?;
        let mut pending: BTreeMap<u64, PendingBatch> = BTreeMap::new();
        let mut checkpoint_failures = 0u32;
        let max_in_flight = config.checkpoint.max_in_flight_batches;
        let mut drain_deadline = None;

        loop {
            tokio::select! {
                biased;

                _ = async {
                    if let Some(deadline) = drain_deadline {
                        tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                }, if drain_deadline.is_some() => {
                    let deadline = drain_deadline.expect("drain deadline must be set");
                    if !pending.is_empty() {
                        otel_warn!(
                            "journald_receiver.drain_timeout",
                            source_id = config.source_id.as_str(),
                            pending_batches = pending.len() as u64,
                            message = "Drain deadline reached with batches still awaiting Ack/Nack; shutting down without advancing their checkpoints"
                        );
                    }
                    let _ =
                        send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                    drop(event_rx);
                    join_worker(worker, &effect_handler).await?;
                    effect_handler.notify_receiver_drained().await?;
                    return Ok(terminal_state(deadline, &metrics));
                }

                msg = ctrl_msg_recv.recv() => {
                    match msg {
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            if let Some(metrics) = metrics.as_mut() {
                                let _ = metrics_reporter.report(metrics);
                            }
                        }
                        Ok(NodeControlMsg::Ack(ack)) => {
                            let Some(batch_id) = batch_id_from_call_data(&ack.unwind.route.calldata) else {
                                continue;
                            };
                            if let Some(effect) = apply_pending_ack(&mut pending, batch_id) {
                                if let Some(metrics) = metrics.as_mut() {
                                    if effect.record_ack {
                                        metrics.acks.add(1);
                                    }
                                }
                                if let Some(command) = effect.command {
                                    send_worker_command(&worker.cmd_tx, command, &effect_handler).await?;
                                }
                            }
                        }
                        Ok(NodeControlMsg::Nack(nack)) => {
                            let Some(batch_id) = batch_id_from_call_data(&nack.unwind.route.calldata) else {
                                continue;
                            };
                            if let Some(effect) =
                                apply_pending_nack(&mut pending, batch_id, config.checkpoint.on_nack)
                            {
                                if let Some(metrics) = metrics.as_mut() {
                                    if effect.record_nack {
                                        metrics.nacks.add(1);
                                    }
                                    if effect.record_rewind {
                                        metrics.rewinds.add(1);
                                    }
                                }
                                if let Some(command) = effect.command {
                                    send_worker_command(&worker.cmd_tx, command, &effect_handler).await?;
                                }
                                if effect.fail_nack {
                                    let _ =
                                        send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                                    drop(event_rx);
                                    join_worker(worker, &effect_handler).await?;
                                    return Err(terminal_error(&effect_handler, "journald batch was Nacked"));
                                }
                            }
                        }
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.drains.add(1);
                            }
                            otel_info!(
                                "journald_receiver.drain_ingress",
                                source_id = config.source_id.as_str()
                            );
                            let local_deadline = StdInstant::now()
                                .checked_add(config.drain_timeout)
                                .unwrap_or(deadline);
                            let deadline = deadline.min(local_deadline);
                            drain_deadline = Some(deadline);
                            send_worker_command(&worker.cmd_tx, WorkerCommand::Drain, &effect_handler).await?;
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.shutdowns.add(1);
                            }
                            otel_info!(
                                "journald_receiver.shutdown",
                                source_id = config.source_id.as_str()
                            );
                            let _ =
                                send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                            drop(event_rx);
                            join_worker(worker, &effect_handler).await?;
                            return Ok(terminal_state(deadline, &metrics));
                        }
                        Ok(_) => {}
                        Err(e) => {
                            let _ =
                                send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                            drop(event_rx);
                            join_worker(worker, &effect_handler).await?;
                            return Err(Error::ChannelRecvError(e));
                        }
                    }
                }

                event = event_rx.recv() => {
                    match event {
                        Some(WorkerEvent::Batch(batch)) => {
                            if pending.len() >= max_in_flight {
                                let _ =
                                    send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                                drop(event_rx);
                                join_worker(worker, &effect_handler).await?;
                                return Err(terminal_error(
                                    &effect_handler,
                                    "journald worker exceeded configured in-flight batch limit",
                                ));
                            }
                            let mut pdata = OtapPdata::new(
                                Context::default(),
                                OtapPayload::OtapArrowRecords(batch.records),
                            );
                            let mut calldata = CallData::new();
                            calldata.push(Context8u8::from(batch.id));
                            effect_handler.subscribe_to(
                                Interests::ACKS | Interests::NACKS,
                                calldata,
                                &mut pdata,
                            );
                            let record_count = batch.record_count;
                            let dropped_fields = batch.dropped_fields;
                            let batch_id = batch.id;
                            let last_cursor = batch.last_cursor.clone();
                            debug_assert_eq!(max_in_flight, 1);
                            debug_assert!(pending.is_empty());
                            let send_result = match effect_handler.try_send_message_with_source_node(pdata) {
                                Ok(()) => Ok(()),
                                Err(TypedError::ChannelSendError(SendError::Full(pdata))) => {
                                    let mut send = Box::pin(effect_handler.send_message_with_source_node(pdata));
                                    loop {
                                        // While the downstream send is blocked, this batch is not
                                        // in `pending` yet. With v1's max_in_flight=1 invariant,
                                        // no Ack/Nack can require receiver state changes here.
                                        let result = tokio::select! {
                                            biased;

                                            _ = async {
                                                if let Some(deadline) = drain_deadline {
                                                    tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
                                                } else {
                                                    std::future::pending::<()>().await;
                                                }
                                            }, if drain_deadline.is_some() => {
                                                let deadline = drain_deadline.expect("drain deadline must be set");
                                                otel_warn!(
                                                    "journald_receiver.drain_timeout",
                                                    source_id = config.source_id.as_str(),
                                                    batch_id = batch_id,
                                                    message = "Drain deadline reached while blocked sending a batch downstream; shutting down without advancing the checkpoint"
                                                );
                                                let _ =
                                                    send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                                                drop(event_rx);
                                                join_worker(worker, &effect_handler).await?;
                                                effect_handler.notify_receiver_drained().await?;
                                                return Ok(terminal_state(deadline, &metrics));
                                            }

                                            msg = ctrl_msg_recv.recv() => {
                                                match msg {
                                                    Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                                                        if let Some(metrics) = metrics.as_mut() {
                                                            let _ = metrics_reporter.report(metrics);
                                                        }
                                                        continue;
                                                    }
                                                    Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                                                        if let Some(metrics) = metrics.as_mut() {
                                                            metrics.drains.add(1);
                                                        }
                                                        let local_deadline = StdInstant::now()
                                                            .checked_add(config.drain_timeout)
                                                            .unwrap_or(deadline);
                                                        let deadline = deadline.min(local_deadline);
                                                        drain_deadline = Some(deadline);
                                                        continue;
                                                    }
                                                    Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                                                        if let Some(metrics) = metrics.as_mut() {
                                                            metrics.shutdowns.add(1);
                                                        }
                                                        let _ =
                                                            send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                                                        drop(event_rx);
                                                        join_worker(worker, &effect_handler).await?;
                                                        return Ok(terminal_state(deadline, &metrics));
                                                    }
                                                    Ok(_) => {
                                                        continue;
                                                    }
                                                    Err(e) => {
                                                        let _ =
                                                            send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                                                        drop(event_rx);
                                                        join_worker(worker, &effect_handler).await?;
                                                        return Err(Error::ChannelRecvError(e));
                                                    }
                                                }
                                            }

                                            result = send.as_mut() => result,
                                        };
                                        break result;
                                    }
                                }
                                Err(err) => Err(err),
                            };
                            match send_result {
                                Ok(()) => {
                                    let _ = pending.insert(
                                        batch_id,
                                        PendingBatch {
                                            last_cursor,
                                            decision: PendingDecision::Pending,
                                        },
                                    );
                                    if let Some(metrics) = metrics.as_mut() {
                                        metrics.batches_sent.add(1);
                                        metrics.records_sent.add(record_count as u64);
                                        metrics.source_dropped_fields.add(dropped_fields);
                                    }
                                    if drain_deadline.is_some() {
                                        send_worker_command(
                                            &worker.cmd_tx,
                                            WorkerCommand::Drain,
                                            &effect_handler,
                                        )
                                        .await?;
                                    }
                                    otel_debug!(
                                        "journald_receiver.batch_sent",
                                        source_id = config.source_id.as_str(),
                                        batch_id = batch_id,
                                        first_cursor = batch.first_cursor.as_str(),
                                        last_cursor = batch.last_cursor.as_str(),
                                        records = record_count
                                    );
                                }
                                Err(TypedError::ChannelSendError(err)) => {
                                    return Err(terminal_error(
                                        &effect_handler,
                                        format!("failed to send journald batch downstream: {err}"),
                                    ));
                                }
                                Err(err) => {
                                    return Err(terminal_error(
                                        &effect_handler,
                                        format!("failed to send journald batch downstream: {err}"),
                                    ));
                                }
                            }
                        }
                        Some(WorkerEvent::CommitResult { batch_id, cursor, result }) => {
                            match result {
                                Ok(()) => {
                                    let _ = pending.remove(&batch_id);
                                    checkpoint_failures = 0;
                                    if let Some(metrics) = metrics.as_mut() {
                                        metrics.cursor_commits.add(1);
                                    }
                                    otel_debug!(
                                        "journald_receiver.cursor_committed",
                                        source_id = config.source_id.as_str(),
                                        batch_id = batch_id,
                                        cursor = cursor.as_str()
                                    );
                                }
                                Err(err) => {
                                    checkpoint_failures = checkpoint_failures.saturating_add(1);
                                    if let Some(metrics) = metrics.as_mut() {
                                        metrics.checkpoint_failures.add(1);
                                    }
                                    otel_warn!(
                                        "journald_receiver.checkpoint_failed",
                                        source_id = config.source_id.as_str(),
                                        batch_id = batch_id,
                                        error = err.to_string()
                                    );
                                    if checkpoint_failures >= config.checkpoint.max_consecutive_failures {
                                        let _ =
                                            send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                                        drop(event_rx);
                                        join_worker(worker, &effect_handler).await?;
                                        return Err(terminal_error(
                                            &effect_handler,
                                            format!("journald checkpoint failed {checkpoint_failures} consecutive times: {err}"),
                                        ));
                                    }
                                    send_worker_command(
                                        &worker.cmd_tx,
                                        WorkerCommand::Commit {
                                            batch_id,
                                            cursor,
                                        },
                                        &effect_handler,
                                    )
                                    .await?;
                                }
                            }
                        }
                        Some(WorkerEvent::Failed(err)) => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.source_failures.add(1);
                            }
                            let error = err.to_string();
                            otel_warn!(
                                "journald_receiver.source_failed",
                                source_id = config.source_id.as_str(),
                                error = error.as_str()
                            );
                            drop(event_rx);
                            join_worker(worker, &effect_handler).await?;
                            return Err(terminal_error(&effect_handler, error));
                        }
                        Some(WorkerEvent::Stopped) | None => {
                            drop(event_rx);
                            join_worker(worker, &effect_handler).await?;
                            if let Some(deadline) = drain_deadline {
                                if pending.is_empty() {
                                    effect_handler.notify_receiver_drained().await?;
                                    return Ok(terminal_state(deadline, &metrics));
                                }
                            }
                            return Err(terminal_error(&effect_handler, "journald worker stopped unexpectedly"));
                        }
                    }
                }
            }
        }
    }
}

// --- Process-local source lease ----------------------------------------------
//
// The receiver design (see `docs/journald-receiver.md`) requires that, within
// a single process, no two journald receivers target the same concrete journal
// source. The lease key is derived from journal root plus namespace; cross-
// process duplication is left to operators in v1.

static JOURNALD_LEASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

struct SourceLease {
    key: String,
}

impl SourceLease {
    fn acquire(key: &str) -> Result<Self, otap_df_config::error::Error> {
        let mut leases = JOURNALD_LEASES.lock().map_err(|_| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: "journald lease registry is unavailable".to_owned(),
            }
        })?;
        if !leases.insert(key.to_owned()) {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: format!("another journald receiver already targets source `{key}`"),
            });
        }
        Ok(Self {
            key: key.to_owned(),
        })
    }
}

impl Drop for SourceLease {
    fn drop(&mut self) {
        if let Ok(mut leases) = JOURNALD_LEASES.lock() {
            let _ = leases.remove(&self.key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn pending_batch(cursor: &str) -> PendingBatch {
        PendingBatch {
            last_cursor: cursor.to_owned(),
            decision: PendingDecision::Pending,
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_unknown_config_field() {
        let json = serde_json::json!({
            "source_id": "system",
            "unknown_field": true,
        });
        assert!(validate_journald_config(&json).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn validates_default_config() {
        let json = serde_json::json!({});
        validate_journald_config(&json).expect("default config must validate");
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn rejects_config_validation_on_non_linux() {
        let json = serde_json::json!({});
        assert!(validate_journald_config(&json).is_err());
    }

    #[test]
    fn lease_blocks_duplicate_source_in_process() {
        // Use a source key not used elsewhere in tests to avoid races.
        let key = "journald:/test-lease-duplicate:<default>";
        let lease1 = SourceLease::acquire(key).expect("first lease must succeed");
        let lease2 = SourceLease::acquire(key);
        assert!(lease2.is_err(), "duplicate source lease must be rejected");
        drop(lease1);
        // Lease must be released on drop.
        let lease3 = SourceLease::acquire(key).expect("lease must be reacquirable after drop");
        drop(lease3);
    }

    #[test]
    fn distinct_sources_can_coexist() {
        let a = SourceLease::acquire("journald:/test-lease-a:<default>").expect("a");
        let b = SourceLease::acquire("journald:/test-lease-b:<default>").expect("b");
        drop(a);
        drop(b);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn factory_rejects_multi_core_pipeline() {
        let registry = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
        let controller = otap_df_engine::context::ControllerContext::new(registry);
        let pipeline =
            controller.pipeline_context_with("test-group".into(), "test-pipeline".into(), 0, 2, 0);
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(JOURNALD_RECEIVER_URN));
        let receiver_config = ReceiverConfig::new("journald");

        let result = create_journald_receiver(
            pipeline,
            NodeId {
                index: 0,
                name: "journald".into(),
            },
            node_config,
            &receiver_config,
        );

        let Err(err) = result else {
            panic!("journald receiver must reject multi-core source pipelines");
        };

        assert!(
            err.to_string().contains("one-core source pipeline"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn ack_marks_pending_batch_for_commit() {
        let mut pending = BTreeMap::from([(7, pending_batch("cursor-7"))]);

        let effect = apply_pending_ack(&mut pending, 7).expect("ack should apply");

        assert_eq!(
            effect,
            PendingControlEffect {
                command: Some(WorkerCommand::Commit {
                    batch_id: 7,
                    cursor: "cursor-7".to_owned(),
                }),
                fail_nack: false,
                record_ack: true,
                record_nack: false,
                record_rewind: false,
            }
        );
        assert_eq!(
            pending.get(&7).map(|batch| batch.decision),
            Some(PendingDecision::CommitSent)
        );
        assert!(apply_pending_ack(&mut pending, 7).is_none());
    }

    #[test]
    fn nack_with_rewind_clears_pending_and_requests_rewind() {
        let mut pending = BTreeMap::from([(7, pending_batch("cursor-7"))]);

        let effect =
            apply_pending_nack(&mut pending, 7, OnNack::Rewind).expect("nack should apply");

        assert_eq!(
            effect,
            PendingControlEffect {
                command: Some(WorkerCommand::Rewind),
                fail_nack: false,
                record_ack: false,
                record_nack: true,
                record_rewind: true,
            }
        );
        assert!(pending.is_empty());
    }

    #[test]
    fn nack_with_fail_requests_terminal_failure_without_rewind() {
        let mut pending = BTreeMap::from([(7, pending_batch("cursor-7"))]);

        let effect = apply_pending_nack(&mut pending, 7, OnNack::Fail).expect("nack should apply");

        assert_eq!(
            effect,
            PendingControlEffect {
                command: None,
                fail_nack: true,
                record_ack: false,
                record_nack: true,
                record_rewind: false,
            }
        );
        assert_eq!(
            pending.get(&7).map(|batch| batch.decision),
            Some(PendingDecision::FailSent)
        );
        assert!(apply_pending_nack(&mut pending, 7, OnNack::Fail).is_none());
    }
}
