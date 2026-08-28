// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle OCI receiver with ACK-driven compound-watermark checkpoints.

use async_trait::async_trait;
use checkpoint::{CheckpointState, CheckpointStore};
use config::{Config, RuntimeConfig};
use linkme::distributed_slice;
use oracle_scraper::{OracleBatch, OracleScraper, OracleScraperError};
use otap_df_channel::error::SendError;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{CallData, Context8u8, NodeControlMsg};
use otap_df_engine::error::{
    Error as EngineError, ReceiverErrorKind, TypedError, format_error_sources,
};
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{
    Interests, MessageSourceLocalEffectHandlerExtension, ProducerEffectHandlerExtension,
    ReceiverFactory,
};
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::{MetricSet, MetricSetSnapshot};
use otap_df_telemetry_macros::metric_set;
use serde_json::Value;
use std::collections::HashSet;
use std::error::Error as StdError;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

mod checkpoint;
mod config;
mod oracle_scraper;

otap_df_telemetry::otel_component_scope!(
    urn = ORACLE_RECEIVER_URN,
    target = "otel.receiver.oracle",
);

/// URN for the Oracle OCI receiver.
pub const ORACLE_RECEIVER_URN: &str = "urn:otel:receiver:oracle";

/// Oracle receiver lifecycle and delivery metrics.
#[metric_set(name = "receiver.oracle")]
#[derive(Debug, Default, Clone)]
pub struct OracleReceiverMetrics {
    /// Receiver starts.
    #[metric(unit = "{start}")]
    pub starts: Counter<u64>,
    /// Oracle page polls.
    #[metric(unit = "{poll}")]
    pub polls: Counter<u64>,
    /// Batches sent downstream.
    #[metric(unit = "{batch}")]
    pub batches_sent: Counter<u64>,
    /// Rows sent downstream.
    #[metric(unit = "{row}")]
    pub rows_sent: Counter<u64>,
    /// Encoded OTLP bytes sent downstream.
    #[metric(unit = "By")]
    pub encoded_bytes_sent: Counter<u64>,
    /// Downstream acknowledgements.
    #[metric(unit = "{ack}")]
    pub acks: Counter<u64>,
    /// Downstream negative acknowledgements.
    #[metric(unit = "{nack}")]
    pub nacks: Counter<u64>,
    /// Replay attempts scheduled after NACK.
    #[metric(unit = "{replay}")]
    pub replays: Counter<u64>,
    /// Durable checkpoint commits.
    #[metric(unit = "{commit}")]
    pub checkpoint_commits: Counter<u64>,
    /// Durable checkpoint write failures.
    #[metric(unit = "{failure}")]
    pub checkpoint_failures: Counter<u64>,
    /// Non-fatal stale checkpoint cleanup failures.
    #[metric(unit = "{failure}")]
    pub checkpoint_cleanup_failures: Counter<u64>,
    /// Clean ingress drains.
    #[metric(unit = "{drain}")]
    pub drains: Counter<u64>,
    /// Immediate shutdowns.
    #[metric(unit = "{shutdown}")]
    pub shutdowns: Counter<u64>,
}

/// Oracle receiver instance.
pub struct OracleReceiver {
    source: OracleScraper,
    config: RuntimeConfig,
    checkpoint: CheckpointStore,
    _lease: SourceLease,
    metrics: Option<MetricSet<OracleReceiverMetrics>>,
}

#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
/// Declares the Oracle receiver as a local receiver factory.
pub static ORACLE_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: ORACLE_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        create_oracle_receiver(pipeline, node, node_config, receiver_config)
    },
    validate_config: validate_oracle_config,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

fn validate_oracle_config(config: &Value) -> Result<(), ConfigError> {
    let config: Config =
        serde_json::from_value(config.clone()).map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })?;
    RuntimeConfig::try_from(config)
        .map(|_| ())
        .map_err(|error| ConfigError::InvalidUserConfig { error })
}

fn create_oracle_receiver(
    pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    receiver_config: &ReceiverConfig,
) -> Result<ReceiverWrapper<OtapPdata>, ConfigError> {
    if pipeline.num_cores() != 1 {
        return Err(ConfigError::InvalidUserConfig {
            error: "Oracle receiver requires a one-core source pipeline; use a topic exporter to fan out downstream"
                .to_owned(),
        });
    }
    let parsed: Config = serde_json::from_value(node_config.config.clone()).map_err(|error| {
        ConfigError::InvalidUserConfig {
            error: error.to_string(),
        }
    })?;
    let config = RuntimeConfig::try_from(parsed)
        .map_err(|error| ConfigError::InvalidUserConfig { error })?;
    let checkpoint = CheckpointStore::new(
        &config.checkpoint.directory,
        pipeline.pipeline_group_id().as_ref(),
        pipeline.pipeline_id().as_ref(),
        receiver_config.name.as_ref(),
        &config.source_id,
        config.config_fingerprint.clone(),
    );
    let lease = SourceLease::acquire(&checkpoint.lease_key())?;
    let source = OracleScraper::new(&config);
    let metrics = Some(pipeline.register_metrics::<OracleReceiverMetrics>());
    Ok(ReceiverWrapper::local(
        OracleReceiver {
            source,
            config,
            checkpoint,
            _lease: lease,
            metrics,
        },
        node,
        node_config,
        receiver_config,
    ))
}

#[derive(Clone, Debug)]
struct PendingBatch {
    id: u64,
    candidate: crate::receivers::sql_polling::CompoundWatermark,
}

#[derive(Clone, Debug)]
struct ReceiverState {
    committed: crate::receivers::sql_polling::CompoundWatermark,
    revision: u64,
    pending: Option<PendingBatch>,
    next_batch_id: u64,
    next_poll: Instant,
    draining: bool,
}

impl ReceiverState {
    fn new(checkpoint: CheckpointState, now: Instant) -> Self {
        Self {
            committed: checkpoint.watermark,
            revision: checkpoint.revision,
            pending: None,
            next_batch_id: 1,
            next_poll: now,
            draining: false,
        }
    }

    fn can_poll(&self) -> bool {
        !self.draining && self.pending.is_none()
    }

    fn schedule_after(&mut self, delay: Duration, now: Instant) {
        self.next_poll = now.checked_add(delay).unwrap_or(now);
    }

    fn record_sent(
        &mut self,
        candidate: crate::receivers::sql_polling::CompoundWatermark,
        poll_interval: Duration,
        now: Instant,
    ) -> u64 {
        debug_assert!(self.pending.is_none());
        let id = self.next_batch_id;
        self.next_batch_id = self.next_batch_id.saturating_add(1);
        self.pending = Some(PendingBatch { id, candidate });
        self.schedule_after(poll_interval, now);
        id
    }

    fn ack_candidate(
        &self,
        batch_id: u64,
    ) -> Option<crate::receivers::sql_polling::CompoundWatermark> {
        self.pending
            .as_ref()
            .filter(|pending| pending.id == batch_id)
            .map(|pending| pending.candidate.clone())
    }

    fn commit(&mut self, checkpoint: CheckpointState) {
        self.committed = checkpoint.watermark;
        self.revision = checkpoint.revision;
        self.pending = None;
    }

    fn nack(&mut self, batch_id: u64, replay_at: Instant) -> bool {
        if self
            .pending
            .as_ref()
            .is_none_or(|pending| pending.id != batch_id)
        {
            return false;
        }
        self.pending = None;
        self.next_poll = replay_at;
        true
    }

    fn begin_drain(&mut self) {
        self.draining = true;
    }
}

fn batch_id_from_call_data(call_data: &CallData) -> Option<u64> {
    call_data.first().copied().map(u64::from)
}

enum SendOutcome {
    Sent,
    Shutdown { deadline: Instant },
    DrainTimedOut { deadline: Instant },
}

enum PollOutcome {
    Complete(Result<Option<OracleBatch>, OracleScraperError>),
    Shutdown { deadline: Instant },
    Drained { deadline: Instant },
}

async fn poll_page_with_control(
    source: &mut OracleScraper,
    watermark: &crate::receivers::sql_polling::CompoundWatermark,
    ctrl: &mut local::ControlChannel<OtapPdata>,
    state: &mut ReceiverState,
    metrics: &mut Option<MetricSet<OracleReceiverMetrics>>,
    drain_deadline: &mut Option<Instant>,
    receiver: &NodeId,
    source_id: &str,
) -> Result<PollOutcome, EngineError> {
    let cancellation = source.cancellation();
    let mut poll = Box::pin(source.poll(watermark));
    loop {
        tokio::select! {
            biased;

            _ = async {
                if let Some(deadline) = *drain_deadline {
                    tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
                } else {
                    std::future::pending::<()>().await;
                }
            }, if drain_deadline.is_some() => {
                let deadline = drain_deadline.expect("deadline checked above");
                let cancellation = cancellation.cancel();
                let finish = async {
                    let cancellation_result = cancellation.await;
                    let _ = poll.as_mut().await;
                    cancellation_result
                };
                if let Ok(Err(error)) = tokio::time::timeout_at(
                    tokio::time::Instant::from_std(deadline),
                    finish,
                )
                .await
                {
                    return Err(receiver_error(receiver.clone(), ReceiverErrorKind::Shutdown, &error));
                }
                return Ok(PollOutcome::Drained { deadline });
            }

            message = ctrl.recv() => {
                match message {
                    Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                        if let Some(metrics) = metrics.as_mut() {
                            let _ = metrics_reporter.report(metrics);
                        }
                    }
                    Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                        if let Some(metrics) = metrics.as_mut() {
                            metrics.drains.add(1);
                        }
                        state.begin_drain();
                        *drain_deadline =
                            Some(drain_deadline.map_or(deadline, |current| current.min(deadline)));
                        otel_info!(
                            "oracle_receiver.drain_ingress",
                            source_id = source_id
                        );
                        let deadline = drain_deadline.expect("drain deadline set above");
                        let cancellation = cancellation.cancel();
                        let finish = async {
                            let cancellation_result = cancellation.await;
                            let _ = poll.as_mut().await;
                            cancellation_result
                        };
                        if let Ok(Err(error)) = tokio::time::timeout_at(
                            tokio::time::Instant::from_std(deadline),
                            finish,
                        )
                        .await
                        {
                            return Err(receiver_error(receiver.clone(), ReceiverErrorKind::Shutdown, &error));
                        }
                        return Ok(PollOutcome::Drained { deadline });
                    }
                    Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                        if let Some(metrics) = metrics.as_mut() {
                            metrics.shutdowns.add(1);
                        }
                        otel_info!(
                            "oracle_receiver.shutdown",
                            source_id = source_id
                        );
                        let cancellation = cancellation.cancel();
                        let finish = async {
                            let cancellation_result = cancellation.await;
                            let _ = poll.as_mut().await;
                            cancellation_result
                        };
                        if let Ok(Err(error)) = tokio::time::timeout_at(
                            tokio::time::Instant::from_std(deadline),
                            finish,
                        )
                        .await
                        {
                            return Err(receiver_error(receiver.clone(), ReceiverErrorKind::Shutdown, &error));
                        }
                        return Ok(PollOutcome::Shutdown { deadline });
                    }
                    Ok(_) => {}
                    Err(error) => {
                        let _ = cancellation.cancel().await;
                        let _ = poll.as_mut().await;
                        return Err(EngineError::ChannelRecvError(error));
                    }
                }
            }

            result = poll.as_mut() => return Ok(PollOutcome::Complete(result)),
        }
    }
}

async fn send_batch_with_control(
    pdata: OtapPdata,
    ctrl: &mut local::ControlChannel<OtapPdata>,
    effect_handler: &local::EffectHandler<OtapPdata>,
    state: &mut ReceiverState,
    metrics: &mut Option<MetricSet<OracleReceiverMetrics>>,
    drain_deadline: &mut Option<Instant>,
) -> Result<SendOutcome, EngineError> {
    match effect_handler.try_send_message_with_source_node(pdata) {
        Ok(()) => Ok(SendOutcome::Sent),
        Err(TypedError::ChannelSendError(SendError::Full(pdata))) => {
            let mut send = Box::pin(effect_handler.send_message_with_source_node(pdata));
            loop {
                tokio::select! {
                    biased;

                    _ = async {
                        if let Some(deadline) = *drain_deadline {
                            tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
                        } else {
                            std::future::pending::<()>().await;
                        }
                    }, if drain_deadline.is_some() => {
                        return Ok(SendOutcome::DrainTimedOut {
                            deadline: drain_deadline.expect("deadline checked above"),
                        });
                    }

                    message = ctrl.recv() => {
                        match message {
                            Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                                if let Some(metrics) = metrics.as_mut() {
                                    let _ = metrics_reporter.report(metrics);
                                }
                            }
                            Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                                state.begin_drain();
                                *drain_deadline = Some(
                                    drain_deadline.map_or(deadline, |current| current.min(deadline))
                                );
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.drains.add(1);
                                }
                            }
                            Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.shutdowns.add(1);
                                }
                                return Ok(SendOutcome::Shutdown { deadline });
                            }
                            Ok(_) => {}
                            Err(error) => return Err(EngineError::ChannelRecvError(error)),
                        }
                    }

                    result = send.as_mut() => {
                        result?;
                        return Ok(SendOutcome::Sent);
                    }
                }
            }
        }
        Err(error) => Err(error.into()),
    }
}

async fn read_checkpoint(
    store: &CheckpointStore,
    initial: &crate::receivers::sql_polling::CompoundWatermark,
    receiver: &NodeId,
) -> Result<CheckpointState, EngineError> {
    let store = store.clone();
    let loaded = tokio::task::spawn_blocking(move || store.read())
        .await
        .map_err(|error| receiver_error(receiver.clone(), ReceiverErrorKind::Other, &error))?
        .map_err(|error| {
            receiver_error(receiver.clone(), ReceiverErrorKind::Configuration, &error)
        })?;
    Ok(loaded.unwrap_or_else(|| CheckpointState {
        revision: 0,
        watermark: initial.clone(),
    }))
}

async fn commit_checkpoint(
    store: &CheckpointStore,
    revision: u64,
    candidate: &crate::receivers::sql_polling::CompoundWatermark,
    max_failures: u32,
    receiver: &NodeId,
    source_id: &str,
    batch_id: u64,
    metrics: &mut Option<MetricSet<OracleReceiverMetrics>>,
    cleanup_failure_streak: &mut u32,
) -> Result<CheckpointState, EngineError> {
    let mut failures = 0u32;
    loop {
        let store = store.clone();
        let candidate = candidate.clone();
        let result = tokio::task::spawn_blocking(move || store.write(revision, &candidate))
            .await
            .map_err(|error| receiver_error(receiver.clone(), ReceiverErrorKind::Other, &error))?;
        match result {
            Ok((checkpoint, outcome)) => {
                if let Some(metrics) = metrics.as_mut() {
                    metrics.checkpoint_commits.add(1);
                    metrics
                        .checkpoint_cleanup_failures
                        .add(outcome.cleanup_failures as u64);
                }
                if outcome.cleanup_failures > 0 {
                    *cleanup_failure_streak = cleanup_failure_streak.saturating_add(1);
                    otel_warn!(
                        "oracle_receiver.checkpoint_cleanup_failed",
                        source_id = source_id,
                        failures = outcome.cleanup_failures as u64
                    );
                    if *cleanup_failure_streak >= max_failures {
                        let error = std::io::Error::other(format!(
                            "checkpoint cleanup failed after {cleanup_failure_streak} consecutive commits"
                        ));
                        return Err(receiver_error(
                            receiver.clone(),
                            ReceiverErrorKind::Other,
                            &error,
                        ));
                    }
                } else {
                    *cleanup_failure_streak = 0;
                }
                otel_debug!(
                    "oracle_receiver.checkpoint_committed",
                    source_id = source_id,
                    batch_id = batch_id,
                    revision = checkpoint.revision
                );
                return Ok(checkpoint);
            }
            Err(error) => {
                failures = failures.saturating_add(1);
                if let Some(metrics) = metrics.as_mut() {
                    metrics.checkpoint_failures.add(1);
                }
                otel_warn!(
                    "oracle_receiver.checkpoint_failed",
                    source_id = source_id,
                    batch_id = batch_id,
                    attempt = failures as u64,
                    error = error.to_string()
                );
                if failures >= max_failures {
                    return Err(receiver_error(
                        receiver.clone(),
                        ReceiverErrorKind::Other,
                        &error,
                    ));
                }
            }
        }
    }
}

async fn shutdown_source(source: &mut OracleScraper, receiver: &NodeId) -> Result<(), EngineError> {
    source
        .shutdown()
        .await
        .map_err(|error| source_error(receiver.clone(), source, &error))
}

fn source_error(
    receiver: NodeId,
    source: &OracleScraper,
    error: &OracleScraperError,
) -> EngineError {
    receiver_error(receiver, source.classify_error(error), error)
}

fn receiver_error(
    receiver: NodeId,
    kind: ReceiverErrorKind,
    error: &(dyn StdError + 'static),
) -> EngineError {
    EngineError::ReceiverError {
        receiver,
        kind,
        error: error.to_string(),
        source_detail: format_error_sources(error),
    }
}

fn terminal_state(
    deadline: Instant,
    metrics: &Option<MetricSet<OracleReceiverMetrics>>,
) -> TerminalState {
    if let Some(metrics) = metrics {
        TerminalState::new(deadline, [metrics.snapshot()])
    } else {
        TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, [])
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for OracleReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let OracleReceiver {
            mut source,
            config,
            checkpoint,
            _lease,
            mut metrics,
        } = *self;
        let receiver_id = effect_handler.receiver_id();
        let checkpoint_state =
            read_checkpoint(&checkpoint, &config.initial_watermark, &receiver_id).await?;
        let mut state = ReceiverState::new(checkpoint_state, Instant::now());
        source
            .start()
            .await
            .map_err(|error| source_error(receiver_id.clone(), &source, &error))?;
        if let Some(metrics) = metrics.as_mut() {
            metrics.starts.add(1);
        }
        if let Err(error) = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await
        {
            shutdown_source(&mut source, &receiver_id).await?;
            return Err(error);
        }
        let mut drain_deadline = None;
        let mut cleanup_failure_streak = 0u32;

        otel_info!(
            "oracle_receiver.start",
            source_id = config.source_id.as_str(),
            checkpoint_revision = state.revision
        );

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
                    let deadline = drain_deadline.expect("deadline checked above");
                    if state.pending.is_some() {
                        otel_warn!(
                            "oracle_receiver.drain_timeout",
                            source_id = config.source_id.as_str(),
                            message = "Drain deadline reached while a batch awaited ACK/NACK; checkpoint was not advanced"
                        );
                    }
                    shutdown_source(&mut source, &receiver_id).await?;
                    effect_handler.notify_receiver_drained().await?;
                    return Ok(terminal_state(deadline, &metrics));
                }

                message = ctrl.recv() => {
                    match message {
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            if let Some(metrics) = metrics.as_mut() {
                                let _ = metrics_reporter.report(metrics);
                            }
                        }
                        Ok(NodeControlMsg::Ack(ack)) => {
                            let Some(batch_id) = batch_id_from_call_data(&ack.unwind.route.calldata) else {
                                continue;
                            };
                            let Some(candidate) = state.ack_candidate(batch_id) else {
                                continue;
                            };
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.acks.add(1);
                            }
                            let committed = match commit_checkpoint(
                                &checkpoint,
                                state.revision,
                                &candidate,
                                config.checkpoint.max_consecutive_failures,
                                &receiver_id,
                                &config.source_id,
                                batch_id,
                                &mut metrics,
                                &mut cleanup_failure_streak,
                            )
                            .await
                            {
                                Ok(committed) => committed,
                                Err(error) => {
                                    shutdown_source(&mut source, &receiver_id).await?;
                                    return Err(error);
                                }
                            };
                            state.commit(committed);
                            if state.draining {
                                let deadline = drain_deadline.unwrap_or_else(Instant::now);
                                shutdown_source(&mut source, &receiver_id).await?;
                                effect_handler.notify_receiver_drained().await?;
                                return Ok(terminal_state(deadline, &metrics));
                            }
                        }
                        Ok(NodeControlMsg::Nack(nack)) => {
                            let Some(batch_id) = batch_id_from_call_data(&nack.unwind.route.calldata) else {
                                continue;
                            };
                            let replay_at = Instant::now()
                                .checked_add(config.nack_backoff)
                                .unwrap_or_else(Instant::now);
                            if state.nack(batch_id, replay_at) {
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.nacks.add(1);
                                    metrics.replays.add(1);
                                }
                                otel_warn!(
                                    "oracle_receiver.batch_nacked",
                                    source_id = config.source_id.as_str(),
                                    batch_id = batch_id,
                                    backoff_millis = config.nack_backoff.as_millis() as u64
                                );
                                if state.draining {
                                    let deadline = drain_deadline.unwrap_or_else(Instant::now);
                                    shutdown_source(&mut source, &receiver_id).await?;
                                    effect_handler.notify_receiver_drained().await?;
                                    return Ok(terminal_state(deadline, &metrics));
                                }
                            }
                        }
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.drains.add(1);
                            }
                            state.begin_drain();
                            drain_deadline =
                                Some(drain_deadline.map_or(deadline, |current: Instant| current.min(deadline)));
                            otel_info!(
                                "oracle_receiver.drain_ingress",
                                source_id = config.source_id.as_str()
                            );
                            if state.pending.is_none() {
                                shutdown_source(&mut source, &receiver_id).await?;
                                effect_handler.notify_receiver_drained().await?;
                                return Ok(terminal_state(deadline, &metrics));
                            }
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.shutdowns.add(1);
                            }
                            otel_info!(
                                "oracle_receiver.shutdown",
                                source_id = config.source_id.as_str()
                            );
                            shutdown_source(&mut source, &receiver_id).await?;
                            return Ok(terminal_state(deadline, &metrics));
                        }
                        Ok(_) => {}
                        Err(error) => {
                            shutdown_source(&mut source, &receiver_id).await?;
                            return Err(EngineError::ChannelRecvError(error));
                        }
                    }
                }

                _ = async {
                    if state.can_poll() {
                        tokio::time::sleep_until(tokio::time::Instant::from_std(state.next_poll)).await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                }, if state.can_poll() => {
                    if let Some(metrics) = metrics.as_mut() {
                        metrics.polls.add(1);
                    }
                    let watermark = state.committed.clone();
                    let page = match poll_page_with_control(
                        &mut source,
                        &watermark,
                        &mut ctrl,
                        &mut state,
                        &mut metrics,
                        &mut drain_deadline,
                        &receiver_id,
                        &config.source_id,
                    )
                    .await?
                    {
                        PollOutcome::Complete(Ok(page)) => page,
                        PollOutcome::Complete(Err(error)) => {
                            let engine_error =
                                source_error(receiver_id.clone(), &source, &error);
                            shutdown_source(&mut source, &receiver_id).await?;
                            return Err(engine_error);
                        }
                        PollOutcome::Shutdown { deadline } => {
                            shutdown_source(&mut source, &receiver_id).await?;
                            return Ok(terminal_state(deadline, &metrics));
                        }
                        PollOutcome::Drained { deadline } => {
                            shutdown_source(&mut source, &receiver_id).await?;
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(terminal_state(deadline, &metrics));
                        }
                    };
                    let now = Instant::now();
                    let Some(OracleBatch {
                        mut pdata,
                        candidate,
                        row_count,
                        encoded_bytes,
                    }) = page
                    else {
                        state.schedule_after(config.poll_interval, now);
                        continue;
                    };

                    let batch_id = state.next_batch_id;
                    let mut call_data = CallData::new();
                    call_data.push(Context8u8::from(batch_id));
                    effect_handler.subscribe_to(
                        Interests::ACKS_OR_NACKS,
                        call_data,
                        &mut pdata,
                    );
                    let send_outcome = match send_batch_with_control(
                        pdata,
                        &mut ctrl,
                        &effect_handler,
                        &mut state,
                        &mut metrics,
                        &mut drain_deadline,
                    )
                    .await
                    {
                        Ok(outcome) => outcome,
                        Err(error) => {
                            shutdown_source(&mut source, &receiver_id).await?;
                            return Err(error);
                        }
                    };
                    match send_outcome {
                        SendOutcome::Sent => {
                            let recorded_id =
                                state.record_sent(candidate, config.poll_interval, now);
                            debug_assert_eq!(recorded_id, batch_id);
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.batches_sent.add(1);
                                metrics.rows_sent.add(row_count as u64);
                                metrics.encoded_bytes_sent.add(encoded_bytes as u64);
                            }
                            otel_debug!(
                                "oracle_receiver.batch_sent",
                                source_id = config.source_id.as_str(),
                                batch_id = batch_id,
                                rows = row_count as u64,
                                encoded_bytes = encoded_bytes as u64
                            );
                        }
                        SendOutcome::Shutdown { deadline } => {
                            shutdown_source(&mut source, &receiver_id).await?;
                            return Ok(terminal_state(deadline, &metrics));
                        }
                        SendOutcome::DrainTimedOut { deadline } => {
                            shutdown_source(&mut source, &receiver_id).await?;
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(terminal_state(deadline, &metrics));
                        }
                    }
                }
            }
        }
    }
}

static ORACLE_SOURCE_LEASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

struct SourceLease {
    key: String,
}

impl SourceLease {
    fn acquire(key: &str) -> Result<Self, ConfigError> {
        let mut leases =
            ORACLE_SOURCE_LEASES
                .lock()
                .map_err(|_| ConfigError::InvalidUserConfig {
                    error: "Oracle source lease registry is unavailable".to_owned(),
                })?;
        if !leases.insert(key.to_owned()) {
            return Err(ConfigError::InvalidUserConfig {
                error: "another Oracle receiver already owns this checkpoint source".to_owned(),
            });
        }
        Ok(Self {
            key: key.to_owned(),
        })
    }
}

impl Drop for SourceLease {
    fn drop(&mut self) {
        if let Ok(mut leases) = ORACLE_SOURCE_LEASES.lock() {
            let _ = leases.remove(&self.key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receivers::sql_polling::CompoundWatermark;

    fn checkpoint(revision: u64, tie_breaker: i64) -> CheckpointState {
        CheckpointState {
            revision,
            watermark: CompoundWatermark {
                timestamp: "2026-01-01 00:00:00".to_owned(),
                tie_breaker,
            },
        }
    }

    /// Scenario: a matching ACK is durably committed after a page is sent.
    /// Guarantees: the checkpoint advances to the final emitted row and clears in-flight state.
    #[test]
    fn ack_commit_advances_watermark() {
        let now = Instant::now();
        let mut state = ReceiverState::new(checkpoint(3, 10), now);
        let id = state.record_sent(checkpoint(0, 20).watermark, Duration::from_secs(1), now);
        let candidate = state.ack_candidate(id).expect("candidate");

        state.commit(CheckpointState {
            revision: 4,
            watermark: candidate,
        });

        assert_eq!(state.revision, 4);
        assert_eq!(state.committed.tie_breaker, 20);
        assert!(state.pending.is_none());
    }

    /// Scenario: a page is NACKed while one batch is in flight.
    /// Guarantees: the committed tuple is retained and polling resumes only after backoff.
    #[test]
    fn nack_retains_checkpoint_and_schedules_replay() {
        let now = Instant::now();
        let mut state = ReceiverState::new(checkpoint(3, 10), now);
        let id = state.record_sent(checkpoint(0, 20).watermark, Duration::from_secs(1), now);
        let replay_at = now + Duration::from_secs(2);

        assert!(state.nack(id, replay_at));
        assert_eq!(state.revision, 3);
        assert_eq!(state.committed.tie_breaker, 10);
        assert_eq!(state.next_poll, replay_at);
        assert!(state.pending.is_none());
    }

    /// Scenario: a receiver begins draining with a batch still in flight.
    /// Guarantees: no new Oracle page can be polled while ACK/NACK completion is awaited.
    #[test]
    fn drain_stops_new_polls_until_pending_resolves() {
        let now = Instant::now();
        let mut state = ReceiverState::new(checkpoint(0, 0), now);
        let _ = state.record_sent(checkpoint(0, 1).watermark, Duration::from_secs(1), now);
        state.begin_drain();

        assert!(!state.can_poll());
        assert!(state.pending.is_some());
    }

    /// Scenario: delayed ACK/NACK feedback arrives for a batch that is no longer in flight.
    /// Guarantees: stale feedback cannot advance, clear, or reschedule the current checkpoint.
    #[test]
    fn stale_feedback_does_not_change_state() {
        let now = Instant::now();
        let mut state = ReceiverState::new(checkpoint(2, 10), now);
        let current_id =
            state.record_sent(checkpoint(0, 20).watermark, Duration::from_secs(1), now);
        let original_next_poll = state.next_poll;

        assert!(state.ack_candidate(current_id + 1).is_none());
        assert!(!state.nack(current_id + 1, now + Duration::from_secs(5)));
        assert_eq!(state.revision, 2);
        assert_eq!(state.committed.tie_breaker, 10);
        assert_eq!(state.next_poll, original_next_poll);
        assert_eq!(
            state.pending.as_ref().map(|pending| pending.id),
            Some(current_id)
        );
    }

    /// Scenario: duplicate Oracle receivers target one process-local checkpoint source.
    /// Guarantees: only one active owner can advance a checkpoint at a time.
    #[test]
    fn source_lease_rejects_duplicate_owner() {
        let key = "oracle-test-source-lease";
        let first = SourceLease::acquire(key).expect("first lease");
        assert!(SourceLease::acquire(key).is_err());
        drop(first);
        assert!(SourceLease::acquire(key).is_ok());
    }
}
