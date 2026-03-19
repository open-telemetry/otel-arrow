// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{DstRng, SimClock, dst_seeds};
use crate::Interests;
use crate::control::NodeControlMsg;
use crate::message::{ExporterMessageChannel, Message, ProcessorMessageChannel, Receiver};
use crate::testing::dst::common::setup_dst_runtime;
use otap_df_channel::mpsc;
use std::time::Duration;

async fn run_message_channel_seed(seed: u64) {
    let clock = SimClock::new();
    let _clock_guard = clock.install();
    let (rt, local_tasks) = setup_dst_runtime();

    rt.block_on(local_tasks.run_until(async move {
        let mut rng = DstRng::new(seed);

        // Fairness phase.
        let (control_tx, control_rx) = mpsc::Channel::<NodeControlMsg<String>>::new(128);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(128);
        let mut channel = ProcessorMessageChannel::new(
            Receiver::Local(crate::local::message::LocalReceiver::mpsc(control_rx)),
            Receiver::Local(crate::local::message::LocalReceiver::mpsc(pdata_rx)),
            1,
            Interests::empty(),
        );
        let pdata = format!("seed-{seed}-fair");
        pdata_tx.send_async(pdata.clone()).await.unwrap();

        let burst = 33 + rng.gen_range(8);
        for _ in 0..burst {
            control_tx
                .send_async(NodeControlMsg::TimerTick {})
                .await
                .unwrap();
        }

        let gated_controls = 1 + rng.gen_range(4);
        for _ in 0..gated_controls {
            let msg = channel.recv_when(false).await.unwrap();
            assert!(
                matches!(msg, Message::Control(NodeControlMsg::TimerTick {})),
                "seed={seed}: recv_when(false) must not deliver pdata"
            );
        }

        let mut saw_pdata = false;
        let mut controls_after_open = 0usize;
        while controls_after_open <= 32 {
            match channel.recv_when(true).await.unwrap() {
                Message::PData(value) => {
                    assert_eq!(value, pdata, "seed={seed}: wrong pdata after control burst");
                    saw_pdata = true;
                    break;
                }
                Message::Control(NodeControlMsg::TimerTick {}) => {
                    controls_after_open += 1;
                }
                other => panic!("seed={seed}: unexpected message {other:?}"),
            }
        }
        assert!(
            saw_pdata,
            "seed={seed}: pdata was starved after {controls_after_open} control messages"
        );

        // Draining phase with explicit deadline expiry.
        let (control_tx, control_rx) = mpsc::Channel::<NodeControlMsg<String>>::new(128);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(128);
        let mut channel = ProcessorMessageChannel::new(
            Receiver::Local(crate::local::message::LocalReceiver::mpsc(control_rx)),
            Receiver::Local(crate::local::message::LocalReceiver::mpsc(pdata_rx)),
            2,
            Interests::empty(),
        );
        pdata_tx.send_async("drain-me".to_owned()).await.unwrap();
        control_tx
            .send_async(NodeControlMsg::Shutdown {
                deadline: clock.now() + Duration::from_millis(20),
                reason: format!("seed-{seed}-shutdown"),
            })
            .await
            .unwrap();
        for _ in 0..40 {
            control_tx
                .send_async(NodeControlMsg::TimerTick {})
                .await
                .unwrap();
        }

        for _ in 0..3 {
            let msg = channel.recv_when(false).await.unwrap();
            assert!(
                matches!(msg, Message::Control(_)),
                "seed={seed}: gated draining should only deliver control"
            );
        }

        let mut drained = false;
        for _ in 0..40 {
            match channel.recv_when(true).await.unwrap() {
                Message::PData(value) => {
                    assert_eq!(value, "drain-me", "seed={seed}: wrong drained pdata");
                    drained = true;
                    break;
                }
                Message::Control(NodeControlMsg::TimerTick {}) => {}
                other => panic!("seed={seed}: unexpected draining message {other:?}"),
            }
        }
        assert!(drained, "seed={seed}: pdata did not drain during shutdown");
        drop(pdata_tx);
        let shutdown = channel.recv_when(true).await.unwrap();
        assert!(
            matches!(shutdown, Message::Control(NodeControlMsg::Shutdown { .. })),
            "seed={seed}: shutdown should follow drained pdata"
        );

        // Exporter draining while admission stays closed.
        let (control_tx, control_rx) = mpsc::Channel::<NodeControlMsg<String>>::new(16);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(16);
        let mut channel = ExporterMessageChannel::new(
            Receiver::Local(crate::local::message::LocalReceiver::mpsc(control_rx)),
            Receiver::Local(crate::local::message::LocalReceiver::mpsc(pdata_rx)),
            3,
            Interests::empty(),
        );
        pdata_tx.send_async("stuck".to_owned()).await.unwrap();
        control_tx
            .send_async(NodeControlMsg::Shutdown {
                deadline: clock.now() + Duration::from_millis(50),
                reason: format!("seed-{seed}-deadline"),
            })
            .await
            .unwrap();
        control_tx
            .send_async(NodeControlMsg::TimerTick {})
            .await
            .unwrap();
        let msg = channel.recv_when(false).await.unwrap();
        assert!(
            matches!(msg, Message::Control(NodeControlMsg::TimerTick {})),
            "seed={seed}: recv_when(false) must keep delivering control while draining"
        );

        let drained = channel.recv_when(false).await.unwrap();
        assert!(
            matches!(drained, Message::PData(ref value) if value == "stuck"),
            "seed={seed}: exporter should drain buffered pdata during shutdown without reopening admission"
        );

        drop(pdata_tx);
        let shutdown = channel.recv_when(false).await.unwrap();
        assert!(
            matches!(shutdown, Message::Control(NodeControlMsg::Shutdown { .. })),
            "seed={seed}: shutdown should follow once buffered pdata is drained"
        );
    }));
}

// Sweep seeded message-channel interleavings that combine bounded-fair control
// vs pdata delivery, processor shutdown draining with admission reopen, and the
// exporter-specific forced-drain behavior once Shutdown is latched.
#[test]
fn dst_message_channel_seeded() {
    for seed in dst_seeds(&[3, 19, 41], 8) {
        futures::executor::block_on(run_message_channel_seed(seed));
    }
}
