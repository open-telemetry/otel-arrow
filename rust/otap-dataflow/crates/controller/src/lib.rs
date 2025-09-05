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

use crate::error::Error;
use crate::thread_task::spawn_thread_local_task;
use core_affinity::CoreId;
use otap_df_config::engine::HttpAdminSettings;
use otap_df_config::{
    PipelineGroupId, PipelineId,
    pipeline::PipelineConfig,
    pipeline_group::{CoreAllocation, Quota},
};
use otap_df_engine::PipelineFactory;
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_engine::control::{
    PipelineCtrlMsgReceiver, PipelineCtrlMsgSender, pipeline_ctrl_msg_channel,
};
use otap_df_state::DeployedPipelineKey;
use otap_df_state::reporter::ObservedEventReporter;
use otap_df_state::store::{ObservedEvent, ObservedStateStore};
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
        admin_settings: HttpAdminSettings,
    ) -> Result<(), Error> {
        // Initialize metrics system and observed event store.
        // ToDo A hierarchical metrics system will be implemented to better support hardware with multiple NUMA nodes.
        let metrics_system = MetricsSystem::default();
        let metrics_reporter = metrics_system.reporter();
        let controller_ctx = ControllerContext::new(metrics_system.registry());
        let obs_state_store = ObservedStateStore::default();
        let obs_evt_reporter = obs_state_store.reporter(); // Only the reporting API
        let obs_state_handle = obs_state_store.handle(); // Only the querying API

        // Start the metrics aggregation
        let metrics_registry = metrics_system.registry();
        let metrics_agg_handle =
            spawn_thread_local_task("metrics-aggregator", move |cancellation_token| {
                metrics_system.run(cancellation_token)
            })?;

        // Start the observed state store background task
        let obs_state_join_handle =
            spawn_thread_local_task("observed-state-store", move |cancellation_token| {
                obs_state_store.run(cancellation_token)
            })?;

        // Start one thread per requested core
        // Get available CPU cores for pinning
        let requested_cores = Self::select_cores_for_quota(
            core_affinity::get_core_ids().ok_or_else(|| Error::CoreDetectionUnavailable)?,
            quota,
        )?;
        let mut threads = Vec::with_capacity(requested_cores.len());
        let mut ctrl_msg_senders = Vec::with_capacity(requested_cores.len());

        // ToDo [LQ] Support multiple pipeline groups in the future.

        for (thread_id, core_id) in requested_cores.into_iter().enumerate() {
            let pipeline_key = DeployedPipelineKey {
                pipeline_group_id: pipeline_group_id.clone(),
                pipeline_id: pipeline_id.clone(),
                core_id: core_id.id,
            };
            obs_evt_reporter.report(ObservedEvent::pipeline_pending(pipeline_key.clone()));
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
            let obs_evt_reporter = obs_evt_reporter.clone();
            let handle = thread::Builder::new()
                .name(thread_name.clone())
                .spawn(move || {
                    Self::run_pipeline_thread(
                        pipeline_key,
                        core_id,
                        pipeline_config,
                        pipeline_factory,
                        pipeline_handle,
                        obs_evt_reporter,
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
                    obs_state_handle,
                    ctrl_msg_senders,
                    metrics_registry,
                    cancellation_token,
                )
            })?;

        // Wait for all pipeline threads to finish and collect their results
        let mut results: Vec<Result<(), Error>> = Vec::with_capacity(threads.len());
        for (thread_name, thread_id, core_id, handle) in threads {
            let pipeline_key = DeployedPipelineKey {
                pipeline_group_id: pipeline_group_id.clone(),
                pipeline_id: pipeline_id.clone(),
                core_id,
            };
            match handle.join() {
                Ok(Ok(_)) => {
                    obs_evt_reporter.report(ObservedEvent::pipeline_stopped(pipeline_key));
                }
                Ok(Err(e)) => {
                    obs_evt_reporter.report(ObservedEvent::pipeline_failed(pipeline_key));
                    // ToDo Report errors as a `condition object` to the pipeline status.
                    results.push(Err(e));
                }
                Err(e) => {
                    // ToDo Report errors as a `condition object` to the pipeline status.
                    obs_evt_reporter.report(ObservedEvent::pipeline_failed(pipeline_key));
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

        // ToDo Add CTRL-C handler to initiate graceful shutdown of pipelines and admin server.

        // In this project phase (alpha), we park the main thread indefinitely. This is useful for
        // debugging and demonstration purposes. The following admin endpoints can be used to
        // inspect the observed state and metrics while the pipelines are running.
        thread::park();

        // All pipelines have finished; shut down the admin HTTP server and metric aggregator gracefully.
        admin_server_handle.shutdown_and_join()?;
        metrics_agg_handle.shutdown_and_join()?;
        obs_state_join_handle.shutdown_and_join()?;

        Ok(())
    }

    /// Selects which CPU cores to use based on the given quota configuration.
    fn select_cores_for_quota(
        mut available_core_ids: Vec<CoreId>,
        quota: Quota,
    ) -> Result<Vec<CoreId>, Error> {
        available_core_ids.sort_by_key(|c| c.id);

        match quota.core_allocation {
            CoreAllocation::AllCores => Ok(available_core_ids),
            CoreAllocation::CoreCount { count } => {
                let count = if count == 0 {
                    available_core_ids.len()
                } else {
                    count.min(available_core_ids.len())
                };
                Ok(available_core_ids.into_iter().take(count).collect())
            }
            CoreAllocation::CoreRange { start, end } => {
                // Validate range
                if start > end {
                    return Err(Error::InvalidCoreRange {
                        start,
                        end,
                        message: "Start of range is greater than end".to_owned(),
                        available: available_core_ids.iter().map(|c| c.id).collect(),
                    });
                }

                // Filter cores in range
                let selected: Vec<_> = available_core_ids
                    .into_iter()
                    .filter(|c| c.id >= start && c.id <= end)
                    .collect();

                if selected.is_empty() {
                    return Err(Error::InvalidCoreRange {
                        start,
                        end,
                        message: "No available cores in the specified range".to_owned(),
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

    /// Runs a single pipeline in the current thread.
    fn run_pipeline_thread(
        pipeline_key: DeployedPipelineKey,
        core_id: CoreId,
        pipeline_config: PipelineConfig,
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_handle: PipelineContext,
        obs_evt_reporter: ObservedEventReporter,
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
            .run_forever(
                pipeline_key,
                obs_evt_reporter,
                metrics_reporter,
                pipeline_ctrl_msg_tx,
                pipeline_ctrl_msg_rx,
            )
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
        let quota = Quota {
            core_allocation: CoreAllocation::AllCores,
        };
        let available_core_ids = available_core_ids();
        let expected_core_ids = available_core_ids.clone();
        let result = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap();
        assert_eq!(to_ids(&result), to_ids(&expected_core_ids));
    }

    #[test]
    fn select_limited_by_num_cores() {
        let quota = Quota {
            core_allocation: CoreAllocation::CoreCount { count: 4 },
        };
        let available_core_ids = available_core_ids();
        let result =
            Controller::<()>::select_cores_for_quota(available_core_ids.clone(), quota).unwrap();
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
        let quota = Quota {
            core_allocation: CoreAllocation::CoreRange {
                start: first_id,
                end: first_id,
            },
        };
        let result = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap();
        assert_eq!(to_ids(&result), vec![first_id]);
    }

    #[test]
    fn select_with_valid_multi_core_range() {
        let quota = Quota {
            core_allocation: CoreAllocation::CoreRange { start: 2, end: 5 },
        };
        let available_core_ids = available_core_ids();
        let result = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap();
        assert_eq!(to_ids(&result), vec![2, 3, 4, 5]);
    }

    #[test]
    fn select_with_inverted_range_errors() {
        let quota = Quota {
            core_allocation: CoreAllocation::CoreRange { start: 2, end: 1 },
        };
        let available_core_ids = available_core_ids();
        let err = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap_err();
        match err {
            Error::InvalidCoreRange { start, end, .. } => {
                assert_eq!(start, 2);
                assert_eq!(end, 1);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_out_of_bounds_range_errors() {
        let start = 100;
        let end = 110;
        let quota = Quota {
            core_allocation: CoreAllocation::CoreRange { start, end },
        };
        let available_core_ids = available_core_ids();
        let err = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap_err();
        match err {
            Error::InvalidCoreRange {
                start: s, end: e, ..
            } => {
                assert_eq!(s, start);
                assert_eq!(e, end);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_zero_count_uses_all_cores() {
        let quota = Quota {
            core_allocation: CoreAllocation::CoreCount { count: 0 },
        };
        let available_core_ids = available_core_ids();
        let expected_core_ids = available_core_ids.clone();
        let result = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap();
        assert_eq!(to_ids(&result), to_ids(&expected_core_ids));
    }
}
