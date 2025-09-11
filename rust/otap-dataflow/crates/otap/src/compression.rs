// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//! Defines a compression enum to abstract from tonic and allows the exporter and receiver to get the respective tonic equivalent
//!

use serde::{Deserialize, Serialize};
use tonic::codec::CompressionEncoding;

/// Enum to represent various compression methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionMethod {
    /// Fastest compression
    #[serde(alias = "zstd")]
    Zstd,
    /// Most compatible compression method
    #[serde(alias = "gzip")]
    Gzip,
    /// Used for legacy systems
    #[serde(alias = "deflate")]
    Deflate,
}

impl CompressionMethod {
    /// map the compression method to the proper tonic compression encoding equivalent
    /// use the CompressionMethod enum to abstract from tonic
    #[must_use]
    pub const fn map_to_compression_encoding(&self) -> CompressionEncoding {
        match *self {
            CompressionMethod::Gzip => CompressionEncoding::Gzip,
            CompressionMethod::Zstd => CompressionEncoding::Zstd,
            CompressionMethod::Deflate => CompressionEncoding::Deflate,
        }
    }
}
