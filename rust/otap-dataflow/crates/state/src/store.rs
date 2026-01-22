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
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::self_tracing::{ConsoleWriter, ScopeMode};
use otap_df_telemetry::{otel_error, otel_info};
use parking_lot::RwLock;
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

    /// Telemetry registry for resolving entity keys to attributes.
    #[serde(skip)]
    registry: Arc<RwLock<Option<TelemetryRegistryHandle>>>,

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
        Self::with_registry(config, None)
    }

    /// Creates a new `ObservedStateStore` with a telemetry registry for resolving entity keys.
    #[must_use]
    pub fn with_registry(
        config: &ObservedStateSettings,
        registry: Option<TelemetryRegistryHandle>,
    ) -> Self {
        let (sender, receiver) = flume::bounded::<ObservedEvent>(config.reporting_channel_size);

        Self {
            default_health_policy: HealthPolicy::default(),
            health_policies: Arc::new(Mutex::new(HashMap::new())),
            sender,
            receiver,
            console: ConsoleWriter::color(),
            registry: Arc::new(RwLock::new(registry)),
            pipelines: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Sets the telemetry registry for resolving entity keys to attributes.
    /// This should be called after the telemetry system is created but before
    /// the store is started.
    pub fn set_registry(&self, registry: TelemetryRegistryHandle) {
        *self.registry.write() = Some(registry);
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
                // Print the log record first (without inline scope)
                self.console
                    .print_log_record_with_mode(log.time, &log.record, ScopeMode::Grouped);

                // If we have a registry and entity context, print scope attributes
                if let Some(ref registry) = *self.registry.read() {
                    self.print_scope_from_registry(&log.record, registry);
                }
            }
        }
        Ok(())
    }

    /// Print scope attributes by looking up entity keys in the registry.
    fn print_scope_from_registry(
        &self,
        record: &otap_df_telemetry::self_tracing::LogRecord,
        registry: &TelemetryRegistryHandle,
    ) {
        use std::io::Write;

        if !record.has_entity_context() {
            return;
        }

        // Build scope line with resolved attributes
        let mut scope_parts = Vec::new();

        if let Some(pipeline_key) = record.pipeline_entity_key {
            registry.visit_entity(pipeline_key, |attrs| {
                let desc = attrs.descriptor();
                let values = attrs.attribute_values();
                // Include all non-empty pipeline attributes
                for (i, field) in desc.fields.iter().enumerate() {
                    if let Some(val) = values.get(i) {
                        let s = val.to_string_value();
                        if !s.is_empty() {
                            scope_parts.push(format!("{}={}", field.key, s));
                        }
                    }
                }
            });
        }

        if let Some(node_key) = record.node_entity_key {
            registry.visit_entity(node_key, |attrs| {
                let desc = attrs.descriptor();
                let values = attrs.attribute_values();
                // Include all non-empty node attributes (skip duplicates from pipeline)
                for (i, field) in desc.fields.iter().enumerate() {
                    // Skip fields already in scope_parts (node inherits from pipeline)
                    if scope_parts.iter().any(|p| p.starts_with(&format!("{}=", field.key))) {
                        continue;
                    }
                    if let Some(val) = values.get(i) {
                        let s = val.to_string_value();
                        if !s.is_empty() {
                            scope_parts.push(format!("{}={}", field.key, s));
                        }
                    }
                }
            });
        }

        if !scope_parts.is_empty() {
            // Print scope line (indented, no timestamp/level)
            let scope_line = format!("                                scope [{}]\n", scope_parts.join(", "));
            let _ = std::io::stderr().write_all(scope_line.as_bytes());
        }
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

#[cfg(test)]
mod tests {
    use otap_df_telemetry::attributes::{AttributeSetHandler, AttributeValue};
    use otap_df_telemetry::descriptor::{AttributeField, AttributeValueType, AttributesDescriptor};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    // A mock attribute set that mimics PipelineAttributeSet
    #[derive(Debug)]
    struct MockPipelineAttrs {
        values: Vec<AttributeValue>,
    }

    static PIPELINE_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "pipeline.attrs",
        fields: &[AttributeField {
            key: "pipeline.id",  // Macro converts pipeline_id -> pipeline.id
            r#type: AttributeValueType::String,
            brief: "Pipeline identifier",
        }],
    };

    impl MockPipelineAttrs {
        fn new(pipeline_id: &str) -> Self {
            Self {
                values: vec![AttributeValue::String(pipeline_id.to_string())],
            }
        }
    }

    impl AttributeSetHandler for MockPipelineAttrs {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &PIPELINE_DESCRIPTOR
        }
        fn attribute_values(&self) -> &[AttributeValue] {
            &self.values
        }
    }

    // A mock attribute set that mimics NodeAttributeSet
    #[derive(Debug)]
    struct MockNodeAttrs {
        values: Vec<AttributeValue>,
    }

    static NODE_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "node.attrs",
        fields: &[AttributeField {
            key: "node.id",  // Macro converts node_id -> node.id
            r#type: AttributeValueType::String,
            brief: "Node identifier",
        }],
    };

    impl MockNodeAttrs {
        fn new(node_id: &str) -> Self {
            Self {
                values: vec![AttributeValue::String(node_id.to_string())],
            }
        }
    }

    impl AttributeSetHandler for MockNodeAttrs {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &NODE_DESCRIPTOR
        }
        fn attribute_values(&self) -> &[AttributeValue] {
            &self.values
        }
    }

    #[test]
    fn test_entity_key_lookup() {
        // Create a registry and register some entities
        let registry = TelemetryRegistryHandle::new();

        // Register a pipeline entity
        let pipeline_key = registry.register_entity(MockPipelineAttrs::new("my-pipeline"));

        // Register a node entity
        let node_key = registry.register_entity(MockNodeAttrs::new("my-node"));

        // Verify we can look them up
        let mut found_pipeline_id = None;
        registry.visit_entity(pipeline_key, |attrs| {
            let desc = attrs.descriptor();
            let values = attrs.attribute_values();
            for (i, field) in desc.fields.iter().enumerate() {
                if field.key == "pipeline.id" {
                    if let Some(val) = values.get(i) {
                        found_pipeline_id = Some(val.to_string_value());
                    }
                }
            }
        });
        assert_eq!(found_pipeline_id, Some("my-pipeline".to_string()));

        let mut found_node_id = None;
        registry.visit_entity(node_key, |attrs| {
            let desc = attrs.descriptor();
            let values = attrs.attribute_values();
            for (i, field) in desc.fields.iter().enumerate() {
                if field.key == "node.id" {
                    if let Some(val) = values.get(i) {
                        found_node_id = Some(val.to_string_value());
                    }
                }
            }
        });
        assert_eq!(found_node_id, Some("my-node".to_string()));
    }
}
