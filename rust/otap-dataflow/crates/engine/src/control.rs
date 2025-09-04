// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Provides types and traits for control messages exchanged between the pipeline engine and nodes.
//! Enables management of node behavior, configuration, and lifecycle events, including shutdown,
//! configuration updates, and timer management.

use crate::error::TypedError;
use crate::message::Sender;
use crate::node::{NodeId, NodeType};
use crate::shared::message::{SharedReceiver, SharedSender};
use otap_df_channel::error::SendError;
use otap_df_telemetry::reporter::MetricsReporter;
use std::collections::HashMap;
use std::time::Duration;

/// Control messages sent by the pipeline engine to nodes to manage their behavior,
/// configuration, and lifecycle.
#[derive(Debug, Clone)]
pub enum NodeControlMsg<PData> {
    /// Acknowledges that a downstream component (internal or external) has reliably received
    /// and processed telemetry data for the specified message ID.
    ///
    /// Typically used for confirming successful delivery or processing.
    Ack {
        /// Unique identifier of the message being acknowledged.
        id: u64,
    },

    /// Indicates that a downstream component failed to process or deliver telemetry data.
    ///
    /// The NACK signal includes a reason, such as exceeding a deadline, downstream system
    /// unavailability, or other conditions preventing successful processing.
    Nack {
        /// Unique identifier of the message not being acknowledged.
        id: u64,
        /// Human-readable reason for the NACK.
        reason: String,

        /// Placeholder for optional return value, making it possible for the
        /// retry sender to be stateless.
        pdata: Option<Box<PData>>,
    },

    /// Notifies the node of a configuration change.
    ///
    /// For example, instructs a Filter Processor to include/exclude attributes, or a Retry
    /// Processor to adjust backoff settings.
    Config {
        /// The new configuration as a JSON value.
        config: serde_json::Value,
    },

    /// Emitted when a scheduled timer expires, used to trigger periodic or scheduled tasks
    /// (e.g., batch emissions).
    ///
    /// This variant currently carries no additional data.
    TimerTick {
        // For future usage
    },

    /// Dedicated signal to ask a node to collect/flush its local telemetry metrics.
    ///
    /// This separates metrics collection from the generic TimerTick to allow
    /// fine-grained scheduling of telemetry without conflating it with other periodic tasks.
    CollectTelemetry {
        /// Metrics reporter used to collect telemetry metrics.
        metrics_reporter: MetricsReporter,
    },

    /// Requests a graceful shutdown, requiring the node to finish processing messages and
    /// release resources by the specified deadline.
    ///
    /// A deadline of zero duration indicates an immediate shutdown.
    Shutdown {
        /// Deadline for shutdown (in seconds).
        deadline: Duration,
        /// Human-readable reason for the shutdown.
        reason: String,
    },
}

/// Control messages sent by nodes to the pipeline engine to manage node-specific operations
/// and control pipeline behavior.
#[derive(Debug, Clone)]
pub enum PipelineControlMsg {
    /// Requests the pipeline engine to start a periodic timer for the specified node.
    StartTimer {
        /// Identifier of the node for which the timer is being started.
        node_id: usize,
        /// Duration of the timer interval.
        duration: Duration,
    },
    /// Requests cancellation of a periodic timer for the specified node.
    CancelTimer {
        /// Identifier of the node for which the timer is being canceled.
        node_id: usize,
    },

    /// Requests the pipeline engine to start a periodic telemetry collection timer
    /// for the specified node.
    StartTelemetryTimer {
        /// Identifier of the node for which the timer is being started.
        node_id: usize,
        /// Duration of the telemetry timer interval.
        duration: Duration,
    },
    /// Requests cancellation of the periodic telemetry collection timer for the specified node.
    CancelTelemetryTimer {
        /// Identifier of the node for which the telemetry timer is being canceled.
        node_id: usize,
    },
    /// Requests shutdown of the pipeline.
    Shutdown {
        /// Human-readable reason for the shutdown.
        reason: String,
    },
}

/// Trait for nodes that can receive and process control messages from the pipeline engine.
///
/// Implement this trait for node types that need to handle control messages such as configuration
/// updates, shutdown requests, or timer events. Implementers are not required to be thread-safe.
#[async_trait::async_trait(?Send)]
pub trait Controllable<PData> {
    /// Returns the sender for control messages to this node.
    ///
    /// Used for direct message passing from the pipeline engine.
    fn control_sender(&self) -> Sender<NodeControlMsg<PData>>;
}

impl<PData> NodeControlMsg<PData> {
    /// Returns `true` if this control message is a shutdown request.
    #[must_use]
    pub const fn is_shutdown(&self) -> bool {
        matches!(self, NodeControlMsg::Shutdown { .. })
    }
}

/// Type alias for the channel sender used by nodes to send requests to the pipeline engine.
///
/// This is a multi-producer, single-consumer (MPSC) channel.
pub type PipelineCtrlMsgSender = SharedSender<PipelineControlMsg>;

/// Type alias for the channel receiver used by the pipeline engine to receive node requests.
///
/// This is a multi-producer, single-consumer (MPSC) channel.
pub type PipelineCtrlMsgReceiver = SharedReceiver<PipelineControlMsg>;

/// Creates a shared node request channel for communication from nodes to the pipeline engine.
///
/// The channel is multi-producer, single-consumer (MPSC), allowing multiple nodes to send requests
/// to a single pipeline engine instance.
///
/// # Arguments
///
/// * `capacity` - The maximum number of messages the channel can buffer.
///
/// # Returns
///
/// A tuple containing the sender and receiver ends of the channel.
pub fn pipeline_ctrl_msg_channel(
    capacity: usize,
) -> (PipelineCtrlMsgSender, PipelineCtrlMsgReceiver) {
    let (tx, rx) = tokio::sync::mpsc::channel(capacity);
    (
        SharedSender::MpscSender(tx),
        SharedReceiver::MpscReceiver(rx),
    )
}

/// Typed control message sender for a specific node type.
pub struct TypedControlSender<PData> {
    /// Unique identifier of the node.
    pub node_id: NodeId,
    /// Type of the node (Receiver, Processor, Exporter).
    pub node_type: NodeType,
    /// The control message sender for the node.
    pub sender: Sender<NodeControlMsg<PData>>,
}

/// Holds the control message senders for all nodes in the pipeline.
pub struct ControlSenders<PData> {
    senders: HashMap<usize, TypedControlSender<PData>>,
}

impl<PData> TypedControlSender<PData> {
    /// Sends a control message to the node, awaiting until the message is sent.
    #[inline]
    pub async fn send(
        &self,
        msg: NodeControlMsg<PData>,
    ) -> Result<(), SendError<NodeControlMsg<PData>>> {
        self.sender.send(msg).await
    }

    /// Tries to send a control message to the node without awaiting.
    #[inline]
    pub fn try_send(
        &self,
        msg: NodeControlMsg<PData>,
    ) -> Result<(), SendError<NodeControlMsg<PData>>> {
        self.sender.try_send(msg)
    }
}

impl<PData> Default for ControlSenders<PData> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PData> ControlSenders<PData> {
    /// Creates a new `ControlSenders` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            senders: HashMap::new(),
        }
    }

    /// Gets the control message sender for a specific node by its ID.
    ///
    /// Returns `None` if no sender is found for the given node ID.
    #[must_use]
    pub fn get(&self, node_id: usize) -> Option<&TypedControlSender<PData>> {
        self.senders.get(&node_id)
    }

    /// Registers a control message sender for a specific node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique identifier of the node.
    /// * `node_type` - Type of the node (Receiver, Processor, Exporter).
    /// * `sender` - The control message sender for the node.
    pub fn register(
        &mut self,
        node_id: NodeId,
        node_type: NodeType,
        sender: Sender<NodeControlMsg<PData>>,
    ) {
        _ = self.senders.insert(
            node_id.index,
            TypedControlSender {
                node_id,
                node_type,
                sender,
            },
        );
    }

    /// Returns the number of registered control message senders.
    #[must_use]
    pub fn len(&self) -> usize {
        self.senders.len()
    }

    /// Returns `true` if there are no registered control message senders.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.senders.is_empty()
    }

    /// Broadcast a shutdown control message to all receivers in order to drain the pipelines.
    ///
    /// Returns `Ok(())` if all messages were sent successfully, or a vector of errors
    /// if any sends failed.
    pub async fn shutdown_receivers(
        &self,
        reason: String,
    ) -> Result<(), Vec<TypedError<NodeControlMsg<PData>>>> {
        self.shutdown_nodes(Some(NodeType::Receiver), reason).await
    }

    /// Broadcast a shutdown control message to all nodes in the pipeline. This is usually not the
    /// preferred way to shut down a pipeline, as it does not allow for graceful draining. Use
    /// `shutdown_receivers` instead to first shut down receivers and let the rest of the
    /// pipeline drain.
    ///
    /// Returns `Ok(())` if all messages were sent successfully, or a vector of errors
    /// if any sends failed.
    pub async fn shutdown_all(
        &self,
        reason: String,
    ) -> Result<(), Vec<TypedError<NodeControlMsg<PData>>>> {
        self.shutdown_nodes(None, reason).await
    }

    /// Internal helper method to broadcast shutdown messages to nodes.
    ///
    /// # Arguments
    ///
    /// - `node_type_filter`: If `Some(node_type)`, only send to nodes of that type.
    ///   If `None`, send to all nodes.
    /// - `reason`: The reason for the shutdown.
    ///
    /// Returns `Ok(())` if all messages were sent successfully, or a vector of errors
    /// if any sends failed.
    async fn shutdown_nodes(
        &self,
        node_type_filter: Option<NodeType>,
        reason: String,
    ) -> Result<(), Vec<TypedError<NodeControlMsg<PData>>>> {
        let mut errors: Vec<TypedError<NodeControlMsg<PData>>> = Vec::new();

        for typed_sender in self.senders.values() {
            // Apply filter if specified
            if let Some(filter_type) = node_type_filter {
                if typed_sender.node_type != filter_type {
                    continue;
                }
            }

            let shutdown_msg = NodeControlMsg::Shutdown {
                deadline: Default::default(),
                reason: reason.clone(),
            };

            if let Err(error) = typed_sender.sender.send(shutdown_msg).await {
                errors.push(TypedError::NodeControlMsgSendError {
                    node: typed_sender.node_id.clone(),
                    error,
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
