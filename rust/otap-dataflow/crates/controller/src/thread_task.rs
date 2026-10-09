// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities to run a non-Send async task on a dedicated OS thread with a
//! single-threaded Tokio runtime and LocalSet, plus a shutdown signal.

use std::future::Future;
use std::thread;
use std::time::{Duration, Instant};
use tokio::{runtime::Builder as RtBuilder, task::LocalSet};
use tokio_util::sync::CancellationToken;

use otel_arrow_dfe_telemetry::TracingSetup;

struct ThreadCleanup<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Drop for ThreadCleanup<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.0.take() {
            cleanup();
        }
    }
}

/// Handle to a task running on a dedicated thread.  This represents
/// OS thread and single-threaded async runtime, without thread
/// pinning, dedicated to things like internal metrics aggregation or
/// internal event processing.
pub struct ThreadLocalTaskHandle<T, E> {
    cancel_token: CancellationToken,
    join_handle: Option<thread::JoinHandle<Result<T, E>>>,
    name: String,
}

impl<T, E> ThreadLocalTaskHandle<T, E> {
    /// Request a graceful shutdown by cancelling the token.
    fn shutdown(&mut self) {
        self.cancel_token.cancel();
    }

    /// Request shutdown and then join, mapping errors into controller::Error.
    pub fn shutdown_and_join(self) -> Result<T, crate::error::Error>
    where
        E: Into<crate::error::Error>,
    {
        self.shutdown_and_join_internal()
    }

    /// Request shutdown and join only until `deadline`.
    ///
    /// If the thread does not stop in time, its join handle is detached and a
    /// timeout error is returned so controller teardown remains bounded.
    pub fn shutdown_and_join_until(mut self, deadline: Instant) -> Result<T, crate::error::Error>
    where
        E: Into<crate::error::Error>,
    {
        self.shutdown();
        let join_handle = self.join_handle.take().expect("join handle missing");
        while !join_handle.is_finished() {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return Err(crate::error::Error::ThreadJoinTimeout {
                    thread_name: self.name.clone(),
                });
            };
            thread::sleep(remaining.min(Duration::from_millis(10)));
        }
        match join_handle.join() {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(error.into()),
            Err(panic) => Err(crate::error::Error::ThreadJoinPanic {
                thread_name: self.name.clone(),
                panic_message: format!("{panic:?}"),
            }),
        }
    }

    fn shutdown_and_join_internal(mut self) -> Result<T, crate::error::Error>
    where
        E: Into<crate::error::Error>,
    {
        self.shutdown();
        match self.join_handle.take().expect("join handle missing").join() {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(e.into()),
            Err(panic) => Err(crate::error::Error::ThreadJoinPanic {
                thread_name: self.name.clone(),
                panic_message: format!("{panic:?}"),
            }),
        }
    }
}

impl<T, E> Drop for ThreadLocalTaskHandle<T, E> {
    #[allow(clippy::print_stderr)]
    fn drop(&mut self) {
        // Best-effort, idempotent shutdown on drop.
        self.cancel_token.cancel();

        // If we still own the join handle, attempt to join the thread.
        if let Some(handle) = self.join_handle.take() {
            match handle.join() {
                Ok(Ok(_)) => {
                    // Task completed successfully; nothing to report.
                }
                Ok(Err(_)) => {
                    // Task returned an error; can't propagate it from Drop, so just log.
                    // ToDo Replace this eprintln once we have selected a logging solution
                    eprintln!(
                        "Thread '{}' finished with an error during drop; error suppressed",
                        self.name
                    );
                }
                Err(panic) => {
                    // Don't panic in Drop; report and suppress.
                    // ToDo Replace this eprintln once we have selected a logging solution
                    eprintln!(
                        "Thread '{}' panicked during drop: {panic:?}; panic suppressed",
                        self.name
                    );
                }
            }
        }
    }
}

/// Spawn a non-Send async task on a dedicated OS thread running a single-threaded
/// Tokio runtime with a LocalSet. Returns a handle to signal shutdown and join.
///
/// Note creates an OS thread and a single-threaded async runtime,
/// without thread pinning, dedicated to things like internal metrics
/// aggregation or internal event processing.
///
/// The `task_factory` receives a CancellationToken that is cancelled when shutdown is requested
/// and must return the async task to run. The task's `Output` is surfaced by `shutdown_and_join()`.
///
/// The tracing subscriber is set as the thread-local default before the task
/// runs, and remains active for the duration of the thread.
///
/// Contract:
/// - The task should observe the cancellation token and exit promptly when it is cancelled.
/// - `T` and `E` must be `Send + 'static` to cross the thread boundary.
pub fn spawn_thread_local_task<T, E, Fut, F>(
    thread_name: impl Into<String>,
    tracing_setup: TracingSetup,
    task_factory: F,
) -> Result<ThreadLocalTaskHandle<T, E>, crate::error::Error>
where
    T: Send + 'static,
    E: Send + 'static,
    Fut: 'static + Future<Output = Result<T, E>>,
    F: 'static + Send + FnOnce(CancellationToken) -> Fut,
{
    spawn_thread_local_task_with_cleanup(thread_name, tracing_setup, task_factory, |_| {})
}

/// Runs cleanup on the same OS thread after its LocalSet and runtime are dropped,
/// including on unwind. Cleanup may wait for other threads without blocking any
/// local async work. Its captured resources survive a timed-out, detached join.
/// The callback receives the same cancellation signal as the task.
pub(crate) fn spawn_thread_local_task_with_cleanup<T, E, Fut, F, C>(
    thread_name: impl Into<String>,
    tracing_setup: TracingSetup,
    task_factory: F,
    cleanup: C,
) -> Result<ThreadLocalTaskHandle<T, E>, crate::error::Error>
where
    T: Send + 'static,
    E: Send + 'static,
    Fut: 'static + Future<Output = Result<T, E>>,
    F: 'static + Send + FnOnce(CancellationToken) -> Fut,
    // The callback, like the factory, moves once onto its dedicated OS thread.
    C: 'static + Send + FnOnce(CancellationToken),
{
    let name = thread_name.into();
    let name_for_thread = name.clone();
    let token = CancellationToken::new();
    let token_for_task = token.clone();
    let token_for_cleanup = token.clone();

    let join_handle = thread::Builder::new()
        .name(name_for_thread)
        .spawn(move || {
            tracing_setup.with_subscriber(|| {
                let _cleanup = ThreadCleanup(Some(move || cleanup(token_for_cleanup)));
                let rt = RtBuilder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create runtime");
                let local = LocalSet::new();

                // Build the task future using the provided factory, passing the cancellation token.
                let fut = task_factory(token_for_task);
                // Run the future to completion on the LocalSet and return its result to the caller.
                rt.block_on(local.run_until(fut))
            })
        })
        .map_err(|e| crate::error::Error::ThreadSpawnError {
            thread_name: name.clone(),
            source: e,
        })?;

    Ok(ThreadLocalTaskHandle {
        cancel_token: token,
        join_handle: Some(join_handle),
        name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use otel_arrow_dfe_config::settings::telemetry::logs::LogLevel;
    use otel_arrow_dfe_telemetry::tracing_init::ProviderSetup;

    struct LocalTaskDrop(std::sync::mpsc::Sender<&'static str>);

    impl Drop for LocalTaskDrop {
        fn drop(&mut self) {
            _ = self.0.send("local task dropped");
        }
    }

    /// Scenario: a scope thread panics with an unfinished task on its LocalSet.
    /// Guarantees: shutdown cleanup runs after local task destruction, while the join still reports the panic.
    #[test]
    fn shutdown_cleanup_runs_after_local_tasks_drop_on_panic() {
        let (events_tx, events_rx) = std::sync::mpsc::channel();
        let cleanup_tx = events_tx.clone();
        let handle: ThreadLocalTaskHandle<(), Error> = spawn_thread_local_task_with_cleanup(
            "test-thread-shutdown-cleanup",
            TracingSetup::new(
                ProviderSetup::Noop,
                LogLevel::default(),
                crate::engine_context,
            ),
            move |_| async move {
                let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
                let _task = tokio::task::spawn_local(async move {
                    let _drop = LocalTaskDrop(events_tx);
                    _ = ready_tx.send(());
                    std::future::pending::<()>().await;
                });
                ready_rx.await.expect("local task should start");
                panic!("synthetic scope thread panic");
            },
            move |_| {
                _ = cleanup_tx.send("shutdown cleanup");
            },
        )
        .expect("test thread should start");
        assert!(matches!(
            handle.shutdown_and_join(),
            Err(Error::ThreadJoinPanic { .. })
        ));
        assert_eq!(
            events_rx.try_iter().collect::<Vec<_>>(),
            ["local task dropped", "shutdown cleanup"]
        );
    }
}
