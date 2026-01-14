// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Context providing general information on the current controller and the current pipeline.

use crate::attributes::{
    ChannelAttributeSet, EngineAttributeSet, NodeAttributeSet, PipelineAttributeSet,
};
use crate::entity_context::{current_node_telemetry_handle, node_entity_key};
use otap_df_config::node::NodeKind;
use otap_df_config::{NodeId, NodeUrn, PipelineGroupId, PipelineId};
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::registry::{EntityKey, TelemetryRegistryHandle};
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
#[derive(Clone, Debug)]
pub struct ControllerContext {
    telemetry_registry_handle: TelemetryRegistryHandle,
    process_instance_id: Cow<'static, str>,
    host_id: Cow<'static, str>,
    container_id: Cow<'static, str>,
    numa_node_id: usize,
}

/// A lightweight/cloneable pipeline context.
#[derive(Clone, Debug)]
pub struct PipelineContext {
    controller_context: ControllerContext,
    core_id: usize,
    thread_id: usize,
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
    node_id: NodeId,
    node_urn: NodeUrn,
    node_kind: NodeKind,
}

impl ControllerContext {
    /// Creates a new `ControllerContext`.
    pub fn new(telemetry_registry_handle: TelemetryRegistryHandle) -> Self {
        Self {
            telemetry_registry_handle,
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
    pub(crate) fn new(
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
            node_urn: Default::default(),
            node_kind: Default::default(),
        }
    }

    /// Returns the pipeline group ID associated with this pipeline context.
    #[must_use]
    pub fn pipeline_group_id(&self) -> PipelineGroupId {
        self.pipeline_group_id.clone()
    }

    /// Returns the pipeline ID associated with this pipeline context.
    #[must_use]
    pub fn pipeline_id(&self) -> PipelineId {
        self.pipeline_id.clone()
    }

    /// Returns the core ID associated with this pipeline context.
    #[must_use]
    pub fn core_id(&self) -> usize {
        self.core_id
    }

    /// Registers a metric set for the given entity key and tracks it in node telemetry if present.
    #[must_use]
    pub fn register_metric_set_for_entity<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        entity_key: EntityKey,
    ) -> MetricSet<T> {
        let metrics = self
            .controller_context
            .telemetry_registry_handle
            .register_metric_set_for_entity::<T>(entity_key);
        if let Some(telemetry) = current_node_telemetry_handle() {
            telemetry.track_metric_set(metrics.metric_set_key());
        }
        metrics
    }

    /// Registers a metric set for the current node entity.
    #[must_use]
    pub fn register_metrics<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
    ) -> MetricSet<T> {
        if let Some(telemetry) = current_node_telemetry_handle() {
            telemetry.register_metric_set::<T>()
        } else if let Some(entity_key) = node_entity_key() {
            self.controller_context
                .telemetry_registry_handle
                .register_metric_set_for_entity::<T>(entity_key)
        } else {
            // Tests often construct nodes directly without the engine's entity scoping. So the
            // following code path is only enabled for test builds.
            #[cfg(feature = "test-utils")]
            {
                self.controller_context
                    .telemetry_registry_handle
                    .register_metric_set::<T>(self.node_attribute_set())
            }
            #[cfg(not(feature = "test-utils"))]
            {
                panic!(
                    "node entity key not set; ensure node entity is registered and instrumented"
                );
            }
        }
    }

    /// Registers a metric set for the given attributes and tracks it in node telemetry if present.
    #[must_use]
    pub fn register_metric_set_with_attrs<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        attrs: impl otap_df_telemetry::attributes::AttributeSetHandler + Send + Sync + 'static,
    ) -> MetricSet<T> {
        let metrics = self
            .controller_context
            .telemetry_registry_handle
            .register_metric_set::<T>(attrs);
        if let Some(telemetry) = current_node_telemetry_handle() {
            telemetry.track_metric_set(metrics.metric_set_key());
        }
        metrics
    }

    /// Registers (or reuses) the pipeline entity for this context.
    #[must_use]
    pub fn register_pipeline_entity(&self) -> EntityKey {
        self.controller_context
            .telemetry_registry_handle
            .register_entity(self.pipeline_attribute_set())
    }

    /// Registers (or reuses) the node entity for this context.
    #[must_use]
    pub fn register_node_entity(&self) -> EntityKey {
        self.controller_context
            .telemetry_registry_handle
            .register_entity(self.node_attribute_set())
    }

    fn engine_attribute_set(&self) -> EngineAttributeSet {
        use crate::attributes::ResourceAttributeSet;

        EngineAttributeSet {
            resource_attrs: ResourceAttributeSet {
                process_instance_id: self.controller_context.process_instance_id.clone(),
                host_id: self.controller_context.host_id.clone(),
                container_id: self.controller_context.container_id.clone(),
            },
            core_id: self.core_id,
            numa_node_id: self.controller_context.numa_node_id,
        }
    }

    /// Returns the pipeline attribute set for the current pipeline context.
    #[must_use]
    pub fn pipeline_attribute_set(&self) -> PipelineAttributeSet {
        PipelineAttributeSet {
            engine_attrs: self.engine_attribute_set(),
            pipeline_id: self.pipeline_id.clone(),
            pipeline_group_id: self.pipeline_group_id.clone(),
        }
    }

    /// Returns the node attribute set for the current node context.
    #[must_use]
    pub fn node_attribute_set(&self) -> NodeAttributeSet {
        NodeAttributeSet {
            pipeline_attrs: self.pipeline_attribute_set(),
            node_id: self.node_id.clone(),
            node_urn: self.node_urn.clone(),
            node_type: self.node_kind.into(),
        }
    }

    /// Returns a channel attribute set tied to this node context.
    #[must_use]
    pub fn channel_attribute_set(
        &self,
        channel_id: Cow<'static, str>,
        channel_kind: &'static str,
        channel_mode: &'static str,
        channel_type: &'static str,
        channel_impl: &'static str,
    ) -> ChannelAttributeSet {
        ChannelAttributeSet {
            node_attrs: self.node_attribute_set(),
            channel_id,
            channel_kind: Cow::Borrowed(channel_kind),
            channel_mode: Cow::Borrowed(channel_mode),
            channel_type: Cow::Borrowed(channel_type),
            channel_impl: Cow::Borrowed(channel_impl),
        }
    }

    /// Registers (or reuses) a channel entity for the given channel attributes.
    #[must_use]
    pub fn register_channel_entity(
        &self,
        channel_id: Cow<'static, str>,
        channel_kind: &'static str,
        channel_mode: &'static str,
        channel_type: &'static str,
        channel_impl: &'static str,
    ) -> EntityKey {
        let attrs = self.channel_attribute_set(
            channel_id,
            channel_kind,
            channel_mode,
            channel_type,
            channel_impl,
        );
        self.controller_context
            .telemetry_registry_handle
            .register_entity(attrs)
    }

    /// Returns a metrics registry handle.
    #[must_use]
    pub fn metrics_registry(&self) -> TelemetryRegistryHandle {
        self.controller_context.telemetry_registry_handle.clone()
    }

    /// Returns a new pipeline context with the given node identifiers.
    #[must_use]
    pub fn with_node_context(
        &self,
        node_id: NodeId,
        node_urn: NodeUrn,
        node_kind: NodeKind,
    ) -> Self {
        Self {
            controller_context: self.controller_context.clone(),
            core_id: self.core_id,
            thread_id: self.thread_id,
            pipeline_group_id: self.pipeline_group_id.clone(),
            pipeline_id: self.pipeline_id.clone(),
            node_id,
            node_urn,
            node_kind,
        }
    }
}
