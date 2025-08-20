// SPDX-License-Identifier: Apache-2.0

//! Pipeline for internal telemetry.
//!
//! # Work-In-Progress
//! - [Phase 1 - Current] Aggregated metrics are displayed in the console.
//! - [Phase 2] Aggregated metrics will be sent via the Rust Client SDK.
//! - [Phase 3 - Exploratory] Aggregated metrics could be processed and delivered by our own
//!    pipeline engine. All processors and exporters could be used (OTLP, OTAP, ...).

use crate::attributes::NodeStaticAttrs;
use crate::descriptor::MetricsField;
use crate::error::Error;
use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

/// A generic pipeline for the internal metrics system.
pub trait MetricsPipeline {
    /// Report the given iterator of (field, value) pairs for a measurement with the associated static attributes.
    fn report_iter<'a>(
        &self,
        measurement: &'static str,
        fields: Box<dyn Iterator<Item = (&'a MetricsField, u64)> + 'a>,
        attrs: &NodeStaticAttrs,
    ) -> Result<(), Error>;
}

/// A simple line protocol pipeline that prints the metrics to stdout.
/// This is a temporary solution for debugging and development purposes.
pub(crate) struct LineProtocolPipeline;

impl MetricsPipeline for LineProtocolPipeline {
    fn report_iter<'a>(
        &self,
        measurement: &'static str,
        fields: Box<dyn Iterator<Item = (&'a MetricsField, u64)> + 'a>,
        attrs: &NodeStaticAttrs,
    ) -> Result<(), Error> {
        let line = format_line_protocol_iter(measurement, fields, attrs);
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

/// Formats the provided metrics fields and attributes into a single line protocol string.
pub(crate) fn format_line_protocol_iter<'a>(
    measurement: &'static str,
    fields: Box<dyn Iterator<Item = (&'a MetricsField, u64)> + 'a>,
    attrs: &NodeStaticAttrs,
) -> String {
    let mut line = String::with_capacity(192);
    line.push_str(measurement);

    // Tags.
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
    let _ = write!(line, ",core_id={}", attrs.core_id);
    let _ = write!(line, ",numa_node_id={}", attrs.numa_node_id);
    let _ = write!(line, ",process_id={}", attrs.process_id);

    line.push(' ');

    let mut first = true;
    for (field_desc, value) in fields {
        if !first { line.push(','); } else { first = false; }
        let _ = write!(line, "{}={}i", field_desc.name, value);
    }

    let ts_ns: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0);
    let _ = write!(line, " {}", ts_ns);

    line
}