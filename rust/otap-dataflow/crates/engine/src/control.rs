// SPDX-License-Identifier: Apache-2.0

//! Control message infrastructure.

use crate::message::Sender;
use crate::shared::message::{SharedReceiver, SharedSender};
use otap_df_channel::error::SendError;
use otap_df_config::NodeId;
use std::time::Duration;

/// Control messages are sent by the pipeline engine to nodes to manage their behavior and
/// operations.
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
    /// resources by a specified deadline. A deadline of 0 indicates an immediate shutdown.
    Shutdown {
        /// The deadline for the shutdown.
        deadline: Duration,
        /// The reason for the shutdown.
        reason: String,
    },
}

/// Requests sent from nodes to the pipeline engine for node-specific control operations.
#[derive(Debug, Clone)]
pub enum PipelineControlMsg {
    /// Start a periodic timer for a node.
    StartTimer {
        /// The ID of the node for which the timer is being started.
        node_id: NodeId,
        /// The duration of the timer.
        duration: Duration,
    },
    /// Cancel a periodic timer for a node.
    CancelTimer {
        /// The ID of the node for which the timer is being canceled.
        node_id: NodeId,
    },
    /// Shutdown the node request manager.
    Shutdown,
}

/// Trait for nodes that can receive control messages.
#[async_trait::async_trait(?Send)]
pub trait Controllable {
    /// Sends a control message to the node.
    async fn send_control_msg(&self, msg: ControlMsg) -> Result<(), SendError<ControlMsg>>;

    /// Returns the control message sender for the node.
    fn control_sender(&self) -> Sender<ControlMsg>;
}

impl ControlMsg {
    /// Checks if this control message is a shutdown message.
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        matches!(self, ControlMsg::Shutdown { .. })
    }
}

/// Channel sender for node requests to the pipeline engine.
pub type NodeRequestSender = SharedSender<PipelineControlMsg>;

/// Channel receiver for node requests from the pipeline engine.
pub type NodeRequestReceiver = SharedReceiver<PipelineControlMsg>;

/// Helper to create a shared node request channel (n nodes, 1 pipeline engine -> MPSC).
pub fn node_request_channel(capacity: usize) -> (NodeRequestSender, NodeRequestReceiver) {
    let (tx, rx) = tokio::sync::mpsc::channel(capacity);
    (
        SharedSender::MpscSender(tx),
        SharedReceiver::MpscReceiver(rx),
    )
}
