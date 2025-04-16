// SPDX-License-Identifier: Apache-2.0

//! OTAP Pipeline Configuration Model.
//!
//! A pipeline is a directed acyclic graph (DAG) of nodes, where each node represents a component
//! in the pipeline.

use crate::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

mod error;

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

/// Node kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    /// A source of signals
    Receiver,
    /// A processor of signals
    Processor,
    /// A sink of signals
    Exporter,
    /// A merged chain of consecutive processors
    ProcessorChain,
}

/// A node in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    id: String,
    kind: NodeKind,
    input_type: SignalType,
    output_type: SignalType,
    #[serde(default)]
    chain_members: Vec<String>,
    #[serde(default)]
    config: Value,
}

impl Node {
    /// Create a new node
    #[must_use]
    pub fn new(
        id: &str,
        kind: NodeKind,
        input_type: SignalType,
        output_type: SignalType,
        config: Value,
    ) -> Self {
        Self {
            id: id.to_string(),
            kind,
            input_type,
            output_type,
            chain_members: vec![],
            config,
        }
    }
}

/// An edge in the pipeline DAG
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    from: String,
    to: String,
}

/// The main pipeline DAG structure
#[derive(Debug, Default)]
pub struct PipelineDag {
    nodes: HashMap<String, Node>,
    edges: Vec<Edge>,
}

impl PipelineDag {
    /// Create an empty DAG
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the DAG
    pub fn add_node(
        &mut self,
        id: &str,
        kind: NodeKind,
        input_type: SignalType,
        output_type: SignalType,
        config: Value,
    ) -> Result<(), Error> {
        let node = Node::new(id, kind, input_type, output_type, config);
        let prev = self.nodes.insert(id.to_owned(), node);
        if prev.is_some() {
            return Err(Error::DuplicatedNodeId(id.to_owned()));
        }
        Ok(())
    }

    /// Add a directed edge
    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.edges.push(Edge {
            from: from.to_string(),
            to: to.to_string(),
        });
    }

    /// Validate the DAG: cycle check + type check
    pub fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = Vec::new();

        if let Some(cycle) = self.detect_cycle() {
            errors.push(Error::CycleDetected(cycle));
        }

        for edge in &self.edges {
            let from_node = match self.nodes.get(&edge.from) {
                Some(n) => n,
                None => {
                    errors.push(Error::UnknownNode(edge.from.clone()));
                    continue;
                }
            };
            let to_node = match self.nodes.get(&edge.to) {
                Some(n) => n,
                None => {
                    errors.push(Error::UnknownNode(edge.to.clone()));
                    continue;
                }
            };
            if from_node.output_type != to_node.input_type {
                errors.push(Error::TypeMismatch {
                    from_id: from_node.id.clone(),
                    to_id: to_node.id.clone(),
                    from_out: from_node.output_type,
                    to_in: to_node.input_type,
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Detects whether there's a cycle in the DAG.
    fn detect_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut in_stack = HashSet::new();

        fn dfs(
            node_id: &str,
            dag: &PipelineDag,
            visited: &mut HashSet<String>,
            stack: &mut Vec<String>,
            in_stack: &mut HashSet<String>,
        ) -> Option<Vec<String>> {
            _ = visited.insert(node_id.to_string());
            stack.push(node_id.to_string());
            _ = in_stack.insert(node_id.to_string());

            for edge in dag.edges.iter().filter(|e| e.from == node_id) {
                let next_id = &edge.to;
                if !visited.contains(next_id) {
                    if let Some(cycle) = dfs(next_id, dag, visited, stack, in_stack) {
                        return Some(cycle);
                    }
                } else if in_stack.contains(next_id) {
                    let cycle_start = stack.iter().position(|n| n == next_id).unwrap_or(0);
                    return Some(stack[cycle_start..].to_vec());
                }
            }

            _ = stack.pop();
            _ = in_stack.remove(node_id);
            None
        }

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                if let Some(path) = dfs(node_id, self, &mut visited, &mut stack, &mut in_stack) {
                    return Some(path);
                }
            }
        }
        None
    }

    /// Merge **consecutive Processor** nodes into a single ProcessorChain node.
    /// (We no longer consider a separate `Connector` kind — any node that changes signal types is just a Processor.)
    pub fn optimize_chains(&mut self) {
        let topo = match self.topological_sort() {
            Ok(o) => o,
            Err(_) => return, // skip if there's a cycle
        };

        let mut visited = HashSet::new();
        for node_id in topo {
            if visited.contains(&node_id) {
                continue;
            }
            let chain = self.gather_chain(&node_id, &mut visited);
            if chain.len() > 1 {
                self.merge_chain(&chain);
            }
        }
    }

    /// Gathers a chain of consecutive Processor nodes, following single-outbound edges,
    /// while each next node has only 1 inbound edge.
    fn gather_chain(&self, start: &str, visited: &mut HashSet<String>) -> Vec<String> {
        let mut chain = Vec::new();
        let mut current = start.to_string();

        while let Some(node) = self.nodes.get(&current) {
            // Only chain if it's a normal Processor
            if node.kind != NodeKind::Processor {
                break;
            }
            // Avoid re-chaining a node
            if visited.contains(&current) {
                break;
            }

            chain.push(current.clone());
            _ = visited.insert(current.clone());

            // Check how many outbound edges
            let successors: Vec<_> = self
                .edges
                .iter()
                .filter(|e| e.from == current)
                .map(|e| e.to.clone())
                .collect();

            if successors.len() != 1 {
                // can't chain further if 0 or multiple out edges
                break;
            }

            let next_id = &successors[0];
            let next_node = match self.nodes.get(next_id) {
                Some(n) => n,
                None => break,
            };

            // The next node must be a Processor with exactly 1 inbound edge
            if next_node.kind == NodeKind::Processor {
                let in_count = self.edges.iter().filter(|e| e.to == *next_id).count();
                if in_count == 1 && !visited.contains(next_id) {
                    // Continue chaining
                    current = next_id.clone();
                } else {
                    // either it has multiple inbound edges or already visited
                    break;
                }
            } else {
                // not a normal Processor
                break;
            }
        }

        chain
    }

    /// Merges the chain of Processor nodes into a single ProcessorChain node.
    fn merge_chain(&mut self, chain: &[String]) {
        let first_id = &chain[0];
        let last_id = &chain[chain.len() - 1];

        let first_node = self.nodes[first_id].clone();
        let last_node = self.nodes[last_id].clone();

        // The chain node's input/output match the first and last node
        let chain_input = first_node.input_type;
        let chain_output = last_node.output_type;

        let chain_id = format!("chain({})", chain.join("+"));
        let chain_config = Value::String(format!("Merged chain: {:?}", chain));

        let chain_node = Node {
            id: chain_id.clone(),
            kind: NodeKind::ProcessorChain,
            input_type: chain_input,
            output_type: chain_output,
            chain_members: chain.to_vec(),
            config: chain_config,
        };

        // Insert chain node
        _ = self.nodes.insert(chain_id.clone(), chain_node);

        // inbound to first
        let inbound: Vec<_> = self
            .edges
            .iter()
            .filter(|e| e.to == *first_id)
            .map(|e| e.from.clone())
            .collect();

        // outbound from last
        let outbound: Vec<_> = self
            .edges
            .iter()
            .filter(|e| e.from == *last_id)
            .map(|e| e.to.clone())
            .collect();

        // Remove edges from/to chain members
        self.edges
            .retain(|e| !chain.contains(&e.from) && !chain.contains(&e.to));

        // Rewire inbound->chain, chain->outbound
        for f in inbound {
            self.edges.push(Edge {
                from: f,
                to: chain_id.clone(),
            });
        }
        for t in outbound {
            self.edges.push(Edge {
                from: chain_id.clone(),
                to: t,
            });
        }

        // Remove old chain members
        for node_id in chain {
            _ = self.nodes.remove(node_id);
        }
    }

    /// Simple topological sort, fails if there's a cycle.
    fn topological_sort(&self) -> Result<Vec<String>, Error> {
        let mut in_degree = HashMap::new();
        for id in self.nodes.keys() {
            _ = in_degree.insert(id.clone(), 0);
        }
        for edge in &self.edges {
            if let Some(d) = in_degree.get_mut(&edge.to) {
                *d += 1;
            }
        }
        let mut queue = VecDeque::new();
        for (nid, deg) in &in_degree {
            if *deg == 0 {
                queue.push_back(nid.clone());
            }
        }

        let mut sorted = Vec::new();
        while let Some(n) = queue.pop_front() {
            sorted.push(n.clone());
            for e in self.edges.iter().filter(|e| e.from == n) {
                let d = in_degree
                    .get_mut(&e.to)
                    .ok_or(Error::UnknownNode(e.to.clone()))?;
                *d -= 1;
                if *d == 0 {
                    queue.push_back(e.to.clone());
                }
            }
        }

        if sorted.len() == self.nodes.len() {
            Ok(sorted)
        } else {
            Err(Error::CycleDetected(vec![
                "Cycle in topological sort".into(),
            ]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config() {
        let mut pipeline_dag = create_pipeline_dag();
        // Validate before optimization
        let res_before = pipeline_dag.validate();
        assert!(res_before.is_ok(), "Should be valid before optimization");

        // Optimize - merges consecutive Processor nodes only
        pipeline_dag.optimize_chains();

        // Validate after optimization
        let res_after = pipeline_dag.validate();
        assert!(res_after.is_ok(), "Should be valid after optimization");
    }

    fn create_pipeline_dag() -> PipelineDag {
        let mut dag = PipelineDag::new();

        // --------------------------------------------------
        // TRACES pipeline
        // --------------------------------------------------
        dag.add_node(
            "receiver_otlp_traces",
            NodeKind::Receiver,
            SignalType::Traces,
            SignalType::Traces,
            serde_json::from_str(r#"{"desc": "OTLP trace receiver"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "processor_batch_traces",
            NodeKind::Processor,
            SignalType::Traces,
            SignalType::Traces,
            serde_json::from_str(r#"{"name": "batch_traces"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "processor_resource_traces",
            NodeKind::Processor,
            SignalType::Traces,
            SignalType::Traces,
            serde_json::from_str(r#"{"name": "resource_traces"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        // This was previously a "connector", now it's just a Processor that changes input->output
        dag.add_node(
            "processor_traces_to_metrics",
            NodeKind::Processor,
            SignalType::Traces,
            SignalType::Metrics,
            serde_json::from_str(r#"{"desc": "convert traces to metrics"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "exporter_otlp_traces",
            NodeKind::Exporter,
            SignalType::Traces,
            SignalType::Traces,
            serde_json::from_str(r#"{"desc": "OTLP trace exporter"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        // Traces edges
        dag.add_edge("receiver_otlp_traces", "processor_batch_traces");
        dag.add_edge("processor_batch_traces", "processor_resource_traces");
        dag.add_edge("processor_resource_traces", "processor_traces_to_metrics");
        dag.add_edge("processor_resource_traces", "exporter_otlp_traces");

        // --------------------------------------------------
        // METRICS pipeline
        // --------------------------------------------------
        dag.add_node(
            "receiver_otlp_metrics",
            NodeKind::Receiver,
            SignalType::Metrics,
            SignalType::Metrics,
            serde_json::from_str(r#"{"desc": "OTLP metric receiver"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "processor_batch_metrics",
            NodeKind::Processor,
            SignalType::Metrics,
            SignalType::Metrics,
            serde_json::from_str(r#"{"name": "batch_metrics"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        // This was previously a "connector_metrics_to_events"
        dag.add_node(
            "processor_metrics_to_events",
            NodeKind::Processor,
            SignalType::Metrics,
            SignalType::Logs,
            serde_json::from_str(r#"{"desc": "convert metrics to events"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "exporter_prometheus",
            NodeKind::Exporter,
            SignalType::Metrics,
            SignalType::Metrics,
            serde_json::from_str(r#"{"desc": "Prometheus exporter"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "exporter_otlp_metrics",
            NodeKind::Exporter,
            SignalType::Metrics,
            SignalType::Metrics,
            serde_json::from_str(r#"{"desc": "OTLP metric exporter"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        // Edges
        dag.add_edge("receiver_otlp_metrics", "processor_batch_metrics");
        dag.add_edge("processor_batch_metrics", "processor_metrics_to_events");
        dag.add_edge("processor_batch_metrics", "exporter_prometheus");
        dag.add_edge("processor_batch_metrics", "exporter_otlp_metrics");
        // Also from traces->metrics
        dag.add_edge("processor_traces_to_metrics", "processor_batch_metrics");

        // --------------------------------------------------
        // LOGS pipeline
        // --------------------------------------------------
        dag.add_node(
            "receiver_filelog",
            NodeKind::Receiver,
            SignalType::Logs,
            SignalType::Logs,
            serde_json::from_str(r#"{"desc": "file log receiver"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "receiver_syslog",
            NodeKind::Receiver,
            SignalType::Logs,
            SignalType::Logs,
            serde_json::from_str(r#"{"desc": "syslog receiver"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "processor_filter_logs",
            NodeKind::Processor,
            SignalType::Logs,
            SignalType::Logs,
            serde_json::from_str(r#"{"name": "filter_logs"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        // formerly "connector_logs_to_events"
        dag.add_node(
            "processor_logs_to_events",
            NodeKind::Processor,
            SignalType::Logs,
            SignalType::Logs,
            serde_json::from_str(r#"{"desc": "convert logs to events"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "exporter_otlp_logs",
            NodeKind::Exporter,
            SignalType::Logs,
            SignalType::Logs,
            serde_json::from_str(r#"{"desc": "OTLP log exporter"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        // Edges
        dag.add_edge("receiver_filelog", "processor_filter_logs");
        dag.add_edge("receiver_syslog", "processor_filter_logs");
        dag.add_edge("processor_filter_logs", "processor_logs_to_events");
        dag.add_edge("processor_filter_logs", "exporter_otlp_logs");

        // --------------------------------------------------
        // EVENTS pipeline
        // --------------------------------------------------
        dag.add_node(
            "receiver_some_events",
            NodeKind::Receiver,
            SignalType::Logs,
            SignalType::Logs,
            serde_json::from_str(r#"{"desc": "custom event receiver"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "processor_enrich_events",
            NodeKind::Processor,
            SignalType::Logs,
            SignalType::Logs,
            serde_json::from_str(r#"{"name": "enrich_events"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        dag.add_node(
            "exporter_queue_events",
            NodeKind::Exporter,
            SignalType::Logs,
            SignalType::Logs,
            serde_json::from_str(r#"{"desc": "push events to queue"}"#).unwrap_or_default(),
        )
        .expect("Should be able to add node");
        // Edges
        dag.add_edge("receiver_some_events", "processor_enrich_events");
        dag.add_edge("processor_enrich_events", "exporter_queue_events");
        // logs->events and metrics->events feed here
        dag.add_edge("processor_logs_to_events", "processor_enrich_events");
        dag.add_edge("processor_metrics_to_events", "processor_enrich_events");
        dag
    }
}
