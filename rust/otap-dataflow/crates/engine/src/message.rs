// SPDX-License-Identifier: Apache-2.0

//! Message definitions for the dataflow engine.

/// A message that can be sent to a node (i.e. receiver, processor, exporter, or connector).
///
/// A message is either a `Data` message, which contains a payload of type `Data`, or a `Control`
/// message, which contains a `ControlMsg`.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

impl<Data> Message<Data> {
    /// Create a data message with the given payload.
    #[must_use]
    pub fn data_msg(data: Data) -> Self {
        Message::Data { data }
    }

    /// Create a ACK control message with the given ID.
    #[must_use]
    pub fn ack_ctrl_msg(id: u64) -> Self {
        Message::Control {
            control: ControlMsg::Ack { id },
        }
    }

    /// Create a NACK control message with the given ID and reason.
    #[must_use]
    pub fn nack_ctrl_msg(id: u64, reason: &str) -> Self {
        Message::Control {
            control: ControlMsg::Nack {
                id,
                reason: reason.to_owned(),
            },
        }
    }

    /// Creates a config control message with the given configuration.
    #[must_use]
    pub fn config_ctrl_msg(config: serde_json::Value) -> Self {
        Message::Control {
            control: ControlMsg::Config { config },
        }
    }

    /// Creates a timer tick control message.
    #[must_use]
    pub fn timer_tick_ctrl_msg() -> Self {
        Message::Control {
            control: ControlMsg::TimerTick {},
        }
    }

    /// Creates a shutdown control message with the given reason.
    #[must_use]
    pub fn shutdown_ctrl_msg(reason: &str) -> Self {
        Message::Control {
            control: ControlMsg::Shutdown {
                reason: reason.to_owned(),
            },
        }
    }

    /// Checks if this message is a data message.
    #[must_use]
    pub fn is_data(&self) -> bool {
        matches!(self, Message::Data { .. })
    }

    /// Checks if this message is a control message.
    #[must_use]
    pub fn is_control(&self) -> bool {
        matches!(self, Message::Control { .. })
    }

    /// Checks if this message is a shutdown control message.
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        matches!(
            self,
            Message::Control {
                control: ControlMsg::Shutdown { .. }
            }
        )
    }
}
