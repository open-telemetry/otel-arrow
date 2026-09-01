// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_channel::mpsc::Channel;
use otel_arrow_dfe_engine::local::message::LocalReceiver;
use otel_arrow_dfe_engine::message::Receiver;
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

fn control_channel(message: NodeControlMsg<OtapPdata>) -> local::ControlChannel<OtapPdata> {
    let (sender, receiver) = Channel::new(1);
    sender
        .send(message)
        .expect("control channel should have capacity");
    local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(receiver)))
}

fn closed_control_channel() -> local::ControlChannel<OtapPdata> {
    let (sender, receiver) = Channel::new(1);
    drop(sender);
    local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(receiver)))
}

#[derive(Clone)]
struct TestCancellation {
    cancelled: Rc<Cell<bool>>,
}

#[derive(Debug, thiserror::Error)]
#[error("test cancellation failed")]
struct TestCancellationError;

#[async_trait(?Send)]
impl DriverCancellation for TestCancellation {
    type Error = TestCancellationError;

    async fn cancel(&self) -> Result<(), Self::Error> {
        self.cancelled.set(true);
        Ok(())
    }
}

/// Scenario: Shutdown arrives while the receiver is waiting for the next polling interval.
/// Guarantees: The scheduler wait is dropped immediately and no new database operation starts.
#[tokio::test]
async fn stop_interrupts_scheduler_wait() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut control = control_channel(NodeControlMsg::Shutdown {
        deadline,
        reason: "test".to_owned(),
    });

    let stop = tokio::time::timeout(
        Duration::from_millis(100),
        wait_for_schedule_or_stop(tokio::time::sleep(Duration::from_secs(60)), &mut control),
    )
    .await
    .expect("scheduler wait should be interrupted")
    .expect("control receive should succeed");

    assert!(matches!(stop, Some(StopRequest::Shutdown(value)) if value == deadline));
}

/// Scenario: Shutdown arrives after a native database operation has started.
/// Guarantees: Cancellation is requested and the operation finishes before termination is reported.
#[tokio::test]
async fn stop_cancels_and_joins_active_operation() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut control = control_channel(NodeControlMsg::Shutdown {
        deadline,
        reason: "test".to_owned(),
    });
    let cancelled = Rc::new(Cell::new(false));
    let completed = Rc::new(Cell::new(false));
    let operation_cancelled = Rc::clone(&cancelled);
    let operation_completed = Rc::clone(&completed);
    let operation = async move {
        while !operation_cancelled.get() {
            tokio::task::yield_now().await;
        }
        operation_completed.set(true);
    };

    let outcome = await_database_operation_or_stop(
        operation,
        TestCancellation {
            cancelled: Rc::clone(&cancelled),
        },
        &mut control,
    )
    .await
    .expect("controlled operation should finish");

    assert!(matches!(
        outcome,
        OperationOutcome::Stopped(StopRequest::Shutdown(value)) if value == deadline
    ));
    assert!(cancelled.get());
    assert!(completed.get());
}

/// Scenario: The control channel closes while a native database operation is active.
/// Guarantees: Channel failure cancels and joins the operation before returning an error.
#[tokio::test]
async fn closed_control_channel_cancels_and_joins_active_operation() {
    let mut control = closed_control_channel();
    let cancelled = Rc::new(Cell::new(false));
    let completed = Rc::new(Cell::new(false));
    let operation_cancelled = Rc::clone(&cancelled);
    let operation_completed = Rc::clone(&completed);
    let operation = async move {
        while !operation_cancelled.get() {
            tokio::task::yield_now().await;
        }
        operation_completed.set(true);
    };

    let result = await_database_operation_or_stop(
        operation,
        TestCancellation {
            cancelled: Rc::clone(&cancelled),
        },
        &mut control,
    )
    .await;

    assert!(matches!(result, Err(Error::ChannelRecvError(_))));
    assert!(cancelled.get());
    assert!(completed.get());
}
