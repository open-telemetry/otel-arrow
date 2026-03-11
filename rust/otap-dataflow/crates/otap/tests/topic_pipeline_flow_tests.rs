// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end topic flow tests for OTAP topic nodes.

use bytes::Bytes;
use otap_df_config::TopicName;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::Interests;
use otap_df_engine::config::{ExporterConfig, ReceiverConfig};
use otap_df_engine::control::{Controllable, NodeControlMsg, pipeline_ctrl_msg_channel};
use otap_df_engine::effect_handler::SourceTagging;
use otap_df_engine::local::message::{LocalReceiver, LocalSender};
use otap_df_engine::message::{Receiver as PDataReceiver, Sender as PDataSender};
use otap_df_engine::node::{NodeWithPDataReceiver, NodeWithPDataSender};
use otap_df_engine::testing::exporter::create_test_pipeline_context;
use otap_df_engine::testing::{create_not_send_channel, setup_test_runtime, test_node};
use otap_df_engine::topic::{TopicBroadcastOnLagPolicy, TopicBroker, TopicOptions, TopicSet};
use otap_df_otap::pdata::OtapPdata;
use otap_df_otap::topic_exporter::{TOPIC_EXPORTER, TOPIC_EXPORTER_URN};
use otap_df_otap::topic_receiver::{TOPIC_RECEIVER, TOPIC_RECEIVER_URN};
use otap_df_pdata::OtlpProtoBytes;
use otap_df_telemetry::reporter::MetricsReporter;
use prost::Message as _;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn make_test_pdata() -> OtapPdata {
    let logs = otap_df_pdata::testing::fixtures::log_with_no_scope();
    let mut bytes = Vec::new();
    logs.encode(&mut bytes).expect("log payload should encode");
    OtapPdata::new_todo_context(OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)).into())
}

#[test]
fn topic_exporter_to_topic_receiver_transfers_pdata() {
    let (rt, local_tasks) = setup_test_runtime();
    rt.block_on(local_tasks.run_until(async move {
        let broker = TopicBroker::<OtapPdata>::new();
        let topic_name = TopicName::parse("ingress").expect("topic name should parse");
        let handle = broker
            .create_in_memory_topic(
                topic_name.clone(),
                TopicOptions::Mixed {
                    balanced_capacity: 16,
                    broadcast_capacity: 16,
                    on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                },
            )
            .expect("topic should be created");

        let exporter_set = TopicSet::new("exporter-set");
        _ = exporter_set.insert(topic_name.clone(), handle.clone());
        let receiver_set = TopicSet::new("receiver-set");
        _ = receiver_set.insert(topic_name.clone(), handle);

        let mut exporter_ctx = create_test_pipeline_context();
        exporter_ctx.set_topic_set(exporter_set);
        let mut receiver_ctx = create_test_pipeline_context();
        receiver_ctx.set_topic_set(receiver_set);

        let exporter_node = test_node("topic_exporter");
        let receiver_node = test_node("topic_receiver");

        let mut exporter_user_cfg = NodeUserConfig::new_exporter_config(TOPIC_EXPORTER_URN);
        exporter_user_cfg.config = json!({
            "topic": "ingress",
            "queue_on_full": "block"
        });
        let mut receiver_user_cfg = NodeUserConfig::new_receiver_config(TOPIC_RECEIVER_URN);
        receiver_user_cfg.config = json!({
            "topic": "ingress",
            "subscription": {
                "mode": "balanced",
                "group": "sut-workers"
            }
        });

        let mut exporter = (TOPIC_EXPORTER.create)(
            exporter_ctx,
            exporter_node.clone(),
            Arc::new(exporter_user_cfg),
            &ExporterConfig::new("topic_exporter"),
        )
        .expect("topic exporter should be created");

        let mut receiver = (TOPIC_RECEIVER.create)(
            receiver_ctx,
            receiver_node.clone(),
            Arc::new(receiver_user_cfg),
            &ReceiverConfig::new("topic_receiver"),
        )
        .expect("topic receiver should be created");

        let (exporter_input_tx, exporter_input_rx) = create_not_send_channel::<OtapPdata>(8);
        exporter
            .set_pdata_receiver(
                exporter_node.clone(),
                PDataReceiver::Local(LocalReceiver::mpsc(exporter_input_rx)),
            )
            .expect("exporter input channel should be wired");

        let (receiver_output_tx, receiver_output_rx) = create_not_send_channel::<OtapPdata>(8);
        receiver
            .set_pdata_sender(
                receiver_node.clone(),
                "".into(),
                PDataSender::Local(LocalSender::mpsc(receiver_output_tx)),
            )
            .expect("receiver output channel should be wired");

        let exporter_ctrl = exporter.control_sender();
        let receiver_ctrl = receiver.control_sender();
        let (pipeline_ctrl_tx, _pipeline_ctrl_rx) = pipeline_ctrl_msg_channel::<OtapPdata>(32);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
        let exporter_metrics = metrics_reporter.clone();
        let receiver_metrics = metrics_reporter.clone();
        let exporter_ctrl_tx = pipeline_ctrl_tx.clone();
        let receiver_ctrl_tx = pipeline_ctrl_tx.clone();

        let exporter_task = tokio::task::spawn_local(async move {
            exporter
                .start(exporter_ctrl_tx, exporter_metrics, Interests::empty())
                .await
        });
        let receiver_task = tokio::task::spawn_local(async move {
            receiver
                .start(receiver_ctrl_tx, receiver_metrics, Interests::empty())
                .await
        });

        let expected = make_test_pdata();
        exporter_input_tx
            .send(expected.clone())
            .expect("message should be sent to topic exporter");

        let delivered = tokio::time::timeout(Duration::from_secs(2), receiver_output_rx.recv())
            .await
            .expect("timed out waiting for topic receiver output")
            .expect("receiver output channel should stay open");

        assert_eq!(
            delivered.num_items(),
            expected.num_items(),
            "topic receiver should forward one pdata message"
        );

        let deadline = Instant::now() + Duration::from_secs(1);
        exporter_ctrl
            .send(NodeControlMsg::Shutdown {
                deadline,
                reason: "test shutdown".to_owned(),
            })
            .await
            .expect("exporter shutdown should be sent");
        receiver_ctrl
            .send(NodeControlMsg::Shutdown {
                deadline,
                reason: "test shutdown".to_owned(),
            })
            .await
            .expect("receiver shutdown should be sent");

        let exporter_result = exporter_task.await.expect("exporter task should join");
        let receiver_result = receiver_task.await.expect("receiver task should join");
        assert!(exporter_result.is_ok(), "exporter should stop cleanly");
        assert!(receiver_result.is_ok(), "receiver should stop cleanly");
    }));
}

#[test]
fn topic_receiver_applies_source_tag_when_enabled() {
    let (rt, local_tasks) = setup_test_runtime();
    rt.block_on(local_tasks.run_until(async move {
        let broker = TopicBroker::<OtapPdata>::new();
        let topic_name = TopicName::parse("ingress").expect("topic name should parse");
        let handle = broker
            .create_in_memory_topic(
                topic_name.clone(),
                TopicOptions::Mixed {
                    balanced_capacity: 16,
                    broadcast_capacity: 16,
                    on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                },
            )
            .expect("topic should be created");

        let exporter_set = TopicSet::new("exporter-set");
        _ = exporter_set.insert(topic_name.clone(), handle.clone());
        let receiver_set = TopicSet::new("receiver-set");
        _ = receiver_set.insert(topic_name.clone(), handle);

        let mut exporter_ctx = create_test_pipeline_context();
        exporter_ctx.set_topic_set(exporter_set);
        let mut receiver_ctx = create_test_pipeline_context();
        receiver_ctx.set_topic_set(receiver_set);

        let exporter_node = test_node("topic_exporter");
        let receiver_node = test_node("topic_receiver");

        let mut exporter_user_cfg = NodeUserConfig::new_exporter_config(TOPIC_EXPORTER_URN);
        exporter_user_cfg.config = json!({"topic": "ingress"});
        let mut receiver_user_cfg = NodeUserConfig::new_receiver_config(TOPIC_RECEIVER_URN);
        receiver_user_cfg.config = json!({
            "topic": "ingress",
            "subscription": {
                "mode": "balanced",
                "group": "sut-workers"
            }
        });

        let mut exporter = (TOPIC_EXPORTER.create)(
            exporter_ctx,
            exporter_node.clone(),
            Arc::new(exporter_user_cfg),
            &ExporterConfig::new("topic_exporter"),
        )
        .expect("topic exporter should be created");

        let mut receiver = (TOPIC_RECEIVER.create)(
            receiver_ctx,
            receiver_node.clone(),
            Arc::new(receiver_user_cfg),
            &ReceiverConfig::new("topic_receiver"),
        )
        .expect("topic receiver should be created");

        // Match runtime fan-in behavior where source tagging is enabled.
        receiver.set_source_tagging(SourceTagging::Enabled);

        let (exporter_input_tx, exporter_input_rx) = create_not_send_channel::<OtapPdata>(8);
        exporter
            .set_pdata_receiver(
                exporter_node.clone(),
                PDataReceiver::Local(LocalReceiver::mpsc(exporter_input_rx)),
            )
            .expect("exporter input channel should be wired");

        let (receiver_output_tx, receiver_output_rx) = create_not_send_channel::<OtapPdata>(8);
        receiver
            .set_pdata_sender(
                receiver_node.clone(),
                "".into(),
                PDataSender::Local(LocalSender::mpsc(receiver_output_tx)),
            )
            .expect("receiver output channel should be wired");

        let exporter_ctrl = exporter.control_sender();
        let receiver_ctrl = receiver.control_sender();
        let (pipeline_ctrl_tx, _pipeline_ctrl_rx) = pipeline_ctrl_msg_channel::<OtapPdata>(32);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
        let exporter_metrics = metrics_reporter.clone();
        let receiver_metrics = metrics_reporter.clone();
        let exporter_ctrl_tx = pipeline_ctrl_tx.clone();
        let receiver_ctrl_tx = pipeline_ctrl_tx.clone();

        let exporter_task = tokio::task::spawn_local(async move {
            exporter
                .start(exporter_ctrl_tx, exporter_metrics, Interests::empty())
                .await
        });
        let receiver_task = tokio::task::spawn_local(async move {
            receiver
                .start(receiver_ctrl_tx, receiver_metrics, Interests::empty())
                .await
        });

        exporter_input_tx
            .send(make_test_pdata())
            .expect("message should be sent to topic exporter");

        let delivered = tokio::time::timeout(Duration::from_secs(2), receiver_output_rx.recv())
            .await
            .expect("timed out waiting for topic receiver output")
            .expect("receiver output channel should stay open");

        assert_eq!(
            delivered.get_source_node(),
            Some(receiver_node.index),
            "receiver should tag outgoing pdata with source node when enabled"
        );

        let deadline = Instant::now() + Duration::from_secs(1);
        exporter_ctrl
            .send(NodeControlMsg::Shutdown {
                deadline,
                reason: "test shutdown".to_owned(),
            })
            .await
            .expect("exporter shutdown should be sent");
        receiver_ctrl
            .send(NodeControlMsg::Shutdown {
                deadline,
                reason: "test shutdown".to_owned(),
            })
            .await
            .expect("receiver shutdown should be sent");

        let exporter_result = exporter_task.await.expect("exporter task should join");
        let receiver_result = receiver_task.await.expect("receiver task should join");
        assert!(exporter_result.is_ok(), "exporter should stop cleanly");
        assert!(receiver_result.is_ok(), "receiver should stop cleanly");
    }));
}
