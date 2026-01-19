// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Attributes describing the resource, engine, pipeline, and node context.
//!
//! Note: At the moment, these attributes are used for metrics aggregation and reporting.

use otap_df_telemetry_macros::attribute_set;
use std::borrow::Cow;

/// Resource attributes (host id, process instance id, container id, ...).
#[attribute_set(name = "resource.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct ResourceAttributeSet {
    /// Unique process instance identifier (base32-encoded UUID v7).
    #[attribute]
    pub process_instance_id: Cow<'static, str>,
    /// Host identifier, when available (e.g. hostname).
    #[attribute]
    pub host_id: Cow<'static, str>,
    /// Container identifier, when available (e.g. Docker or containerd container ID).
    #[attribute]
    pub container_id: Cow<'static, str>,
}

/// Engine attributes (core id, numa node id, ...).
#[attribute_set(name = "controller.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct EngineAttributeSet {
    /// Resource attributes.
    #[compose]
    pub resource_attrs: ResourceAttributeSet,

    /// Core identifier.
    #[attribute]
    pub core_id: usize,
    /// NUMA node identifier.
    #[attribute]
    pub numa_node_id: usize,
}

/// Pipeline attributes.
#[attribute_set(name = "pipeline.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct PipelineAttributeSet {
    /// Engine attributes.
    #[compose]
    pub engine_attrs: EngineAttributeSet,

    /// Pipeline group identifier.
    #[attribute]
    pub pipeline_group_id: Cow<'static, str>,
    /// Pipeline identifier as defined in the configuration.
    #[attribute]
    pub pipeline_id: Cow<'static, str>,
}

/// Node attributes.
#[attribute_set(name = "node.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct NodeAttributeSet {
    /// Pipeline attributes.
    #[compose]
    pub pipeline_attrs: PipelineAttributeSet,

    /// Node unique identifier (in scope of the pipeline).
    #[attribute]
    pub node_id: Cow<'static, str>,
    /// Node plugin URN.
    #[attribute(key = "node.urn")]
    pub node_urn: Cow<'static, str>,
    /// Node type (e.g., "receiver", "processor", "exporter").
    #[attribute]
    pub node_type: Cow<'static, str>,
}

/// Channel endpoint attributes (sender or receiver).
#[attribute_set(name = "channel.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct ChannelAttributeSet {
    /// Node attributes.
    #[compose]
    pub node_attrs: NodeAttributeSet,

    /// Unique channel identifier (in scope of the pipeline).
    #[attribute(key = "channel.id")]
    pub channel_id: Cow<'static, str>,
    /// Channel payload kind ("control" or "pdata").
    #[attribute(key = "channel.kind")]
    pub channel_kind: Cow<'static, str>,
    /// Concurrency mode of the channel ("local" or "shared").
    #[attribute(key = "channel.mode")]
    pub channel_mode: Cow<'static, str>,
    /// Channel type ("mpsc", "mpmc", "spsc", "spmc").
    #[attribute(key = "channel.type")]
    pub channel_type: Cow<'static, str>,
    /// Channel implementation ("tokio", "flume", "internal").
    #[attribute(key = "channel.impl")]
    pub channel_impl: Cow<'static, str>,
}
