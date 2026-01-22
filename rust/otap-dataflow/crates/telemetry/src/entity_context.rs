// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Thread/task-local storage for entity keys used by our internal telemetry instrumentation
//! to associate metrics and events with the correct pipeline entity, node entity or the correct
//! input/output channel entities.

use crate::metrics::{MetricSet, MetricSetHandler};
use crate::registry::{EntityKey, MetricSetKey, TelemetryRegistryHandle};
use otap_df_config::PortName;
use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use std::future::Future;
use std::rc::Rc;

thread_local! {
    /// Defined when building a pipeline to associate telemetry with the pipeline entity.
    /// Present for the pipeline lifetime.
    static PIPELINE_ENTITY_KEY: Cell<Option<EntityKey>> = const { Cell::new(None) };
    /// Set for each node being built to provide telemetry handle during construction.
    static BUILD_NODE_TELEMETRY_HANDLE: RefCell<Option<NodeTelemetryHandle>> = const { RefCell::new(None) };
}

/// RAII guard that clears and unregisters the pipeline entity on drop.
pub struct PipelineEntityScope {
    registry: TelemetryRegistryHandle,
    key: EntityKey,
}

impl Drop for PipelineEntityScope {
    fn drop(&mut self) {
        PIPELINE_ENTITY_KEY.with(|cell| cell.set(None));
        let _ = self.registry.unregister_entity(self.key);
    }
}

/// Sets the pipeline entity key for the current thread.
#[must_use]
pub fn set_pipeline_entity_key(
    registry: TelemetryRegistryHandle,
    key: EntityKey,
) -> PipelineEntityScope {
    PIPELINE_ENTITY_KEY.with(|cell| cell.set(Some(key)));
    PipelineEntityScope { registry, key }
}

/// Returns the pipeline entity key for the current thread, if set.
#[must_use]
pub fn pipeline_entity_key() -> Option<EntityKey> {
    PIPELINE_ENTITY_KEY.with(Cell::get)
}

tokio::task_local! {
    /// Task-local context for nodes, including entity keys and telemetry handle.
    /// This task local can be used by our internal telemetry instrumentation to associate metrics
    /// and events with the correct node entity or the correct input/output channels.
    static NODE_TASK_CONTEXT: NodeTaskContext;
}

/// Per-task snapshot for fast lookups.
/// Separate from NodeTelemetryState to avoid RefCell borrows on hot paths during node execution.
#[derive(Debug)]
pub struct NodeTaskContext {
    telemetry_handle: Option<NodeTelemetryHandle>,

    /// Entities associated with this node task.
    entity_key: Option<EntityKey>,
    input_channel_key: Option<EntityKey>,
    output_channel_keys: Vec<(PortName, EntityKey)>,
}

impl NodeTaskContext {
    /// Task-local context for a node in the pipeline.
    #[must_use]
    pub fn new(
        entity_key: Option<EntityKey>,
        telemetry_handle: Option<NodeTelemetryHandle>,
        input_channel_key: Option<EntityKey>,
        output_channel_keys: Vec<(PortName, EntityKey)>,
    ) -> Self {
        Self {
            entity_key,
            telemetry_handle,
            input_channel_key,
            output_channel_keys,
        }
    }
}

/// Returns the node entity key for the current task, if set.
#[inline]
#[must_use]
pub fn node_entity_key() -> Option<EntityKey> {
    NODE_TASK_CONTEXT
        .try_with(|ctx| ctx.entity_key)
        .ok()
        .flatten()
}

/// Returns the input channel entity key for the current task, if set.
#[inline]
#[must_use]
pub fn node_input_channel_key() -> Option<EntityKey> {
    NODE_TASK_CONTEXT
        .try_with(|ctx| ctx.input_channel_key)
        .ok()
        .flatten()
}

/// Returns the output channel entity key for the given port in the current task, if set.
///
/// Implementation detail: This function looks up the output channel keys stored in the
/// NodeTaskContext for the current task, searching for a matching port name and returning
/// the associated EntityKey if found. If the number of output channels becomes large, consider
/// optimizing the storage structure for faster lookups. Right now, a linear search seems a good
/// trade-off given the expected small number of output channels per node.
#[inline]
#[must_use]
pub fn node_output_channel_key(port: &str) -> Option<EntityKey> {
    NODE_TASK_CONTEXT
        .try_with(|ctx| {
            ctx.output_channel_keys
                .iter()
                .find(|(name, _)| name.as_ref() == port)
                .map(|(_, key)| *key)
        })
        .ok()
        .flatten()
}

/// Runs the given future with the provided node task context in task-local storage.
pub fn instrument_with_node_context<F, T>(ctx: NodeTaskContext, fut: F) -> impl Future<Output = T>
where
    F: Future<Output = T>,
{
    NODE_TASK_CONTEXT.scope(ctx, fut)
}

/// Returns the current node telemetry handle, if set.
#[must_use]
pub fn current_node_telemetry_handle() -> Option<NodeTelemetryHandle> {
    // Runtime code uses the task-local context.
    // Build-time code uses the thread-local fallback before node tasks are spawned.
    if let Ok(handle) = NODE_TASK_CONTEXT.try_with(|ctx| ctx.telemetry_handle.clone()) {
        return handle;
    }
    BUILD_NODE_TELEMETRY_HANDLE.with(|cell| cell.borrow().clone())
}

/// Runs a closure with the provided node telemetry handle in thread-local storage.
pub fn with_node_telemetry_handle<T>(handle: NodeTelemetryHandle, f: impl FnOnce() -> T) -> T {
    // Build-time helper: node construction happens before task locals exist, so we scope a
    // thread-local handle to let builders access current_node_telemetry_handle().
    BUILD_NODE_TELEMETRY_HANDLE.with(|cell| {
        let _ = cell.replace(Some(handle));
        let result = f();
        let _ = cell.replace(None);
        result
    })
}

/// TODO why was this pub(crate)?
#[derive(Clone)]
pub struct NodeTelemetryHandle {
    registry: TelemetryRegistryHandle,
    state: Rc<RefCell<NodeTelemetryState>>,
}

impl Debug for NodeTelemetryHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entity_key = self.state.borrow().entity_key;
        f.debug_struct("NodeTelemetryHandle")
            .field("entity_key", &entity_key)
            .finish()
    }
}

// Per-node mutable lifecycle state used for metric/entity tracking and cleanup.
struct NodeTelemetryState {
    entity_key: EntityKey,
    metric_keys: Vec<MetricSetKey>,
    input_channel_key: Option<EntityKey>,
    output_channel_keys: Vec<(PortName, EntityKey)>,
    control_channel_key: Option<EntityKey>,
    cleaned: bool,
}

impl NodeTelemetryHandle {
    /// Create a handle that owns registry access and per-node cleanup state.
    #[must_use]
    pub fn new(registry: TelemetryRegistryHandle, entity_key: EntityKey) -> Self {
        Self {
            registry,
            state: Rc::new(RefCell::new(NodeTelemetryState {
                entity_key,
                metric_keys: Vec::new(),
                input_channel_key: None,
                output_channel_keys: Vec::new(),
                control_channel_key: None,
                cleaned: false,
            })),
        }
    }

    /// Return the node entity key for associating metrics/entities.
    #[must_use]
    pub fn entity_key(&self) -> EntityKey {
        self.state.borrow().entity_key
    }

    /// Register a metric set tied to this node entity and track it for cleanup.
    #[must_use]
    pub fn register_metric_set<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
    ) -> MetricSet<T> {
        let entity_key = self.state.borrow().entity_key;
        let metrics = self
            .registry
            .register_metric_set_for_entity::<T>(entity_key);
        self.track_metric_set(metrics.metric_set_key());
        metrics
    }

    /// Record an externally-created metric set so it can be unregistered on cleanup.
    pub fn track_metric_set(&self, metrics_key: MetricSetKey) {
        self.state.borrow_mut().metric_keys.push(metrics_key);
    }

    /// Associate the inbound channel entity key with this node for task-local scoping.
    pub fn set_input_channel_key(&self, key: EntityKey) {
        let mut state = self.state.borrow_mut();
        debug_assert!(
            state.input_channel_key.is_none(),
            "input channel key already set"
        );
        state.input_channel_key = Some(key);
    }

    /// Associate an output channel entity key keyed by port name.
    pub fn add_output_channel_key(&self, port: PortName, key: EntityKey) {
        let mut state = self.state.borrow_mut();
        if let Some((_, existing_key)) = state
            .output_channel_keys
            .iter_mut()
            .find(|(name, _)| name == &port)
        {
            *existing_key = key;
            return;
        }
        state.output_channel_keys.push((port, key));
    }

    /// Associate the control channel entity key with this node.
    pub fn set_control_channel_key(&self, key: EntityKey) {
        let mut state = self.state.borrow_mut();
        debug_assert!(
            state.control_channel_key.is_none(),
            "control channel key already set"
        );
        state.control_channel_key = Some(key);
    }

    /// Read the input channel entity key for task-local scoping.
    #[must_use]
    pub fn input_channel_key(&self) -> Option<EntityKey> {
        self.state.borrow().input_channel_key
    }

    /// Read output channel entity keys for task-local scoping.
    #[must_use]
    pub fn output_channel_keys(&self) -> Vec<(PortName, EntityKey)> {
        self.state.borrow().output_channel_keys.clone()
    }

    /// Unregister tracked metric sets and entities; safe to call once.
    pub(crate) fn cleanup(&self) {
        let mut state = self.state.borrow_mut();
        if state.cleaned {
            return;
        }
        state.cleaned = true;
        let keys = std::mem::take(&mut state.metric_keys);
        let entity_key = state.entity_key;
        let input_channel_key = state.input_channel_key.take();
        let output_channel_keys = std::mem::take(&mut state.output_channel_keys);
        let control_channel_key = state.control_channel_key.take();
        drop(state);

        for key in keys {
            let _ = self.registry.unregister_metric_set(key);
        }
        if let Some(key) = input_channel_key {
            let _ = self.registry.unregister_entity(key);
        }
        if let Some(key) = control_channel_key {
            let _ = self.registry.unregister_entity(key);
        }
        for (_, key) in output_channel_keys {
            let _ = self.registry.unregister_entity(key);
        }
        let _ = self.registry.unregister_entity(entity_key);
    }
}

/// A guard for node-level telemetry context.
pub struct NodeTelemetryGuard {
    handle: NodeTelemetryHandle,
}

impl NodeTelemetryGuard {
    /// Enter the guarded state.
    #[must_use]
    pub fn new(handle: NodeTelemetryHandle) -> Self {
        Self { handle }
    }

    /// Return the entity key.
    #[must_use]
    pub fn entity_key(&self) -> EntityKey {
        self.handle.entity_key()
    }

    /// Return the handle (for reporting, registration).
    #[must_use]
    pub fn handle(&self) -> NodeTelemetryHandle {
        self.handle.clone()
    }
}

impl Drop for NodeTelemetryGuard {
    fn drop(&mut self) {
        self.handle.cleanup();
    }
}
