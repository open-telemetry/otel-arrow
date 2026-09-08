// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Built-in codec for uncompressed OTLP protobuf service-request messages.
//!
//! The decoder converts independently decodable logs, metrics, and traces
//! requests into native OTAP Arrow records. A best-effort implementation uses
//! borrowed protobuf views and validates only outer framing. A strict
//! implementation decodes the complete nested message with Prost before
//! conversion. The pipeline policy selects one implementation when its lazy
//! decoder instance is created.
//!
//! The encoder keeps lazy, signal-specific protobuf encoders and bounded
//! scratch buffers so repeated output avoids reallocating while an unused
//! signal consumes no buffer.

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::encode::{
    encode_logs_otap_batch, encode_metrics_otap_batch, encode_spans_otap_batch,
};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otlp::common::MAX_OTLP_SIZE_LIMIT;
use otel_arrow_dfe_pdata::otlp::logs::LogsProtoBytesEncoder;
use otel_arrow_dfe_pdata::otlp::metrics::MetricsProtoBytesEncoder;
use otel_arrow_dfe_pdata::otlp::traces::TracesProtoBytesEncoder;
use otel_arrow_dfe_pdata::otlp::{BoundedBuf, ProtoBuffer, ProtoBytesEncoder};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::MetricsData;
use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::TracesData;
use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogsData;
use otel_arrow_dfe_pdata::views::otlp::bytes::metrics::RawMetricsData;
use otel_arrow_dfe_pdata::views::otlp::bytes::traces::RawTraceData;
use otel_arrow_dfe_pdata::{OtapPayloadHelpers, OtlpProtoBytes, TryIntoWithOptions};
use prost::Message;

use crate::{
    CodecError, CodecMetadata, CodecOperation, CodecRegistration, DecodePolicy, DecodeValidation,
    EncodeOutput, EncodePolicy, PdataDecoder, PdataEncoder, PdataEncoding,
};

/// Stable identity of uncompressed OTLP protobuf service-request bytes.
pub const OTLP_ENCODING: PdataEncoding = PdataEncoding::OTLP;

const INITIAL_BUFFER_CAPACITY: usize = 8 * 1024;
const MAX_RETAINED_BUFFER_CAPACITY: usize = 256 * 1024;

/// OTLP decoder using borrowed raw views and best-effort validation.
///
/// Top-level protobuf framing is validated before conversion. Nested fields
/// are parsed lazily by the views, whose iterators cannot currently distinguish
/// malformed input from clean exhaustion. Use strict pipeline validation when
/// malformed content anywhere in the message must reject the complete batch.
#[derive(Default)]
pub struct OtlpBestEffortDecoder;

impl PdataDecoder for OtlpBestEffortDecoder {
    fn decode(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<OtapArrowRecords, CodecError> {
        let outer_framing = match signal {
            SignalType::Logs => RawLogsData::try_new(bytes).map(|_| ()),
            SignalType::Metrics => RawMetricsData::try_new(bytes).map(|_| ()),
            SignalType::Traces => RawTraceData::try_new(bytes).map(|_| ()),
        };
        outer_framing.map_err(decode_error)?;

        OtlpProtoBytes::new_from_bytes(signal, bytes.clone())
            .try_into_with_default()
            .map_err(decode_error)
    }
}

/// OTLP decoder that validates the complete nested protobuf message.
struct OtlpStrictDecoder;

impl PdataDecoder for OtlpStrictDecoder {
    fn decode(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<OtapArrowRecords, CodecError> {
        match signal {
            SignalType::Logs => {
                let data = LogsData::decode(bytes.clone()).map_err(decode_error)?;
                encode_logs_otap_batch(&data).map_err(decode_error)
            }
            SignalType::Metrics => {
                let data = MetricsData::decode(bytes.clone()).map_err(decode_error)?;
                encode_metrics_otap_batch(&data).map_err(decode_error)
            }
            SignalType::Traces => {
                let data = TracesData::decode(bytes.clone()).map_err(decode_error)?;
                encode_spans_otap_batch(&data).map_err(decode_error)
            }
        }
    }
}

fn create_decoder(policy: DecodePolicy) -> Box<dyn PdataDecoder> {
    match policy.validation() {
        DecodeValidation::BestEffort => Box::new(OtlpBestEffortDecoder),
        DecodeValidation::Strict => Box::new(OtlpStrictDecoder),
    }
}

fn decode_error(error: impl std::error::Error + Send + Sync + 'static) -> CodecError {
    CodecError::operation(&OTLP_ENCODING, CodecOperation::Decode, error)
}

struct SignalEncoder<E> {
    encoder: E,
    buffer: ProtoBuffer,
}

impl<E: Default> Default for SignalEncoder<E> {
    fn default() -> Self {
        Self {
            encoder: E::default(),
            buffer: ProtoBuffer::with_capacity(INITIAL_BUFFER_CAPACITY),
        }
    }
}

#[derive(Default)]
struct EncoderState {
    // Each signal encoder includes its reusable buffer. Boxing keeps the base
    // OTLP encoder small and allocates the state only after that signal is used.
    // This matters for pipelines that export only one telemetry signal.
    logs: Option<Box<SignalEncoder<LogsProtoBytesEncoder>>>,
    metrics: Option<Box<SignalEncoder<MetricsProtoBytesEncoder>>>,
    traces: Option<Box<SignalEncoder<TracesProtoBytesEncoder>>>,
}

/// OTLP encoder with lazy signal-specific reusable buffers.
pub struct OtlpEncoder {
    output_limit: usize,
    state: EncoderState,
}

impl OtlpEncoder {
    fn new(policy: EncodePolicy) -> Self {
        let output_limit = policy
            .max_encoded_size
            .map_or(MAX_OTLP_SIZE_LIMIT, |limit| {
                limit.get().min(MAX_OTLP_SIZE_LIMIT)
            });
        Self {
            output_limit,
            state: EncoderState::default(),
        }
    }

    /// Prepares logs output with the lazily allocated logs encoder and buffer.
    ///
    /// Keeping this large, signal-specific generic path out of line prevents
    /// the dispatch method from accumulating all three encoder implementations,
    /// reducing generated code and instruction-cache pressure.
    #[inline(never)]
    fn prepare_logs<'a>(
        state: &'a mut EncoderState,
        records: &mut OtapArrowRecords,
        output_limit: usize,
    ) -> Result<EncodeOutput<'a>, CodecError> {
        let state = state
            .logs
            .get_or_insert_with(|| Box::new(SignalEncoder::default()));
        prepare_signal(&mut state.encoder, &mut state.buffer, records, output_limit)
    }

    /// Prepares metrics output with the lazily allocated metrics encoder and buffer.
    ///
    /// Keeping this large, signal-specific generic path out of line prevents
    /// the dispatch method from accumulating all three encoder implementations,
    /// reducing generated code and instruction-cache pressure.
    #[inline(never)]
    fn prepare_metrics<'a>(
        state: &'a mut EncoderState,
        records: &mut OtapArrowRecords,
        output_limit: usize,
    ) -> Result<EncodeOutput<'a>, CodecError> {
        let state = state
            .metrics
            .get_or_insert_with(|| Box::new(SignalEncoder::default()));
        prepare_signal(&mut state.encoder, &mut state.buffer, records, output_limit)
    }

    /// Prepares traces output with the lazily allocated traces encoder and buffer.
    ///
    /// Keeping this large, signal-specific generic path out of line prevents
    /// the dispatch method from accumulating all three encoder implementations,
    /// reducing generated code and instruction-cache pressure.
    #[inline(never)]
    fn prepare_traces<'a>(
        state: &'a mut EncoderState,
        records: &mut OtapArrowRecords,
        output_limit: usize,
    ) -> Result<EncodeOutput<'a>, CodecError> {
        let state = state
            .traces
            .get_or_insert_with(|| Box::new(SignalEncoder::default()));
        prepare_signal(&mut state.encoder, &mut state.buffer, records, output_limit)
    }
}

fn prepare_signal<'a, E: ProtoBytesEncoder>(
    encoder: &mut E,
    buffer: &'a mut ProtoBuffer,
    records: &mut OtapArrowRecords,
    output_limit: usize,
) -> Result<EncodeOutput<'a>, CodecError> {
    buffer.clear();
    buffer.set_limit(output_limit);
    if let Err(error) = encoder.encode(records, buffer) {
        buffer.retain_capacity(MAX_RETAINED_BUFFER_CAPACITY);
        return Err(CodecError::operation(
            &OTLP_ENCODING,
            CodecOperation::Encode,
            error,
        ));
    }
    Ok(EncodeOutput::buffer(buffer, MAX_RETAINED_BUFFER_CAPACITY))
}

impl PdataEncoder for OtlpEncoder {
    fn encode(&mut self, mut records: OtapArrowRecords) -> Result<Bytes, CodecError> {
        Ok(self.prepare_encode(&mut records)?.into_bytes())
    }

    fn prepare_encode<'a>(
        &'a mut self,
        records: &mut OtapArrowRecords,
    ) -> Result<EncodeOutput<'a>, CodecError> {
        match records.signal_type() {
            SignalType::Logs => Self::prepare_logs(&mut self.state, records, self.output_limit),
            SignalType::Metrics => {
                Self::prepare_metrics(&mut self.state, records, self.output_limit)
            }
            SignalType::Traces => Self::prepare_traces(&mut self.state, records, self.output_limit),
        }
    }
}

static OTLP_METADATA: CodecMetadata = CodecMetadata::new(
    OTLP_ENCODING,
    &[SignalType::Logs, SignalType::Metrics, SignalType::Traces],
);

crate::register_pdata_codec!(
    OTLP_CODEC,
    CodecRegistration::new(&OTLP_METADATA)
        .with_decoder(create_decoder)
        .with_encoder(|policy| Ok(Box::new(OtlpEncoder::new(policy))))
        .with_item_counter(|signal, bytes| Some(count_items(signal, bytes))),
);

fn count_items(signal: SignalType, bytes: &[u8]) -> usize {
    match signal {
        SignalType::Logs => {
            let view = RawLogsData::new(bytes);
            use otel_arrow_dfe_pdata_views::views::logs::{
                LogsDataView, ResourceLogsView, ScopeLogsView,
            };
            view.resources()
                .map(|resource| {
                    resource
                        .scopes()
                        .map(|scope| scope.log_records().count())
                        .sum::<usize>()
                })
                .sum()
        }
        SignalType::Traces => {
            let view = RawTraceData::new(bytes);
            use otel_arrow_dfe_pdata_views::views::trace::{
                ResourceSpansView, ScopeSpansView, TracesView,
            };
            view.resources()
                .map(|resource| {
                    resource
                        .scopes()
                        .map(|scope| scope.spans().count())
                        .sum::<usize>()
                })
                .sum()
        }
        SignalType::Metrics => {
            let view = RawMetricsData::new(bytes);
            use otel_arrow_dfe_pdata_views::views::metrics::{
                DataView, ExponentialHistogramView, GaugeView, HistogramView, MetricView,
                MetricsView, ResourceMetricsView, ScopeMetricsView, SumView, SummaryView,
            };
            view.resources()
                .map(|resource| {
                    resource
                        .scopes()
                        .map(|scope| {
                            scope
                                .metrics()
                                .map(|metric| {
                                    metric
                                        .data()
                                        .map(|data| {
                                            if let Some(gauge) = data.as_gauge() {
                                                gauge.data_points().count()
                                            } else if let Some(sum) = data.as_sum() {
                                                sum.data_points().count()
                                            } else if let Some(histogram) = data.as_histogram() {
                                                histogram.data_points().count()
                                            } else if let Some(histogram) =
                                                data.as_exponential_histogram()
                                            {
                                                histogram.data_points().count()
                                            } else if let Some(summary) = data.as_summary() {
                                                summary.data_points().count()
                                            } else {
                                                0
                                            }
                                        })
                                        .unwrap_or(0)
                                })
                                .sum::<usize>()
                        })
                        .sum::<usize>()
                })
                .sum()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use prost::Message;

    use super::*;
    use crate::{
        CodecRegistry, CodecService, CodecServiceBuilder, DecodePolicy, DecodeValidation,
        EncodingPlan, InspectionPlan, PdataView,
    };
    use otel_arrow_dfe_pdata::testing::fixtures::{
        logs_with_full_resource_and_scope, metrics_sum_with_full_resource_and_scope,
        traces_with_full_resource_and_scope,
    };

    fn logs_bytes() -> Bytes {
        logs_with_full_resource_and_scope().encode_to_vec().into()
    }

    fn service(validation: DecodeValidation) -> CodecService {
        CodecServiceBuilder::from_global_registry()
            .expect("valid codec registry")
            .with_decode_policy(DecodePolicy::new(validation))
            .build()
    }

    /// Scenario: OTLP admission uses the validated registry without mutable state.
    /// Guarantees: Admission preserves the shared buffer and stateless item count.
    #[test]
    fn admission_does_not_create_an_instance() {
        let service = CodecService::new().unwrap();
        let codec = service
            .registry()
            .resolve_decoder(&OTLP_ENCODING, SignalType::Logs)
            .unwrap();
        let bytes = logs_bytes();
        let pointer = bytes.as_ptr();
        let encoded = codec.admit(SignalType::Logs, bytes).unwrap();
        assert_eq!(encoded.bytes().as_ptr(), pointer);
        assert_eq!(
            codec.count_items(SignalType::Logs, encoded.bytes()),
            Some(4)
        );
        assert_eq!(service.test_instance_count().unwrap(), 0);
    }

    /// Scenario: A read-only consumer accepts OTLP and another requires native OTAP.
    /// Guarantees: The accepted path borrows bytes and fallback decoding reuses state.
    #[test]
    fn representation_neutral_view_and_fallback() {
        let service = CodecService::new().unwrap();
        let codec = service
            .registry()
            .resolve_decoder(&OTLP_ENCODING, SignalType::Logs)
            .unwrap();
        let encoded = codec.admit(SignalType::Logs, logs_bytes()).unwrap();
        let pointer = encoded.bytes().as_ptr();
        match service
            .view(&encoded, &InspectionPlan::accept_encoded([codec]))
            .unwrap()
        {
            PdataView::Encoded(view) => assert_eq!(view.bytes().as_ptr(), pointer),
            PdataView::Native(_) => panic!("the accepted representation must remain encoded"),
        }
        match service.view(&encoded, &InspectionPlan::native()).unwrap() {
            PdataView::Native(records) => assert_eq!(records.num_items(), 4),
            PdataView::Encoded(_) => panic!("native fallback must decode"),
        }
        assert_eq!(service.test_instance_count().unwrap(), 1);
    }

    /// Scenario: One codec service encodes logs with a startup-resolved size policy.
    /// Guarantees: Encoder state is reused and recovers after a bounded-output error.
    #[test]
    fn encoder_reuses_state_and_recovers() {
        let service = CodecService::new().unwrap();
        let registry = CodecRegistry::global().unwrap();
        let codec = registry
            .resolve_decoder(&OTLP_ENCODING, SignalType::Logs)
            .unwrap();
        let encoded = codec.admit(SignalType::Logs, logs_bytes()).unwrap();
        let records = service.decode(&encoded).unwrap();
        let limited = EncodingPlan::resolve(
            &registry,
            &OTLP_ENCODING,
            EncodePolicy {
                max_encoded_size: NonZeroUsize::new(1),
            },
        )
        .unwrap();
        assert!(
            service
                .encode_bytes(&mut records.clone(), &limited)
                .is_err()
        );
        let normal =
            EncodingPlan::resolve(&registry, &OTLP_ENCODING, EncodePolicy::default()).unwrap();
        assert!(
            !service
                .encode_bytes(&mut records.clone(), &normal)
                .unwrap()
                .is_empty()
        );
    }

    /// Scenario: OTLP logs, metrics, and traces decode under both validation policies.
    /// Guarantees: Both implementations preserve the signal and primary item count.
    #[test]
    fn decodes_all_otlp_signals_in_both_modes() {
        let cases = [
            (
                SignalType::Logs,
                Bytes::from(logs_with_full_resource_and_scope().encode_to_vec()),
                4,
            ),
            (
                SignalType::Metrics,
                Bytes::from(metrics_sum_with_full_resource_and_scope().encode_to_vec()),
                2,
            ),
            (
                SignalType::Traces,
                Bytes::from(traces_with_full_resource_and_scope().encode_to_vec()),
                2,
            ),
        ];
        for validation in [DecodeValidation::BestEffort, DecodeValidation::Strict] {
            let service = service(validation);
            for (signal, bytes, expected) in &cases {
                let codec = service
                    .registry()
                    .resolve_decoder(&OTLP_ENCODING, *signal)
                    .unwrap();
                let encoded = codec.admit(*signal, bytes.clone()).unwrap();
                assert_eq!(service.decode(&encoded).unwrap().num_items(), *expected);
            }
        }
    }

    /// Scenario: Valid OTLP logs are followed by a resource with malformed nested framing.
    /// Guarantees: Best-effort decoding retains the legacy behavior of returning the valid prefix.
    #[test]
    fn best_effort_may_omit_malformed_nested_content() {
        let service = service(DecodeValidation::BestEffort);
        let codec = service
            .registry()
            .resolve_decoder(&OTLP_ENCODING, SignalType::Logs)
            .unwrap();
        let mut bytes = logs_bytes().to_vec();
        // resource_logs { scope_logs: <declared length 5, one byte present> }
        bytes.extend_from_slice(&[0x0a, 0x03, 0x1a, 0x05, 0x00]);
        let encoded = codec.admit(SignalType::Logs, bytes.into()).unwrap();

        assert_eq!(service.decode(&encoded).unwrap().num_items(), 4);
    }

    /// Scenario: Each OTLP signal contains valid records followed by malformed nested framing.
    /// Guarantees: Strict decoding rejects the complete batch repeatedly and then recovers.
    #[test]
    fn strict_rejects_nested_malformed_content_and_recovers() {
        let cases = [
            (
                SignalType::Logs,
                Bytes::from(logs_with_full_resource_and_scope().encode_to_vec()),
            ),
            (
                SignalType::Metrics,
                Bytes::from(metrics_sum_with_full_resource_and_scope().encode_to_vec()),
            ),
            (
                SignalType::Traces,
                Bytes::from(traces_with_full_resource_and_scope().encode_to_vec()),
            ),
        ];
        let service = service(DecodeValidation::Strict);
        for (signal, valid_bytes) in cases {
            let codec = service
                .registry()
                .resolve_decoder(&OTLP_ENCODING, signal)
                .unwrap();
            let mut malformed_bytes = valid_bytes.to_vec();
            malformed_bytes.extend_from_slice(&[0x0a, 0x03, 0x1a, 0x05, 0x00]);
            let malformed = codec.admit(signal, malformed_bytes.into()).unwrap();

            for _ in 0..2 {
                assert!(matches!(
                    service.decode(&malformed),
                    Err(CodecError::Operation {
                        operation: CodecOperation::Decode,
                        ..
                    })
                ));
            }
            let valid = codec.admit(signal, valid_bytes).unwrap();
            assert!(service.decode(&valid).is_ok());
        }
        assert_eq!(service.test_instance_count().unwrap(), 1);
    }
}
