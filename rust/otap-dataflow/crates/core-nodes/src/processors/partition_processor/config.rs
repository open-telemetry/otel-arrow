// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroUsize;

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    /// configuration for how to compute the partition
    pub partition_by: PartitionByConfig,

    /// name of the transport header to which the partition value will be written
    pub partition_header_name: String,

    /// strategy to use when serializing partition results.
    #[serde(default)]
    pub header_serialization_strategy: PartitionValueSerializeStrategy,

    #[serde(default = "default_inbound_request_limit")]
    pub inbound_request_limit: NonZeroUsize,

    #[serde(default = "default_outbound_request_limit")]
    pub outbound_request_limit: NonZeroUsize,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PartitionByConfig {
    /// partition the OTAP batches by the result of evaluating this OPL expression
    OplExpression(String),
}

/// Configuration for strategy of how the partition values are converted to bytes so they can be
/// inserted into the pdata context headers.
///
/// OTAP Headers can only take on values of Binary and Text, whereas the expression used to
/// partition the batch may result in a variety of types including Ints, Doubles, Bools or Null,
/// so there needs to be a conversion. Different strategies may optimize for performance,
/// vs preserving tpe information
///
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PartitionValueSerializeStrategy {
    /// Simply convert the header value to Binary by taking the bytes.
    /// - Text/Binary will take the bytes of the value
    /// - Int/Double will use little endian byte representation
    /// - Boolean `true` will be encoded as [1], and `false` will be represented as [0].
    /// - Null will be represented as an empty vec
    ///
    /// This does not preserve type information. It also means that there will not be a
    /// distinction between the partition values that have the same byte serialization.
    /// For example, a boolean `false` may have the same serialization as binary [0x00].
    ///
    /// This is a good strategy to use when, for example, there is some a-priori knowledge that
    /// all the types produced by the expression are the same, or when the serialization collisions
    /// between different types don't matter to downstream consumers.
    ///
    /// The header `value_kind` will be set to `Binary` for all non-string partition values.
    /// When the value is a string value, the value_kind is controlled by the
    /// `text_as_binary_header` flag.
    ToBytesLossy {
        /// Whether to set the `value_kind`` as `Text` in cases where the partition value is
        /// a string value. When `false`, the value_kind will be set to `Binary` as it is for
        /// all other types.
        #[serde(default = "default_text_as_binary_header")]
        text_as_binary_header: bool,
    },

    /// Partition values serialized as JSON string.
    ///
    /// This will produce headers with value_kind `Text`
    Json,
}

impl Default for PartitionValueSerializeStrategy {
    fn default() -> Self {
        Self::ToBytesLossy {
            text_as_binary_header: default_text_as_binary_header(),
        }
    }
}

const fn default_text_as_binary_header() -> bool {
    false
}

const fn default_inbound_request_limit() -> NonZeroUsize {
    NonZeroUsize::new(1024).expect("ok")
}

const fn default_outbound_request_limit() -> NonZeroUsize {
    NonZeroUsize::new(2048).expect("ok")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize_defaults() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "partition_by": { "opl_expression": "name" },
            "partition_header_name": "part.name"
        }))
        .unwrap();

        assert_eq!(
            config,
            Config {
                partition_by: PartitionByConfig::OplExpression("name".to_string()),
                partition_header_name: "part.name".to_string(),
                header_serialization_strategy: PartitionValueSerializeStrategy::ToBytesLossy {
                    text_as_binary_header: false,
                },
                inbound_request_limit: NonZeroUsize::new(1024).unwrap(),
                outbound_request_limit: NonZeroUsize::new(2048).unwrap(),
            }
        );
    }

    #[test]
    fn test_choose_partition_serialization_strategy_json() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "partition_by": { "opl_expression": "name" },
            "partition_header_name": "part.name",
            "header_serialization_strategy": "json"
        }))
        .unwrap();

        assert_eq!(
            config.header_serialization_strategy,
            PartitionValueSerializeStrategy::Json
        )
    }

    #[test]
    fn test_choose_partition_serialization_strategy_to_bytes_lossy() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "partition_by": { "opl_expression": "name" },
            "partition_header_name": "part.name",
            "header_serialization_strategy":  {
                "to_bytes_lossy": {
                    "text_as_binary_header": true
                }
            }
        }))
        .unwrap();

        assert_eq!(
            config.header_serialization_strategy,
            PartitionValueSerializeStrategy::ToBytesLossy {
                text_as_binary_header: true
            }
        )
    }
}
