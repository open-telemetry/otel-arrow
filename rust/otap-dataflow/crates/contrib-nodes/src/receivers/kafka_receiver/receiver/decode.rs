// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Payload decode and the CallData codec.
//!
//! Packs and unpacks Kafka message identity (topic id, partition, offset,
//! delivery generation) into [`CallData`] for Ack/Nack routing, and decodes
//! OTLP-proto / OTAP-proto / Syslog payloads (optionally applying header
//! extractions) into [`OtapPdata`].

use super::super::config::HeaderExtraction;
use super::super::headers::{HeaderExtractions, decode_syslog_logs};
use super::super::identity::DeliveryGeneration;
use crate::common::kafka::MessageFormat;
use bytes::Bytes;
use otel_arrow_dfe_engine::control::{CallData, Context8u8};
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::Consumer as PdataConsumer;
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::otap::{OtapArrowRecords, from_record_messages};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords;
use prost::Message;
use rdkafka::message::BorrowedMessage;
use smallvec::smallvec;
use std::collections::HashMap;

/// Encode Kafka message identity into [`CallData`] for Ack/Nack routing.
///
/// Slot 0: `(topic_id << 32) | (partition as u32)` packed into a `u64`.
/// Slot 1: `offset` cast to `u64`.
/// Slot 2: receiver-local delivery `generation`.
///
/// [`CallData`] inlines three slots, so carrying the generation adds no
/// heap allocation.
pub(super) fn encode_calldata(
    topic_id: u32,
    partition: i32,
    offset: i64,
    delivery_generation: DeliveryGeneration,
) -> CallData {
    let topic_partition = ((topic_id as u64) << 32) | (partition as u32 as u64);
    smallvec![
        Context8u8::from(topic_partition),
        Context8u8::from(offset as u64),
        Context8u8::from(delivery_generation.raw()),
    ]
}

/// Decode Kafka message identity from [`CallData`] returned in Ack/Nack.
///
/// A calldata without the delivery-generation slot (legacy 2-slot form)
/// decodes as generation `0`.
pub(super) fn decode_calldata(calldata: &CallData) -> (u32, i32, i64, DeliveryGeneration) {
    let topic_partition: u64 = calldata[0].into();
    let topic_id = (topic_partition >> 32) as u32;
    let partition = (topic_partition & 0xFFFF_FFFF) as i32;
    let offset: u64 = calldata[1].into();
    let delivery_generation: u64 = calldata.get(2).copied().map(Into::into).unwrap_or(0);
    (
        topic_id,
        partition,
        offset as i64,
        DeliveryGeneration::from_raw(delivery_generation),
    )
}

/// Decode a traces payload into `OtapPdata`.
pub(super) fn decode_traces_payload(
    data: &[u8],
    message_format: MessageFormat,
) -> Result<OtapPdata, EngineError> {
    match message_format {
        MessageFormat::OtlpProto => Ok(OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportTracesRequest(Bytes::copy_from_slice(data)).into(),
        )),
        MessageFormat::OtapProto => {
            let mut bar =
                BatchArrowRecords::decode(data).map_err(|e| EngineError::PdataConversionError {
                    error: e.to_string(),
                })?;
            let mut pdc = PdataConsumer::default();
            let record_messages = pdc.consume_bar(&mut bar)?;
            Ok(OtapPdata::new(
                Context::default(),
                OtapArrowRecords::Traces(from_record_messages(record_messages).map_err(|e| {
                    EngineError::PdataConversionError {
                        error: e.to_string(),
                    }
                })?)
                .into(),
            ))
        }
        MessageFormat::Syslog => Err(EngineError::PdataConversionError {
            error: "syslog encoding is only supported for logs".to_string(),
        }),
    }
}

/// Decode a metrics payload into `OtapPdata`.
pub(super) fn decode_metrics_payload(
    data: &[u8],
    message_format: MessageFormat,
) -> Result<OtapPdata, EngineError> {
    match message_format {
        MessageFormat::OtlpProto => Ok(OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportMetricsRequest(Bytes::copy_from_slice(data)).into(),
        )),
        MessageFormat::OtapProto => {
            let mut bar =
                BatchArrowRecords::decode(data).map_err(|e| EngineError::PdataConversionError {
                    error: e.to_string(),
                })?;
            let mut pdc = PdataConsumer::default();
            let record_messages = pdc.consume_bar(&mut bar)?;
            Ok(OtapPdata::new(
                Context::default(),
                OtapArrowRecords::Metrics(from_record_messages(record_messages).map_err(|e| {
                    EngineError::PdataConversionError {
                        error: e.to_string(),
                    }
                })?)
                .into(),
            ))
        }
        MessageFormat::Syslog => Err(EngineError::PdataConversionError {
            error: "syslog encoding is only supported for logs".to_string(),
        }),
    }
}

/// Decode a logs payload into `OtapPdata`.
pub(super) fn decode_logs_payload(
    data: &[u8],
    message_format: MessageFormat,
) -> Result<OtapPdata, EngineError> {
    match message_format {
        MessageFormat::OtlpProto => Ok(OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportLogsRequest(Bytes::copy_from_slice(data)).into(),
        )),
        MessageFormat::OtapProto => {
            let mut bar =
                BatchArrowRecords::decode(data).map_err(|e| EngineError::PdataConversionError {
                    error: e.to_string(),
                })?;
            let mut pdc = PdataConsumer::default();
            let record_messages = pdc.consume_bar(&mut bar)?;
            Ok(OtapPdata::new(
                Context::default(),
                OtapArrowRecords::Logs(from_record_messages(record_messages).map_err(|e| {
                    EngineError::PdataConversionError {
                        error: e.to_string(),
                    }
                })?)
                .into(),
            ))
        }
        MessageFormat::Syslog => Ok(OtapPdata::new(
            Context::default(),
            decode_syslog_logs(data)?.into(),
        )),
    }
}

/// Reject Syslog decoding for signal types other than logs.
pub(super) fn reject_syslog_for_non_log_signal(
    _extractions: &HeaderExtractions,
    _data: &[u8],
) -> Result<OtapPdata, EngineError> {
    Err(EngineError::PdataConversionError {
        error: "syslog encoding is only supported for logs".to_string(),
    })
}

/// Decode a Kafka payload with optional header extraction applied to resource
/// attributes.
///
/// When `extractors` is non-empty the Kafka message headers are scanned once
/// and, if any configured header is found, the matching `apply_*` function is
/// used to decode **and** inject the attributes in a single pass. When no
/// extractors are configured (or none matched) the plain `decode` function is
/// used instead.
pub(super) fn decode_with_extractions(
    kafka_message: &BorrowedMessage<'_>,
    extractors: &HashMap<String, HeaderExtraction>,
    data: &[u8],
    message_format: MessageFormat,
    apply_otlp: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError>,
    apply_otap: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError>,
    apply_syslog: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError>,
    decode: fn(&[u8], MessageFormat) -> Result<OtapPdata, EngineError>,
) -> Result<OtapPdata, EngineError> {
    if !extractors.is_empty() {
        let extractions = match message_format {
            MessageFormat::OtlpProto => HeaderExtractions::otlp(kafka_message, extractors),
            MessageFormat::OtapProto => HeaderExtractions::otap(kafka_message, extractors),
            MessageFormat::Syslog => HeaderExtractions::otap(kafka_message, extractors),
        };
        if extractions.has_any() {
            return match message_format {
                MessageFormat::OtlpProto => apply_otlp(&extractions, data),
                MessageFormat::OtapProto => apply_otap(&extractions, data),
                MessageFormat::Syslog => apply_syslog(&extractions, data),
            };
        }
    }
    decode(data, message_format)
}
