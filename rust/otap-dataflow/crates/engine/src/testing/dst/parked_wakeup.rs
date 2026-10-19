// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Seeded scenario found by the TLA+ model in
//! `rust/otap-dataflow/formal/shutdown` (spec.md lead L7): a wakeup that owns
//! a parked payload must survive the shutdown latch.

use super::{DstRng, SimClock, dst_seeds};
use crate::Interests;
use crate::control::{NodeControlMsg, WakeupSlot};
use crate::local::message::LocalReceiver;
use crate::message::{Message, ProcessorInbox, Receiver};
use crate::node_local_scheduler::NodeLocalSchedulerHandle;
use crate::testing::dst::common::{setup_dst_runtime, yield_cycles};
use otel_arrow_dfe_channel::mpsc;
use std::time::Duration;

fn router_inbox() -> (
    mpsc::Sender<NodeControlMsg<String>>,
    mpsc::Sender<String>,
    NodeLocalSchedulerHandle<String>,
    ProcessorInbox<String>,
) {
    let (control_tx, control_rx) = mpsc::Channel::<NodeControlMsg<String>>::new(64);
    let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(64);
    let scheduler = NodeLocalSchedulerHandle::new(64, 4);
    let inbox = ProcessorInbox::new_with_local_scheduler(
        Receiver::Local(LocalReceiver::mpsc(control_rx)),
        Receiver::Local(LocalReceiver::mpsc(pdata_rx)),
        scheduler.clone(),
        23,
        Interests::empty(),
    );
    (control_tx, pdata_tx, scheduler, inbox)
}

// L7. An exclusive router in backpressure mode parks a message for a full
// port, arms a wakeup to re-probe it, and closes admission while every
// candidate port is parked (`exclusive_router_admission.rs` 169-206, 244-274,
// 290-296). The test reproduces that pattern on the real inbox and scheduler:
// the payload lives in router state, keyed by the wakeup slot.
async fn run_parked_wakeup_seed(seed: u64) {
    let clock = SimClock::new();
    let _clock_guard = clock.install();
    let (rt, local_tasks) = setup_dst_runtime();

    rt.block_on(local_tasks.run_until(async move {
        let mut rng = DstRng::new(seed);
        let (control_tx, pdata_tx, scheduler, inbox) = router_inbox();

        let wake_at = clock.now() + Duration::from_millis(50 + rng.gen_range(100) as u64);
        let deadline = clock.now() + Duration::from_secs(5);
        let _ = scheduler
            .set_wakeup(WakeupSlot(0), wake_at)
            .expect("router re-probe wakeup should schedule");
        pdata_tx
            .send_async(format!("buffered-{seed}"))
            .await
            .expect("upstream pdata should enqueue");
        control_tx
            .send_async(NodeControlMsg::Shutdown {
                deadline,
                reason: format!("parked-wakeup-{seed}"),
            })
            .await
            .expect("shutdown should enqueue");
        drop(pdata_tx);

        // Admission stays closed while the message is parked.
        let next = tokio::task::spawn_local(async move {
            let mut inbox = inbox;
            inbox.recv_when(false).await
        });
        yield_cycles(4).await;

        // Time passes until the wakeup's original deadline, well before the
        // shutdown deadline.
        clock.advance_to(wake_at + Duration::from_millis(1));
        yield_cycles(8).await;
        assert!(
            next.is_finished(),
            "seed={seed}: the router's re-probe wakeup was not delivered after the shutdown \
             latch; the parked message and the buffered input can only leave at the deadline"
        );
        let msg = next.await.expect("inbox task should not panic");
        assert!(
            matches!(msg, Ok(Message::Control(NodeControlMsg::Wakeup { .. }))),
            "seed={seed}: expected the re-probe Wakeup, got {msg:?}"
        );
    }));
}

/// Scenario: a router-like processor has armed a wakeup to re-probe a parked
/// message and keeps admission closed, with upstream pdata buffered, when
/// `Shutdown` is latched.
/// Guarantees: the wakeup that owns the parked payload is still delivered
/// during drain, so the processor can forward it and reopen admission before
/// the deadline (spec.md S7, lead L7).
#[test]
fn dst_parked_payload_wakeup_survives_shutdown_latch_seeded() {
    for seed in dst_seeds(&[7, 8], 4) {
        futures::executor::block_on(run_parked_wakeup_seed(seed));
    }
}
