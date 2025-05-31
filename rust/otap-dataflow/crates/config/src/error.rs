// SPDX-License-Identifier: Apache-2.0

//! Errors for the config crate.

use crate::node::DispatchStrategy;
use crate::{NodeId, PipelineId, PortName, TenantId};

/// Errors that can occur while processing the configuration of a data plane, a tenant, a pipeline,
/// or a node.
///
/// Note: All errors are contextualized with the tenant and pipeline ids, if applicable.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A collection of errors that occurred during parsing or validating the configuration.
    #[error("Invalid configuration: {errors:?}")]
    InvalidConfiguration {
        /// A list of errors that occurred during parsing or validating the configuration.
        errors: Vec<Error>,
    },

    /// An error that occurred while reading a configuration file.
    #[error("File read error: {details}\nTenant: {tenant_id:?}, Pipeline: {pipeline_id:?}")]
    FileReadError {
        /// The tenant id, if applicable.
        tenant_id: Option<TenantId>,
        /// The pipeline id, if applicable.
        pipeline_id: Option<PipelineId>,
        /// A description of the error that occurred.
        details: String,
    },

    /// An error that occurred while deserializing a configuration file.
    #[error(
        "{format} deserialization error: {details}\nTenant: {tenant_id:?}, Pipeline: {pipeline_id:?}"
    )]
    DeserializationError {
        /// The tenant id, if applicable.
        tenant_id: Option<TenantId>,
        /// The pipeline id, if applicable.
        pipeline_id: Option<PipelineId>,
        /// The format of the configuration file (e.g. "JSON").
        format: String,
        /// A description of the error that occurred.
        details: String,
    },

    /// A cycle was detected in the pipeline configuration.
    #[error(
        "Cycle detected involving nodes: {nodes:?}\nTenant: {tenant_id:?}, Pipeline: {pipeline_id:?}"
    )]
    CycleDetected {
        /// The tenant id, if applicable.
        tenant_id: Option<TenantId>,
        /// The pipeline id, if applicable.
        pipeline_id: Option<PipelineId>,
        /// The nodes involved in the cycle.
        nodes: Vec<NodeId>,
    },

    /// A node with the same id already exists in the pipeline.
    #[error("Duplicated node id `{node_id}`\nTenant: {tenant_id:?}, Pipeline: {pipeline_id:?}")]
    DuplicateNode {
        /// The tenant id, if applicable.
        tenant_id: Option<TenantId>,
        /// The pipeline id, if applicable.
        pipeline_id: Option<PipelineId>,
        /// The id of the node that was duplicated.
        node_id: NodeId,
    },

    /// The same out‐port was connected more than once on a single node.
    #[error(
        "The same out-port `{port}` was connected more than once on the node `{source_node}`\nTenant: {tenant_id:?}, Pipeline: {pipeline_id:?}"
    )]
    DuplicateOutPort {
        /// The tenant id, if applicable.
        tenant_id: Option<TenantId>,
        /// The pipeline id, if applicable.
        pipeline_id: Option<PipelineId>,
        /// The node on which the port was duplicated.
        source_node: NodeId,
        /// The port name that was used twice.
        port: PortName,
    },

    /// An edge was specified with a source node or target nodes that do not exist in the pipeline.
    #[error(
        "Invalid hyper-edge specification: {source_node} -> {target_nodes:?}\nTenant: {tenant_id:?}, Pipeline: {pipeline_id:?}"
    )]
    InvalidHyperEdgeSpec {
        /// The tenant id, if applicable.
        tenant_id: Option<TenantId>,
        /// The pipeline id, if applicable.
        pipeline_id: Option<PipelineId>,

        /// The source node of the hyper-edge.
        source_node: NodeId,
        /// The target nodes of the hyper-edge.
        target_nodes: Vec<NodeId>,
        /// The dispatch strategy for the hyper-edge.
        dispatch_strategy: DispatchStrategy,
        /// Whether the source node is missing.
        missing_source: bool,
        /// The target nodes that are missing in the pipeline.
        missing_targets: Vec<NodeId>,
    },
}
