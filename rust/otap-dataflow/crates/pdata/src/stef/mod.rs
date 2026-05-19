// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal STEF metrics codec compatible with the Splunk/Collector STEF metrics schema.
//!
//! The codec is split around the three responsibilities that matter for maintenance:
//! public API and error types, high-level OTLP/OTAP encode/decode orchestration, and the
//! low-level STEF wire primitives. The current implementation supports the metrics profile
//! used by the Go Collector `stefreceiver` and `stefexporter`: gauge and sum number data
//! points plus simple OTLP attribute values.

mod decode;
mod encode;
mod error;
#[cfg(test)]
mod tests;
mod wire;

pub use decode::{MetricsStreamDecoder, decode_metrics_otap, decode_metrics_otap_with_count};
pub use encode::{
    EncodedFrame, MetricsStreamEncoder, encode_metrics_otap, encode_metrics_otap_with_count,
    encode_metrics_otap_with_count_and_compression, encode_metrics_view,
    encode_metrics_view_with_count, encode_metrics_view_with_count_and_compression,
};
pub use error::Error;
use serde::Deserialize;

/// Native STEF frame payload compression.
///
/// This is encoded in the STEF fixed header and applies to each frame's content.
/// It is distinct from gRPC message compression.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StefCompression {
    /// Store STEF frame content without native payload compression.
    #[default]
    None,
    /// Compress STEF frame content with zstd, matching the Go STEF implementation.
    Zstd,
}

/// Root struct name used by the STEF metrics schema.
pub const METRICS_ROOT_STRUCT_NAME: &str = "Metrics";

/// Wire schema serialized by the Go-generated `otelstef.MetricsWireSchema()`.
pub const METRICS_WIRE_SCHEMA: &[u8] = &[
    0x0F, 0x06, 0x01, 0x08, 0x07, 0x03, 0x05, 0x04, 0x05, 0x05, 0x09, 0x02, 0x03, 0x02, 0x05, 0x02,
];
