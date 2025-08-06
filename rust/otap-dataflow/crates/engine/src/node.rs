// SPDX-License-Identifier: Apache-2.0

//! Set of traits defining the common properties between all types of nodes in the pipeline engine.
//!
//! Receivers, processors, and exporters implement the [`Node`] trait.
//! Receivers and processors implement the [`NodeWithPDataSender`] trait.
//! Processors and exporters implement the [`NodeWithPDataReceiver`] trait.

use crate::control::NodeControlMsg;
use crate::error::Error;
use crate::message::{Receiver, Sender};
use otap_df_channel::error::SendError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::{NodeId, PortName};
use std::sync::Arc;

/// Common trait for nodes in the pipeline.
#[async_trait::async_trait(?Send)]
pub trait Node {
    /// Flag indicating whether the node is shared (true) or local (false).
    #[must_use]
    fn is_shared(&self) -> bool;

    /// Returns a reference to the node's user configuration.
    #[must_use]
    fn user_config(&self) -> Arc<NodeUserConfig>;

    /// Sends a control message to the node.
    async fn send_control_msg(&self, msg: NodeControlMsg) -> Result<(), SendError<NodeControlMsg>>;
}

/// Trait for nodes that can send pdata to a specific port.
pub trait NodeWithPDataSender<PData>: Node {
    /// Sets the sender for pdata messages on the node.
    fn set_pdata_sender(
        &mut self,
        node_id: NodeId,
        _port: PortName,
        sender: Sender<PData>,
    ) -> Result<(), Error<PData>>;
}

/// Trait for nodes that can receive pdata.
pub trait NodeWithPDataReceiver<PData>: Node {
    /// Sets the receiver for pdata messages on the node.
    fn set_pdata_receiver(
        &mut self,
        node_id: NodeId,
        receiver: Receiver<PData>,
    ) -> Result<(), Error<PData>>;
}
