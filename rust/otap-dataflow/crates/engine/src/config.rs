// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of system configuration structures used by the engine, for example, to define channel sizes.
//!
//! Note: This type of system configuration is distinct from the pipeline configuration, which
//! focuses instead on defining the interconnection of nodes within the DAG and each node’s specific
//! settings.

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

/// Runtime configuration for an extension.
///
/// Extensions only have a control channel — they do not process pipeline data.
#[derive(Clone, Debug)]
pub struct ExtensionConfig {
    /// Name of the extension.
    pub name: NodeId,
    /// Configuration for control channel.
    pub control_channel: ControlChannelConfig,
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

impl ExtensionConfig {
    /// Creates a new extension configuration with default channel capacities.
    #[must_use]
    pub fn new<T>(name: T) -> Self
    where
        T: Into<NodeId>,
    {
        Self::with_control_channel_capacity(name, DEFAULT_CONTROL_CHANNEL_CAPACITY)
    }

    /// Creates a new extension configuration with explicit control channel capacity.
    #[must_use]
    pub fn with_control_channel_capacity<T>(name: T, control_channel_capacity: usize) -> Self
    where
        T: Into<NodeId>,
    {
        ExtensionConfig {
            name: name.into(),
            control_channel: ControlChannelConfig {
                capacity: control_channel_capacity,
            },
        }
    }
}
