// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Provides types and traits for control messages exchanged between the pipeline engine and nodes.
//! Enables management of node behavior, configuration, and lifecycle events, including shutdown,
//! configuration updates, and timer management.

use crate::clock;
use crate::error::{Error, TypedError};
use crate::message::Sender;
use crate::node::{NodeId, NodeType};
use crate::shared::message::{SharedReceiver, SharedSender};
use bytemuck::Pod;
use otap_df_channel::error::SendError;
use otap_df_telemetry::reporter::MetricsReporter;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

/// Returns a monotonic timestamp in nanoseconds since an arbitrary process epoch.
/// Used for duration calculations in pipeline component metrics.
///
/// `0` is reserved as the sentinel for "timestamp not set", so this function
/// always returns a strictly positive value.
///
#[must_use]
pub fn nanos_since_birth() -> u64 {
    clock::nanos_since_birth()
}

/// A 8-byte context value. Supports conversion to and from plain data
/// using bytemuck.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Context8u8([u8; 8]);

impl<T: Pod> From<T> for Context8u8 {
    /// From T to Context8u8
    fn from(v: T) -> Self {
        const {
            assert!(size_of::<T>() == 8);
        }
        Self(bytemuck::cast(v))
    }
}

// --- From Context8u8 to and from Ts of interest

impl TryFrom<Context8u8> for usize {
    type Error = Error;

    fn try_from(v: Context8u8) -> Result<Self, Self::Error> {
        bytemuck::try_cast(v.0).map_err(|_| Error::InternalError {
            message: "bytecast error".into(),
        })
    }
}

impl From<Context8u8> for u64 {
    fn from(v: Context8u8) -> u64 {
        bytemuck::cast(v.0)
    }
}

impl From<Context8u8> for f64 {
    fn from(v: Context8u8) -> f64 {
        bytemuck::cast(v.0)
    }
}

/// Standard context values hold three caller-specified fields. The
/// size is arbitrary, but shouldn't be larger than needed by
/// callers. For example: retry count, sequence and generation
/// numbers, deadline, num_items, etc.
pub type CallData = SmallVec<[Context8u8; 3]>;

/// Engine-managed call data envelope. Wraps the CallData with an envelope
/// containing timestamp. Lives on the forward path (in context stack frames).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RouteData {
    /// Component-specific opaque data (formerly the entire `CallData`).
    pub calldata: CallData,
    /// Entry timestamp, see nanos_since_birth().
    pub entry_time_ns: u64,
    /// Producer's output port index.
    pub output_port_index: u16,
}

/// Return-path data carried in AckMsg/NackMsg. Contains all the fields
/// from the forward-path RouteData plus a return-path timestamp.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnwindData {
    /// The route we are returning through.
    pub route: RouteData,
    /// Ack or Nack timestamp, see nanos_since_birth().
    pub return_time_ns: u64,
}

impl UnwindData {
    /// Build new return route data.
    #[must_use]
    pub fn new(route: RouteData, return_time_ns: u64) -> Self {
        Self {
            route,
            return_time_ns,
        }
    }
}

/// Per-node interests, context, and identity.
///
/// A frame lives on the context stack carried inside PData. Each node
/// that subscribes (or has metrics interests) pushes one frame. During
/// ack/nack unwinding the controller pops frames one at a time.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// Declares the set of interests this node has (Acks, Nacks, ...)
    pub interests: crate::Interests,
    /// The caller's route data.
    pub route: RouteData,
    /// The caller's node_id for routing.
    pub node_id: usize,
}

/// The ACK message.
#[derive(Debug, Clone)]
pub struct AckMsg<PData> {
    /// Accepted pdata being returned.
    pub accepted: Box<PData>,

    /// Routing information returned.
    pub unwind: UnwindData,
}

impl<PData> AckMsg<PData> {
    /// Creates a new ACK.
    pub fn new(accepted: PData) -> Self {
        Self {
            accepted: Box::new(accepted),
            unwind: UnwindData::default(),
        }
    }
}

/// The NACK message.
#[derive(Debug, Clone)]
pub struct NackMsg<PData> {
    /// Human-readable reason for the NACK.
    pub reason: String,

    /// Refused pdata being returned.
    pub refused: Box<PData>,

    /// Subscriber information returned.
    pub unwind: UnwindData,

    /// Permanent status.
    pub permanent: bool,
}

impl<PData> NackMsg<PData> {
    /// Creates a new non-permanent NACK.
    pub fn new<T: Into<String>>(reason: T, refused: PData) -> Self {
        Self::new_internal(reason, refused, false)
    }

    /// Creates a new permanent NACK.
    pub fn new_permanent<T: Into<String>>(reason: T, refused: PData) -> Self {
        Self::new_internal(reason, refused, true)
    }

    fn new_internal<T: Into<String>>(reason: T, refused: PData, permanent: bool) -> Self {
        Self {
            reason: reason.into(),
            refused: Box::new(refused),
            unwind: UnwindData::default(),
            permanent,
        }
    }
}

/// Control messages sent by the pipeline engine to nodes to manage their behavior,
/// configuration, and lifecycle.
#[derive(Debug, Clone)]
pub enum NodeControlMsg<PData> {
    /// Acknowledges that a downstream component (internal or external) has reliably received
    /// and processed telemetry data for the specified message ID.
    ///
    /// Typically used for confirming successful delivery or processing.
    Ack(AckMsg<PData>),

    /// Indicates that a downstream component failed to process or deliver telemetry data.
    ///
    /// The NACK signal includes a reason, such as exceeding a deadline, downstream system
    /// unavailability, or other conditions preventing successful processing.
    Nack(NackMsg<PData>),

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

    /// Delayed data returning to the node which delayed it.
    DelayedData {
        /// When resumed
        when: Instant,

        /// The data.
        data: Box<PData>,
    },

    /// Requests that a receiver stop admitting new external work while keeping
    /// already-admitted work alive until it can finish receiver-local drain work.
    DrainIngress {
        /// Deadline for draining ingress.
        deadline: Instant,
        /// Human-readable reason for the shutdown.
        reason: String,
    },

    /// Requests a graceful shutdown, requiring the node to finish processing messages and
    /// release resources by the specified deadline.
    ///
    /// A deadline of zero duration indicates an immediate shutdown.
    Shutdown {
        /// Deadline for shutdown.
        deadline: Instant,
        /// Human-readable reason for the shutdown.
        reason: String,
    },
}

/// Runtime-control messages sent by nodes to the pipeline runtime for
/// orchestration, delayed-data handling, and shutdown.
#[derive(Debug, Clone)]
pub enum RuntimeControlMsg<PData> {
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

        /// Temporarily placed, see #1083. Placement is arbitrary.
        _temp: PhantomData<PData>,
    },
    /// Delay this data.
    DelayData {
        /// The delayer's node_id
        node_id: usize,

        /// When to resume
        when: Instant,

        /// The data
        data: Box<PData>,
    },
    /// Indicates that a receiver has stopped admitting new ingress and
    /// completed any receiver-local drain work needed before downstream
    /// shutdown begins.
    ReceiverDrained {
        /// Identifier of the receiver that completed ingress drain.
        node_id: usize,
    },
    /// Requests shutdown of the pipeline.
    Shutdown {
        /// Deadline for shutdown.
        deadline: Instant,

        /// Human-readable reason for the shutdown.
        reason: String,
    },
}

/// Pipeline-result messages sent by nodes to the pipeline runtime for
/// Ack/Nack unwinding.
#[derive(Debug, Clone)]
pub enum PipelineCompletionMsg<PData> {
    /// Deliver an Ack to the preceding subscriber in the pipeline.
    DeliverAck {
        /// The Ack.
        ack: AckMsg<PData>,
    },
    /// Deliver a Nack to the preceding subscriber in the pipeline.
    DeliverNack {
        /// The Nack.
        nack: NackMsg<PData>,
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

    /// Returns `true` if this control message is an ingress-drain request.
    #[must_use]
    pub const fn is_drain_ingress(&self) -> bool {
        matches!(self, NodeControlMsg::DrainIngress { .. })
    }
}

/// Type alias for the channel sender used by nodes to send requests to the pipeline runtime.
///
/// This is a multi-producer, single-consumer (MPSC) channel.
pub type RuntimeCtrlMsgSender<PData> = SharedSender<RuntimeControlMsg<PData>>;

/// Type alias for the channel receiver used by the pipeline runtime to receive node requests.
///
/// This is a multi-producer, single-consumer (MPSC) channel.
pub type RuntimeCtrlMsgReceiver<PData> = SharedReceiver<RuntimeControlMsg<PData>>;

/// Type alias for the channel sender used by nodes to send Ack/Nack unwind
/// requests to the pipeline runtime.
pub type PipelineCompletionMsgSender<PData> = SharedSender<PipelineCompletionMsg<PData>>;

/// Type alias for the channel receiver used by the pipeline runtime to receive
/// Ack/Nack unwind requests from nodes.
pub type PipelineCompletionMsgReceiver<PData> = SharedReceiver<PipelineCompletionMsg<PData>>;

/// Trait for sending admin commands without depending on the pipeline data type.
pub trait PipelineAdminSender: Send + Sync {
    /// Attempts to send a shutdown request to the pipeline with the provided deadline.
    fn try_send_shutdown(&self, deadline: Instant, reason: String) -> Result<(), Error>;
}

/// Creates a shared runtime-control channel for communication from nodes to the pipeline runtime.
///
/// The channel is multi-producer, single-consumer (MPSC), allowing multiple nodes to send requests
/// to a single pipeline runtime instance.
///
/// # Arguments
///
/// * `capacity` - The maximum number of messages the channel can buffer.
///
/// # Returns
///
/// A tuple containing the sender and receiver ends of the channel.
pub fn runtime_ctrl_msg_channel<PData>(
    capacity: usize,
) -> (RuntimeCtrlMsgSender<PData>, RuntimeCtrlMsgReceiver<PData>) {
    let (tx, rx) = tokio::sync::mpsc::channel(capacity);
    (SharedSender::mpsc(tx), SharedReceiver::mpsc(rx))
}

/// Creates the shared pipeline-completion channel used for Ack/Nack unwinding.
pub fn pipeline_completion_msg_channel<PData>(
    capacity: usize,
) -> (
    PipelineCompletionMsgSender<PData>,
    PipelineCompletionMsgReceiver<PData>,
) {
    let (tx, rx) = tokio::sync::mpsc::channel(capacity);
    (SharedSender::mpsc(tx), SharedReceiver::mpsc(rx))
}

/// Typed control message sender for a specific node type.
#[derive(Clone)]
pub struct TypedControlSender<PData> {
    /// Unique identifier of the node.
    pub node_id: NodeId,
    /// Type of the node (Receiver, Processor, Exporter).
    pub node_type: NodeType,
    /// The control message sender for the node.
    pub sender: Sender<NodeControlMsg<PData>>,
}

/// Holds the control message senders for all nodes in the pipeline.
#[derive(Clone)]
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

    /// Returns the number of registered receivers.
    #[must_use]
    pub fn receiver_count(&self) -> usize {
        self.senders
            .values()
            .filter(|sender| sender.node_type == NodeType::Receiver)
            .count()
    }

    /// Returns the registered receiver ids.
    #[must_use]
    pub fn receiver_ids(&self) -> Vec<usize> {
        self.senders
            .values()
            .filter(|sender| sender.node_type == NodeType::Receiver)
            .map(|sender| sender.node_id.index)
            .collect()
    }

    /// Returns the registered non-receiver ids.
    #[must_use]
    pub fn non_receiver_ids(&self) -> Vec<usize> {
        self.senders
            .values()
            .filter(|sender| sender.node_type != NodeType::Receiver)
            .map(|sender| sender.node_id.index)
            .collect()
    }

    /// Broadcast an ingress-drain control message to all receivers so they
    /// stop accepting new external work before downstream shutdown begins.
    pub async fn drain_receivers(
        &self,
        deadline: Instant,
        reason: String,
    ) -> Result<(), Vec<TypedError<NodeControlMsg<PData>>>> {
        let mut errors: Vec<TypedError<NodeControlMsg<PData>>> = Vec::new();

        for typed_sender in self.senders.values() {
            if typed_sender.node_type != NodeType::Receiver {
                continue;
            }

            if let Err(error) = typed_sender
                .sender
                .send(NodeControlMsg::DrainIngress {
                    deadline,
                    reason: reason.clone(),
                })
                .await
            {
                errors.push(TypedError::NodeControlMsgSendError {
                    node_id: typed_sender.node_id.index,
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

    /// Broadcast a shutdown control message to all receivers in order to drain the pipelines.
    ///
    /// Returns `Ok(())` if all messages were sent successfully, or a vector of errors
    /// if any sends failed.
    pub async fn shutdown_receivers(
        &self,
        deadline: Instant,
        reason: String,
    ) -> Result<(), Vec<TypedError<NodeControlMsg<PData>>>> {
        self.shutdown_nodes(Some(NodeType::Receiver), deadline, reason)
            .await
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
        deadline: Instant,
        reason: String,
    ) -> Result<(), Vec<TypedError<NodeControlMsg<PData>>>> {
        self.shutdown_nodes(None, deadline, reason).await
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
        deadline: Instant,
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
                deadline,
                reason: reason.clone(),
            };

            if let Err(error) = typed_sender.sender.send(shutdown_msg).await {
                errors.push(TypedError::NodeControlMsgSendError {
                    node_id: typed_sender.node_id.index,
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

impl<PData> PipelineAdminSender for SharedSender<RuntimeControlMsg<PData>>
where
    PData: Send + Sync + 'static,
{
    fn try_send_shutdown(&self, deadline: Instant, reason: String) -> Result<(), Error> {
        let shutdown_msg = RuntimeControlMsg::Shutdown { deadline, reason };

        self.try_send(shutdown_msg)
            .map_err(|e| Error::RuntimeMsgError {
                error: format!("Failed to send shutdown message: {}", e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permanent_status() {
        assert!(!NackMsg::new("just bad news", ()).permanent);
        assert!(NackMsg::new_permanent("very bad news", ()).permanent);
    }
}
