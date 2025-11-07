// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the pipeline data that is passed between pipeline components.
//!
//! Internally, the data can be represented in the following formats:
//! - OTLP Bytes - contain the OTLP service request messages serialized as protobuf
//! - OTAP Arrow Bytes - the data is contained in `BatchArrowRecords` type which
//!   contains the Arrow batches for each payload, serialized as Arrow IPC. This type is
//!   what we'd receive from the OTAP GRPC service.
//! - OTAP Arrow Records - the data is contained in Arrow `[RecordBatch]`s organized by
//!   for efficient access by the arrow payload type.
//!
//! This module also contains conversions between the various types using the `From`
//! and `TryFrom` traits. For example:
//! ```
//! # use std::sync::Arc;
//! # use arrow::array::{RecordBatch, UInt16Array};
//! # use arrow::datatypes::{DataType, Field, Schema};
//! # use otap_df_pdata::otap::{OtapArrowRecords, Logs};
//! # use otap_df_pdata::proto::opentelemetry::{
//!     arrow::v1::ArrowPayloadType,
//!     collector::logs::v1::ExportLogsServiceRequest,
//!     common::v1::{AnyValue, InstrumentationScope, KeyValue},
//!     logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
//!     resource::v1::Resource
//! };
//! # use otap_df_otap::pdata::{Context, OtapPdata};
//! # use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
//! # use prost::Message;
//! let otlp_service_req = ExportLogsServiceRequest::new(vec![
//!    ResourceLogs::new(
//!        Resource::default(),
//!        vec![
//!            ScopeLogs::new(
//!                InstrumentationScope::default(),
//!                vec![
//!                    LogRecord::build()
//!                        .time_unix_nano(2u64)
//!                        .severity_number(SeverityNumber::Info)
//!                        .event_name("event")
//!                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
//!                        .finish(),
//!                ],
//!            ),
//!        ],
//!    ),
//!  ]);
//! let mut buf = Vec::new();
//! otlp_service_req.encode(&mut buf).unwrap();
//!
//! // Create a new OtapPdata with default context
//! let context = Context::default();
//! let mut pdata = OtapPdata::new(context, OtlpProtoBytes::ExportLogsRequest(buf).into());
//!
//! // Split the request, convert to Otap Arrow Records
//! let (context, payload) = pdata.into_parts();
//! let otap_arrow_records: OtapArrowRecords = payload.try_into().unwrap();
//! ```
//!
//! Internally, conversions are happening using various utility functions:
//! ```text
//!                                      ┌───────────────────────┐
//!                                      │                       │
//!                                      │      OTLP Bytes       │
//!                                      │                       │
//!                                      └───┬───────────────────┘
//!                                          │                 ▲
//!                                          │                 │
//!                                          │                 │
//!                                          ▼                 │
//!    otap_df_otap::encoder::encode_<signal>_otap_batch    otap_df_pdata::otlp::<signal>::<signal_>_from()
//!                                          │                 ▲
//!                                          │                 │
//!                                          │                 │
//!                                          ▼                 │
//!                                      ┌─────────────────────┴───┐
//!                                      │                         │
//!                                      │    OTAP Arrow Records   │
//!                                      │                         │
//!                                      └─────────────────────────┘
//! ```
// ^^ TODO we're currently in the process of reworking conversion between OTLP & OTAP to go
// directly from OTAP -> OTLP bytes. The utility functions we use might change as part of
// this diagram may need to be updated (https://github.com/open-telemetry/otel-arrow/issues/1095)

use async_trait::async_trait;
use otap_df_config::SignalType;
use otap_df_engine::error::Error;
use otap_df_engine::{
    ConsumerEffectHandlerExtension, Interests, ProducerEffectHandlerExtension,
    control::{AckMsg, CallData, NackMsg},
};
use otap_df_pdata::OtapPayload;

/// Context for OTAP requests
#[derive(Clone, Debug, Default)]
pub struct Context {
    stack: Vec<Frame>,
}

impl Context {
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
}

/// Per-node interests, context, and identity.
#[derive(Clone, Debug)]
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

    /// Return the current calldata.
    #[must_use]
    pub fn current_calldata(&self) -> Option<CallData> {
        self.context.current_calldata()
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::{TestCallData, create_test_pdata};

    fn create_test() -> (TestCallData, OtapPdata) {
        (TestCallData::default(), create_test_pdata())
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
