// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Controller-owned Basic hosts. No coordinator runs on a pipeline thread.

use crate::{
    error::Error,
    thread_task::{ThreadLocalTaskHandle, spawn_thread_local_task},
};
use otel_arrow_dfe_config::{PipelineGroupId, PipelineId, pipeline::PipelineConfig};
use otel_arrow_dfe_engine::{
    PipelineFactory,
    work_coordinator::{CoordinatorBindings, CoordinatorKey, ParticipantSession},
};
use otel_arrow_dfe_telemetry::TracingSetup;
use std::{
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// All hosts attached to one logical deployment. Clones of the deployment share it.
pub(crate) struct PipelineCoordinators {
    pub(crate) bindings: Arc<CoordinatorBindings>,
    hosts: Mutex<HostState>,
    _permits: Vec<OwnedSemaphorePermit>,
}

#[derive(Default)]
struct HostState {
    threads: Vec<ThreadLocalTaskHandle<(), Error>>,
    failure: Option<String>,
}

impl fmt::Debug for PipelineCoordinators {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PipelineCoordinators")
            .finish_non_exhaustive()
    }
}

fn failure(error: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::PipelineRuntimeError {
        source: Box::new(error),
    }
}

impl PipelineCoordinators {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn start<PData: 'static + Clone + fmt::Debug>(
        factory: &PipelineFactory<PData>,
        config: &PipelineConfig,
        group: PipelineGroupId,
        pipeline: PipelineId,
        generation: u64,
        cores: impl Iterator<Item = usize>,
        budget: Arc<Semaphore>,
        tracing: TracingSetup,
        on_failure: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Result<Arc<Self>, String> {
        let declarations = factory
            .work_coordinator_declarations(config)
            .map_err(|e| e.to_string())?;
        let sessions: Vec<_> = cores
            .enumerate()
            .map(|(index, core_id)| ParticipantSession {
                core_id,
                generation,
                nonce: index as u64 + 1,
            })
            .collect();
        let mut bindings = CoordinatorBindings::default();
        let mut hosts = Vec::new();
        let mut permits = Vec::new();
        let mut readiness = Vec::new();
        // Validate the whole fleet and reserve the complete host budget before spawning.
        for (_, declaration) in &declarations {
            if sessions.len() > declaration.limits.participants {
                return Err("coordinator participant limit exceeded".into());
            }
            permits.push(
                Arc::clone(&budget)
                    .try_acquire_owned()
                    .map_err(|_| "engine work coordinator host limit exceeded")?,
            );
        }
        // The cleanup owner exists before spawning the first host. Every partial
        // preparation/startup failure stops and joins already-created hosts.
        let mut owner = Self {
            bindings: Arc::default(),
            hosts: Mutex::new(HostState::default()),
            _permits: permits,
        };
        let result: Result<(), String> = (|| {
            for (node, declaration) in declarations {
                let key = CoordinatorKey {
                    group_id: group.clone(),
                    pipeline_id: pipeline.clone(),
                    node_id: node.clone(),
                    role: declaration.role.clone(),
                };
                let role = declaration.role.clone();
                let (binding, task) = declaration
                    .prepare(key, sessions.clone(), Arc::clone(&on_failure))
                    .map_err(|e| e.to_string())?;
                bindings
                    .insert(node, role, binding.clone())
                    .map_err(|e| e.to_string())?;
                let (ready, run) = task.into_parts();
                // Shutdown uses bindings.stop(), then joins the provider cleanup.
                // Ignoring the generic cancellation token avoids dropping that cleanup.
                let host = spawn_thread_local_task(
                    "work-coordinator",
                    tracing.clone(),
                    move |_cancel| async move { run().await.map_err(failure) },
                )
                .map_err(|e| e.to_string())?;
                hosts.push(host);
                readiness.push(ready);
            }
            let deadline = Instant::now() + Duration::from_secs(30);
            for ready in readiness {
                ready
                    .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    .map_err(|e| format!("work coordinator readiness failed: {e}"))??;
            }
            Ok(())
        })();
        owner.bindings = Arc::new(bindings);
        owner
            .hosts
            .get_mut()
            .unwrap_or_else(|e| e.into_inner())
            .threads = hosts;
        result?;
        Ok(Arc::new(owner))
    }

    pub(crate) fn shutdown(&self) -> Result<(), Error> {
        self.bindings.stop();
        let mut state = self.hosts.lock().unwrap_or_else(|e| e.into_inner());
        let hosts = std::mem::take(&mut state.threads);
        for host in hosts {
            if let Err(error) = host.shutdown_and_join() {
                let _ = state.failure.get_or_insert_with(|| error.to_string());
            }
        }
        match &state.failure {
            Some(error) => Err(failure(std::io::Error::other(error.clone()))),
            None => Ok(()),
        }
    }
}

impl Drop for PipelineCoordinators {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown() {
            otel_error!(
                "work_coordinator.shutdown.failed",
                error = error.to_string()
            );
        }
    }
}
