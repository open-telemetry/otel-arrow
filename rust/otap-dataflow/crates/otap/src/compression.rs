// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//! Defines a compression enum to abstract from tonic and allows the exporter and receiver to get the respective tonic equivalent
//!

use serde::{Deserialize, Serialize};
use tonic::codec::CompressionEncoding;

/// Enum to represent various compression methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionMethod {
    /// Fastest compression
    Zstd,
    /// Most compatible compression method
    Gzip,
    /// Used for legacy systems
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_method_accepts_snake_case_only() {
        // Valid canonical snake_case values
        let zstd: CompressionMethod = serde_json::from_str("\"zstd\"").unwrap();
        assert!(matches!(zstd, CompressionMethod::Zstd));
        let gzip: CompressionMethod = serde_json::from_str("\"gzip\"").unwrap();
        assert!(matches!(gzip, CompressionMethod::Gzip));
        let deflate: CompressionMethod = serde_json::from_str("\"deflate\"").unwrap();
        assert!(matches!(deflate, CompressionMethod::Deflate));

        // Mixed/camel case should fail under strict snake_case
        assert!(serde_json::from_str::<CompressionMethod>("\"Gzip\"").is_err());
        assert!(serde_json::from_str::<CompressionMethod>("\"Zstd\"").is_err());
        assert!(serde_json::from_str::<CompressionMethod>("\"Deflate\"").is_err());
    }
}
