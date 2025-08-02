// SPDX-License-Identifier: Apache-2.0

//! OTAP Dataflow Engine Controller
//!
//! This controller is responsible for managing and monitoring the execution of pipelines within
//! the current process. The execution model supported by this controller is based on a
//! thread-per-core approach, allowing efficient use of multi-core architectures while
//! minimizing contention risks between threads. Additionally, each thread created by this
//! controller is associated with a specific core, ensuring that pipelines run optimally by
//! using CPU resources efficiently and predictably.
//!
//! Future work includes:
//! - todo: Status and health checks for pipelines
//! - todo: Graceful shutdown of pipelines
//! - todo: Auto-restart threads in case of panic
//! - todo: Live pipeline updates
//! - todo: Better resource control
//! - todo: Monitoring
//! - todo: Support pipeline groups

use otap_df_config::{pipeline::PipelineConfig, pipeline_group::Quota};
use otap_df_engine::PipelineFactory;
use std::thread;

/// Error messages.
pub mod error;

/// Controller for managing pipelines.
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
    pub fn start_pipeline(
        &self,
        pipeline: PipelineConfig,
        quota: Quota,
    ) -> Result<(), error::Error> {
        // Get available CPU cores for pinning
        let all_core_ids =
            core_affinity::get_core_ids().ok_or_else(|| error::Error::InternalError {
                message: "Failed to get available CPU cores".to_owned(),
            })?;

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

            let handle = thread::Builder::new()
                .name(format!("pipeline-core-{}", core_id.id))
                .spawn(move || {
                    Self::run_pipeline_thread(core_id, pipeline_config, pipeline_factory)
                })
                .map_err(|e| error::Error::InternalError {
                    message: format!("Failed to spawn thread {thread_id}: {e}"),
                })?;

            threads.push(handle);
        }

        // Wait for all threads to finish
        // Note: In a real-world scenario, you might want to handle thread panics and errors more gracefully.
        // For now, we will just log the errors and continue.
        for (thread_id, handle) in threads.into_iter().enumerate() {
            match handle.join() {
                Ok(_) => {
                    // Thread completed successfully
                }
                Err(e) => {
                    // Thread join failed, handle the error
                    return Err(error::Error::InternalError {
                        message: format!(
                            "Failed to join thread pipeline-core-{thread_id:?}: {e:?}"
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Runs a single pipeline in the current thread.
    fn run_pipeline_thread(
        core_id: core_affinity::CoreId,
        pipeline_config: PipelineConfig,
        pipeline_factory: &'static PipelineFactory<PData>,
    ) -> Result<(), error::Error> {
        // Pin thread to specific core
        if !core_affinity::set_for_current(core_id) {
            // Continue execution even if pinning fails
        }

        // Build the runtime pipeline from the configuration
        let runtime_pipeline = pipeline_factory
            .build(pipeline_config.clone())
            .map_err(|e| error::Error::PipelineBuildFailed {
                source: Box::new(e),
            })?;

        // Start the pipeline (this will use the current thread's Tokio runtime)
        runtime_pipeline
            .start()
            .map_err(|e| error::Error::PipelineBuildFailed {
                source: Box::new(e),
            })?;

        Ok(())
    }
}
