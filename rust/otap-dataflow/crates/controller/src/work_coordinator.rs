// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Controller-owned Basic hosts. No coordinator runs on a pipeline thread.

use crate::error::Error;
use otel_arrow_dfe_config::{PipelineGroupId, PipelineId, pipeline::PipelineConfig};
use otel_arrow_dfe_engine::{
    PipelineFactory,
    work_coordinator::{
        CapabilityProfile, CoordinatorBinding, CoordinatorBindings, CoordinatorDescriptor,
        CoordinatorKey, OperationId, ParticipantSession, ParticipantSpec, ReconciliationOutcome,
        ReconciliationReceipt,
    },
};
use otel_arrow_dfe_telemetry::TracingSetup;
use std::{
    collections::BTreeMap,
    fmt,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
mod supervision;
use supervision::Host;
pub(crate) use supervision::{HostSupervisor, SHUTDOWN_TIMEOUT};

/// All hosts attached to one logical deployment. Clones of the deployment share it.
pub(crate) struct PipelineCoordinators {
    pub(crate) bindings: Arc<CoordinatorBindings>,
    hosts: Mutex<HostState>,
    _supervisor: Arc<HostSupervisor>,
    descriptors: Mutex<BTreeMap<BindingKey, CoordinatorDescriptor>>,
    pending: Mutex<Option<Arc<PendingReconciliation>>>,
}

type BindingKey = (otel_arrow_dfe_config::NodeId, String);
pub(crate) type CoordinatorRevisions = BTreeMap<BindingKey, u64>;
struct PendingReconciliation {
    operations: Mutex<Vec<(BindingKey, CoordinatorBinding, ReconciliationReceipt)>>,
    descriptors: BTreeMap<BindingKey, CoordinatorDescriptor>,
    decided: AtomicBool,
    final_receipts: Mutex<Vec<ReconciliationReceipt>>,
}

fn descriptors<PData: 'static + Clone + fmt::Debug>(
    factory: &PipelineFactory<PData>,
    config: &PipelineConfig,
) -> Result<BTreeMap<BindingKey, CoordinatorDescriptor>, String> {
    factory
        .work_coordinator_declarations(config)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(node, declaration)| {
            let descriptor = declaration.descriptor().map_err(|e| e.to_string())?;
            Ok(((node, declaration.role), descriptor))
        })
        .collect()
}

pub(crate) fn validate_update<PData: 'static + Clone + fmt::Debug>(
    factory: &PipelineFactory<PData>,
    current: &PipelineConfig,
    target: &PipelineConfig,
) -> Result<bool, String> {
    let old = descriptors(factory, current)?;
    let next = descriptors(factory, target)?;
    if old.is_empty() && next.is_empty() {
        return Ok(false);
    }
    if old.keys().ne(next.keys()) {
        return Err("coordinator identities changed; restart required".into());
    }
    for (key, descriptor) in &old {
        let old_node = &current.nodes()[&key.0];
        let new_node = &target.nodes()[&key.0];
        if old_node.kind() != new_node.kind() || old_node.r#type != new_node.r#type {
            return Err("coordinator node provider changed; restart required".into());
        }
        descriptor
            .validate_update(&next[key])
            .map_err(|e| e.to_string())?;
    }
    Ok(true)
}

#[derive(Default)]
struct HostState {
    threads: Vec<Arc<Host>>,
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
        budget: Arc<HostSupervisor>,
        tracing: TracingSetup,
        on_failure: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Result<Arc<Self>, String> {
        let declarations = factory
            .work_coordinator_declarations(config)
            .map_err(|e| e.to_string())?;
        let metadata = descriptors(factory, config)?;
        let sessions: Vec<_> = cores
            .enumerate()
            .map(|(index, core_id)| ParticipantSession {
                core_id,
                generation,
                nonce: index as u64 + 1,
            })
            .collect();
        let mut bindings = CoordinatorBindings::default();
        let mut readiness = Vec::new();
        // Validate the whole fleet and reserve the complete host budget before spawning.
        for (_, declaration) in &declarations {
            if sessions.len() > declaration.limits.participants {
                return Err("coordinator participant limit exceeded".into());
            }
        }
        let hosts = budget.reserve(
            declarations
                .iter()
                .map(|(node, declaration)| CoordinatorKey {
                    group_id: group.clone(),
                    pipeline_id: pipeline.clone(),
                    node_id: node.clone(),
                    role: declaration.role.clone(),
                })
                .collect(),
        )?;
        // The cleanup owner exists before spawning the first host. Every partial
        // preparation/startup failure stops and joins already-created hosts.
        let mut owner = Self {
            bindings: Arc::default(),
            hosts: Mutex::new(HostState {
                threads: hosts.clone(),
                failure: None,
            }),
            _supervisor: budget,
            descriptors: Mutex::new(metadata),
            pending: Mutex::new(None),
        };
        let result: Result<(), String> = (|| {
            for ((node, declaration), host) in declarations.into_iter().zip(hosts) {
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
                host.start(binding, tracing.clone(), move || async move {
                    run().await.map_err(failure)
                })?;
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
        result?;
        Ok(Arc::new(owner))
    }

    pub(crate) fn is_cooperative(&self) -> bool {
        let descriptors = self.descriptors.lock().unwrap_or_else(|e| e.into_inner());
        !descriptors.is_empty()
            && descriptors
                .values()
                .all(|descriptor| descriptor.profile == CapabilityProfile::Cooperative)
    }

    pub(crate) fn prepare<PData: 'static + Clone + fmt::Debug>(
        &self,
        factory: &PipelineFactory<PData>,
        target: &PipelineConfig,
        deadline: Instant,
    ) -> Result<CoordinatorRevisions, String> {
        let next = descriptors(factory, target)?;
        let old = self
            .descriptors
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if old.keys().ne(next.keys()) {
            return Err("coordinator identities changed; restart required".into());
        }
        for (key, descriptor) in &old {
            descriptor
                .validate_update(&next[key])
                .map_err(|e| e.to_string())?;
        }
        if self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some()
        {
            return Err("coordinator finalization or abort is still pending".into());
        }
        let pending = Arc::new(PendingReconciliation {
            operations: Mutex::new(Vec::new()),
            descriptors: next.clone(),
            decided: AtomicBool::new(false),
            final_receipts: Mutex::new(Vec::new()),
        });
        *self.pending.lock().unwrap_or_else(|e| e.into_inner()) = Some(Arc::clone(&pending));
        for (key, binding) in self.bindings.iter() {
            let receipt = binding
                .prepare_reconciliation(Arc::clone(&next[key].settings))
                .map_err(|e| e.to_string())?;
            pending
                .operations
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push((key.clone(), binding.clone(), receipt));
        }
        let operations = pending
            .operations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let mut revisions = CoordinatorRevisions::new();
        for (key, _, receipt) in operations {
            let result = receipt.wait_until(deadline).map_err(|e| e.to_string())?;
            let ReconciliationOutcome::Prepared(revision) = result else {
                return Err("unexpected coordinator preparation receipt".into());
            };
            let _ = revisions.insert(key, revision);
        }
        Ok(revisions)
    }

    pub(crate) fn ensure_instance(
        &self,
        spec: ParticipantSpec,
        revisions: Option<&CoordinatorRevisions>,
        deadline: Instant,
    ) -> Result<(), String> {
        for (key, binding) in self.bindings.iter() {
            if binding.profile() != CapabilityProfile::Cooperative
                || binding.has_pending_instance(spec)
            {
                continue;
            }
            let revision = revisions
                .and_then(|revisions| revisions.get(key).copied())
                .unwrap_or(0);
            let _ = binding
                .register_instance(spec, revision)
                .map_err(|e| e.to_string())?
                .wait_until(deadline)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub(crate) fn activate(&self, spec: ParticipantSpec, deadline: Instant) -> Result<(), String> {
        let mut receipts = Vec::new();
        for (_, binding) in self.bindings.iter() {
            if binding.profile() == CapabilityProfile::Cooperative {
                receipts.push(binding.activate_instance(spec).map_err(|e| e.to_string())?);
            }
        }
        for receipt in receipts {
            let _ = receipt.wait_until(deadline).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub(crate) fn retire(&self, spec: ParticipantSpec, deadline: Instant) -> Result<(), String> {
        let mut receipts = Vec::new();
        for (_, binding) in self.bindings.iter() {
            if binding.profile() == CapabilityProfile::Cooperative {
                receipts.push(binding.retire_instance(spec).map_err(|e| e.to_string())?);
            }
        }
        for receipt in receipts {
            let _ = receipt.wait_until(deadline).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub(crate) fn confirm_exit(&self, spec: ParticipantSpec) -> bool {
        self.is_cooperative()
            && self
                .bindings
                .iter()
                .all(|(_, binding)| binding.confirm_instance_exit(spec))
    }

    pub(crate) fn mark_decision(&self) {
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if let Some(pending) = pending {
            pending.decided.store(true, Ordering::Release);
            *self.descriptors.lock().unwrap_or_else(|e| e.into_inner()) =
                pending.descriptors.clone();
        }
    }

    pub(crate) fn finish(&self, commit: bool, deadline: Instant) -> Result<(), String> {
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let Some(pending) = pending else {
            return Ok(());
        };
        if !commit && pending.decided.load(Ordering::Acquire) {
            return Err("controller already committed the coordinator revision".into());
        }
        let operations = pending
            .operations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let cached = pending
            .final_receipts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let receipts = if commit && !cached.is_empty() {
            cached
        } else {
            let mut receipts = Vec::new();
            for (_, binding, prepared) in operations {
                let operation: OperationId = prepared.operation();
                receipts.push(
                    if commit {
                        binding.commit(operation)
                    } else {
                        binding.abort(operation)
                    }
                    .map_err(|e| e.to_string())?,
                );
            }
            if commit {
                *pending
                    .final_receipts
                    .lock()
                    .unwrap_or_else(|e| e.into_inner()) = receipts.clone();
            }
            receipts
        };
        for receipt in receipts {
            let _ = receipt.wait_until(deadline).map_err(|e| e.to_string())?;
        }
        *self.pending.lock().unwrap_or_else(|e| e.into_inner()) = None;
        Ok(())
    }

    pub(crate) fn committed(&self) -> bool {
        self.pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .is_some_and(|pending| pending.decided.load(Ordering::Acquire))
    }

    pub(crate) fn shutdown(&self) -> Result<(), Error> {
        self.shutdown_until(Instant::now() + SHUTDOWN_TIMEOUT)
    }

    pub(crate) fn shutdown_until(&self, deadline: Instant) -> Result<(), Error> {
        self.bindings.stop();
        let hosts = self
            .hosts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .threads
            .clone();
        for host in &hosts {
            let _ = host.begin_shutdown();
        }
        let mut first = None;
        for host in &hosts {
            if let Err(error) = host.wait_until(deadline) {
                let _ = first.get_or_insert(error);
            }
        }
        let mut state = self.hosts.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(error) = first {
            // Host failures are sticky; refresh the summary to include a
            // provider error learned after an earlier cleanup timeout.
            state.failure = Some(error.to_string());
        }
        match &state.failure {
            Some(error) => Err(failure(std::io::Error::other(error.clone()))),
            None => Ok(()),
        }
    }
}

impl Drop for PipelineCoordinators {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown_until(Instant::now()) {
            otel_error!(
                "work_coordinator.shutdown.failed",
                error = error.to_string()
            );
        }
    }
}
