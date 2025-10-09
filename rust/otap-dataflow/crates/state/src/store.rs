// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of structs defining an event-driven observed state store.

use crate::PipelineKey;
use crate::error::Error;
use crate::event::{EventType, ObservedEvent, ObservedEventRingBuffer};
use crate::phase::PipelinePhase;
use crate::pipeline_rt_status::{ApplyOutcome, PipelineRuntimeStatus};
use crate::pipeline_status::PipelineStatus;
use crate::reporter::ObservedEventReporter;
use otap_df_config::pipeline::PipelineSettings;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
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
    config: PipelineSettings,

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

impl ObservedStateHandle {
    /// Returns a cloned snapshot of the current pipeline statuses.
    #[must_use]
    pub fn snapshot(&self) -> HashMap<PipelineKey, PipelineStatus> {
        match self.pipelines.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                log::warn!(
                    "ObservedStateHandle mutex was poisoned; returning possibly stale snapshot"
                );
                poisoned.into_inner().clone()
            }
        }
    }
}

pub(crate) fn ts_to_rfc3339<S>(t: &SystemTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let dt: chrono::DateTime<chrono::Utc> = (*t).into();
    s.serialize_str(&dt.to_rfc3339())
}

impl ObservedStateStore {
    /// Creates a new `ObservedStateStore` with the given configuration.
    #[must_use]
    pub fn new(config: &PipelineSettings) -> Self {
        let (sender, receiver) =
            flume::bounded::<ObservedEvent>(config.observed_state.reporting_channel_size);

        Self {
            config: config.clone(),
            sender,
            receiver,
            pipelines: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns a reporter that can be used to send observed events to this store.
    #[must_use]
    pub fn reporter(&self) -> ObservedEventReporter {
        ObservedEventReporter::new(
            self.config.observed_state.reporting_timeout,
            self.sender.clone(),
        )
    }

    /// Returns a handle that can be used to read the current observed state.
    #[must_use]
    pub fn handle(&self) -> ObservedStateHandle {
        ObservedStateHandle {
            pipelines: self.pipelines.clone(),
        }
    }

    /// Reports a new observed event in the store.
    #[allow(
        clippy::print_stderr,
        reason = "Use `eprintln!` while waiting for https://github.com/open-telemetry/otel-arrow/issues/1237."
    )]
    fn report(&self, observed_event: ObservedEvent) -> Result<ApplyOutcome, Error> {
        // ToDo Event reporting see: https://github.com/open-telemetry/otel-arrow/issues/1237
        // The code below is temporary and should be replaced with a proper event reporting
        // mechanism (see previous todo).
        match &observed_event.r#type {
            EventType::Request(_) | EventType::Error(_) => {
                eprintln!("Observed event: {observed_event:?}")
            }
            EventType::Success(_) => { /* no console output for success events */ }
        }

        let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
            log::warn!(
                "ObservedStateStore mutex was poisoned; continuing with possibly inconsistent state"
            );
            poisoned.into_inner()
        });
        let pipeline_key = PipelineKey {
            pipeline_group_id: observed_event.key.pipeline_group_id.clone(),
            pipeline_id: observed_event.key.pipeline_id.clone(),
        };

        let ps = pipelines
            .entry(pipeline_key)
            .or_insert_with(|| PipelineStatus::new(self.config.health_policy.clone()));

        // Upsert the core record and its condition snapshot
        let cs = ps
            .cores
            .entry(observed_event.key.core_id)
            .or_insert(PipelineRuntimeStatus {
                phase: PipelinePhase::Pending,
                last_beat: observed_event.time,
                recent_events: ObservedEventRingBuffer::new(RECENT_EVENTS_CAPACITY),
                delete_pending: false,
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
                        log::error!("Error reporting observed event: {e}");
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
