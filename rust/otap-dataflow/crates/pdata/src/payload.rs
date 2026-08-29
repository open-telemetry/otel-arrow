// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Helpers and direct conversions for native OTAP records and OTLP protobuf bytes.
//!
//! The runtime payload wrapper lives in `otel-arrow-dfe-pdata-codec` so encoded
//! extension identities do not create a dependency cycle with this low-level
//! data model crate.

use crate::TryFromWithOptions;
use crate::encode::{encode_logs_otap_batch, encode_metrics_otap_batch, encode_spans_otap_batch};
use crate::error::Error;
use crate::otap::{OtapArrowRecords, OtapBatchStore};
use crate::otlp::logs::LogsProtoBytesEncoder;
use crate::otlp::metrics::MetricsProtoBytesEncoder;
use crate::otlp::traces::TracesProtoBytesEncoder;
use crate::otlp::{OtlpProtoBytes, ProtoBuffer, ProtoBytesEncoder};
use crate::views::otlp::bytes::logs::RawLogsData;
use crate::views::otlp::bytes::metrics::RawMetricsData;
use crate::views::otlp::bytes::traces::RawTraceData;
use otel_arrow_dfe_config::{ConversionOptions, SignalType};

/// Common measurements and ownership operations for low-level pdata forms.
pub trait OtapPayloadHelpers: Sized {
    /// Returns the telemetry signal represented by this value.
    fn signal_type(&self) -> SignalType;

    /// Returns the number of primary-signal items.
    fn num_items(&self) -> usize;

    /// Returns the logical byte size, if measurable.
    fn num_bytes(&self) -> Option<usize>;

    /// Returns the best available retained-memory byte estimate.
    fn retained_memory_bytes(&self) -> usize;

    /// Returns true when the value contains no primary-signal items.
    fn is_empty(&self) -> bool;

    /// Takes the value, leaving an empty value of the same signal behind.
    fn take_payload(&mut self) -> Self;
}

impl OtapPayloadHelpers for OtapArrowRecords {
    fn signal_type(&self) -> SignalType {
        match self {
            Self::Logs(_) => SignalType::Logs,
            Self::Metrics(_) => SignalType::Metrics,
            Self::Traces(_) => SignalType::Traces,
        }
    }

    fn num_items(&self) -> usize {
        match self {
            Self::Logs(records) => records.num_items(),
            Self::Traces(records) => records.num_items(),
            Self::Metrics(records) => records.num_items(),
        }
    }

    fn num_bytes(&self) -> Option<usize> {
        self.logical_arrow_bytes().ok()
    }

    fn retained_memory_bytes(&self) -> usize {
        self.retained_memory_bytes()
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Logs(_) => self
                .get(crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs)
                .is_none_or(|batch| batch.num_rows() == 0),
            Self::Traces(_) => self
                .get(crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::Spans)
                .is_none_or(|batch| batch.num_rows() == 0),
            Self::Metrics(_) => self
                .get(crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::UnivariateMetrics)
                .is_none_or(|batch| batch.num_rows() == 0),
        }
    }

    fn take_payload(&mut self) -> Self {
        match self {
            Self::Logs(value) => Self::Logs(std::mem::take(value)),
            Self::Metrics(value) => Self::Metrics(std::mem::take(value)),
            Self::Traces(value) => Self::Traces(std::mem::take(value)),
        }
    }
}

impl OtapPayloadHelpers for OtlpProtoBytes {
    fn signal_type(&self) -> SignalType {
        match self {
            Self::ExportLogsRequest(_) => SignalType::Logs,
            Self::ExportMetricsRequest(_) => SignalType::Metrics,
            Self::ExportTracesRequest(_) => SignalType::Traces,
        }
    }

    fn num_items(&self) -> usize {
        count_otlp_items(self.signal_type(), self.as_bytes())
    }

    fn num_bytes(&self) -> Option<usize> {
        Some(self.num_bytes())
    }

    fn retained_memory_bytes(&self) -> usize {
        self.as_bytes().len()
    }

    fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    fn take_payload(&mut self) -> Self {
        match self {
            Self::ExportLogsRequest(value) => Self::ExportLogsRequest(std::mem::take(value)),
            Self::ExportMetricsRequest(value) => Self::ExportMetricsRequest(std::mem::take(value)),
            Self::ExportTracesRequest(value) => Self::ExportTracesRequest(std::mem::take(value)),
        }
    }
}

/// Counts primary-signal OTLP items for compatibility adapters.
#[must_use]
pub fn count_otlp_items(signal: SignalType, bytes: &[u8]) -> usize {
    // Counting traverses the encoded protobuf record hierarchy without
    // constructing an owned request.
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

impl TryFromWithOptions<OtapArrowRecords> for OtlpProtoBytes {
    type Error = Error;

    fn try_from_with_options(
        mut value: OtapArrowRecords,
        options: ConversionOptions,
    ) -> Result<Self, Self::Error> {
        match value {
            OtapArrowRecords::Logs(_) => {
                let mut encoder = LogsProtoBytesEncoder::new();
                let mut buffer = ProtoBuffer::new(options);
                encoder.encode(&mut value, &mut buffer)?;
                Ok(Self::ExportLogsRequest(buffer.into_bytes()))
            }
            OtapArrowRecords::Metrics(_) => {
                let mut encoder = MetricsProtoBytesEncoder::new();
                let mut buffer = ProtoBuffer::new(options);
                encoder.encode(&mut value, &mut buffer)?;
                Ok(Self::ExportMetricsRequest(buffer.into_bytes()))
            }
            OtapArrowRecords::Traces(_) => {
                let mut encoder = TracesProtoBytesEncoder::new();
                let mut buffer = ProtoBuffer::new(options);
                encoder.encode(&mut value, &mut buffer)?;
                Ok(Self::ExportTracesRequest(buffer.into_bytes()))
            }
        }
    }
}

impl TryFromWithOptions<OtlpProtoBytes> for OtapArrowRecords {
    type Error = crate::encode::Error;

    fn try_from_with_options(
        value: OtlpProtoBytes,
        _options: ConversionOptions,
    ) -> Result<Self, Self::Error> {
        match value {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                encode_logs_otap_batch(&RawLogsData::new(bytes.as_ref()))
            }
            OtlpProtoBytes::ExportTracesRequest(bytes) => {
                encode_spans_otap_batch(&RawTraceData::new(bytes.as_ref()))
            }
            OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                encode_metrics_otap_batch(&RawMetricsData::new(bytes.as_ref()))
            }
        }
    }
}
