// SPDX-License-Identifier: Apache-2.0

//! Context providing general information on the current controller and the current pipeline.

use crate::attributes::{EngineAttributeSet, NodeAttributeSet, PipelineAttributeSet};
use otap_df_config::node::NodeKind;
use otap_df_config::{NodeId, PipelineGroupId, PipelineId};
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::registry::MetricsRegistryHandle;
use std::fmt::Debug;

// Generate a stable, unique identifier per process instance (base32-encoded UUID v7)
// Choose UUID v7 for better sortability in telemetry signals
use data_encoding::BASE32_NOPAD;
use std::borrow::Cow;
use std::sync::LazyLock;
use uuid::Uuid;

static PROCESS_INSTANCE_ID: LazyLock<Cow<'static, str>> = LazyLock::new(|| {
    let uuid = Uuid::now_v7();
    let encoded = BASE32_NOPAD.encode(uuid.as_bytes());
    Cow::Owned(encoded)
});

// Best-effort host id detection
fn detect_host_id() -> Option<String> {
    // Priority 1: HOSTNAME env var
    if let Ok(h) = std::env::var("HOSTNAME") {
        if !h.is_empty() {
            return Some(h);
        }
    }
    // Priority 2: /etc/hostname
    if let Ok(s) = std::fs::read_to_string("/etc/hostname") {
        let h = s.trim().to_string();
        if !h.is_empty() {
            return Some(h);
        }
    }
    None
}

// Best-effort container id detection (Docker/containerd/k8s) from /proc/self/cgroup
fn detect_container_id() -> Option<String> {
    let Ok(cg) = std::fs::read_to_string("/proc/self/cgroup") else {
        return None;
    };
    // Look for 64-hex tokens which commonly represent container IDs
    for line in cg.lines() {
        // Format: hierarchy-ID:controller-list:cgroup-path
        let path = line.split(':').nth(2).unwrap_or("");
        for part in path.split('/') {
            let token = part.trim();
            if token.len() >= 32 && token.len() <= 128 {
                // Heuristic: mostly hex
                if token
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() || c == '.' || c == '-' || c == '_')
                {
                    // Pick the longest plausible hex-ish token
                    // Further refine: prefer 64-hex
                    let hex_only: String =
                        token.chars().filter(|c| c.is_ascii_hexdigit()).collect();
                    if hex_only.len() >= 32 {
                        return Some(token.to_string());
                    }
                }
            }
        }
    }
    None
}

static HOST_ID: LazyLock<Cow<'static, str>> =
    LazyLock::new(|| detect_host_id().map_or(Cow::Borrowed(""), Cow::Owned));

static CONTAINER_ID: LazyLock<Cow<'static, str>> =
    LazyLock::new(|| detect_container_id().map_or(Cow::Borrowed(""), Cow::Owned));

/// A lightweight/cloneable controller context.
#[derive(Clone)]
pub struct ControllerContext {
    metrics_registry_handle: MetricsRegistryHandle,
    process_instance_id: Cow<'static, str>,
    host_id: Cow<'static, str>,
    container_id: Cow<'static, str>,
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
            process_instance_id: PROCESS_INSTANCE_ID.clone(),
            host_id: HOST_ID.clone(),
            container_id: CONTAINER_ID.clone(),
            numa_node_id: 0, // ToDo(LQ): Set NUMA node ID if available
        }
    }

    /// Returns a new pipeline context with the given identifiers and the current controller context
    /// as the parent context.
    #[must_use]
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
    #[must_use]
    pub fn register_metrics<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
    ) -> MetricSet<T> {
        use crate::attributes::ResourceAttributeSet;

        self.controller_context
            .metrics_registry_handle
            .register::<T>(NodeAttributeSet {
                pipeline_attrs: PipelineAttributeSet {
                    engine_attrs: EngineAttributeSet {
                        resource_attrs: ResourceAttributeSet {
                            process_instance_id: self
                                .controller_context
                                .process_instance_id
                                .clone(),
                            host_id: self.controller_context.host_id.clone(),
                            container_id: self.controller_context.container_id.clone(),
                        },
                        core_id: self.core_id,
                        numa_node_id: self.controller_context.numa_node_id,
                    },
                    pipeline_id: self.pipeline_id.clone(),
                },
                node_id: self.node_id.clone(),
                node_type: self.node_kind.into(),
            })
    }

    /// Returns a metrics registry handle.
    #[must_use]
    pub fn metrics_registry(&self) -> MetricsRegistryHandle {
        self.controller_context.metrics_registry_handle.clone()
    }

    /// Returns a new pipeline context with the given node identifiers.
    #[must_use]
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
