// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine-scoped ownership of coordinator threads, including failed cleanup.
//!
//! Native provider joins run on at most one cleanup worker per reserved host.
//! Their completion includes runtime and thread-local destruction. Cleanup
//! workers install no subscriber/runtime and execute no provider code themselves.

use super::{Error, failure};
use crate::thread_task::{ThreadLocalTaskHandle, spawn_thread_local_task};
use otel_arrow_dfe_engine::work_coordinator::{CoordinatorBinding, CoordinatorKey};
use otel_arrow_dfe_telemetry::TracingSetup;
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Shared shutdown deadline, not a separate allowance for each host.
pub(crate) const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Keeps unfinished hosts reserved after deployment removal or startup failure.
pub(crate) struct HostSupervisor {
    budget: Arc<Semaphore>,
    hosts: Mutex<HashMap<CoordinatorKey, Arc<Host>>>,
}

impl HostSupervisor {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            budget: Arc::new(Semaphore::new(limit)),
            hosts: Mutex::default(),
        }
    }

    pub(super) fn reserve(&self, keys: Vec<CoordinatorKey>) -> Result<Vec<Arc<Host>>, String> {
        let mut hosts = self.hosts.lock().unwrap_or_else(|e| e.into_inner());
        hosts.retain(|_, host| !host.collect());
        let mut unique = HashSet::new();
        if keys
            .iter()
            .any(|key| !unique.insert(key) || hosts.contains_key(key))
        {
            return Err(
                "work coordinator identity is already reserved; cleanup may be pending".into(),
            );
        }
        let permits = keys
            .iter()
            .map(|_| {
                Arc::clone(&self.budget)
                    .try_acquire_owned()
                    .map_err(|_| "engine work coordinator host limit exceeded".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let reserved: Vec<_> = keys
            .into_iter()
            .zip(permits)
            .map(|(key, permit)| {
                Arc::new(Host {
                    key,
                    state: Mutex::new(State {
                        permit: Some(permit),
                        ..State::default()
                    }),
                    changed: Condvar::new(),
                })
            })
            .collect();
        for host in &reserved {
            let _ = hosts.insert(host.key.clone(), Arc::clone(host));
        }
        Ok(reserved)
    }
}

impl Drop for HostSupervisor {
    fn drop(&mut self) {
        // A cleanup worker owns an Arc<Host> until the provider's actual join
        // finishes. It retains the binding and permit even if the engine exits
        // first. Dropping its observer handle is not treated as cleanup.
        for (_, host) in self
            .hosts
            .get_mut()
            .unwrap_or_else(|e| e.into_inner())
            .drain()
        {
            if !host.collect()
                && let Err(error) = host.begin_shutdown()
            {
                otel_error!("work_coordinator.cleanup_worker.failed", error = error);
                // OS thread exhaustion leaves no safe execution context for a
                // potentially blocking join. Retain ownership until process exit.
                // At most the engine's reserved host budget can reach this path.
                std::mem::forget(host);
            }
        }
    }
}

#[derive(Default)]
struct State {
    binding: Option<CoordinatorBinding>,
    task: Option<ThreadLocalTaskHandle<(), Error>>,
    joiner: Option<JoinHandle<()>>,
    failure: Option<String>,
    cleanup_error: Option<String>,
    finished: bool,
    permit: Option<OwnedSemaphorePermit>,
}

pub(super) struct Host {
    key: CoordinatorKey,
    state: Mutex<State>,
    changed: Condvar,
}

impl Host {
    pub(super) fn start<Fut, F>(
        &self,
        binding: CoordinatorBinding,
        tracing: TracingSetup,
        run: F,
    ) -> Result<(), String>
    where
        Fut: 'static + Future<Output = Result<(), Error>>,
        F: Send + 'static + FnOnce() -> Fut,
    {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.binding = Some(binding);
        state.task = Some(
            spawn_thread_local_task("work-coordinator", tracing, move |_cancel| run())
                .map_err(|e| e.to_string())?,
        );
        Ok(())
    }

    pub(super) fn begin_shutdown(self: &Arc<Self>) -> Result<(), String> {
        let binding = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .binding
            .clone();
        if let Some(binding) = binding {
            binding.stop();
        }
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.finished || state.joiner.is_some() {
            return Ok(());
        }
        if state.task.is_none() {
            // A reserved host that never spawned during partial startup.
            state.finished = true;
            let _ = state.permit.take();
            return Ok(());
        }
        let host = Arc::clone(self);
        match thread::Builder::new()
            .name("work-coordinator-cleanup".into())
            .spawn(move || {
                let task = host
                    .state
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .task
                    .take()
                    .expect("one cleanup worker owns the provider");
                let result = task.shutdown_and_join().map_err(|error| {
                    format!(
                        "work coordinator {}/{}/{}/{}: {error}",
                        host.key.group_id, host.key.pipeline_id, host.key.node_id, host.key.role,
                    )
                });
                let mut state = host.state.lock().unwrap_or_else(|e| e.into_inner());
                if let Err(error) = &result {
                    let _ = state.failure.get_or_insert_with(|| error.clone());
                    state.cleanup_error = Some(error.clone());
                }
                host.changed.notify_all();
            }) {
            Ok(joiner) => state.joiner = Some(joiner),
            Err(error) => {
                let error = format!("cannot supervise work coordinator cleanup: {error}");
                let _ = state.failure.get_or_insert_with(|| error.clone());
                return Err(error);
            }
        }
        Ok(())
    }

    /// Reap only the internal cleanup worker, after its potentially blocking
    /// provider join and all provider TLS destruction have already completed.
    fn collect(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.joiner.as_ref().is_some_and(JoinHandle::is_finished) {
            if state
                .joiner
                .take()
                .expect("finished cleanup worker")
                .join()
                .is_err()
            {
                state.cleanup_error = Some("coordinator cleanup worker panicked".into());
                let _ = state
                    .failure
                    .get_or_insert_with(|| "coordinator cleanup worker panicked".into());
            }
            state.finished = true;
            let _ = state.permit.take();
        }
        state.finished
    }

    pub(super) fn wait_until(&self, deadline: Instant) -> Result<(), Error> {
        loop {
            let _ = self.collect();
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.finished {
                return state.failure.as_ref().map_or(Ok(()), |error| {
                    let error = state
                        .cleanup_error
                        .as_ref()
                        .filter(|cleanup| *cleanup != error)
                        .map_or_else(
                            || error.clone(),
                            |cleanup| format!("{error}; cleanup also failed: {cleanup}"),
                        );
                    Err(failure(std::io::Error::other(error)))
                });
            }
            if state.joiner.is_none()
                && let Some(error) = &state.failure
            {
                return Err(failure(std::io::Error::other(error.clone())));
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                let error = state.failure.get_or_insert_with(|| format!(
                    "work coordinator {}/{}/{}/{} cleanup timed out; ownership remains reserved",
                    self.key.group_id, self.key.pipeline_id, self.key.node_id, self.key.role,
                )).clone();
                return Err(failure(std::io::Error::other(error)));
            }
            // JoinHandle::is_finished is native-thread state, not Tokio time.
            // The completion signal may arrive just before the cleanup worker
            // itself exits; bounded polling closes that final observation gap.
            let _ = self
                .changed
                .wait_timeout(state, remaining.min(Duration::from_millis(5)))
                .unwrap_or_else(|e| e.into_inner());
        }
    }
}

#[cfg(test)]
mod tests;
