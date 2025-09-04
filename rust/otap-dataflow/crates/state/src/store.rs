// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of structs defining an event-driven observed state store.

use crate::config::Config;
use crate::error::Error;
use crate::reporter::ObservedEventReporter;
use crate::{CoreId, DeployedPipelineKey, PipelineKey};
use otap_df_config::{PipelineGroupId, PipelineId};
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio_util::sync::CancellationToken;

/// High-level lifecycle of a pipeline as seen by the controller.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub enum PipelinePhase {
    /// The pipeline is pending and has not started yet.
    Pending,
    /// The pipeline is currently running and actively processing telemetry data.
    Running,
    /// A graceful stop has been requested. Ingress is quiescing and in-flight
    /// data is draining, possibly with a deadline.
    Draining,
    /// The pipeline has been stopped.
    Stopped,
    /// Entered a terminal error state (e.g. unrecoverable apply error). The
    /// controller may attempt retries based on policy, but phase reflects the
    /// current failure.
    Failed,
    /// The controller cannot currently determine the state (e.g. missing
    /// heartbeats past the freshness window).
    Unknown,
}

/// Types of events that can be observed from a pipeline engine instance.
#[derive(Debug)]
pub enum ObservedEvent {
    /// A pipeline phase change event.
    PipelinePhase {
        /// Unique key identifying the pipeline instance.
        key: PipelineKey,
        /// Core ID reporting the phase change.
        core_id: CoreId,
        /// New phase of the pipeline instance.
        phase: PipelinePhase,
    },
    /// A periodic heartbeat event indicating the pipeline is alive.
    Heartbeat {
        /// Unique key identifying the pipeline instance.
        key: PipelineKey,
        /// Core ID sending the heartbeat.
        core_id: CoreId,
    },
}

/// The per-core view of a pipeline instance, as reported by an engine.
#[derive(Debug, Serialize, Clone)]
pub struct CoreStatus {
    /// Current phase of the pipeline instance.
    pub phase: PipelinePhase,
    /// Timestamp of the most recent event/heartbeat received for this core.
    #[serde(serialize_with = "ts_to_rfc3339")]
    last_beat: SystemTime,
}

/// Aggregated, controller-synthesized view for a pipeline across all targeted
/// cores. This is what external APIs will return for `status`.
#[derive(Debug, Serialize, Clone)]
pub struct PipelineStatus {
    /// Coarse phase synthesized from all per-core phases.
    phase: PipelinePhase,
    /// Per-core details to aid debugging and aggregation.
    per_core: HashMap<CoreId, CoreStatus>,
}

/// Event-driven observed state store representing what we know about the state of the
/// pipelines (DAG executors) controlled by the controller.
///
/// This store is cloneable and thread-safe, allowing multiple threads to record events
/// concurrently or read the current state.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedStateStore {
    #[serde(skip)]
    config: Config,

    #[serde(skip)]
    sender: flume::Sender<ObservedEvent>,

    #[serde(skip)]
    receiver: flume::Receiver<ObservedEvent>,

    pipelines: Arc<Mutex<HashMap<PipelineKey, PipelineStatus>>>,
}

/// A handle to the observed state, suitable for serialization and external consumption.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedStateHandle {
    pipelines: Arc<Mutex<HashMap<PipelineKey, PipelineStatus>>>,
}

fn ts_to_rfc3339<S>(t: &SystemTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let dt: chrono::DateTime<chrono::Utc> = (*t).into();
    s.serialize_str(&dt.to_rfc3339())
}

impl Default for ObservedStateStore {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl ObservedStateStore {
    /// Creates a new `ObservedStateStore` with the given configuration.
    #[must_use]
    pub fn new(config: Config) -> Self {
        let (sender, receiver) = flume::bounded::<ObservedEvent>(config.reporting_channel_size);

        Self {
            config,
            sender,
            receiver,
            pipelines: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns a reporter that can be used to send observed events to this store.
    #[must_use]
    pub fn reporter(&self) -> ObservedEventReporter {
        ObservedEventReporter::new(self.config.reporting_timeout, self.sender.clone())
    }

    /// Returns a handle that can be used to read the current observed state.
    #[must_use]
    pub fn handle(&self) -> ObservedStateHandle {
        ObservedStateHandle {
            pipelines: self.pipelines.clone(),
        }
    }

    /// Reports a new observed event in the store.
    fn report(&self, observed_event: ObservedEvent) {
        match observed_event {
            ObservedEvent::PipelinePhase {
                key,
                core_id,
                phase,
            } => {
                let mut pipelines = self.pipelines.lock().unwrap(); // todo report error
                let pipeline_status =
                    pipelines
                        .entry(key.clone())
                        .or_insert_with(|| PipelineStatus {
                            phase: PipelinePhase::Pending,
                            per_core: HashMap::new(),
                        });
                let core_status =
                    pipeline_status
                        .per_core
                        .entry(core_id)
                        .or_insert_with(|| CoreStatus {
                            phase: PipelinePhase::Pending,
                            last_beat: SystemTime::now(),
                        });
                core_status.phase = phase;
                core_status.last_beat = SystemTime::now();

                // Update the overall phase based on the new core status
                pipeline_status.phase =
                    aggregate_pipeline_phase(pipeline_status.per_core.values().map(|c| c.phase));
            }
            ObservedEvent::Heartbeat { key, core_id } => {
                let mut pipelines = self.pipelines.lock().unwrap(); // todo report error
                let pipeline_status =
                    pipelines
                        .entry(key.clone())
                        .or_insert_with(|| PipelineStatus {
                            phase: PipelinePhase::Pending,
                            per_core: HashMap::new(),
                        });
                let core_status =
                    pipeline_status
                        .per_core
                        .entry(core_id)
                        .or_insert_with(|| CoreStatus {
                            phase: PipelinePhase::Pending,
                            last_beat: SystemTime::now(),
                        });
                core_status.last_beat = SystemTime::now();
            }
        }
    }

    /// Runs the collection loop, receiving observed events and updating the observed store.
    /// This function runs indefinitely until the channel is closed or the cancellation token is
    /// triggered.
    pub async fn run(self, cancel: CancellationToken) -> Result<(), Error> {
        tokio::select! {
            _ = async {
                // Continuously receive events and report them
                // Exit the loop if the channel is closed
                while let Ok(event) = self.receiver.recv_async().await {
                    self.report(event);
                }
            } => { /* Channel closed, exit gracefully */ }
            _ = cancel.cancelled() => { /* Cancellation requested, exit gracefully */ }
        }
        Ok(())
    }
}

impl ObservedStateHandle {
    /// Retrieves the current status of a pipeline by its key.
    /// Returns a snapshot clone of the status if present.
    pub fn pipeline_status(&self, pipeline_key: &PipelineKey) -> Option<PipelineStatus> {
        let pipelines = self.pipelines.lock().ok()?;
        pipelines.get(pipeline_key).cloned()
    }
}

impl ObservedEvent {
    /// Creates a new `PipelinePhase` event.
    pub fn pipeline_phase_event(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        core_id: CoreId,
        phase: PipelinePhase,
    ) -> Self {
        ObservedEvent::PipelinePhase {
            key: PipelineKey {
                pipeline_group_id,
                pipeline_id,
            },
            core_id,
            phase,
        }
    }

    /// Creates a new `PipelinePhase::Pending` event.
    pub fn pipeline_pending(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Pending,
        )
    }

    /// Creates a new `PipelinePhase::Running` event.
    pub fn pipeline_running(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Running,
        )
    }

    /// Creates a new `PipelinePhase::Draining` event.
    pub fn pipeline_draining(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Draining,
        )
    }

    /// Creates a new `PipelinePhase::Stopped` event.
    pub fn pipeline_stopped(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Stopped,
        )
    }

    /// Creates a new `PipelinePhase::Failed` event.
    pub fn pipeline_failed(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Failed,
        )
    }

    /// Creates a new `PipelinePhase::Unknown` event.
    pub fn pipeline_unknown(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Unknown,
        )
    }

    /// Creates a new `Heartbeat` event.
    pub fn heartbeat_event(pipeline_key: DeployedPipelineKey) -> Self {
        ObservedEvent::Heartbeat {
            key: PipelineKey {
                pipeline_group_id: pipeline_key.pipeline_group_id,
                pipeline_id: pipeline_key.pipeline_id,
            },
            core_id: pipeline_key.core_id,
        }
    }
}

/// Aggregates multiple `PipelinePhase` values into a single representative phase.
/// This function bubble up the most consequential state so operators don't miss failures/drains in
/// progress, and don't show Stopped while some cores are actually running.
///
/// The function applies this priority order:
///
/// - Failed (if any core is Failed)
/// - Draining (if any core is Draining)
/// - Running
///   - Running if all cores are Running, or
///   - Running if any core is Running (even if others are Pending/Stopped/Unknown)
/// - Stopped (only if all cores are Stopped)
/// - Pending (if any core is Pending and none above matched)
/// - Unknown (if any core is Unknown and none above matched, or if there are no cores)
///
/// Examples:
/// - `[Running, Running]` => Running (all running)
/// - `[Running, Stopped]` => Running (some cores still working)
/// - `[Draining, Running]` => Draining (drain in progress)
/// - `[Failed, Running]` => Failed (fail fast)
/// - `[Stopped, Stopped]` => Stopped (fully stopped)
/// - `[Pending, Stopped]` => Pending (bring-up in progress)
/// - `[Unknown, Stopped]` => Unknown (no running/draining/failure and not all stopped)
fn aggregate_pipeline_phase<I>(phases: I) -> PipelinePhase
where
    I: IntoIterator<Item = PipelinePhase>,
{
    use PipelinePhase::*;

    let mut saw_any = false;
    let mut any_failed = false;
    let mut any_draining = false;
    let mut any_running = false;
    let mut any_pending = false;
    let mut any_unknown = false;

    let mut all_running = true;
    let mut all_stopped = true;

    for p in phases {
        saw_any = true;
        match p {
            Failed => {
                any_failed = true;
                all_running = false;
                all_stopped = false;
            }
            Draining => {
                any_draining = true;
                all_running = false;
                all_stopped = false;
            }
            Running => {
                any_running = true;
                all_stopped = false;
            }
            Pending => {
                any_pending = true;
                all_running = false;
                all_stopped = false;
            }
            Stopped => {
                all_running = false;
            }
            Unknown => {
                any_unknown = true;
                all_running = false;
                all_stopped = false;
            }
        }
    }

    if !saw_any {
        return Unknown;
    }
    if any_failed {
        return Failed;
    }
    if any_draining {
        return Draining;
    }
    if all_running {
        return Running;
    }
    if any_running {
        return Running;
    }
    if all_stopped {
        return Stopped;
    }
    if any_pending {
        return Pending;
    }
    if any_unknown {
        return Unknown;
    }
    Unknown
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod tests {
        use crate::store::PipelinePhase::*;
        use crate::store::{PipelinePhase, aggregate_pipeline_phase};

        #[test]
        fn aggregate_pipeline_phase_basics() {
            // empty => Unknown
            assert_eq!(
                aggregate_pipeline_phase(Vec::<PipelinePhase>::new()),
                Unknown
            );

            // all running => Running
            assert_eq!(aggregate_pipeline_phase([Running, Running]), Running);

            // mixed running/stopped => Running
            assert_eq!(aggregate_pipeline_phase([Running, Stopped]), Running);

            // any draining dominates running => Draining
            assert_eq!(aggregate_pipeline_phase([Draining, Running]), Draining);

            // any failed dominates everything => Failed
            assert_eq!(
                aggregate_pipeline_phase([Failed, Running, Draining]),
                Failed
            );

            // all stopped => Stopped
            assert_eq!(aggregate_pipeline_phase([Stopped, Stopped]), Stopped);

            // pending + stopped (no running/draining/failed) => Pending
            assert_eq!(aggregate_pipeline_phase([Pending, Stopped]), Pending);

            // unknown + stopped (no running/draining/failed and not all stopped) => Unknown
            assert_eq!(aggregate_pipeline_phase([Unknown, Stopped]), Unknown);

            // unknown + pending + stopped => Pending wins over Unknown
            assert_eq!(
                aggregate_pipeline_phase([Unknown, Pending, Stopped]),
                Pending
            );
        }
    }
}
