// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration coverage for transient-NACK Kafka replay.

use super::super::replay::{ReplayConsumerOperations, ReplayOperation};
use super::*;
use crate::receivers::kafka_receiver::retry::BeginRetry;
use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_config::observed_state::{ObservedStateSettings, SendPolicy};
use otel_arrow_dfe_config::pipeline::{PipelineConfigBuilder, PipelineType};
use otel_arrow_dfe_config::policy::{ChannelCapacityPolicy, TelemetryPolicy};
use otel_arrow_dfe_config::{DeployedPipelineKey, PipelineGroupId, PipelineId};
use otel_arrow_dfe_core_nodes::processors::retry_processor::RETRY_PROCESSOR_URN;
use otel_arrow_dfe_engine::ConsumerEffectHandlerExtension;
use otel_arrow_dfe_engine::ExporterFactory;
use otel_arrow_dfe_engine::config::ExporterConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{
    AckMsg, NackMsg, NodeControlMsg, pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
};
use otel_arrow_dfe_engine::entity_context::set_pipeline_entity_key;
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message as EngineMessage};
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES;
use otel_arrow_dfe_otap::OTAP_PIPELINE_FACTORY;
use otel_arrow_dfe_state::store::ObservedStateStore;
use otel_arrow_dfe_telemetry::InternalTelemetrySystem;
use serde_json::json;
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Arc, LazyLock, Mutex};

const REPLAY_TOPOLOGY_EXPORTER_URN: &str = "urn:otel:exporter:kafka-replay-topology-test";

static REPLAY_TOPOLOGY_ATTEMPTS: LazyLock<Mutex<Vec<u64>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
static REPLAY_TOPOLOGY_ACKED: AtomicBool = AtomicBool::new(false);

struct ReplayTopologyExporter;

#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
static REPLAY_TOPOLOGY_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: REPLAY_TOPOLOGY_EXPORTER_URN,
    create:
        |_pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         exporter_config: &ExporterConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            Ok(ExporterWrapper::local(
                ReplayTopologyExporter,
                node,
                node_config,
                exporter_config,
            ))
        },
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |_| Ok(()),
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for ReplayTopologyExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        loop {
            match msg_chan.recv().await? {
                EngineMessage::Control(NodeControlMsg::Shutdown { .. }) => break,
                EngineMessage::PData(data) => {
                    let retry_count = data
                        .source_route()
                        .and_then(|route| route.calldata.first().copied())
                        .map(u64::from)
                        .expect("retry processor must provide retry state");
                    let attempt = {
                        let mut attempts = REPLAY_TOPOLOGY_ATTEMPTS
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        attempts.push(retry_count);
                        attempts.len()
                    };
                    if attempt <= 2 {
                        effect_handler
                            .notify_nack(NackMsg::new("topology retry", data))
                            .await?;
                    } else {
                        effect_handler.notify_ack(AckMsg::new(data)).await?;
                        REPLAY_TOPOLOGY_ACKED.store(true, AtomicOrdering::Release);
                    }
                }
                _ => {}
            }
        }
        Ok(TerminalState::default())
    }
}

struct FailingReplayConsumer {
    failed_operation: ReplayOperation,
    calls: RefCell<Vec<ReplayOperation>>,
}

impl FailingReplayConsumer {
    fn new(failed_operation: ReplayOperation) -> Self {
        Self {
            failed_operation,
            calls: RefCell::new(Vec::new()),
        }
    }

    fn execute(&self, operation: ReplayOperation) -> Result<(), KafkaError> {
        self.calls.borrow_mut().push(operation);
        if operation != self.failed_operation {
            return Ok(());
        }
        match operation {
            ReplayOperation::Pause | ReplayOperation::Resume => Err(KafkaError::PauseResume(
                format!("injected {} failure", operation.as_str()),
            )),
            ReplayOperation::Seek => Err(KafkaError::Seek("injected seek failure".to_string())),
        }
    }
}

impl ReplayConsumerOperations for FailingReplayConsumer {
    fn pause_partition(&self, _topic: &str, _partition: i32) -> Result<(), KafkaError> {
        self.execute(ReplayOperation::Pause)
    }

    fn seek_partition(
        &self,
        _topic: &str,
        _partition: i32,
        _offset: i64,
    ) -> Result<(), KafkaError> {
        self.execute(ReplayOperation::Seek)
    }

    fn resume_partition(&self, _topic: &str, _partition: i32) -> Result<(), KafkaError> {
        self.execute(ReplayOperation::Resume)
    }
}

#[derive(Copy, Clone)]
enum ReplayOwnershipChange {
    Revoke,
    Reassign,
}

impl ReplayOwnershipChange {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Revoke => "revocation",
            Self::Reassign => "reassignment",
        }
    }
}

struct OwnershipChangingReplayConsumer {
    change_during: ReplayOperation,
    change: ReplayOwnershipChange,
    rebalance_state: Arc<RebalanceState>,
    calls: RefCell<Vec<ReplayOperation>>,
    changed: Cell<bool>,
}

impl OwnershipChangingReplayConsumer {
    fn new(
        change_during: ReplayOperation,
        change: ReplayOwnershipChange,
        rebalance_state: Arc<RebalanceState>,
    ) -> Self {
        Self {
            change_during,
            change,
            rebalance_state,
            calls: RefCell::new(Vec::new()),
            changed: Cell::new(false),
        }
    }

    fn execute(&self, operation: ReplayOperation, topic: &str, partition: i32) {
        self.calls.borrow_mut().push(operation);
        if operation != self.change_during || self.changed.replace(true) {
            return;
        }

        let revoked_generation = self.rebalance_state.current_generation(topic, partition);
        self.rebalance_state
            .push_revoked_for_test(topic, partition, revoked_generation);
        self.rebalance_state
            .set_assignment_for_test(&TopicPartitionList::new());
        if matches!(self.change, ReplayOwnershipChange::Reassign) {
            let mut reassignment = TopicPartitionList::new();
            let _ = reassignment.add_partition(topic, partition);
            self.rebalance_state.set_assignment_for_test(&reassignment);
        }
    }
}

impl ReplayConsumerOperations for OwnershipChangingReplayConsumer {
    fn pause_partition(&self, topic: &str, partition: i32) -> Result<(), KafkaError> {
        self.execute(ReplayOperation::Pause, topic, partition);
        Ok(())
    }

    fn seek_partition(&self, topic: &str, partition: i32, _offset: i64) -> Result<(), KafkaError> {
        self.execute(ReplayOperation::Seek, topic, partition);
        Ok(())
    }

    fn resume_partition(&self, topic: &str, partition: i32) -> Result<(), KafkaError> {
        self.execute(ReplayOperation::Resume, topic, partition);
        Ok(())
    }
}

/// Receives until the requested partition is observed or the shared deadline expires.
async fn recv_partition_delivery(
    receiver: &mut KafkaReceiverHarness,
    partition: i32,
    timeout: Duration,
) -> OtapPdata {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let pdata = receiver.try_recv_pdata(remaining).await.unwrap_or_else(|| {
            panic!("partition {partition} was not delivered within {timeout:?}")
        });
        let route = pdata
            .source_route()
            .expect("redelivered pdata carries source calldata");
        let (_, delivered_partition, _, _) = decode_calldata(&route.calldata);
        if delivered_partition == partition {
            return pdata;
        }
    }
}

/// Scenario: A partition is revoked while its failed offset is in transient-NACK backoff.
/// Guarantees: Reconciliation drops tracking and retry state without advancing the failed offset.
#[test]
fn reconcile_revocation_drops_active_retry_without_advancing_offset() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");
    let mut assignment = TopicPartitionList::new();
    let _ = assignment.add_partition("traces", 0);
    receiver
        .rebalance_state
        .set_assignment_for_test(&assignment);
    let ownership_raw = receiver.rebalance_state.current_generation("traces", 0);
    let ownership_generation = OwnershipGeneration::from_raw(ownership_raw);
    receiver
        .offset_tracker
        .track("traces", 0, 12, ownership_raw);
    let original_delivery =
        receiver
            .retry_manager
            .delivery_generation("traces", 0, ownership_generation);
    let rewind = receiver
        .offset_tracker
        .prepare_replay("traces", 0, 12, ownership_raw)
        .expect("failed offset remains pending");
    let retry_config = receiver
        .config
        .replay_backoff()
        .expect("replay config")
        .clone();
    let replay_delivery = receiver.retry_manager.begin_retry(
        BeginRetry {
            topic: "traces",
            partition: 0,
            ownership_generation,
            failed_offset: 12,
            rewind_offset: rewind,
            paused: true,
            now: Instant::now(),
        },
        &retry_config,
    );
    assert_ne!(original_delivery, replay_delivery);

    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, ownership_raw);
    receiver.reconcile_rebalance_state();

    assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 0);
    assert!(
        !receiver
            .offset_tracker
            .committable_snapshot()
            .contains_key(&("traces".to_string(), 0))
    );
    assert_eq!(receiver.retry_manager.paused_count(), 0);
    assert!(receiver.retry_manager.next_deadline().is_none());
    assert_eq!(
        receiver
            .retry_manager
            .feedback_ownership_generation("traces", 0, replay_delivery),
        Some(ownership_generation),
    );
}

/// Scenario: Pause, seek, and resume failures occur in the production replay handler.
/// Guarantees: Every failure preserves offset 12 and schedules another bounded operation attempt.
#[test]
fn replay_operation_failures_preserve_failed_offset_watermark() {
    let scenarios = [
        (
            ReplayOperation::Pause,
            false,
            vec![ReplayOperation::Pause],
            0,
        ),
        (ReplayOperation::Seek, true, vec![ReplayOperation::Seek], 1),
        (
            ReplayOperation::Resume,
            true,
            vec![ReplayOperation::Seek, ReplayOperation::Resume],
            1,
        ),
    ];

    for (failed_operation, initially_paused, expected_calls, expected_paused) in scenarios {
        let cfg = manual_traces_config_with_replay_backoff(
            "localhost:9092",
            "operation-failure-group",
            "traces",
            1,
            4,
        );
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");
        let mut assignment = TopicPartitionList::new();
        let _ = assignment.add_partition("traces", 0);
        receiver
            .rebalance_state
            .set_assignment_for_test(&assignment);
        let ownership_raw = receiver.rebalance_state.current_generation("traces", 0);
        let ownership_generation = OwnershipGeneration::from_raw(ownership_raw);
        receiver
            .offset_tracker
            .track("traces", 0, 12, ownership_raw);
        receiver
            .offset_tracker
            .track("traces", 0, 13, ownership_raw);
        assert!(!receiver.offset_tracker.acknowledge("traces", 0, 13));
        let _ = receiver
            .retry_manager
            .delivery_generation("traces", 0, ownership_generation);
        let rewind = receiver
            .offset_tracker
            .prepare_replay("traces", 0, 12, ownership_raw)
            .expect("failed offset remains pending");
        let retry_config = receiver
            .config
            .replay_backoff()
            .expect("replay config")
            .clone();
        let _ = receiver.retry_manager.begin_retry(
            BeginRetry {
                topic: "traces",
                partition: 0,
                ownership_generation,
                failed_offset: 12,
                rewind_offset: rewind,
                paused: initially_paused,
                now: Instant::now() - Duration::from_secs(1),
            },
            &retry_config,
        );
        let consumer = FailingReplayConsumer::new(failed_operation);

        receiver.process_due_replays(&consumer);

        assert_eq!(
            receiver
                .offset_tracker
                .committable_snapshot()
                .get(&("traces".to_string(), 0)),
            Some(&12),
        );
        assert!(receiver.retry_manager.blocks_delivery("traces", 0));
        assert!(receiver.retry_manager.next_deadline().is_some());
        assert_eq!(receiver.retry_manager.paused_count(), expected_paused);
        assert_eq!(*consumer.calls.borrow(), expected_calls);
    }
}

/// Scenario: A partition is revoked or reassigned during each pause, seek, or resume phase.
/// Guarantees: Stale replay work stops, its retry state is dropped, and its offset is not advanced.
#[test]
fn replay_phase_ownership_change_discards_stale_state_without_advancing_offset() {
    let scenarios = [
        (ReplayOperation::Pause, false, vec![ReplayOperation::Pause]),
        (ReplayOperation::Seek, true, vec![ReplayOperation::Seek]),
        (
            ReplayOperation::Resume,
            true,
            vec![ReplayOperation::Seek, ReplayOperation::Resume],
        ),
    ];

    for change in [
        ReplayOwnershipChange::Revoke,
        ReplayOwnershipChange::Reassign,
    ] {
        for (change_during, initially_paused, expected_calls) in &scenarios {
            let cfg = manual_traces_config_with_replay_backoff(
                "localhost:9092",
                "operation-ownership-change-group",
                "traces",
                1,
                4,
            );
            let ctx = make_pipeline_ctx();
            let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");
            let mut assignment = TopicPartitionList::new();
            let _ = assignment.add_partition("traces", 0);
            receiver
                .rebalance_state
                .set_assignment_for_test(&assignment);
            let ownership_raw = receiver.rebalance_state.current_generation("traces", 0);
            let ownership_generation = OwnershipGeneration::from_raw(ownership_raw);
            receiver
                .offset_tracker
                .track("traces", 0, 12, ownership_raw);
            receiver
                .offset_tracker
                .track("traces", 0, 13, ownership_raw);
            assert!(!receiver.offset_tracker.acknowledge("traces", 0, 13));
            let _ = receiver
                .retry_manager
                .delivery_generation("traces", 0, ownership_generation);
            let rewind = receiver
                .offset_tracker
                .prepare_replay("traces", 0, 12, ownership_raw)
                .expect("failed offset remains pending");
            let retry_config = receiver
                .config
                .replay_backoff()
                .expect("replay config")
                .clone();
            let replay_delivery = receiver.retry_manager.begin_retry(
                BeginRetry {
                    topic: "traces",
                    partition: 0,
                    ownership_generation,
                    failed_offset: 12,
                    rewind_offset: rewind,
                    paused: *initially_paused,
                    now: Instant::now() - Duration::from_secs(1),
                },
                &retry_config,
            );
            let consumer = OwnershipChangingReplayConsumer::new(
                *change_during,
                change,
                receiver.rebalance_state_for_test(),
            );

            receiver.process_due_replays(&consumer);

            match change {
                ReplayOwnershipChange::Revoke => assert!(
                    !receiver.rebalance_state.is_assigned("traces", 0),
                    "revocation during {} must remove the assignment",
                    change_during.as_str(),
                ),
                ReplayOwnershipChange::Reassign => assert!(
                    receiver.rebalance_state.current_generation("traces", 0) > ownership_raw,
                    "reassignment during {} must allocate a new ownership generation",
                    change_during.as_str(),
                ),
            }
            assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 0);
            assert!(
                !receiver
                    .offset_tracker
                    .committable_snapshot()
                    .contains_key(&("traces".to_string(), 0)),
                "{} during {} must not advance the revoked ownership period's offset",
                change.as_str(),
                change_during.as_str(),
            );
            assert_eq!(receiver.retry_manager.paused_count(), 0);
            assert!(receiver.retry_manager.next_deadline().is_none());
            assert_eq!(
                receiver
                    .retry_manager
                    .feedback_ownership_generation("traces", 0, replay_delivery,),
                Some(ownership_generation),
            );
            assert_eq!(*consumer.calls.borrow(), *expected_calls);

            receiver.reconcile_rebalance_state();
            assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 0);
            assert_eq!(receiver.retry_manager.paused_count(), 0);
        }
    }
}

fn manual_traces_config_with_commit_and_skip(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
) -> KafkaReceiverConfig {
    let builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
        .with_traces(
            SignalConfig::new(vec![traces_topic.to_string()])
                .with_encoding(MessageFormat::OtlpProto),
        )
        .with_commit(CommitConfig {
            mode: ConfigCommitMode::Manual,
            interval_ms: None,
        })
        .with_transient_nack(TransientNackConfig {
            mode: TransientNackMode::CommitAndSkip,
            ..Default::default()
        })
        .with_auto_offset_reset(AutoOffsetReset::Earliest)
        .with_isolation_level(IsolationLevel::ReadUncommitted);
    KafkaReceiverConfig::try_from(builder).expect("test commit-and-skip config valid")
}

fn manual_traces_config_with_replay(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
) -> KafkaReceiverConfig {
    manual_traces_config_with_replay_backoff(brokers, group_id, traces_topic, 10, 40)
}

fn manual_traces_config_with_replay_backoff(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
) -> KafkaReceiverConfig {
    let builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
        .with_traces(
            SignalConfig::new(vec![traces_topic.to_string()])
                .with_encoding(MessageFormat::OtlpProto),
        )
        .with_commit(CommitConfig {
            mode: ConfigCommitMode::Manual,
            interval_ms: None,
        })
        .with_transient_nack(TransientNackConfig {
            mode: TransientNackMode::Replay,
            initial_backoff_ms,
            max_backoff_ms,
        })
        .with_enable_idempotency(true)
        .with_auto_offset_reset(AutoOffsetReset::Earliest)
        .with_isolation_level(IsolationLevel::ReadUncommitted);
    KafkaReceiverConfig::try_from(builder).expect("test replay config valid")
}

fn run_retry_topology_pipeline(bootstrap_servers: String) {
    const TOPIC: &str = "retry-topology-fallback-traces";
    const GROUP: &str = "retry-topology-fallback-group";
    let pipeline_group_id: PipelineGroupId = "kafka-replay-topology".into();
    let pipeline_id: PipelineId = "retry-fallback".into();
    let config = PipelineConfigBuilder::new()
        .add_receiver(
            "kafka_receiver",
            KAFKA_RECEIVER_URN,
            Some(json!({
                "brokers": bootstrap_servers.clone(),
                "group_id": GROUP,
                "client_id": "retry-topology-client",
                "traces": {
                    "topics": [TOPIC],
                    "encoding": "otlp_proto"
                },
                "commit": { "mode": "manual" },
                "transient_nack": {
                    "mode": "replay",
                    "initial_backoff_ms": 20,
                    "max_backoff_ms": 20
                },
                "auto_offset_reset": "earliest",
                "isolation_level": "read_uncommitted",
                "enable_idempotency": true
            })),
        )
        .add_processor(
            "retry",
            RETRY_PROCESSOR_URN,
            Some(json!({
                "initial_interval": "500ms",
                "max_interval": "2s",
                "max_elapsed_time": "1500ms",
                "multiplier": 2.0,
                "exhaustion_action": "propagate_transient"
            })),
        )
        .add_exporter("topology_exporter", REPLAY_TOPOLOGY_EXPORTER_URN, None)
        .one_of("kafka_receiver", ["retry"])
        .one_of("retry", ["topology_exporter"])
        .build(
            PipelineType::Otap,
            pipeline_group_id.clone(),
            pipeline_id.clone(),
        )
        .expect("build Kafka replay topology");

    let telemetry_system = InternalTelemetrySystem::default();
    let registry = telemetry_system.registry();
    let controller_ctx = ControllerContext::new(registry.clone());
    let pipeline_ctx = controller_ctx.pipeline_context_with(
        pipeline_group_id.clone(),
        pipeline_id.clone(),
        0,
        1,
        0,
    );
    let pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
    let channel_capacity_policy = ChannelCapacityPolicy::default();
    let runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(
            pipeline_ctx.clone(),
            config,
            channel_capacity_policy.clone(),
            TelemetryPolicy::default(),
            None,
            std::collections::BTreeMap::new(),
            None,
            None,
        )
        .expect("build runtime Kafka replay topology");

    let (runtime_ctrl_tx, runtime_ctrl_rx) =
        runtime_ctrl_msg_channel(channel_capacity_policy.control.pipeline);
    let (pipeline_completion_tx, pipeline_completion_rx) =
        pipeline_completion_msg_channel(channel_capacity_policy.control.completion);
    let runtime_ctrl_tx_for_shutdown = runtime_ctrl_tx.clone();
    let shutdown_handle = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !REPLAY_TOPOLOGY_ACKED.load(AtomicOrdering::Acquire) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
        }
        let acked = REPLAY_TOPOLOGY_ACKED.load(AtomicOrdering::Acquire);
        if acked {
            std::thread::sleep(Duration::from_millis(500));
        }
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build topology shutdown runtime")
            .block_on(
                runtime_ctrl_tx_for_shutdown.send(RuntimeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(2),
                    reason: "Kafka replay topology test complete".to_string(),
                }),
            )
            .expect("send topology shutdown");
        acked
    });

    let observed_state_store =
        ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());
    let pipeline_key = DeployedPipelineKey {
        pipeline_group_id,
        pipeline_id,
        core_id: 0,
        deployment_generation: 0,
    };
    let metrics_reporter = telemetry_system.reporter();
    let event_reporter = observed_state_store.reporter(SendPolicy::default());
    let run_result = {
        let _pipeline_entity_guard =
            set_pipeline_entity_key(pipeline_ctx.metrics_registry(), pipeline_entity_key);
        let (_memory_pressure_tx, memory_pressure_rx) = tokio::sync::watch::channel(
            otel_arrow_dfe_engine::memory_limiter::MemoryPressureChanged::initial(),
        );
        runtime_pipeline.run_forever(
            pipeline_key,
            pipeline_ctx,
            event_reporter,
            metrics_reporter,
            Duration::from_millis(100),
            memory_pressure_rx,
            runtime_ctrl_tx,
            runtime_ctrl_rx,
            pipeline_completion_tx,
            pipeline_completion_rx,
        )
    };
    let acked = shutdown_handle.join().expect("join topology shutdown");

    assert!(
        run_result.is_ok(),
        "Kafka replay topology must shut down cleanly: {run_result:?}"
    );
    assert!(acked, "exporter must ACK the Kafka replay before shutdown");
}

/// Scenario: an exporter transiently NACKs through a real retry processor until its local
/// retry budget expires, then ACKs the record only after the Kafka receiver replays it.
/// Guarantees: exporter failures are retried payload-locally first, the exhausted transient
/// NACK reaches the receiver, and Kafka replay starts a fresh retry-processor attempt that commits.
#[tokio::test]
async fn retry_processor_exhaustion_falls_back_to_kafka_replay() {
    const TOPIC: &str = "retry-topology-fallback-traces";
    const GROUP: &str = "retry-topology-fallback-group";
    REPLAY_TOPOLOGY_ACKED.store(false, AtomicOrdering::Release);
    REPLAY_TOPOLOGY_ATTEMPTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();

    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = Vec::new();
            req.encode(&mut bytes).expect("encode trace request");
            producer
                .send_full(SendRecord::new(TOPIC, &bytes))
                .await
                .expect("produce topology record");
            drop(producer);

            let bootstrap_servers = cluster.bootstrap_servers().to_string();
            let brokers = bootstrap_servers.clone();
            std::thread::spawn(move || run_retry_topology_pipeline(bootstrap_servers))
                .join()
                .expect("run Kafka replay topology");

            let committed = poll_until(Duration::from_secs(5), Duration::from_millis(25), || {
                committed_offset(&brokers, GROUP, TOPIC, 0)
                    .expect("probe topology committed offset")
                    .is_some_and(|offset| offset >= 1)
            })
            .await;
            assert!(committed, "ACK after Kafka replay must commit offset 1");
        },
    )
    .await;

    assert_eq!(
        *REPLAY_TOPOLOGY_ATTEMPTS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec![0, 1, 0],
        "the retry processor must retry locally before Kafka starts a fresh delivery"
    );
}

/// Scenario: Explicit commit-and-skip handles transient and permanent NACKs as terminal outcomes.
/// Guarantees: Both records advance the committed offset despite replay being the manual default.
#[tokio::test]
async fn explicit_commit_and_skip_advances_transient_and_permanent_nacks() {
    const TOPIC: &str = "offset-nack-parity-traces";
    const RECORDS: usize = 2;
    let group = "offset-nack-parity-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            for i in 0..RECORDS {
                let key = format!("rec-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send record");
            }

            let cfg = manual_traces_config_with_commit_and_skip(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            let mut by_offset: HashMap<i64, OtapPdata> = HashMap::new();
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                let route = pdata
                    .source_route()
                    .expect("delivered pdata carries source calldata");
                let (_, _, offset, _) = decode_calldata(&route.calldata);
                let _ = by_offset.insert(offset, pdata);
            }

            receiver.nack_transient(
                "transient failure",
                by_offset.remove(&0).expect("offset 0 delivered"),
            );
            receiver.nack_permanent(
                "permanent failure",
                by_offset.remove(&1).expect("offset 1 delivered"),
            );

            let brokers = cluster.bootstrap_servers().to_string();
            let advanced = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|offset| offset >= RECORDS as i64)
            })
            .await;
            assert!(advanced, "both terminal outcomes must advance the offset");

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario: Offset 0 is transiently NACKed while obsolete feedback ACKs offset 1.
/// Guarantees: Both offsets replay under a fresh generation and stale feedback cannot commit them.
#[tokio::test]
async fn transient_nack_replays_without_committing_past_failure() {
    const TOPIC: &str = "offset-transient-replay-traces";
    const RECORDS: usize = 2;
    let group = "offset-transient-replay-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            for i in 0..RECORDS {
                let key = format!("rec-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send record");
            }

            let cfg = manual_traces_config_with_replay(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            let mut original = HashMap::new();
            let mut original_delivery_generation = None;
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                let route = pdata.source_route().expect("source route");
                let (_, partition, offset, delivery_generation) = decode_calldata(&route.calldata);
                assert_eq!(partition, 0);
                if let Some(expected) = original_delivery_generation {
                    assert_eq!(delivery_generation, expected);
                } else {
                    original_delivery_generation = Some(delivery_generation);
                }
                let _ = original.insert(offset, pdata);
            }

            receiver.nack_transient(
                "retry from Kafka",
                original.remove(&0).expect("offset 0 delivered"),
            );
            receiver.ack(original.remove(&1).expect("offset 1 delivered"));

            let mut replayed = HashMap::new();
            let mut replay_delivery_generation = None;
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                let route = pdata.source_route().expect("replay source route");
                let (_, partition, offset, delivery_generation) = decode_calldata(&route.calldata);
                assert_eq!(partition, 0);
                assert_ne!(Some(delivery_generation), original_delivery_generation);
                if let Some(expected) = replay_delivery_generation {
                    assert_eq!(delivery_generation, expected);
                } else {
                    replay_delivery_generation = Some(delivery_generation);
                }
                let _ = replayed.insert(offset, pdata);
            }

            let brokers = cluster.bootstrap_servers().to_string();
            let committed_before_replay_ack =
                committed_offset(&brokers, group, TOPIC, 0).expect("committed-offset probe");
            assert!(committed_before_replay_ack.is_none_or(|offset| offset <= 0));

            receiver.ack(replayed.remove(&1).expect("replayed offset 1"));
            receiver.wait_for_control_barrier().await;
            let committed_out_of_order =
                committed_offset(&brokers, group, TOPIC, 0).expect("committed-offset probe");
            assert!(committed_out_of_order.is_none_or(|offset| offset <= 0));

            receiver.ack(replayed.remove(&0).expect("replayed offset 0"));
            let advanced = poll_until(Duration::from_secs(5), Duration::from_millis(25), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("committed-offset probe")
                    .is_some_and(|offset| offset >= RECORDS as i64)
            })
            .await;
            assert!(advanced, "ACKing the rewind record must release progress");

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario: One partition backs off while a sibling continues and the receiver then shuts down.
/// Guarantees: Replay blocks only the failed partition and shutdown does not commit its offset.
#[tokio::test]
async fn transient_nack_pause_is_partition_local_and_shutdown_safe() {
    const TOPIC: &str = "offset-transient-partition-local-traces";
    let group = "offset-transient-partition-local-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer.produce_per_partition(TOPIC, 2, 1, &bytes).await;

            let cfg = manual_traces_config_with_replay_backoff(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                5_000,
                5_000,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            let mut first_by_partition = HashMap::new();
            for _ in 0..2 {
                let pdata = receiver.recv_pdata().await;
                let (_, partition, offset, _) =
                    decode_calldata(&pdata.source_route().expect("source route").calldata);
                assert_eq!(offset, 0);
                let _ = first_by_partition.insert(partition, pdata);
            }

            receiver.nack_transient(
                "pause partition zero",
                first_by_partition
                    .remove(&0)
                    .expect("partition 0 delivered"),
            );
            receiver.ack(
                first_by_partition
                    .remove(&1)
                    .expect("partition 1 delivered"),
            );
            producer
                .send_to_partition(TOPIC, 1, &bytes)
                .await
                .expect("send sibling record");

            let sibling = receiver
                .try_recv_pdata(Duration::from_secs(2))
                .await
                .expect("healthy partition continues during replay backoff");
            let (_, partition, offset, _) =
                decode_calldata(&sibling.source_route().expect("source route").calldata);
            assert_eq!((partition, offset), (1, 1));
            receiver.ack(sibling);

            let brokers = cluster.bootstrap_servers().to_string();
            let sibling_advanced =
                poll_until(Duration::from_secs(5), Duration::from_millis(25), || {
                    committed_offset(&brokers, group, TOPIC, 1)
                        .expect("committed-offset probe")
                        .is_some_and(|committed| committed >= 2)
                })
                .await;
            assert!(
                sibling_advanced,
                "healthy partition must commit independently"
            );
            assert!(
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("committed-offset probe")
                    .is_none_or(|committed| committed <= 0),
                "failed partition must remain at offset 0",
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
            assert!(
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("committed-offset probe")
                    .is_none_or(|committed| committed <= 0),
                "shutdown during retry must not commit past the failed offset",
            );
        },
    )
    .await;
}

/// Scenario: A second group member acquires a partition held in transient-NACK backoff.
/// Guarantees: The new owner receives the uncommitted failed offset from Kafka.
#[tokio::test]
async fn transient_nack_rebalance_replays_failed_offset_on_new_owner() {
    const TOPIC: &str = "offset-transient-rebalance-traces";
    let group = "offset-transient-rebalance-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer.produce_per_partition(TOPIC, 2, 1, &bytes).await;

            let cfg = manual_traces_config_with_replay_backoff(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                30_000,
                30_000,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            for _ in 0..2 {
                let pdata = receiver.recv_pdata().await;
                receiver.nack_transient("hold for rebalance", pdata);
            }
            receiver.wait_for_control_barrier().await;

            let new_owner = cluster
                .consumer()
                .group_id(group)
                .enable_auto_commit(false)
                .auto_offset_reset("earliest")
                .subscribe(&[TOPIC]);
            assert!(
                new_owner
                    .wait_for_assignment(1, Duration::from_secs(30))
                    .await,
                "second consumer never acquired a partition",
            );
            let assigned_partition = new_owner
                .assignment()
                .into_iter()
                .find_map(|(topic, partition)| (topic == TOPIC).then_some(partition))
                .expect("second consumer owns a test partition");
            receiver
                .wait_for_partition_revocation(TOPIC, assigned_partition, Duration::from_secs(10))
                .await;

            let replayed = new_owner
                .try_recv(Duration::from_secs(10))
                .await
                .expect("new owner should receive the failed offset");
            assert_eq!(replayed.topic, TOPIC);
            assert_eq!(replayed.partition, assigned_partition);
            assert_eq!(replayed.offset, 0);
            assert!(
                new_owner
                    .committed_offset(TOPIC, assigned_partition)
                    .expect("committed-offset probe")
                    .is_none_or(|offset| offset <= 0),
                "rebalance must not commit past the failed offset",
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
            drop(new_owner);
        },
    )
    .await;
}

/// Scenario: A paused partition is revoked and then reassigned to the original consumer.
/// Guarantees: Rebalance clears persistent pause state and redelivers the failed offset.
#[tokio::test]
async fn transient_nack_reassignment_to_same_consumer_redelivers_failed_offset() {
    const TOPIC: &str = "offset-transient-reassign-same-consumer-traces";
    let group = "offset-transient-reassign-same-consumer-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer.produce_per_partition(TOPIC, 2, 1, &bytes).await;

            let cfg = manual_traces_config_with_replay_backoff(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                30_000,
                30_000,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            for _ in 0..2 {
                let pdata = receiver.recv_pdata().await;
                receiver.nack_transient("hold through reassignment", pdata);
            }
            receiver.wait_for_control_barrier().await;

            let trigger =
                RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30)).await;
            let revoked_partition = trigger
                .assignment()
                .into_iter()
                .find_map(|(topic, partition)| (topic == TOPIC).then_some(partition))
                .expect("temporary member should own a test partition");
            receiver
                .wait_for_partition_revocation(TOPIC, revoked_partition, Duration::from_secs(10))
                .await;

            drop(trigger);
            receiver
                .wait_for_partition_assignment(TOPIC, revoked_partition, Duration::from_secs(30))
                .await;

            // Kafka does not define ordering across partitions. A delivery from
            // the retained partition can race with the reacquired partition's
            // redelivery, so select the partition this scenario is exercising.
            let replayed =
                recv_partition_delivery(&mut receiver, revoked_partition, Duration::from_secs(10))
                    .await;
            let route = replayed
                .source_route()
                .expect("redelivered pdata carries source calldata");
            let (_, partition, offset, _) = decode_calldata(&route.calldata);
            assert_eq!(partition, revoked_partition);
            assert_eq!(offset, 0);

            receiver.ack(replayed);
            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}
