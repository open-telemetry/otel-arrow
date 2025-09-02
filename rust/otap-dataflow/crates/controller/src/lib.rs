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
use otap_df_engine::control::{
    PipelineCtrlMsgReceiver, PipelineCtrlMsgSender, pipeline_ctrl_msg_channel,
};
use otap_df_telemetry::MetricsSystem;
use otap_df_telemetry::reporter::MetricsReporter;
use std::thread;
use core_affinity::CoreId;
use crate::error::Error;

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
        admin_settings: HttpAdminSettings,
    ) -> Result<(), Error> {
        // Initialize a global metrics system and reporter.
        // ToDo A hierarchical metrics system will be implemented to better support hardware with multiple NUMA nodes.
        let metrics_system = MetricsSystem::default();
        let metrics_reporter = metrics_system.reporter();
        let controller_ctx = ControllerContext::new(metrics_system.registry());

        // Start the metrics aggregation
        let metrics_registry = metrics_system.registry();
        let metrics_agg_handle =
            spawn_thread_local_task("metrics-aggregator", move |cancellation_token| {
                metrics_system.run(cancellation_token)
            })?;

        // Start one thread per requested core
        // Get available CPU cores for pinning
        let requested_cores = Self::compute_requested_cores(
            core_affinity::get_core_ids().ok_or_else(|| Error::CoreDetectionUnavailable)?,
            quota
        )?;
        let mut threads = Vec::with_capacity(requested_cores.len());
        let mut ctrl_msg_senders = Vec::with_capacity(requested_cores.len());

        for (thread_id, core_id) in requested_cores.into_iter().enumerate() {
            let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(
                pipeline
                    .pipeline_settings()
                    .default_pipeline_ctrl_msg_channel_size,
            );
            ctrl_msg_senders.push(pipeline_ctrl_msg_tx.clone());

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
                        pipeline_ctrl_msg_tx,
                        pipeline_ctrl_msg_rx,
                    )
                })
                .map_err(|e| Error::ThreadSpawnError {
                    thread_name: thread_name.clone(),
                    source: e,
                })?;

            threads.push((thread_name, thread_id, core_id.id, handle));
        }

        // Drop the original metrics sender so only pipeline threads hold references
        drop(metrics_reporter);

        // Start the admin HTTP server
        let admin_server_handle =
            spawn_thread_local_task("http-admin", move |cancellation_token| {
                otap_df_admin::run(
                    admin_settings,
                    ctrl_msg_senders,
                    metrics_registry,
                    cancellation_token,
                )
            })?;

        // Wait for all pipeline threads to finish and collect their results
        let mut results = Vec::with_capacity(threads.len());
        for (thread_name, thread_id, core_id, handle) in threads {
            match handle.join() {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    // Thread join failed, handle the error
                    return Err(Error::ThreadPanic {
                        thread_name,
                        thread_id,
                        core_id,
                        panic_message: format!("{e:?}"),
                    });
                }
            }
        }

        // In this project phase (alpha), we just print the results of the pipelines and park the
        // main thread indefinitely. This is useful for debugging and demonstration purposes.
        // We can for example use the shutdown endpoint and still inspect the metrics.
        // ToDo Add CTRL-C handler to initiate graceful shutdown of pipelines and admin server.
        // ToDo Maintain an internal Global Observed State that will exposed by the admin endpoints and use by a reconciler to orchestrate pipeline updates.
        #[allow(clippy::dbg_macro)] // Use for demonstration purposes (temp)
        {
            dbg!(&results);
        }
        thread::park();

        // All pipelines have finished; shut down the admin HTTP server and metric aggregator gracefully.
        admin_server_handle.shutdown_and_join()?;
        metrics_agg_handle.shutdown_and_join()?;

        Ok(())
    }

    /// Returns the list of CPU core IDs to use based on the given quota.
    fn compute_requested_cores(all_core_ids: Vec<CoreId>, quota: Quota) -> Result<Vec<CoreId>, Error> {
        // If a specific core ID range is requested, select matching cores, otherwise derive by count
        let requested_cores = if let Some(range) = quota.core_id_range.as_ref() {
            let mut available_ids: Vec<_> = all_core_ids.clone().into_iter().collect();
            available_ids.sort_by_key(|c| c.id);
            if range.start > range.end {
                return Err(Error::InvalidCoreRange {
                    start: range.start,
                    end: range.end,
                    message: "Start of range is greater than end".to_owned(),
                    available: available_ids.iter().map(|c| c.id).collect(),
                });
            }
            let selected: Vec<_> = available_ids
                .into_iter()
                .filter(|c| c.id >= range.start && c.id <= range.end)
                .collect();
            if selected.is_empty() {
                return Err(Error::InvalidCoreRange {
                    start: range.start,
                    end: range.end,
                    message: "No available cores in the specified range".to_owned(),
                    available: core_affinity::get_core_ids()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|c| c.id)
                        .collect(),
                });
            }
            selected
        } else {
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

            let mut available_ids: Vec<_> = all_core_ids.into_iter().collect();
            available_ids.sort_by_key(|c| c.id);
            available_ids.into_iter().take(num_requested_cores).collect()
        };
        Ok(requested_cores)
    }

    /// Runs a single pipeline in the current thread.
    fn run_pipeline_thread(
        core_id: CoreId,
        pipeline_config: PipelineConfig,
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_handle: PipelineContext,
        metrics_reporter: MetricsReporter,
        pipeline_ctrl_msg_tx: PipelineCtrlMsgSender,
        pipeline_ctrl_msg_rx: PipelineCtrlMsgReceiver,
    ) -> Result<Vec<()>, Error> {
        // Pin thread to specific core
        if !core_affinity::set_for_current(core_id) {
            // Continue execution even if pinning fails.
            // This is acceptable because the OS will still schedule the thread, but performance may be less predictable.
            // ToDo Add a warning here once logging is implemented.
        }

        // Build the runtime pipeline from the configuration
        let runtime_pipeline = pipeline_factory
            .build(pipeline_handle, pipeline_config.clone())
            .map_err(|e| Error::PipelineRuntimeError {
                source: Box::new(e),
            })?;

        // Start the pipeline (this will use the current thread's Tokio runtime)
        runtime_pipeline
            .run_forever(metrics_reporter, pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx)
            .map_err(|e| Error::PipelineRuntimeError {
                source: Box::new(e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available_core_ids() -> Vec<CoreId> {
        vec![
            CoreId { id: 0 }, CoreId { id: 1 }, CoreId { id: 2 }, CoreId { id: 3 },
            CoreId { id: 4 }, CoreId { id: 5 }, CoreId { id: 6 }, CoreId { id: 7 },
        ]
    }

    fn to_ids(v: &[CoreId]) -> Vec<usize> {
        v.iter().map(|c| c.id).collect()
    }

    #[test]
    fn compute_all_cores_by_default() {
        let quota = Quota {
            num_cores: 0,
            core_id_range: None,
        };
        let available_core_ids = available_core_ids();
        let expected_core_ids = available_core_ids.clone();
        let result = Controller::<()>::compute_requested_cores(available_core_ids, quota).unwrap();
        assert_eq!(to_ids(&result), to_ids(&expected_core_ids));
    }

    #[test]
    fn compute_limited_by_num_cores() {
        let quota = Quota {
            num_cores: 4,
            core_id_range: None,
        };
        let available_core_ids = available_core_ids();
        let result = Controller::<()>::compute_requested_cores(available_core_ids.clone(), quota).unwrap();
        assert_eq!(result.len(), 4);
        let expected_ids: Vec<usize> = available_core_ids.into_iter().take(4).map(|c| c.id).collect();
        assert_eq!(to_ids(&result), expected_ids);
    }

    #[test]
    fn compute_with_valid_single_core_range() {
        let available_core_ids = available_core_ids();
        let first_id = available_core_ids[0].id;
        let quota = Quota {
            num_cores: 0,
            core_id_range: Some(otap_df_config::pipeline_group::CoreIdRange {
                start: first_id,
                end: first_id,
            }),
        };
        let result = Controller::<()>::compute_requested_cores(available_core_ids, quota).unwrap();
        assert_eq!(to_ids(&result), vec![first_id]);
    }

    #[test]
    fn compute_with_valid_multi_core_range() {
        let quota = Quota {
            num_cores: 0,
            core_id_range: Some(otap_df_config::pipeline_group::CoreIdRange { start: 2, end: 5 }),
        };
        let available_core_ids = available_core_ids();
        let result = Controller::<()>::compute_requested_cores(available_core_ids, quota).unwrap();
        assert_eq!(to_ids(&result), vec![2, 3, 4, 5]);
    }

    #[test]
    fn compute_with_inverted_range_errors() {
        let quota = Quota {
            num_cores: 0,
            core_id_range: Some(otap_df_config::pipeline_group::CoreIdRange { start: 2, end: 1 }),
        };
        let available_core_ids = available_core_ids();
        let err = Controller::<()>::compute_requested_cores(available_core_ids, quota).unwrap_err();
        match err {
            Error::InvalidCoreRange { start, end, .. } => {
                assert_eq!(start, 2);
                assert_eq!(end, 1);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn compute_with_out_of_bounds_range_errors() {
        let start = 100;
        let end = 110;
        let quota = Quota {
            num_cores: 0,
            core_id_range: Some(otap_df_config::pipeline_group::CoreIdRange { start, end }),
        };
        let available_core_ids = available_core_ids();
        let err = Controller::<()>::compute_requested_cores(available_core_ids, quota).unwrap_err();
        match err {
            Error::InvalidCoreRange { start: s, end: e, .. } => {
                assert_eq!(s, start);
                assert_eq!(e, end);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
