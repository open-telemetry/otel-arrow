// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Seeded shutdown-drain scenarios found by the TLA+ model in
//! `rust/otap-dataflow/formal/shutdown` (spec.md leads L2b and L2c).
//!
//! Each scenario runs the real `ProcessorInbox` and node-local scheduler under
//! a `SimClock`. They encode the drain contract of `shutdown_drain_complete`:
//! a processor is released from drain only once its input is closed and empty
//! and every payload it retained locally has come back to it.

use super::{DstRng, SimClock, dst_seeds};
use crate::Interests;
use crate::control::NodeControlMsg;
use crate::local::message::LocalReceiver;
use crate::message::{Message, ProcessorInbox, Receiver};
use crate::node_local_scheduler::NodeLocalSchedulerHandle;
use crate::testing::dst::common::setup_dst_runtime;
use otel_arrow_dfe_channel::mpsc;
use std::time::{Duration, Instant};

type Inbox = (
    mpsc::Sender<NodeControlMsg<String>>,
    mpsc::Sender<String>,
    NodeLocalSchedulerHandle<String>,
    ProcessorInbox<String>,
);

fn processor_inbox_with_scheduler(node_id: usize) -> Inbox {
    let (control_tx, control_rx) = mpsc::Channel::<NodeControlMsg<String>>::new(64);
    let (pdata_tx, pdata_rx) = mpsc::Channel::<String>::new(64);
    let scheduler = NodeLocalSchedulerHandle::new(64, 4);
    let inbox = ProcessorInbox::new_with_local_scheduler(
        Receiver::Local(LocalReceiver::mpsc(control_rx)),
        Receiver::Local(LocalReceiver::mpsc(pdata_rx)),
        scheduler.clone(),
        node_id,
        Interests::empty(),
    );
    (control_tx, pdata_tx, scheduler, inbox)
}

fn shutdown(deadline: Instant, seed: u64) -> NodeControlMsg<String> {
    NodeControlMsg::Shutdown {
        deadline,
        reason: format!("shutdown-drain-{seed}"),
    }
}

// L2b. A retry-like processor holds payloads in `requeue_later`. Its upstream
// receiver drains, returns and drops its pdata sender. The manager's Shutdown
// for this node may arrive before that, or only after the node has already
// seen its input close: the seed picks the order.
async fn run_input_closes_before_shutdown_seed(seed: u64) {
    let clock = SimClock::new();
    let _clock_guard = clock.install();
    let (rt, local_tasks) = setup_dst_runtime();

    rt.block_on(local_tasks.run_until(async move {
        let mut rng = DstRng::new(seed);
        let (control_tx, pdata_tx, scheduler, mut inbox) = processor_inbox_with_scheduler(21);

        let retained = 1 + rng.gen_range(4);
        for idx in 0..retained {
            scheduler
                .requeue_later(
                    clock.now() + Duration::from_millis(100 + idx as u64),
                    Box::new(format!("retained-{seed}-{idx}")),
                )
                .expect("delayed resume should schedule");
        }

        let deadline = clock.now() + Duration::from_secs(5);
        let shutdown_first = rng.next_bool();
        if shutdown_first {
            control_tx
                .send_async(shutdown(deadline, seed))
                .await
                .expect("shutdown should enqueue");
        }
        drop(pdata_tx);

        let mut shutdown_pending = !shutdown_first;
        let mut resumed = 0usize;
        loop {
            let msg = inbox.recv_when(true).await;
            if shutdown_pending {
                // The manager's Shutdown arrives after the node already
                // polled its inbox; the channel may be gone by then.
                shutdown_pending = false;
                let _ = control_tx.send_async(shutdown(deadline, seed)).await;
            }
            match msg {
                Ok(Message::Control(NodeControlMsg::ResumeData { .. })) => resumed += 1,
                Ok(Message::Control(NodeControlMsg::Shutdown { .. })) => {}
                Ok(other) => panic!("seed={seed}: unexpected message {other:?}"),
                Err(_) => break,
            }
        }

        assert_eq!(
            resumed, retained,
            "seed={seed} shutdown_first={shutdown_first}: every retained payload must come \
             back as ResumeData before the inbox closes"
        );
    }));
}

/// Scenario: a processor holds payloads in `requeue_later` when its upstream
/// drops the pdata sender, with the node's `Shutdown` delivered either before
/// the input closes or only after the node has observed the closed input.
/// Guarantees: in both orders every retained payload is delivered back as
/// `ResumeData` before the inbox reports `Closed` (spec.md S3, S4, lead L2b).
#[test]
fn dst_retained_payloads_survive_input_closing_before_shutdown_seeded() {
    for seed in dst_seeds(&[1, 2, 3, 4], 8) {
        futures::executor::block_on(run_input_closes_before_shutdown_seed(seed));
    }
}

// L2c. Shutdown is latched with more retained payloads than
// CONTROL_BURST_LIMIT and the input already closed. Due ResumeData count as
// control, so the drain-mode fairness path probes pdata after 32 of them.
async fn run_burst_limit_retained_seed(seed: u64) {
    let clock = SimClock::new();
    let _clock_guard = clock.install();
    let (rt, local_tasks) = setup_dst_runtime();

    rt.block_on(local_tasks.run_until(async move {
        let mut rng = DstRng::new(seed);
        let (control_tx, pdata_tx, scheduler, mut inbox) = processor_inbox_with_scheduler(22);

        let retained = 33 + rng.gen_range(8);
        for idx in 0..retained {
            scheduler
                .requeue_later(
                    clock.now() + Duration::from_millis(100),
                    Box::new(format!("retained-{seed}-{idx}")),
                )
                .expect("delayed resume should schedule");
        }
        control_tx
            .send_async(shutdown(clock.now() + Duration::from_secs(5), seed))
            .await
            .expect("shutdown should enqueue");
        drop(pdata_tx);

        let mut resumed = 0usize;
        let mut released = false;
        loop {
            match inbox.recv_when(true).await {
                Ok(Message::Control(NodeControlMsg::ResumeData { .. })) => {
                    assert!(
                        !released,
                        "seed={seed}: ResumeData after the Shutdown release"
                    );
                    resumed += 1;
                }
                Ok(Message::Control(NodeControlMsg::Shutdown { .. })) => released = true,
                Ok(other) => panic!("seed={seed}: unexpected message {other:?}"),
                Err(_) => break,
            }
        }

        assert_eq!(
            resumed, retained,
            "seed={seed}: Shutdown was released with retained payloads still in the scheduler"
        );
    }));
}

/// Scenario: shutdown is latched while a processor retains more payloads
/// than `CONTROL_BURST_LIMIT` and its pdata input is already closed and
/// empty.
/// Guarantees: the control-fairness path does not release `Shutdown` early;
/// every retained payload is delivered as `ResumeData` first (spec.md S3, S4,
/// lead L2c).
#[test]
fn dst_retained_payloads_survive_control_burst_during_drain_seeded() {
    for seed in dst_seeds(&[5, 6], 4) {
        futures::executor::block_on(run_burst_limit_retained_seed(seed));
    }
}
