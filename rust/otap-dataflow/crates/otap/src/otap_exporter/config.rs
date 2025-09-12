// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the OTAP Exporter

use crate::compression::CompressionMethod;
use serde::{Deserialize, Deserializer};

/// Configuration for the OTAP Exporter
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The grpc endpoint to which OTAP service requests will be sent
    pub grpc_endpoint: String,

    /// The type of compression to use for the gRPC messages. default = zstd.
    /// The value "none" can be used to disable compression,
    #[serde(
        default = "default_compression_method",
        deserialize_with = "deserialize_compression_method"
    )]
    pub compression_method: Option<CompressionMethod>,

    /// Configuration for the arrow payloads
    #[serde(default)]
    pub arrow: ArrowConfig,
}

/// Configuration for the arrow payloads produced by the [`OtapExporter`]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrowConfig {
    /// Compression to use for IPC serialized payloads within the BatchArrowMessages
    ///
    /// This defaults to "zstd", wherein arrow IPC stream will be compressed with IPC
    /// independently of whatever compression gRPC may have been configured. This is on by
    /// default, achieving "double compression" because:
    /// (a) relatively cheap in CPU terms
    /// (b) minor compression benefit
    /// (c) helps stay under gRPC request size limits
    ///
    /// The value "none" can be used to disable compression.
    #[serde(
        default = "default_arrow_payload_compression",
        deserialize_with = "deserialize_payload_compression"
    )]
    pub payload_compression: Option<ArrowPayloadCompression>,
}

/// Compression options for arrow payloads
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArrowPayloadCompression {
    /// Zstd compression
    Zstd,
}

impl Default for ArrowConfig {
    fn default() -> Self {
        Self {
            payload_compression: default_arrow_payload_compression(),
        }
    }
}

fn default_compression_method() -> Option<CompressionMethod> {
    Some(CompressionMethod::Zstd)
}

fn default_arrow_payload_compression() -> Option<ArrowPayloadCompression> {
    Some(ArrowPayloadCompression::Zstd)
}

/// helper method to deserialize the text "none" as the None option. This is needed to override
/// the default compression method, which is zstd, and it keeps the config value consistent with
/// the go collector.
fn deserialize_option_with_none<'de, D, T, F>(
    deserializer: D,
    default: F,
) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
    F: Fn() -> Option<T>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        None => Ok(default()),    // field missing -> apply default
        Some("none") => Ok(None), // explicit "none" -> None
        Some(v) => {
            // Re-parse from string into T
            T::deserialize(serde::de::IntoDeserializer::into_deserializer(v)).map(Some)
        }
    }
}

fn deserialize_payload_compression<'de, D>(
    deserializer: D,
) -> Result<Option<ArrowPayloadCompression>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_option_with_none(deserializer, default_arrow_payload_compression)
}

fn deserialize_compression_method<'de, D>(
    deserializer: D,
) -> Result<Option<CompressionMethod>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_option_with_none(deserializer, default_compression_method)
}
