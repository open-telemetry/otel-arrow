// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Thread/task-local storage for entity keys used by telemetry instrumentation.

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
    static NODE_ENTITY_KEY: EntityKey;
}
tokio::task_local! {
    static NODE_TELEMETRY_HANDLE: NodeTelemetryHandle;
}

/// Returns the node entity key for the current task, if set.
#[must_use]
pub fn node_entity_key() -> Option<EntityKey> {
    NODE_ENTITY_KEY.try_with(|key| *key).ok()
}

/// Runs the given future with the provided node entity key in task-local storage.
pub fn instrument_with_node_entity_key<F, T>(key: EntityKey, fut: F) -> impl Future<Output = T>
where
    F: Future<Output = T>,
{
    NODE_ENTITY_KEY.scope(key, fut)
}

/// Runs the given future with the provided node telemetry handle in task-local storage.
pub(crate) fn instrument_with_node_telemetry_handle<F, T>(
    handle: NodeTelemetryHandle,
    fut: F,
) -> impl Future<Output = T>
where
    F: Future<Output = T>,
{
    NODE_TELEMETRY_HANDLE.scope(handle, fut)
}

/// Returns the current node telemetry handle, if set.
pub(crate) fn current_node_telemetry_handle() -> Option<NodeTelemetryHandle> {
    if let Ok(handle) = NODE_TELEMETRY_HANDLE.try_with(|handle| handle.clone()) {
        return Some(handle);
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
    cleaned: bool,
}

impl NodeTelemetryHandle {
    pub(crate) fn new(registry: TelemetryRegistryHandle, entity_key: EntityKey) -> Self {
        Self {
            registry,
            state: Rc::new(RefCell::new(NodeTelemetryState {
                entity_key,
                metric_keys: Vec::new(),
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

    pub(crate) fn cleanup(&self) {
        let mut state = self.state.borrow_mut();
        if state.cleaned {
            return;
        }
        state.cleaned = true;
        let keys = std::mem::take(&mut state.metric_keys);
        let entity_key = state.entity_key;
        drop(state);

        for key in keys {
            let _ = self.registry.unregister_metric_set(key);
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
