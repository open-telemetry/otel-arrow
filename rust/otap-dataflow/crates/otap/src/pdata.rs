// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module manages how telemetry pipeline data (pdata) flows through pipeline
//! components (receivers, processors, exporters) and how components receive
//! acknowledgments (Ack) and negative acknowledgments (Nack) about this pdata.
//!
//! This includes:
//! - Context: a stack of pipeline component subscriptions
//! - Interests: allows components to declare whether they subscribe to Ack/Nack
//!
//! Components can subscribe to receive notifications when their pdata is accepted (Ack) or
//! encountered issues (Nack) downstream, optionally preserving the payload for retry or logging.
//! This functionality is exposed through various traits implemented by effect handlers.

use async_trait::async_trait;
use otap_df_config::PortName;
use otap_df_config::{SignalFormat, SignalType};
use otap_df_engine::error::{Error, TypedError};
use otap_df_engine::{
    ConsumerEffectHandlerExtension, Interests, MessageSourceLocalEffectHandlerExtension,
    MessageSourceSharedEffectHandlerExtension, ProducerEffectHandlerExtension,
    control::{AckMsg, CallData, NackMsg},
};
use otap_df_pdata::OtapPayload;

/// Context for OTAP requests
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Context {
    stack: Vec<Frame>,
}

impl Context {
    /// Create a context with reserved frame capacity to avoid reallocating
    /// when the first subscriber is pushed.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
        }
    }

    /// Subscribe to a set of interests.
    pub(crate) fn subscribe_to(
        &mut self,
        mut interests: Interests,
        calldata: CallData,
        node_id: usize,
    ) {
        if let Some(last) = self.stack.last_mut() {
            // Inherit the preceding frame's RETURN_DATA bit
            interests |= last.interests & Interests::RETURN_DATA;

            // We should never subscribe twice.
            debug_assert_ne!(node_id, last.node_id);
        }
        self.stack.push(Frame {
            interests,
            node_id,
            calldata,
        });
    }

    /// Consume frames to locate the most recent subscriber with ACKS.
    /// This is a "transfer function" used in the engine for route_ack.
    #[must_use]
    pub fn next_ack(mut ack: AckMsg<OtapPdata>) -> Option<(usize, AckMsg<OtapPdata>)> {
        ack.accepted
            .context
            .next_with_interest(Interests::ACKS)
            .map(|frame| {
                if (frame.interests & Interests::RETURN_DATA).is_empty() {
                    let _drop = ack.accepted.take_payload();
                }
                ack.calldata = frame.calldata;
                (frame.node_id, ack)
            })
    }

    /// Consume frames to locate the most recent subscriber with NACKS.
    /// This is a "transfer function" used in the engine for route_nack.
    #[must_use]
    pub fn next_nack(mut nack: NackMsg<OtapPdata>) -> Option<(usize, NackMsg<OtapPdata>)> {
        nack.refused
            .context
            .next_with_interest(Interests::NACKS)
            .map(|frame| {
                if (frame.interests & Interests::RETURN_DATA).is_empty() {
                    let _drop = nack.refused.take_payload();
                }
                nack.calldata = frame.calldata;
                (frame.node_id, nack)
            })
    }

    fn next_with_interest(&mut self, int: Interests) -> Option<Frame> {
        while let Some(frame) = self.stack.pop() {
            if frame.interests.contains(int) {
                return Some(frame);
            }
        }
        None
    }

    /// Determine whether the context is requesting payload returned.
    #[must_use]
    pub fn may_return_payload(&self) -> bool {
        self.stack
            .last()
            .map(|f| f.interests & Interests::RETURN_DATA != Interests::empty())
            .unwrap_or(false)
    }

    /// Return the current source calldata. This is used with the
    /// DelayedData message, in which a node delivers a message to
    /// itself.
    ///
    /// This is also useful in testing, it indicates the data that was
    /// sent by the source node.
    #[must_use]
    pub fn source_calldata(&self) -> Option<CallData> {
        self.stack.last().map(|f| f.calldata.clone())
    }

    /// Are there any subscribers with actual interests (ACKS or NACKS)?
    #[must_use]
    pub fn has_subscribers(&self) -> bool {
        self.stack.iter().any(|f| !f.interests.is_empty())
    }

    /// Set the source node for this context.
    pub fn set_source_node(&mut self, node_id: usize) {
        let mut interests = Interests::empty();
        if let Some(last) = self.stack.last() {
            if node_id == last.node_id {
                // The node called subscribe_to() itself.
                return;
            }
            // Inherit the preceding frame's RETURN_DATA bit.
            interests = last.interests & Interests::RETURN_DATA;
        }
        self.stack.push(Frame {
            interests,
            node_id,
            calldata: CallData::default(),
        });
    }

    /// Get the source node for this context.
    #[must_use]
    pub fn source_node(&self) -> Option<usize> {
        self.stack.last().map(|f| f.node_id)
    }

    /// Returns a reference to the context stack frames (test-only).
    #[cfg(test)]
    #[must_use]
    pub fn frames(&self) -> &[Frame] {
        &self.stack
    }
}

/// Per-node interests, context, and identity.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// Declares the set of interests this node has (Acks, Nacks, ...)
    pub interests: Interests,
    /// The caller's data returns via AckMsg.context or Ack.context.
    pub calldata: CallData,
    /// The caller's node_id for routing.
    pub node_id: usize,
}

/// Context + container for telemetry data
#[derive(Clone, Debug)]
pub struct OtapPdata {
    context: Context,
    payload: OtapPayload,
}

/* -------- Signal type -------- */

impl OtapPdata {
    /// Construct new OtapData with payload using default context.
    /// This is a test-only form.
    #[must_use]
    #[cfg(test)]
    pub fn new_default(payload: OtapPayload) -> Self {
        Self {
            context: Context::default(),
            payload,
        }
    }

    /// New OtapData with payload using TODO(#1098) context. This is
    /// a definite problem. See issue #1098.
    #[must_use]
    pub fn new_todo_context(payload: OtapPayload) -> Self {
        Self {
            context: Context::default(),
            payload,
        }
    }

    /// Construct new OtapData with context and payload
    #[must_use]
    pub const fn new(context: Context, payload: OtapPayload) -> Self {
        Self { context, payload }
    }

    /// Returns the type of signal represented by this `OtapPdata` instance.
    #[must_use]
    pub fn signal_type(&self) -> SignalType {
        self.payload.signal_type()
    }

    /// Returns the format of signal represented by this `OtapPdata` instance.
    #[must_use]
    pub const fn signal_format(&self) -> SignalFormat {
        self.payload.signal_format()
    }

    /// True if the payload is empty. By definition, we can skip sending an
    /// empty request.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }

    /// Returns the payload from this request, consuming it.  This is
    /// only considered useful in testing.  Use into_parts() to split an
    /// OtapPdata into (Context, OtapPayload).
    #[must_use]
    #[cfg(test)]
    pub fn payload(self) -> OtapPayload {
        self.payload
    }

    /// Take the payload
    #[must_use]
    pub fn take_payload(&mut self) -> OtapPayload {
        self.payload.take_payload()
    }

    /// Borrow the payload.
    #[must_use]
    pub const fn payload_ref(&self) -> &OtapPayload {
        &self.payload
    }

    /// Splits the context and payload from this request, consuming it.
    #[must_use]
    pub fn into_parts(self) -> (Context, OtapPayload) {
        (self.context, self.payload)
    }

    /// Returns the number of items of the primary signal (spans, data
    /// points, log records).
    #[must_use]
    pub fn num_items(&self) -> usize {
        self.payload.num_items()
    }

    /// Enable testing Ack/Nack without an effect handler. Consumes,
    /// modifies and returns self.
    #[cfg(test)]
    #[must_use]
    pub fn test_subscribe_to(
        mut self,
        interests: Interests,
        calldata: CallData,
        node_id: usize,
    ) -> Self {
        self.context.subscribe_to(interests, calldata, node_id);
        self
    }

    /// Returns Context::has_subscribers()
    #[cfg(test)]
    #[must_use]
    pub fn has_subscribers(&self) -> bool {
        self.context.has_subscribers()
    }

    /// Return the source's calldata. Note that after a subscribe_to()
    /// has been called, the current node becomes the source.
    ///
    /// Return the current source calldata. This is used with the
    /// DelayedData message, in which a node delivers a message to
    /// itself.
    ///
    /// This is also useful in testing, it indicates the data that was
    /// sent by the source node.
    #[must_use]
    pub fn source_calldata(&self) -> Option<CallData> {
        self.context.source_calldata()
    }

    /// Update the source node. See also subscribe_to() which supports
    /// updating the source calldata.
    pub fn add_source_node(mut self, node_id: usize) -> Self {
        self.context.set_source_node(node_id);
        self
    }

    /// return the source node field
    #[must_use]
    pub fn get_source_node(&self) -> Option<usize> {
        self.context.source_node()
    }
}

/* -------- Producer effect handler extensions (shared, local) -------- */

#[async_trait(?Send)]
impl ProducerEffectHandlerExtension<OtapPdata>
    for otap_df_engine::local::processor::EffectHandler<OtapPdata>
{
    fn subscribe_to(&self, int: Interests, ctx: CallData, data: &mut OtapPdata) {
        data.context
            .subscribe_to(int, ctx, self.processor_id().index)
    }
}

#[async_trait(?Send)]
impl ProducerEffectHandlerExtension<OtapPdata>
    for otap_df_engine::local::receiver::EffectHandler<OtapPdata>
{
    fn subscribe_to(&self, int: Interests, ctx: CallData, data: &mut OtapPdata) {
        data.context
            .subscribe_to(int, ctx, self.receiver_id().index)
    }
}

#[async_trait(?Send)]
impl ProducerEffectHandlerExtension<OtapPdata>
    for otap_df_engine::shared::processor::EffectHandler<OtapPdata>
{
    fn subscribe_to(&self, int: Interests, ctx: CallData, data: &mut OtapPdata) {
        data.context
            .subscribe_to(int, ctx, self.processor_id().index)
    }
}

#[async_trait(?Send)]
impl ProducerEffectHandlerExtension<OtapPdata>
    for otap_df_engine::shared::receiver::EffectHandler<OtapPdata>
{
    fn subscribe_to(&self, int: Interests, ctx: CallData, data: &mut OtapPdata) {
        data.context
            .subscribe_to(int, ctx, self.receiver_id().index)
    }
}

/* -------- Consumer effect handler extensions (shared, local) -------- */

#[async_trait(?Send)]
impl ConsumerEffectHandlerExtension<OtapPdata>
    for otap_df_engine::local::processor::EffectHandler<OtapPdata>
{
    async fn notify_ack(&self, ack: AckMsg<OtapPdata>) -> Result<(), Error> {
        self.route_ack(ack, Context::next_ack).await
    }

    async fn notify_nack(&self, nack: NackMsg<OtapPdata>) -> Result<(), Error> {
        self.route_nack(nack, Context::next_nack).await
    }
}

#[async_trait(?Send)]
impl ConsumerEffectHandlerExtension<OtapPdata>
    for otap_df_engine::local::exporter::EffectHandler<OtapPdata>
{
    async fn notify_ack(&self, ack: AckMsg<OtapPdata>) -> Result<(), Error> {
        self.route_ack(ack, Context::next_ack).await
    }

    async fn notify_nack(&self, nack: NackMsg<OtapPdata>) -> Result<(), Error> {
        self.route_nack(nack, Context::next_nack).await
    }
}

#[async_trait(?Send)]
impl ConsumerEffectHandlerExtension<OtapPdata>
    for otap_df_engine::shared::processor::EffectHandler<OtapPdata>
{
    async fn notify_ack(&self, ack: AckMsg<OtapPdata>) -> Result<(), Error> {
        self.route_ack(ack, Context::next_ack).await
    }

    async fn notify_nack(&self, nack: NackMsg<OtapPdata>) -> Result<(), Error> {
        self.route_nack(nack, Context::next_nack).await
    }
}

#[async_trait(?Send)]
impl ConsumerEffectHandlerExtension<OtapPdata>
    for otap_df_engine::shared::exporter::EffectHandler<OtapPdata>
{
    async fn notify_ack(&self, ack: AckMsg<OtapPdata>) -> Result<(), Error> {
        self.route_ack(ack, Context::next_ack).await
    }

    async fn notify_nack(&self, nack: NackMsg<OtapPdata>) -> Result<(), Error> {
        self.route_nack(nack, Context::next_nack).await
    }
}

/* --------  effect handler extensions (shared, local) -------- */

#[async_trait(?Send)]
impl MessageSourceLocalEffectHandlerExtension<OtapPdata>
    for otap_df_engine::local::processor::EffectHandler<OtapPdata>
{
    async fn send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.processor_id().index)
        } else {
            data
        };
        self.send_message(data).await
    }

    fn try_send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.processor_id().index)
        } else {
            data
        };
        self.try_send_message(data)
    }

    async fn send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>>
    where
        P: Into<PortName> + Send + 'static,
    {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.processor_id().index)
        } else {
            data
        };
        self.send_message_to(port, data).await
    }

    fn try_send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>>
    where
        P: Into<PortName> + Send + 'static,
    {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.processor_id().index)
        } else {
            data
        };
        self.try_send_message_to(port, data)
    }
}

#[async_trait(?Send)]
impl MessageSourceLocalEffectHandlerExtension<OtapPdata>
    for otap_df_engine::local::receiver::EffectHandler<OtapPdata>
{
    async fn send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.receiver_id().index)
        } else {
            data
        };
        self.send_message(data).await
    }

    fn try_send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.receiver_id().index)
        } else {
            data
        };
        self.try_send_message(data)
    }

    async fn send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>>
    where
        P: Into<PortName> + Send,
    {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.receiver_id().index)
        } else {
            data
        };
        self.send_message_to(port, data).await
    }

    fn try_send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>>
    where
        P: Into<PortName> + Send,
    {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.receiver_id().index)
        } else {
            data
        };
        self.try_send_message_to(port, data)
    }
}

#[async_trait]
impl MessageSourceSharedEffectHandlerExtension<OtapPdata>
    for otap_df_engine::shared::processor::EffectHandler<OtapPdata>
{
    async fn send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.processor_id().index)
        } else {
            data
        };
        self.send_message(data).await
    }

    fn try_send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.processor_id().index)
        } else {
            data
        };
        self.try_send_message(data)
    }

    async fn send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>>
    where
        P: Into<PortName> + Send + 'static,
    {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.processor_id().index)
        } else {
            data
        };
        self.send_message_to(port, data).await
    }

    fn try_send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>>
    where
        P: Into<PortName> + Send + 'static,
    {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.processor_id().index)
        } else {
            data
        };
        self.try_send_message_to(port, data)
    }
}

#[async_trait]
impl MessageSourceSharedEffectHandlerExtension<OtapPdata>
    for otap_df_engine::shared::receiver::EffectHandler<OtapPdata>
{
    async fn send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.receiver_id().index)
        } else {
            data
        };
        self.send_message(data).await
    }

    fn try_send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.receiver_id().index)
        } else {
            data
        };
        self.try_send_message(data)
    }

    async fn send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>>
    where
        P: Into<PortName> + Send + 'static,
    {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.receiver_id().index)
        } else {
            data
        };
        self.send_message_to(port, data).await
    }

    fn try_send_message_with_source_node_to<P>(
        &self,
        port: P,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>>
    where
        P: Into<PortName> + Send + 'static,
    {
        let data = if self.needs_source_tag() {
            data.add_source_node(self.receiver_id().index)
        } else {
            data
        };
        self.try_send_message_to(port, data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::testing::{TestCallData, create_test_pdata};
    use otap_df_channel::mpsc::Channel as LocalChannel;
    use otap_df_engine::control::pipeline_ctrl_msg_channel;
    use otap_df_engine::local::message::LocalSender;
    use otap_df_engine::local::processor::EffectHandler as LocalProcessorEffectHandler;
    use otap_df_engine::local::receiver::EffectHandler as LocalReceiverEffectHandler;
    use otap_df_engine::message::Sender;
    use otap_df_engine::node::NodeId;
    use otap_df_engine::shared::message::SharedSender;
    use otap_df_engine::shared::processor::EffectHandler as SharedProcessorEffectHandler;
    use otap_df_engine::shared::receiver::EffectHandler as SharedReceiverEffectHandler;
    use otap_df_telemetry::reporter::MetricsReporter;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use tokio::sync::mpsc;

    fn create_test() -> (TestCallData, OtapPdata) {
        (TestCallData::default(), create_test_pdata())
    }

    #[tokio::test]
    async fn shared_receiver_send_with_source_node() {
        let (tx, mut rx) = mpsc::channel::<OtapPdata>(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), SharedSender::mpsc(tx));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = SharedReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_node".into(),
            },
            senders,
            Some("out".into()),
            ctrl_tx,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node(pdata)
            .await
            .expect("send ok");

        let sent = rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn shared_processor_send_with_source_node() {
        let (tx, mut rx) = mpsc::channel::<OtapPdata>(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), SharedSender::mpsc(tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = SharedProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_node".into(),
            },
            senders,
            Some("out".into()),
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node(pdata)
            .await
            .expect("send ok");

        let sent = rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn shared_processor_send_with_source_to_named_port() {
        let (a_tx, mut a_rx) = mpsc::channel::<OtapPdata>(4);
        let (b_tx, mut b_rx) = mpsc::channel::<OtapPdata>(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), SharedSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), SharedSender::mpsc(b_tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = SharedProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_node".into(),
            },
            senders,
            None,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node_to("b", pdata)
            .await
            .expect("send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn shared_receiver_send_with_source_to_named_port() {
        let (a_tx, mut a_rx) = mpsc::channel::<OtapPdata>(4);
        let (b_tx, mut b_rx) = mpsc::channel::<OtapPdata>(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), SharedSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), SharedSender::mpsc(b_tx));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = SharedReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_node".into(),
            },
            senders,
            None,
            ctrl_tx,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node_to("b", pdata)
            .await
            .expect("send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn shared_processor_try_send_with_source() {
        let (tx, mut rx) = mpsc::channel::<OtapPdata>(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), SharedSender::mpsc(tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = SharedProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_node".into(),
            },
            senders,
            Some("out".into()),
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node(pdata)
            .expect("try_send ok");

        let sent = rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn shared_processor_try_send_with_source_to_named_port() {
        let (a_tx, mut a_rx) = mpsc::channel::<OtapPdata>(1);
        let (b_tx, mut b_rx) = mpsc::channel::<OtapPdata>(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), SharedSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), SharedSender::mpsc(b_tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = SharedProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_node".into(),
            },
            senders,
            None,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node_to("b", pdata)
            .expect("try_send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn shared_receiver_try_send_with_source_to_named_port() {
        let (a_tx, mut a_rx) = mpsc::channel::<OtapPdata>(1);
        let (b_tx, mut b_rx) = mpsc::channel::<OtapPdata>(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), SharedSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), SharedSender::mpsc(b_tx));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = SharedReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_node".into(),
            },
            senders,
            None,
            ctrl_tx,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node_to("b", pdata)
            .expect("try_send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn local_processor_send_with_source_node() {
        let (tx, rx) = LocalChannel::<OtapPdata>::new(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx)));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = LocalProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_local".into(),
            },
            senders,
            Some("out".into()),
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node(pdata)
            .await
            .expect("send ok");

        let sent = rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn local_processor_send_with_source_to_named_port() {
        let (a_tx, a_rx) = LocalChannel::<OtapPdata>::new(4);
        let (b_tx, b_rx) = LocalChannel::<OtapPdata>::new(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), Sender::Local(LocalSender::mpsc(a_tx)));
        let _ = senders.insert("b".into(), Sender::Local(LocalSender::mpsc(b_tx)));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = LocalProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_local".into(),
            },
            senders,
            None,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node_to("b", pdata)
            .await
            .expect("send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn local_processor_try_send_with_source_node() {
        let (tx, rx) = LocalChannel::<OtapPdata>::new(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx)));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = LocalProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_local".into(),
            },
            senders,
            Some("out".into()),
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node(pdata)
            .expect("try_send ok");

        let sent = rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn local_processor_try_send_with_source_to_named_port() {
        let (a_tx, a_rx) = LocalChannel::<OtapPdata>::new(1);
        let (b_tx, b_rx) = LocalChannel::<OtapPdata>::new(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), Sender::Local(LocalSender::mpsc(a_tx)));
        let _ = senders.insert("b".into(), Sender::Local(LocalSender::mpsc(b_tx)));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = LocalProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_local".into(),
            },
            senders,
            None,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node_to("b", pdata)
            .expect("try_send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn local_receiver_try_send_with_source_to_named_port() {
        let (a_tx, a_rx) = LocalChannel::<OtapPdata>::new(1);
        let (b_tx, b_rx) = LocalChannel::<OtapPdata>::new(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), Sender::Local(LocalSender::mpsc(a_tx)));
        let _ = senders.insert("b".into(), Sender::Local(LocalSender::mpsc(b_tx)));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = LocalReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_local".into(),
            },
            senders,
            None,
            ctrl_tx,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node_to("b", pdata)
            .expect("try_send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn local_receiver_send_with_source_node() {
        let (tx, rx) = LocalChannel::<OtapPdata>::new(2);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx)));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = LocalReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_local".into(),
            },
            senders,
            Some("out".into()),
            ctrl_tx,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node(pdata)
            .await
            .expect("send ok");

        let sent = rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn local_receiver_send_with_source_to_named_port() {
        let (a_tx, a_rx) = LocalChannel::<OtapPdata>::new(2);
        let (b_tx, b_rx) = LocalChannel::<OtapPdata>::new(2);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), Sender::Local(LocalSender::mpsc(a_tx)));
        let _ = senders.insert("b".into(), Sender::Local(LocalSender::mpsc(b_tx)));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = LocalReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_local".into(),
            },
            senders,
            None,
            ctrl_tx,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node_to("b", pdata)
            .await
            .expect("send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn local_receiver_try_send_with_source_node() {
        let (tx, rx) = LocalChannel::<OtapPdata>::new(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx)));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler = LocalReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_local".into(),
            },
            senders,
            Some("out".into()),
            ctrl_tx,
            metrics_reporter,
        );
        handler.set_needs_source_tag(true);

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node(pdata)
            .expect("try_send ok");

        let sent = rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some(0));
    }

    #[tokio::test]
    async fn needs_source_tag_controls_source_frame() {
        let (tx_off, mut rx_off) = mpsc::channel::<OtapPdata>(4);
        let (tx_on, mut rx_on) = mpsc::channel::<OtapPdata>(4);

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler_off = SharedProcessorEffectHandler::new(
            NodeId {
                index: 7,
                name: "proc_off".into(),
            },
            HashMap::from([("out".into(), SharedSender::mpsc(tx_off))]),
            Some("out".into()),
            metrics_reporter,
        );

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut handler_on = SharedProcessorEffectHandler::new(
            NodeId {
                index: 7,
                name: "proc_on".into(),
            },
            HashMap::from([("out".into(), SharedSender::mpsc(tx_on))]),
            Some("out".into()),
            metrics_reporter,
        );

        // Default is false
        assert!(!handler_off.needs_source_tag());
        assert!(!handler_on.needs_source_tag());

        // Enable on one handler
        handler_on.set_needs_source_tag(true);
        assert!(handler_on.needs_source_tag());

        // Send through both
        handler_off
            .send_message_with_source_node(create_test_pdata())
            .await
            .expect("send ok");
        handler_on
            .send_message_with_source_node(create_test_pdata())
            .await
            .expect("send ok");

        // When disabled: no source frame pushed
        let sent_off = rx_off.recv().await.expect("received");
        assert_eq!(sent_off.get_source_node(), None);
        assert_eq!(sent_off.context.frames().len(), 0);

        // When enabled: source frame pushed with empty interests
        let sent_on = rx_on.recv().await.expect("received");
        assert_eq!(sent_on.get_source_node(), Some(7));
        let frames = sent_on.context.frames();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].node_id, 7);
        assert_eq!(frames[0].interests, Interests::empty());
    }

    #[test]
    fn test_context_next_ack_drops_payload_without_return_data() {
        let (test_data, pdata) = create_test();

        // Subscribe WITHOUT RETURN_DATA interest
        let pdata = pdata.test_subscribe_to(Interests::ACKS, test_data.clone().into(), 1234);

        assert_eq!(pdata.num_items(), 1);
        assert!(!pdata.is_empty());
        assert!(!pdata.context.may_return_payload());

        let ack = AckMsg::new(pdata);

        let result = Context::next_ack(ack);
        assert!(result.is_some());

        let (node_id, ack_msg) = result.unwrap();
        assert_eq!(node_id, 1234);
        let recv_data: TestCallData = ack_msg.calldata.try_into().expect("has");
        assert_eq!(recv_data, test_data);

        // Payload should be dropped
        assert_eq!(ack_msg.accepted.num_items(), 0);
        assert!(ack_msg.accepted.is_empty());
        assert_eq!(ack_msg.accepted.signal_type(), SignalType::Logs);
    }

    #[test]
    fn test_context_next_ack_preserves_payload_with_return_data() {
        let (test_data, pdata) = create_test();

        // Subscribe WITH RETURN_DATA interest
        let pdata = pdata.test_subscribe_to(
            Interests::ACKS | Interests::RETURN_DATA,
            test_data.clone().into(),
            1234,
        );

        assert_eq!(pdata.num_items(), 1);
        assert!(!pdata.is_empty());
        assert!(pdata.context.may_return_payload());

        let ack = AckMsg::new(pdata);

        let result = Context::next_ack(ack);
        assert!(result.is_some());

        let (node_id, ack_msg) = result.expect("has");
        assert_eq!(node_id, 1234);
        let recv_data: TestCallData = ack_msg.calldata.try_into().expect("has");
        assert_eq!(recv_data, test_data);

        // Payload should be preserved
        assert_eq!(ack_msg.accepted.num_items(), 1);
        assert!(!ack_msg.accepted.is_empty());
        assert_eq!(ack_msg.accepted.signal_type(), SignalType::Logs);
    }

    #[test]
    fn test_context_next_nack_drops_payload_without_return_data() {
        let (test_data, pdata) = create_test();

        // Subscribe WITHOUT RETURN_DATA interest
        let pdata = pdata.test_subscribe_to(Interests::NACKS, test_data.clone().into(), 1234);

        assert_eq!(pdata.num_items(), 1);
        assert!(!pdata.is_empty());
        assert!(!pdata.context.may_return_payload());

        let nack = NackMsg::new("test error".to_string(), pdata);
        let result = Context::next_nack(nack);
        assert!(result.is_some());

        let (node_id, nack_msg) = result.unwrap();
        assert_eq!(node_id, 1234);
        let recv_data: TestCallData = nack_msg.calldata.try_into().expect("has");
        assert_eq!(recv_data, test_data);

        // Payload should be dropped
        assert_eq!(nack_msg.refused.num_items(), 0);
        assert!(nack_msg.refused.is_empty());
        assert_eq!(nack_msg.refused.signal_type(), SignalType::Logs);
    }

    #[test]
    fn test_context_next_nack_preserves_payload_with_return_data() {
        let (test_data, pdata) = create_test();

        // Subscribe WITH RETURN_DATA interest
        let pdata = pdata.test_subscribe_to(
            Interests::NACKS | Interests::RETURN_DATA,
            test_data.clone().into(),
            1234,
        );

        assert_eq!(pdata.num_items(), 1);
        assert!(!pdata.is_empty());
        assert!(pdata.context.may_return_payload());

        let nack = NackMsg::new("test error", pdata);

        let result = Context::next_nack(nack);
        assert!(result.is_some());

        let (node_id, nack_msg) = result.unwrap();
        assert_eq!(node_id, 1234);
        let recv_data: TestCallData = nack_msg.calldata.try_into().expect("has");
        assert_eq!(recv_data, test_data);

        // Payload should be preserved
        assert_eq!(nack_msg.refused.num_items(), 1);
        assert!(!nack_msg.refused.is_empty());
        assert_eq!(nack_msg.refused.signal_type(), SignalType::Logs);
    }

    #[test]
    fn test_context_next_ack_nack() {
        let (test_data, pdata) = create_test();

        // Subscribe multiple frames. RETURN_DATA propagates automatically.
        let pdata = pdata
            .test_subscribe_to(
                Interests::NACKS | Interests::RETURN_DATA,
                test_data.clone().into(),
                1,
            )
            .test_subscribe_to(Interests::ACKS, CallData::default(), 2)
            .test_subscribe_to(Interests::NACKS, CallData::default(), 3)
            .test_subscribe_to(Interests::ACKS, CallData::default(), 4);
        assert!(pdata.context.may_return_payload());

        let ack = AckMsg::new(pdata);

        let result = Context::next_ack(ack);
        assert!(result.is_some());
        let (node_id, ack_msg) = result.unwrap();
        assert_eq!(node_id, 4);

        // Skipped node 3 (Nack) because call to next_ack()

        let result = Context::next_ack(ack_msg);
        assert!(result.is_some());
        let (node_id, ack_msg) = result.unwrap();
        assert_eq!(node_id, 2);

        // Payload should be preserved because node 1 has RETURN_DATA
        assert_eq!(ack_msg.accepted.num_items(), 1);
        assert!(!ack_msg.accepted.is_empty());

        let nack = NackMsg::new("nope nope", *ack_msg.accepted);

        // Node 1 last, is a Nack.
        let result = Context::next_nack(nack);
        assert!(result.is_some());
        let (node_id, nack_msg) = result.unwrap();
        assert_eq!(node_id, 1);
        let recv_data: TestCallData = nack_msg.calldata.try_into().expect("has");
        assert_eq!(recv_data, test_data);
    }

    #[test]
    fn test_context_no_ack() {
        let (_, pdata) = create_test();

        let ack = AckMsg::new(pdata);

        let result = Context::next_ack(ack);
        assert!(result.is_none());
    }

    #[test]
    fn test_context_no_nack() {
        let (_, pdata) = create_test();

        let nack = NackMsg::new("hey now", pdata);

        let result = Context::next_nack(nack);
        assert!(result.is_none());
    }

    // ---- set_source_node and subscribe_to frame-reuse tests ----

    #[test]
    fn test_set_source_node_basics() {
        let mut ctx = Context::default();
        assert_eq!(ctx.source_node(), None);

        // Push onto empty context.
        ctx.set_source_node(42);
        assert_eq!(ctx.source_node(), Some(42));
        assert_eq!(ctx.stack.len(), 1);
        // Source-node-only frames have empty interests — not subscribers.
        assert!(!ctx.has_subscribers());

        // Same node_id is a no-op (dedup).
        ctx.set_source_node(42);
        assert_eq!(ctx.stack.len(), 1);

        // Different node_id pushes a new frame.
        ctx.set_source_node(99);
        assert_eq!(ctx.source_node(), Some(99));
        assert_eq!(ctx.stack.len(), 2);
        assert!(!ctx.has_subscribers());
    }

    #[test]
    fn test_next_ack_nack_skip_source_node_frames() {
        let (test_data, pdata) = create_test();

        // Real subscriber, then a source-node-only frame on top.
        let pdata = pdata.test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            test_data.clone().into(),
            100,
        );
        let pdata = pdata.add_source_node(200);
        assert!(pdata.has_subscribers());

        // next_ack skips the empty-interests frame and finds node 100.
        let ack = AckMsg::new(pdata);
        let (node_id, ack_msg) = Context::next_ack(ack).expect("should find subscriber");
        assert_eq!(node_id, 100);
        let recv: TestCallData = ack_msg.calldata.try_into().expect("has");
        assert_eq!(recv, test_data);
    }

    #[test]
    fn test_source_node_frame_propagates_return_data() {
        // Scenario: a retry processor (node 1) subscribes with ACKS | RETURN_DATA,
        // then a multi-source source-tag frame is pushed (node 2), then a
        // downstream processor (node 3) subscribes with ACKS only.
        //
        // Without RETURN_DATA propagation in set_source_node, the source frame
        // breaks the chain and node 3's frame won't inherit RETURN_DATA.
        // When next_ack finds node 3, it sees no RETURN_DATA and drops the
        // payload — even though node 1 needs it for retry.
        let (test_data, pdata) = create_test();

        let pdata = pdata
            // Node 1: retry processor wants payload back
            .test_subscribe_to(
                Interests::ACKS | Interests::RETURN_DATA,
                test_data.clone().into(),
                1,
            )
            // Node 2: source-tag from a multi-source edge
            .add_source_node(2);

        // Verify RETURN_DATA survived through the source frame.
        assert!(
            pdata.context.may_return_payload(),
            "source-tag frame must propagate RETURN_DATA from preceding subscriber"
        );

        // Node 3: downstream processor subscribes with ACKS (no explicit RETURN_DATA)
        let pdata = pdata.test_subscribe_to(Interests::ACKS, CallData::default(), 3);

        // Node 3's frame should have inherited RETURN_DATA through the source frame.
        assert!(
            pdata.context.may_return_payload(),
            "subscribe_to must inherit RETURN_DATA through the source-tag frame"
        );

        // Ack path: next_ack finds node 3 first
        let ack = AckMsg::new(pdata);
        let (node_id, ack_msg) = Context::next_ack(ack).expect("should find node 3");
        assert_eq!(node_id, 3);

        // The payload must be preserved — node 1 needs it for retry.
        assert_eq!(
            ack_msg.accepted.num_items(),
            1,
            "payload must be preserved because an earlier subscriber requested RETURN_DATA"
        );
        assert!(!ack_msg.accepted.is_empty());

        // Continue to node 1
        let (node_id, ack_msg) = Context::next_ack(ack_msg).expect("should find node 1");
        assert_eq!(node_id, 1);
        let recv: TestCallData = ack_msg.calldata.try_into().expect("has");
        assert_eq!(recv, test_data);

        // Payload still intact for the retry processor
        assert_eq!(ack_msg.accepted.num_items(), 1);
    }
}
