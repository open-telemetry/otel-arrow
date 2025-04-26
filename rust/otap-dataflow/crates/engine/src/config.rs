// SPDX-License-Identifier: Apache-2.0

//! Generic configuration for the pipeline engine.
//!
//! Note: Custom configuration types should be defined in the respective node implementations.

const DEFAULT_CHANNEL_CAPACITY: usize = 100;

/// Generic configuration for a control channel.
pub struct ControlChannelConfig {
    /// Max capacity of the channel.
    pub capacity: usize,
}

/// Generic configuration for a pdata channel.
pub struct PdataChannelConfig {
    /// Max capacity of the channel.
    pub capacity: usize,
}

/// Generic configuration for a receiver.
pub struct ReceiverConfig {
    /// Name of the receiver.
    pub name: String,
    /// Configuration for control channel.
    pub control_channel: ControlChannelConfig,
    /// Configuration for output pdata channel.
    pub output_pdata_channel: PdataChannelConfig,
}

/// Generic configuration for a processor.
pub struct ProcessorConfig {
    /// Name of the processor.
    pub name: String,
    /// Configuration for control channel.
    pub control_channel: ControlChannelConfig,
    /// Configuration for input pdata channel.
    pub input_pdata_channel: PdataChannelConfig,
    /// Configuration for output pdata channel.
    pub output_pdata_channel: PdataChannelConfig,
}

/// Generic configuration for an exporter.
pub struct ExporterConfig {
    /// Name of the exporter.
    pub name: String,
    /// Configuration for control channel.
    pub control_channel: ControlChannelConfig,
    /// Configuration for input pdata channel.
    pub input_pdata_channel: PdataChannelConfig,
}

impl ReceiverConfig {
    /// Creates a new receiver configuration with the given name and default channel capacity.
    pub fn new(name: &str) -> Self {
        ReceiverConfig {
            name: name.to_owned(),
            control_channel: ControlChannelConfig {
                capacity: DEFAULT_CHANNEL_CAPACITY,
            },
            output_pdata_channel: PdataChannelConfig {
                capacity: DEFAULT_CHANNEL_CAPACITY,
            },
        }
    }
}

impl ProcessorConfig {
    /// Creates a new processor configuration with the given name and default channel capacity.
    pub fn new(name: &str) -> Self {
        ProcessorConfig {
            name: name.to_owned(),
            control_channel: ControlChannelConfig {
                capacity: DEFAULT_CHANNEL_CAPACITY,
            },
            input_pdata_channel: PdataChannelConfig {
                capacity: DEFAULT_CHANNEL_CAPACITY,
            },
            output_pdata_channel: PdataChannelConfig {
                capacity: DEFAULT_CHANNEL_CAPACITY,
            },
        }
    }
}

impl ExporterConfig {
    /// Creates a new exporter configuration with the given name and default channel capacity.
    pub fn new(name: &str) -> Self {
        ExporterConfig {
            name: name.to_owned(),
            control_channel: ControlChannelConfig {
                capacity: DEFAULT_CHANNEL_CAPACITY,
            },
            input_pdata_channel: PdataChannelConfig {
                capacity: DEFAULT_CHANNEL_CAPACITY,
            },
        }
    }
}
