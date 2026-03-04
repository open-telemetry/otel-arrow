// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for the config crate.

use crate::node::NodeKind;
use crate::pipeline::DispatchPolicy;
use crate::{NodeId, PipelineGroupId, PipelineId, PortName};
use miette::Diagnostic;
use std::fmt::Display;

/// Details about an invalid hyper-edge specification.
#[derive(Debug)]
pub struct HyperEdgeSpecDetails {
    /// The target nodes of the hyper-edge.
    pub target_nodes: Vec<NodeId>,
    /// The dispatch policy for the hyper-edge.
    pub dispatch_policy: DispatchPolicy,
    /// The target nodes that are missing in the pipeline.
    pub missing_targets: Vec<NodeId>,
}

/// Errors that can occur while processing the configuration of a data plane, a pipeline group, a
/// pipeline, or a node.
///
/// Note: All errors are contextualized with the pipeline group and pipeline ids, if applicable.
#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum Error {
    /// A collection of errors that occurred during parsing or validating the configuration.
    #[error("Invalid configuration: {errors:?}")]
    #[diagnostic(code(data_plane::invalid_configuration), url(docsrs))]
    InvalidConfiguration {
        /// A list of errors that occurred during parsing or validating the configuration.
        #[related]
        errors: Vec<Error>,
    },

    /// An error that occurred while reading a configuration file.
    #[error("File read error: {details}\nContext: {context}")]
    #[diagnostic(code(data_plane::file_read_error), url(docsrs))]
    FileReadError {
        /// The context in which the error occurred.
        context: Context,
        /// A description of the error that occurred.
        details: String,
    },

    /// An error that occurred while deserializing a configuration file.
    #[error("{format} deserialization error: {details}\nContext: {context}")]
    #[diagnostic(code(data_plane::deserialization_error), url(docsrs))]
    DeserializationError {
        /// The context in which the error occurred.
        context: Context,
        /// The format of the configuration file (e.g. "JSON").
        format: String,
        /// A description of the error that occurred.
        details: String,
    },

    /// A cycle was detected in the pipeline configuration.
    #[error("Cycle detected involving nodes: {nodes:?}\nContext: {context}")]
    #[diagnostic(code(data_plane::cycle_detected), url(docsrs))]
    CycleDetected {
        /// The context in which the error occurred.
        context: Context,
        /// The nodes involved in the cycle.
        nodes: Vec<NodeId>,
    },

    /// A node with the same id already exists in the pipeline.
    #[error("Duplicated node id `{node_id}`\nContext: {context}")]
    #[diagnostic(code(data_plane::duplicate_node), url(docsrs))]
    DuplicateNode {
        /// The context in which the error occurred.
        context: Context,
        /// The id of the node that was duplicated.
        node_id: NodeId,
    },

    /// The same output port was connected more than once on a single node.
    #[error(
        "The same output port `{port}` was connected more than once on the node `{source_node}`\nContext: {context}"
    )]
    #[diagnostic(code(data_plane::duplicate_out_port), url(docsrs))]
    DuplicateOutputPort {
        /// The context in which the error occurred.
        context: Context,
        /// The node on which the port was duplicated.
        source_node: NodeId,
        /// The port name that was used twice.
        port: PortName,
    },

    /// An edge was specified with a source node or target nodes that do not exist in the pipeline.
    #[error("Invalid hyper-edge specification: {source_node} -> {details:?}\nContext: {context}")]
    #[diagnostic(code(data_plane::invalid_hyper_edge_spec), url(docsrs))]
    InvalidHyperEdgeSpec {
        /// The context in which the error occurred.
        context: Context,

        /// The source node of the hyper-edge.
        source_node: NodeId,
        /// Whether the source node is missing.
        missing_source: bool,
        /// Details about the hyper-edge specification.
        details: Box<HyperEdgeSpecDetails>,
    },

    /// A connection source selector is malformed.
    #[error("Invalid connection source `{selector}`: {message}")]
    #[diagnostic(code(data_plane::invalid_connection_source), url(docsrs))]
    InvalidConnectionSource {
        /// The raw connection source selector.
        selector: String,
        /// A description of why parsing failed.
        message: String,
    },

    /// A node has an invalid type URN in pipeline configuration.
    #[error("Invalid node `type` for node `{node_id}`: {details}\nContext: {context}")]
    #[diagnostic(code(data_plane::invalid_node_type), url(docsrs))]
    InvalidNodeType {
        /// The context in which the error occurred.
        context: Box<Context>,
        /// The node containing the invalid type.
        node_id: NodeId,
        /// Validation details from the URN parser.
        details: String,
    },

    /// A connection uses an unsupported dispatch policy for the requested topology.
    #[error(
        "Unsupported dispatch policy `{dispatch_policy:?}` for connection {source_nodes:?} -> {target_nodes:?}\nContext: {context}"
    )]
    #[diagnostic(code(data_plane::unsupported_dispatch_policy), url(docsrs))]
    UnsupportedConnectionDispatchPolicy {
        /// The context in which the error occurred.
        context: Box<Context>,
        /// The dispatch policy configured on the connection.
        dispatch_policy: DispatchPolicy,
        /// Source node ids for the connection.
        source_nodes: Box<[NodeId]>,
        /// Destination node ids for the connection.
        target_nodes: Box<[NodeId]>,
    },

    /// A connection has an empty endpoint set.
    #[error(
        "Invalid connection endpoint set: `from` and `to` must both be non-empty (from_empty={from_empty}, to_empty={to_empty})\nContext: {context}"
    )]
    #[diagnostic(code(data_plane::empty_connection_endpoint_set), url(docsrs))]
    EmptyConnectionEndpointSet {
        /// The context in which the error occurred.
        context: Box<Context>,
        /// Whether the connection source set is empty.
        from_empty: bool,
        /// Whether the connection destination set is empty.
        to_empty: bool,
    },

    /// A connection endpoint references a node kind not allowed at that endpoint.
    #[error(
        "Invalid connection `{endpoint}` endpoint node kind for `{node_id}`: expected one of {expected_kinds:?}, got {actual_kind:?}\nContext: {context}"
    )]
    #[diagnostic(code(data_plane::invalid_connection_node_kind), url(docsrs))]
    InvalidConnectionNodeKind {
        /// The context in which the error occurred.
        context: Box<Context>,
        /// The node id referenced by the endpoint.
        node_id: NodeId,
        /// The endpoint role (`from` or `to`).
        endpoint: String,
        /// The actual node kind for this node id.
        actual_kind: NodeKind,
        /// The allowed node kinds for this endpoint.
        expected_kinds: Box<[NodeKind]>,
    },

    /// A connection selects an output that is not declared by the source node.
    #[error(
        "Connection source `{source_node}` selects output `{selected_output}` which is not declared in `outputs` ({declared_outputs:?})\nContext: {context}"
    )]
    #[diagnostic(code(data_plane::undeclared_connection_output), url(docsrs))]
    UndeclaredConnectionOutput {
        /// The context in which the error occurred.
        context: Box<Context>,
        /// The source node id.
        source_node: NodeId,
        /// The selected output name.
        selected_output: PortName,
        /// The output names declared on the source node.
        declared_outputs: Box<[PortName]>,
    },

    /// An invalid user configuration occurred.
    #[error("An invalid user configuration occurred: {error}")]
    InvalidUserConfig {
        /// An error message.
        error: String,
    },

    /// A pipeline with the same id already exists in the pipeline group.
    #[error("Pipeline with id `{pipeline_id}` already exists in the pipeline group")]
    DuplicatePipeline {
        /// The id of the pipeline that was duplicated.
        pipeline_id: PipelineId,
    },
}

/// Information that all errors provide to help identify
/// the context in which they occurred.
#[derive(Debug, Default)]
pub struct Context {
    /// The pipeline group id, if applicable.
    pub pipeline_group_id: Option<PipelineGroupId>,
    /// The pipeline id, if applicable.
    pub pipeline_id: Option<PipelineId>,
}

impl Context {
    /// Creates a new context with the given pipeline group and pipeline ids.
    #[must_use]
    pub const fn new(pipeline_group_id: PipelineGroupId, pipeline_id: PipelineId) -> Self {
        Self {
            pipeline_group_id: Some(pipeline_group_id),
            pipeline_id: Some(pipeline_id),
        }
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pipeline_group_id) = &self.pipeline_group_id {
            write!(f, "Pipeline group: '{pipeline_group_id}'")?;
        }
        if let Some(pipeline_id) = &self.pipeline_id {
            write!(f, " Pipeline: '{pipeline_id}'")?;
        }
        Ok(())
    }
}
