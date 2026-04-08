// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of system configuration structures used by the engine, for example, to define channel sizes.
//!
//! Note: This type of system configuration is distinct from the pipeline configuration, which
//! focuses instead on defining the interconnection of nodes within the DAG and each node’s specific
//! settings.

use indexmap::IndexMap;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::NodeId;

/// Default control channel capacity used by legacy constructor paths.
const DEFAULT_CONTROL_CHANNEL_CAPACITY: usize = 32;
/// Default pdata channel capacity used by legacy constructor paths.
const DEFAULT_PDATA_CHANNEL_CAPACITY: usize = 256;

/// Generic configuration for a control channel.
#[derive(Clone, Debug)]
pub struct ControlChannelConfig {
    /// Max capacity of the channel.
    pub capacity: usize,
}

/// Generic configuration for a pdata channel.
#[derive(Clone, Debug)]
pub struct PdataChannelConfig {
    /// Max capacity of the channel.
    pub capacity: usize,
}

/// Runtime configuration for a receiver.
#[derive(Clone, Debug)]
pub struct ReceiverConfig {
    /// Name of the receiver.
    pub name: NodeId,
    /// Configuration for control channel.
    pub control_channel: ControlChannelConfig,
    /// Configuration for output pdata channel.
    pub output_pdata_channel: PdataChannelConfig,
}

/// Generic configuration for a processor.
#[derive(Clone, Debug)]
pub struct ProcessorConfig {
    /// Name of the processor.
    pub name: NodeId,
    /// Configuration for control channel.
    pub control_channel: ControlChannelConfig,
    /// Configuration for input pdata channel.
    pub input_pdata_channel: PdataChannelConfig,
    /// Configuration for output pdata channel.
    pub output_pdata_channel: PdataChannelConfig,
}

/// Generic configuration for an exporter.
#[derive(Clone, Debug)]
pub struct ExporterConfig {
    /// Name of the exporter.
    pub name: NodeId,
    /// Configuration for control channel.
    pub control_channel: ControlChannelConfig,
    /// Configuration for input pdata channel.
    pub input_pdata_channel: PdataChannelConfig,
}

impl ReceiverConfig {
    /// Creates a new receiver configuration with default channel capacities.
    pub fn new<T>(name: T) -> Self
    where
        T: Into<NodeId>,
    {
        Self::with_channel_capacities(
            name,
            DEFAULT_CONTROL_CHANNEL_CAPACITY,
            DEFAULT_PDATA_CHANNEL_CAPACITY,
        )
    }

    /// Creates a new receiver configuration with explicit channel capacities.
    pub fn with_channel_capacities<T>(
        name: T,
        control_channel_capacity: usize,
        pdata_channel_capacity: usize,
    ) -> Self
    where
        T: Into<NodeId>,
    {
        ReceiverConfig {
            name: name.into(),
            control_channel: ControlChannelConfig {
                capacity: control_channel_capacity,
            },
            output_pdata_channel: PdataChannelConfig {
                capacity: pdata_channel_capacity,
            },
        }
    }
}

impl ProcessorConfig {
    /// Creates a new processor configuration with default channel capacities.
    #[must_use]
    pub fn new<T>(name: T) -> Self
    where
        T: Into<NodeId>,
    {
        Self::with_channel_capacities(
            name,
            DEFAULT_CONTROL_CHANNEL_CAPACITY,
            DEFAULT_PDATA_CHANNEL_CAPACITY,
        )
    }

    /// Creates a new processor configuration with explicit channel capacities.
    #[must_use]
    pub fn with_channel_capacities<T>(
        name: T,
        control_channel_capacity: usize,
        pdata_channel_capacity: usize,
    ) -> Self
    where
        T: Into<NodeId>,
    {
        ProcessorConfig {
            name: name.into(),
            control_channel: ControlChannelConfig {
                capacity: control_channel_capacity,
            },
            input_pdata_channel: PdataChannelConfig {
                capacity: pdata_channel_capacity,
            },
            output_pdata_channel: PdataChannelConfig {
                capacity: pdata_channel_capacity,
            },
        }
    }
}

impl ExporterConfig {
    /// Creates a new exporter configuration with default channel capacities.
    #[must_use]
    pub fn new<T>(name: T) -> Self
    where
        T: Into<NodeId>,
    {
        Self::with_channel_capacities(
            name,
            DEFAULT_CONTROL_CHANNEL_CAPACITY,
            DEFAULT_PDATA_CHANNEL_CAPACITY,
        )
    }

    /// Creates a new exporter configuration with explicit channel capacities.
    #[must_use]
    pub fn with_channel_capacities<T>(
        name: T,
        control_channel_capacity: usize,
        pdata_channel_capacity: usize,
    ) -> Self
    where
        T: Into<NodeId>,
    {
        ExporterConfig {
            name: name.into(),
            control_channel: ControlChannelConfig {
                capacity: control_channel_capacity,
            },
            input_pdata_channel: PdataChannelConfig {
                capacity: pdata_channel_capacity,
            },
        }
    }
}

/// User-facing configuration for a processor node chain
///
/// Deserialized from the `config` blob of a `processor_chain:*` node
///
/// ```yaml
/// my_chain:
///   type: processor_chain:composite
///   config:
///     processors:
///       insert_B:
///         type: processor:attribute
///         config: { ... }
///       condense:
///         type: processor:condense_attributes
///         config: { ... }
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ProcessorChainConfig {
    /// Sub-processors keyed by name, in declaration order.
    ///
    /// The map key is used as the `node.id` suffix in telemetry
    /// (e.g. `chain/insert_B`). Insertion order is preserved by
    /// [`IndexMap`] and determines execution order.
    pub processors: IndexMap<String, NodeUserConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processor_chain_config_from_json() {
        let json = serde_json::json!({
            "processors": {
                "attr": { "type": "processor:attribute", "config": { "actions": [] } },
                "dbg": { "type": "processor:debug", "config": { "verbosity": "basic" } }
            }
        });

        let cfg: ProcessorChainConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.processors.len(), 2);

        let keys: Vec<&String> = cfg.processors.keys().collect();
        assert_eq!(keys, vec!["attr", "dbg"]);

        assert_eq!(cfg.processors["attr"].r#type.as_str(), "urn:otel:processor:attribute");
        assert_eq!(cfg.processors["dbg"].r#type.as_str(), "urn:otel:processor:debug");
        assert!(cfg.processors["attr"].config.is_object());
    }

    #[test]
    fn processor_chain_config_default_config() {
        let json = serde_json::json!({
            "processors": {
                "dbg": { "type": "processor:debug" }
            }
        });

        let cfg: ProcessorChainConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.processors.len(), 1);
        assert!(cfg.processors["dbg"].config.is_null());
    }

    #[test]
    fn processor_chain_config_empty_map() {
        let json = serde_json::json!({ "processors": {} });
        let cfg: ProcessorChainConfig = serde_json::from_value(json).unwrap();
        assert!(cfg.processors.is_empty());
    }

    #[test]
    fn processor_chain_config_missing_processors_field() {
        let json = serde_json::json!({});
        let result = serde_json::from_value::<ProcessorChainConfig>(json);
        assert!(result.is_err());
    }
}
