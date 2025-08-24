// SPDX-License-Identifier: Apache-2.0

//! Utilities to run a non-Send async task on a dedicated OS thread with a
//! single-threaded Tokio runtime and LocalSet, plus a shutdown signal.

use std::future::Future;
use std::thread;
use tokio::{runtime::Builder as RtBuilder, task::LocalSet};
use tokio_util::sync::CancellationToken;

/// Handle to a task running on a dedicated thread.
///
/// - `shutdown()` requests cancellation via the token (idempotent, best-effort).
/// - `shutdown_and_join()` requests shutdown and then waits for completion, returning controller::Error on failure.
/// - `join_raw()` waits for the thread to finish and returns the raw nested result.
pub struct ThreadLocalTaskHandle<T, E> {
    cancel_token: CancellationToken,
    join_handle: Option<thread::JoinHandle<Result<T, E>>>,
    name: String,
}

impl<T, E> ThreadLocalTaskHandle<T, E> {
    /// Request a graceful shutdown by cancelling the token.
    pub fn shutdown(&mut self) {
        self.cancel_token.cancel();
    }

    /// Request shutdown and then join, mapping errors into controller::Error.
    pub fn shutdown_and_join(self) -> Result<T, crate::error::Error>
    where
        E: Into<crate::error::Error>,
    {
        self.shutdown_and_join_internal()
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
                thread_name: self.name,
                panic_message: format!("{panic:?}"),
            }),
        }
    }
}

/// Spawn a non-Send async task on a dedicated OS thread running a single-threaded
/// Tokio runtime with a LocalSet. Returns a handle to signal shutdown and join.
///
/// The `task_factory` receives a CancellationToken that is cancelled when shutdown is requested
/// and must return the async task to run. The task's `Output` is surfaced by `shutdown_and_join()`.
///
/// Contract:
/// - The task should observe the cancellation token and exit promptly when it is cancelled.
/// - `T` and `E` must be `Send + 'static` to cross the thread boundary.
pub fn spawn_thread_local_task<T, E, Fut, F>(
    thread_name: impl Into<String>,
    task_factory: F,
) -> Result<ThreadLocalTaskHandle<T, E>, crate::error::Error>
where
    T: Send + 'static,
    E: Send + 'static,
    Fut: 'static + Future<Output = Result<T, E>>,
    F: 'static + Send + FnOnce(CancellationToken) -> Fut,
{
    let name = thread_name.into();
    let name_for_thread = name.clone();
    let token = CancellationToken::new();
    let token_for_task = token.clone();

    let join_handle = thread::Builder::new()
        .name(name_for_thread)
        .spawn(move || {
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
