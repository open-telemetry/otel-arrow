// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::time::{Duration, Instant};

struct DropProbe(std::sync::mpsc::Sender<std::thread::ThreadId>);

impl Drop for DropProbe {
    fn drop(&mut self) {
        let _ = self.0.send(std::thread::current().id());
    }
}

/// Scenario: A successful worker is explicitly stopped or dropped between jobs.
/// Guarantees: State is destroyed off-core, and awaited shutdown acknowledges only completed cleanup.
#[tokio::test]
async fn successful_job_and_cleanup_stay_on_worker() {
    for await_cleanup in [true, false] {
        let (dropped, observed) = std::sync::mpsc::channel();
        let mut worker =
            NativeWorker::<Option<DropProbe>>::new("oracle-cleanup-test").expect("worker");
        let operation = worker
            .run(move |state| {
                *state = Some(DropProbe(dropped));
                std::thread::current().id()
            })
            .expect("accepted");
        let thread = receive(operation).await.expect("successful operation");
        assert_ne!(thread, std::thread::current().id());
        assert!(observed.try_recv().is_err());
        let dropped_on = if await_cleanup {
            worker.stop().await.expect("confirmed cleanup");
            observed.try_recv().expect("cleanup before acknowledgement")
        } else {
            drop(worker);
            observed
                .recv_timeout(Duration::from_secs(2))
                .expect("cleanup")
        };
        assert_eq!(dropped_on, thread);
    }
}

/// Scenario: One worker job is active and another occupies the only queue slot.
/// Guarantees: Further submissions fail immediately and accepted jobs finish after release.
#[tokio::test]
async fn worker_queue_is_bounded() {
    let mut worker = NativeWorker::<()>::new("oracle-capacity-test").expect("worker");
    let (release, gate) = sync_channel(1);
    let (started, ready) = oneshot::channel();
    let first = worker
        .run(move |_| {
            let _ = started.send(());
            gate.recv().expect("released");
        })
        .expect("first job");
    receive(ready).await.expect("worker started");
    let second = worker.run(|_| ()).expect("queued job");
    assert_eq!(
        worker.run(|_| ()).expect_err("queue must be full").kind(),
        io::ErrorKind::WouldBlock
    );
    release.send(()).expect("release worker");
    receive(first).await.expect("first result");
    receive(second).await.expect("second result");
    worker.stop().await.expect("cleanup");
}

/// Scenario: Native worker code panics before replying.
/// Guarantees: Neither the reply nor cleanup channel can report false success.
#[tokio::test]
async fn worker_panic_does_not_confirm_cleanup() {
    let mut worker = NativeWorker::<()>::new("oracle-panic-test").expect("worker");
    let result = worker
        .run::<()>(|_| panic!("synthetic worker failure"))
        .expect("accepted");
    assert_eq!(
        receive(result).await.expect_err("no reply").kind(),
        io::ErrorKind::BrokenPipe
    );
    assert_eq!(
        worker
            .stop()
            .await
            .expect_err("no cleanup confirmation")
            .kind(),
        io::ErrorKind::BrokenPipe
    );
}

struct StuckDrop;

impl Drop for StuckDrop {
    fn drop(&mut self) {
        loop {
            std::thread::park();
        }
    }
}

/// Scenario: A real worker operation or its resource destructor never returns.
/// Guarantees: An async deadline and ordinary Tokio runtime destruction finish without joining that thread.
#[test]
fn stuck_work_does_not_hold_tokio_runtime() {
    const CHILD: &str = "OTEL_ORACLE_WORKER_HANG_CASE";
    if let Ok(mode) = std::env::var(CHILD) {
        assert!(matches!(mode.as_str(), "operation" | "cleanup"));
        let stuck_operation = mode == "operation";
        let mut worker =
            NativeWorker::<Option<StuckDrop>>::new("oracle-stuck-test").expect("worker");
        let (started, ready) = oneshot::channel();
        let _operation = worker
            .run(move |state| {
                *state = (!stuck_operation).then(|| StuckDrop);
                let _ = started.send(());
                if stuck_operation {
                    loop {
                        std::thread::park();
                    }
                }
            })
            .expect("job");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let began = Instant::now();
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(1), receive(ready))
                .await
                .expect("worker started promptly")
                .expect("start notification");
            assert!(
                tokio::time::timeout(Duration::from_millis(30), worker.stop())
                    .await
                    .is_err()
            );
        });
        drop(runtime);
        assert!(began.elapsed() < Duration::from_secs(2));
        return;
    }

    for mode in ["operation", "cleanup"] {
        let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "receivers::oracle_receiver::worker::tests::stuck_work_does_not_hold_tokio_runtime",
                "--nocapture",
            ])
            .env(CHILD, mode)
            .spawn()
            .expect("subprocess");
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait().expect("child status") {
                assert!(status.success(), "{mode} subprocess failed");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().expect("kill hung regression subprocess");
                let _ = child.wait();
                panic!("runtime destruction waited for the {mode} worker");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
