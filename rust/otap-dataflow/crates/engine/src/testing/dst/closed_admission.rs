// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{DstRng, SimClock, dst_seeds};
use crate::Interests;
use crate::control::NodeControlMsg;
use crate::message::{Message, ProcessorMessageChannel, Receiver};
use crate::testing::dst::common::{setup_dst_runtime, yield_cycles};
use otap_df_channel::mpsc;
use std::time::Duration;

async fn run_closed_admission_deadline_seed(seed: u64) {
    let clock = SimClock::new();
    let _clock_guard = clock.install();
    let (rt, local_tasks) = setup_dst_runtime();

    rt.block_on(local_tasks.run_until(async move {
        let mut rng = DstRng::new(seed);
        let (control_tx, control_rx) = mpsc::Channel::<NodeControlMsg<String>>::new(32);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(32);
        let mut channel = ProcessorMessageChannel::new(
            Receiver::Local(crate::local::message::LocalReceiver::mpsc(control_rx)),
            Receiver::Local(crate::local::message::LocalReceiver::mpsc(pdata_rx)),
            9,
            Interests::empty(),
        );

        let buffered = 3 + rng.gen_range(3);
        for idx in 0..buffered {
            pdata_tx
                .send_async(format!("stuck-{seed}-{idx}"))
                .await
                .expect("pdata should enqueue");
        }

        control_tx
            .send_async(NodeControlMsg::Shutdown {
                deadline: clock.now() + Duration::from_millis(20),
                reason: format!("closed-admission-{seed}"),
            })
            .await
            .expect("shutdown should enqueue");
        control_tx
            .send_async(NodeControlMsg::TimerTick {})
            .await
            .expect("control should enqueue");
        control_tx
            .send_async(NodeControlMsg::Config {
                config: serde_json::json!({ "seed": seed }),
            })
            .await
            .expect("config should enqueue");

        let first = channel
            .recv_when(false)
            .await
            .expect("first control should arrive");
        assert!(
            matches!(first, Message::Control(NodeControlMsg::TimerTick {})),
            "seed={seed}: closed admission should still deliver TimerTick while draining"
        );

        let second = channel
            .recv_when(false)
            .await
            .expect("second control should arrive");
        assert!(
            matches!(second, Message::Control(NodeControlMsg::Config { .. })),
            "seed={seed}: closed admission should still deliver Config while draining"
        );

        clock.advance(Duration::from_millis(25));
        yield_cycles(2).await;

        let shutdown = channel
            .recv_when(false)
            .await
            .expect("shutdown should arrive after deadline");
        assert!(
            matches!(shutdown, Message::Control(NodeControlMsg::Shutdown { .. })),
            "seed={seed}: deadline should force shutdown while admission stays closed"
        );

        let closed = channel.recv_when(true).await;
        assert!(
            closed.is_err(),
            "seed={seed}: after forced shutdown the buffered pdata is no longer recoverable"
        );
    }));
}

// Keep the known processor limitation explicit in the DST suite: if shutdown is
// latched and admission never reopens, buffered pdata remains stranded until
// the deadline and is abandoned when forced Shutdown is returned.
#[test]
fn dst_closed_admission_deadline_abandons_buffered_pdata_seeded() {
    for seed in dst_seeds(&[11, 37, 43], 8) {
        futures::executor::block_on(run_closed_admission_deadline_seed(seed));
    }
}
