// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP Dataflow Engine Controller
//!
//! This controller is responsible for deploying, managing, and monitoring pipeline groups
//! within the current process.
//!
//! Each pipeline configuration declares its CPU requirements through
//! `policies.resources.core_allocation`.
//! Based on this policy, the controller allocates CPU cores and spawns one dedicated
//! thread per assigned core. Threads are pinned to distinct CPU cores, following a
//! strict thread-per-core model.
//!
//! A pipeline deployed on `n` cores results in `n` worker threads. Hot data paths are
//! fully contained within each thread to maximize CPU cache locality and minimize
//! cross-thread contention. Inter-thread communication is restricted to control
//! messages and internal telemetry only.
//!
//! By default, pipelines are expected to run on dedicated CPU cores. It is possible
//! to deploy multiple pipeline configurations on the same cores, primarily for
//! consolidation, testing, or transitional deployments. This comes at the cost of
//! reduced efficiency, especially cache locality. Even in this mode, pipeline
//! instances run in independent threads and do not share mutable data structures.
//!
//! Pipelines do not perform implicit work stealing, dynamic scheduling, or automatic
//! load balancing across threads. Any form of cross-pipeline or cross-thread data
//! exchange must be explicitly modeled.
//!
//! In the future, controller-managed named channels will be introduced as the
//! recommended mechanism to implement explicit load balancing and routing schemes
//! within the engine. These channels will complement the existing SO_REUSEPORT-based
//! load balancing mechanism already supported at the receiver level on operating
//! systems that provide it.
//!
//! Pipelines can be gracefully shut down by sending control messages through their
//! control channels.
//!
//! Future work includes:
//! - TODO: Complete status and health checks for pipelines
//! - TODO: Auto-restart threads in case of panic
//! - TODO: Live pipeline updates
//! - TODO: Better resource control

use crate::error::Error;
use crate::thread_task::spawn_thread_local_task;
use core_affinity::CoreId;
use otap_df_config::engine::{
    OtelDataflowSpec, ResolvedPipelineConfig, ResolvedPipelineRole,
    SYSTEM_OBSERVABILITY_PIPELINE_ID, SYSTEM_PIPELINE_GROUP_ID,
};
use otap_df_config::policy::{ChannelCapacityPolicy, CoreAllocation, TelemetryPolicy};
use otap_df_config::{DeployedPipelineKey, PipelineKey, pipeline::PipelineConfig};
use otap_df_engine::PipelineFactory;
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_engine::control::{
    PipelineCtrlMsgReceiver, PipelineCtrlMsgSender, pipeline_ctrl_msg_channel,
};
use otap_df_engine::entity_context::{
    node_entity_key, pipeline_entity_key, set_pipeline_entity_key,
};
use otap_df_engine::error::{Error as EngineError, error_summary_from};
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::event::{EngineEvent, ErrorSummary, ObservedEventReporter};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry::{
    InternalTelemetrySettings, InternalTelemetrySystem, TracingSetup, otel_info, otel_info_span,
    otel_warn, self_tracing::LogContext,
};
use smallvec::smallvec;
use std::sync::Arc;
use std::sync::mpsc as std_mpsc;
use std::thread;

/// Error types and helpers for the controller module.
pub mod error;
/// Utilities to spawn async tasks on dedicated threads with graceful shutdown.
pub mod thread_task;

/// Controller for managing pipelines in a thread-per-core model.
///
/// # Thread Safety
/// This struct is designed to be used in multi-threaded contexts. Each pipeline is run on a
/// dedicated thread pinned to a CPU core.
/// Intended for use as a long-lived process controller.
pub struct Controller<PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    /// The pipeline factory used to build runtime pipelines.
    pipeline_factory: &'static PipelineFactory<PData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    ParkMainThread,
    ShutdownWhenDone,
}

/// Returns the set of entity keys relevant to this context.
fn engine_context() -> LogContext {
    if let Some(node) = node_entity_key() {
        smallvec![node]
    } else if let Some(pipeline) = pipeline_entity_key() {
        smallvec![pipeline]
    } else {
        smallvec![]
    }
}

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug> Controller<PData> {
    /// Creates a new controller with the given pipeline factory.
    pub const fn new(pipeline_factory: &'static PipelineFactory<PData>) -> Self {
        Self { pipeline_factory }
    }

    /// Starts the controller with the given engine configurations.
    pub fn run_forever(&self, engine_config: OtelDataflowSpec) -> Result<(), Error> {
        self.run_with_mode(engine_config, RunMode::ParkMainThread)
    }

    /// Starts the controller with the given engine configurations.
    ///
    /// Runs until pipelines are shut down, then closes telemetry/admin services.
    pub fn run_till_shutdown(&self, engine_config: OtelDataflowSpec) -> Result<(), Error> {
        self.run_with_mode(engine_config, RunMode::ShutdownWhenDone)
    }

    fn run_with_mode(
        &self,
        engine_config: OtelDataflowSpec,
        run_mode: RunMode,
    ) -> Result<(), Error> {
        let num_pipeline_groups = engine_config.groups.len();
        let resolved_config = engine_config.resolve();
        let (engine, pipelines, observability_pipeline) = resolved_config.into_parts();
        let num_pipelines = pipelines.len();
        let admin_settings = engine.http_admin.clone().unwrap_or_default();
        // Initialize metrics system and observed event store.
        // ToDo A hierarchical metrics system will be implemented to better support hardware with multiple NUMA nodes.
        let telemetry_config = &engine.telemetry;
        otel_info!(
            "controller.start",
            num_pipeline_groups = num_pipeline_groups,
            num_pipelines = num_pipelines
        );

        // Create the shared telemetry registry first - it will be used by both
        // the observed state store and the internal telemetry system.
        let telemetry_registry = TelemetryRegistryHandle::new();

        // Create the observed state store for the telemetry system.
        let obs_state_store =
            ObservedStateStore::new(&engine.observed_state, telemetry_registry.clone());
        let obs_state_handle = obs_state_store.handle();
        let engine_evt_reporter = obs_state_store.reporter(engine.observed_state.engine_events);
        let console_async_reporter = telemetry_config
            .logs
            .providers
            .uses_console_async_provider()
            .then(|| obs_state_store.reporter(engine.observed_state.logging_events));

        // Create the telemetry system. The console_async_reporter is passed when any
        // providers use ConsoleAsync. The its_logs_receiver is passed when any
        // providers use the ITS mode.
        let telemetry_system = InternalTelemetrySystem::new(
            telemetry_config,
            telemetry_registry.clone(),
            console_async_reporter,
            engine_context,
        )?;

        let admin_tracing_setup = telemetry_system.admin_tracing_setup();
        let internal_tracing_setup = telemetry_system.internal_tracing_setup();

        let metrics_dispatcher = telemetry_system.dispatcher();
        let metrics_reporter = telemetry_system.reporter();
        let controller_ctx = ControllerContext::new(telemetry_system.registry());

        for pipeline_entry in &pipelines {
            let pipeline_key = PipelineKey::new(
                pipeline_entry.pipeline_group_id.clone(),
                pipeline_entry.pipeline_id.clone(),
            );
            obs_state_store.register_pipeline_health_policy(
                pipeline_key,
                pipeline_entry.policies.health.clone(),
            );
        }

        let pipeline_count = pipelines.len();
        let all_cores =
            core_affinity::get_core_ids().ok_or_else(|| Error::CoreDetectionUnavailable)?;
        let its_core = *all_cores.first().expect("a cpu core");
        let its_key = Self::internal_pipeline_key(its_core);
        if let Some(pipeline) = observability_pipeline.as_ref() {
            obs_state_store.register_pipeline_health_policy(
                PipelineKey::new(
                    its_key.pipeline_group_id.clone(),
                    its_key.pipeline_id.clone(),
                ),
                pipeline.policies.health.clone(),
            );
        }
        let available_core_ids = if pipeline_count == 0 {
            Vec::new()
        } else {
            all_cores
        };

        let internal_pipeline_handle = Self::spawn_internal_pipeline_if_configured(
            its_key.clone(),
            its_core,
            observability_pipeline,
            &telemetry_system,
            self.pipeline_factory,
            &controller_ctx,
            &engine_evt_reporter,
            &metrics_reporter,
            internal_tracing_setup,
        )?;

        // TODO: This should be validated somewhere, that engine observability pipeline is
        // defined when ITS is requested. Possibly we could fill in a default.
        let has_internal_pipeline = internal_pipeline_handle.is_some();
        match (
            has_internal_pipeline,
            telemetry_config.logs.providers.uses_its_provider(),
        ) {
            (false, true) => {
                otel_warn!(
                    "ITS provider requested yet engine.observability.pipeline is not defined"
                )
            }
            (true, false) => {
                otel_warn!(
                    "engine.observability.pipeline is defined yet ITS provider is not requested"
                )
            }
            _ => {}
        };

        // Initialize the global subscriber AFTER the internal pipeline has signaled
        // successful startup. This ensures the channel receiver is being consumed
        // before we start sending logs.
        telemetry_system.init_global_subscriber();

        let internal_collector = telemetry_system.collector();
        let metrics_agg_handle = spawn_thread_local_task(
            "metrics-aggregator",
            admin_tracing_setup.clone(),
            move |cancellation_token| internal_collector.run(cancellation_token),
        )?;

        // Start the metrics dispatcher only if there are metric readers configured.
        let metrics_dispatcher_handle = if telemetry_config.metrics.has_readers() {
            Some(spawn_thread_local_task(
                "metrics-dispatcher",
                admin_tracing_setup.clone(),
                move |cancellation_token| metrics_dispatcher.run_dispatch_loop(cancellation_token),
            )?)
        } else {
            None
        };

        // Start the observed state store background task
        let obs_state_join_handle = spawn_thread_local_task(
            "observed-state-store",
            admin_tracing_setup.clone(),
            move |cancellation_token| obs_state_store.run(cancellation_token),
        )?;

        let mut threads = Vec::new();
        let mut ctrl_msg_senders = Vec::new();

        // TODO: We do not have proper thread::current().id assignment.
        let mut next_thread_id: usize = 1;
        let its_thread_id: usize = 0;

        // Add internal pipeline to threads list if present
        if let Some((thread_name, handle)) = internal_pipeline_handle {
            threads.push((thread_name, its_thread_id, its_key, handle));
        }

        for pipeline_entry in pipelines {
            let channel_capacity_policy = pipeline_entry.policies.channel_capacity;
            let telemetry_policy = pipeline_entry.policies.telemetry;
            let resources_policy = pipeline_entry.policies.resources;
            let pipeline_group_id = pipeline_entry.pipeline_group_id;
            let pipeline_id = pipeline_entry.pipeline_id;
            let pipeline = pipeline_entry.pipeline;
            let requested_cores = Self::select_cores_for_allocation(
                available_core_ids.clone(),
                &resources_policy.core_allocation,
            )?;

            let num_cores = requested_cores.len();
            for core_id in requested_cores {
                let pipeline_key = DeployedPipelineKey {
                    pipeline_group_id: pipeline_group_id.clone(),
                    pipeline_id: pipeline_id.clone(),
                    core_id: core_id.id,
                };
                let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) =
                    pipeline_ctrl_msg_channel(channel_capacity_policy.control.pipeline);
                ctrl_msg_senders.push(pipeline_ctrl_msg_tx.clone());

                let pipeline_config = pipeline.clone();
                let pipeline_factory = self.pipeline_factory;
                let thread_id = next_thread_id;
                next_thread_id += 1;
                let pipeline_handle = controller_ctx.pipeline_context_with(
                    pipeline_group_id.clone(),
                    pipeline_id.clone(),
                    core_id.id,
                    num_cores,
                    thread_id,
                );
                let metrics_reporter = metrics_reporter.clone();

                let thread_name = format!(
                    "pipeline-{}-{}-core-{}",
                    pipeline_group_id.as_ref(),
                    pipeline_id.as_ref(),
                    core_id.id
                );

                let run_key = pipeline_key.clone();
                let engine_tracing_setup = telemetry_system.engine_tracing_setup();
                let engine_evt_reporter = engine_evt_reporter.clone();
                let effective_channel_capacity_policy = channel_capacity_policy.clone();
                let effective_telemetry_policy = telemetry_policy.clone();
                let handle = thread::Builder::new()
                    .name(thread_name.clone())
                    .spawn(move || {
                        Self::run_pipeline_thread(
                            run_key,
                            core_id,
                            pipeline_config,
                            effective_channel_capacity_policy,
                            effective_telemetry_policy,
                            pipeline_factory,
                            pipeline_handle,
                            engine_evt_reporter,
                            metrics_reporter,
                            pipeline_ctrl_msg_tx,
                            pipeline_ctrl_msg_rx,
                            engine_tracing_setup,
                            None,
                        )
                    })
                    .map_err(|e| Error::ThreadSpawnError {
                        thread_name: thread_name.clone(),
                        source: e,
                    })?;

                threads.push((thread_name, thread_id, pipeline_key, handle));
            }
        }

        // Drop the original metrics sender so only pipeline threads hold references
        drop(metrics_reporter);

        // Start the admin HTTP server
        let admin_server_handle = spawn_thread_local_task(
            "http-admin",
            admin_tracing_setup,
            move |cancellation_token| {
                // Convert the concrete senders to trait objects for the admin crate
                let admin_senders: Vec<Arc<dyn otap_df_engine::control::PipelineAdminSender>> =
                    ctrl_msg_senders
                        .into_iter()
                        .map(|sender| {
                            Arc::new(sender)
                                as Arc<dyn otap_df_engine::control::PipelineAdminSender>
                        })
                        .collect();

                otap_df_admin::run(
                    admin_settings,
                    obs_state_handle,
                    admin_senders,
                    telemetry_registry,
                    cancellation_token,
                )
            },
        )?;

        // Wait for all pipeline threads to finish and collect their results
        let mut results: Vec<Result<(), Error>> = Vec::with_capacity(threads.len());
        for (thread_name, thread_id, pipeline_key, handle) in threads {
            match handle.join() {
                Ok(Ok(_)) => {
                    engine_evt_reporter.report(EngineEvent::drained(pipeline_key, None));
                }
                Ok(Err(e)) => {
                    let err_summary: ErrorSummary = error_summary_from_gen(&e);
                    engine_evt_reporter.report(EngineEvent::pipeline_runtime_error(
                        pipeline_key.clone(),
                        "Pipeline encountered a runtime error.",
                        err_summary,
                    ));
                    results.push(Err(e));
                }
                Err(e) => {
                    let err_summary = ErrorSummary::Pipeline {
                        error_kind: "panic".into(),
                        message: "The pipeline panicked during execution.".into(),
                        source: Some(format!("{e:?}")),
                    };
                    engine_evt_reporter.report(EngineEvent::pipeline_runtime_error(
                        pipeline_key.clone(),
                        "The pipeline panicked during execution.",
                        err_summary,
                    ));
                    // Thread join failed, handle the error
                    let core_id = pipeline_key.core_id;
                    return Err(Error::ThreadPanic {
                        thread_name,
                        thread_id,
                        core_id,
                        panic_message: format!("{e:?}"),
                    });
                }
            }
        }

        // Check if any pipeline threads returned an error
        if let Some(err) = results.into_iter().find_map(Result::err) {
            return Err(err);
        }

        // In standard engine mode we keep the main thread parked after startup.
        if run_mode == RunMode::ParkMainThread {
            thread::park();
        }

        // All pipelines have finished; shut down the admin HTTP server and metric aggregator gracefully.
        admin_server_handle.shutdown_and_join()?;
        metrics_agg_handle.shutdown_and_join()?;
        if let Some(handle) = metrics_dispatcher_handle {
            handle.shutdown_and_join()?;
        }
        obs_state_join_handle.shutdown_and_join()?;
        telemetry_system.shutdown_otel()?;

        Ok(())
    }

    /// Selects which CPU cores to use based on the given allocation.
    fn select_cores_for_allocation(
        mut available_core_ids: Vec<CoreId>,
        core_allocation: &CoreAllocation,
    ) -> Result<Vec<CoreId>, Error> {
        available_core_ids.sort_by_key(|c| c.id);

        let max_core_id = available_core_ids.iter().map(|c| c.id).max().unwrap_or(0);
        let num_cores = available_core_ids.len();

        match core_allocation {
            CoreAllocation::AllCores => Ok(available_core_ids),
            CoreAllocation::CoreCount { count } => {
                if *count == 0 {
                    Ok(available_core_ids)
                } else if *count > num_cores {
                    Err(Error::InvalidCoreAllocation {
                        alloc: core_allocation.clone(),
                        message: format!(
                            "Requested {} cores but only {} cores available on this system",
                            count, num_cores
                        ),
                        available: available_core_ids.iter().map(|c| c.id).collect(),
                    })
                } else {
                    Ok(available_core_ids.into_iter().take(*count).collect())
                }
            }
            CoreAllocation::CoreSet { set } => {
                // Validate all ranges first
                for r in set.iter() {
                    if r.start > r.end {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: core_allocation.clone(),
                            message: format!(
                                "Invalid core range: start ({}) is greater than end ({})",
                                r.start, r.end
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                    if r.start > max_core_id {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: core_allocation.clone(),
                            message: format!(
                                "Core ID {} exceeds available cores (system has cores 0-{})",
                                r.start, max_core_id
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                    if r.end > max_core_id {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: core_allocation.clone(),
                            message: format!(
                                "Core ID {} exceeds available cores (system has cores 0-{})",
                                r.end, max_core_id
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                }

                // Check for overlapping ranges
                for (i, r1) in set.iter().enumerate() {
                    for r2 in set.iter().skip(i + 1) {
                        // Two ranges overlap if they share any common cores
                        if r1.start <= r2.end && r2.start <= r1.end {
                            let overlap_start = r1.start.max(r2.start);
                            let overlap_end = r1.end.min(r2.end);
                            return Err(Error::InvalidCoreAllocation {
                                alloc: core_allocation.clone(),
                                message: format!(
                                    "Core ranges overlap: {}-{} and {}-{} share cores {}-{}",
                                    r1.start, r1.end, r2.start, r2.end, overlap_start, overlap_end
                                ),
                                available: available_core_ids.iter().map(|c| c.id).collect(),
                            });
                        }
                    }
                }

                // Filter cores in range
                let selected: Vec<_> = available_core_ids
                    .into_iter()
                    // Naively check if each interval contains the point
                    // This problem is known as the "Interval Stabbing Problem"
                    // and has more efficient but more complex solutions
                    .filter(|c| set.iter().any(|r| r.start <= c.id && c.id <= r.end))
                    .collect();

                if selected.is_empty() {
                    return Err(Error::InvalidCoreAllocation {
                        alloc: core_allocation.clone(),
                        message: "No available cores in the specified ranges".to_owned(),
                        available: core_affinity::get_core_ids()
                            .unwrap_or_default()
                            .iter()
                            .map(|c| c.id)
                            .collect(),
                    });
                }

                Ok(selected)
            }
        }
    }

    fn internal_pipeline_key(core_id: CoreId) -> DeployedPipelineKey {
        DeployedPipelineKey {
            pipeline_group_id: SYSTEM_PIPELINE_GROUP_ID.into(),
            pipeline_id: SYSTEM_OBSERVABILITY_PIPELINE_ID.into(),
            core_id: core_id.id,
        }
    }

    /// Spawns the internal telemetry pipeline if engine observability config provides one.
    ///
    /// Returns the thread handle if an internal pipeline was spawned
    /// and waits for it to start, or None.
    #[allow(clippy::too_many_arguments)]
    fn spawn_internal_pipeline_if_configured(
        its_key: DeployedPipelineKey,
        its_core: CoreId,
        observability_pipeline: Option<ResolvedPipelineConfig>,
        telemetry_system: &InternalTelemetrySystem,
        pipeline_factory: &'static PipelineFactory<PData>,
        controller_ctx: &ControllerContext,
        engine_evt_reporter: &ObservedEventReporter,
        metrics_reporter: &MetricsReporter,
        tracing_setup: TracingSetup,
    ) -> Result<Option<(String, thread::JoinHandle<Result<Vec<()>, Error>>)>, Error> {
        let (internal_config, channel_capacity_policy, telemetry_policy): (
            PipelineConfig,
            ChannelCapacityPolicy,
            TelemetryPolicy,
        ) = match observability_pipeline {
            Some(config) if config.role == ResolvedPipelineRole::ObservabilityInternal => {
                let channel_capacity_policy = config.policies.channel_capacity;
                let telemetry_policy = config.policies.telemetry;
                (config.pipeline, channel_capacity_policy, telemetry_policy)
            }
            Some(_) => {
                // Note: This path is internal-only and should be filtered by caller.
                return Ok(None);
            }
            _ => {
                // Note: Inconsistent configurations are checked elsewhere.
                // This method is "_if_configured()" for lifetime reasons,
                // so a silent return.
                return Ok(None);
            }
        };

        let its_settings = match telemetry_system.internal_telemetry_settings() {
            None => {
                // Note: An inconsistency warning will be logged by the
                // calling function.
                return Ok(None);
            }
            Some(its_settings) => its_settings,
        };

        let internal_pipeline_ctx = controller_ctx.pipeline_context_with(
            its_key.pipeline_group_id.clone(),
            its_key.pipeline_id.clone(),
            its_key.core_id,
            1, // Internal telemetry pipeline runs on a single core
            0, // TODO: we do not have a thread_id
        );

        // Create control message channel for internal pipeline
        let (internal_ctrl_tx, internal_ctrl_rx) =
            pipeline_ctrl_msg_channel(channel_capacity_policy.control.pipeline);

        // Create a channel to signal startup success/failure
        let (startup_tx, startup_rx) = std_mpsc::sync_channel::<Result<(), EngineError>>(1);

        let thread_name = "internal-pipeline".to_string();
        let internal_evt_reporter = engine_evt_reporter.clone();
        let internal_metrics_reporter = metrics_reporter.clone();
        let internal_channel_capacity_policy = channel_capacity_policy;
        let internal_telemetry_policy = telemetry_policy;

        let handle = thread::Builder::new()
            .name(thread_name.clone())
            .spawn(move || {
                Self::run_pipeline_thread(
                    its_key,
                    its_core,
                    internal_config,
                    internal_channel_capacity_policy,
                    internal_telemetry_policy,
                    pipeline_factory,
                    internal_pipeline_ctx,
                    internal_evt_reporter,
                    internal_metrics_reporter,
                    internal_ctrl_tx,
                    internal_ctrl_rx,
                    tracing_setup,
                    Some((its_settings, startup_tx)),
                )
            })
            .map_err(|e| Error::ThreadSpawnError {
                thread_name: thread_name.clone(),
                source: e,
            })?;

        // Wait for the internal pipeline to signal successful startup
        match startup_rx.recv() {
            Ok(Ok(())) => {
                otel_info!(
                    "internal_pipeline.started",
                    message = "Internal telemetry pipeline started successfully"
                );
            }
            Ok(Err(e)) => {
                // Internal pipeline failed to build - propagate the error
                return Err(Error::PipelineRuntimeError {
                    source: Box::new(e),
                });
            }
            Err(err) => {
                // Channel closed unexpectedly - thread may have panicked
                return Err(Error::PipelineRuntimeError {
                    source: Box::new(err),
                });
            }
        }

        Ok(Some((thread_name, handle)))
    }

    /// Runs a single pipeline in the current thread.
    fn run_pipeline_thread(
        pipeline_key: DeployedPipelineKey,
        core_id: CoreId,
        pipeline_config: PipelineConfig,
        channel_capacity_policy: ChannelCapacityPolicy,
        telemetry_policy: TelemetryPolicy,
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_context: PipelineContext,
        obs_evt_reporter: ObservedEventReporter,
        metrics_reporter: MetricsReporter,
        pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<PData>,
        pipeline_ctrl_msg_rx: PipelineCtrlMsgReceiver<PData>,
        tracing_setup: TracingSetup,
        internal_telemetry: Option<(
            InternalTelemetrySettings,
            std_mpsc::SyncSender<Result<(), EngineError>>,
        )>,
    ) -> Result<Vec<()>, Error> {
        // Pin thread to specific core. As much as possible, we pin
        // before allocating memory.
        if !core_affinity::set_for_current(core_id) {
            // Continue execution even if pinning fails.
            // This is acceptable because the OS will still schedule the thread, but performance may be less predictable.
            otel_warn!(
                "core_affinity.set_failed",
                message = "Failed to set core affinity for pipeline thread. Performance may be less predictable."
            );
        }

        // Run the pipeline with thread-local tracing subscriber active.
        tracing_setup.with_subscriber(|| {
            // Create a tracing span for this pipeline thread
            // so that all logs within this scope include pipeline context.
            let span = otel_info_span!("pipeline_thread", core.id = core_id.id);
            let _guard = span.enter();

            // The controller creates a pipeline instance into a dedicated thread. The corresponding
            // entity is registered here for proper context tracking and set into thread-local storage
            // in order to be accessible by all components within this thread.
            let pipeline_entity_key = pipeline_context.register_pipeline_entity();
            let _pipeline_entity_guard =
                set_pipeline_entity_key(pipeline_context.metrics_registry(), pipeline_entity_key);

            obs_evt_reporter.report(EngineEvent::admitted(
                pipeline_key.clone(),
                Some("Pipeline admission successful.".to_owned()),
            ));

            // Build the runtime pipeline from the configuration
            let its_settings = internal_telemetry.as_ref().map(|(s, _)| s).cloned();
            let runtime_pipeline = pipeline_factory
                .build(
                    pipeline_context.clone(),
                    pipeline_config.clone(),
                    channel_capacity_policy,
                    telemetry_policy,
                    its_settings,
                )
                .map_err(|e| {
                    if let Some((_, startup_tx)) = internal_telemetry.as_ref() {
                        let _ = startup_tx.send(Err(EngineError::InternalError {
                            message: e.to_string(),
                        }));
                    }
                    Error::PipelineRuntimeError {
                        source: Box::new(e),
                    }
                })?;

            obs_evt_reporter.report(EngineEvent::ready(
                pipeline_key.clone(),
                Some("Pipeline initialization successful.".to_owned()),
            ));

            if let Some((_, startup_tx)) = internal_telemetry.as_ref() {
                let _ = startup_tx.send(Ok(()));
            }

            // Start the pipeline (this will use the current thread's Tokio runtime)
            runtime_pipeline
                .run_forever(
                    pipeline_key,
                    pipeline_context,
                    obs_evt_reporter,
                    metrics_reporter,
                    pipeline_ctrl_msg_tx,
                    pipeline_ctrl_msg_rx,
                )
                .map_err(|e| Error::PipelineRuntimeError {
                    source: Box::new(e),
                })
        })
    }
}

fn error_summary_from_gen(error: &Error) -> ErrorSummary {
    match error {
        Error::PipelineRuntimeError { source } => {
            if let Some(engine_error) = source.downcast_ref::<EngineError>() {
                error_summary_from(engine_error)
            } else {
                ErrorSummary::Pipeline {
                    error_kind: "runtime".into(),
                    message: source.to_string(),
                    source: None,
                }
            }
        }
        _ => ErrorSummary::Pipeline {
            error_kind: "runtime".into(),
            message: error.to_string(),
            source: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::policy::CoreRange;

    fn available_core_ids() -> Vec<CoreId> {
        vec![
            CoreId { id: 0 },
            CoreId { id: 1 },
            CoreId { id: 2 },
            CoreId { id: 3 },
            CoreId { id: 4 },
            CoreId { id: 5 },
            CoreId { id: 6 },
            CoreId { id: 7 },
        ]
    }

    fn to_ids(v: &[CoreId]) -> Vec<usize> {
        v.iter().map(|c| c.id).collect()
    }

    #[test]
    fn select_all_cores_by_default() {
        let core_allocation = CoreAllocation::AllCores;
        let available_core_ids = available_core_ids();
        let expected_core_ids = available_core_ids.clone();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), to_ids(&expected_core_ids));
    }

    #[test]
    fn select_limited_by_num_cores() {
        let core_allocation = CoreAllocation::CoreCount { count: 4 };
        let available_core_ids = available_core_ids();
        let result = Controller::<()>::select_cores_for_allocation(
            available_core_ids.clone(),
            &core_allocation,
        )
        .unwrap();
        assert_eq!(result.len(), 4);
        let expected_ids: Vec<usize> = available_core_ids
            .into_iter()
            .take(4)
            .map(|c| c.id)
            .collect();
        assert_eq!(to_ids(&result), expected_ids);
    }

    #[test]
    fn select_with_valid_single_core_range() {
        let available_core_ids = available_core_ids();
        let first_id = available_core_ids[0].id;
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange {
                start: first_id,
                end: first_id,
            }],
        };
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), vec![first_id]);
    }

    #[test]
    fn select_with_valid_multi_core_range() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 5 },
                CoreRange { start: 6, end: 6 },
            ],
        };
        let available_core_ids = available_core_ids();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), vec![2, 3, 4, 5, 6]);
    }

    #[test]
    fn select_with_inverted_range_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange { start: 2, end: 1 }],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, .. } => {
                assert_eq!(alloc, core_allocation);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_out_of_bounds_range_errors() {
        let start = 100;
        let end = 110;
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange { start, end }],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, .. } => {
                assert_eq!(alloc, core_allocation);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_zero_count_uses_all_cores() {
        let core_allocation = CoreAllocation::CoreCount { count: 0 };
        let available_core_ids = available_core_ids();
        let expected_core_ids = available_core_ids.clone();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), to_ids(&expected_core_ids));
    }

    #[test]
    fn select_with_overlapping_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 5 },
                CoreRange { start: 4, end: 7 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_fully_overlapping_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 6 },
                CoreRange { start: 3, end: 5 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_identical_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 3, end: 5 },
                CoreRange { start: 3, end: 5 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_adjacent_ranges_succeeds() {
        // Adjacent but non-overlapping ranges should work
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 3 },
                CoreRange { start: 4, end: 5 },
            ],
        };
        let available_core_ids = available_core_ids();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), vec![2, 3, 4, 5]);
    }

    #[test]
    fn select_with_multiple_overlapping_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 1, end: 3 },
                CoreRange { start: 2, end: 4 },
                CoreRange { start: 5, end: 6 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
