// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Context providing general information on the current controller and the current pipeline.

use crate::attributes::{
    CustomAttributeSet, EngineAttributeSet, EngineEntityAttributeSet, ExtensionAttributeSet,
    ExtensionChannelAttributeSet, ExtensionScopeAttributeSet, NodeAttributeSet,
    NodeChannelAttributeSet, NodeWithCustomAttributeSet, NodeWithCustomTopicAttributeSet,
    NodeWithTopicAttributeSet, PipelineAttributeSet, config_map_to_telemetry,
};
use crate::entity_context::{current_node_telemetry_handle, node_entity_key};
use crate::memory_limiter::MemoryPressureState;
use crate::node::NodeId as EngineNodeId;
use otap_df_config::node::NodeKind;
use otap_df_config::pipeline::telemetry::TelemetryAttribute;
use otap_df_config::{NodeId as ConfigNodeId, NodeUrn, PipelineGroupId, PipelineId};
use otap_df_telemetry::InternalTelemetrySettings;
use otap_df_telemetry::metrics::MetricSetRegistrar;
use otap_df_telemetry::metrics::{
    MeasurementMetricSet, MeasurementMetricSetHandler, MetricSet, MetricSetHandler,
    RegistrationMetricSetHandler,
};
use otap_df_telemetry::registry::{EntityKey, MetricSetKey, TelemetryRegistryHandle};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// A shared, immutable mapping from otap_df_config node names
/// (without index numbers) to their engine-specific pipeline indices.
pub type NodeNameIndex = Arc<HashMap<ConfigNodeId, EngineNodeId>>;

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
    /// Unique process instance identifier (base32-encoded UUID v7).
    process_instance_id: Cow<'static, str>,
    /// Host identifier, when available (e.g. hostname).
    host_id: Cow<'static, str>,
    /// Container identifier, when available (e.g. Docker or containerd container ID).
    container_id: Cow<'static, str>,
    numa_node_id: usize,
    memory_pressure_state: MemoryPressureState,
}

/// Parameters required to create a pipeline context.
#[derive(Clone, Debug)]
pub struct PipelineContextParams {
    /// Pipeline group ID for the current pipeline execution context.
    pub pipeline_group_id: PipelineGroupId,
    /// Pipeline ID for the current pipeline execution context.
    pub pipeline_id: PipelineId,
    /// Core ID for the current pipeline execution context.
    pub core_id: usize,
    /// Total number of cores allocated to this pipeline.
    /// Used by nodes that need to share resources across cores (e.g., disk budgets).
    pub num_cores: usize,
    /// Thread ID for the current pipeline execution context.
    pub thread_id: usize,
}

/// A lightweight/cloneable pipeline context.
#[derive(Clone, Debug)]
pub struct PipelineContext {
    controller_context: ControllerContext,
    pipeline_context_params: PipelineContextParams,
    deployment_generation: u64,
    pipeline_telemetry_attrs: HashMap<String, TelemetryAttribute>,
    node_id: ConfigNodeId,
    node_urn: NodeUrn,
    node_kind: NodeKind,
    node_telemetry_attrs: HashMap<String, TelemetryAttribute>,

    /// Internal telemetry settings for the Internal Telemetry Receiver (ITR).
    /// Only the ITR factory reads this; other receivers ignore it.
    internal_telemetry: Option<InternalTelemetrySettings>,
    /// Shared mapping from node names to pipeline indices for mapping
    /// node names to the index used to send node control messages by,
    /// for example to map source-node name to index for inferring
    /// routes at runtime (e.g., how crates/validation works).
    node_names: NodeNameIndex,
    /// Optional pipeline-scoped topic set injected by the controller.
    /// ToDo: Make PipelineContext generic over a TopicSet type to avoid dynamic typing here.
    topic_set: Option<Arc<dyn Any + Send + Sync>>,
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
            memory_pressure_state: MemoryPressureState::default(),
        }
    }

    /// Creates a `ControllerContext` with explicit auto-detected identity values.
    ///
    /// Test-only: the production [`new`](Self::new) constructor derives these
    /// from the host environment, which is unsuitable for asserting the
    /// semantic-convention mapping in [`resource_attributes`](Self::resource_attributes).
    #[cfg(test)]
    fn new_with_identity(
        telemetry_registry_handle: TelemetryRegistryHandle,
        process_instance_id: impl Into<Cow<'static, str>>,
        host_id: impl Into<Cow<'static, str>>,
        container_id: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            telemetry_registry_handle,
            process_instance_id: process_instance_id.into(),
            host_id: host_id.into(),
            container_id: container_id.into(),
            numa_node_id: 0,
            memory_pressure_state: MemoryPressureState::default(),
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
        num_cores: usize,
        thread_id: usize,
    ) -> PipelineContext {
        self.pipeline_context_with_generation(
            pipeline_group_id,
            pipeline_id,
            core_id,
            num_cores,
            thread_id,
            0,
        )
    }

    /// Returns a new pipeline context with an explicit deployment generation.
    #[must_use]
    pub fn pipeline_context_with_generation(
        &self,
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        core_id: usize,
        num_cores: usize,
        thread_id: usize,
        deployment_generation: u64,
    ) -> PipelineContext {
        PipelineContext::new_with_generation(
            self.clone(),
            PipelineContextParams {
                pipeline_group_id,
                pipeline_id,
                core_id,
                num_cores,
                thread_id,
            },
            deployment_generation,
        )
    }

    /// Registers the engine-level entity for engine-wide metrics with no scope attributes.
    ///
    /// Returns the [`EntityKey`] to pass to
    /// [`EngineMetricsMonitor::new`](crate::engine_metrics::EngineMetricsMonitor::new).
    #[must_use]
    pub fn register_engine_entity(&self) -> EntityKey {
        self.telemetry_registry_handle
            .register_entity(EngineEntityAttributeSet)
    }

    /// Returns the auto-detected process/host resource attributes mapped to
    /// OpenTelemetry semantic-convention keys. Empty values are omitted.
    /// Keys: `host.id`, `container.id`, `service.instance.id`.
    #[must_use]
    pub fn resource_attributes(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if !self.host_id.is_empty() {
            out.push(("host.id".to_string(), self.host_id.to_string()));
        }
        if !self.container_id.is_empty() {
            out.push(("container.id".to_string(), self.container_id.to_string()));
        }
        if !self.process_instance_id.is_empty() {
            out.push((
                "service.instance.id".to_string(),
                self.process_instance_id.to_string(),
            ));
        }
        out
    }

    /// Returns a handle to the telemetry registry.
    #[must_use]
    pub fn telemetry_registry(&self) -> TelemetryRegistryHandle {
        self.telemetry_registry_handle.clone()
    }

    /// Returns the shared process-wide memory pressure state.
    #[must_use]
    pub fn memory_pressure_state(&self) -> MemoryPressureState {
        self.memory_pressure_state.clone()
    }
}

impl PipelineContext {
    /// Creates a new `PipelineContext`.
    #[allow(dead_code)]
    pub(crate) fn new(
        parent_ctx: ControllerContext,
        pipeline_context_params: PipelineContextParams,
    ) -> Self {
        Self::new_with_generation(parent_ctx, pipeline_context_params, 0)
    }

    /// Creates a new `PipelineContext` with an explicit deployment generation.
    pub(crate) fn new_with_generation(
        parent_ctx: ControllerContext,
        pipeline_context_params: PipelineContextParams,
        deployment_generation: u64,
    ) -> Self {
        Self {
            controller_context: parent_ctx,
            pipeline_context_params,
            deployment_generation,
            node_id: Default::default(),
            node_urn: Default::default(),
            node_kind: Default::default(),
            node_telemetry_attrs: HashMap::new(),
            pipeline_telemetry_attrs: HashMap::new(),
            internal_telemetry: None,
            node_names: Arc::new(HashMap::new()),
            topic_set: None,
        }
    }

    /// Returns the pipeline group ID associated with this pipeline context.
    #[must_use]
    pub fn pipeline_group_id(&self) -> PipelineGroupId {
        self.pipeline_context_params.pipeline_group_id.clone()
    }

    /// Returns the pipeline ID associated with this pipeline context.
    #[must_use]
    pub fn pipeline_id(&self) -> PipelineId {
        self.pipeline_context_params.pipeline_id.clone()
    }

    /// Returns the core ID associated with this pipeline context.
    #[must_use]
    pub const fn core_id(&self) -> usize {
        self.pipeline_context_params.core_id
    }

    /// Returns the deployment generation associated with this pipeline runtime.
    #[must_use]
    pub const fn deployment_generation(&self) -> u64 {
        self.deployment_generation
    }

    /// Returns the total number of cores allocated to this pipeline.
    ///
    /// This is useful for nodes that need to share resources (like disk budgets)
    /// across all cores running the same pipeline.
    #[must_use]
    pub const fn num_cores(&self) -> usize {
        self.pipeline_context_params.num_cores
    }

    /// Sets the internal telemetry settings for the Internal Telemetry Receiver.
    ///
    /// This is called by the pipeline factory when building the internal telemetry pipeline.
    /// The ITR factory will read these settings during node construction.
    pub fn set_internal_telemetry(&mut self, settings: InternalTelemetrySettings) {
        self.internal_telemetry = Some(settings);
    }

    /// Returns the internal telemetry settings, if configured.
    ///
    /// Only the Internal Telemetry Receiver factory uses this to obtain the logs
    /// channel and resource bytes it needs for operation.
    #[must_use]
    pub const fn internal_telemetry(&self) -> Option<&InternalTelemetrySettings> {
        self.internal_telemetry.as_ref()
    }

    /// Returns the shared process-wide memory pressure state.
    #[must_use]
    pub fn memory_pressure_state(&self) -> MemoryPressureState {
        self.controller_context.memory_pressure_state()
    }

    /// Sets the shared node-name-to-index mapping for this pipeline context.
    pub fn set_node_names(&mut self, node_names: NodeNameIndex) {
        self.node_names = node_names;
    }

    /// Sets the pipeline-scoped topic set resource.
    pub fn set_topic_set<T: Send + Sync + 'static>(
        &mut self,
        topic_set: crate::topic::TopicSet<T>,
    ) {
        self.topic_set = Some(Arc::new(topic_set));
    }

    /// Returns the pipeline-scoped topic set, if one was injected.
    #[must_use]
    pub fn topic_set<T: Send + Sync + 'static>(&self) -> Option<crate::topic::TopicSet<T>> {
        self.topic_set
            .as_ref()
            .and_then(|resource| resource.downcast_ref::<crate::topic::TopicSet<T>>())
            .cloned()
    }

    /// Returns the pipeline index for the given node name, if it exists.
    #[must_use]
    pub fn node_by_name(&self, name: &str) -> Option<EngineNodeId> {
        self.node_names.get(name).cloned()
    }

    /// Takes the internal telemetry settings, leaving None in its place.
    ///
    /// Used by the ITR factory to consume the settings during construction.
    #[must_use]
    pub const fn take_internal_telemetry(&mut self) -> Option<InternalTelemetrySettings> {
        self.internal_telemetry.take()
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

    /// Shared entity-resolution skeleton for the `register_*_metrics` family.
    ///
    /// Resolves the current node's telemetry scope in priority order — active node
    /// telemetry handle, then ambient node entity key — and registers the metric set
    /// via `for_entity`. When registering against an active node handle, the resulting
    /// metric-set key (obtained through `metric_set_key`) is tracked so the set is
    /// unregistered as part of node cleanup.
    ///
    /// Tests often construct nodes directly without the engine's entity scoping, so a
    /// final fallback registers the set against this context's own attribute set via
    /// `with_scope` (test builds only); production builds panic.
    fn register_scoped_metrics<R>(
        &self,
        for_entity: impl FnOnce(&TelemetryRegistryHandle, EntityKey) -> R,
        metric_set_key: impl FnOnce(&R) -> MetricSetKey,
        with_scope: impl FnOnce(&Self, &TelemetryRegistryHandle) -> R,
    ) -> R {
        let handle = &self.controller_context.telemetry_registry_handle;
        if let Some(telemetry) = current_node_telemetry_handle() {
            let metrics = for_entity(handle, telemetry.entity_key());
            telemetry.track_metric_set(metric_set_key(&metrics));
            metrics
        } else if let Some(entity_key) = node_entity_key() {
            for_entity(handle, entity_key)
        } else {
            #[cfg(feature = "test-utils")]
            {
                with_scope(self, handle)
            }
            #[cfg(not(feature = "test-utils"))]
            {
                let _ = with_scope;
                panic!(
                    "node entity key not set; ensure node entity is registered and instrumented"
                );
            }
        }
    }

    /// Registers a metric set for the current node entity.
    #[must_use]
    pub fn register_metrics<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
    ) -> MetricSet<T> {
        self.register_scoped_metrics(
            |handle, entity_key| handle.register_metric_set_for_entity::<T>(entity_key),
            MetricSet::metric_set_key,
            |ctx, handle| {
                if ctx.node_telemetry_attrs.is_empty() {
                    handle.register_metric_set::<T>(ctx.node_attribute_set())
                } else {
                    handle.register_metric_set::<T>(ctx.node_with_custom_attribute_set())
                }
            },
        )
    }

    /// Registers a metric set for the current node entity, scoped by an additional `topic` attribute.
    ///
    /// This is used by topic-aware nodes so their metric series can be filtered by `topic`.
    #[must_use]
    pub fn register_metrics_with_topic<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        topic: Cow<'static, str>,
    ) -> MetricSet<T> {
        let entity_key = if self.node_telemetry_attrs.is_empty() {
            self.controller_context
                .telemetry_registry_handle
                .register_entity(NodeWithTopicAttributeSet {
                    node_attrs: self.node_attribute_set(),
                    topic,
                })
        } else {
            self.controller_context
                .telemetry_registry_handle
                .register_entity(NodeWithCustomTopicAttributeSet {
                    node_custom_attrs: self.node_with_custom_attribute_set(),
                    topic,
                })
        };

        let metrics = self
            .controller_context
            .telemetry_registry_handle
            .register_metric_set_for_entity::<T>(entity_key);

        if let Some(telemetry) = current_node_telemetry_handle() {
            telemetry.track_metric_set(metrics.metric_set_key());
            telemetry.track_entity(entity_key);
        }

        metrics
    }

    /// Registers the pipeline entity for this context.
    #[must_use]
    pub fn register_pipeline_entity(&self) -> EntityKey {
        self.controller_context
            .telemetry_registry_handle
            .register_entity(self.pipeline_attribute_set())
    }

    /// Returns an [`ExtensionContext`] scoped to this pipeline.
    #[must_use]
    pub fn extension_context(&self) -> ExtensionContext {
        ExtensionContext::new(
            self.controller_context.clone(),
            ExtensionScopeAttributeSet::pipeline(self.pipeline_attribute_set()),
        )
    }

    /// Registers the node entity for this context.
    ///
    /// If the node has custom telemetry attributes configured, they are included
    /// in the entity registration. Otherwise, only the base node attributes are used,
    /// keeping telemetry output clean for nodes without custom attributes.
    #[must_use]
    pub fn register_node_entity(&self) -> EntityKey {
        if self.node_telemetry_attrs.is_empty() {
            self.controller_context
                .telemetry_registry_handle
                .register_entity(self.node_attribute_set())
        } else {
            self.controller_context
                .telemetry_registry_handle
                .register_entity(self.node_with_custom_attribute_set())
        }
    }

    fn engine_attribute_set(&self) -> EngineAttributeSet {
        EngineAttributeSet {
            core_id: self.pipeline_context_params.core_id,
            numa_node_id: self.controller_context.numa_node_id,
        }
    }

    /// Returns the pipeline attribute set for the current pipeline context.
    #[must_use]
    pub fn pipeline_attribute_set(&self) -> PipelineAttributeSet {
        PipelineAttributeSet {
            engine_attrs: self.engine_attribute_set(),
            pipeline_id: self.pipeline_context_params.pipeline_id.clone(),
            pipeline_group_id: self.pipeline_context_params.pipeline_group_id.clone(),
            deployment_generation: self.deployment_generation,
        }
    }

    /// Returns the node attribute set for the current node context.
    #[must_use]
    pub fn node_attribute_set(&self) -> NodeAttributeSet {
        NodeAttributeSet {
            pipeline_attrs: self.pipeline_attribute_set(),
            node_id: self.node_id.clone(),
            node_urn: self.node_urn.clone().into(),
            node_type: self.node_kind.into(),
        }
    }

    /// Returns the node attribute set extended with custom telemetry attributes.
    ///
    /// Only used when the node has non-empty `entity.extend.identity_attributes` configured.
    #[must_use]
    pub fn node_with_custom_attribute_set(&self) -> NodeWithCustomAttributeSet {
        NodeWithCustomAttributeSet {
            node_attrs: self.node_attribute_set(),
            custom_attrs: CustomAttributeSet::new(config_map_to_telemetry(
                &self.node_telemetry_attrs,
            )),
        }
    }

    /// Returns a channel attribute set tied to this node context.
    #[must_use]
    pub fn node_channel_attribute_set(
        &self,
        channel_id: Cow<'static, str>,
        node_port: Cow<'static, str>,
        channel_kind: &'static str,
        channel_mode: &'static str,
        channel_type: &'static str,
        channel_impl: &'static str,
    ) -> NodeChannelAttributeSet {
        NodeChannelAttributeSet {
            node_attrs: self.node_attribute_set(),
            node_port,
            channel_id,
            channel_kind: Cow::Borrowed(channel_kind),
            channel_mode: Cow::Borrowed(channel_mode),
            channel_type: Cow::Borrowed(channel_type),
            channel_impl: Cow::Borrowed(channel_impl),
        }
    }

    /// Registers a node-scoped channel entity for the given channel attributes.
    #[must_use]
    pub fn register_node_channel_entity(
        &self,
        channel_id: Cow<'static, str>,
        node_port: Cow<'static, str>,
        channel_kind: &'static str,
        channel_mode: &'static str,
        channel_type: &'static str,
        channel_impl: &'static str,
    ) -> EntityKey {
        let attrs = self.node_channel_attribute_set(
            channel_id,
            node_port,
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
        node_id: ConfigNodeId,
        node_urn: NodeUrn,
        node_kind: NodeKind,
        node_telemetry_attrs: HashMap<String, TelemetryAttribute>,
    ) -> Self {
        Self {
            controller_context: self.controller_context.clone(),
            pipeline_context_params: self.pipeline_context_params.clone(),
            deployment_generation: self.deployment_generation,
            pipeline_telemetry_attrs: self.pipeline_telemetry_attrs.clone(),
            node_id,
            node_urn,
            node_kind,
            node_telemetry_attrs,
            internal_telemetry: None,
            node_names: self.node_names.clone(),
            topic_set: self.topic_set.clone(),
        }
    }
}

impl MetricSetRegistrar for PipelineContext {
    fn register_metric_set<M: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
    ) -> MetricSet<M> {
        self.register_metrics::<M>()
    }

    fn register_registration_metric_set<M: RegistrationMetricSetHandler + Debug + Send + Sync>(
        &self,
        registration_attrs: &M::RegistrationAttributes,
    ) -> MetricSet<M> {
        self.register_scoped_metrics(
            |handle, entity_key| {
                handle.register_metric_set_with_registration_attributes_for_entity::<M>(
                    entity_key,
                    registration_attrs,
                )
            },
            MetricSet::metric_set_key,
            |ctx, handle| {
                if ctx.node_telemetry_attrs.is_empty() {
                    handle.register_metric_set_with_registration_attributes::<M>(
                        ctx.node_attribute_set(),
                        registration_attrs,
                    )
                } else {
                    handle.register_metric_set_with_registration_attributes::<M>(
                        ctx.node_with_custom_attribute_set(),
                        registration_attrs,
                    )
                }
            },
        )
    }

    fn register_measurement_metric_set<M: MeasurementMetricSetHandler + Debug + Send + Sync>(
        &self,
    ) -> MeasurementMetricSet<M> {
        self.register_scoped_metrics(
            |handle, entity_key| {
                handle.register_metric_set_with_measurement_attributes_for_entity::<M>(entity_key)
            },
            MeasurementMetricSet::metric_set_key,
            |ctx, handle| {
                if ctx.node_telemetry_attrs.is_empty() {
                    handle.register_metric_set_with_measurement_attributes::<M>(
                        ctx.node_attribute_set(),
                    )
                } else {
                    handle.register_metric_set_with_measurement_attributes::<M>(
                        ctx.node_with_custom_attribute_set(),
                    )
                }
            },
        )
    }

    fn register_registration_and_measurement_metric_set<
        M: RegistrationMetricSetHandler + MeasurementMetricSetHandler + Debug + Send + Sync,
    >(
        &self,
        registration_attrs: &M::RegistrationAttributes,
    ) -> MeasurementMetricSet<M> {
        self.register_scoped_metrics(
            |handle, entity_key| {
                handle
                    .register_metric_set_with_registration_and_measurement_attributes_for_entity::<M>(
                    entity_key,
                    registration_attrs,
                )
            },
            MeasurementMetricSet::metric_set_key,
            |ctx, handle| {
                if ctx.node_telemetry_attrs.is_empty() {
                    handle.register_metric_set_with_registration_and_measurement_attributes::<M>(
                        ctx.node_attribute_set(),
                        registration_attrs,
                    )
                } else {
                    handle.register_metric_set_with_registration_and_measurement_attributes::<M>(
                        ctx.node_with_custom_attribute_set(),
                        registration_attrs,
                    )
                }
            },
        )
    }
}

/// Host-scope context for extensions.
#[derive(Clone, Debug)]
pub struct ExtensionContext {
    controller_context: ControllerContext,
    extension_scope: ExtensionScopeAttributeSet,
}

impl ExtensionContext {
    /// Creates an `ExtensionContext` for the supplied host scope.
    ///
    /// # Panics (debug builds)
    ///
    /// Panics if `extension_scope.kind` is empty.
    #[must_use]
    pub fn new(
        controller_context: ControllerContext,
        extension_scope: ExtensionScopeAttributeSet,
    ) -> Self {
        debug_assert!(
            !extension_scope.kind.is_empty(),
            "ExtensionContext requires a non-empty scope kind"
        );
        Self {
            controller_context,
            extension_scope,
        }
    }

    /// Returns the telemetry registry handle for this scope.
    #[must_use]
    pub fn metrics_registry(&self) -> TelemetryRegistryHandle {
        self.controller_context.telemetry_registry_handle.clone()
    }

    /// Returns the attribute set for an extension hosted at this scope.
    #[must_use]
    pub fn extension_attribute_set(
        &self,
        extension_id: Cow<'static, str>,
        variant: crate::extension::wrapper::ExtensionVariant,
    ) -> ExtensionAttributeSet {
        ExtensionAttributeSet {
            extension_id,
            extension_variant: Cow::Borrowed(variant.as_str()),
            extension_scope: self.extension_scope.clone(),
        }
    }

    /// Registers an extension entity at this scope.
    #[must_use]
    pub fn register_extension_entity(
        &self,
        extension_id: Cow<'static, str>,
        variant: crate::extension::wrapper::ExtensionVariant,
    ) -> EntityKey {
        self.controller_context
            .telemetry_registry_handle
            .register_entity(self.extension_attribute_set(extension_id, variant))
    }

    /// Returns a channel attribute set tied to the given extension.
    #[must_use]
    pub fn extension_channel_attribute_set(
        &self,
        extension_id: Cow<'static, str>,
        variant: crate::extension::wrapper::ExtensionVariant,
        channel_id: Cow<'static, str>,
        channel_mode: &'static str,
        channel_impl: &'static str,
    ) -> ExtensionChannelAttributeSet {
        ExtensionChannelAttributeSet {
            extension_attrs: self.extension_attribute_set(extension_id, variant),
            channel_id,
            channel_mode: Cow::Borrowed(channel_mode),
            channel_impl: Cow::Borrowed(channel_impl),
        }
    }

    /// Registers an extension-scoped channel entity for the given attributes.
    #[must_use]
    pub fn register_extension_channel_entity(
        &self,
        extension_id: Cow<'static, str>,
        variant: crate::extension::wrapper::ExtensionVariant,
        channel_id: Cow<'static, str>,
        channel_mode: &'static str,
        channel_impl: &'static str,
    ) -> EntityKey {
        let attrs = self.extension_channel_attribute_set(
            extension_id,
            variant,
            channel_id,
            channel_mode,
            channel_impl,
        );
        self.controller_context
            .telemetry_registry_handle
            .register_entity(attrs)
    }

    /// Registers a metric set for the given entity key.
    ///
    /// Unlike [`PipelineContext::register_metric_set_for_entity`], this does
    /// not hook into any ambient node telemetry — extension entities own their
    /// own lifecycle via the per-variant `EntityTelemetryGuard`.
    #[must_use]
    pub fn register_metric_set_for_entity<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        entity_key: EntityKey,
    ) -> MetricSet<T> {
        self.controller_context
            .telemetry_registry_handle
            .register_metric_set_for_entity::<T>(entity_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_attributes_maps_semconv_keys() {
        let ctx = ControllerContext::new_with_identity(
            TelemetryRegistryHandle::new(),
            "proc-123",
            "machine-abc",
            "container-xyz",
        );

        // All three identities present: emitted in a stable order under their
        // OpenTelemetry semantic-convention keys.
        assert_eq!(
            ctx.resource_attributes(),
            vec![
                ("host.id".to_string(), "machine-abc".to_string()),
                ("container.id".to_string(), "container-xyz".to_string()),
                ("service.instance.id".to_string(), "proc-123".to_string()),
            ]
        );
    }

    #[test]
    fn resource_attributes_omits_empty_values() {
        // Empty host/container should be skipped entirely (no empty-valued
        // attributes), leaving only the populated service.instance.id.
        let ctx = ControllerContext::new_with_identity(
            TelemetryRegistryHandle::new(),
            "proc-123",
            "",
            "",
        );

        assert_eq!(
            ctx.resource_attributes(),
            vec![("service.instance.id".to_string(), "proc-123".to_string())]
        );
    }
}
