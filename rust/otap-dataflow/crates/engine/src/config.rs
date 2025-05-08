// SPDX-License-Identifier: Apache-2.0

//! Set of system configuration structures used by the engine, for example, to define channel sizes.
//!
// Note: This type of system configuration is distinct from the pipeline configuration, which
// focuses instead on defining the interconnection of nodes within the DAG and each node’s specific
// settings.

/// For now, the channel capacity is set to 256 (a power of two). This value is currently somewhat
/// arbitrary and will likely be adjusted (and made configurable) in the future once we have more
/// insight into the engine’s performance. The general idea is to choose a default that is big
/// enough to absorb short-lived rate mismatches yet small enough to expose back-pressure quickly
/// and stay L1/L2-cache-friendly.
///
/// The default capacity is generally guided by the following formula:
///
/// `capacity = producer_rate * worst_case_consumer_pause * safety_margin`
///
/// - producer_rate: the number of messages per second each producer can burst.
/// - worst_case_consumer_pause: the maximum duration (in seconds) a consumer might be unable to make progress
///   (e.g. syscall, I/O hiccup, thread pre-emption).
/// - safety_margin: a factor of 1.5–2* is typically sufficient to prevent occasional spikes from
///   overwhelming the system scheduler.
///
/// ToDo: Make this default value configurable and based on performance testing.
const DEFAULT_CONTROL_CHANNEL_CAPACITY: usize = 32;
const DEFAULT_PDATA_CHANNEL_CAPACITY: usize = 256;

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
                capacity: DEFAULT_CONTROL_CHANNEL_CAPACITY,
            },
            output_pdata_channel: PdataChannelConfig {
                capacity: DEFAULT_PDATA_CHANNEL_CAPACITY,
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
                capacity: DEFAULT_CONTROL_CHANNEL_CAPACITY,
            },
            input_pdata_channel: PdataChannelConfig {
                capacity: DEFAULT_PDATA_CHANNEL_CAPACITY,
            },
            output_pdata_channel: PdataChannelConfig {
                capacity: DEFAULT_PDATA_CHANNEL_CAPACITY,
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
                capacity: DEFAULT_CONTROL_CHANNEL_CAPACITY,
            },
            input_pdata_channel: PdataChannelConfig {
                capacity: DEFAULT_PDATA_CHANNEL_CAPACITY,
            },
        }
    }
}
