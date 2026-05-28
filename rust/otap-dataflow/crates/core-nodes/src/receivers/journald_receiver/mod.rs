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
use otap_df_telemetry::{otel_info, otel_warn};
use otap_df_telemetry_macros::metric_set;
use serde_json::Value;
#[cfg(target_os = "linux")]
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
mod journal;

use config::RuntimeConfig;
pub use config::{
    BatchConfig, CheckpointConfig, Config, DEFAULT_JOURNAL_ROOT_PATH, DEFAULT_SOURCE_ID,
    JournalConfig, MaxPriority, OnNack, StartAt, severity_number_from_priority,
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

fn validate_journald_config(config: &Value) -> Result<(), otap_df_config::error::Error> {
    let parsed: Config = serde_json::from_value(config.clone()).map_err(|e| {
        otap_df_config::error::Error::InvalidUserConfig {
            error: e.to_string(),
        }
    })?;
    RuntimeConfig::try_from(parsed).map(|_| ())
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
struct WorkerBatch {
    id: u64,
    first_cursor: String,
    last_cursor: String,
    records: otap_df_pdata::otap::OtapArrowRecords,
    record_count: usize,
}

#[cfg(target_os = "linux")]
enum WorkerEvent {
    Batch(WorkerBatch),
    CommitResult {
        batch_id: u64,
        cursor: String,
        result: Result<(), String>,
    },
    Failed(String),
    Stopped,
}

#[cfg(target_os = "linux")]
enum WorkerCommand {
    Commit { batch_id: u64, cursor: String },
    Rewind,
    Shutdown,
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct PendingBatch {
    last_cursor: String,
    decision: PendingDecision,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Eq, PartialEq)]
enum PendingDecision {
    Pending,
    CommitSent,
    RewindSent,
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
) -> Result<(), String> {
    let mut committed_cursor = checkpoint::read_cursor(&checkpoint_path)?;
    let mut reader = journal::SdJournalReader::open(&config, committed_cursor.as_deref())?;
    let mut next_batch_id = 1u64;
    let mut builder = arrow_records_encoder::JournaldArrowRecordsBuilder::new();
    let mut first_cursor = String::new();
    let mut last_cursor = String::new();
    let mut first_record_at = StdInstant::now();
    let mut in_flight = false;

    loop {
        if in_flight {
            match cmd_rx.recv() {
                Ok(WorkerCommand::Commit { batch_id, cursor }) => {
                    let result = checkpoint::write_cursor(&checkpoint_path, &cursor);
                    if result.is_ok() {
                        committed_cursor = Some(cursor.clone());
                        in_flight = false;
                    }
                    let _ = event_tx.blocking_send(WorkerEvent::CommitResult {
                        batch_id,
                        cursor,
                        result,
                    });
                }
                Ok(WorkerCommand::Rewind) => {
                    if committed_cursor.is_none() {
                        return Err(
                            "journald cannot rewind before the first checkpoint is committed"
                                .to_owned(),
                        );
                    }
                    builder = arrow_records_encoder::JournaldArrowRecordsBuilder::new();
                    first_cursor.clear();
                    last_cursor.clear();
                    reader = journal::SdJournalReader::open(&config, committed_cursor.as_deref())?;
                    in_flight = false;
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
                }
                WorkerCommand::Rewind => {
                    if committed_cursor.is_none() {
                        return Err(
                            "journald cannot rewind before the first checkpoint is committed"
                                .to_owned(),
                        );
                    }
                    builder = arrow_records_encoder::JournaldArrowRecordsBuilder::new();
                    first_cursor.clear();
                    last_cursor.clear();
                    reader = journal::SdJournalReader::open(&config, committed_cursor.as_deref())?;
                }
                WorkerCommand::Shutdown => return Ok(()),
            }
        }

        if let Some(entry) = reader.next_entry()? {
            if builder.len() == 0 {
                first_cursor = entry.cursor.clone();
                first_record_at = StdInstant::now();
            }
            last_cursor = entry.cursor.clone();
            builder.append(&entry);
        }

        let should_flush = builder.len() as usize >= config.batch.max_records
            || (builder.len() > 0 && first_record_at.elapsed() >= config.batch.max_flush_period);
        if should_flush {
            let record_count = usize::from(builder.len());
            let records = std::mem::replace(
                &mut builder,
                arrow_records_encoder::JournaldArrowRecordsBuilder::new(),
            )
            .build()
            .map_err(|err| format!("failed to encode journald batch: {err}"))?;
            let batch = WorkerBatch {
                id: next_batch_id,
                first_cursor: std::mem::take(&mut first_cursor),
                last_cursor: std::mem::take(&mut last_cursor),
                records,
                record_count,
            };
            next_batch_id = next_batch_id.saturating_add(1);
            if !send_batch_or_observe_shutdown(event_tx, cmd_rx, batch)? {
                return Ok(());
            }
            in_flight = true;
        }
    }
}

#[cfg(target_os = "linux")]
fn send_batch_or_observe_shutdown(
    event_tx: &tokio::sync::mpsc::Sender<WorkerEvent>,
    cmd_rx: &std::sync::mpsc::Receiver<WorkerCommand>,
    batch: WorkerBatch,
) -> Result<bool, String> {
    let mut event = WorkerEvent::Batch(batch);
    loop {
        match event_tx.try_send(event) {
            Ok(()) => return Ok(true),
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                return Err("journald receiver event channel closed".to_owned());
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(returned)) => {
                event = returned;
                match cmd_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                    Ok(WorkerCommand::Shutdown)
                    | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        return Ok(false);
                    }
                    Ok(WorkerCommand::Commit { .. } | WorkerCommand::Rewind) => {
                        return Err(
                            "journald worker received an unexpected command while handing off a batch"
                                .to_owned(),
                        );
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
                            if let Some(pending_batch) = pending.get_mut(&batch_id) {
                                if pending_batch.decision != PendingDecision::Pending {
                                    continue;
                                }
                                pending_batch.decision = PendingDecision::CommitSent;
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.acks.add(1);
                                }
                                let cursor = pending_batch.last_cursor.clone();
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
                        Ok(NodeControlMsg::Nack(nack)) => {
                            let Some(batch_id) = batch_id_from_call_data(&nack.unwind.route.calldata) else {
                                continue;
                            };
                            if let Some(pending_batch) = pending.get_mut(&batch_id) {
                                if pending_batch.decision != PendingDecision::Pending {
                                    continue;
                                }
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.nacks.add(1);
                                    metrics.rewinds.add(1);
                                }
                                match config.checkpoint.on_nack {
                                    OnNack::Rewind => {
                                        pending_batch.decision = PendingDecision::RewindSent;
                                        pending.clear();
                                        send_worker_command(&worker.cmd_tx, WorkerCommand::Rewind, &effect_handler).await?;
                                    }
                                    OnNack::Fail => {
                                        let _ =
                                            send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                                        drop(event_rx);
                                        join_worker(worker, &effect_handler).await?;
                                        return Err(terminal_error(&effect_handler, "journald batch was Nacked"));
                                    }
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
                            if pending.is_empty() {
                                let _ =
                                    send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                                drop(event_rx);
                                join_worker(worker, &effect_handler).await?;
                                effect_handler.notify_receiver_drained().await?;
                                return Ok(terminal_state(deadline, &metrics));
                            }
                            drain_deadline = Some(deadline);
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
                        Err(e) => return Err(Error::ChannelRecvError(e)),
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
                            let batch_id = batch.id;
                            let last_cursor = batch.last_cursor.clone();
                            let send_result = match effect_handler.try_send_message_with_source_node(pdata) {
                                Ok(()) => Ok(()),
                                Err(TypedError::ChannelSendError(SendError::Full(pdata))) => {
                                    let mut send = Box::pin(effect_handler.send_message_with_source_node(pdata));
                                    loop {
                                        let result = tokio::select! {
                                            biased;

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
                                                        let _ =
                                                            send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                                                        drop(event_rx);
                                                        join_worker(worker, &effect_handler).await?;
                                                        effect_handler.notify_receiver_drained().await?;
                                                        return Ok(terminal_state(deadline, &metrics));
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
                                                    Err(e) => return Err(Error::ChannelRecvError(e)),
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
                                    }
                                    otel_info!(
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
                                    otel_info!(
                                        "journald_receiver.cursor_committed",
                                        source_id = config.source_id.as_str(),
                                        batch_id = batch_id,
                                        cursor = cursor.as_str()
                                    );
                                    if let Some(deadline) = drain_deadline && pending.is_empty() {
                                        let _ =
                                            send_worker_command(&worker.cmd_tx, WorkerCommand::Shutdown, &effect_handler).await;
                                        drop(event_rx);
                                        join_worker(worker, &effect_handler).await?;
                                        effect_handler.notify_receiver_drained().await?;
                                        return Ok(terminal_state(deadline, &metrics));
                                    }
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
                                        error = err.as_str()
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
                                }
                            }
                        }
                        Some(WorkerEvent::Failed(err)) => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.source_failures.add(1);
                            }
                            drop(event_rx);
                            join_worker(worker, &effect_handler).await?;
                            return Err(terminal_error(&effect_handler, err));
                        }
                        Some(WorkerEvent::Stopped) | None => {
                            drop(event_rx);
                            join_worker(worker, &effect_handler).await?;
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

    #[test]
    fn rejects_unknown_config_field() {
        let json = serde_json::json!({
            "source_id": "system",
            "unknown_field": true,
        });
        assert!(validate_journald_config(&json).is_err());
    }

    #[test]
    fn validates_default_config() {
        let json = serde_json::json!({});
        validate_journald_config(&json).expect("default config must validate");
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
}
