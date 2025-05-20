// SPDX-License-Identifier: Apache-2.0

//! Hyper-DAG configuration model.
//!
//! IMPORTANT NOTE: This is a work in progress.

use crate::error::Error;
use crate::{NodeKind, NodeName, PortName};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// The concept of Hyper-DAG is used to express specific semantics on hyper-edges that represent
/// communication channels between DAG nodes. A node connected to multiple outgoing nodes through a
/// hyper-edge can express different communication semantics. For example, it could be a broadcast
/// channel that sends the same message to all destination nodes, or it might have a round-robin or
/// least-loaded semantic, similar to an SPMC channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperDAG {
    nodes: HashMap<NodeName, Node>,
}

/// A node in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    name: NodeName,
    kind: NodeKind,
    out_ports: HashMap<PortName, HyperEdge>,

    /// The custom configuration for this node.
    ///
    /// This configuration is interpreted by the node itself and is not interpreted and validated by
    /// the pipeline engine.
    #[serde(default)]
    config: Value,
}

/// A directed hyper-edge in the pipeline establishing a connection between one source node and
/// one or more target nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    /// The target nodes of this hyper-edge.
    ///
    /// When there is only one target node, the hyper-edge is a simple edge and the dispatch
    /// strategy is ignored.
    targets: HashSet<NodeName>,

    /// The strategy used to dispatch data to the targets of this hyper-edge.
    dispatch_strategy: DispatchStrategy,
}

/// The strategy used to dispatch data to the targets of a hyper-edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DispatchStrategy {
    /// Broadcast the data to all targeted nodes.
    Broadcast,
    /// Round-robin dispatching to the targets.
    RoundRobin,
    /// Randomly select a target node to dispatch the data to.
    Random,
    /// Dispatch the data to the least loaded target node.
    LeastLoaded,
}

/// A builder for constructing a HyperDAG.
pub struct HyperDagBuilder {
    nodes: HashMap<NodeName, Node>,
    duplicate_nodes: Vec<NodeName>,
    pending_connections: Vec<PendingConnection>,
}

struct PendingConnection {
    src: NodeName,
    out_port: PortName,
    targets: HashSet<NodeName>,
    strategy: DispatchStrategy,
}

impl HyperDagBuilder {
    /// Create a new HyperDagBuilder.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            duplicate_nodes: Vec::new(),
            pending_connections: Vec::new(),
        }
    }

    /// Add a node with a given name and kind.
    /// Optionally provide config.
    pub fn add_node<S: Into<NodeName>>(
        mut self,
        name: S,
        kind: NodeKind,
        config: Option<Value>,
    ) -> Self {
        let name = name.into();
        if self.nodes.contains_key(&name) {
            self.duplicate_nodes.push(name.clone());
        } else {
            _ = self.nodes.insert(
                name.clone(),
                Node {
                    name,
                    kind,
                    out_ports: HashMap::new(),
                    config: config.unwrap_or(Value::Null),
                },
            );
        }
        self
    }

    /// Add a receiver node.
    pub fn add_receiver<S: Into<NodeName>>(self, name: S, config: Option<Value>) -> Self {
        self.add_node(name, NodeKind::Receiver, config)
    }

    /// Add a processor node.
    pub fn add_processor<S: Into<NodeName>>(self, name: S, config: Option<Value>) -> Self {
        self.add_node(name, NodeKind::Processor, config)
    }

    /// Add an exporter node.
    pub fn add_exporter<S: Into<NodeName>>(self, name: S, config: Option<Value>) -> Self {
        self.add_node(name, NodeKind::Exporter, config)
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
        S: Into<NodeName>,
        P: Into<PortName>,
        T: Into<NodeName>,
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
        S: Into<NodeName>,
        P: Into<PortName>,
        T: Into<NodeName>,
        I: IntoIterator<Item = T>,
    {
        self.connect(src, out_port, targets, DispatchStrategy::Broadcast)
    }

    /// Connect source node's out_port to one or more target nodes
    /// with a round-robin dispatch strategy.
    pub fn round_robin<S, P, T, I>(self, src: S, out_port: P, targets: I) -> Self
    where
        S: Into<NodeName>,
        P: Into<PortName>,
        T: Into<NodeName>,
        I: IntoIterator<Item = T>,
    {
        self.connect(src, out_port, targets, DispatchStrategy::RoundRobin)
    }

    /// Connect source node's out_port to one or more target nodes
    /// with a random dispatch strategy.
    pub fn random<S, P, T, I>(self, src: S, out_port: P, targets: I) -> Self
    where
        S: Into<NodeName>,
        P: Into<PortName>,
        T: Into<NodeName>,
        I: IntoIterator<Item = T>,
    {
        self.connect(src, out_port, targets, DispatchStrategy::Random)
    }

    /// Connect source node's out_port to one or more target nodes
    /// with a least-loaded dispatch strategy.
    pub fn least_loaded<S, P, T, I>(self, src: S, out_port: P, targets: I) -> Self
    where
        S: Into<NodeName>,
        P: Into<PortName>,
        T: Into<NodeName>,
        I: IntoIterator<Item = T>,
    {
        self.connect(src, out_port, targets, DispatchStrategy::LeastLoaded)
    }

    /// Validate and build the HyperDAG.
    ///
    /// We collect all possible errors (duplicate nodes, duplicate out-ports,
    /// missing source/targets, invalid edges, cycles) into one `InvalidHyperDag`
    /// report. This lets callers see every problem at once, rather than failing
    /// fast on the first error.
    pub fn build(mut self) -> Result<HyperDAG, Error> {
        let mut errors = Vec::new();

        // Report duplicated nodes
        for node_name in &self.duplicate_nodes {
            errors.push(Error::DuplicateNode {
                node_name: node_name.clone(),
            });
        }

        // Detect duplicate out‐ports (same src + port used twice)
        {
            let mut seen_ports = HashSet::new();
            for conn in &self.pending_connections {
                let key = (conn.src.clone(), conn.out_port.clone());
                if !seen_ports.insert(key.clone()) {
                    errors.push(Error::DuplicateOutPort {
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

            // if anything is missing, record as InvalidHyperEdge
            if !src_exists || !missing.is_empty() {
                errors.push(Error::InvalidHyperEdge {
                    source_node: conn.src.clone(),
                    target_nodes: conn.targets.iter().cloned().collect(),
                    dispatch_strategy: conn.strategy,
                    missing_source: !src_exists,
                    missing_targets: missing,
                });
                continue;
            }

            // finally insert into the node’s out_ports
            if let Some(node) = self.nodes.get_mut(&conn.src) {
                let _ = node.out_ports.insert(
                    conn.out_port.clone(),
                    HyperEdge {
                        targets: conn.targets.clone(),
                        dispatch_strategy: conn.strategy,
                    },
                );
            }
        }

        // If we haven’t already gathered errors, check for cycles
        if errors.is_empty() {
            let cycles = Self::detect_cycles(&self.nodes);
            for cycle in cycles {
                errors.push(Error::CycleDetected(cycle));
            }
        }

        if !errors.is_empty() {
            Err(Error::InvalidHyperDag { errors })
        } else {
            Ok(HyperDAG { nodes: self.nodes })
        }
    }

    fn detect_cycles(nodes: &HashMap<NodeName, Node>) -> Vec<Vec<NodeName>> {
        fn visit(
            node: &NodeName,
            nodes: &HashMap<NodeName, Node>,
            visiting: &mut HashSet<NodeName>,
            visited: &mut HashSet<NodeName>,
            current_path: &mut Vec<NodeName>,
            cycles: &mut Vec<Vec<NodeName>>,
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
                    for tgt in &edge.targets {
                        visit(tgt, nodes, visiting, visited, current_path, cycles);
                    }
                }
            }

            _ = visiting.remove(node);
            _ = visited.insert(node.clone());
            _ = current_path.pop();
        }

        let mut visiting = HashSet::new(); // Nodes in current DFS call stack
        let mut current_path = Vec::new(); // Nodes in current DFS path (for cycle path reconstruction)
        let mut visited = HashSet::new(); // Nodes already fully explored
        let mut cycles = Vec::new();

        for node in nodes.keys() {
            if !visited.contains(node) {
                visit(
                    node,
                    nodes,
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

// (Optionally, implement Default for convenience)
impl Default for HyperDagBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::node::{DispatchStrategy, HyperDagBuilder};
    use serde_json::json;

    #[test]
    fn test_duplicate_node_errors() {
        let result = HyperDagBuilder::new()
            .add_receiver("A", None)
            .add_processor("A", None) // duplicate
            .build();

        match result {
            Err(Error::InvalidHyperDag { errors }) => {
                // Should only report one DuplicateNode
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::DuplicateNode { node_name } if node_name == "A" => {}
                    other => panic!("expected DuplicateNode(\"A\"), got {:?}", other),
                }
            }
            other => panic!("expected Err(InvalidHyperDag), got {:?}", other),
        }
    }

    #[test]
    fn test_duplicate_outport_errors() {
        let result = HyperDagBuilder::new()
            .add_receiver("A", None)
            .add_exporter("B", None)
            .round_robin("A", "p", ["B"])
            .round_robin("A", "p", ["B"]) // duplicate port on A
            .build();

        match result {
            Err(Error::InvalidHyperDag { errors }) => {
                // One DuplicateOutPort, no InvalidHyperEdge, no cycles
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::DuplicateOutPort { source_node, port }
                        if source_node == "A" && port == "p" => {}
                    other => panic!("expected DuplicateOutPort(A, p), got {:?}", other),
                }
            }
            other => panic!("expected Err(InvalidHyperDag), got {:?}", other),
        }
    }

    #[test]
    fn test_missing_source_error() {
        let result = HyperDagBuilder::new()
            .add_receiver("B", None)
            .connect("X", "out", ["B"], DispatchStrategy::Broadcast) // X does not exist
            .build();

        match result {
            Err(Error::InvalidHyperDag { errors }) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::InvalidHyperEdge {
                        source_node,
                        missing_source,
                        missing_targets,
                        ..
                    } if source_node == "X" && *missing_source && missing_targets.is_empty() => {}
                    other => panic!("expected InvalidHyperEdge missing_source, got {:?}", other),
                }
            }
            other => panic!("expected Err(InvalidHyperDag), got {:?}", other),
        }
    }

    #[test]
    fn test_missing_target_error() {
        let result = HyperDagBuilder::new()
            .add_receiver("A", None)
            .connect("A", "out", ["Y"], DispatchStrategy::Broadcast) // Y does not exist
            .build();

        match result {
            Err(Error::InvalidHyperDag { errors }) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    Error::InvalidHyperEdge {
                        source_node,
                        missing_source,
                        missing_targets,
                        target_nodes,
                        ..
                    } if source_node == "A"
                        && !*missing_source
                        && missing_targets.as_slice() == &["Y"]
                        && target_nodes.as_slice() == &["Y"] => {}
                    other => panic!("expected InvalidHyperEdge missing_targets, got {:?}", other),
                }
            }
            other => panic!("expected Err(InvalidHyperDag), got {:?}", other),
        }
    }

    #[test]
    fn test_cycle_detection_error() {
        let result = HyperDagBuilder::new()
            .add_processor("A", None)
            .add_processor("B", None)
            .round_robin("A", "p", ["B"])
            .round_robin("B", "p", ["A"])
            .build();

        match result {
            Err(Error::InvalidHyperDag { errors }) => {
                // exactly one cycle error
                let mut found = false;
                for err in errors {
                    if let Error::CycleDetected(cycle) = err {
                        // cycle should include A and B
                        assert!(cycle.contains(&"A".into()));
                        assert!(cycle.contains(&"B".into()));
                        found = true;
                    }
                }
                assert!(found, "expected a CycleDetected error");
            }
            other => panic!("expected Err(InvalidHyperDag), got {:?}", other),
        }
    }

    #[test]
    fn test_successful_simple_build() {
        let dag = HyperDagBuilder::new()
            .add_receiver("Start", Some(json!({"foo": 1})))
            .add_exporter("End", None)
            .broadcast("Start", "out", ["End"])
            .build();

        match dag {
            Ok(hyperdag) => {
                // two nodes, one edge on Start
                assert_eq!(hyperdag.nodes.len(), 2);
                let start = &hyperdag.nodes["Start"];
                assert_eq!(start.out_ports.len(), 1);
                let edge = &start.out_ports["out"];
                assert!(edge.targets.contains("End"));
            }
            Err(e) => panic!("expected successful build, got {:?}", e),
        }
    }

    #[test]
    fn test_valid_complex_pipeline_hyperdag() {
        let dag = HyperDagBuilder::new()
            // ----- TRACES pipeline -----
            .add_receiver(
                "receiver_otlp_traces",
                Some(json!({"desc": "OTLP trace receiver"})),
            )
            .add_processor(
                "processor_batch_traces",
                Some(json!({"name": "batch_traces"})),
            )
            .add_processor(
                "processor_resource_traces",
                Some(json!({"name": "resource_traces"})),
            )
            .add_processor(
                "processor_traces_to_metrics",
                Some(json!({"desc": "convert traces to metrics"})),
            )
            .add_exporter(
                "exporter_otlp_traces",
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
                Some(json!({"desc": "OTLP metric receiver"})),
            )
            .add_processor(
                "processor_batch_metrics",
                Some(json!({"name": "batch_metrics"})),
            )
            .add_processor(
                "processor_metrics_to_events",
                Some(json!({"desc": "convert metrics to events"})),
            )
            .add_exporter(
                "exporter_prometheus",
                Some(json!({"desc": "Prometheus exporter"})),
            )
            .add_exporter(
                "exporter_otlp_metrics",
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
                Some(json!({"desc": "file log receiver"})),
            )
            .add_receiver("receiver_syslog", Some(json!({"desc": "syslog receiver"})))
            .add_processor(
                "processor_filter_logs",
                Some(json!({"name": "filter_logs"})),
            )
            .add_processor(
                "processor_logs_to_events",
                Some(json!({"desc": "convert logs to events"})),
            )
            .add_exporter(
                "exporter_otlp_logs",
                Some(json!({"desc": "OTLP log exporter"})),
            )
            .round_robin("receiver_filelog", "out", ["processor_filter_logs"])
            .round_robin("receiver_syslog", "out", ["processor_filter_logs"])
            .round_robin("processor_filter_logs", "out", ["processor_logs_to_events"])
            .round_robin("processor_filter_logs", "out2", ["exporter_otlp_logs"])
            // ----- EVENTS pipeline -----
            .add_receiver(
                "receiver_some_events",
                Some(json!({"desc": "custom event receiver"})),
            )
            .add_processor(
                "processor_enrich_events",
                Some(json!({"name": "enrich_events"})),
            )
            .add_exporter(
                "exporter_queue_events",
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
            .build();

        // Assert the DAG is valid and acyclic
        match dag {
            Ok(hyperdag) => {
                assert_eq!(hyperdag.nodes.len(), 18);
            }
            Err(e) => panic!("Failed to build pipeline DAG: {e:?}"),
        }
    }
}