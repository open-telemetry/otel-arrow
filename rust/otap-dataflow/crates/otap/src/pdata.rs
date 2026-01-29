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
pub use otap_df_config::NodeId;
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
    source_node: Option<NodeId>,
    stack: Vec<Frame>,
}

impl Context {
    /// Create a context with reserved frame capacity to avoid reallocating
    /// when the first subscriber is pushed.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            source_node: None,
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
        if let Some(last) = self.stack.last() {
            // Inherit the preceding frame's RETURN_DATA bit
            interests |= last.interests & Interests::RETURN_DATA;
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

    /// Return the current calldata.
    #[must_use]
    pub fn current_calldata(&self) -> Option<CallData> {
        self.stack.last().map(|f| f.calldata.clone())
    }

    /// Are there any subscribers?
    #[must_use]
    pub fn has_subscribers(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Set the source node for this context.
    pub fn set_source_node(&mut self, node: Option<NodeId>) {
        self.source_node = node;
    }

    /// Get the source node for this context.
    pub fn source_node(&self) -> Option<NodeId> {
        self.source_node.clone()
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
    pub fn new(context: Context, payload: OtapPayload) -> Self {
        Self { context, payload }
    }

    /// Returns the type of signal represented by this `OtapPdata` instance.
    #[must_use]
    pub fn signal_type(&self) -> SignalType {
        self.payload.signal_type()
    }

    /// Returns the format of signal represented by this `OtapPdata` instance.
    #[must_use]
    pub fn signal_format(&self) -> SignalFormat {
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
    pub fn payload_ref(&self) -> &OtapPayload {
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

    /// Return the current calldata.
    #[must_use]
    pub fn current_calldata(&self) -> Option<CallData> {
        self.context.current_calldata()
    }

    /// update the source node
    pub fn add_source_node(mut self, node_id: Option<NodeId>) -> Self {
        self.context.set_source_node(node_id);
        self
    }

    /// return the source node field
    pub fn get_source_node(&self) -> Option<NodeId> {
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
        let data = data.add_source_node(Some(self.processor_id().name));
        self.send_message(data).await
    }

    fn try_send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = data.add_source_node(Some(self.processor_id().name));
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
        let data = data.add_source_node(Some(self.processor_id().name));
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
        let data = data.add_source_node(Some(self.processor_id().name));
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
        let data = data.add_source_node(Some(self.receiver_id().name));
        self.send_message(data).await
    }

    fn try_send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = data.add_source_node(Some(self.receiver_id().name));
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
        let data = data.add_source_node(Some(self.receiver_id().name));
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
        let data = data.add_source_node(Some(self.receiver_id().name));
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
        let data = data.add_source_node(Some(self.processor_id().name));
        self.send_message(data).await
    }

    fn try_send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = data.add_source_node(Some(self.processor_id().name));
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
        let data = data.add_source_node(Some(self.processor_id().name));
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
        let data = data.add_source_node(Some(self.processor_id().name));
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
        let data = data.add_source_node(Some(self.receiver_id().name));
        self.send_message(data).await
    }

    fn try_send_message_with_source_node(
        &self,
        data: OtapPdata,
    ) -> Result<(), TypedError<OtapPdata>> {
        let data = data.add_source_node(Some(self.receiver_id().name));
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
        let data = data.add_source_node(Some(self.receiver_id().name));
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
        let data = data.add_source_node(Some(self.receiver_id().name));
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
        let handler = SharedReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_node".into(),
            },
            senders,
            Some("out".into()),
            ctrl_tx,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node(pdata)
            .await
            .expect("send ok");

        let sent = rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some("recv_node".into()));
    }

    #[tokio::test]
    async fn shared_processor_send_with_source_node() {
        let (tx, mut rx) = mpsc::channel::<OtapPdata>(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), SharedSender::mpsc(tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = SharedProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_node".into(),
            },
            senders,
            Some("out".into()),
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node(pdata)
            .await
            .expect("send ok");

        let sent = rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some("proc_node".into()));
    }

    #[tokio::test]
    async fn shared_processor_send_with_source_to_named_port() {
        let (a_tx, mut a_rx) = mpsc::channel::<OtapPdata>(4);
        let (b_tx, mut b_rx) = mpsc::channel::<OtapPdata>(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), SharedSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), SharedSender::mpsc(b_tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = SharedProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_node".into(),
            },
            senders,
            None,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node_to("b", pdata)
            .await
            .expect("send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some("proc_node".into()));
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
        let handler = SharedReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_node".into(),
            },
            senders,
            None,
            ctrl_tx,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node_to("b", pdata)
            .await
            .expect("send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some("recv_node".into()));
    }

    #[tokio::test]
    async fn shared_processor_try_send_with_source() {
        let (tx, mut rx) = mpsc::channel::<OtapPdata>(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), SharedSender::mpsc(tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = SharedProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_node".into(),
            },
            senders,
            Some("out".into()),
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node(pdata)
            .expect("try_send ok");

        let sent = rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some("proc_node".into()));
    }

    #[tokio::test]
    async fn shared_processor_try_send_with_source_to_named_port() {
        let (a_tx, mut a_rx) = mpsc::channel::<OtapPdata>(1);
        let (b_tx, mut b_rx) = mpsc::channel::<OtapPdata>(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), SharedSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), SharedSender::mpsc(b_tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = SharedProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_node".into(),
            },
            senders,
            None,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node_to("b", pdata)
            .expect("try_send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some("proc_node".into()));
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
        let handler = SharedReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_node".into(),
            },
            senders,
            None,
            ctrl_tx,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node_to("b", pdata)
            .expect("try_send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some("recv_node".into()));
    }

    #[tokio::test]
    async fn local_processor_send_with_source_node() {
        let (tx, rx) = LocalChannel::new(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), LocalSender::mpsc(tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = LocalProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_local".into(),
            },
            senders,
            Some("out".into()),
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node(pdata)
            .await
            .expect("send ok");

        let sent = rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some("proc_local".into()));
    }

    #[tokio::test]
    async fn local_processor_send_with_source_to_named_port() {
        let (a_tx, a_rx) = LocalChannel::new(4);
        let (b_tx, b_rx) = LocalChannel::new(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), LocalSender::mpsc(b_tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = LocalProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_local".into(),
            },
            senders,
            None,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node_to("b", pdata)
            .await
            .expect("send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some("proc_local".into()));
    }

    #[tokio::test]
    async fn local_processor_try_send_with_source_node() {
        let (tx, rx) = LocalChannel::new(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), LocalSender::mpsc(tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = LocalProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_local".into(),
            },
            senders,
            Some("out".into()),
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node(pdata)
            .expect("try_send ok");

        let sent = rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some("proc_local".into()));
    }

    #[tokio::test]
    async fn local_processor_try_send_with_source_to_named_port() {
        let (a_tx, a_rx) = LocalChannel::new(1);
        let (b_tx, b_rx) = LocalChannel::new(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), LocalSender::mpsc(b_tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = LocalProcessorEffectHandler::new(
            NodeId {
                index: 0,
                name: "proc_local".into(),
            },
            senders,
            None,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node_to("b", pdata)
            .expect("try_send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some("proc_local".into()));
    }

    #[tokio::test]
    async fn local_receiver_try_send_with_source_to_named_port() {
        let (a_tx, a_rx) = LocalChannel::new(1);
        let (b_tx, b_rx) = LocalChannel::new(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), LocalSender::mpsc(b_tx));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = LocalReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_local".into(),
            },
            senders,
            None,
            ctrl_tx,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node_to("b", pdata)
            .expect("try_send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some("recv_local".into()));
    }

    #[tokio::test]
    async fn local_receiver_send_with_source_node() {
        let (tx, rx) = LocalChannel::new(2);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), LocalSender::mpsc(tx));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = LocalReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_local".into(),
            },
            senders,
            Some("out".into()),
            ctrl_tx,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node(pdata)
            .await
            .expect("send ok");

        let sent = rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some("recv_local".into()));
    }

    #[tokio::test]
    async fn local_receiver_send_with_source_to_named_port() {
        let (a_tx, a_rx) = LocalChannel::new(2);
        let (b_tx, b_rx) = LocalChannel::new(2);
        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), LocalSender::mpsc(b_tx));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = LocalReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_local".into(),
            },
            senders,
            None,
            ctrl_tx,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .send_message_with_source_node_to("b", pdata)
            .await
            .expect("send ok");

        assert!(a_rx.try_recv().is_err());
        let sent = b_rx.recv().await.expect("message received");
        assert_eq!(sent.get_source_node(), Some("recv_local".into()));
    }

    #[tokio::test]
    async fn local_receiver_try_send_with_source_node() {
        let (tx, rx) = LocalChannel::new(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), LocalSender::mpsc(tx));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let handler = LocalReceiverEffectHandler::new(
            NodeId {
                index: 0,
                name: "recv_local".into(),
            },
            senders,
            Some("out".into()),
            ctrl_tx,
            metrics_reporter,
        );

        let pdata = create_test_pdata();
        handler
            .try_send_message_with_source_node(pdata)
            .expect("try_send ok");

        let sent = rx.try_recv().expect("message received");
        assert_eq!(sent.get_source_node(), Some("recv_local".into()));
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
}
