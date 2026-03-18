// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{DstRng, SimClock, dst_seeds};
use crate::Interests;
use crate::clock;
use crate::control::{
    AckMsg, ControlSenders, NackMsg, NodeControlMsg, PipelineResultMsg, RuntimeControlMsg,
    pipeline_result_msg_channel,
};
use crate::message::{Message, MessageChannel, Receiver};
use crate::node::NodeType;
use crate::pipeline_ctrl::PipelineResultMsgDispatcher;
use crate::testing::dst::common::{
    DstPData, build_manager, create_mock_control_sender, empty_node_metric_handles, frame,
    setup_dst_runtime, yield_cycles,
};
use crate::testing::test_nodes;
use otap_df_channel::mpsc;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Default)]
struct FlowState {
    admitted: HashSet<u64>,
    forwarded: HashSet<u64>,
    completed: HashSet<u64>,
    ack_count: usize,
    nack_count: usize,
    receiver_blocked_count: usize,
    processor_timer_ticks: usize,
    processor_paused_seen: bool,
    processor_resumed_seen: bool,
    receiver_drain_seen: bool,
    receiver_drained_sent: bool,
    processor_shutdown_seen: bool,
    exporter_shutdown_seen: bool,
    runtime_noise_sent: usize,
}

async fn run_backpressure_interblock_seed(seed: u64) {
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
        let mut receiver_control_rx = None;
        let mut processor_control_rx = None;
        let mut exporter_control_rx = None;
        for (node, node_type, capacity) in [
            (receiver_id.clone(), NodeType::Receiver, 16usize),
            (processor_id.clone(), NodeType::Processor, 8usize),
            (exporter_id.clone(), NodeType::Exporter, 8usize),
        ] {
            let (sender, receiver) = create_mock_control_sender::<DstPData>(capacity);
            control_senders.register(node.clone(), node_type, sender);
            match node.index {
                idx if idx == receiver_id.index => receiver_control_rx = Some(receiver),
                idx if idx == processor_id.index => processor_control_rx = Some(receiver),
                idx if idx == exporter_id.index => exporter_control_rx = Some(receiver),
                _ => unreachable!("unexpected node id"),
            }
        }

        let (manager, runtime_tx, _scope, _pipeline_context) =
            build_manager::<DstPData>(32, control_senders.clone());
        let (result_tx, result_rx) = pipeline_result_msg_channel(16);
        let dispatcher = PipelineResultMsgDispatcher::new(
            result_rx,
            control_senders.clone(),
            empty_node_metric_handles(),
        );

        let manager_handle = tokio::task::spawn_local(async move { manager.run().await });
        let dispatcher_handle = tokio::task::spawn_local(async move { dispatcher.run().await });

        let (recv_to_proc_tx, recv_to_proc_rx) = mpsc::Channel::<DstPData>::new(4);
        let (proc_to_export_tx, proc_to_export_rx) = mpsc::Channel::<DstPData>::new(4);

        let state = Rc::new(RefCell::new(FlowState::default()));
        let total_messages = 24 + rng.gen_range(8) as u64;
        let max_inflight = 4usize + rng.gen_range(2);

        let receiver_state = state.clone();
        let receiver_runtime_tx = runtime_tx.clone();
        let receiver_handle = tokio::task::spawn_local(async move {
            let mut control_rx = receiver_control_rx.expect("receiver control receiver");
            let mut pending = None;
            let mut next_id = 0u64;
            let mut draining = false;
            let drained_reported = false;

            loop {
                if draining {
                    if !drained_reported {
                        receiver_runtime_tx
                            .send(RuntimeControlMsg::ReceiverDrained {
                                node_id: receiver_id.index,
                            })
                            .await
                            .expect("receiver drained should be sent");
                        receiver_state.borrow_mut().receiver_drained_sent = true;
                    }
                    break;
                }

                if pending.is_none() && next_id < total_messages {
                    let msg_id = next_id;
                    next_id += 1;
                    pending = Some(DstPData::with_frames(
                        msg_id,
                        vec![
                            frame(
                                receiver_id.index,
                                Interests::PRODUCER_METRICS,
                                msg_id * 10 + 1,
                            ),
                            frame(
                                processor_id.index,
                                Interests::ACKS
                                    | Interests::NACKS
                                    | if msg_id.is_multiple_of(3) {
                                        Interests::RETURN_DATA
                                    } else {
                                        Interests::empty()
                                    },
                                msg_id * 10 + 2,
                            ),
                            frame(
                                exporter_id.index,
                                Interests::CONSUMER_METRICS,
                                msg_id * 10 + 3,
                            ),
                        ],
                    ));
                }

                if let Some(pdata) = pending.take() {
                    tokio::select! {
                        biased;

                        ctrl = control_rx.recv() => match ctrl {
                            Ok(NodeControlMsg::DrainIngress { .. }) => {
                                receiver_state.borrow_mut().receiver_drain_seen = true;
                                draining = true;
                                pending = Some(pdata);
                            }
                            Ok(NodeControlMsg::Shutdown { .. }) => {
                                break;
                            }
                            Ok(_) => {
                                pending = Some(pdata);
                            }
                            Err(_) => break,
                        },

                        _ = tokio::task::yield_now() => {
                            let msg_id = pdata.id;
                            match recv_to_proc_tx.send(pdata) {
                                Ok(()) => {
                                    let _ = receiver_state.borrow_mut().admitted.insert(msg_id);
                                }
                                Err(otap_df_channel::error::SendError::Full(pdata)) => {
                                    receiver_state.borrow_mut().receiver_blocked_count += 1;
                                    pending = Some(pdata);
                                }
                                Err(otap_df_channel::error::SendError::Closed(_)) => break,
                            }
                        }
                    }
                } else {
                    match control_rx.recv().await {
                        Ok(NodeControlMsg::DrainIngress { .. }) => {
                            receiver_state.borrow_mut().receiver_drain_seen = true;
                            draining = true;
                        }
                        Ok(NodeControlMsg::Shutdown { .. }) => break,
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            }

            drop(recv_to_proc_tx);
        });

        let processor_state = state.clone();
        let processor_handle = tokio::task::spawn_local(async move {
            let mut inflight = 0usize;
            let mut was_paused = false;
            let mut msg_channel = MessageChannel::new(
                processor_control_rx.expect("processor control receiver"),
                Receiver::Local(crate::local::message::LocalReceiver::mpsc(recv_to_proc_rx)),
                processor_id.index,
                Interests::empty(),
            );

            while let Ok(msg) = msg_channel.recv_when(inflight < max_inflight).await {
                let paused = inflight >= max_inflight;
                if paused {
                    processor_state.borrow_mut().processor_paused_seen = true;
                } else if was_paused {
                    processor_state.borrow_mut().processor_resumed_seen = true;
                }
                was_paused = paused;

                match msg {
                    Message::PData(pdata) => {
                        let msg_id = pdata.id;
                        proc_to_export_tx
                            .send_async(pdata)
                            .await
                            .expect("processor should forward pdata");
                        inflight += 1;
                        let _ = processor_state.borrow_mut().forwarded.insert(msg_id);
                    }
                    Message::Control(NodeControlMsg::Ack(ack)) => {
                        inflight = inflight.saturating_sub(1);
                        let mut state = processor_state.borrow_mut();
                        state.ack_count += 1;
                        let inserted = state.completed.insert(ack.accepted.id);
                        assert!(inserted, "seed={seed}: duplicate ack completion");
                    }
                    Message::Control(NodeControlMsg::Nack(nack)) => {
                        inflight = inflight.saturating_sub(1);
                        let mut state = processor_state.borrow_mut();
                        state.nack_count += 1;
                        let inserted = state.completed.insert(nack.refused.id);
                        assert!(inserted, "seed={seed}: duplicate nack completion");
                    }
                    Message::Control(NodeControlMsg::TimerTick {}) => {
                        processor_state.borrow_mut().processor_timer_ticks += 1;
                    }
                    Message::Control(NodeControlMsg::Shutdown { .. }) => {
                        processor_state.borrow_mut().processor_shutdown_seen = true;
                        break;
                    }
                    Message::Control(_) => {}
                }
            }

            drop(proc_to_export_tx);
        });

        let exporter_state = state.clone();
        let exporter_result_tx = result_tx.clone();
        let exporter_handle = tokio::task::spawn_local(async move {
            struct ScheduledCompletion {
                when: std::time::Instant,
                pdata: DstPData,
                permanent: bool,
                nack: bool,
            }

            let mut msg_channel = MessageChannel::new(
                exporter_control_rx.expect("exporter control receiver"),
                Receiver::Local(crate::local::message::LocalReceiver::mpsc(proc_to_export_rx)),
                exporter_id.index,
                Interests::empty(),
            );
            let mut scheduled: Vec<ScheduledCompletion> = Vec::new();

            loop {
                let next_due = scheduled.iter().map(|item| item.when).min();

                tokio::select! {
                    biased;

                    _ = async {
                        if let Some(when) = next_due {
                            if when > clock::now() {
                                clock::sleep_until(when).await;
                            }
                        }
                    }, if next_due.is_some() => {
                        let now = clock::now();
                        let mut ready = Vec::new();
                        let mut pending = Vec::with_capacity(scheduled.len());
                        for item in scheduled.drain(..) {
                            if item.when <= now {
                                ready.push(item);
                            } else {
                                pending.push(item);
                            }
                        }
                        scheduled = pending;

                        for item in ready {
                            if item.nack {
                                let nack = if item.permanent {
                                    NackMsg::new_permanent("dst-permanent", item.pdata)
                                } else {
                                    NackMsg::new("dst-temporary", item.pdata)
                                };
                                exporter_result_tx
                                    .send(PipelineResultMsg::DeliverNack { nack })
                                    .await
                                    .expect("exporter should deliver nack");
                            } else {
                                exporter_result_tx
                                    .send(PipelineResultMsg::DeliverAck {
                                        ack: AckMsg::new(item.pdata),
                                    })
                                    .await
                                    .expect("exporter should deliver ack");
                            }
                        }
                    }

                    msg = msg_channel.recv() => match msg {
                        Ok(Message::PData(pdata)) => {
                            let msg_id = pdata.id;
                            let delay_ms = 2 + ((msg_id + seed) % 4);
                            scheduled.push(ScheduledCompletion {
                                when: clock::now() + Duration::from_millis(delay_ms),
                                permanent: msg_id.is_multiple_of(11),
                                nack: msg_id.is_multiple_of(7),
                                pdata,
                            });
                        }
                        Ok(Message::Control(NodeControlMsg::Shutdown { .. })) => {
                            exporter_state.borrow_mut().exporter_shutdown_seen = true;
                            break;
                        }
                        Ok(Message::Control(_)) => {}
                        Err(_) => break,
                    }
                }
            }
        });

        runtime_tx
            .send(RuntimeControlMsg::StartTimer {
                node_id: processor_id.index,
                duration: Duration::from_millis(3),
            })
            .await
            .unwrap();

        let completion_target = total_messages as usize;
        for step in 0..400usize {
            for _ in 0..(2 + rng.gen_range(3)) {
                runtime_tx
                    .send(RuntimeControlMsg::CancelTelemetryTimer {
                        node_id: exporter_id.index,
                        _temp: Default::default(),
                    })
                    .await
                    .unwrap();
                state.borrow_mut().runtime_noise_sent += 1;
            }

            clock.advance(Duration::from_millis(1));
            yield_cycles(4).await;

            let done = {
                let state = state.borrow();
                state.completed.len() == completion_target
                    && state.admitted.len() == completion_target
                    && state.forwarded.len() == completion_target
                    && state.processor_paused_seen
                    && state.processor_resumed_seen
                    && state.processor_timer_ticks > 0
                    && state.receiver_blocked_count > 0
            };

            if done {
                assert!(
                    step < 399,
                    "seed={seed}: backpressure scenario only completed at final step"
                );
                break;
            }

            if step == 399 {
                let snapshot = state.borrow();
                panic!(
                    "seed={seed}: heavy ingress scenario stalled: admitted={}, forwarded={}, completed={}, blocked={}, timer_ticks={}, paused={}, resumed={}",
                    snapshot.admitted.len(),
                    snapshot.forwarded.len(),
                    snapshot.completed.len(),
                    snapshot.receiver_blocked_count,
                    snapshot.processor_timer_ticks,
                    snapshot.processor_paused_seen,
                    snapshot.processor_resumed_seen,
                );
            }
        }

        {
            let snapshot = state.borrow();
            assert_eq!(
                snapshot.admitted.len(),
                completion_target,
                "seed={seed}: receiver did not admit all expected pdata"
            );
            assert_eq!(
                snapshot.forwarded.len(),
                completion_target,
                "seed={seed}: processor did not forward all admitted pdata"
            );
            assert_eq!(
                snapshot.completed.len(),
                completion_target,
                "seed={seed}: processor did not observe terminal completion for all forwarded pdata"
            );
            assert!(
                snapshot.ack_count > 0 && snapshot.nack_count > 0,
                "seed={seed}: scenario should exercise both ack and nack completions"
            );
        }

        runtime_tx
            .send(RuntimeControlMsg::Shutdown {
                deadline: clock.now() + Duration::from_millis(50),
                reason: format!("dst-heavy-ingress-{seed}"),
            })
            .await
            .unwrap();
        clock.advance(Duration::from_millis(1));
        yield_cycles(8).await;

        timeout(Duration::from_secs(1), receiver_handle)
            .await
            .expect("receiver should exit")
            .unwrap();

        clock.advance(Duration::from_millis(5));
        yield_cycles(8).await;

        timeout(Duration::from_secs(1), processor_handle)
            .await
            .expect("processor should exit")
            .unwrap();
        timeout(Duration::from_secs(1), exporter_handle)
            .await
            .expect("exporter should exit")
            .unwrap();

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

        let snapshot = state.borrow();
        assert!(
            snapshot.receiver_drain_seen,
            "seed={seed}: receiver did not observe DrainIngress"
        );
        assert!(
            snapshot.receiver_drained_sent,
            "seed={seed}: receiver did not report ReceiverDrained"
        );
        assert!(
            snapshot.processor_shutdown_seen,
            "seed={seed}: processor did not observe Shutdown"
        );
        assert!(
            snapshot.exporter_shutdown_seen,
            "seed={seed}: exporter did not observe Shutdown"
        );
        assert!(
            snapshot.runtime_noise_sent > 0,
            "seed={seed}: runtime noise was not generated"
        );
    }));
}

#[test]
fn dst_heavy_ingress_backpressure_seeded() {
    for seed in dst_seeds(&[7, 23, 31], 8) {
        futures::executor::block_on(run_backpressure_interblock_seed(seed));
    }
}
