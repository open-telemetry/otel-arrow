// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{DstRng, SimClock, dst_seeds};
use crate::Interests;
use crate::control::{
    AckMsg, ControlSenders, NackMsg, NodeControlMsg, PipelineResultMsg, RuntimeControlMsg,
    pipeline_result_msg_channel,
};
use crate::message::Receiver;
use crate::node::NodeType;
use crate::pipeline_ctrl::PipelineResultMsgDispatcher;
use crate::testing::dst::common::{
    DstPData, build_manager, create_mock_control_sender, empty_node_metric_handles, frame,
    recv_controls, recv_until, setup_dst_runtime, yield_cycles,
};
use crate::testing::test_nodes;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

async fn run_control_plane_seed(seed: u64) {
    let clock = SimClock::new();
    let _clock_guard = clock.install();
    let (rt, local_tasks) = setup_dst_runtime();

    rt.block_on(local_tasks.run_until(async move {
        let mut rng = DstRng::new(seed);
        let nodes = test_nodes(vec!["receiver", "processor", "exporter"]);
        let receiver_id = nodes[0].clone();
        let processor_id = nodes[1].clone();
        let exporter_id = nodes[2].clone();

        let mut control_senders = ControlSenders::new();
        let mut control_receivers: HashMap<usize, Receiver<NodeControlMsg<DstPData>>> =
            HashMap::new();
        for (node, node_type) in [
            (receiver_id.clone(), NodeType::Receiver),
            (processor_id.clone(), NodeType::Processor),
            (exporter_id.clone(), NodeType::Exporter),
        ] {
            let (sender, receiver) = create_mock_control_sender::<DstPData>(128);
            control_senders.register(node.clone(), node_type, sender);
            let _ = control_receivers.insert(node.index, receiver);
        }

        let (manager, runtime_tx, _scope, _pipeline_context) =
            build_manager::<DstPData>(256, control_senders.clone());
        let (result_tx, result_rx) = pipeline_result_msg_channel(256);
        let dispatcher = PipelineResultMsgDispatcher::new(
            result_rx,
            control_senders.clone(),
            empty_node_metric_handles(),
        );

        let manager_handle = tokio::task::spawn_local(async move { manager.run().await });
        let dispatcher_handle = tokio::task::spawn_local(async move { dispatcher.run().await });

        runtime_tx
            .send(RuntimeControlMsg::StartTimer {
                node_id: processor_id.index,
                duration: Duration::from_millis(10),
            })
            .await
            .unwrap();
        runtime_tx
            .send(RuntimeControlMsg::DelayData {
                node_id: processor_id.index,
                when: clock.now() + Duration::from_millis(12),
                data: Box::new(DstPData::new(9000)),
            })
            .await
            .unwrap();
        yield_cycles(2).await;
        for _ in 0..(70 + rng.gen_range(6)) {
            runtime_tx
                .send(RuntimeControlMsg::CancelTelemetryTimer {
                    node_id: exporter_id.index,
                    _temp: Default::default(),
                })
                .await
                .unwrap();
        }

        let return_data = rng.next_bool();
        let ack = AckMsg::new(DstPData::with_frames(
            100,
            vec![
                frame(receiver_id.index, Interests::PRODUCER_METRICS, 1),
                frame(
                    processor_id.index,
                    Interests::ACKS
                        | Interests::NACKS
                        | if return_data {
                            Interests::RETURN_DATA
                        } else {
                            Interests::empty()
                        },
                    2,
                ),
                frame(exporter_id.index, Interests::CONSUMER_METRICS, 3),
            ],
        ));
        result_tx
            .send(PipelineResultMsg::DeliverAck { ack })
            .await
            .unwrap();

        let permanent_nack = rng.next_bool();
        let nack = if permanent_nack {
            NackMsg::new_permanent(
                "permanent",
                DstPData::with_frames(
                    101,
                    vec![
                        frame(receiver_id.index, Interests::PRODUCER_METRICS, 4),
                        frame(processor_id.index, Interests::NACKS, 5),
                        frame(exporter_id.index, Interests::CONSUMER_METRICS, 6),
                    ],
                ),
            )
        } else {
            NackMsg::new(
                "temporary",
                DstPData::with_frames(
                    101,
                    vec![
                        frame(receiver_id.index, Interests::PRODUCER_METRICS, 4),
                        frame(processor_id.index, Interests::NACKS, 5),
                        frame(exporter_id.index, Interests::CONSUMER_METRICS, 6),
                    ],
                ),
            )
        };
        result_tx
            .send(PipelineResultMsg::DeliverNack { nack })
            .await
            .unwrap();
        result_tx
            .send(PipelineResultMsg::DeliverAck {
                ack: AckMsg::new(DstPData::new(102)),
            })
            .await
            .unwrap();

        clock.advance(Duration::from_millis(20));
        let processor_msgs = recv_until(
            control_receivers
                .get_mut(&processor_id.index)
                .expect("processor control receiver"),
            Duration::from_secs(1),
            |msgs| {
                msgs.iter()
                    .any(|msg| matches!(msg, NodeControlMsg::TimerTick {}))
                    && msgs.iter().any(|msg| {
                        matches!(
                            msg,
                            NodeControlMsg::DelayedData { data, .. } if data.id == 9000
                        )
                    })
                    && msgs.iter().any(|msg| matches!(msg, NodeControlMsg::Ack(_)))
                    && msgs
                        .iter()
                        .any(|msg| matches!(msg, NodeControlMsg::Nack(_)))
            },
            &format!(
                "seed={seed}: processor control receiver did not observe timer/delay/ack/nack"
            ),
        )
        .await;
        assert!(
            processor_msgs
                .iter()
                .any(|msg| matches!(msg, NodeControlMsg::TimerTick {})),
            "seed={seed}: due timer tick was starved under runtime-control burst"
        );
        assert!(
            processor_msgs.iter().any(
                |msg| matches!(msg, NodeControlMsg::DelayedData { data, .. } if data.id == 9000)
            ),
            "seed={seed}: delayed data did not resume under control pressure"
        );

        let ack_msg = processor_msgs.iter().find_map(|msg| match msg {
            NodeControlMsg::Ack(ack) => Some(ack),
            _ => None,
        });
        assert!(
            ack_msg.is_some(),
            "seed={seed}: subscribed ack was not delivered"
        );
        assert_eq!(
            ack_msg
                .expect("ack present")
                .accepted
                .payload
                .as_ref()
                .is_some(),
            return_data,
            "seed={seed}: RETURN_DATA retention mismatch"
        );

        let nack_msg = processor_msgs.iter().find_map(|msg| match msg {
            NodeControlMsg::Nack(nack) => Some(nack),
            _ => None,
        });
        assert!(
            nack_msg.is_some(),
            "seed={seed}: subscribed nack was not delivered"
        );
        assert_eq!(
            nack_msg.expect("nack present").permanent,
            permanent_nack,
            "seed={seed}: nack permanence mismatch"
        );

        runtime_tx
            .send(RuntimeControlMsg::Shutdown {
                deadline: clock.now() + Duration::from_millis(40),
                reason: format!("dst-seed-{seed}"),
            })
            .await
            .unwrap();
        let receiver_msgs = recv_until(
            control_receivers
                .get_mut(&receiver_id.index)
                .expect("receiver control receiver"),
            Duration::from_secs(1),
            |msgs| {
                msgs.iter()
                    .any(|msg| matches!(msg, NodeControlMsg::DrainIngress { .. }))
            },
            &format!("seed={seed}: receiver did not observe DrainIngress"),
        )
        .await;
        assert!(
            receiver_msgs
                .iter()
                .any(|msg| matches!(msg, NodeControlMsg::DrainIngress { .. })),
            "seed={seed}: receiver did not receive DrainIngress"
        );

        let exporter_msgs = recv_controls(
            control_receivers
                .get_mut(&exporter_id.index)
                .expect("exporter control receiver"),
        )
        .await;
        assert!(
            exporter_msgs
                .iter()
                .all(|msg| !matches!(msg, NodeControlMsg::Shutdown { .. })),
            "seed={seed}: exporter shutdown arrived before ReceiverDrained"
        );

        runtime_tx
            .send(RuntimeControlMsg::ReceiverDrained {
                node_id: receiver_id.index,
            })
            .await
            .unwrap();
        let exporter_msgs = recv_until(
            control_receivers
                .get_mut(&exporter_id.index)
                .expect("exporter control receiver"),
            Duration::from_secs(1),
            |msgs| {
                msgs.iter()
                    .any(|msg| matches!(msg, NodeControlMsg::Shutdown { .. }))
            },
            &format!("seed={seed}: exporter did not observe Shutdown after ReceiverDrained"),
        )
        .await;
        assert!(
            exporter_msgs
                .iter()
                .any(|msg| matches!(msg, NodeControlMsg::Shutdown { .. })),
            "seed={seed}: downstream shutdown did not wait for ReceiverDrained"
        );

        drop(result_tx);
        drop(runtime_tx);

        timeout(Duration::from_secs(1), manager_handle)
            .await
            .expect("manager should exit")
            .unwrap()
            .expect("manager should succeed");
        timeout(Duration::from_secs(1), dispatcher_handle)
            .await
            .expect("dispatcher should exit")
            .unwrap()
            .expect("dispatcher should succeed");
    }));
}

#[test]
fn dst_runtime_control_plane_seeded() {
    for seed in dst_seeds(&[5, 17, 29], 8) {
        futures::executor::block_on(run_control_plane_seed(seed));
    }
}
