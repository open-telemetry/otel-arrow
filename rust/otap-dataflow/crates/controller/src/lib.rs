// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP Dataflow Engine Controller
//!
//! This controller manages and monitors the execution of pipelines within the current process.
//! It uses a thread-per-core model, where each thread is pinned to a specific CPU core.
//! This approach maximizes multi-core efficiency and reduces contention between threads,
//! ensuring that each pipeline runs on a dedicated core for predictable and optimal CPU usage.
//!
//! Future work includes:
//! - TODO: Status and health checks for pipelines
//! - TODO: Graceful shutdown of pipelines
//! - TODO: Auto-restart threads in case of panic
//! - TODO: Live pipeline updates
//! - TODO: Better resource control
//! - TODO: Monitoring
//! - TODO: Support pipeline groups

use crate::thread_task::spawn_thread_local_task;
use otap_df_config::engine::HttpAdminSettings;
use otap_df_config::{
    PipelineGroupId, PipelineId, pipeline::PipelineConfig, pipeline_group::Quota,
};
use otap_df_engine::PipelineFactory;
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_telemetry::MetricsSystem;
use otap_df_telemetry::reporter::MetricsReporter;
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

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug> Controller<PData> {
    /// Creates a new controller with the given pipeline factory.
    pub fn new(pipeline_factory: &'static PipelineFactory<PData>) -> Self {
        Self { pipeline_factory }
    }

    /// Starts the controller with the given pipeline configuration and quota.
    pub fn run_forever(
        &self,
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        pipeline: PipelineConfig,
        quota: Quota,
    ) -> Result<(), error::Error> {
        // Initialize a global metrics system and reporter.
        // ToDo A hierarchical metrics system will be implemented to better support hardware with multiple NUMA nodes.
        let metrics_system = MetricsSystem::default();
        let metrics_reporter = metrics_system.reporter();
        let controller_ctx = ControllerContext::new(metrics_system.registry());
        let http_settings = HttpAdminSettings::default();

        // Start the admin HTTP server
        let metrics_registry = metrics_system.registry();
        let admin_server_handle =
            spawn_thread_local_task("http-admin", move |cancellation_token| {
                otap_df_admin::run(http_settings, metrics_registry, cancellation_token)
            })?;

        // Start the metrics aggregation
        let metrics_agg_handle =
            spawn_thread_local_task("metrics-aggregator", move |cancellation_token| {
                metrics_system.run(cancellation_token)
            })?;

        // Get available CPU cores for pinning
        let all_core_ids =
            core_affinity::get_core_ids().ok_or_else(|| error::Error::CoreDetectionUnavailable)?;

        // Determine the number of CPU cores available and requested
        // If quota.num_cores is 0, use all available cores
        // If quota.num_cores is greater than available cores, use the minimum
        // If quota.num_cores is less than available cores, use the requested number
        let num_cpu_cores = all_core_ids.len();
        let num_requested_cores = if quota.num_cores == 0 {
            num_cpu_cores
        } else {
            quota.num_cores.min(num_cpu_cores)
        };

        let requested_cores = all_core_ids
            .into_iter()
            .take(num_requested_cores)
            .collect::<Vec<_>>();

        // Start one thread per core
        let mut threads = Vec::with_capacity(requested_cores.len());
        for (thread_id, core_id) in requested_cores.into_iter().enumerate() {
            let pipeline_config = pipeline.clone();
            let pipeline_factory = self.pipeline_factory;
            let pipeline_handle = controller_ctx.pipeline_context_with(
                pipeline_group_id.clone(),
                pipeline_id.clone(),
                core_id.id,
                thread_id,
            );
            let metrics_reporter = metrics_reporter.clone();

            let thread_name = format!("pipeline-core-{}", core_id.id);
            let handle = thread::Builder::new()
                .name(thread_name.clone())
                .spawn(move || {
                    Self::run_pipeline_thread(
                        core_id,
                        pipeline_config,
                        pipeline_factory,
                        pipeline_handle,
                        metrics_reporter,
                    )
                })
                .map_err(|e| error::Error::ThreadSpawnError {
                    thread_name: thread_name.clone(),
                    source: e,
                })?;

            threads.push((thread_name, thread_id, core_id.id, handle));
        }

        // Drop the original metrics sender so only pipeline threads hold references
        drop(metrics_reporter);

        // Wait for all pipeline threads to finish
        for (thread_name, thread_id, core_id, handle) in threads {
            match handle.join() {
                Ok(Ok(_)) => {
                    // Thread completed successfully
                }
                Ok(Err(e)) => {
                    return Err(e);
                }
                Err(e) => {
                    // Thread join failed, handle the error
                    return Err(error::Error::ThreadPanic {
                        thread_name,
                        thread_id,
                        core_id,
                        panic_message: format!("{e:?}"),
                    });
                }
            }
        }

        // All pipelines have finished; shut down the admin HTTP server and metric aggregator gracefully.
        admin_server_handle.shutdown_and_join()?;
        metrics_agg_handle.shutdown_and_join()?;

        Ok(())
    }

    /// Runs a single pipeline in the current thread.
    fn run_pipeline_thread(
        core_id: core_affinity::CoreId,
        pipeline_config: PipelineConfig,
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_handle: PipelineContext,
        metrics_reporter: MetricsReporter,
    ) -> Result<Vec<()>, error::Error> {
        // Pin thread to specific core
        if !core_affinity::set_for_current(core_id) {
            // Continue execution even if pinning fails.
            // This is acceptable because the OS will still schedule the thread, but performance may be less predictable.
            // ToDo Add a warning here once logging is implemented.
        }

        // Build the runtime pipeline from the configuration
        let runtime_pipeline = pipeline_factory
            .build(pipeline_handle, pipeline_config.clone())
            .map_err(|e| error::Error::PipelineRuntimeError {
                source: Box::new(e),
            })?;

        // Start the pipeline (this will use the current thread's Tokio runtime)
        runtime_pipeline.run_forever(metrics_reporter).map_err(|e| {
            error::Error::PipelineRuntimeError {
                source: Box::new(e),
            }
        })
    }
}
