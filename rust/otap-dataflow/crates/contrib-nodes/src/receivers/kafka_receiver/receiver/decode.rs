// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Payload decode and the CallData codec.
//!
//! Packs and unpacks Kafka message identity (topic id, partition, offset,
//! delivery generation) into [`CallData`] for Ack/Nack routing, and decodes
//! OTLP-proto / OTAP-proto / Syslog payloads (optionally applying header
//! extractions) into [`OtapPdata`].
//!
//! [`SignalDecoder`] maps a [`SignalType`] onto the configured encoding,
//! the OTLP/OTAP/Syslog payload decoders, and the header-extraction dispatch.

use super::super::config::HeaderExtraction;
use super::super::headers::HeaderExtractions;
use super::super::identity::DeliveryGeneration;
use crate::common::kafka::MessageFormat;
use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_core_nodes::receivers::syslog_cef_receiver::{
    MAX_MESSAGE_SIZE as MAX_SYSLOG_MESSAGE_SIZE,
    arrow_records_encoder::ArrowRecordsBuilder as SyslogArrowRecordsBuilder,
    parser::parse as parse_syslog,
};
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

/// The canonical error for a Syslog payload routed to a non-logs signal.
///
/// Defined once so the plain decode path and the extraction path share the
/// exact same "logs only" message.
fn syslog_logs_only_error() -> EngineError {
    EngineError::PdataConversionError {
        error: "syslog encoding is only supported for logs".to_string(),
    }
}

/// Reject Syslog decoding for signal types other than logs.
///
/// Shaped as an `apply_*` header-extraction function pointer so the extraction
/// dispatch can select it for traces/metrics.
pub(super) fn reject_syslog_for_non_log_signal(
    _extractions: &HeaderExtractions,
    _data: &[u8],
) -> Result<OtapPdata, EngineError> {
    Err(syslog_logs_only_error())
}

/// Stateless, signal-keyed decoder.
///
/// A single `SignalDecoder` serves every [`SignalType`]; the `signal` is passed
/// to each associated function rather than owned. This groups all per-signal
/// decode behavior (encoding selection, telemetry label, OTLP/OTAP/Syslog
/// decode, and header-extraction dispatch) so message formats, the Syslog
/// restriction, and error mapping have one source of truth.
pub(crate) struct SignalDecoder;

impl SignalDecoder {
    /// The stable telemetry label for `signal` (`"traces"` / `"metrics"` /
    /// `"logs"`). Used for both metric attributes and structured error events.
    #[must_use]
    pub(crate) fn label(signal: SignalType) -> &'static str {
        match signal {
            SignalType::Traces => "traces",
            SignalType::Metrics => "metrics",
            SignalType::Logs => "logs",
        }
    }

    /// Wrap OTLP-proto bytes in the [`OtlpProtoBytes`] request variant for
    /// `signal`.
    fn otlp_pdata(signal: SignalType, data: &[u8]) -> OtapPdata {
        let bytes = Bytes::copy_from_slice(data);
        let otlp = match signal {
            SignalType::Traces => OtlpProtoBytes::ExportTracesRequest(bytes),
            SignalType::Metrics => OtlpProtoBytes::ExportMetricsRequest(bytes),
            SignalType::Logs => OtlpProtoBytes::ExportLogsRequest(bytes),
        };
        OtapPdata::new(Context::default(), otlp.into())
    }

    /// Decode OTAP Arrow bytes into the [`OtapArrowRecords`] variant for
    /// `signal`.
    ///
    /// The Arrow-batch decode and consume steps are shared across signals; only
    /// the final, statically-typed `from_record_messages::<T>` conversion (and
    /// the variant it is wrapped in) depends on the signal, because each
    /// [`OtapArrowRecords`] variant carries a distinct store type.
    pub(crate) fn decode_otap(
        signal: SignalType,
        data: &[u8],
    ) -> Result<OtapArrowRecords, EngineError> {
        let mut bar =
            BatchArrowRecords::decode(data).map_err(|e| EngineError::PdataConversionError {
                error: e.to_string(),
            })?;
        let mut pdc = PdataConsumer::default();
        let record_messages = pdc.consume_bar(&mut bar)?;
        let to_conversion_error =
            |e: otel_arrow_dfe_pdata::error::Error| EngineError::PdataConversionError {
                error: e.to_string(),
            };
        Ok(match signal {
            SignalType::Traces => OtapArrowRecords::Traces(
                from_record_messages(record_messages).map_err(to_conversion_error)?,
            ),
            SignalType::Metrics => OtapArrowRecords::Metrics(
                from_record_messages(record_messages).map_err(to_conversion_error)?,
            ),
            SignalType::Logs => OtapArrowRecords::Logs(
                from_record_messages(record_messages).map_err(to_conversion_error)?,
            ),
        })
    }

    /// Parse one complete Syslog message and encode it as OTAP Arrow logs.
    ///
    /// Signal-agnostic (logs-only): Syslog is never valid for traces or metrics,
    /// so this takes no `signal`. The logs-only restriction is enforced by the
    /// callers ([`SignalDecoder::decode_signal_payload`] rejects Syslog for
    /// traces/metrics; the header-extraction path only calls this for logs).
    pub(crate) fn decode_syslog_logs(data: &[u8]) -> Result<OtapArrowRecords, EngineError> {
        if data.len() > MAX_SYSLOG_MESSAGE_SIZE {
            return Err(EngineError::PdataConversionError {
                error: format!(
                    "Syslog/CEF payload size {} exceeds the maximum of {} bytes",
                    data.len(),
                    MAX_SYSLOG_MESSAGE_SIZE,
                ),
            });
        }

        let parsed = parse_syslog(data).map_err(|e| EngineError::PdataConversionError {
            error: format!("Failed to parse Syslog payload: {e:?}"),
        })?;
        let mut builder = SyslogArrowRecordsBuilder::new();
        builder.append_syslog(parsed);
        builder
            .build()
            .map_err(|e| EngineError::PdataConversionError {
                error: format!("Failed to encode Syslog payload as Arrow records: {e}"),
            })
    }

    /// Decode a Kafka payload for `signal` into [`OtapPdata`] without header
    /// extraction.
    ///
    /// This is the one plain decode path. The OTLP and OTAP arms differ only by
    /// the request/record variant selected for `signal`; the Syslog arm is
    /// logs-only (traces/metrics reject with the canonical error), so the "logs
    /// only" restriction is defined once.
    pub(crate) fn decode_signal_payload(
        signal: SignalType,
        data: &[u8],
        message_format: MessageFormat,
    ) -> Result<OtapPdata, EngineError> {
        match message_format {
            MessageFormat::OtlpProto => Ok(Self::otlp_pdata(signal, data)),
            MessageFormat::OtapProto => {
                let records = Self::decode_otap(signal, data)?;
                Ok(OtapPdata::new(Context::default(), records.into()))
            }
            MessageFormat::Syslog => match signal {
                SignalType::Logs => Ok(OtapPdata::new(
                    Context::default(),
                    Self::decode_syslog_logs(data)?.into(),
                )),
                SignalType::Traces | SignalType::Metrics => Err(syslog_logs_only_error()),
            },
        }
    }

    /// Decode a Kafka payload for `signal`, applying header extractions when any
    /// configured header is present.
    ///
    /// Binds the signal-specific `HeaderExtractions::apply_*` methods (and the
    /// Syslog policy) for `signal`, then defers to the shared
    /// [`decode_with_extractions`] machinery. When no extractor matches it falls
    /// back to [`SignalDecoder::decode_signal_payload`].
    pub(crate) fn decode_signal_with_extractions(
        signal: SignalType,
        kafka_message: &BorrowedMessage<'_>,
        extractors: &HashMap<String, HeaderExtraction>,
        data: &[u8],
        message_format: MessageFormat,
    ) -> Result<OtapPdata, EngineError> {
        let apply_otlp: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError> =
            match signal {
                SignalType::Traces => HeaderExtractions::apply_otlp_traces,
                SignalType::Metrics => HeaderExtractions::apply_otlp_metrics,
                SignalType::Logs => HeaderExtractions::apply_otlp_logs,
            };
        let apply_otap: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError> =
            match signal {
                SignalType::Traces => HeaderExtractions::apply_otap_traces,
                SignalType::Metrics => HeaderExtractions::apply_otap_metrics,
                SignalType::Logs => HeaderExtractions::apply_otap_logs,
            };
        // Syslog is logs-only: logs inject into the parsed record, traces and
        // metrics reject with the canonical decode error.
        let apply_syslog: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError> =
            match signal {
                SignalType::Logs => HeaderExtractions::apply_syslog_logs,
                SignalType::Traces | SignalType::Metrics => reject_syslog_for_non_log_signal,
            };

        decode_with_extractions(
            kafka_message,
            extractors,
            data,
            message_format,
            apply_otlp,
            apply_otap,
            apply_syslog,
            move |data, message_format| Self::decode_signal_payload(signal, data, message_format),
        )
    }
}

/// Decode a Kafka payload with optional header extraction applied to resource
/// attributes.
///
/// When `extractors` is non-empty the Kafka message headers are scanned once
/// and, if any configured header is found, the matching `apply_*` function is
/// used to decode **and** inject the attributes in a single pass. When no
/// extractors are configured (or none matched) the plain `decode` fallback is
/// used instead.
pub(super) fn decode_with_extractions<F>(
    kafka_message: &BorrowedMessage<'_>,
    extractors: &HashMap<String, HeaderExtraction>,
    data: &[u8],
    message_format: MessageFormat,
    apply_otlp: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError>,
    apply_otap: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError>,
    apply_syslog: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError>,
    decode: F,
) -> Result<OtapPdata, EngineError>
where
    F: FnOnce(&[u8], MessageFormat) -> Result<OtapPdata, EngineError>,
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_pdata::{OtlpProtoBytes, TryIntoWithOptions};
    use prost::Message;

    /// Build a syslog payload of exactly `size` bytes with a valid RFC 5424
    /// header prefix, padding the remainder.
    fn syslog_payload_with_size(size: usize) -> Vec<u8> {
        let header = b"<34>1 2024-01-15T10:30:45.123Z host app - ID47 - ";
        assert!(size >= header.len());
        let mut payload = Vec::with_capacity(size);
        payload.extend_from_slice(header);
        payload.resize(size, b'X');
        payload
    }

    /// Scenario: the signal-keyed telemetry label is requested for each signal.
    /// Guarantees: the labels stay "traces", "metrics", and "logs", the exact
    /// strings the per-signal event path emits, so centralizing the mapping on
    /// SignalDecoder does not change emitted telemetry.
    #[test]
    fn signal_label_is_stable_per_signal() {
        assert_eq!(SignalDecoder::label(SignalType::Traces), "traces");
        assert_eq!(SignalDecoder::label(SignalType::Metrics), "metrics");
        assert_eq!(SignalDecoder::label(SignalType::Logs), "logs");
    }

    /// Scenario: an OTLP-proto payload is decoded for each signal through the
    /// single signal-keyed decode path.
    /// Guarantees: each signal wraps the bytes in its matching
    /// OtlpProtoBytes::Export*Request variant, so one decode_signal_payload path
    /// preserves the per-signal OTLP request typing.
    #[test]
    fn decode_signal_payload_otlp_routes_each_signal_to_its_request_variant() {
        // An empty request encodes to zero bytes, a valid OTLP payload.
        let cases: [(SignalType, fn(&OtlpProtoBytes) -> bool); 3] = [
            (SignalType::Traces, |p| {
                matches!(p, OtlpProtoBytes::ExportTracesRequest(_))
            }),
            (SignalType::Metrics, |p| {
                matches!(p, OtlpProtoBytes::ExportMetricsRequest(_))
            }),
            (SignalType::Logs, |p| {
                matches!(p, OtlpProtoBytes::ExportLogsRequest(_))
            }),
        ];
        for (signal, is_expected) in cases {
            let mut pdata =
                SignalDecoder::decode_signal_payload(signal, b"", MessageFormat::OtlpProto)
                    .expect("empty OTLP payload decodes");
            let proto: OtlpProtoBytes = pdata
                .take_payload()
                .try_into_with_default()
                .expect("payload is OtlpProtoBytes");
            assert!(
                is_expected(&proto),
                "signal {signal:?} routed to the wrong OTLP request variant",
            );
        }
    }

    /// Scenario: a Syslog payload is decoded for traces and metrics (non-log
    /// signals).
    /// Guarantees: the "logs only" restriction is enforced in one place --
    /// decode_signal_payload rejects Syslog for traces and metrics with the
    /// canonical error message, and never for logs by accident.
    #[test]
    fn decode_signal_payload_rejects_syslog_for_non_log_signals() {
        for signal in [SignalType::Traces, SignalType::Metrics] {
            let err =
                SignalDecoder::decode_signal_payload(signal, b"whatever", MessageFormat::Syslog)
                    .expect_err("syslog must be rejected for non-log signals");
            let EngineError::PdataConversionError { error } = err else {
                panic!("expected PdataConversionError for {signal:?}");
            };
            assert_eq!(error, "syslog encoding is only supported for logs");
        }
    }

    /// Scenario: a valid RFC 5424 Syslog record is decoded for the logs signal.
    /// Guarantees: logs is the one signal that accepts Syslog, so a well-formed
    /// record decodes successfully through the same single decode path.
    #[test]
    fn decode_signal_payload_accepts_syslog_for_logs() {
        let input = b"<34>1 2003-10-11T22:14:15.003Z host app - ID47 - Test message";
        let mut pdata =
            SignalDecoder::decode_signal_payload(SignalType::Logs, input, MessageFormat::Syslog)
                .expect("valid syslog logs decode");
        let proto: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("syslog logs convert to OTLP bytes");
        let request =
            otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest::decode(
                proto.as_bytes(),
            )
            .expect("decode OTLP logs");
        assert_eq!(request.resource_logs.len(), 1);
    }

    /// Scenario: a Kafka record contains a valid Syslog message exactly at the
    /// shared Syslog receiver size limit.
    /// Guarantees: the boundary-sized payload is accepted and encoded as Arrow
    /// logs.
    #[test]
    fn decode_syslog_logs_accepts_payload_at_size_limit() {
        let payload = syslog_payload_with_size(MAX_SYSLOG_MESSAGE_SIZE);
        let _ = SignalDecoder::decode_syslog_logs(&payload)
            .expect("payload at the size limit should decode");
    }

    /// Scenario: a Kafka record contains a Syslog message one byte larger than
    /// the shared Syslog receiver size limit.
    /// Guarantees: the payload is rejected before parsing to bound parser and
    /// Arrow encoder resource use.
    #[test]
    fn decode_syslog_logs_rejects_payload_over_size_limit() {
        let payload = syslog_payload_with_size(MAX_SYSLOG_MESSAGE_SIZE + 1);
        let error =
            SignalDecoder::decode_syslog_logs(&payload).expect_err("oversized payload should fail");
        assert!(
            error
                .to_string()
                .contains("exceeds the maximum of 16384 bytes")
        );
    }
}
