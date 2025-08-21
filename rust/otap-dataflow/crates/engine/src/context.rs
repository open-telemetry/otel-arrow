// SPDX-License-Identifier: Apache-2.0

//! Context providing general information on the current controller and the current pipeline.

use otap_df_config::{NodeId, PipelineGroupId, PipelineId};
use otap_df_telemetry::metrics::{MetricSetHandler, MetricSet};
use otap_df_telemetry::registry::MetricsRegistryHandle;
use std::fmt::Debug;
use otap_df_config::node::NodeKind;
use crate::attributes::PipelineAttributeSet;

/// A lightweight/cloneable controller context.
#[derive(Clone)]
pub struct ControllerContext {
    metrics_registry_handle: MetricsRegistryHandle,
    process_id: u32,
    numa_node_id: usize,
}

/// A lightweight/cloneable pipeline context.
#[derive(Clone)]
pub struct PipelineContext {
    controller_context: ControllerContext,
    core_id: usize,
    thread_id: usize,
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
    node_id: NodeId,
    node_kind: NodeKind,
}

impl ControllerContext {
    /// Creates a new `ControllerContext`.
    pub fn new(metrics_registry_handle: MetricsRegistryHandle) -> Self {
        Self {
            metrics_registry_handle,
            process_id: std::process::id(),
            numa_node_id: 0, // ToDo(LQ): Set NUMA node ID if available
        }
    }

    /// Returns a new pipeline context with the given identifiers and the current controller context
    /// as the parent context.
    pub fn pipeline_context_with(
        &self,
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        core_id: usize,
        thread_id: usize,
    ) -> PipelineContext {
        PipelineContext::new(
            self.clone(),
            pipeline_group_id,
            pipeline_id,
            core_id,
            thread_id,
        )
    }
}

impl PipelineContext {
    /// Creates a new `PipelineContext`.
    fn new(
        parent_ctx: ControllerContext,
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        core_id: usize,
        thread_id: usize,
    ) -> Self {
        Self {
            controller_context: parent_ctx,
            pipeline_id,
            pipeline_group_id,
            core_id,
            thread_id,
            node_id: Default::default(),
            node_kind: Default::default(),
        }
    }

    /// Registers a new multivariate metrics instance with the metrics registry.
    pub fn register_metrics<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self
    ) -> MetricSet<T> {
        use crate::attributes::EngineAttributeSet;

        self.controller_context.metrics_registry_handle.register::<T>(
            PipelineAttributeSet {
                engine: EngineAttributeSet {
                    core_id: self.core_id,
                    numa_node_id: self.controller_context.numa_node_id,
                    process_id: self.controller_context.process_id,
                },
                node_id: self.node_id.clone(),
                node_type: self.node_kind.into(),
                pipeline_id: self.pipeline_id.clone(),
            },
        )
    }

    /// Returns a metrics registry handle.
    pub fn metrics_registry(&self) -> MetricsRegistryHandle {
        self.controller_context.metrics_registry_handle.clone()
    }

    /// Returns a new pipeline context with the given node identifiers.
    pub fn with_node_context(&self, node_id: NodeId, node_kind: NodeKind) -> Self {
        Self {
            controller_context: self.controller_context.clone(),
            core_id: self.core_id,
            thread_id: self.thread_id,
            pipeline_group_id: self.pipeline_group_id.clone(),
            pipeline_id: self.pipeline_id.clone(),
            node_id,
            node_kind,
        }
    }
}
