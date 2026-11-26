// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::{future::Future, task::Poll};
use tokio::sync::oneshot;

const GUARD: Duration = Duration::from_secs(30);
const IDLE: Duration = Duration::from_secs(3600);

fn setup() -> (
    Workers,
    Discovery<u8>,
    ReadCheckpoint<u8, u8, u8>,
    Async<u8, u8>,
) {
    let workers = Workers::new(GUARD).unwrap();
    let (discovery, reader, owner) = channels(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
        workers.cancellation(),
        workers.cancellation(),
    );
    (workers, discovery, reader, owner)
}

// OS threads and their wakeups use real time. Timeouts only guard explicit
// signals; no elapsed interval establishes readiness or proves backpressure.
async fn signalled<T>(signal: oneshot::Receiver<T>) -> T {
    tokio::time::timeout(GUARD, signal)
        .await
        .expect("worker did not reach the signalled transition")
        .expect("worker dropped its transition signal")
}

async fn pending<F: Future>(future: F, signal: oneshot::Sender<()>) -> F::Output {
    let mut signal = Some(signal);
    let mut future = std::pin::pin!(future);
    futures::future::poll_fn(|cx| match future.as_mut().poll(cx) {
        Poll::Pending => {
            if let Some(signal) = signal.take() {
                signal.send(()).expect("pending observer dropped");
            }
            Poll::Pending
        }
        Poll::Ready(value) => Poll::Ready(value),
    })
    .await
}

fn wait_for_cancellation(cancellation: &Cancellation) {
    assert_eq!(cancellation.wait(IDLE), Err(ChannelError::Cancelled));
}

fn joined(report: JoinReport, discovery: bool, reader: bool) {
    assert_eq!(
        report,
        JoinReport {
            discovery: discovery.then_some(WorkerOutcome::Joined {
                deadline_missed: false,
            }),
            read_checkpoint: reader.then_some(WorkerOutcome::Joined {
                deadline_missed: false,
            }),
        }
    );
}

fn control_clock(cancellation: &Cancellation) {
    *cancellation.0.test_now.lock() = Some(Instant::now());
}

fn advance_clock(cancellation: &Cancellation, duration: Duration) {
    let mut now = cancellation.0.test_now.lock();
    *now = Some(
        now.expect("controlled worker clock")
            .checked_add(duration)
            .unwrap(),
    );
}

async fn wait_for_publication(completion: &JoinCompletion) {
    tokio::time::timeout(GUARD, async {
        loop {
            let ready = completion.ready.notified();
            if completion.completed.lock().is_some() {
                return;
            }
            ready.await;
        }
    })
    .await
    .expect("native join observer did not publish completion");
}

/// Scenario: Async cancellation is observed both before and after shutdown is requested.
/// Guarantees: A pending waiter wakes without a channel message, and a later waiter is immediately ready.
#[tokio::test]
async fn async_cancellation_wakes_pending_and_late_waiters() {
    let mut workers = Workers::new(GUARD).unwrap();
    let cancellation = workers.cancellation();
    let mut waiting = std::pin::pin!(cancellation.cancelled());
    assert!(futures::poll!(&mut waiting).is_pending());
    _ = cancellation.cancel();
    assert!(futures::poll!(&mut waiting).is_ready());
    let mut late = std::pin::pin!(cancellation.cancelled());
    assert!(futures::poll!(&mut late).is_ready());
    joined(workers.join().await, false, false);
}

/// Scenario: Discovery hands an observation to a reader that fills the batch slot.
/// Guarantees: Batches arrive in order and commands are acknowledged before the slot is freed.
#[tokio::test]
async fn roundtrip_services_commands_while_batch_slot_is_full() {
    let (mut workers, discovery, mut reader, mut owner) = setup();
    let (blocked_tx, blocked_rx) = oneshot::channel();
    let (ack_tx, ack_rx) = oneshot::channel();
    workers
        .start_discovery(move |cancel| {
            discovery.observe(7).unwrap();
            wait_for_cancellation(&cancel);
        })
        .unwrap();
    workers
        .start_read_checkpoint(move |cancel| {
            assert_eq!(reader.next(), Ok(Input::Observation(7)));
            reader.publish(7, |_| panic!("unexpected command")).unwrap();
            let mut ack = Some(ack_tx);
            block_on(pending(
                reader.publish_async(8, |command| {
                    assert_eq!(command, 9);
                    ack.take().unwrap().send(()).unwrap();
                }),
                blocked_tx,
            ))
            .unwrap();
            wait_for_cancellation(&cancel);
        })
        .unwrap();

    signalled(blocked_rx).await;
    assert_eq!(owner.batches.len(), 1);
    owner.command(9).await.unwrap();
    signalled(ack_rx).await;
    assert_eq!(owner.batches.len(), 1);
    assert_eq!(owner.batch().await, Ok(7));
    assert_eq!(owner.batch().await, Ok(8));
    joined(workers.join().await, true, true);
}

/// Scenario: Discovery is blocked sending a second observation to a slow reader.
/// Guarantees: The queue remains bounded and cancellation wakes the producer without draining it.
#[tokio::test]
async fn cancellation_wakes_full_observation_queue() {
    let (mut workers, discovery, reader, _owner) = setup();
    let (blocked_tx, blocked_rx) = oneshot::channel();
    let (result_tx, result_rx) = oneshot::channel();
    workers
        .start_discovery(move |_| {
            discovery.observe(1).unwrap();
            let result = block_on(pending(discovery.observe_async(2), blocked_tx));
            result_tx.send(result).unwrap();
        })
        .unwrap();

    signalled(blocked_rx).await;
    assert_eq!(reader.observations.len(), 1);
    _ = workers.cancellation().cancel();
    assert_eq!(signalled(result_rx).await, Err(ChannelError::Cancelled));
    assert_eq!(reader.observations.len(), 1);
    joined(workers.join().await, true, false);
}

/// Scenario: Discovery is idle between passes and the reader has no observations or commands.
/// Guarantees: Out-of-band cancellation wakes both waits without sending any channel message.
#[tokio::test]
async fn cancellation_wakes_idle_discovery_and_empty_reader() {
    let (mut workers, _discovery, mut reader, _owner) = setup();
    let (idle_tx, idle_rx) = oneshot::channel();
    let (empty_tx, empty_rx) = oneshot::channel();
    workers
        .start_discovery(move |cancel| {
            assert_eq!(
                block_on(pending(cancel.wait_async(IDLE), idle_tx)),
                Err(ChannelError::Cancelled)
            );
        })
        .unwrap();
    workers
        .start_read_checkpoint(move |_| {
            assert_eq!(
                block_on(pending(
                    reader.next_async(true, std::future::pending()),
                    empty_tx
                )),
                Err(ChannelError::Cancelled)
            );
        })
        .unwrap();

    signalled(idle_rx).await;
    signalled(empty_rx).await;
    joined(workers.join().await, true, true);
}

/// Scenario: A second batch is waiting behind a full async handoff slot.
/// Guarantees: Cancellation wakes the reader without requiring the consumer to free the slot.
#[tokio::test]
async fn cancellation_wakes_full_batch_slot() {
    let (mut workers, _discovery, mut reader, owner) = setup();
    let (blocked_tx, blocked_rx) = oneshot::channel();
    workers
        .start_read_checkpoint(move |_| {
            reader.publish(1, |_| panic!("unexpected command")).unwrap();
            assert_eq!(
                block_on(pending(
                    reader.publish_async(2, |_| panic!("unexpected command")),
                    blocked_tx,
                )),
                Err(ChannelError::Cancelled)
            );
        })
        .unwrap();

    signalled(blocked_rx).await;
    assert_eq!(owner.batches.len(), 1);
    joined(workers.join().await, false, true);
    assert_eq!(owner.batches.len(), 1);
}

/// Scenario: A controlled reader wait leaves the command queue full and another sender pending.
/// Guarantees: Cancellation wakes the sender and takes priority over an already queued command.
#[tokio::test]
async fn cancellation_is_independent_of_command_capacity() {
    let (mut workers, _discovery, mut reader, owner) = setup();
    let (release_tx, release_rx) = oneshot::channel();
    workers
        .start_read_checkpoint(move |_| {
            block_on(release_rx).unwrap();
            assert_eq!(reader.next(), Err(ChannelError::Cancelled));
        })
        .unwrap();
    owner.command(1).await.unwrap();
    let mut send = std::pin::pin!(owner.command(2));
    assert!(futures::poll!(&mut send).is_pending());
    _ = workers.cancellation().cancel();
    assert_eq!(send.await, Err(ChannelError::Cancelled));
    release_tx.send(()).unwrap();
    joined(workers.join().await, false, true);
}

async fn panic_cancels_peer(discovery_panics: bool) {
    let (mut workers, _discovery, _reader, _owner) = setup();
    let (waiting_tx, waiting_rx) = oneshot::channel();
    let (panic_tx, panic_rx) = oneshot::channel();
    let (cancelled_tx, cancelled_rx) = oneshot::channel();
    let panic = move |_: Cancellation| {
        block_on(panic_rx).unwrap();
        panic!("controlled worker panic");
    };
    let peer = move |cancel: Cancellation| {
        assert_eq!(
            block_on(pending(cancel.wait_async(IDLE), waiting_tx)),
            Err(ChannelError::Cancelled)
        );
        cancelled_tx.send(()).unwrap();
    };
    if discovery_panics {
        workers.start_discovery(panic).unwrap();
        workers.start_read_checkpoint(peer).unwrap();
    } else {
        workers.start_discovery(peer).unwrap();
        workers.start_read_checkpoint(panic).unwrap();
    }
    signalled(waiting_rx).await;
    panic_tx.send(()).unwrap();
    // Observe peer cancellation before join itself could request cancellation.
    signalled(cancelled_rx).await;
    let report = workers.join().await;
    let panicked = Some(WorkerOutcome::Panicked {
        message: "controlled worker panic".to_owned(),
        deadline_missed: false,
    });
    assert_eq!(
        report,
        if discovery_panics {
            JoinReport {
                discovery: panicked,
                read_checkpoint: Some(WorkerOutcome::Joined {
                    deadline_missed: false,
                }),
            }
        } else {
            JoinReport {
                discovery: Some(WorkerOutcome::Joined {
                    deadline_missed: false,
                }),
                read_checkpoint: panicked,
            }
        }
    );
}

/// Scenario: Discovery panics while the read/checkpoint worker is idle.
/// Guarantees: Its peer is cancelled automatically and the report identifies discovery's panic.
#[tokio::test]
async fn discovery_panic_cancels_reader() {
    panic_cancels_peer(true).await;
}

/// Scenario: The read/checkpoint worker panics while discovery is idle.
/// Guarantees: Discovery is cancelled automatically and the report identifies the reader's panic.
#[tokio::test]
async fn reader_panic_cancels_discovery() {
    panic_cancels_peer(false).await;
}

/// Scenario: One worker ignores cancellation while retaining a resource through the shared deadline.
/// Guarantees: Joining stays async, repeated cancellation preserves the deadline, and only exit drops the resource.
#[tokio::test]
async fn deadline_retains_running_worker_and_its_resources() {
    let (mut workers, _discovery, _reader, _owner) = setup();
    let cancellation = workers.cancellation();
    let (held_tx, held_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let (dropped_tx, mut dropped_rx) = oneshot::channel();
    let (exited_tx, exited_rx) = oneshot::channel();
    let (discovery_held_tx, discovery_held_rx) = oneshot::channel();
    let (discovery_release_tx, discovery_release_rx) = oneshot::channel();
    let (discovery_exited_tx, discovery_exited_rx) = oneshot::channel();
    struct Resource(Option<oneshot::Sender<()>>);
    impl Drop for Resource {
        fn drop(&mut self) {
            _ = self.0.take().unwrap().send(());
        }
    }
    workers
        .start_discovery(move |_| {
            discovery_held_tx.send(()).unwrap();
            _ = block_on(discovery_release_rx);
            discovery_exited_tx.send(()).unwrap();
            panic!("late discovery panic");
        })
        .unwrap();
    workers
        .start_read_checkpoint(move |_| {
            let _guard = Resource(Some(dropped_tx));
            held_tx.send(()).unwrap();
            _ = block_on(release_rx);
            exited_tx.send(()).unwrap();
        })
        .unwrap();
    signalled(held_rx).await;
    signalled(discovery_held_rx).await;

    // Pause only after the real thread has acknowledged its controlled wait.
    control_clock(&cancellation);
    tokio::time::pause();
    let deadline = cancellation.cancel();
    {
        let mut join = std::pin::pin!(workers.join());
        assert!(futures::poll!(&mut join).is_pending());
        advance_clock(&cancellation, GUARD / 2);
        tokio::time::advance(GUARD / 2).await;
        assert_eq!(cancellation.cancel(), deadline);
        assert!(futures::poll!(&mut join).is_pending());
        advance_clock(&cancellation, GUARD / 2);
        tokio::time::advance(GUARD / 2).await;
        assert_eq!(
            join.await,
            JoinReport {
                discovery: Some(WorkerOutcome::StillRunning),
                read_checkpoint: Some(WorkerOutcome::StillRunning),
            }
        );
    }
    assert_eq!(
        dropped_rx.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    );

    tokio::time::resume();
    // The clock still reads exactly the deadline when both outcomes publish.
    discovery_release_tx.send(()).unwrap();
    release_tx.send(()).unwrap();
    signalled(discovery_exited_rx).await;
    signalled(exited_rx).await;
    signalled(dropped_rx).await;
    // Wait for actual native joins, not merely the closure's resource drop.
    wait_for_publication(&workers.discovery.as_ref().unwrap().completion).await;
    wait_for_publication(&workers.read_checkpoint.as_ref().unwrap().completion).await;
    assert_eq!(
        workers.join().await,
        JoinReport {
            discovery: Some(WorkerOutcome::Panicked {
                message: "late discovery panic".to_owned(),
                deadline_missed: true,
            }),
            read_checkpoint: Some(WorkerOutcome::Joined {
                deadline_missed: true,
            }),
        }
    );
    assert_eq!(cancellation.cancel(), deadline);
}

/// Scenario: Native cleanup completes on time, but the pipeline next polls after its deadline.
/// Guarantees: The recorded native completion reports an on-time join despite late pipeline scheduling.
#[tokio::test]
async fn late_pipeline_poll_preserves_on_time_native_join() {
    let mut workers = Workers::new(GUARD).unwrap();
    let cancellation = workers.cancellation();
    let (held_tx, held_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    workers
        .start_discovery(move |_| {
            held_tx.send(()).unwrap();
            _ = block_on(release_rx);
        })
        .unwrap();
    signalled(held_rx).await;
    control_clock(&cancellation);
    let deadline = cancellation.cancel();
    {
        let mut first_poll = std::pin::pin!(workers.join());
        assert!(futures::poll!(&mut first_poll).is_pending());
    }
    advance_clock(&cancellation, GUARD);
    release_tx.send(()).unwrap();
    wait_for_publication(&workers.discovery.as_ref().unwrap().completion).await;
    advance_clock(&cancellation, Duration::from_nanos(1));
    joined(workers.join().await, true, false);
    assert_eq!(cancellation.cancel(), deadline);
    joined(workers.join().await, true, false);
}

/// Scenario: Native cleanup finishes just after the deadline and before the pipeline's next poll.
/// Guarantees: Both the first and later reports preserve the missed deadline even though cleanup finished.
#[tokio::test]
async fn late_native_join_cannot_hide_between_pipeline_polls() {
    let mut workers = Workers::new(GUARD).unwrap();
    let cancellation = workers.cancellation();
    let (held_tx, held_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    workers
        .start_discovery(move |_| {
            held_tx.send(()).unwrap();
            _ = block_on(release_rx);
        })
        .unwrap();
    signalled(held_rx).await;
    control_clock(&cancellation);
    let deadline = cancellation.cancel();
    advance_clock(&cancellation, GUARD + Duration::from_nanos(1));
    release_tx.send(()).unwrap();
    wait_for_publication(&workers.discovery.as_ref().unwrap().completion).await;

    let expected = JoinReport {
        discovery: Some(WorkerOutcome::Joined {
            deadline_missed: true,
        }),
        read_checkpoint: None,
    };
    assert_eq!(workers.join().await, expected);
    advance_clock(&cancellation, GUARD);
    assert_eq!(workers.join().await, expected);
    assert_eq!(cancellation.cancel(), deadline);
}

/// Scenario: A returned worker blocks inside a thread-local destructor beyond its shutdown deadline.
/// Guarantees: The pipeline reports timeout while cleanup is blocked, then records a late native join.
#[tokio::test]
async fn thread_local_cleanup_cannot_block_pipeline_deadline() {
    struct Cleanup {
        entered: Option<oneshot::Sender<()>>,
        release: std::sync::mpsc::Receiver<()>,
        dropped: Option<oneshot::Sender<bool>>,
    }
    impl Drop for Cleanup {
        fn drop(&mut self) {
            _ = self.entered.take().unwrap().send(());
            // A real-time guard also releases cleanup if a regressed native
            // pipeline join blocks the runtime; the test must still fail.
            let released = self.release.recv_timeout(GUARD).is_ok();
            _ = self.dropped.take().unwrap().send(released);
        }
    }
    thread_local! {
        static CLEANUP: std::cell::RefCell<Option<Cleanup>> = const {
            std::cell::RefCell::new(None)
        };
    }
    let mut workers = Workers::new(GUARD).unwrap();
    let cancellation = workers.cancellation();
    let (held_tx, held_rx) = oneshot::channel();
    let (return_tx, return_rx) = oneshot::channel();
    let (entered_tx, entered_rx) = oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let (dropped_tx, mut dropped_rx) = oneshot::channel();
    workers
        .start_read_checkpoint(move |_| {
            CLEANUP.with(|slot| {
                *slot.borrow_mut() = Some(Cleanup {
                    entered: Some(entered_tx),
                    release: release_rx,
                    dropped: Some(dropped_tx),
                });
            });
            held_tx.send(()).unwrap();
            _ = block_on(return_rx);
        })
        .unwrap();
    signalled(held_rx).await;
    control_clock(&cancellation);
    let deadline = cancellation.cancel();
    return_tx.send(()).unwrap();
    signalled(entered_rx).await;

    tokio::time::pause();
    {
        let mut join = std::pin::pin!(workers.join());
        assert!(futures::poll!(&mut join).is_pending());
        advance_clock(&cancellation, GUARD);
        tokio::time::advance(GUARD).await;
        assert_eq!(
            join.await,
            JoinReport {
                discovery: None,
                read_checkpoint: Some(WorkerOutcome::StillRunning),
            }
        );
    }
    assert_eq!(
        dropped_rx.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    );
    tokio::time::resume();
    advance_clock(&cancellation, Duration::from_nanos(1));
    release_tx.send(()).unwrap();
    assert!(
        signalled(dropped_rx).await,
        "thread-local cleanup hit its guard"
    );
    wait_for_publication(&workers.read_checkpoint.as_ref().unwrap().completion).await;
    let expected = JoinReport {
        discovery: None,
        read_checkpoint: Some(WorkerOutcome::Joined {
            deadline_missed: true,
        }),
    };
    assert_eq!(workers.join().await, expected);
    assert_eq!(workers.join().await, expected);
    assert_eq!(cancellation.cancel(), deadline);
}

/// Scenario: A worker returns normally without an owner cancellation request.
/// Guarantees: Unexpected return cancels its peer just like panic, but both joins report success.
#[tokio::test]
async fn unexpected_return_cancels_peer() {
    let (mut workers, _discovery, _reader, _owner) = setup();
    let (waiting_tx, waiting_rx) = oneshot::channel();
    let (exit_tx, exit_rx) = oneshot::channel();
    let (cancelled_tx, cancelled_rx) = oneshot::channel();
    workers
        .start_discovery(move |cancel| {
            assert_eq!(
                block_on(pending(cancel.wait_async(IDLE), waiting_tx)),
                Err(ChannelError::Cancelled)
            );
            cancelled_tx.send(()).unwrap();
        })
        .unwrap();
    workers
        .start_read_checkpoint(move |_| {
            block_on(exit_rx).unwrap();
        })
        .unwrap();
    signalled(waiting_rx).await;
    exit_tx.send(()).unwrap();
    signalled(cancelled_rx).await;
    joined(workers.join().await, true, true);
}

/// Scenario: Cancellation occurs before startup or with no workers started.
/// Guarantees: Empty owners join cleanly and cannot start work after cancellation.
#[tokio::test]
async fn cancellation_before_startup_prevents_new_workers() {
    let mut workers = Workers::new(GUARD).unwrap();
    joined(workers.join().await, false, false);
    assert_eq!(
        workers
            .start_discovery(|_| panic!("must not start"))
            .unwrap_err()
            .kind(),
        io::ErrorKind::Interrupted
    );
    assert_eq!(
        workers
            .start_read_checkpoint(|_| panic!("must not start"))
            .unwrap_err()
            .kind(),
        io::ErrorKind::Interrupted
    );
}

/// Scenario: The async task is waiting for its first batch when shutdown starts.
/// Guarantees: Cancellation wakes an empty handoff without requiring a worker to publish.
#[tokio::test]
async fn cancellation_wakes_empty_async_handoff() {
    let (workers, _discovery, _reader, mut owner) = setup();
    let mut batch = std::pin::pin!(owner.batch());
    assert!(futures::poll!(&mut batch).is_pending());
    _ = workers.cancellation().cancel();
    assert_eq!(batch.await, Err(ChannelError::Cancelled));
}

/// Scenario: Peers drop each of the transport endpoints without cancelling the owner.
/// Guarantees: Channel closure is a typed failure, not a fabricated message or successful handoff.
#[tokio::test]
async fn closed_endpoints_report_transport_failure() {
    {
        let (_workers, discovery, reader, mut owner) = setup();
        drop(reader);
        assert_eq!(discovery.observe(1), Err(ChannelError::ObservationsClosed));
        assert_eq!(owner.command(2).await, Err(ChannelError::CommandsClosed));
        assert_eq!(owner.batch().await, Err(ChannelError::BatchesClosed));
    }
    {
        let (_workers, discovery, mut reader, _owner) = setup();
        drop(discovery);
        assert_eq!(reader.next(), Err(ChannelError::ObservationsClosed));
    }
    {
        let (_workers, _discovery, mut reader, owner) = setup();
        drop(owner);
        assert_eq!(reader.next(), Err(ChannelError::CommandsClosed));
    }
}

/// Scenario: Discovery and reading belong to different owners, as with a future coordinator.
/// Guarantees: Cancelling the namespace reader does not cancel separately owned discovery.
#[tokio::test]
async fn independently_owned_discovery_has_separate_shutdown() {
    let mut discovery_workers = Workers::new(GUARD).unwrap();
    let mut read_workers = Workers::new(GUARD).unwrap();
    let (_discovery, _reader, _owner) = channels::<u8, u8, u8>(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
        discovery_workers.cancellation(),
        read_workers.cancellation(),
    );
    let (waiting_tx, waiting_rx) = oneshot::channel();
    discovery_workers
        .start_discovery(move |cancel| {
            assert_eq!(
                block_on(pending(cancel.wait_async(IDLE), waiting_tx)),
                Err(ChannelError::Cancelled)
            );
        })
        .unwrap();
    signalled(waiting_rx).await;
    joined(read_workers.join().await, false, false);
    assert!(!discovery_workers.cancellation().is_cancelled());
    joined(discovery_workers.join().await, true, false);
}

/// Scenario: A control event interrupts Ack forwarding while the command queue is full.
/// Guarantees: The caller can handle control first and deliver its retained Ack exactly once.
#[tokio::test]
async fn ack_survives_interrupted_reservation() {
    let workers = Workers::new(GUARD).unwrap();
    let (_discovery, mut reader, owner) = channels::<(), (), String>(
        NonZeroUsize::MIN,
        NonZeroUsize::MIN,
        workers.cancellation(),
        workers.cancellation(),
    );
    owner.command("first".to_owned()).await.unwrap();
    let ack = "ack".to_owned();
    let (control, event) = oneshot::channel();
    let forward = async {
        tokio::select! {
            biased;
            permit = owner.reserve_command() => panic!("unexpected capacity: {:?}", permit.is_ok()),
            result = event => result.unwrap(),
        }
    };
    let mut forward = std::pin::pin!(forward);
    assert!(futures::poll!(&mut forward).is_pending());
    control.send(()).unwrap();
    forward.await;
    assert_eq!(reader.commands.try_recv().unwrap(), "first");
    owner.reserve_command().await.unwrap().send(ack);
    assert_eq!(reader.commands.try_recv().unwrap(), "ack");
    assert_eq!(
        reader.commands.try_recv(),
        Err(mpsc::error::TryRecvError::Empty)
    );
}

/// Scenario: An idle worker awaits its earliest deadline without observations or commands.
/// Guarantees: The armed timer wakes the worker for due work, including an already due deadline.
#[tokio::test(start_paused = true)]
async fn idle_wait_services_deadline() {
    let (_workers, _discovery, mut reader, _owner) = setup();
    {
        let timer = tokio::time::sleep(GUARD);
        let mut waiting = std::pin::pin!(reader.next_async(true, timer));
        assert!(futures::poll!(&mut waiting).is_pending());
        tokio::time::advance(GUARD).await;
        assert_eq!(waiting.await, Ok(Input::Deadline));
    }
    assert_eq!(
        reader.next_until(false, Some(Instant::now())),
        Ok(Input::Deadline)
    );
}

/// Scenario: Intake pauses with an observation queued while control and cancellation arrive.
/// Guarantees: Both wake the worker while the queued observation remains untouched.
#[tokio::test]
async fn paused_intake_services_commands_and_cancellation() {
    let (workers, discovery, mut reader, owner) = setup();
    discovery.observe(7).unwrap();
    {
        let mut waiting = std::pin::pin!(reader.next_async(false, std::future::pending()));
        assert!(futures::poll!(&mut waiting).is_pending());
        owner.command(2).await.unwrap();
        assert_eq!(waiting.await, Ok(Input::Command(2)));
    }
    {
        let mut waiting = std::pin::pin!(reader.next_async(false, std::future::pending()));
        assert!(futures::poll!(&mut waiting).is_pending());
        _ = workers.cancellation().cancel();
        assert_eq!(waiting.await, Err(ChannelError::Cancelled));
    }
    assert_eq!(reader.observations.try_recv(), Ok(7));
}

/// Scenario: The observer is suspended after sampling the completion time at the deadline.
/// Guarantees: A competing timeout check cannot observe an empty slot during that interval.
#[tokio::test]
async fn timestamp_and_publication_exclude_a_competing_timeout_check() {
    let workers = Workers::new(GUARD).unwrap();
    let clock = workers.cancellation();
    control_clock(&clock);
    let deadline = clock.cancel();
    advance_clock(&clock, GUARD);

    let completion = Arc::new(JoinCompletion {
        completed: Mutex::new(None),
        ready: Notify::new(),
    });
    let observer = completion.clone();
    let observer_clock = clock.clone();
    let (sampled, sample) = oneshot::channel();
    let (release, resume) = std::sync::mpsc::channel();
    let publisher = thread::spawn(move || {
        observer.publish(Ok(()), || {
            let at = observer_clock.now();
            sampled.send(at).unwrap();
            resume.recv_timeout(GUARD).unwrap();
            at
        });
    });
    assert_eq!(signalled(sample).await, deadline);
    advance_clock(&clock, GUARD);
    let could_inspect_slot = completion.completed.try_lock().is_some();
    release.send(()).unwrap();
    publisher.join().unwrap();
    assert!(
        !could_inspect_slot,
        "timeout check could inspect the unpublished slot"
    );
    assert_eq!(completion.completed.lock().as_ref().unwrap().at, deadline);
}

/// Scenario: A paused reader has an overdue deadline and its one-slot command queue is refilled.
/// Guarantees: Commands and due work alternate while the queued observation stays untouched.
#[tokio::test]
async fn paused_reader_services_due_work_with_a_refilled_command_queue() {
    let (workers, discovery, mut reader, owner) = setup();
    let clock = workers.cancellation();
    control_clock(&clock);
    let deadline = Some(clock.now());
    discovery.observe(1).unwrap();
    owner.command(2).await.unwrap();
    assert_eq!(reader.next_until(false, deadline), Ok(Input::Command(2)));
    owner.command(3).await.unwrap();
    assert_eq!(reader.next_until(false, deadline), Ok(Input::Deadline));
    assert_eq!(reader.next_until(false, deadline), Ok(Input::Command(3)));
    assert_eq!(reader.next(), Ok(Input::Observation(1)));
}

/// Scenario: An idle native worker waits on a future deadline with source intake paused.
/// Guarantees: The production timer wakes its blocking executor without any channel input.
#[tokio::test]
async fn native_timer_wakes_idle_worker() {
    let (mut workers, _discovery, mut reader, _owner) = setup();
    let clock = workers.cancellation();
    control_clock(&clock);
    // Freeze the clock used to compute the relative delay. Even if the native
    // worker starts late, next_until must arm a positive-duration real timer.
    let deadline = clock.now() + Duration::from_millis(1);
    let (done, result) = oneshot::channel();
    workers
        .start_read_checkpoint(move |_| {
            done.send(reader.next_until(false, Some(deadline))).unwrap();
        })
        .unwrap();
    assert_eq!(signalled(result).await, Ok(Input::Deadline));
    joined(workers.join().await, false, true);
}

/// Scenario: Discovery closes while the worker is paused and a command arrives.
/// Guarantees: Control still progresses and resuming intake reports the closed observation channel.
#[tokio::test]
async fn paused_reader_defers_closed_observation_channel() {
    let (_workers, discovery, mut reader, owner) = setup();
    drop(discovery);
    {
        let mut waiting = std::pin::pin!(reader.next_async(false, std::future::pending()));
        assert!(futures::poll!(&mut waiting).is_pending());
        owner.command(1).await.unwrap();
        assert_eq!(waiting.await, Ok(Input::Command(1)));
    }
    assert_eq!(reader.next(), Err(ChannelError::ObservationsClosed));
}

/// Scenario: A full command queue closes or shutdown interrupts a pending Ack reservation.
/// Guarantees: The sender receives the specific transport error and retains its unsent Ack.
#[tokio::test]
async fn failed_reservation_preserves_unsent_ack() {
    for expected in [ChannelError::Cancelled, ChannelError::CommandsClosed] {
        let workers = Workers::new(GUARD).unwrap();
        let (_discovery, reader, owner) = channels::<(), (), String>(
            NonZeroUsize::MIN,
            NonZeroUsize::MIN,
            workers.cancellation(),
            workers.cancellation(),
        );
        owner.command("first".into()).await.unwrap();
        let mut ack = Some(String::from("ack"));
        {
            let send = async {
                owner.reserve_command().await?.send(ack.take().unwrap());
                Ok(())
            };
            let mut send = std::pin::pin!(send);
            assert!(futures::poll!(&mut send).is_pending());
            if expected == ChannelError::Cancelled {
                _ = workers.cancellation().cancel();
            } else {
                drop(reader);
            }
            assert_eq!(send.await, Err(expected));
        }
        assert_eq!(ack.as_deref(), Some("ack"));
    }
}

/// Scenario: A full batch handoff receives a command and a deadline before capacity returns.
/// Guarantees: Control runs first, due work regains the unsent batch, and retry sends it exactly once.
#[tokio::test]
async fn stalled_publish_preserves_batch_across_control_and_deadline() {
    let (mut workers, _discovery, mut reader, mut owner) = setup();
    let clock = workers.cancellation();
    control_clock(&clock);
    let due = Some(clock.now());
    assert_eq!(reader.publish_until(1, None), Ok(Publish::Sent));
    owner.command(9).await.unwrap();
    let Publish::Command { batch, command: 9 } = reader.publish_until(2, due).unwrap() else {
        panic!("command should precede the elapsed deadline")
    };
    owner.command(10).await.unwrap();
    let Publish::Deadline(batch) = reader.publish_until(batch, due).unwrap() else {
        panic!("due work must regain the unsent batch despite the queued command")
    };
    let Publish::Command { batch, command: 10 } = reader.publish_until(batch, due).unwrap() else {
        panic!("the still-due deadline must yield to the waiting command")
    };
    let (done, result) = oneshot::channel();
    workers
        .start_read_checkpoint(move |cancel| {
            // The pipeline keeps the handoff full until the real native timer wakes us.
            let future_deadline = cancel.now() + Duration::from_millis(1);
            let Publish::Deadline(batch) =
                reader.publish_until(batch, Some(future_deadline)).unwrap()
            else {
                panic!("stalled handoff must wake for the next deadline")
            };
            done.send(()).unwrap();
            assert_eq!(reader.publish_until(batch, None), Ok(Publish::Sent));
            wait_for_cancellation(&cancel);
        })
        .unwrap();
    signalled(result).await;
    assert_eq!(owner.batch().await, Ok(1));
    assert_eq!(
        tokio::time::timeout(GUARD, owner.batch()).await.unwrap(),
        Ok(2)
    );
    assert!(owner.batches.is_empty());
    joined(workers.join().await, false, true);
}

/// Scenario: Join observes an empty slot and reads the clock at the shutdown deadline.
/// Guarantees: Publication cannot intervene between those checks and cause a false timeout.
#[tokio::test]
async fn join_checks_the_empty_slot_and_deadline_together() {
    let mut workers = Workers::new(GUARD).unwrap();
    let (release, held) = oneshot::channel();
    workers
        .start_read_checkpoint(move |_| {
            _ = block_on(held);
        })
        .unwrap();
    let clock = workers.cancellation();
    control_clock(&clock);
    let deadline = clock.cancel();
    advance_clock(&clock, GUARD);

    let worker = workers.read_checkpoint.as_mut().unwrap();
    let completion = worker.completion.clone();
    let publication_possible = std::cell::Cell::new(None);
    let outcome = worker
        .join(deadline, || {
            // Act as the competing publisher precisely when the pipeline samples time.
            publication_possible.set(Some(completion.completed.try_lock().is_some()));
            clock.now()
        })
        .await;
    release.send(()).unwrap();
    wait_for_publication(&completion).await;
    assert_eq!(outcome, WorkerOutcome::StillRunning);
    assert_eq!(publication_possible.get(), Some(false));
    assert_eq!(
        workers.join().await.read_checkpoint,
        Some(WorkerOutcome::Joined {
            deadline_missed: true
        })
    );
}

/// Scenario: A prepared batch has a free handoff slot when the worker deadline expires.
/// Guarantees: The owner receives the batch once and the same deadline is still serviced on the next wait.
#[tokio::test]
async fn free_batch_slot_precedes_due_deadline() {
    let (workers, _discovery, mut reader, mut owner) = setup();
    let clock = workers.cancellation();
    control_clock(&clock);
    let due = Some(clock.now());
    assert_eq!(reader.publish_until(7, due), Ok(Publish::Sent));
    assert_eq!(owner.batches.try_recv(), Ok(7));
    assert_eq!(
        owner.batches.try_recv(),
        Err(mpsc::error::TryRecvError::Empty)
    );
    assert_eq!(reader.next_until(false, due), Ok(Input::Deadline));
}

/// Scenario: A future deadline is supplied with commands, observations, or cancellation ready.
/// Guarantees: Commands precede observations, and cancellation leaves a queued command untouched.
#[tokio::test]
async fn ready_input_with_a_future_deadline() {
    let (workers, discovery, mut reader, owner) = setup();
    let clock = workers.cancellation();
    control_clock(&clock);
    let later = Some(clock.now() + GUARD);
    discovery.observe(1).unwrap();
    owner.command(2).await.unwrap();
    assert_eq!(reader.next_until(true, later), Ok(Input::Command(2)));
    assert_eq!(reader.next_until(true, later), Ok(Input::Observation(1)));
    owner.command(3).await.unwrap();
    _ = clock.cancel();
    assert_eq!(reader.next_until(true, later), Err(ChannelError::Cancelled));
    assert_eq!(reader.commands.try_recv(), Ok(3));
}
