// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Defines a compression enum to abstract from tonic and allows the exporter and receiver to get
//! the respective tonic equivalent.

use std::io::{self, Write};

use flate2::Compression;
use flate2::write::{DeflateEncoder, GzEncoder};
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error as _;
use tonic::codec::CompressionEncoding;

/// Default zstd compression level. Matches tonic's default (3).
const ZSTD_DEFAULT_LEVEL: i32 = 3;

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

    /// Returns the canonical HTTP `Content-Encoding` / `Accept-Encoding` token
    /// for this compression method. Values are IANA-registered and match the
    /// tokens tonic emits for the gRPC `grpc-encoding` header.
    #[must_use]
    pub const fn as_http_content_encoding(&self) -> &'static str {
        match *self {
            CompressionMethod::Gzip => "gzip",
            CompressionMethod::Zstd => "zstd",
            CompressionMethod::Deflate => "deflate",
        }
    }

    /// Compresses `input` into `out`, reusing the caller's buffer.
    ///
    /// The output buffer is cleared before encoding so callers can keep a
    /// long-lived scratch `Vec` across requests without per-request
    /// allocations.
    pub fn encode(&self, input: &[u8], out: &mut Vec<u8>) -> io::Result<()> {
        // Precondition: scratch buffer is owned by the caller; we reset it
        // before writing so prior contents don't leak into the output.
        out.clear();
        match *self {
            CompressionMethod::Gzip => {
                let mut encoder = GzEncoder::new(out, Compression::default());
                encoder.write_all(input)?;
                _ = encoder.finish()?;
                Ok(())
            }
            CompressionMethod::Deflate => {
                let mut encoder = DeflateEncoder::new(out, Compression::default());
                encoder.write_all(input)?;
                _ = encoder.finish()?;
                Ok(())
            }
            CompressionMethod::Zstd => {
                zstd::stream::copy_encode(input, out, ZSTD_DEFAULT_LEVEL)?;
                Ok(())
            }
        }
    }
}

/// Deserializer that accepts a single compression method, the `"none"`
/// keyword, or an absent field. Sharing [`CompressionConfigValue`] with the
/// list-form deserializer keeps the `"none"` semantics consistent across
/// configs. A list value is rejected with a clear error rather than silently
/// truncated.
pub fn deserialize_compression_method<'de, D>(
    deserializer: D,
) -> Result<Option<CompressionMethod>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<CompressionConfigValue>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        CompressionConfigValue::Single(method) => Ok(Some(method)),
        CompressionConfigValue::NoneKeyword(CompressionNone::None) => Ok(None),
        CompressionConfigValue::List(_) => Err(D::Error::custom(
            "expected a single compression method or \"none\", got a list",
        )),
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
    fn as_http_content_encoding_returns_iana_tokens() {
        assert_eq!(CompressionMethod::Gzip.as_http_content_encoding(), "gzip");
        assert_eq!(CompressionMethod::Zstd.as_http_content_encoding(), "zstd");
        assert_eq!(
            CompressionMethod::Deflate.as_http_content_encoding(),
            "deflate"
        );
    }

    #[test]
    fn encode_round_trips_each_method() {
        use std::io::Read;

        let payload = b"hello otap compression - this is a reasonably sized payload to compress";
        let mut buf = Vec::new();

        // gzip
        CompressionMethod::Gzip.encode(payload, &mut buf).unwrap();
        let mut decoded = Vec::new();
        _ = flate2::read::GzDecoder::new(buf.as_slice())
            .read_to_end(&mut decoded)
            .unwrap();
        assert_eq!(decoded.as_slice(), payload);

        // deflate
        CompressionMethod::Deflate.encode(payload, &mut buf).unwrap();
        decoded.clear();
        _ = flate2::read::DeflateDecoder::new(buf.as_slice())
            .read_to_end(&mut decoded)
            .unwrap();
        assert_eq!(decoded.as_slice(), payload);

        // zstd
        CompressionMethod::Zstd.encode(payload, &mut buf).unwrap();
        decoded.clear();
        zstd::stream::copy_decode(buf.as_slice(), &mut decoded).unwrap();
        assert_eq!(decoded.as_slice(), payload);
    }

    #[test]
    fn encode_clears_existing_buffer_contents() {
        let mut buf = vec![0xAA, 0xBB, 0xCC, 0xDD];
        CompressionMethod::Gzip.encode(b"abc", &mut buf).unwrap();
        // gzip header magic bytes
        assert_eq!(&buf[..2], &[0x1f, 0x8b]);
    }

    #[derive(Debug, Deserialize)]
    struct ConfWithSingleCompression {
        #[serde(default, deserialize_with = "deserialize_compression_method")]
        compression: Option<CompressionMethod>,
    }

    #[test]
    fn deserialize_single_supports_single_value() {
        let conf: ConfWithSingleCompression =
            serde_json::from_str(r#"{ "compression": "gzip" }"#).unwrap();
        assert_eq!(conf.compression, Some(CompressionMethod::Gzip));
    }

    #[test]
    fn deserialize_single_supports_none_keyword() {
        let conf: ConfWithSingleCompression =
            serde_json::from_str(r#"{ "compression": "none" }"#).unwrap();
        assert_eq!(conf.compression, None);
    }

    #[test]
    fn deserialize_single_supports_absence() {
        let conf: ConfWithSingleCompression = serde_json::from_str("{}").unwrap();
        assert_eq!(conf.compression, None);
    }

    #[test]
    fn deserialize_single_rejects_list() {
        let err = serde_json::from_str::<ConfWithSingleCompression>(
            r#"{ "compression": ["gzip", "zstd"] }"#,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("single compression method"),
            "unexpected error: {err}"
        );
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
