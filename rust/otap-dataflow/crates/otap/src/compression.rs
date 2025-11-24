// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Defines a compression enum to abstract from tonic and allows the exporter and receiver to get
//! the respective tonic equivalent.

use serde::{Deserialize, Deserializer, Serialize};
use tonic::codec::CompressionEncoding;

/// Enum to represent various compression methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
            CompressionMethod::Zstd => CompressionEncoding::Zstd,
            CompressionMethod::Gzip => CompressionEncoding::Gzip,
            CompressionMethod::Deflate => CompressionEncoding::Deflate,
        }
    }
}

/// Default set of compression methods that are accepted when no configuration is provided.
pub const DEFAULT_COMPRESSION_METHODS: [CompressionMethod; 3] = [
    CompressionMethod::Zstd,
    CompressionMethod::Gzip,
    CompressionMethod::Deflate,
];

#[derive(Deserialize)]
#[serde(untagged)]
enum CompressionConfigValue {
    Single(CompressionMethod),
    List(Vec<CompressionMethod>),
    NoneKeyword(CompressionNone),
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum CompressionNone {
    None,
}

/// Deserializer that accepts either a single compression method, a list, or the string `"none"`.
/// Absence of the field keeps the default behaviour (all methods).
pub fn deserialize_compression_methods<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<CompressionMethod>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<CompressionConfigValue>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };

    let methods = match value {
        CompressionConfigValue::Single(method) => vec![method],
        CompressionConfigValue::List(methods) => methods,
        CompressionConfigValue::NoneKeyword(CompressionNone::None) => Vec::new(),
    };

    let mut deduped = Vec::with_capacity(methods.len());
    for method in methods {
        if !deduped.contains(&method) {
            deduped.push(method);
        }
    }

    Ok(Some(deduped))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct ConfWithCompression {
        #[serde(default, deserialize_with = "deserialize_compression_methods")]
        methods: Option<Vec<CompressionMethod>>,
    }

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

    #[test]
    fn deserialize_supports_single_value() {
        let conf: ConfWithCompression = serde_json::from_str(r#"{ "methods": "gzip" }"#).unwrap();
        assert_eq!(conf.methods, Some(vec![CompressionMethod::Gzip]));
    }

    #[test]
    fn deserialize_supports_list() {
        let conf: ConfWithCompression =
            serde_json::from_str(r#"{ "methods": ["gzip", "zstd", "gzip"] }"#).unwrap();
        assert_eq!(
            conf.methods,
            Some(vec![CompressionMethod::Gzip, CompressionMethod::Zstd])
        );
    }

    #[test]
    fn deserialize_supports_none_keyword() {
        let conf: ConfWithCompression = serde_json::from_str(r#"{ "methods": "none" }"#).unwrap();
        assert_eq!(conf.methods, Some(vec![]));
    }

    #[test]
    fn deserialize_supports_absence() {
        #[derive(Debug, Deserialize)]
        struct Conf {
            #[serde(default, deserialize_with = "deserialize_compression_methods")]
            methods: Option<Vec<CompressionMethod>>,
        }
        let conf: Conf = serde_json::from_str("{}").unwrap();
        assert_eq!(conf.methods, None);
    }
}
