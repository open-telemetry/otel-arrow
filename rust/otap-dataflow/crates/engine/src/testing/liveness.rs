// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared liveness-oriented test helpers.
//!
//! These helpers keep the "eventual progress or explicit terminal outcome"
//! assertions consistent across node-local tests and small integration tests.

use crate::control::{
    PipelineCompletionMsg, PipelineCompletionMsgReceiver, RuntimeCtrlMsgReceiver,
};
use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;

/// Awaits a future and fails the test if it does not complete within `duration`.
pub async fn completes_within<Fut, T>(duration: Duration, label: &str, future: Fut) -> T
where
    Fut: Future<Output = T>,
{
    timeout(duration, future)
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for {label}"))
}

/// Awaits the next pipeline-completion message and fails if the timeout expires.
pub async fn next_completion<PData>(
    receiver: &mut PipelineCompletionMsgReceiver<PData>,
    duration: Duration,
    label: &str,
) -> PipelineCompletionMsg<PData> {
    completes_within(duration, label, receiver.recv())
        .await
        .expect("pipeline-completion channel closed unexpectedly")
}

/// Awaits the next runtime-control message and fails if the timeout expires.
pub async fn next_runtime_control<PData>(
    receiver: &mut RuntimeCtrlMsgReceiver<PData>,
    duration: Duration,
    label: &str,
) -> crate::control::RuntimeControlMsg<PData> {
    completes_within(duration, label, receiver.recv())
        .await
        .expect("runtime-control channel closed unexpectedly")
}

/// Polls a condition until it becomes true or the timeout expires.
pub fn wait_for_condition<F>(timeout: Duration, poll_interval: Duration, condition: F) -> bool
where
    F: Fn() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        std::thread::sleep(poll_interval);
    }
    false
}
