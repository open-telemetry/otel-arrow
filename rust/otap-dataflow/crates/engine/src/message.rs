// SPDX-License-Identifier: Apache-2.0

//! Message definitions for the dataflow engine.

/// A message that can be sent to a node (i.e. receiver, processor, exporter, or connector).
///
/// A message is either a `Data` message, which contains a payload of type `Data`, or a `Control`
/// message, which contains a `ControlMsg`.
pub enum Message<Data> {
    /// A data message.
    Data {
        /// The data traversing the dataflow.
        data: Data,
    },

    /// A control message.
    Control {
        /// The control message.
        control: ControlMsg,
    },
}

/// Control messages for the dataflow engine.
pub enum ControlMsg {
    /// Indicates that a downstream component (either internal or external) has reliably received
    /// and processed telemetry data.
    Ack {
        /// The ID of the message being acknowledged.
        id: u64,
    },

    /// Indicates that a downstream component (either internal or external) failed to process or
    /// deliver telemetry data. The NACK signal includes a reason, such as exceeding a deadline,
    /// downstream system unavailability, or other conditions preventing successful processing.
    Nack {
        /// The ID of the message not being acknowledged.
        id: u64,
        /// The reason for the NACK.
        reason: String,
    },

    /// Indicates a change in the configuration of a node. For example, a config message can
    /// instruct a Filter Processor to include or exclude certain attributes, or notify a Retry
    /// Processor to adjust backoff settings.
    Config {
        /// The new configuration.
        config: serde_json::Value,
    },

    /// Emitted upon timer expiration, used to trigger scheduled tasks (e.g., batch emissions).
    TimerTick {
        // TBD
    },

    /// A graceful shutdown message requiring the node to finish processing messages and release
    /// resources.
    Shutdown {
        /// The reason for the shutdown.
        reason: String,
    },
}
