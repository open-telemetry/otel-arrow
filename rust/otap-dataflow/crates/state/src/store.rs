// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of structs defining an event-driven observed state store.

use crate::ObservedEventRingBuffer;
use crate::error::Error;
use crate::phase::PipelinePhase;
use crate::pipeline_rt_status::{ApplyOutcome, PipelineRuntimeStatus};
use crate::pipeline_status::PipelineStatus;
use otap_df_config::PipelineKey;
use otap_df_config::health::HealthPolicy;
use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_telemetry::event::{EngineEvent, EventType, ObservedEvent, ObservedEventReporter};
use otap_df_telemetry::self_tracing::ConsoleWriter;
use otap_df_telemetry::{otel_error, otel_info};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

const RECENT_EVENTS_CAPACITY: usize = 10;

/// Event-driven observed state store representing what we know about the state of the
/// pipelines (DAG executors) controlled by the controller.
///
/// This store is cloneable and thread-safe, allowing multiple threads to record events
/// concurrently or read the current state.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedStateStore {
    #[serde(skip)]
    default_health_policy: HealthPolicy,

    #[serde(skip)]
    health_policies: Arc<Mutex<HashMap<PipelineKey, HealthPolicy>>>,

    #[serde(skip)]
    sender: flume::Sender<ObservedEvent>,

    #[serde(skip)]
    receiver: flume::Receiver<ObservedEvent>,

    /// Console is used only for Log events when this component acts
    /// as the ConsoleAsync consumer and logs to the console.
    #[serde(skip)]
    console: ConsoleWriter,

    pipelines: Arc<Mutex<HashMap<PipelineKey, PipelineStatus>>>,
}

/// A handle to the observed state, suitable for serialization and external consumption.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedStateHandle {
    pipelines: Arc<Mutex<HashMap<PipelineKey, PipelineStatus>>>,
}

impl ObservedStateHandle {
    /// Returns a cloned snapshot of the current pipeline statuses.
    #[must_use]
    pub fn snapshot(&self) -> HashMap<PipelineKey, PipelineStatus> {
        match self.pipelines.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                otel_error!(
                    "state.mutex_poisoned",
                    action = "continuing with stale snapshot"
                );
                poisoned.into_inner().clone()
            }
        }
    }
}

impl ObservedStateStore {
    /// Creates a new `ObservedStateStore` with the given configuration.
    #[must_use]
    pub fn new(config: &ObservedStateSettings) -> Self {
        let (sender, receiver) = flume::bounded::<ObservedEvent>(config.reporting_channel_size);

        Self {
            default_health_policy: HealthPolicy::default(),
            health_policies: Arc::new(Mutex::new(HashMap::new())),
            sender,
            receiver,
            console: ConsoleWriter::color(),
            pipelines: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns a reporter that can be used to send observed events to this store.
    #[must_use]
    pub fn reporter(&self, policy: SendPolicy) -> ObservedEventReporter {
        ObservedEventReporter::new(policy, self.sender.clone())
    }

    /// Registers or updates the health policy for a specific pipeline.
    pub fn register_pipeline_health_policy(
        &self,
        pipeline_key: PipelineKey,
        health_policy: HealthPolicy,
    ) {
        let mut policies = self.health_policies.lock().unwrap_or_else(|poisoned| {
            otel_error!(
                "state.mutex_poisoned",
                action = "continuing with stale health policy"
            );
            poisoned.into_inner()
        });
        _ = policies.insert(pipeline_key, health_policy);
    }

    /// Returns a handle that can be used to read the current observed state.
    #[must_use]
    pub fn handle(&self) -> ObservedStateHandle {
        ObservedStateHandle {
            pipelines: self.pipelines.clone(),
        }
    }

    /// Reports a new observed event in the store.
    fn report(&self, observed_event: ObservedEvent) -> Result<(), Error> {
        match observed_event {
            ObservedEvent::Engine(engine) => {
                let _ = self.report_engine(engine)?;
            }
            ObservedEvent::Log(log) => {
                self.console.print_log_record(log.time, &log.record, |_| {});
            }
        }
        Ok(())
    }

    /// Reports a new observed event in the store.
    fn report_engine(&self, observed_event: EngineEvent) -> Result<ApplyOutcome, Error> {
        match &observed_event.r#type {
            EventType::Request(_) => {
                otel_info!("state.observed_event", observed_event = ?observed_event);
            }
            EventType::Error(_) => {
                otel_error!("state.observed_error", observed_event = ?observed_event);
            }
            EventType::Success(_) => {}
        };

        // Events without a pipeline key don't update state.
        let key = &observed_event.key;

        let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
            otel_error!(
                "state.mutex_poisoned",
                action = "continuing with possibly inconsistent state"
            );
            poisoned.into_inner()
        });
        let pipeline_key = PipelineKey::new(key.pipeline_group_id.clone(), key.pipeline_id.clone());

        let health_policy = self
            .health_policies
            .lock()
            .ok()
            .and_then(|policies| policies.get(&pipeline_key).cloned())
            .unwrap_or_else(|| self.default_health_policy.clone());
        let ps = pipelines
            .entry(pipeline_key)
            .or_insert_with(|| PipelineStatus::new(health_policy));

        // Upsert the core record and its condition snapshot
        let cs = ps
            .cores
            .entry(key.core_id)
            .or_insert_with(|| PipelineRuntimeStatus {
                phase: PipelinePhase::Pending,
                last_heartbeat_time: observed_event.time,
                recent_events: ObservedEventRingBuffer::new(RECENT_EVENTS_CAPACITY),
                ..Default::default()
            });
        cs.apply_event(observed_event)
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
                    if let Err(e) = self.report(event) {
                        otel_error!("state.report_failed", error = ?e);
                    }
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
    #[must_use]
    pub fn pipeline_status(&self, pipeline_key: &PipelineKey) -> Option<PipelineStatus> {
        let pipelines = self.pipelines.lock().ok()?;
        pipelines.get(pipeline_key).cloned()
    }

    /// Checks if a pipeline is considered live based on its observed status.
    #[must_use]
    pub fn liveness(&self, pipeline_key: &PipelineKey) -> bool {
        self.pipelines
            .lock()
            .ok()
            .is_some_and(|pipelines| pipelines.get(pipeline_key).is_some_and(|ps| ps.liveness()))
    }

    /// Checks if a pipeline is considered ready based on its observed status.
    #[must_use]
    pub fn readiness(&self, pipeline_key: &PipelineKey) -> bool {
        self.pipelines
            .lock()
            .ok()
            .is_some_and(|pipelines| pipelines.get(pipeline_key).is_some_and(|ps| ps.readiness()))
    }
}
