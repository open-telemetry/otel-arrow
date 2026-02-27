// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP data plane configuration.
//!
//! Data Model:
//! - data plane
//!   - pipeline groups
//!     - pipelines
//!       - nodes
//!
//! A data plane is a collection of pipeline groups, where each group can have multiple pipelines.
//! A pipeline is a collection of nodes interconnected in a directed acyclic graph (DAG).

use serde::{Deserialize, Serialize, ser::Serializer};
use std::borrow::Cow;
use std::hash::Hash;

pub mod byte_units;
pub mod engine;
pub mod error;
pub mod health;
pub mod node;
/// Node type URN value object.
pub mod node_urn;
pub mod observed_state;
pub mod pipeline;
pub mod pipeline_group;
pub mod policy;
/// Engine telemetry settings.
pub mod settings;
/// TLS configuration.
pub mod tls;
pub mod topic;
pub use topic::{SubscriptionGroupName, TopicName};
/// Validation helpers for node configuration.
pub mod validation;

/// Signal types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalType {
    /// Signal representing a stream of spans.
    Traces,
    /// Signal representing a stream of metrics.
    Metrics,
    /// Signal representing a stream of logs.
    Logs,
}

/// Signal formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalFormat {
    /// OTel-Arrow records
    OtapRecords,
    /// OTLP protobuf bytes
    OtlpBytes,
    // TODO: maybe add types not included in OtapPdata including
    // OtlpProtoMessage, OtapArrowBytes, and possible opaque.
}

/// The id of a pipeline group.
pub type PipelineGroupId = Cow<'static, str>;

/// The id of a pipeline.
pub type PipelineId = Cow<'static, str>;

/// The id of a node in the pipeline.
pub type NodeId = Cow<'static, str>;

/// The URN of a node type.
pub use node_urn::NodeUrn;

/// The name of a node output port in the pipeline.
pub type PortName = Cow<'static, str>;

/// The description of a pipeline or a node.
pub type Description = Cow<'static, str>;

/// Type alias for CPU core identifier.
/// Note: Not using core_affinity::CoreId directly to avoid dependency leakage in this public API
pub type CoreId = usize;

/// Unique key for identifying a pipeline within a pipeline group.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PipelineKey {
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
}

impl PipelineKey {
    /// Construct a new PipelineKey from group and pipeline ids.
    #[must_use]
    pub const fn new(pipeline_group_id: PipelineGroupId, pipeline_id: PipelineId) -> Self {
        Self {
            pipeline_group_id,
            pipeline_id,
        }
    }

    /// Returns the pipeline group identifier.
    #[must_use]
    pub const fn pipeline_group_id(&self) -> &PipelineGroupId {
        &self.pipeline_group_id
    }

    /// Returns the pipeline identifier.
    #[must_use]
    pub const fn pipeline_id(&self) -> &PipelineId {
        &self.pipeline_id
    }

    /// Returns a `group_id:pipeline_id` string representation.
    #[must_use]
    pub fn as_string(&self) -> String {
        format!("{}:{}", self.pipeline_group_id, self.pipeline_id)
    }
}

impl Serialize for PipelineKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.as_string();
        serializer.serialize_str(&s)
    }
}

/// Unique key for identifying a pipeline running on a specific core.
#[derive(Debug, Clone, Serialize)]
pub struct DeployedPipelineKey {
    /// The unique ID of the pipeline group the pipeline belongs to.
    pub pipeline_group_id: PipelineGroupId,

    /// The unique ID of the pipeline within its group.
    pub pipeline_id: PipelineId,

    /// The CPU core ID the pipeline is pinned to.
    pub core_id: CoreId,
}
