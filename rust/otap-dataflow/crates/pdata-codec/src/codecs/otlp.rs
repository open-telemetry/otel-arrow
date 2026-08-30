// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Built-in codec for uncompressed OTLP protobuf service-request messages.
//!
//! The decoder validates and converts independently decodable logs, metrics,
//! and traces requests into native OTAP Arrow records. The encoder keeps lazy,
//! signal-specific protobuf encoders and bounded scratch buffers so repeated
//! output avoids reallocating while an unused signal consumes no buffer.

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otlp::common::MAX_OTLP_SIZE_LIMIT;
use otel_arrow_dfe_pdata::otlp::logs::LogsProtoBytesEncoder;
use otel_arrow_dfe_pdata::otlp::metrics::MetricsProtoBytesEncoder;
use otel_arrow_dfe_pdata::otlp::traces::TracesProtoBytesEncoder;
use otel_arrow_dfe_pdata::otlp::{BoundedBuf, ProtoBuffer, ProtoBytesEncoder};
use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogsData;
use otel_arrow_dfe_pdata::views::otlp::bytes::metrics::RawMetricsData;
use otel_arrow_dfe_pdata::views::otlp::bytes::traces::RawTraceData;
use otel_arrow_dfe_pdata::{OtapPayloadHelpers, OtlpProtoBytes, TryIntoWithOptions};

use crate::{
    CodecError, CodecMetadata, CodecOperation, CodecRegistration, EncodeOutput, EncodePolicy,
    PdataDecoder, PdataEncoder, PdataEncoding,
};

/// Stable identity of uncompressed OTLP protobuf service-request bytes.
pub const OTLP_ENCODING: PdataEncoding = PdataEncoding::OTLP;

const INITIAL_BUFFER_CAPACITY: usize = 8 * 1024;
const MAX_RETAINED_BUFFER_CAPACITY: usize = 256 * 1024;

/// Stateless OTLP protobuf decoder.
#[derive(Default)]
pub struct OtlpDecoder;

impl PdataDecoder for OtlpDecoder {
    fn decode(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<OtapArrowRecords, CodecError> {
        let strict_result = match signal {
            SignalType::Logs => RawLogsData::try_new(bytes).map(|_| ()),
            SignalType::Metrics => RawMetricsData::try_new(bytes).map(|_| ()),
            SignalType::Traces => RawTraceData::try_new(bytes).map(|_| ()),
        };
        strict_result.map_err(|error| {
            CodecError::operation(&OTLP_ENCODING, CodecOperation::Decode, error)
        })?;
        OtlpProtoBytes::new_from_bytes(signal, bytes.clone())
            .try_into_with_default()
            .map_err(|error| CodecError::operation(&OTLP_ENCODING, CodecOperation::Decode, error))
    }
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
        .with_decoder(|| Box::new(OtlpDecoder))
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
    use crate::{CodecRegistry, CodecService, EncodingPlan, PdataView, ViewPlan};
    use otel_arrow_dfe_pdata::testing::fixtures::{
        logs_with_full_resource_and_scope, metrics_sum_with_full_resource_and_scope,
        traces_with_full_resource_and_scope,
    };

    fn logs_bytes() -> Bytes {
        logs_with_full_resource_and_scope().encode_to_vec().into()
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
        assert_eq!(service.test_instance_count(), 0);
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
            .view(&encoded, &ViewPlan::accept_encoded([codec]))
            .unwrap()
        {
            PdataView::Encoded(view) => assert_eq!(view.bytes().as_ptr(), pointer),
            PdataView::Native(_) => panic!("the accepted representation must remain encoded"),
        }
        match service.view(&encoded, &ViewPlan::native()).unwrap() {
            PdataView::Native(records) => assert_eq!(records.num_items(), 4),
            PdataView::Encoded(_) => panic!("native fallback must decode"),
        }
        assert_eq!(service.test_instance_count(), 1);
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
            SignalType::Logs,
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
        let normal = EncodingPlan::resolve(
            &registry,
            &OTLP_ENCODING,
            SignalType::Logs,
            EncodePolicy::default(),
        )
        .unwrap();
        assert!(
            !service
                .encode_bytes(&mut records.clone(), &normal)
                .unwrap()
                .is_empty()
        );
    }

    /// Scenario: OTLP logs, metrics, and traces decode through the same extension contract.
    /// Guarantees: Each supported signal preserves its primary item count.
    #[test]
    fn decodes_all_otlp_signals() {
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
        let service = CodecService::new().unwrap();
        for (signal, bytes, expected) in cases {
            let codec = service
                .registry()
                .resolve_decoder(&OTLP_ENCODING, signal)
                .unwrap();
            let encoded = codec.admit(signal, bytes).unwrap();
            assert_eq!(service.decode(&encoded).unwrap().num_items(), expected);
        }
    }
}
