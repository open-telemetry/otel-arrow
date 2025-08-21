// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmark tests for config implementations

use criterion::{
    BatchSize, BenchmarkGroup, Criterion, criterion_group, criterion_main, measurement::WallTime,
};

use otap_df_config::experimental::{PipelineDag, SignalType};
use otap_df_config::node::NodeKind;

#[allow(missing_docs)]
pub mod bench_entry {
    use super::*;

    // optimize_chains benchmark function
    fn optimize_chains(c: &mut Criterion) {
        let mut group: BenchmarkGroup<'_, WallTime> = c.benchmark_group("config");

        let _ = group.bench_function("optimize_chains", |b| {
            b.iter_batched(
                || {
                    let dataflow_dag = create_pipeline_dag();
                    Box::new(dataflow_dag)
                },
                |mut dataflow_dag| {
                    dataflow_dag.optimize_chains();
                },
                BatchSize::SmallInput,
            );
        });
        group.finish();
    }

    // create_pipeline_dag creates a pipeline dag with some configuration for optimization benchmarks
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

    criterion_group!(
        name = benches;
        config = Criterion::default()
            .warm_up_time(std::time::Duration::from_secs(1))
            .measurement_time(std::time::Duration::from_secs(2));
        targets = optimize_chains
    );
}

criterion_main!(bench_entry::benches,);
