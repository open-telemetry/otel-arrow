// SPDX-License-Identifier: Apache-2.0

//! Errors for the config crate.

use crate::node::DispatchStrategy;
use crate::{NamespaceId, NodeId, PipelineId, PortName};
use miette::Diagnostic;
use std::fmt::Display;

/// Errors that can occur while processing the configuration of a data plane, a namespace, a pipeline,
/// or a node.
///
/// Note: All errors are contextualized with the namespace and pipeline ids, if applicable.
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

    /// The same out‐port was connected more than once on a single node.
    #[error(
        "The same out-port `{port}` was connected more than once on the node `{source_node}`\nContext: {context}"
    )]
    #[diagnostic(code(data_plane::duplicate_out_port), url(docsrs))]
    DuplicateOutPort {
        /// The context in which the error occurred.
        context: Context,
        /// The node on which the port was duplicated.
        source_node: NodeId,
        /// The port name that was used twice.
        port: PortName,
    },

    /// An edge was specified with a source node or target nodes that do not exist in the pipeline.
    #[error(
        "Invalid hyper-edge specification: {source_node} -> {target_nodes:?}\nContext: {context}"
    )]
    #[diagnostic(code(data_plane::invalid_hyper_edge_spec), url(docsrs))]
    InvalidHyperEdgeSpec {
        /// The context in which the error occurred.
        context: Context,

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

    /// An invalid user configuration occurred.
    #[error("An invalid user configuration occurred: {error}")]
    InvalidUserConfig {
        /// An error message.
        error: String,
    },
}

/// Information that all errors provide to help identify
/// the context in which they occurred.
#[derive(Debug, Default)]
pub struct Context {
    /// The namespace id, if applicable.
    pub namespace_id: Option<NamespaceId>,
    /// The pipeline id, if applicable.
    pub pipeline_id: Option<PipelineId>,
}

impl Context {
    /// Creates a new context with the given namespace and pipeline ids.
    #[must_use]
    pub fn new(namespace_id: NamespaceId, pipeline_id: PipelineId) -> Self {
        Self {
            namespace_id: Some(namespace_id),
            pipeline_id: Some(pipeline_id),
        }
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(namespace_id) = &self.namespace_id {
            write!(f, "Namespace: '{namespace_id}'")?;
        }
        if let Some(pipeline_id) = &self.pipeline_id {
            write!(f, " Pipeline: '{pipeline_id}'")?;
        }
        Ok(())
    }
}
