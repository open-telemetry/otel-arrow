// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Provides types and traits for control messages exchanged between the pipeline engine and nodes.
//! Enables management of node behavior, configuration, and lifecycle events, including shutdown,
//! configuration updates, and timer management.

use crate::message::Sender;
use crate::shared::message::{SharedReceiver, SharedSender};
use otap_df_telemetry::reporter::MetricsReporter;
use std::time::Duration;

/// The ACK
#[derive(Debug, Clone)]
pub struct AckMsg<PData> {
    /// Reply-to information.
    // Note! This Box<PData> should be <Context> which can be
    // accomplished by making PData bound by a trait and associated
    // type i.e., we would use PData::Context. That will be a large
    // change, for now we expect PData with empty data and returning
    // context.
    context: Option<Box<PData>>,

    /// Rejected items
    rejected: Option<(i64, String)>,
}

impl<PData> AckMsg<PData> {
    pub fn take_context(&mut self) -> Context {
        self.context.take().unwrap().context()
    }
}

impl<PData> AckMsg<PData> {
    /// Create a new Ack message
    pub fn new(context: PData, rejected: Option<(i64, String)>) -> Self {
        Self {
            context: Box::new(context),
            rejected,
        }
    }
}

/// The NACK
#[derive(Debug, Clone)]
pub struct NackMsg<PData> {
    /// Reply-to information.
    // Note! This Box<PData> should be <Context> which can be
    // accomplished by making PData bound by a trait and associated
    // type i.e., we would use PData::Context. That will be a large
    // change, for now we expect PData with empty data and returning
    // context.
    context: Box<PData>,

    /// Human-readable reason for the NACK.
    reason: String,

    /// Protocol-independent permanent status
    permanent: bool,

    /// Protocol-independent code number
    code: Option<i32>,

    /// Optional return pdata
    pdata: Option<Box<PData>>,
}

impl<PData> NackMsg<PData> {
    /// Create a new Nack.
    pub fn new(
        context: PData,
        reason: String,
        permanent: bool,
        code: Option<i32>,
        pdata: Option<PData>,
    ) -> Self {
        Self {
            reason,
            permanent,
            code,
            context: Box::new(context),
            pdata: pdata.map(Box::new),
        }
    }
}

/// An Ack or a Nack
pub enum AckOrNack<PData> {
    /// The Ack
    Ack(AckMsg<PData>),
    /// The Nack
    Nack(NackMsg<PData>),
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
pub enum PipelineControlMsg<PData> {
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
    /// Requests shutdown of the node request manager.
    Shutdown,

    /// Let the pipeline manager deliver an Ack.
    DeliverAck {
        // From which component
        // from_node_id: usize,
        //
        /// Acknowledgement context
        ack: AckMsg<PData>,
    },

    /// Let the pipeline manager deliver an Nack.
    DeliverNack {
        // From which component
        // from_node_id: usize,
        //
        /// Acknowledgement context
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
}

/// Type alias for the channel sender used by nodes to send requests to the pipeline engine.
///
/// This is a multi-producer, single-consumer (MPSC) channel.
pub type PipelineCtrlMsgSender<PData> = SharedSender<PipelineControlMsg<PData>>;

/// Type alias for the channel receiver used by the pipeline engine to receive node requests.
///
/// This is a multi-producer, single-consumer (MPSC) channel.
pub type PipelineCtrlMsgReceiver<PData> = SharedReceiver<PipelineControlMsg<PData>>;

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
pub fn pipeline_ctrl_msg_channel<PData>(
    capacity: usize,
) -> (PipelineCtrlMsgSender<PData>, PipelineCtrlMsgReceiver<PData>) {
    let (tx, rx) = tokio::sync::mpsc::channel(capacity);
    (
        SharedSender::MpscSender(tx),
        SharedReceiver::MpscReceiver(rx),
    )
}
