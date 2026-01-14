// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Thread/task-local storage for entity keys used by telemetry instrumentation.

use otap_df_config::PortName;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::registry::{EntityKey, MetricSetKey, TelemetryRegistryHandle};
use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use std::future::Future;
use std::rc::Rc;

thread_local! {
    static PIPELINE_ENTITY_KEY: Cell<Option<EntityKey>> = Cell::new(None);
    static BUILD_NODE_TELEMETRY_HANDLE: RefCell<Option<NodeTelemetryHandle>> = RefCell::new(None);
}

/// RAII guard that clears the pipeline entity key on drop.
pub struct PipelineEntityScope {}

impl Drop for PipelineEntityScope {
    fn drop(&mut self) {
        PIPELINE_ENTITY_KEY.with(|cell| cell.set(None));
    }
}

/// Sets the pipeline entity key for the current thread.
#[must_use]
pub fn set_pipeline_entity_key(key: EntityKey) -> PipelineEntityScope {
    PIPELINE_ENTITY_KEY.with(|cell| cell.set(Some(key)));
    PipelineEntityScope {}
}

/// Returns the pipeline entity key for the current thread, if set.
#[must_use]
pub fn pipeline_entity_key() -> Option<EntityKey> {
    PIPELINE_ENTITY_KEY.with(Cell::get)
}

tokio::task_local! {
    static NODE_TASK_CONTEXT: NodeTaskContext;
}

#[derive(Debug)]
pub(crate) struct NodeTaskContext {
    entity_key: Option<EntityKey>,
    telemetry_handle: Option<NodeTelemetryHandle>,
    input_channel_key: Option<EntityKey>,
    output_channel_keys: Vec<(PortName, EntityKey)>,
}

impl NodeTaskContext {
    pub(crate) fn new(
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
#[must_use]
pub fn node_entity_key() -> Option<EntityKey> {
    NODE_TASK_CONTEXT
        .try_with(|ctx| ctx.entity_key)
        .ok()
        .flatten()
}

/// Returns the input channel entity key for the current task, if set.
#[must_use]
pub fn node_input_channel_key() -> Option<EntityKey> {
    NODE_TASK_CONTEXT
        .try_with(|ctx| ctx.input_channel_key)
        .ok()
        .flatten()
}

/// Returns the output channel entity key for the given port in the current task, if set.
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
pub(crate) fn instrument_with_node_context<F, T>(
    ctx: NodeTaskContext,
    fut: F,
) -> impl Future<Output = T>
where
    F: Future<Output = T>,
{
    NODE_TASK_CONTEXT.scope(ctx, fut)
}

/// Returns the current node telemetry handle, if set.
pub(crate) fn current_node_telemetry_handle() -> Option<NodeTelemetryHandle> {
    if let Ok(handle) = NODE_TASK_CONTEXT.try_with(|ctx| ctx.telemetry_handle.clone()) {
        return handle;
    }
    BUILD_NODE_TELEMETRY_HANDLE.with(|cell| cell.borrow().clone())
}

/// Runs a closure with the provided node telemetry handle in thread-local storage.
pub(crate) fn with_node_telemetry_handle<T>(
    handle: NodeTelemetryHandle,
    f: impl FnOnce() -> T,
) -> T {
    BUILD_NODE_TELEMETRY_HANDLE.with(|cell| {
        let _ = cell.replace(Some(handle));
        let result = f();
        let _ = cell.replace(None);
        result
    })
}

#[derive(Clone)]
pub(crate) struct NodeTelemetryHandle {
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

struct NodeTelemetryState {
    entity_key: EntityKey,
    metric_keys: Vec<MetricSetKey>,
    input_channel_key: Option<EntityKey>,
    output_channel_keys: Vec<(PortName, EntityKey)>,
    control_channel_key: Option<EntityKey>,
    cleaned: bool,
}

impl NodeTelemetryHandle {
    pub(crate) fn new(registry: TelemetryRegistryHandle, entity_key: EntityKey) -> Self {
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

    pub(crate) fn entity_key(&self) -> EntityKey {
        self.state.borrow().entity_key
    }

    pub(crate) fn register_metric_set<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
    ) -> MetricSet<T> {
        let entity_key = self.state.borrow().entity_key;
        let metrics = self
            .registry
            .register_metric_set_for_entity::<T>(entity_key);
        self.track_metric_set(metrics.metric_set_key());
        metrics
    }

    pub(crate) fn track_metric_set(&self, metrics_key: MetricSetKey) {
        self.state.borrow_mut().metric_keys.push(metrics_key);
    }

    pub(crate) fn set_input_channel_key(&self, key: EntityKey) {
        let mut state = self.state.borrow_mut();
        debug_assert!(
            state.input_channel_key.is_none(),
            "input channel key already set"
        );
        state.input_channel_key = Some(key);
    }

    pub(crate) fn add_output_channel_key(&self, port: PortName, key: EntityKey) {
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

    pub(crate) fn set_control_channel_key(&self, key: EntityKey) {
        let mut state = self.state.borrow_mut();
        debug_assert!(
            state.control_channel_key.is_none(),
            "control channel key already set"
        );
        state.control_channel_key = Some(key);
    }

    pub(crate) fn input_channel_key(&self) -> Option<EntityKey> {
        self.state.borrow().input_channel_key
    }

    pub(crate) fn output_channel_keys(&self) -> Vec<(PortName, EntityKey)> {
        self.state.borrow().output_channel_keys.clone()
    }

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

#[doc(hidden)]
pub struct NodeTelemetryGuard {
    handle: NodeTelemetryHandle,
}

impl NodeTelemetryGuard {
    pub(crate) fn new(handle: NodeTelemetryHandle) -> Self {
        Self { handle }
    }

    pub(crate) fn entity_key(&self) -> EntityKey {
        self.handle.entity_key()
    }

    pub(crate) fn handle(&self) -> NodeTelemetryHandle {
        self.handle.clone()
    }
}

impl Drop for NodeTelemetryGuard {
    fn drop(&mut self) {
        self.handle.cleanup();
    }
}
