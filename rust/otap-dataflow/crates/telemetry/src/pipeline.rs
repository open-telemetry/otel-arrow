// SPDX-License-Identifier: Apache-2.0

//! Pipeline for internal telemetry.
//!
//! # Work-In-Progress
//! - [Phase 1 - Current] Aggregated metrics are displayed in the console.
//! - [Phase 2] Aggregated metrics will be sent via the Rust Client SDK.
//! - [Phase 3 - Exploratory] Aggregated metrics could be processed and delivered by our own
//!    pipeline engine. All processors and exporters could be used (OTLP, OTAP, ...).

use crate::attributes::NodeStaticAttrs;
use crate::error::Error;
use crate::metrics::MultivariateMetrics;
use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

/// A generic pipeline for the internal metrics system.
pub trait MetricsPipeline {
    /// Report the given multivariate metrics with the associated static attributes.
    fn report(&self, metrics: &dyn MultivariateMetrics, attrs: NodeStaticAttrs) -> Result<(), Error>;
}

/// A simple line protocol pipeline that prints the metrics to stdout.
/// This is a temporary solution for debugging and development purposes.
pub(crate) struct LineProtocolPipeline;

impl MetricsPipeline for LineProtocolPipeline {
    fn report(&self, metrics: &dyn MultivariateMetrics, attrs: NodeStaticAttrs) -> Result<(), Error> {
        let line = format_line_protocol(metrics, &attrs);
        println!("{}", line);
        Ok(())
    }
}

/// Escape tag keys/values according to InfluxDB line protocol (commas, spaces, equals).
fn escape_tag(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            ',' | ' ' | '=' => { out.push('\\'); out.push(ch); },
            _ => out.push(ch)
        }
    }
    out
}

/// Formats the provided metrics and attributes into a single line protocol string.
pub(crate) fn format_line_protocol(metrics: &dyn MultivariateMetrics, attrs: &NodeStaticAttrs) -> String {
    let desc = metrics.descriptor();
    // Measurement name.
    let mut line = String::with_capacity(192);
    line.push_str(desc.name);

    // Tags.
    // Static string tags first.
    let static_tags = [
        ("node_id", attrs.node_id.as_ref()),
        ("node_type", attrs.node_type.as_ref()),
        ("pipeline_id", attrs.pipeline_id.as_ref()),
    ];
    for (k,v) in static_tags.iter() {
        line.push(',');
        line.push_str(escape_tag(k).as_str());
        line.push('=');
        line.push_str(escape_tag(v).as_str());
    }
    // Numeric tags (avoid temporary reference issues by writing directly).
    let _ = write!(line, ",core_id={}", attrs.core_id);
    let _ = write!(line, ",numa_node_id={}", attrs.numa_node_id);
    let _ = write!(line, ",process_id={}", attrs.process_id);

    line.push(' ');

    // Fields (ordered as in descriptor, mapping to concrete type values).
    let mut first = true;
    for (field_desc, value) in metrics.field_values() {
        if !first { line.push(','); } else { first = false; }
        let _ = write!(line, "{}={}i", field_desc.name, value);
    }

    // Append timestamp (nanoseconds since Unix epoch) per line protocol syntax.
    let ts_ns: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0);
    let _ = write!(line, " {}", ts_ns);

    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{OtlpReceiverMetrics, PerfExporterPdataMetrics};
    use std::borrow::Cow;

    fn sample_attrs() -> NodeStaticAttrs { NodeStaticAttrs {
        node_id: Cow::Borrowed("nodeA"),
        node_type: Cow::Borrowed("receiver"),
        pipeline_id: Cow::Borrowed("p1"),
        core_id: 0,
        numa_node_id: 0,
        process_id: 1234,
    }}

    #[test]
    fn test_receiver_line_protocol() {
        let mut m = OtlpReceiverMetrics::default();
        m.bytes_received.add(42);
        m.messages_received.inc();
        let line = format_line_protocol(&m, &sample_attrs());
        assert!(line.starts_with("receiver_metrics,node_id=nodeA,node_type=receiver,pipeline_id=p1"));
        assert!(line.contains("bytes.received=42i"));
        assert!(line.contains("messages.received=1i"));
        assert!(line.rsplit_once(' ').unwrap().1.parse::<i64>().is_ok());
    }

    #[test]
    fn test_perf_exporter_line_protocol() {
        let mut m = PerfExporterPdataMetrics::default();
        m.logs.add(5);
        m.metrics.inc();
        let line = format_line_protocol(&m, &NodeStaticAttrs { node_id: Cow::Borrowed("x"), node_type: Cow::Borrowed("exporter"), pipeline_id: Cow::Borrowed("p2"), core_id: 1, numa_node_id: 0, process_id: 4321 });
        assert!(line.starts_with("perf_exporter_metrics,node_id=x,node_type=exporter,pipeline_id=p2"));
        assert!(line.contains("bytes.total=10i"));
        assert!(line.contains("logs=5i"));
        assert!(line.contains("metrics=1i"));
        assert!(line.rsplit_once(' ').unwrap().1.parse::<i64>().is_ok());
    }

    #[test]
    fn test_timestamp_present_and_order() {
        let m = OtlpReceiverMetrics::default();
        let line = format_line_protocol(&m, &sample_attrs());
        // Ensure exactly two space separators: one between tags & fields, one before timestamp.
        let space_count = line.chars().filter(|c| *c == ' ').count();
        assert_eq!(space_count, 2, "expected two spaces (tags-fields, fields-timestamp)");
        let (_before_ts, ts_str) = line.rsplit_once(' ').unwrap();
        assert!(ts_str.parse::<i64>().is_ok(), "timestamp not parseable: {ts_str}");
    }
}
