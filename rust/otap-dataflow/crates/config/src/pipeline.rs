// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline configuration specification.
pub mod telemetry;

use crate::error::{Context, Error, HyperEdgeSpecDetails};
use crate::node::{NodeKind, NodeUserConfig};
use crate::policy::Policies;
use crate::{Description, NodeId, NodeUrn, PipelineGroupId, PipelineId, PortName};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// A pipeline configuration describing the interconnections between nodes.
/// A pipeline is a directed acyclic graph that could be qualified as a hyper-DAG:
/// - "Hyper" because the edges connecting the nodes can be hyper-edges.
/// - A node can be connected to multiple outgoing nodes.
/// - The way messages are dispatched over each hyper-edge is defined by a dispatch policy representing
///   communication semantics. For example, it can route each message to one destination (`one_of`), or
///   in the future it can broadcast to every destination.
///
/// This configuration defines the pipeline’s nodes, the interconnections
/// (hyper-edges) and optional pipeline-level policies.
///
/// Use `PipelineConfig::from_yaml` or `PipelineConfig::from_json` instead of
/// deserializing directly with serde to ensure plugin URNs are normalized.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PipelineConfig {
    /// Type of the pipeline, which determines the type of PData it processes.
    ///
    /// Note: Even though technically our engine can support several types of pdata, we
    /// are focusing our efforts on the OTAP pipeline (hence the default value).
    #[serde(default = "default_pipeline_type")]
    r#type: PipelineType,

    /// Optional policy set for this pipeline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    policies: Option<Policies>,

    /// All nodes in this pipeline, keyed by node ID.
    #[serde(default)]
    nodes: PipelineNodes,

    /// Explicit graph connections between nodes.
    ///
    /// When provided, these connections are used as the authoritative topology for
    /// the main pipeline graph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    connections: Vec<PipelineConnection>,
}

const fn default_pipeline_type() -> PipelineType {
    PipelineType::Otap
}

/// The type of pipeline, which can be either OTLP (OpenTelemetry Protocol) or
/// OTAP (OpenTelemetry with Apache Arrow Protocol).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PipelineType {
    /// OpenTelemetry Protocol (OTLP) pipeline.
    /// ToDo: With the recent benchmark results on proto_bytes->views->OTAP, we could consider to get rid of the OTLP pipeline type.
    Otlp,
    /// OpenTelemetry with Apache Arrow Protocol (OTAP) pipeline.
    Otap,
}

/// Connection source selector.
///
/// Supports either:
/// - `<node-id>`
/// - `<node-id>["<port-name>"]`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct ConnectionSource {
    node_id: NodeId,
    output_port: Option<PortName>,
}

const DEFAULT_CONNECTION_SOURCE_PORT: &str = "default";

fn default_connection_source_port() -> PortName {
    DEFAULT_CONNECTION_SOURCE_PORT.into()
}

impl ConnectionSource {
    /// Creates a connection source referencing a node with no explicit port selector.
    #[must_use]
    pub const fn from_node(node_id: NodeId) -> Self {
        Self {
            node_id,
            output_port: None,
        }
    }

    /// Creates a connection source referencing a node and an explicit port selector.
    #[must_use]
    pub const fn from_node_with_port(node_id: NodeId, output_port: PortName) -> Self {
        Self {
            node_id,
            output_port: Some(output_port),
        }
    }

    /// Returns the source node id.
    #[must_use]
    pub const fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Returns the optional explicit source output port selector.
    #[must_use]
    pub const fn output_port(&self) -> Option<&PortName> {
        self.output_port.as_ref()
    }

    /// Returns the effective output port for this source.
    #[must_use]
    pub fn resolved_output_port(&self) -> PortName {
        self.output_port
            .clone()
            .unwrap_or_else(default_connection_source_port)
    }
}

impl TryFrom<String> for ConnectionSource {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.is_empty() {
            return Err(Error::InvalidConnectionSource {
                selector: value.to_string(),
                message: "connection source must be non-empty".to_string(),
            });
        }

        if let Some(open) = value.find('[') {
            if !value.ends_with(']') {
                return Err(Error::InvalidConnectionSource {
                    selector: value.to_string(),
                    message: "expected `<node>[\"<output>\"]`".to_string(),
                });
            }
            let node = value[..open].trim();
            if node.is_empty() {
                return Err(Error::InvalidConnectionSource {
                    selector: value.to_string(),
                    message: "missing node id before `[`".to_string(),
                });
            }
            let selector = value[open + 1..value.len() - 1].trim();
            let quoted_port = if selector.len() >= 2
                && ((selector.starts_with('"') && selector.ends_with('"'))
                    || (selector.starts_with('\'') && selector.ends_with('\'')))
            {
                &selector[1..selector.len() - 1]
            } else {
                return Err(Error::InvalidConnectionSource {
                    selector: value.to_string(),
                    message: "expected quoted output selector".to_string(),
                });
            };
            if quoted_port.is_empty() {
                return Err(Error::InvalidConnectionSource {
                    selector: value.to_string(),
                    message: "output selector must be non-empty".to_string(),
                });
            }
            Ok(Self::from_node_with_port(
                node.to_string().into(),
                quoted_port.to_string().into(),
            ))
        } else {
            if value.contains(']') {
                return Err(Error::InvalidConnectionSource {
                    selector: value.to_string(),
                    message: "unexpected closing `]`".to_string(),
                });
            }
            Ok(Self::from_node(value.to_string().into()))
        }
    }
}

impl From<ConnectionSource> for String {
    fn from(value: ConnectionSource) -> Self {
        match value.output_port {
            Some(output_port) => format!("{}[\"{}\"]", value.node_id, output_port),
            None => value.node_id.to_string(),
        }
    }
}

/// A set of source selectors used by a connection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum ConnectionSourceSet {
    /// A single source selector.
    One(ConnectionSource),
    /// Multiple source selectors (fan-in).
    Many(Vec<ConnectionSource>),
}

impl ConnectionSourceSet {
    /// Returns this source set as a concrete list of selectors.
    #[must_use]
    pub fn sources(&self) -> Vec<ConnectionSource> {
        match self {
            Self::One(source) => vec![source.clone()],
            Self::Many(sources) => sources.clone(),
        }
    }
}

/// A set of destination node ids used by a connection endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum ConnectionNodeSet {
    /// A single node id.
    One(NodeId),
    /// Multiple node ids (fan-out).
    Many(Vec<NodeId>),
}

impl ConnectionNodeSet {
    /// Returns this endpoint set as a concrete list of node ids.
    #[must_use]
    pub fn nodes(&self) -> Vec<NodeId> {
        match self {
            Self::One(node_id) => vec![node_id.clone()],
            Self::Many(node_ids) => node_ids.clone(),
        }
    }
}

/// A connection between source and destination node sets.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PipelineConnection {
    /// Source node selector(s). A list denotes fan-in.
    pub from: ConnectionSourceSet,
    /// Destination node id(s). A list denotes fan-out.
    pub to: ConnectionNodeSet,
    /// Optional policy set for this connection.
    #[serde(default, skip_serializing_if = "ConnectionPolicies::is_empty")]
    pub policies: ConnectionPolicies,
}

impl PipelineConnection {
    /// Returns the list of source selectors for this connection.
    #[must_use]
    pub fn from_sources(&self) -> Vec<ConnectionSource> {
        self.from.sources()
    }

    /// Returns the list of source node ids for this connection.
    #[must_use]
    pub fn from_nodes(&self) -> Vec<NodeId> {
        self.from_sources()
            .into_iter()
            .map(|source| source.node_id)
            .collect()
    }

    /// Returns the list of destination node ids for this connection.
    #[must_use]
    pub fn to_nodes(&self) -> Vec<NodeId> {
        self.to.nodes()
    }

    /// Returns the effective dispatch policy for this connection.
    ///
    /// For single-destination connections this always resolves to `one_of`,
    /// regardless of the configured policy.
    #[must_use]
    pub fn effective_dispatch_policy(&self) -> DispatchPolicy {
        if !self.has_multiple_destinations() {
            return DispatchPolicy::OneOf;
        }
        self.policies
            .dispatch
            .clone()
            .unwrap_or(DispatchPolicy::OneOf)
    }

    #[must_use]
    const fn has_multiple_destinations(&self) -> bool {
        match &self.to {
            ConnectionNodeSet::One(_) => false,
            ConnectionNodeSet::Many(node_ids) => node_ids.len() > 1,
        }
    }
}

/// Policy set for a connection.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConnectionPolicies {
    /// Optional dispatch policy for this connection.
    ///
    /// When omitted, `one_of` is used.
    /// For single-destination connections this field has no runtime effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispatch: Option<DispatchPolicy>,
}

impl ConnectionPolicies {
    /// Returns whether this policy set has no configured policies.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.dispatch.is_none()
    }
}

/// Dispatch policy for a connection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DispatchPolicy {
    /// Send each message to exactly one destination.
    ///
    /// When `to` contains multiple destinations, they act as competing
    /// consumers of the same queue. Selection is runtime-driven and
    /// best-effort balanced (not strict deterministic round-robin).
    OneOf,
    /// Send each message to every destination.
    Broadcast,
}

/// A collection of nodes forming a pipeline graph.
///
/// Note: We use `Arc<NodeUserConfig>` to allow sharing the same pipeline configuration
/// across multiple cores/threads without cloning the entire configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(transparent)]
pub struct PipelineNodes(HashMap<NodeId, Arc<NodeUserConfig>>);

impl PipelineNodes {
    /// Returns true if the node collection is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a reference to the node with the given ID, if it exists.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Arc<NodeUserConfig>> {
        self.0.get(id)
    }

    /// Returns true if a node with the given ID exists.
    #[must_use]
    pub fn contains_key(&self, id: &str) -> bool {
        self.0.contains_key(id)
    }

    /// Returns an iterator visiting all nodes.
    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &Arc<NodeUserConfig>)> {
        self.0.iter()
    }

    /// Canonicalize node type URNs for all nodes in this collection.
    ///
    /// This rewrites each node's `type` to the canonical form and
    /// attaches node/pipeline context to any validation errors.
    fn canonicalize_plugin_urns(
        &mut self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
    ) -> Result<(), Error> {
        for (node_id, node) in self.0.iter_mut() {
            let mut updated = (**node).clone();
            let normalized = crate::node_urn::canonicalize_plugin_urn(updated.r#type.as_ref())
                .map_err(|e| {
                    if let Error::InvalidUserConfig { error } = e {
                        Error::InvalidNodeType {
                            context: Box::new(Context::new(
                                pipeline_group_id.clone(),
                                pipeline_id.clone(),
                            )),
                            node_id: node_id.clone(),
                            details: error,
                        }
                    } else {
                        e
                    }
                })?;
            updated.r#type = normalized;
            *node = Arc::new(updated);
        }
        Ok(())
    }

    /// Returns an iterator over node IDs.
    pub fn keys(&self) -> impl Iterator<Item = &NodeId> {
        self.0.keys()
    }
}

impl std::ops::Index<&str> for PipelineNodes {
    type Output = Arc<NodeUserConfig>;

    fn index(&self, id: &str) -> &Self::Output {
        &self.0[id]
    }
}

impl IntoIterator for PipelineNodes {
    type Item = (NodeId, Arc<NodeUserConfig>);
    type IntoIter = std::collections::hash_map::IntoIter<NodeId, Arc<NodeUserConfig>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(NodeId, Arc<NodeUserConfig>)> for PipelineNodes {
    fn from_iter<T: IntoIterator<Item = (NodeId, Arc<NodeUserConfig>)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl PipelineConfig {
    /// Create a new [`PipelineConfig`] from a JSON string.
    pub fn from_json(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        json_str: &str,
    ) -> Result<Self, Error> {
        let mut cfg: PipelineConfig =
            serde_json::from_str(json_str).map_err(|e| Error::DeserializationError {
                context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                format: "JSON".to_string(),
                details: e.to_string(),
            })?;

        cfg.canonicalize_plugin_urns(&pipeline_group_id, &pipeline_id)?;
        cfg.validate(&pipeline_group_id, &pipeline_id)?;
        Ok(cfg)
    }

    /// Create a new [`PipelineConfig`] from a YAML string.
    pub fn from_yaml(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        yaml_str: &str,
    ) -> Result<Self, Error> {
        let mut spec: PipelineConfig =
            serde_yaml::from_str(yaml_str).map_err(|e| Error::DeserializationError {
                context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                format: "YAML".to_string(),
                details: e.to_string(),
            })?;

        spec.canonicalize_plugin_urns(&pipeline_group_id, &pipeline_id)?;
        spec.validate(&pipeline_group_id, &pipeline_id)?;
        Ok(spec)
    }

    /// Returns the policy set for this pipeline, if defined.
    #[must_use]
    pub const fn policies(&self) -> Option<&Policies> {
        self.policies.as_ref()
    }

    /// Returns a reference to the main pipeline nodes.
    #[must_use]
    pub const fn nodes(&self) -> &PipelineNodes {
        &self.nodes
    }

    /// Returns an iterator visiting all nodes in the pipeline.
    pub fn node_iter(&self) -> impl Iterator<Item = (&NodeId, &Arc<NodeUserConfig>)> {
        self.nodes.iter()
    }

    /// Returns true if the pipeline graph is defined with top-level connections.
    #[must_use]
    pub fn has_connections(&self) -> bool {
        !self.connections.is_empty()
    }

    /// Returns an iterator over top-level pipeline connections.
    pub fn connection_iter(&self) -> impl Iterator<Item = &PipelineConnection> {
        self.connections.iter()
    }

    /// Creates a consuming iterator over the nodes in the pipeline.
    pub fn node_into_iter(self) -> impl Iterator<Item = (NodeId, Arc<NodeUserConfig>)> {
        self.nodes.into_iter()
    }

    /// Remove unconnected nodes from the main pipeline graph and return removed node descriptors.
    ///
    /// Connectivity is defined by top-level `connections`:
    /// - receiver: must have at least one outgoing connection
    /// - processor / processor_chain: must have at least one incoming and one outgoing connection
    /// - exporter: must have at least one incoming connection
    ///
    /// Removal is iterative. Removing one node can orphan additional nodes, which are removed in
    /// subsequent passes. Connections are pruned as nodes are removed.
    pub fn remove_unconnected_nodes(&mut self) -> Vec<(NodeId, NodeKind)> {
        let mut removed = Vec::new();

        loop {
            let connected = self.connected_sets();
            let mut to_remove = Vec::new();

            for (node_id, node_cfg) in self.nodes.iter() {
                let has_incoming = connected.incoming.contains(node_id);
                let has_outgoing = connected.outgoing.contains(node_id);
                let should_remove = match node_cfg.kind() {
                    NodeKind::Receiver => !has_outgoing,
                    NodeKind::Processor | NodeKind::ProcessorChain => {
                        !has_incoming || !has_outgoing
                    }
                    NodeKind::Exporter => !has_incoming,
                };

                if should_remove {
                    to_remove.push((node_id.clone(), node_cfg.kind()));
                }
            }

            if to_remove.is_empty() {
                break;
            }

            let removed_ids: HashSet<NodeId> = to_remove
                .iter()
                .map(|(node_id, _)| node_id.clone())
                .collect();

            for (node_id, node_kind) in &to_remove {
                let _ = self.nodes.0.remove(node_id);
                removed.push((node_id.clone(), *node_kind));
            }

            self.connections = self
                .connections
                .drain(..)
                .filter_map(|connection| prune_connection(connection, &removed_ids))
                .collect();
        }

        removed
    }

    fn connected_sets(&self) -> ConnectedSets {
        let mut incoming = HashSet::new();
        let mut outgoing = HashSet::new();

        for connection in &self.connections {
            for source in connection.from_sources() {
                let source_id = source.node_id().clone();
                if self.nodes.contains_key(source_id.as_ref()) {
                    _ = outgoing.insert(source_id);
                }
            }

            for target in connection.to_nodes() {
                if self.nodes.contains_key(target.as_ref()) {
                    _ = incoming.insert(target);
                }
            }
        }

        ConnectedSets { incoming, outgoing }
    }

    /// Builds a dedicated engine observability pipeline configuration.
    #[must_use]
    pub fn for_observability_pipeline(
        policies: Option<Policies>,
        nodes: PipelineNodes,
        connections: Vec<PipelineConnection>,
    ) -> Self {
        Self {
            r#type: PipelineType::Otap,
            policies,
            nodes,
            connections,
        }
    }

    /// Normalize plugin URNs for pipeline nodes.
    fn canonicalize_plugin_urns(
        &mut self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
    ) -> Result<(), Error> {
        self.nodes
            .canonicalize_plugin_urns(pipeline_group_id, pipeline_id)
    }

    /// Validate the pipeline specification.
    ///
    /// This method checks for:
    /// - Duplicate node IDs
    /// - Duplicate output ports (same source node + port name)
    /// - Invalid hyper-edges (missing source or target nodes)
    pub fn validate(
        &self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
    ) -> Result<(), Error> {
        let mut errors = Vec::new();

        self.validate_connections(
            &self.nodes,
            &self.connections,
            pipeline_group_id,
            pipeline_id,
            &mut errors,
        );

        if !errors.is_empty() {
            Err(Error::InvalidConfiguration { errors })
        } else {
            Ok(())
        }
    }

    fn validate_connections(
        &self,
        nodes: &PipelineNodes,
        connections: &[PipelineConnection],
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
        errors: &mut Vec<Error>,
    ) {
        for connection in connections {
            let source_selectors = connection.from_sources();
            let source_nodes = source_selectors
                .iter()
                .map(|source| source.node_id().clone())
                .collect::<Vec<_>>();
            let target_nodes = connection.to_nodes();
            let from_empty = source_selectors.is_empty();
            let to_empty = target_nodes.is_empty();
            if from_empty || to_empty {
                errors.push(Error::EmptyConnectionEndpointSet {
                    context: Box::new(Context::new(pipeline_group_id.clone(), pipeline_id.clone())),
                    from_empty,
                    to_empty,
                });
                continue;
            }
            if let Some(DispatchPolicy::Broadcast) = &connection.policies.dispatch
                && target_nodes.len() > 1
            {
                errors.push(Error::UnsupportedConnectionDispatchPolicy {
                    context: Box::new(Context::new(pipeline_group_id.clone(), pipeline_id.clone())),
                    dispatch_policy: DispatchPolicy::Broadcast,
                    source_nodes: source_nodes.into_boxed_slice(),
                    target_nodes: target_nodes.into_boxed_slice(),
                });
                continue;
            }

            let missing_targets: Vec<_> = target_nodes
                .iter()
                .filter(|target| !nodes.contains_key(target.as_ref()))
                .cloned()
                .collect();

            for source in source_selectors {
                let source_node = source.node_id().clone();
                let missing_source = !nodes.contains_key(source_node.as_ref());
                if missing_source || !missing_targets.is_empty() {
                    errors.push(Error::InvalidHyperEdgeSpec {
                        context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                        source_node: source_node.clone(),
                        missing_source,
                        details: Box::new(HyperEdgeSpecDetails {
                            target_nodes: target_nodes.clone(),
                            dispatch_policy: connection.effective_dispatch_policy(),
                            missing_targets: missing_targets.clone(),
                        }),
                    });
                    continue;
                }

                if let Some(node_cfg) = nodes.get(source_node.as_ref()) {
                    let source_kind = node_cfg.kind();
                    if !matches!(source_kind, NodeKind::Receiver | NodeKind::Processor) {
                        errors.push(Error::InvalidConnectionNodeKind {
                            context: Box::new(Context::new(
                                pipeline_group_id.clone(),
                                pipeline_id.clone(),
                            )),
                            node_id: source_node.clone(),
                            endpoint: "from".to_string(),
                            actual_kind: source_kind,
                            expected_kinds: vec![NodeKind::Receiver, NodeKind::Processor]
                                .into_boxed_slice(),
                        });
                        continue;
                    }
                    let resolved_port = source.resolved_output_port();
                    if !node_cfg.outputs.is_empty()
                        && !node_cfg
                            .outputs
                            .iter()
                            .any(|output| output == &resolved_port)
                    {
                        errors.push(Error::UndeclaredConnectionOutput {
                            context: Box::new(Context::new(
                                pipeline_group_id.clone(),
                                pipeline_id.clone(),
                            )),
                            source_node: source_node.clone(),
                            selected_output: resolved_port.clone(),
                            declared_outputs: node_cfg.outputs.clone().into_boxed_slice(),
                        });
                    }
                }
            }
        }

        // Only check for cycles if no hyper-edge errors.
        if errors.is_empty() {
            for cycle in Self::detect_cycles_from_connections(nodes, connections) {
                errors.push(Error::CycleDetected {
                    context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                    nodes: cycle,
                });
            }
        }
    }

    fn detect_cycles_from_connections(
        nodes: &PipelineNodes,
        connections: &[PipelineConnection],
    ) -> Vec<Vec<NodeId>> {
        fn visit(
            node: &NodeId,
            adjacency: &HashMap<NodeId, Vec<NodeId>>,
            visiting: &mut HashSet<NodeId>,
            visited: &mut HashSet<NodeId>,
            current_path: &mut Vec<NodeId>,
            cycles: &mut Vec<Vec<NodeId>>,
        ) {
            if visited.contains(node) {
                return;
            }
            if visiting.contains(node) {
                if let Some(pos) = current_path.iter().position(|n| n == node) {
                    cycles.push(current_path[pos..].to_vec());
                }
                return;
            }
            let _ = visiting.insert(node.clone());
            current_path.push(node.clone());

            if let Some(targets) = adjacency.get(node) {
                for target in targets {
                    visit(target, adjacency, visiting, visited, current_path, cycles);
                }
            }

            let _ = visiting.remove(node);
            let _ = visited.insert(node.clone());
            let _ = current_path.pop();
        }

        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for connection in connections {
            let sources = connection.from_nodes();
            let targets = connection.to_nodes();
            for source in &sources {
                let entry = adjacency.entry(source.clone()).or_default();
                for target in &targets {
                    if !entry.contains(target) {
                        entry.push(target.clone());
                    }
                }
            }
        }

        let mut visiting = HashSet::new();
        let mut current_path = Vec::new();
        let mut visited = HashSet::new();
        let mut cycles = Vec::new();

        for node in nodes.keys() {
            if !visited.contains(node) {
                visit(
                    node,
                    &adjacency,
                    &mut visiting,
                    &mut visited,
                    &mut current_path,
                    &mut cycles,
                );
            }
        }

        cycles
    }
}

struct ConnectedSets {
    incoming: HashSet<NodeId>,
    outgoing: HashSet<NodeId>,
}

fn prune_connection(
    connection: PipelineConnection,
    removed_ids: &HashSet<NodeId>,
) -> Option<PipelineConnection> {
    let mut kept_sources = connection
        .from_sources()
        .into_iter()
        .filter(|source| !removed_ids.contains(source.node_id()))
        .collect::<Vec<_>>();
    let mut kept_targets = connection
        .to_nodes()
        .into_iter()
        .filter(|target| !removed_ids.contains(target))
        .collect::<Vec<_>>();

    if kept_sources.is_empty() || kept_targets.is_empty() {
        return None;
    }

    kept_sources.sort_by(|left, right| {
        let left_key = (
            left.node_id().as_ref(),
            left.output_port().map(|port| port.as_ref()),
        );
        let right_key = (
            right.node_id().as_ref(),
            right.output_port().map(|port| port.as_ref()),
        );
        left_key.cmp(&right_key)
    });
    kept_sources.dedup_by(|left, right| {
        left.node_id().as_ref() == right.node_id().as_ref()
            && left.output_port().map(|port| port.as_ref())
                == right.output_port().map(|port| port.as_ref())
    });

    kept_targets.sort_unstable_by(|left, right| left.as_ref().cmp(right.as_ref()));
    kept_targets.dedup_by(|left, right| left.as_ref() == right.as_ref());

    let from = if kept_sources.len() == 1 {
        ConnectionSourceSet::One(kept_sources.remove(0))
    } else {
        ConnectionSourceSet::Many(kept_sources)
    };
    let to = if kept_targets.len() == 1 {
        ConnectionNodeSet::One(kept_targets.remove(0))
    } else {
        ConnectionNodeSet::Many(kept_targets)
    };

    Some(PipelineConnection {
        from,
        to,
        policies: connection.policies,
    })
}

/// A builder for constructing a [`PipelineConfig`]. This type is used
/// for easy testing of the PipelineNodes logic.
pub struct PipelineConfigBuilder {
    description: Option<Description>,
    nodes: HashMap<NodeId, NodeUserConfig>,
    duplicate_nodes: Vec<NodeId>,
    pending_connections: Vec<PendingConnection>,
}

struct PendingConnection {
    src: NodeId,
    output_port: PortName,
    targets: HashSet<NodeId>,
    dispatch: DispatchPolicy,
}

impl PipelineConfigBuilder {
    /// Create a new pipeline builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            description: None,
            nodes: HashMap::new(),
            duplicate_nodes: Vec::new(),
            pending_connections: Vec::new(),
        }
    }

    /// Set the description of the pipeline.
    #[must_use]
    pub fn description(mut self, description: Description) -> Self {
        self.description = Some(description);
        self
    }

    /// Add a node with a given id and type URN.
    /// Optionally provide config.
    pub fn add_node<S: Into<NodeId>, U: Into<NodeUrn>>(
        mut self,
        id: S,
        node_type: U,
        config: Option<Value>,
    ) -> Self {
        let id = id.into();
        let node_type = node_type.into();
        if self.nodes.contains_key(&id) {
            self.duplicate_nodes.push(id.clone());
        } else {
            _ = self.nodes.insert(
                id.clone(),
                NodeUserConfig {
                    r#type: node_type,
                    description: None,
                    telemetry_attributes: HashMap::new(),
                    outputs: Vec::new(),
                    default_output: None,
                    config: config.unwrap_or(Value::Null),
                },
            );
        }
        self
    }

    /// Add a receiver node.
    pub fn add_receiver<S: Into<NodeId>, U: Into<NodeUrn>>(
        self,
        id: S,
        node_type: U,
        config: Option<Value>,
    ) -> Self {
        self.add_node(id, node_type, config)
    }

    /// Add a processor node.
    pub fn add_processor<S: Into<NodeId>, U: Into<NodeUrn>>(
        self,
        id: S,
        node_type: U,
        config: Option<Value>,
    ) -> Self {
        self.add_node(id, node_type, config)
    }

    /// Add an exporter node.
    pub fn add_exporter<S: Into<NodeId>, U: Into<NodeUrn>>(
        self,
        id: S,
        node_type: U,
        config: Option<Value>,
    ) -> Self {
        self.add_node(id, node_type, config)
    }

    /// Connects a source node output port to one or more target nodes
    /// with a given dispatch policy.
    pub fn connect<S, P, T, I>(
        mut self,
        src: S,
        output_port: P,
        targets: I,
        dispatch: DispatchPolicy,
    ) -> Self
    where
        S: Into<NodeId>,
        P: Into<PortName>,
        T: Into<NodeId>,
        I: IntoIterator<Item = T>,
    {
        self.pending_connections.push(PendingConnection {
            src: src.into(),
            output_port: output_port.into(),
            targets: targets.into_iter().map(Into::into).collect(),
            dispatch,
        });
        self
    }

    /// Connects a source node default output port to one or more target nodes
    /// with a broadcast dispatch policy.
    pub fn broadcast<S, T, I>(self, src: S, targets: I) -> Self
    where
        S: Into<NodeId>,
        T: Into<NodeId>,
        I: IntoIterator<Item = T>,
    {
        self.broadcast_output(src, DEFAULT_CONNECTION_SOURCE_PORT, targets)
    }

    /// Connects a source node output port to one or more target nodes
    /// with a broadcast dispatch policy.
    pub fn broadcast_output<S, P, T, I>(self, src: S, output_port: P, targets: I) -> Self
    where
        S: Into<NodeId>,
        P: Into<PortName>,
        T: Into<NodeId>,
        I: IntoIterator<Item = T>,
    {
        self.connect(src, output_port, targets, DispatchPolicy::Broadcast)
    }

    /// Connects a source node default output port to one or more target nodes
    /// with a one-of dispatch policy.
    pub fn one_of<S, T, I>(self, src: S, targets: I) -> Self
    where
        S: Into<NodeId>,
        T: Into<NodeId>,
        I: IntoIterator<Item = T>,
    {
        self.one_of_output(src, DEFAULT_CONNECTION_SOURCE_PORT, targets)
    }

    /// Connects a source node output port to one or more target nodes
    /// with a one-of dispatch policy.
    pub fn one_of_output<S, P, T, I>(self, src: S, output_port: P, targets: I) -> Self
    where
        S: Into<NodeId>,
        P: Into<PortName>,
        T: Into<NodeId>,
        I: IntoIterator<Item = T>,
    {
        self.connect(src, output_port, targets, DispatchPolicy::OneOf)
    }

    /// Connects a source node default output to a single destination node
    /// using one-of dispatch.
    pub fn to<S, T>(self, src: S, dst: T) -> Self
    where
        S: Into<NodeId>,
        T: Into<NodeId>,
    {
        self.one_of(src, std::iter::once(dst))
    }

    /// Connects a source node named output to a single destination node
    /// using one-of dispatch.
    pub fn to_output<S, P, T>(self, src: S, output_port: P, dst: T) -> Self
    where
        S: Into<NodeId>,
        P: Into<PortName>,
        T: Into<NodeId>,
    {
        self.one_of_output(src, output_port, std::iter::once(dst))
    }

    /// Validate and build the pipeline specification.
    ///
    /// We collect all possible errors (duplicate nodes, duplicate output ports,
    /// missing source/targets, invalid edges, cycles) into one `InvalidHyperDag`
    /// report. This lets callers see every problem at once, rather than failing
    /// fast on the first error.
    pub fn build<T, P>(
        mut self,
        pipeline_type: PipelineType,
        pipeline_group_id: T,
        pipeline_id: P,
    ) -> Result<PipelineConfig, Error>
    where
        T: Into<PipelineGroupId>,
        P: Into<PipelineId>,
    {
        let mut errors = Vec::new();
        let pipeline_group_id = pipeline_group_id.into();
        let pipeline_id = pipeline_id.into();

        // Report duplicated nodes
        for node_id in &self.duplicate_nodes {
            errors.push(Error::DuplicateNode {
                context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                node_id: node_id.clone(),
            });
        }

        // Detect duplicate output ports (same src + port used twice)
        {
            let mut seen_ports = HashSet::new();
            for conn in &self.pending_connections {
                let key = (conn.src.clone(), conn.output_port.clone());
                if !seen_ports.insert(key.clone()) {
                    errors.push(Error::DuplicateOutputPort {
                        context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                        source_node: conn.src.clone(),
                        port: conn.output_port.clone(),
                    });
                }
            }
        }

        // Process each pending connection (skipping any 2nd+ duplicates)
        let mut inserted_ports = HashSet::new();
        let mut built_connections = Vec::new();
        for conn in self.pending_connections {
            let dispatch_policy = conn.dispatch.clone();
            let key = (conn.src.clone(), conn.output_port.clone());
            if !inserted_ports.insert(key.clone()) {
                // skip this duplicate
                continue;
            }

            // check that source & all targets exist
            let mut missing = Vec::new();
            let src_exists = self.nodes.contains_key(&conn.src);
            for t in &conn.targets {
                if !self.nodes.contains_key(t) {
                    missing.push(t.clone());
                }
            }

            // if anything is missing, record as InvalidHyperEdgeSpec
            if !src_exists || !missing.is_empty() {
                errors.push(Error::InvalidHyperEdgeSpec {
                    context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                    source_node: conn.src.clone(),
                    missing_source: !src_exists,
                    details: Box::new(HyperEdgeSpecDetails {
                        target_nodes: conn.targets.iter().cloned().collect(),
                        dispatch_policy: dispatch_policy.clone(),
                        missing_targets: missing,
                    }),
                });
                continue;
            }

            // record declared output for this source node
            if let Some(node) = self.nodes.get_mut(&conn.src) {
                node.add_output(conn.output_port.clone());
            }

            let mut targets = conn.targets.iter().cloned().collect::<Vec<_>>();
            targets.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));
            let source = if conn.output_port.as_ref() == DEFAULT_CONNECTION_SOURCE_PORT {
                ConnectionSource::from_node(conn.src)
            } else {
                ConnectionSource::from_node_with_port(conn.src, conn.output_port)
            };
            built_connections.push(PipelineConnection {
                from: ConnectionSourceSet::One(source),
                to: ConnectionNodeSet::Many(targets),
                policies: ConnectionPolicies {
                    dispatch: (dispatch_policy != DispatchPolicy::OneOf).then_some(dispatch_policy),
                },
            });
        }

        if !errors.is_empty() {
            Err(Error::InvalidConfiguration { errors })
        } else {
            // Build the spec and validate it
            let mut spec = PipelineConfig {
                nodes: self
                    .nodes
                    .into_iter()
                    .map(|(id, node)| (id, Arc::new(node)))
                    .collect(),
                connections: built_connections,
                policies: None,
                r#type: pipeline_type,
            };

            spec.canonicalize_plugin_urns(&pipeline_group_id, &pipeline_id)?;
            spec.validate(&pipeline_group_id, &pipeline_id)?;
            Ok(spec)
        }
    }
}

impl Default for PipelineConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::node::NodeKind;
    use crate::pipeline::DispatchPolicy;
    use crate::pipeline::telemetry::metrics::MetricsConfig;
    use crate::pipeline::telemetry::metrics::readers::periodic::MetricsPeriodicExporterConfig;
    use crate::pipeline::telemetry::metrics::readers::{
        MetricsReaderConfig, MetricsReaderPeriodicConfig,
    };
    use crate::pipeline::telemetry::{AttributeValue, TelemetryConfig};
    use crate::pipeline::{PipelineConfigBuilder, PipelineType};
    use serde_json::json;

    #[test]
    fn test_duplicate_node_errors() {
        let result = PipelineConfigBuilder::new()
            .add_receiver("A", "urn:test:example:receiver", None)
            .add_processor("A", "urn:test:example:processor", None) // duplicate
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match result {
            Err(Error::InvalidConfiguration { errors }) => {
                // Should only report one DuplicateNode
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::DuplicateNode { node_id, .. } if node_id == "A" => {}
                    other => panic!("expected DuplicateNode(\"A\"), got {other:?}"),
                }
            }
            other => panic!("expected Err(InvalidPipelineSpec), got {other:?}"),
        }
    }

    #[test]
    fn test_duplicate_output_port_errors() {
        let result = PipelineConfigBuilder::new()
            .add_receiver("A", "urn:test:example:receiver", None)
            .add_exporter("B", "urn:test:example:exporter", None)
            .one_of_output("A", "p", ["B"])
            .one_of_output("A", "p", ["B"]) // duplicate port on A
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match result {
            Err(Error::InvalidConfiguration { errors }) => {
                // One DuplicateOutPort, no InvalidHyperEdge, no cycles
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::DuplicateOutputPort {
                        source_node, port, ..
                    } if source_node == "A" && port == "p" => {}
                    other => panic!("expected DuplicateOutPort(A, p), got {other:?}"),
                }
            }
            other => panic!("expected Err(InvalidPipelineSpec), got {other:?}"),
        }
    }

    #[test]
    fn test_missing_source_error() {
        let result = PipelineConfigBuilder::new()
            .add_receiver("B", "urn:test:example:receiver", None)
            .connect("X", "out", ["B"], DispatchPolicy::Broadcast) // X does not exist
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match result {
            Err(Error::InvalidConfiguration { errors }) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::InvalidHyperEdgeSpec {
                        source_node,
                        missing_source,
                        details,
                        ..
                    } if source_node == "X"
                        && *missing_source
                        && details.missing_targets.is_empty() => {}
                    other => panic!("expected InvalidHyperEdge missing_source, got {other:?}"),
                }
            }
            other => panic!("expected Err(InvalidPipelineSpec), got {other:?}"),
        }
    }

    #[test]
    fn test_missing_target_error() {
        let result = PipelineConfigBuilder::new()
            .add_receiver("A", "urn:test:example:receiver", None)
            .connect("A", "out", ["Y"], DispatchPolicy::Broadcast) // Y does not exist
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match result {
            Err(Error::InvalidConfiguration { errors }) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::InvalidHyperEdgeSpec {
                        source_node,
                        missing_source,
                        details,
                        ..
                    } if source_node == "A"
                        && !*missing_source
                        && details.missing_targets.as_slice() == ["Y"]
                        && details.target_nodes.as_slice() == ["Y"] => {}
                    other => panic!("expected InvalidHyperEdge missing_targets, got {other:?}"),
                }
            }
            other => panic!("expected Err(InvalidPipelineSpec), got {other:?}"),
        }
    }

    #[test]
    fn test_builder_rejects_empty_target_set() {
        let result = PipelineConfigBuilder::new()
            .add_receiver("A", "urn:test:example:receiver", None)
            .connect(
                "A",
                "out",
                std::iter::empty::<&str>(),
                DispatchPolicy::OneOf,
            )
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match result {
            Err(Error::InvalidConfiguration { errors }) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::EmptyConnectionEndpointSet {
                        from_empty,
                        to_empty,
                        ..
                    } => {
                        assert!(!from_empty);
                        assert!(*to_empty);
                    }
                    other => panic!("expected EmptyConnectionEndpointSet, got {other:?}"),
                }
            }
            other => panic!("expected Err(InvalidPipelineSpec), got {other:?}"),
        }
    }

    #[test]
    fn test_cycle_detection_error() {
        let result = PipelineConfigBuilder::new()
            .add_processor("A", "urn:test:example:processor", None)
            .add_processor("B", "urn:test:example:processor", None)
            .one_of_output("A", "p", ["B"])
            .one_of_output("B", "p", ["A"])
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match result {
            Err(Error::InvalidConfiguration { errors }) => {
                // exactly one cycle error
                let mut found = false;
                for err in errors {
                    if let Error::CycleDetected { nodes, .. } = err {
                        // cycle should include A and B
                        assert!(nodes.contains(&"A".into()));
                        assert!(nodes.contains(&"B".into()));
                        found = true;
                    }
                }
                assert!(found, "expected a CycleDetected error");
            }
            other => panic!("expected Err(InvalidPipelineSpec), got {other:?}"),
        }
    }

    #[test]
    fn test_successful_simple_build() {
        let dag = PipelineConfigBuilder::new()
            .add_receiver(
                "Start",
                "urn:test:example:receiver",
                Some(json!({"foo": 1})),
            )
            .add_exporter("End", "urn:test:example:exporter", None)
            .broadcast_output("Start", "out", ["End"])
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match dag {
            Ok(pipeline_spec) => {
                // two nodes, one edge on Start
                assert_eq!(pipeline_spec.nodes.len(), 2);
                let start = &pipeline_spec.nodes["Start"];
                assert_eq!(start.outputs, vec![crate::PortName::from("out")]);
                assert_eq!(pipeline_spec.connection_iter().count(), 1);
                let conn = pipeline_spec
                    .connection_iter()
                    .next()
                    .expect("missing connection");
                assert_eq!(
                    conn.from_sources()
                        .iter()
                        .map(|source| source.node_id().as_ref())
                        .collect::<Vec<_>>(),
                    vec!["Start"]
                );
                assert_eq!(
                    conn.to_nodes()
                        .iter()
                        .map(|id| id.as_ref())
                        .collect::<Vec<_>>(),
                    vec!["End"]
                );
            }
            Err(e) => panic!("expected successful build, got {e:?}"),
        }
    }

    #[test]
    fn test_builder_to_uses_default_output() {
        let dag = PipelineConfigBuilder::new()
            .add_receiver(
                "Start",
                "urn:test:example:receiver",
                Some(json!({"foo": 1})),
            )
            .add_exporter("End", "urn:test:example:exporter", None)
            .to("Start", "End")
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match dag {
            Ok(pipeline_spec) => {
                let start = &pipeline_spec.nodes["Start"];
                assert_eq!(start.outputs, vec![crate::PortName::from("default")]);
                let conn = pipeline_spec
                    .connection_iter()
                    .next()
                    .expect("missing connection");
                assert!(
                    conn.from_sources()
                        .iter()
                        .all(|source| source.output_port().is_none()),
                    "default output should be implicit in source selector"
                );
            }
            Err(e) => panic!("expected successful build, got {e:?}"),
        }
    }

    #[test]
    fn test_builder_to_output_uses_named_output() {
        let dag = PipelineConfigBuilder::new()
            .add_receiver(
                "Start",
                "urn:test:example:receiver",
                Some(json!({"foo": 1})),
            )
            .add_exporter("End", "urn:test:example:exporter", None)
            .to_output("Start", "alt", "End")
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match dag {
            Ok(pipeline_spec) => {
                let start = &pipeline_spec.nodes["Start"];
                assert_eq!(start.outputs, vec![crate::PortName::from("alt")]);
                let conn = pipeline_spec
                    .connection_iter()
                    .next()
                    .expect("missing connection");
                let from_sources = conn.from_sources();
                assert_eq!(from_sources.len(), 1);
                assert_eq!(
                    from_sources[0].output_port().map(|port| port.as_ref()),
                    Some("alt")
                );
            }
            Err(e) => panic!("expected successful build, got {e:?}"),
        }
    }

    #[test]
    fn test_valid_complex_pipeline_spec() {
        let dag = PipelineConfigBuilder::new()
            // ----- TRACES pipeline -----
            .add_receiver(
                "receiver_otlp_traces",
                "urn:test:example:receiver",
                Some(json!({"desc": "OTLP trace receiver"})),
            )
            .add_processor(
                "processor_batch_traces",
                "urn:test:example:processor",
                Some(json!({"name": "batch_traces"})),
            )
            .add_processor(
                "processor_resource_traces",
                "urn:test:example:processor",
                Some(json!({"name": "resource_traces"})),
            )
            .add_processor(
                "processor_traces_to_metrics",
                "urn:test:example:processor",
                Some(json!({"desc": "convert traces to metrics"})),
            )
            .add_exporter(
                "exporter_otlp_traces",
                "urn:test:example:exporter",
                Some(json!({"desc": "OTLP trace exporter"})),
            )
            .one_of("receiver_otlp_traces", ["processor_batch_traces"])
            .one_of("processor_batch_traces", ["processor_resource_traces"])
            .one_of("processor_resource_traces", ["processor_traces_to_metrics"])
            .one_of_output(
                "processor_resource_traces",
                "out2",
                ["exporter_otlp_traces"],
            )
            // ----- METRICS pipeline -----
            .add_receiver(
                "receiver_otlp_metrics",
                "urn:test:example:receiver",
                Some(json!({"desc": "OTLP metric receiver"})),
            )
            .add_processor(
                "processor_batch_metrics",
                "urn:test:example:processor",
                Some(json!({"name": "batch_metrics"})),
            )
            .add_processor(
                "processor_metrics_to_events",
                "urn:test:example:processor",
                Some(json!({"desc": "convert metrics to events"})),
            )
            .add_exporter(
                "exporter_prometheus",
                "urn:test:example:exporter",
                Some(json!({"desc": "Prometheus exporter"})),
            )
            .add_exporter(
                "exporter_otlp_metrics",
                "urn:test:example:exporter",
                Some(json!({"desc": "OTLP metric exporter"})),
            )
            .one_of("receiver_otlp_metrics", ["processor_batch_metrics"])
            .one_of("processor_batch_metrics", ["processor_metrics_to_events"])
            .one_of_output("processor_batch_metrics", "out2", ["exporter_prometheus"])
            .one_of_output("processor_batch_metrics", "out3", ["exporter_otlp_metrics"])
            .one_of("processor_traces_to_metrics", ["processor_batch_metrics"])
            // ----- LOGS pipeline -----
            .add_receiver(
                "receiver_filelog",
                "urn:test:example:receiver",
                Some(json!({"desc": "file log receiver"})),
            )
            .add_receiver(
                "receiver_syslog",
                "urn:test:example:receiver",
                Some(json!({"desc": "syslog receiver"})),
            )
            .add_processor(
                "processor_filter_logs",
                "urn:test:example:processor",
                Some(json!({"name": "filter_logs"})),
            )
            .add_processor(
                "processor_logs_to_events",
                "urn:test:example:processor",
                Some(json!({"desc": "convert logs to events"})),
            )
            .add_exporter(
                "exporter_otlp_logs",
                "urn:test:example:exporter",
                Some(json!({"desc": "OTLP log exporter"})),
            )
            .one_of("receiver_filelog", ["processor_filter_logs"])
            .one_of("receiver_syslog", ["processor_filter_logs"])
            .one_of("processor_filter_logs", ["processor_logs_to_events"])
            .one_of_output("processor_filter_logs", "out2", ["exporter_otlp_logs"])
            // ----- EVENTS pipeline -----
            .add_receiver(
                "receiver_some_events",
                "urn:test:example:receiver",
                Some(json!({"desc": "custom event receiver"})),
            )
            .add_processor(
                "processor_enrich_events",
                "urn:test:example:processor",
                Some(json!({"name": "enrich_events"})),
            )
            .add_exporter(
                "exporter_queue_events",
                "urn:test:example:exporter",
                Some(json!({"desc": "push events to queue"})),
            )
            .one_of("receiver_some_events", ["processor_enrich_events"])
            .one_of("processor_enrich_events", ["exporter_queue_events"])
            .one_of("processor_logs_to_events", ["processor_enrich_events"])
            .one_of("processor_metrics_to_events", ["processor_enrich_events"])
            // Finalize build
            .build(PipelineType::Otap, "pgroup", "pipeline");

        // Assert the DAG is valid and acyclic
        match dag {
            Ok(pipeline_spec) => {
                assert_eq!(pipeline_spec.nodes.len(), 18);
            }
            Err(e) => panic!("Failed to build pipeline DAG: {e:?}"),
        }
    }

    #[test]
    fn test_telemetry_config_deserialization() {
        let yaml_data: &str = r#"
            reporting_channel_size: 200
            reporting_interval: "5s"
            resource:
              service.name: "my_service"
              service.version: "1.2.3"
            metrics:
              readers:
                - periodic:
                    interval: "15s"
                    exporter:
                      console: {}
            "#;
        let config: TelemetryConfig = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(config.reporting_channel_size, 200);
        assert_eq!(config.reporting_interval.as_secs(), 5);

        let resource_attrs = &config.resource;

        if let AttributeValue::String(ref val) = resource_attrs["service.name"] {
            assert_eq!(val, "my_service");
        } else {
            panic!("Expected service.name to be a string");
        }
        if let AttributeValue::String(ref val) = resource_attrs["service.version"] {
            assert_eq!(val, "1.2.3");
        } else {
            panic!("Expected service.version to be a string");
        }

        let readers = &config.metrics.readers;
        assert_eq!(readers.len(), 1);
        if let MetricsReaderConfig::Periodic(periodic_config) = &readers[0] {
            assert_eq!(periodic_config.interval.as_secs(), 15);
            if MetricsPeriodicExporterConfig::Console != periodic_config.exporter {
                panic!("Expected Console exporter config");
            }
        } else {
            panic!("Expected Periodic reader config");
        }
    }

    #[test]
    fn test_metrics_reader_deserialization() {
        let yaml_data = r#"
            readers:
              - periodic:
                  interval: "10s"
                  exporter:
                    console:
            "#;
        let config: MetricsConfig = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(config.readers.len(), 1);
        if let MetricsReaderConfig::Periodic(periodic_config) = &config.readers[0] {
            assert_eq!(periodic_config.interval.as_secs(), 10);
            if MetricsPeriodicExporterConfig::Console != periodic_config.exporter {
                panic!("Expected Console exporter config");
            }
        } else {
            panic!("Expected Periodic reader config");
        }
    }

    #[test]
    fn test_metrics_reader_periodic_config_deserialization() {
        let yaml_data = r#"
            interval: "20s"
            exporter:
              console:
            "#;
        let metrics_reader_periodic_config: MetricsReaderPeriodicConfig =
            serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(metrics_reader_periodic_config.interval.as_secs(), 20);
        if MetricsPeriodicExporterConfig::Console != metrics_reader_periodic_config.exporter {
            panic!("Expected Console exporter config");
        }
    }

    #[test]
    fn test_metrics_reader_periodic_config_deserialization_unknown_exporter() {
        let yaml_data = r#"
            interval: "20s"
            exporter:
              unknown: {}
            "#;
        let metrics_reader_periodic_config_result: Result<
            MetricsReaderPeriodicConfig,
            serde_yaml::Error,
        > = serde_yaml::from_str(yaml_data);
        if let Err(e) = metrics_reader_periodic_config_result {
            let err_msg = e.to_string();
            assert!(err_msg.contains("unknown field `unknown`"));
        } else {
            panic!("Expected deserialization to fail due to unknown exporter");
        }
    }

    #[test]
    fn test_for_observability_pipeline_defaults() {
        let nodes: super::PipelineNodes = serde_yaml::from_str(
            r#"
itr:
  type: "urn:otel:internal_telemetry:receiver"
  config: {}
sink:
  type: "urn:otel:console:exporter"
  config: {}
"#,
        )
        .expect("nodes should parse");
        let connections: Vec<super::PipelineConnection> = serde_yaml::from_str(
            r#"
- from: itr
  to: sink
"#,
        )
        .expect("connections should parse");

        let config = super::PipelineConfig::for_observability_pipeline(None, nodes, connections);
        assert_eq!(config.node_iter().count(), 2);
        assert_eq!(config.connection_iter().count(), 1);
        assert!(config.policies().is_none());
    }

    #[test]
    fn test_pipeline_from_yaml_normalizes_plugin_urns() {
        let yaml = r#"
            nodes:
              receiver:
                type: "otlp:receiver"
                config: {}
              processor:
                type: "attribute:processor"
                config: {}
              exporter:
                type: "urn:otel:otlp:exporter"
                config: {}
            connections:
              - from: receiver
                to: processor
              - from: processor
                to: exporter
        "#;

        let config = super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml)
            .expect("should parse");
        assert_eq!(
            config.nodes["receiver"].r#type.as_ref(),
            "urn:otel:otlp:receiver"
        );
        assert_eq!(
            config.nodes["processor"].r#type.as_ref(),
            "urn:otel:attribute:processor"
        );
        assert_eq!(
            config.nodes["exporter"].r#type.as_ref(),
            "urn:otel:otlp:exporter"
        );
    }

    #[test]
    fn test_pipeline_from_yaml_rejects_invalid_urns_with_doc_link() {
        let yaml = r#"
            nodes:
              exporter:
                type: "urn:otel:otap:perf:exporter"
                config: {}
        "#;

        let err =
            super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("invalid plugin urn"));
        assert!(message.contains("urn:<namespace>:<id>:<kind>"));
        assert!(message.contains("rust/otap-dataflow/docs/urns.md"));
    }

    #[test]
    fn test_pipeline_from_yaml_with_connections() {
        let yaml = r#"
            nodes:
              receiver:
                type: "otlp:receiver"
                config: {}
              processor:
                type: "attribute:processor"
                config: {}
              exporter:
                type: "urn:otel:otlp:exporter"
                config: {}
            connections:
              - from: receiver
                to: processor
              - from: processor
                to: exporter
        "#;

        let config = super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml)
            .expect("should parse");
        assert!(config.has_connections());
        assert_eq!(config.connection_iter().count(), 2);
        for connection in config.connection_iter() {
            for source in connection.from_sources() {
                assert!(source.output_port().is_none());
            }
            assert!(matches!(
                connection.effective_dispatch_policy(),
                DispatchPolicy::OneOf
            ));
        }
        assert_eq!(
            config.nodes["receiver"].r#type.as_ref(),
            "urn:otel:otlp:receiver"
        );
        assert_eq!(
            config.nodes["processor"].r#type.as_ref(),
            "urn:otel:attribute:processor"
        );
        assert_eq!(
            config.nodes["exporter"].r#type.as_ref(),
            "urn:otel:otlp:exporter"
        );
    }

    #[test]
    fn test_pipeline_from_yaml_with_fan_in_connection() {
        let yaml = r#"
            nodes:
              receiver_a:
                type: "otlp:receiver"
                config: {}
              receiver_b:
                type: "otlp:receiver"
                config: {}
              exporter:
                type: "noop:exporter"
                config: {}
            connections:
              - from: [receiver_a, receiver_b]
                to: exporter
        "#;

        let config = super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml)
            .expect("should parse");
        let connection = config
            .connection_iter()
            .next()
            .expect("missing expected connection");
        let mut from_nodes = connection
            .from_nodes()
            .into_iter()
            .map(|node| node.as_ref().to_string())
            .collect::<Vec<_>>();
        from_nodes.sort();
        assert_eq!(from_nodes, vec!["receiver_a", "receiver_b"]);
        let to_nodes = connection.to_nodes();
        assert_eq!(to_nodes.len(), 1);
        assert_eq!(to_nodes[0].as_ref(), "exporter");
    }

    #[test]
    fn test_pipeline_from_yaml_with_named_source_port() {
        let yaml = r#"
            nodes:
              router:
                type: "type_router:processor"
                outputs: ["logs", "metrics", "traces"]
                config: {}
              exporter:
                type: "noop:exporter"
                config: {}
            connections:
              - from: router["logs"]
                to: exporter
        "#;

        let config = super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml)
            .expect("should parse");
        let connection = config
            .connection_iter()
            .next()
            .expect("missing expected connection");
        let from_sources = connection.from_sources();
        assert_eq!(from_sources.len(), 1);
        assert_eq!(from_sources[0].node_id().as_ref(), "router");
        assert_eq!(
            from_sources[0].output_port().map(|port| port.as_ref()),
            Some("logs")
        );
    }

    #[test]
    fn test_pipeline_from_yaml_rejects_unknown_declared_output_port_usage() {
        let yaml = r#"
            nodes:
              router:
                type: "type_router:processor"
                outputs: ["metrics"]
                config: {}
              exporter:
                type: "noop:exporter"
                config: {}
            connections:
              - from: router["logs"]
                to: exporter
        "#;

        let err =
            super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml).unwrap_err();
        match err {
            Error::InvalidConfiguration { errors } => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::UndeclaredConnectionOutput {
                        source_node,
                        selected_output,
                        declared_outputs,
                        ..
                    } => {
                        assert_eq!(source_node.as_ref(), "router");
                        assert_eq!(selected_output.as_ref(), "logs");
                        assert_eq!(declared_outputs.as_ref(), ["metrics"]);
                    }
                    other => panic!("expected UndeclaredConnectionOutput, got {other:?}"),
                }
            }
            other => panic!("expected InvalidConfiguration, got {other:?}"),
        }
    }

    #[test]
    fn test_pipeline_from_yaml_rejects_invalid_from_node_kind() {
        let yaml = r#"
            nodes:
              exporter:
                type: "noop:exporter"
                config: {}
              processor:
                type: "attribute:processor"
                config: {}
            connections:
              - from: exporter
                to: processor
        "#;

        let err =
            super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml).unwrap_err();
        match err {
            Error::InvalidConfiguration { errors } => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::InvalidConnectionNodeKind {
                        node_id,
                        endpoint,
                        actual_kind,
                        expected_kinds,
                        ..
                    } => {
                        assert_eq!(node_id.as_ref(), "exporter");
                        assert_eq!(endpoint, "from");
                        assert!(matches!(actual_kind, NodeKind::Exporter));
                        assert_eq!(
                            expected_kinds.as_ref(),
                            [NodeKind::Receiver, NodeKind::Processor]
                        );
                    }
                    other => panic!("expected InvalidConnectionNodeKind, got {other:?}"),
                }
            }
            other => panic!("expected InvalidConfiguration, got {other:?}"),
        }
    }

    #[test]
    fn test_pipeline_from_yaml_with_policies_dispatch_one_of() {
        let yaml = r#"
            nodes:
              receiver:
                type: "otlp:receiver"
                config: {}
              exporter_a:
                type: "noop:exporter"
                config: {}
              exporter_b:
                type: "noop:exporter"
                config: {}
            connections:
              - from: receiver
                to: [exporter_a, exporter_b]
                policies:
                  dispatch: one_of
        "#;

        let config = super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml)
            .expect("should parse");
        let connection = config
            .connection_iter()
            .next()
            .expect("missing expected connection");
        assert!(matches!(
            connection.effective_dispatch_policy(),
            DispatchPolicy::OneOf
        ));
    }

    #[test]
    fn test_pipeline_from_yaml_with_single_destination_broadcast_noop() {
        let yaml = r#"
            nodes:
              receiver:
                type: "otlp:receiver"
                config: {}
              exporter:
                type: "noop:exporter"
                config: {}
            connections:
              - from: receiver
                to: exporter
                policies:
                  dispatch: broadcast
        "#;

        let config = super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml)
            .expect("should parse");
        let connection = config
            .connection_iter()
            .next()
            .expect("missing expected connection");
        assert!(matches!(
            connection.effective_dispatch_policy(),
            DispatchPolicy::OneOf
        ));
    }

    #[test]
    fn test_pipeline_from_yaml_rejects_multi_destination_broadcast_dispatch() {
        let yaml = r#"
            nodes:
              receiver:
                type: "otlp:receiver"
                config: {}
              exporter_a:
                type: "noop:exporter"
                config: {}
              exporter_b:
                type: "noop:exporter"
                config: {}
            connections:
              - from: receiver
                to: [exporter_a, exporter_b]
                policies:
                  dispatch: broadcast
        "#;

        let err =
            super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml).unwrap_err();
        match err {
            Error::InvalidConfiguration { errors } => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::UnsupportedConnectionDispatchPolicy {
                        dispatch_policy,
                        source_nodes,
                        target_nodes,
                        ..
                    } => {
                        assert!(matches!(dispatch_policy, DispatchPolicy::Broadcast));
                        assert_eq!(source_nodes.as_ref(), ["receiver"]);
                        assert_eq!(target_nodes.as_ref(), ["exporter_a", "exporter_b"]);
                    }
                    other => panic!("expected UnsupportedConnectionDispatchPolicy, got {other:?}"),
                }
            }
            other => panic!("expected InvalidConfiguration, got {other:?}"),
        }
    }

    #[test]
    fn test_pipeline_from_yaml_rejects_empty_from_set() {
        let yaml = r#"
            nodes:
              receiver:
                type: "otlp:receiver"
                config: {}
              exporter:
                type: "noop:exporter"
                config: {}
            connections:
              - from: []
                to: exporter
        "#;

        let err =
            super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml).unwrap_err();
        match err {
            Error::InvalidConfiguration { errors } => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::EmptyConnectionEndpointSet {
                        from_empty,
                        to_empty,
                        ..
                    } => {
                        assert!(*from_empty);
                        assert!(!to_empty);
                    }
                    other => panic!("expected EmptyConnectionEndpointSet, got {other:?}"),
                }
            }
            other => panic!("expected InvalidConfiguration, got {other:?}"),
        }
    }

    #[test]
    fn test_pipeline_from_yaml_rejects_empty_to_set() {
        let yaml = r#"
            nodes:
              receiver:
                type: "otlp:receiver"
                config: {}
              exporter:
                type: "noop:exporter"
                config: {}
            connections:
              - from: receiver
                to: []
        "#;

        let err =
            super::PipelineConfig::from_yaml("group".into(), "pipe".into(), yaml).unwrap_err();
        match err {
            Error::InvalidConfiguration { errors } => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::EmptyConnectionEndpointSet {
                        from_empty,
                        to_empty,
                        ..
                    } => {
                        assert!(!from_empty);
                        assert!(*to_empty);
                    }
                    other => panic!("expected EmptyConnectionEndpointSet, got {other:?}"),
                }
            }
            other => panic!("expected InvalidConfiguration, got {other:?}"),
        }
    }

    #[test]
    fn test_remove_unconnected_receiver_with_no_outgoing_connection() {
        // A receiver with no outgoing connection should be removed.
        let yaml = r#"
            nodes:
              connected_recv:
                type: "urn:test:a:receiver"
                config: {}
              disconnected_recv:
                type: "urn:test:b:receiver"
                config: {}
              exporter:
                type: "urn:test:c:exporter"
                config: {}
            connections:
              - from: connected_recv
                to: exporter
        "#;
        let mut config = super::PipelineConfig::from_yaml("g".into(), "p".into(), yaml).unwrap();

        let removed = config.remove_unconnected_nodes();
        let removed_ids: Vec<_> = removed.iter().map(|(id, _)| id.as_ref()).collect();
        assert!(
            removed_ids.contains(&"disconnected_recv"),
            "should remove disconnected receiver, got: {removed_ids:?}"
        );
        assert!(config.nodes().contains_key("connected_recv"));
        assert!(config.nodes().contains_key("exporter"));
        assert!(!config.nodes().contains_key("disconnected_recv"));
    }

    #[test]
    fn test_remove_unconnected_processor_not_a_destination() {
        // A processor not referenced as any node's destination should be removed.
        let yaml = r#"
            nodes:
              recv:
                type: "urn:test:a:receiver"
                config: {}
              connected_proc:
                type: "urn:test:b:processor"
                config: {}
              orphan_proc:
                type: "urn:test:c:processor"
                config: {}
              exporter:
                type: "urn:test:d:exporter"
                config: {}
            connections:
              - from: recv
                to: connected_proc
              - from: connected_proc
                to: exporter
              - from: orphan_proc
                to: exporter
        "#;
        let mut config = super::PipelineConfig::from_yaml("g".into(), "p".into(), yaml).unwrap();

        let removed = config.remove_unconnected_nodes();
        let removed_ids: Vec<_> = removed.iter().map(|(id, _)| id.as_ref()).collect();
        assert!(
            removed_ids.contains(&"orphan_proc"),
            "should remove orphan processor, got: {removed_ids:?}"
        );
        assert!(config.nodes().contains_key("recv"));
        assert!(config.nodes().contains_key("connected_proc"));
        assert!(config.nodes().contains_key("exporter"));
    }

    #[test]
    fn test_remove_unconnected_exporter_not_a_destination() {
        // An exporter not referenced as any node's destination should be removed.
        let yaml = r#"
            nodes:
              recv:
                type: "urn:test:a:receiver"
                config: {}
              connected_exp:
                type: "urn:test:b:exporter"
                config: {}
              orphan_exp:
                type: "urn:test:c:exporter"
                config: {}
            connections:
              - from: recv
                to: connected_exp
        "#;
        let mut config = super::PipelineConfig::from_yaml("g".into(), "p".into(), yaml).unwrap();

        let removed = config.remove_unconnected_nodes();
        let removed_ids: Vec<_> = removed.iter().map(|(id, _)| id.as_ref()).collect();
        assert!(
            removed_ids.contains(&"orphan_exp"),
            "should remove orphan exporter, got: {removed_ids:?}"
        );
        assert!(config.nodes().contains_key("recv"));
        assert!(config.nodes().contains_key("connected_exp"));
    }

    #[test]
    fn test_remove_unconnected_cascading_removal() {
        // 3-level cascade: orphan_proc1 has no incoming edges so it is removed on pass 1.
        // orphan_proc2's only incoming was from orphan_proc1, so it becomes orphaned on pass 2.
        // orphan_exp's only incoming was from orphan_proc2, so it becomes orphaned on pass 3.
        // Meanwhile, recv -> connected_exp stays intact.
        let yaml = r#"
            nodes:
              recv:
                type: "urn:test:a:receiver"
                config: {}
              orphan_proc1:
                type: "urn:test:b:processor"
                config: {}
              orphan_proc2:
                type: "urn:test:c:processor"
                config: {}
              connected_exp:
                type: "urn:test:d:exporter"
                config: {}
              orphan_exp:
                type: "urn:test:e:exporter"
                config: {}
            connections:
              - from: recv
                to: connected_exp
              - from: orphan_proc1
                to: orphan_proc2
              - from: orphan_proc2
                to: orphan_exp
        "#;
        let mut config = super::PipelineConfig::from_yaml("g".into(), "p".into(), yaml).unwrap();

        let removed = config.remove_unconnected_nodes();
        let removed_ids: Vec<_> = removed.iter().map(|(id, _)| id.as_ref()).collect();
        // Pass 1: orphan_proc1 removed (no incoming edges)
        assert!(
            removed_ids.contains(&"orphan_proc1"),
            "orphan_proc1 should be removed, got: {removed_ids:?}"
        );
        // Pass 2: orphan_proc2 removed (only incoming was orphan_proc1)
        assert!(
            removed_ids.contains(&"orphan_proc2"),
            "orphan_proc2 should cascade-remove, got: {removed_ids:?}"
        );
        // Pass 3: orphan_exp removed (only incoming was orphan_proc2)
        assert!(
            removed_ids.contains(&"orphan_exp"),
            "orphan_exp should cascade-remove, got: {removed_ids:?}"
        );
        // Only the connected chain should remain
        assert_eq!(config.nodes().len(), 2);
        assert!(config.nodes().contains_key("recv"));
        assert!(config.nodes().contains_key("connected_exp"));
    }

    #[test]
    fn test_remove_unconnected_nodes_fully_connected_no_removals() {
        // A fully connected pipeline should have no removals.
        let yaml = r#"
            nodes:
              recv:
                type: "urn:test:a:receiver"
                config: {}
              proc:
                type: "urn:test:b:processor"
                config: {}
              exp:
                type: "urn:test:c:exporter"
                config: {}
            connections:
              - from: recv
                to: proc
              - from: proc
                to: exp
        "#;
        let mut config = super::PipelineConfig::from_yaml("g".into(), "p".into(), yaml).unwrap();

        let removed = config.remove_unconnected_nodes();
        assert!(
            removed.is_empty(),
            "fully connected pipeline should have no removals, got: {removed:?}"
        );
        assert_eq!(config.nodes().len(), 3);
    }

    #[test]
    fn test_remove_unconnected_all_nodes_removed() {
        // Receiver has no outgoing connection, processor has no incoming
        // connection, and exporter is only reachable from that orphan processor.
        // All nodes should eventually be removed.
        let yaml = r#"
            nodes:
              recv:
                type: "urn:test:a:receiver"
                config: {}
              proc:
                type: "urn:test:b:processor"
                config: {}
              exp:
                type: "urn:test:c:exporter"
                config: {}
            connections:
              - from: proc
                to: exp
        "#;
        let mut config = super::PipelineConfig::from_yaml("g".into(), "p".into(), yaml).unwrap();

        let removed = config.remove_unconnected_nodes();
        let removed_ids: Vec<_> = removed.iter().map(|(id, _)| id.as_ref()).collect();
        assert!(
            removed_ids.contains(&"recv"),
            "recv should be removed (no out_ports)"
        );
        assert!(
            removed_ids.contains(&"proc"),
            "proc should be removed (no incoming)"
        );
        assert!(
            removed_ids.contains(&"exp"),
            "exp should be removed (cascade)"
        );
        assert_eq!(
            config.nodes().len(),
            0,
            "all nodes should be removed, got: {:?}",
            config.nodes().keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_remove_unconnected_prunes_connections_for_removed_nodes() {
        let yaml = r#"
            nodes:
              recv:
                type: "urn:test:a:receiver"
                config: {}
              orphan_proc:
                type: "urn:test:b:processor"
                config: {}
              exp:
                type: "urn:test:c:exporter"
                config: {}
            connections:
              - from: orphan_proc
                to: exp
        "#;
        let mut config = super::PipelineConfig::from_yaml("g".into(), "p".into(), yaml).unwrap();

        let removed = config.remove_unconnected_nodes();
        let removed_ids: Vec<_> = removed.iter().map(|(id, _)| id.as_ref()).collect();
        assert_eq!(removed_ids.len(), 3);
        assert!(config.connection_iter().next().is_none());
    }
}
