// SPDX-License-Identifier: Apache-2.0

//! Pipeline configuration specification.

use crate::error::{Context, Error, HyperEdgeSpecDetails};
use crate::node::{DispatchStrategy, HyperEdgeConfig, NodeKind, NodeUserConfig};
use crate::{Description, NamespaceId, NodeId, PipelineId, PortName, Urn};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;

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
    r#type: PipelineType,

    /// Optional description of the pipeline’s purpose.
    description: Option<Description>,

    /// Settings for this pipeline.
    #[serde(default)]
    settings: PipelineSettings,

    /// All nodes in this pipeline, keyed by node ID.
    nodes: HashMap<NodeId, Rc<NodeUserConfig>>,
}

fn default_control_channel_size() -> usize {
    100
}
fn default_pdata_channel_size() -> usize {
    100
}

/// The type of pipeline, which can be either OTLP (OpenTelemetry Protocol) or
/// OTAP (OpenTelemetry with Apache Arrow Protocol).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum PipelineType {
    /// OpenTelemetry Protocol (OTLP) pipeline.
    OTLP,
    /// OpenTelemetry with Apache Arrow Protocol (OTAP) pipeline.
    OTAP,
}
/// A configuration for a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineSettings {
    /// The default size of the control channels.
    #[serde(default = "default_control_channel_size")]
    pub default_control_channel_size: usize,

    /// The default size of the pdata channels.
    #[serde(default = "default_pdata_channel_size")]
    pub default_pdata_channel_size: usize,
}

impl Default for PipelineSettings {
    fn default() -> Self {
        Self {
            default_control_channel_size: default_control_channel_size(),
            default_pdata_channel_size: default_pdata_channel_size(),
        }
    }
}

impl PipelineConfig {
    /// Create a new PipelineSpec from a JSON string.
    pub fn from_json(
        namespace_id: NamespaceId,
        pipeline_id: PipelineId,
        json_str: &str,
    ) -> Result<Self, Error> {
        let spec: PipelineConfig =
            serde_json::from_str(json_str).map_err(|e| Error::DeserializationError {
                context: Context::new(namespace_id.clone(), pipeline_id.clone()),
                format: "JSON".to_string(),
                details: e.to_string(),
            })?;

        spec.validate(&namespace_id, &pipeline_id)?;
        Ok(spec)
    }

    /// Load a PipelineSpec from a JSON file.
    pub fn from_json_file<P: AsRef<Path>>(
        namespace_id: NamespaceId,
        pipeline_id: PipelineId,
        path: P,
    ) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(|e| Error::FileReadError {
            context: Context::new(namespace_id.clone(), pipeline_id.clone()),
            details: e.to_string(),
        })?;
        Self::from_json(namespace_id, pipeline_id, &contents)
    }

    /// Returns an iterator visiting all nodes in the pipeline.
    pub fn node_iter(&self) -> impl Iterator<Item = (&NodeId, &Rc<NodeUserConfig>)> {
        self.nodes.iter()
    }

    /// Creates a consuming iterator over the nodes in the pipeline.
    pub fn node_into_iter(self) -> impl Iterator<Item = (NodeId, Rc<NodeUserConfig>)> {
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
        namespace_id: &NamespaceId,
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
                        context: Context::new(namespace_id.clone(), pipeline_id.clone()),
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
                    context: Context::new(namespace_id.clone(), pipeline_id.clone()),
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
            nodes: &HashMap<NodeId, Rc<NodeUserConfig>>,
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
        namespace_id: T,
        pipeline_id: P,
    ) -> Result<PipelineConfig, Error>
    where
        T: Into<NamespaceId>,
        P: Into<PipelineId>,
    {
        let mut errors = Vec::new();
        let namespace_id = namespace_id.into();
        let pipeline_id = pipeline_id.into();

        // Report duplicated nodes
        for node_id in &self.duplicate_nodes {
            errors.push(Error::DuplicateNode {
                context: Context::new(namespace_id.clone(), pipeline_id.clone()),
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
                        context: Context::new(namespace_id.clone(), pipeline_id.clone()),
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
                    context: Context::new(namespace_id.clone(), pipeline_id.clone()),
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
                description: self.description,
                nodes: self
                    .nodes
                    .into_iter()
                    .map(|(id, node)| (id, Rc::new(node)))
                    .collect(),
                settings: PipelineSettings::default(),
                r#type: pipeline_type,
            };

            spec.validate(&namespace_id, &pipeline_id)?;
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
            .build(PipelineType::OTAP, "namespace", "pipeline");

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
            .build(PipelineType::OTAP, "namespace", "pipeline");

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
            .build(PipelineType::OTAP, "namespace", "pipeline");

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
            .build(PipelineType::OTAP, "namespace", "pipeline");

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
            .build(PipelineType::OTAP, "namespace", "pipeline");

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
            .build(PipelineType::OTAP, "namespace", "pipeline");

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
            .build(PipelineType::OTAP, "namespace", "pipeline");

        // Assert the DAG is valid and acyclic
        match dag {
            Ok(pipeline_spec) => {
                assert_eq!(pipeline_spec.nodes.len(), 18);
            }
            Err(e) => panic!("Failed to build pipeline DAG: {e:?}"),
        }
    }
}
