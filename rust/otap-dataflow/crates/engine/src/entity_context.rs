// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Thread/task-local storage for entity keys used by our internal telemetry instrumentation
//! to associate metrics and events with the correct pipeline entity, node entity or the correct
//! input/output channel entities.

use otap_df_config::PortName;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::registry::{EntityKey, MetricSetKey, TelemetryRegistryHandle};
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
pub(crate) struct NodeTaskContext {
    telemetry_handle: Option<NodeTelemetryHandle>,

    /// Entities associated with this node task.
    entity_key: Option<EntityKey>,
    input_channel_key: Option<EntityKey>,
    output_channel_keys: Vec<(PortName, EntityKey)>,
}

impl NodeTaskContext {
    pub(crate) const fn new(
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
    // Runtime code uses the task-local context.
    // Build-time code uses the thread-local fallback before node tasks are spawned.
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
    // Build-time helper: node construction happens before task locals exist, so we scope a
    // thread-local handle to let builders access current_node_telemetry_handle().
    BUILD_NODE_TELEMETRY_HANDLE.with(|cell| {
        let _ = cell.replace(Some(handle));
        let result = f();
        let _ = cell.replace(None);
        result
    })
}

/// Handle for per-node telemetry state, including entity keys, metric sets,
/// channel associations, and custom log record attributes.
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

    /// Return the node entity key for associating metrics/entities.
    pub(crate) fn entity_key(&self) -> EntityKey {
        self.state.borrow().entity_key
    }

    /// Register a metric set tied to this node entity and track it for cleanup.
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

    /// Record an externally-created metric set so it can be unregistered on cleanup.
    pub(crate) fn track_metric_set(&self, metrics_key: MetricSetKey) {
        self.state.borrow_mut().metric_keys.push(metrics_key);
    }

    /// Associate the inbound channel entity key with this node for task-local scoping.
    pub(crate) fn set_input_channel_key(&self, key: EntityKey) {
        let mut state = self.state.borrow_mut();
        debug_assert!(
            state.input_channel_key.is_none(),
            "input channel key already set"
        );
        state.input_channel_key = Some(key);
    }

    /// Associate an output channel entity key keyed by port name.
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

    /// Associate the control channel entity key with this node.
    pub(crate) fn set_control_channel_key(&self, key: EntityKey) {
        let mut state = self.state.borrow_mut();
        debug_assert!(
            state.control_channel_key.is_none(),
            "control channel key already set"
        );
        state.control_channel_key = Some(key);
    }

    /// Read the input channel entity key for task-local scoping.
    pub(crate) fn input_channel_key(&self) -> Option<EntityKey> {
        self.state.borrow().input_channel_key
    }

    /// Read output channel entity keys for task-local scoping.
    pub(crate) fn output_channel_keys(&self) -> Vec<(PortName, EntityKey)> {
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

#[doc(hidden)]
pub struct NodeTelemetryGuard {
    handle: NodeTelemetryHandle,
}

impl NodeTelemetryGuard {
    pub(crate) const fn new(handle: NodeTelemetryHandle) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel_metrics::{
        CHANNEL_IMPL_INTERNAL, CHANNEL_KIND_CONTROL, CHANNEL_KIND_PDATA, CHANNEL_MODE_LOCAL,
        CHANNEL_TYPE_MPSC, ChannelReceiverMetrics, ChannelSenderMetrics,
    };
    use crate::context::ControllerContext;
    use crate::pipeline_metrics::PipelineMetricsMonitor;
    use otap_df_config::node::NodeKind;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use std::borrow::Cow;
    use std::collections::HashMap;

    #[test]
    fn pipeline_cleanup_unregisters_entities() {
        let registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("group".into(), "pipe".into(), 0, 1, 0);

        let pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
        let _pipeline_entity_guard =
            set_pipeline_entity_key(pipeline_ctx.metrics_registry(), pipeline_entity_key);
        let _pipeline_metrics = PipelineMetricsMonitor::new(pipeline_ctx.clone());

        let source_ctx = pipeline_ctx.with_node_context(
            "source".into(),
            "urn:test:example:receiver".into(),
            NodeKind::Receiver,
            HashMap::new(),
        );
        let dest_ctx = pipeline_ctx.with_node_context(
            "dest".into(),
            "urn:test:example:processor".into(),
            NodeKind::Processor,
            HashMap::new(),
        );

        let source_entity_key = source_ctx.register_node_entity();
        let dest_entity_key = dest_ctx.register_node_entity();
        let source_handle =
            NodeTelemetryHandle::new(source_ctx.metrics_registry(), source_entity_key);
        let dest_handle = NodeTelemetryHandle::new(dest_ctx.metrics_registry(), dest_entity_key);
        let source_guard = NodeTelemetryGuard::new(source_handle.clone());
        let dest_guard = NodeTelemetryGuard::new(dest_handle.clone());

        let channel_id: Cow<'static, str> = "chan:pdata".into();
        let out_key = source_ctx.register_channel_entity(
            channel_id.clone(),
            "out".into(),
            CHANNEL_KIND_PDATA,
            CHANNEL_MODE_LOCAL,
            CHANNEL_TYPE_MPSC,
            CHANNEL_IMPL_INTERNAL,
        );
        source_handle.add_output_channel_key("out".into(), out_key);
        let in_key = dest_ctx.register_channel_entity(
            channel_id,
            "input".into(),
            CHANNEL_KIND_PDATA,
            CHANNEL_MODE_LOCAL,
            CHANNEL_TYPE_MPSC,
            CHANNEL_IMPL_INTERNAL,
        );
        dest_handle.set_input_channel_key(in_key);
        let ctrl_key = source_ctx.register_channel_entity(
            "chan:ctrl".into(),
            "input".into(),
            CHANNEL_KIND_CONTROL,
            CHANNEL_MODE_LOCAL,
            CHANNEL_TYPE_MPSC,
            CHANNEL_IMPL_INTERNAL,
        );
        source_handle.set_control_channel_key(ctrl_key);

        let out_metrics =
            source_ctx.register_metric_set_for_entity::<ChannelSenderMetrics>(out_key);
        source_handle.track_metric_set(out_metrics.metric_set_key());
        let in_metrics = dest_ctx.register_metric_set_for_entity::<ChannelReceiverMetrics>(in_key);
        dest_handle.track_metric_set(in_metrics.metric_set_key());
        let _ = source_handle.register_metric_set::<ChannelSenderMetrics>();
        let _ = dest_handle.register_metric_set::<ChannelSenderMetrics>();

        assert_eq!(registry.entity_count(), 6);
        assert_eq!(registry.metric_set_count(), 6);

        drop(dest_guard);
        drop(source_guard);
        drop(_pipeline_metrics);
        drop(_pipeline_entity_guard);

        assert_eq!(registry.metric_set_count(), 0);
        assert_eq!(registry.entity_count(), 0);
    }
}
