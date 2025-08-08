// SPDX-License-Identifier: Apache-2.0

//! Pipeline configuration specification.

use crate::error::{Context, Error, HyperEdgeSpecDetails};
use crate::node::{DispatchStrategy, HyperEdgeConfig, NodeKind, NodeUserConfig};
use crate::{Description, NodeId, PipelineGroupId, PipelineId, PortName, Urn};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

/// A pipeline configuration describing the interconnections between nodes.
/// A pipeline is a directed acyclic graph that could be qualified as a hyper-DAG:
/// - "Hyper" because the edges connecting the nodes can be hyper-edges.  
/// - A node can be connected to multiple outgoing nodes.  
/// - The way messages are dispatched over each hyper-edge is defined by a dispatch strategy representing
///   different communication model semantics. For example, it could be a broadcast channel that sends
///   the same message to all destination nodes, or it might have a round-robin or least-loaded semantic,
///   similar to an SPMC channel.
///
/// This configuration defines the pipeline’s nodes, the interconnections (hyper-edges), and pipeline-level settings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineConfig {
    /// Type of the pipeline, which determines the type of PData it processes.
    ///
    /// Note: Even though technically our engine can support several types of pdata, we
    /// are focusing our efforts on the OTAP pipeline (hence the default value).
    #[serde(default = "default_pipeline_type")]
    r#type: PipelineType,

    /// Settings for this pipeline.
    #[serde(default)]
    settings: PipelineSettings,

    /// All nodes in this pipeline, keyed by node ID.
    ///
    /// Note: We use `Arc<NodeUserConfig>` to allow sharing the same pipeline configuration
    /// across multiple cores/threads without cloning the entire configuration.
    nodes: HashMap<NodeId, Arc<NodeUserConfig>>,
}

fn default_pipeline_type() -> PipelineType {
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
/// A configuration for a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineSettings {
    /// The default size of the node control message channels.
    /// These channels are used for sending control messages by the pipeline engine to nodes.
    #[serde(default = "default_node_ctrl_msg_channel_size")]
    pub default_node_ctrl_msg_channel_size: usize,

    /// The default size of the pipeline control message channels.
    /// This MPSC channel is used for sending control messages from nodes to the pipeline engine.
    #[serde(default = "default_pipeline_ctrl_msg_channel_size")]
    pub default_pipeline_ctrl_msg_channel_size: usize,

    /// The default size of the pdata channels.
    #[serde(default = "default_pdata_channel_size")]
    pub default_pdata_channel_size: usize,
}

fn default_node_ctrl_msg_channel_size() -> usize {
    100
}
fn default_pipeline_ctrl_msg_channel_size() -> usize {
    100
}
fn default_pdata_channel_size() -> usize {
    100
}

impl Default for PipelineSettings {
    fn default() -> Self {
        Self {
            default_node_ctrl_msg_channel_size: default_node_ctrl_msg_channel_size(),
            default_pipeline_ctrl_msg_channel_size: default_pipeline_ctrl_msg_channel_size(),
            default_pdata_channel_size: default_pdata_channel_size(),
        }
    }
}

impl PipelineConfig {
    /// Create a new [`PipelineConfig`] from a JSON string.
    pub fn from_json(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        json_str: &str,
    ) -> Result<Self, Error> {
        let cfg: PipelineConfig =
            serde_json::from_str(json_str).map_err(|e| Error::DeserializationError {
                context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                format: "JSON".to_string(),
                details: e.to_string(),
            })?;

        cfg.validate(&pipeline_group_id, &pipeline_id)?;
        Ok(cfg)
    }

    /// Create a new [`PipelineConfig`] from a YAML string.
    pub fn from_yaml(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        yaml_str: &str,
    ) -> Result<Self, Error> {
        let spec: PipelineConfig =
            serde_yaml::from_str(yaml_str).map_err(|e| Error::DeserializationError {
                context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                format: "YAML".to_string(),
                details: e.to_string(),
            })?;

        spec.validate(&pipeline_group_id, &pipeline_id)?;
        Ok(spec)
    }

    /// Load a [`PipelineConfig`] from a JSON file.
    pub fn from_json_file<P: AsRef<Path>>(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        path: P,
    ) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(|e| Error::FileReadError {
            context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
            details: e.to_string(),
        })?;
        Self::from_json(pipeline_group_id, pipeline_id, &contents)
    }

    /// Load a [`PipelineConfig`] from a YAML file.
    pub fn from_yaml_file<P: AsRef<Path>>(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        path: P,
    ) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(|e| Error::FileReadError {
            context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
            details: e.to_string(),
        })?;
        Self::from_yaml(pipeline_group_id, pipeline_id, &contents)
    }

    /// Load a [`PipelineConfig`] from a file, automatically detecting the format based on file extension.
    ///
    /// Supports:
    /// - JSON files: `.json`
    /// - YAML files: `.yaml`, `.yml`
    pub fn from_file<P: AsRef<Path>>(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        path: P,
    ) -> Result<Self, Error> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension.as_deref() {
            Some("json") => Self::from_json_file(pipeline_group_id, pipeline_id, path),
            Some("yaml") | Some("yml") => {
                Self::from_yaml_file(pipeline_group_id, pipeline_id, path)
            }
            _ => {
                let context = Context::new(pipeline_group_id, pipeline_id);
                let details = format!(
                    "Unsupported file extension: {}. Supported extensions are: .json, .yaml, .yml",
                    extension.unwrap_or_else(|| "<none>".to_string())
                );
                Err(Error::FileReadError { context, details })
            }
        }
    }

    /// Returns the general settings for this pipeline.
    #[must_use]
    pub fn pipeline_settings(&self) -> &PipelineSettings {
        &self.settings
    }

    /// Returns an iterator visiting all nodes in the pipeline.
    pub fn node_iter(&self) -> impl Iterator<Item = (&NodeId, &Arc<NodeUserConfig>)> {
        self.nodes.iter()
    }

    /// Creates a consuming iterator over the nodes in the pipeline.
    pub fn node_into_iter(self) -> impl Iterator<Item = (NodeId, Arc<NodeUserConfig>)> {
        self.nodes.into_iter()
    }

    /// Validate the pipeline specification.
    ///
    /// This method checks for:
    /// - Duplicate node IDs
    /// - Duplicate out-ports (same source node + port name)
    /// - Invalid hyper-edges (missing source or target nodes)
    /// - Cycles in the DAG
    pub fn validate(
        &self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
    ) -> Result<(), Error> {
        let mut errors = Vec::new();

        // Check for invalid hyper-edges (references to non-existent nodes)
        for (node_id, node) in self.nodes.iter() {
            for edge in node.out_ports.values() {
                let mut missing_targets = Vec::new();

                for target in &edge.destinations {
                    if !self.nodes.contains_key(target) {
                        missing_targets.push(target.clone());
                    }
                }

                if !missing_targets.is_empty() {
                    errors.push(Error::InvalidHyperEdgeSpec {
                        context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                        source_node: node_id.clone(),
                        missing_source: false, // source exists since we're iterating over nodes
                        details: Box::new(HyperEdgeSpecDetails {
                            target_nodes: edge.destinations.iter().cloned().collect(),
                            dispatch_strategy: edge.dispatch_strategy.clone(),
                            missing_targets,
                        }),
                    });
                }
            }
        }

        // Check for cycles if no errors so far
        if errors.is_empty() {
            let cycles = self.detect_cycles();
            for cycle in cycles {
                errors.push(Error::CycleDetected {
                    context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                    nodes: cycle,
                });
            }
        }

        if !errors.is_empty() {
            Err(Error::InvalidConfiguration { errors })
        } else {
            Ok(())
        }
    }

    fn detect_cycles(&self) -> Vec<Vec<NodeId>> {
        fn visit(
            node: &NodeId,
            nodes: &HashMap<NodeId, Arc<NodeUserConfig>>,
            visiting: &mut HashSet<NodeId>,
            visited: &mut HashSet<NodeId>,
            current_path: &mut Vec<NodeId>,
            cycles: &mut Vec<Vec<NodeId>>,
        ) {
            if visited.contains(node) {
                return;
            }
            if visiting.contains(node) {
                // Cycle found
                if let Some(pos) = current_path.iter().position(|n| n == node) {
                    cycles.push(current_path[pos..].to_vec());
                }
                return;
            }
            _ = visiting.insert(node.clone());
            current_path.push(node.clone());

            if let Some(n) = nodes.get(node) {
                for edge in n.out_ports.values() {
                    for tgt in &edge.destinations {
                        visit(tgt, nodes, visiting, visited, current_path, cycles);
                    }
                }
            }

            _ = visiting.remove(node);
            _ = visited.insert(node.clone());
            _ = current_path.pop();
        }

        let mut visiting = HashSet::new();
        let mut current_path = Vec::new();
        let mut visited = HashSet::new();
        let mut cycles = Vec::new();

        for node in self.nodes.keys() {
            if !visited.contains(node) {
                visit(
                    node,
                    &self.nodes,
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

/// A builder for constructing a [`PipelineConfig`].
pub struct PipelineConfigBuilder {
    description: Option<Description>,
    nodes: HashMap<NodeId, NodeUserConfig>,
    duplicate_nodes: Vec<NodeId>,
    pending_connections: Vec<PendingConnection>,
}

struct PendingConnection {
    src: NodeId,
    out_port: PortName,
    targets: HashSet<NodeId>,
    strategy: DispatchStrategy,
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

    /// Add a node with a given id and kind.
    /// Optionally provide config.
    pub fn add_node<S: Into<NodeId>, U: Into<Urn>>(
        mut self,
        id: S,
        kind: NodeKind,
        plugin_urn: U,
        config: Option<Value>,
    ) -> Self {
        let id = id.into();
        let plugin_urn = plugin_urn.into();
        if self.nodes.contains_key(&id) {
            self.duplicate_nodes.push(id.clone());
        } else {
            _ = self.nodes.insert(
                id.clone(),
                NodeUserConfig {
                    kind,
                    plugin_urn,
                    description: None,
                    out_ports: HashMap::new(),
                    config: config.unwrap_or(Value::Null),
                },
            );
        }
        self
    }

    /// Add a receiver node.
    pub fn add_receiver<S: Into<NodeId>, U: Into<Urn>>(
        self,
        id: S,
        plugin_urn: U,
        config: Option<Value>,
    ) -> Self {
        self.add_node(id, NodeKind::Receiver, plugin_urn, config)
    }

    /// Add a processor node.
    pub fn add_processor<S: Into<NodeId>, U: Into<Urn>>(
        self,
        id: S,
        plugin_urn: U,
        config: Option<Value>,
    ) -> Self {
        self.add_node(id, NodeKind::Processor, plugin_urn, config)
    }

    /// Add an exporter node.
    pub fn add_exporter<S: Into<NodeId>, U: Into<Urn>>(
        self,
        id: S,
        plugin_urn: U,
        config: Option<Value>,
    ) -> Self {
        self.add_node(id, NodeKind::Exporter, plugin_urn, config)
    }

    /// Connect source node's out_port to one or more target nodes
    /// with a given dispatch strategy.
    pub fn connect<S, P, T, I>(
        mut self,
        src: S,
        out_port: P,
        targets: I,
        strategy: DispatchStrategy,
    ) -> Self
    where
        S: Into<NodeId>,
        P: Into<PortName>,
        T: Into<NodeId>,
        I: IntoIterator<Item = T>,
    {
        self.pending_connections.push(PendingConnection {
            src: src.into(),
            out_port: out_port.into(),
            targets: targets.into_iter().map(Into::into).collect(),
            strategy,
        });
        self
    }

    /// Connect source node's out_port to one or more target nodes
    /// with a round-robin dispatch strategy.
    pub fn broadcast<S, P, T, I>(self, src: S, out_port: P, targets: I) -> Self
    where
        S: Into<NodeId>,
        P: Into<PortName>,
        T: Into<NodeId>,
        I: IntoIterator<Item = T>,
    {
        self.connect(src, out_port, targets, DispatchStrategy::Broadcast)
    }

    /// Connect source node's out_port to one or more target nodes
    /// with a round-robin dispatch strategy.
    pub fn round_robin<S, P, T, I>(self, src: S, out_port: P, targets: I) -> Self
    where
        S: Into<NodeId>,
        P: Into<PortName>,
        T: Into<NodeId>,
        I: IntoIterator<Item = T>,
    {
        self.connect(src, out_port, targets, DispatchStrategy::RoundRobin)
    }

    /// Connect source node's out_port to one or more target nodes
    /// with a random dispatch strategy.
    pub fn random<S, P, T, I>(self, src: S, out_port: P, targets: I) -> Self
    where
        S: Into<NodeId>,
        P: Into<PortName>,
        T: Into<NodeId>,
        I: IntoIterator<Item = T>,
    {
        self.connect(src, out_port, targets, DispatchStrategy::Random)
    }

    /// Connect source node's out_port to one or more target nodes
    /// with a least-loaded dispatch strategy.
    pub fn least_loaded<S, P, T, I>(self, src: S, out_port: P, targets: I) -> Self
    where
        S: Into<NodeId>,
        P: Into<PortName>,
        T: Into<NodeId>,
        I: IntoIterator<Item = T>,
    {
        self.connect(src, out_port, targets, DispatchStrategy::LeastLoaded)
    }

    /// Validate and build the pipeline specification.
    ///
    /// We collect all possible errors (duplicate nodes, duplicate out-ports,
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

        // Detect duplicate out‐ports (same src + port used twice)
        {
            let mut seen_ports = HashSet::new();
            for conn in &self.pending_connections {
                let key = (conn.src.clone(), conn.out_port.clone());
                if !seen_ports.insert(key.clone()) {
                    errors.push(Error::DuplicateOutPort {
                        context: Context::new(pipeline_group_id.clone(), pipeline_id.clone()),
                        source_node: conn.src.clone(),
                        port: conn.out_port.clone(),
                    });
                }
            }
        }

        // Process each pending connection (skipping any 2nd+ duplicates)
        let mut inserted_ports = HashSet::new();
        for conn in self.pending_connections {
            let key = (conn.src.clone(), conn.out_port.clone());
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
                        dispatch_strategy: conn.strategy,
                        missing_targets: missing,
                    }),
                });
                continue;
            }

            // finally, insert into the node’s out_ports
            if let Some(node) = self.nodes.get_mut(&conn.src) {
                let _ = node.out_ports.insert(
                    conn.out_port.clone(),
                    HyperEdgeConfig {
                        destinations: conn.targets.clone(),
                        dispatch_strategy: conn.strategy,
                    },
                );
            }
        }

        if !errors.is_empty() {
            Err(Error::InvalidConfiguration { errors })
        } else {
            // Build the spec and validate it
            let spec = PipelineConfig {
                nodes: self
                    .nodes
                    .into_iter()
                    .map(|(id, node)| (id, Arc::new(node)))
                    .collect(),
                settings: PipelineSettings::default(),
                r#type: pipeline_type,
            };

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
    use crate::node::DispatchStrategy;
    use crate::pipeline::{PipelineConfigBuilder, PipelineType};
    use serde_json::json;

    #[test]
    fn test_duplicate_node_errors() {
        let result = PipelineConfigBuilder::new()
            .add_receiver("A", "urn:test:receiver", None)
            .add_processor("A", "urn:test:processor", None) // duplicate
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
    fn test_duplicate_outport_errors() {
        let result = PipelineConfigBuilder::new()
            .add_receiver("A", "urn:test:receiver", None)
            .add_exporter("B", "urn:test:exporter", None)
            .round_robin("A", "p", ["B"])
            .round_robin("A", "p", ["B"]) // duplicate port on A
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match result {
            Err(Error::InvalidConfiguration { errors }) => {
                // One DuplicateOutPort, no InvalidHyperEdge, no cycles
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::DuplicateOutPort {
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
            .add_receiver("B", "urn:test:receiver", None)
            .connect("X", "out", ["B"], DispatchStrategy::Broadcast) // X does not exist
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
            .add_receiver("A", "urn:test:receiver", None)
            .connect("A", "out", ["Y"], DispatchStrategy::Broadcast) // Y does not exist
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
    fn test_cycle_detection_error() {
        let result = PipelineConfigBuilder::new()
            .add_processor("A", "urn:test:processor", None)
            .add_processor("B", "urn:test:processor", None)
            .round_robin("A", "p", ["B"])
            .round_robin("B", "p", ["A"])
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
            .add_receiver("Start", "urn:test:receiver", Some(json!({"foo": 1})))
            .add_exporter("End", "urn:test:exporter", None)
            .broadcast("Start", "out", ["End"])
            .build(PipelineType::Otap, "pgroup", "pipeline");

        match dag {
            Ok(pipeline_spec) => {
                // two nodes, one edge on Start
                assert_eq!(pipeline_spec.nodes.len(), 2);
                let start = &pipeline_spec.nodes["Start"];
                assert_eq!(start.out_ports.len(), 1);
                let edge = &start.out_ports["out"];
                assert!(edge.destinations.contains("End"));
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
                "urn:test:receiver",
                Some(json!({"desc": "OTLP trace receiver"})),
            )
            .add_processor(
                "processor_batch_traces",
                "urn:test:processor",
                Some(json!({"name": "batch_traces"})),
            )
            .add_processor(
                "processor_resource_traces",
                "urn:test:processor",
                Some(json!({"name": "resource_traces"})),
            )
            .add_processor(
                "processor_traces_to_metrics",
                "urn:test:processor",
                Some(json!({"desc": "convert traces to metrics"})),
            )
            .add_exporter(
                "exporter_otlp_traces",
                "urn:test:exporter",
                Some(json!({"desc": "OTLP trace exporter"})),
            )
            .round_robin("receiver_otlp_traces", "out", ["processor_batch_traces"])
            .round_robin(
                "processor_batch_traces",
                "out",
                ["processor_resource_traces"],
            )
            .round_robin(
                "processor_resource_traces",
                "out",
                ["processor_traces_to_metrics"],
            )
            .round_robin(
                "processor_resource_traces",
                "out2",
                ["exporter_otlp_traces"],
            )
            // ----- METRICS pipeline -----
            .add_receiver(
                "receiver_otlp_metrics",
                "urn:test:receiver",
                Some(json!({"desc": "OTLP metric receiver"})),
            )
            .add_processor(
                "processor_batch_metrics",
                "urn:test:processor",
                Some(json!({"name": "batch_metrics"})),
            )
            .add_processor(
                "processor_metrics_to_events",
                "urn:test:processor",
                Some(json!({"desc": "convert metrics to events"})),
            )
            .add_exporter(
                "exporter_prometheus",
                "urn:test:exporter",
                Some(json!({"desc": "Prometheus exporter"})),
            )
            .add_exporter(
                "exporter_otlp_metrics",
                "urn:test:exporter",
                Some(json!({"desc": "OTLP metric exporter"})),
            )
            .round_robin("receiver_otlp_metrics", "out", ["processor_batch_metrics"])
            .round_robin(
                "processor_batch_metrics",
                "out",
                ["processor_metrics_to_events"],
            )
            .round_robin("processor_batch_metrics", "out2", ["exporter_prometheus"])
            .round_robin("processor_batch_metrics", "out3", ["exporter_otlp_metrics"])
            .round_robin(
                "processor_traces_to_metrics",
                "out",
                ["processor_batch_metrics"],
            )
            // ----- LOGS pipeline -----
            .add_receiver(
                "receiver_filelog",
                "urn:test:receiver",
                Some(json!({"desc": "file log receiver"})),
            )
            .add_receiver(
                "receiver_syslog",
                "urn:test:receiver",
                Some(json!({"desc": "syslog receiver"})),
            )
            .add_processor(
                "processor_filter_logs",
                "urn:test:processor",
                Some(json!({"name": "filter_logs"})),
            )
            .add_processor(
                "processor_logs_to_events",
                "urn:test:processor",
                Some(json!({"desc": "convert logs to events"})),
            )
            .add_exporter(
                "exporter_otlp_logs",
                "urn:test:exporter",
                Some(json!({"desc": "OTLP log exporter"})),
            )
            .round_robin("receiver_filelog", "out", ["processor_filter_logs"])
            .round_robin("receiver_syslog", "out", ["processor_filter_logs"])
            .round_robin("processor_filter_logs", "out", ["processor_logs_to_events"])
            .round_robin("processor_filter_logs", "out2", ["exporter_otlp_logs"])
            // ----- EVENTS pipeline -----
            .add_receiver(
                "receiver_some_events",
                "urn:test:receiver",
                Some(json!({"desc": "custom event receiver"})),
            )
            .add_processor(
                "processor_enrich_events",
                "urn:test:processor",
                Some(json!({"name": "enrich_events"})),
            )
            .add_exporter(
                "exporter_queue_events",
                "urn:test:exporter",
                Some(json!({"desc": "push events to queue"})),
            )
            .round_robin("receiver_some_events", "out", ["processor_enrich_events"])
            .round_robin("processor_enrich_events", "out", ["exporter_queue_events"])
            .round_robin(
                "processor_logs_to_events",
                "out",
                ["processor_enrich_events"],
            )
            .round_robin(
                "processor_metrics_to_events",
                "out",
                ["processor_enrich_events"],
            )
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
    fn test_from_json_file() {
        // Use a dedicated test fixture file
        let file_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/test_pipeline.json"
        );

        // Test loading from JSON file
        let result = super::PipelineConfig::from_json_file(
            "test_group".into(),
            "test_pipeline".into(),
            file_path,
        );

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.nodes.len(), 2);
        assert!(config.nodes.contains_key("receiver1"));
        assert!(config.nodes.contains_key("exporter1"));
    }

    #[test]
    fn test_from_yaml_file() {
        // Use a dedicated test fixture file
        let file_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/test_pipeline.yaml"
        );

        // Test loading from YAML file
        let result = super::PipelineConfig::from_yaml_file(
            "test_group".into(),
            "test_pipeline".into(),
            file_path,
        );

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.nodes.len(), 3);
        assert!(config.nodes.contains_key("receiver1"));
        assert!(config.nodes.contains_key("processor1"));
        assert!(config.nodes.contains_key("exporter1"));
    }

    #[test]
    fn test_from_json_file_nonexistent_file() {
        let result = super::PipelineConfig::from_json_file(
            "test_group".into(),
            "test_pipeline".into(),
            "/nonexistent/path/pipeline.json",
        );

        assert!(result.is_err());
        match result {
            Err(Error::FileReadError { .. }) => {}
            other => panic!("Expected FileReadError, got {other:?}"),
        }
    }

    #[test]
    fn test_from_yaml_file_nonexistent_file() {
        let result = super::PipelineConfig::from_yaml_file(
            "test_group".into(),
            "test_pipeline".into(),
            "/nonexistent/path/pipeline.yaml",
        );

        assert!(result.is_err());
        match result {
            Err(Error::FileReadError { .. }) => {}
            other => panic!("Expected FileReadError, got {other:?}"),
        }
    }

    #[test]
    fn test_from_file_json_extension() {
        // Test auto-detection with .json extension
        let file_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/test_pipeline.json"
        );

        let result = super::PipelineConfig::from_file(
            "test_group".into(),
            "test_pipeline".into(),
            file_path,
        );

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.nodes.len(), 2);
        assert!(config.nodes.contains_key("receiver1"));
        assert!(config.nodes.contains_key("exporter1"));
    }

    #[test]
    fn test_from_file_yaml_extension() {
        // Test auto-detection with .yaml extension
        let file_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/test_pipeline.yaml"
        );

        let result = super::PipelineConfig::from_file(
            "test_group".into(),
            "test_pipeline".into(),
            file_path,
        );

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.nodes.len(), 3);
        assert!(config.nodes.contains_key("receiver1"));
        assert!(config.nodes.contains_key("processor1"));
        assert!(config.nodes.contains_key("exporter1"));
    }

    #[test]
    fn test_from_file_yml_extension() {
        // Test auto-detection with .yml extension (alternative YAML extension)
        // We'll create a simple test using a path that would have .yml extension
        let result = super::PipelineConfig::from_file(
            "test_group".into(),
            "test_pipeline".into(),
            "/nonexistent/test.yml", // This will fail at file reading, but should pass extension detection
        );

        assert!(result.is_err());
        // Should be FileReadError (file doesn't exist), not unsupported extension
        match result {
            Err(Error::FileReadError { details, .. }) => {
                // Make sure it's a file read error and not an extension error
                assert!(!details.contains("Unsupported file extension"));
            }
            other => panic!("Expected FileReadError, got {other:?}"),
        }
    }

    #[test]
    fn test_from_file_unsupported_extension() {
        // Test with unsupported file extension
        let result = super::PipelineConfig::from_file(
            "test_group".into(),
            "test_pipeline".into(),
            "/some/path/config.txt",
        );

        assert!(result.is_err());
        match result {
            Err(Error::FileReadError { details, .. }) => {
                assert!(details.contains("Unsupported file extension"));
                assert!(details.contains("txt"));
                assert!(details.contains(".json, .yaml, .yml"));
            }
            other => panic!("Expected FileReadError with unsupported extension, got {other:?}"),
        }
    }

    #[test]
    fn test_from_file_no_extension() {
        // Test with file that has no extension
        let result = super::PipelineConfig::from_file(
            "test_group".into(),
            "test_pipeline".into(),
            "/some/path/config",
        );

        assert!(result.is_err());
        match result {
            Err(Error::FileReadError { details, .. }) => {
                assert!(details.contains("Unsupported file extension"));
                assert!(details.contains("<none>"));
                assert!(details.contains(".json, .yaml, .yml"));
            }
            other => panic!("Expected FileReadError with no extension, got {other:?}"),
        }
    }
}
