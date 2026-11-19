// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio runtime construction for pipeline-local, non-Send tasks.

use std::io;
use tokio::runtime::{Builder, LocalOptions, LocalRuntime};

/// Builds a named local runtime while preserving Tokio's scheduler defaults.
pub(crate) fn build_local_runtime(name: impl Into<String>) -> io::Result<LocalRuntime> {
    Builder::new_current_thread()
        .enable_all()
        .name(name)
        .build_local(LocalOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    /// Scenario: a non-Send task is spawned before the pipeline runtime is driven.
    /// Guarantees: the task retains its Rc state and runs on its owning thread across a yield.
    #[test]
    fn local_runtime_drives_non_send_tasks_on_owner_thread() {
        let rt = build_local_runtime("non-send-test").unwrap();
        let owner = std::thread::current().id();
        let value = Rc::new(42);
        let task = rt.spawn_local(async move {
            assert_eq!(std::thread::current().id(), owner);
            tokio::task::yield_now().await;
            assert_eq!(std::thread::current().id(), owner);
            *value
        });
        assert_eq!(rt.block_on(task).unwrap(), 42);
    }

    /// Scenario: a local task waits for a Tokio timer with virtual time paused.
    /// Guarantees: the runtime enables its timer driver and completes the joined non-Send task.
    #[test]
    fn local_runtime_drives_timers() {
        let rt = build_local_runtime("timer-test").unwrap();
        rt.block_on(async {
            tokio::time::pause();
            let value = Rc::new(42);
            let task = tokio::task::spawn_local(async move {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                *value
            });
            assert_eq!(task.await.unwrap(), 42);
        });
    }

    /// Scenario: a local task has started but remains pending when its runtime is dropped.
    /// Guarantees: dropping the runtime cancels the task and releases its task-owned Rc state.
    #[test]
    fn local_runtime_drop_releases_pending_tasks() {
        let rt = build_local_runtime("drop-test").unwrap();
        let value = Rc::new(42);
        let weak = Rc::downgrade(&value);
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let _task = rt.spawn_local(async move {
            started_tx.send(()).unwrap();
            std::future::pending::<()>().await;
            drop(value);
        });
        rt.block_on(started_rx).unwrap();
        assert!(weak.upgrade().is_some());
        drop(rt);
        assert!(weak.upgrade().is_none());
    }
}
