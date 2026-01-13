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
use otap_df_config::pipeline::service::telemetry::logs::{
    INTERNAL_TELEMETRY_RECEIVER_URN, ProviderMode,
};
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
use otap_df_engine::error::{Error as EngineError, error_summary_from};
use otap_df_state::DeployedPipelineKey;
use otap_df_state::event::{ErrorSummary, ObservedEvent};
use otap_df_state::reporter::ObservedEventReporter;
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::logs::EngineLogsSetup;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry::telemetry_settings::TelemetrySettings;
use otap_df_telemetry::{InternalTelemetrySystem, otel_info, otel_info_span, otel_warn};
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
        let telemetry_config = &pipeline.service().telemetry;
        let settings = pipeline.pipeline_settings();
        otel_info!(
            "Controller.Start",
            num_nodes = pipeline.node_iter().count(),
            pdata_channel_size = settings.default_pdata_channel_size,
            node_ctrl_msg_channel_size = settings.default_node_ctrl_msg_channel_size,
            pipeline_ctrl_msg_channel_size = settings.default_pipeline_ctrl_msg_channel_size
        );

        // Validate logs configuration.
        // TODO: add metrics, validate the whole config.
        telemetry_config
            .logs
            .validate()
            .map_err(|msg| Error::ConfigurationError {
                message: msg.to_string(),
            })?;

        // Create telemetry settings.  This creates logs reporter/receiver internally
        let mut telemetry_settings = TelemetrySettings::new(telemetry_config)?;

        // Start the logs collector thread if needed for Direct output mode.
        let _logs_collector_handle =
            if let Some(logs_collector) = telemetry_settings.take_logs_collector() {
                Some(spawn_thread_local_task(
                    "logs-collector",
                    move |_cancellation_token| logs_collector.run(),
                )?)
            } else {
                None
            };

        // Get logs receiver for Internal output mode (passed to internal pipeline)
        let mut logs_receiver = telemetry_settings.take_logs_receiver();

        let metrics_system = InternalTelemetrySystem::new(telemetry_config);
        let metrics_dispatcher = metrics_system.dispatcher();
        let metrics_reporter = metrics_system.reporter();
        let controller_ctx = ControllerContext::new(metrics_system.registry());
        let obs_state_store = ObservedStateStore::new(pipeline.pipeline_settings());
        let obs_evt_reporter = obs_state_store.reporter(); // Only the reporting API
        let obs_state_handle = obs_state_store.handle(); // Only the querying API

        // Start the metrics aggregation
        let telemetry_registry = metrics_system.registry();
        let metrics_agg_handle =
            spawn_thread_local_task("metrics-aggregator", move |cancellation_token| {
                metrics_system.run(cancellation_token)
            })?;

        // Start the metrics dispatcher only if there are metric readers configured.
        let metrics_dispatcher_handle = if telemetry_config.metrics.has_readers() {
            Some(spawn_thread_local_task(
                "metrics-dispatcher",
                move |cancellation_token| metrics_dispatcher.run_dispatch_loop(cancellation_token),
            )?)
        } else {
            None
        };

        // Start the observed state store background task
        let obs_state_join_handle =
            spawn_thread_local_task("observed-state-store", move |cancellation_token| {
                obs_state_store.run(cancellation_token)
            })?;

        // Create engine logs setup based on provider configuration.
        let engine_logs_setup = match telemetry_config.logs.providers.engine {
            ProviderMode::Noop => EngineLogsSetup::Noop,
            ProviderMode::Raw => EngineLogsSetup::Raw,
            ProviderMode::Immediate => EngineLogsSetup::Immediate {
                reporter: telemetry_settings
                    .logs_reporter()
                    .cloned()
                    .expect("validated: immediate requires reporter"),
            },
            ProviderMode::OpenTelemetry => EngineLogsSetup::OpenTelemetry {
                logger_provider: telemetry_settings
                    .logger_provider()
                    .clone()
                    .expect("validated: opentelemetry engine requires logger_provider from global"),
            },
        };
        let log_level = telemetry_config.logs.level;

        // Spawn internal pipeline thread if configured.
        let internal_pipeline_thread =
            if let Some(internal_config) = pipeline.extract_internal_config() {
                // Internal pipeline only exists when output mode is Internal
                // The logs_receiver goes to the internal pipeline's ITR node
                let internal_logs_receiver = logs_receiver.take();
                let internal_factory = self.pipeline_factory;
                let internal_pipeline_id: PipelineId = "internal".into();
                let internal_pipeline_key = DeployedPipelineKey {
                    pipeline_group_id: pipeline_group_id.clone(),
                    pipeline_id: internal_pipeline_id.clone(),
                    core_id: 0, // Virtual core ID for internal pipeline
                };
                let internal_pipeline_ctx = controller_ctx.pipeline_context_with(
                    pipeline_group_id.clone(),
                    internal_pipeline_id.clone(),
                    0, // Virtual core ID
                    0, // Virtual thread ID
                );
                let internal_obs_evt_reporter = obs_evt_reporter.clone();
                let internal_metrics_reporter = metrics_reporter.clone();

                // Internal pipeline uses Raw logging (direct console output)
                // to avoid feedback loops - it can't log through itself
                let internal_engine_logs_setup = EngineLogsSetup::Raw;
                let internal_log_level = log_level;

                // Create control message channel for internal pipeline
                let (internal_ctrl_tx, internal_ctrl_rx) = pipeline_ctrl_msg_channel(
                    internal_config
                        .pipeline_settings()
                        .default_pipeline_ctrl_msg_channel_size,
                );

                let thread_name = "internal-pipeline".to_string();
                let handle = thread::Builder::new()
                    .name(thread_name.clone())
                    .spawn(move || {
                        Self::run_pipeline_thread(
                            internal_pipeline_key,
                            CoreId { id: 0 }, // No pinning for internal pipeline
                            internal_config,
                            internal_factory,
                            internal_pipeline_ctx,
                            internal_obs_evt_reporter,
                            internal_metrics_reporter,
                            internal_engine_logs_setup,
                            internal_log_level,
                            internal_logs_receiver,
                            internal_ctrl_tx,
                            internal_ctrl_rx,
                        )
                    })
                    .map_err(|e| Error::ThreadSpawnError {
                        thread_name: thread_name.clone(),
                        source: e,
                    })?;

                otel_info!(
                    "InternalPipeline.Started",
                    num_nodes = pipeline.internal_nodes().len()
                );

                Some((thread_name, handle))
            } else {
                None
            };

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
            let engine_logs_setup = engine_logs_setup.clone();
            let logs_receiver = logs_receiver.clone();

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
                        engine_logs_setup,
                        log_level,
                        logs_receiver,
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
                // Convert the concrete senders to trait objects for the admin crate
                let admin_senders: Vec<
                    std::sync::Arc<dyn otap_df_engine::control::PipelineAdminSender>,
                > = ctrl_msg_senders
                    .into_iter()
                    .map(|sender| {
                        std::sync::Arc::new(sender)
                            as std::sync::Arc<dyn otap_df_engine::control::PipelineAdminSender>
                    })
                    .collect();

                otap_df_admin::run(
                    admin_settings,
                    obs_state_handle,
                    admin_senders,
                    telemetry_registry,
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
                    obs_evt_reporter.report(ObservedEvent::drained(pipeline_key, None));
                }
                Ok(Err(e)) => {
                    let err_summary: ErrorSummary = error_summary_from_gen(&e);
                    obs_evt_reporter.report(ObservedEvent::pipeline_runtime_error(
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
                    obs_evt_reporter.report(ObservedEvent::pipeline_runtime_error(
                        pipeline_key.clone(),
                        "The pipeline panicked during execution.",
                        err_summary,
                    ));
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

        // Wait for internal pipeline thread if it was spawned
        if let Some((_thread_name, handle)) = internal_pipeline_thread {
            let internal_pipeline_id: PipelineId = "internal".into();
            let pipeline_key = DeployedPipelineKey {
                pipeline_group_id: pipeline_group_id.clone(),
                pipeline_id: internal_pipeline_id,
                core_id: 0, // Virtual core ID for internal pipeline
            };
            match handle.join() {
                Ok(Ok(_)) => {
                    obs_evt_reporter.report(ObservedEvent::drained(pipeline_key, None));
                }
                Ok(Err(e)) => {
                    let err_summary: ErrorSummary = error_summary_from_gen(&e);
                    obs_evt_reporter.report(ObservedEvent::pipeline_runtime_error(
                        pipeline_key.clone(),
                        "Internal pipeline encountered a runtime error.",
                        err_summary,
                    ));
                    // Log but don't fail - internal pipeline errors shouldn't bring down main
                    otel_warn!(
                        "InternalPipeline.Error",
                        message = "Internal telemetry pipeline failed",
                        error = format!("{e:?}")
                    );
                }
                Err(e) => {
                    otel_warn!(
                        "InternalPipeline.Panic",
                        message = "Internal telemetry pipeline panicked",
                        panic_message = format!("{e:?}")
                    );
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
        if let Some(handle) = metrics_dispatcher_handle {
            handle.shutdown_and_join()?;
        }
        obs_state_join_handle.shutdown_and_join()?;
        telemetry_settings.shutdown()?;

        Ok(())
    }

    /// Selects which CPU cores to use based on the given quota configuration.
    fn select_cores_for_quota(
        mut available_core_ids: Vec<CoreId>,
        quota: Quota,
    ) -> Result<Vec<CoreId>, Error> {
        available_core_ids.sort_by_key(|c| c.id);

        let max_core_id = available_core_ids.iter().map(|c| c.id).max().unwrap_or(0);
        let num_cores = available_core_ids.len();

        match quota.core_allocation {
            CoreAllocation::AllCores => Ok(available_core_ids),
            CoreAllocation::CoreCount { count } => {
                if count == 0 {
                    Ok(available_core_ids)
                } else if count > num_cores {
                    Err(Error::InvalidCoreAllocation {
                        alloc: quota.core_allocation.clone(),
                        message: format!(
                            "Requested {} cores but only {} cores available on this system",
                            count, num_cores
                        ),
                        available: available_core_ids.iter().map(|c| c.id).collect(),
                    })
                } else {
                    Ok(available_core_ids.into_iter().take(count).collect())
                }
            }
            CoreAllocation::CoreSet { ref set } => {
                // Validate all ranges first
                for r in set.iter() {
                    if r.start > r.end {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: quota.core_allocation.clone(),
                            message: format!(
                                "Invalid core range: start ({}) is greater than end ({})",
                                r.start, r.end
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                    if r.start > max_core_id {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: quota.core_allocation.clone(),
                            message: format!(
                                "Core ID {} exceeds available cores (system has cores 0-{})",
                                r.start, max_core_id
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                    if r.end > max_core_id {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: quota.core_allocation.clone(),
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
                                alloc: quota.core_allocation.clone(),
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
                        alloc: quota.core_allocation.clone(),
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

    /// Runs a single pipeline in the current thread.
    fn run_pipeline_thread(
        pipeline_key: DeployedPipelineKey,
        core_id: CoreId,
        pipeline_config: PipelineConfig,
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_context: PipelineContext,
        obs_evt_reporter: ObservedEventReporter,
        metrics_reporter: MetricsReporter,
        engine_logs_setup: EngineLogsSetup,
        log_level: otap_df_config::pipeline::service::telemetry::logs::LogLevel,
        logs_receiver: Option<otap_df_telemetry::LogsReceiver>,
        pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<PData>,
        pipeline_ctrl_msg_rx: PipelineCtrlMsgReceiver<PData>,
    ) -> Result<Vec<()>, Error> {
        // Run with the engine-appropriate tracing subscriber.
        // The closure receives a LogsFlusher for buffered mode.
        engine_logs_setup.with_engine_subscriber(log_level, || {
            // Create a tracing span for this pipeline thread
            // so that all logs within this scope include pipeline context.
            let span = otel_info_span!("pipeline_thread", core.id = core_id.id);
            let _guard = span.enter();

            // Pin thread to specific core
            if !core_affinity::set_for_current(core_id) {
                // Continue execution even if pinning fails.
                // This is acceptable because the OS will still schedule the thread, but performance may be less predictable.
                otel_warn!(
                    "CoreAffinity.SetFailed",
                    message = "Failed to set core affinity for pipeline thread. Performance may be less predictable."
                );
            }

            obs_evt_reporter.report(ObservedEvent::admitted(
                pipeline_key.clone(),
                Some("Pipeline admission successful.".to_owned()),
            ));

            // Build the runtime pipeline from the configuration
            // Pass logs_receiver for injection into ITR node (if present)
            let logs_receiver_param = logs_receiver
                .map(|rx| (INTERNAL_TELEMETRY_RECEIVER_URN, rx));
            let runtime_pipeline = pipeline_factory
                .build(pipeline_context.clone(), pipeline_config.clone(), logs_receiver_param)
                .map_err(|e| Error::PipelineRuntimeError {
                    source: Box::new(e),
                })?;

            obs_evt_reporter.report(ObservedEvent::ready(
                pipeline_key.clone(),
                Some("Pipeline initialization successful.".to_owned()),
            ));

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
    use otap_df_config::pipeline_group::CoreRange;

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
            core_allocation: CoreAllocation::CoreSet {
                set: vec![CoreRange {
                    start: first_id,
                    end: first_id,
                }],
            },
        };
        let result = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap();
        assert_eq!(to_ids(&result), vec![first_id]);
    }

    #[test]
    fn select_with_valid_multi_core_range() {
        let quota = Quota {
            core_allocation: CoreAllocation::CoreSet {
                set: vec![
                    CoreRange { start: 2, end: 5 },
                    CoreRange { start: 6, end: 6 },
                ],
            },
        };
        let available_core_ids = available_core_ids();
        let result = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap();
        assert_eq!(to_ids(&result), vec![2, 3, 4, 5, 6]);
    }

    #[test]
    fn select_with_inverted_range_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange { start: 2, end: 1 }],
        };
        let quota = Quota {
            core_allocation: core_allocation.clone(),
        };
        let available_core_ids = available_core_ids();
        let err = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap_err();
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
        let quota = Quota {
            core_allocation: core_allocation.clone(),
        };
        let available_core_ids = available_core_ids();
        let err = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, .. } => {
                assert_eq!(alloc, core_allocation);
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

    #[test]
    fn select_with_overlapping_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 5 },
                CoreRange { start: 4, end: 7 },
            ],
        };
        let quota = Quota {
            core_allocation: core_allocation.clone(),
        };
        let available_core_ids = available_core_ids();
        let err = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap_err();
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
        let quota = Quota {
            core_allocation: core_allocation.clone(),
        };
        let available_core_ids = available_core_ids();
        let err = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap_err();
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
        let quota = Quota {
            core_allocation: core_allocation.clone(),
        };
        let available_core_ids = available_core_ids();
        let err = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap_err();
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
        let quota = Quota {
            core_allocation: CoreAllocation::CoreSet {
                set: vec![
                    CoreRange { start: 2, end: 3 },
                    CoreRange { start: 4, end: 5 },
                ],
            },
        };
        let available_core_ids = available_core_ids();
        let result = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap();
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
        let quota = Quota {
            core_allocation: core_allocation.clone(),
        };
        let available_core_ids = available_core_ids();
        let err = Controller::<()>::select_cores_for_quota(available_core_ids, quota).unwrap_err();
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
