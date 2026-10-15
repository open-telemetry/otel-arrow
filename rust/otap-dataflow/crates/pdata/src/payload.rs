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

use crate::proto::consts::field_num::logs::{
    LOGS_DATA_RESOURCE, RESOURCE_LOGS_SCOPE_LOGS, SCOPE_LOGS_LOG_RECORDS,
};
use crate::proto::consts::field_num::metrics::{
    METRIC_EXPONENTIAL_HISTOGRAM, METRIC_GAUGE, METRIC_HISTOGRAM, METRIC_SUM, METRIC_SUMMARY,
    METRICS_DATA_RESOURCE_METRICS, RESOURCE_METRICS_SCOPE_METRICS, SCOPE_METRICS_METRICS,
};
use crate::proto::consts::field_num::traces::{
    RESOURCE_SPANS_SCOPE_SPANS, SCOPE_SPANS_SPANS, TRACES_DATA_RESOURCE_SPANS,
};
use crate::proto::consts::wire_types;
use crate::views::otlp::bytes::decode::{field_value_range, read_varint};

// Compile-time validation that metrics data-point fields are protobuf field 1, which is leveraged in count_metrics_data_points.
const _: () = {
    use crate::proto::consts::field_num::metrics::{
        EXPONENTIAL_HISTOGRAM_DATA_POINTS, GAUGE_DATA_POINTS, HISTOGRAM_DATA_POINTS,
        SUM_DATA_POINTS, SUMMARY_DATA_POINTS,
    };
    assert!(
        GAUGE_DATA_POINTS == 1,
        "count_metrics_data_points assumes GAUGE_DATA_POINTS == 1"
    );
    assert!(
        SUM_DATA_POINTS == 1,
        "count_metrics_data_points assumes SUM_DATA_POINTS == 1"
    );
    assert!(
        HISTOGRAM_DATA_POINTS == 1,
        "count_metrics_data_points assumes HISTOGRAM_DATA_POINTS == 1"
    );
    assert!(
        EXPONENTIAL_HISTOGRAM_DATA_POINTS == 1,
        "count_metrics_data_points assumes EXPONENTIAL_HISTOGRAM_DATA_POINTS == 1"
    );
    assert!(
        SUMMARY_DATA_POINTS == 1,
        "count_metrics_data_points assumes SUMMARY_DATA_POINTS == 1"
    );
};

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

/// Single-pass allocation free parsing of OTLP items bytes to count items.
#[must_use]
pub fn count_otlp_items(signal: SignalType, bytes: &[u8]) -> usize {
    match signal {
        SignalType::Logs => count_logs_records(bytes).unwrap_or(0),
        SignalType::Traces => count_trace_spans(bytes).unwrap_or(0),
        SignalType::Metrics => count_metrics_data_points(bytes).unwrap_or(0),
    }
}

fn count_logs_records(bytes: &[u8]) -> Result<usize, Error> {
    let mut count: usize = 0;
    let mut request_position = 0;

    while let Some((field, wire_type, resource_bytes)) = next_field(bytes, &mut request_position)? {
        if field != LOGS_DATA_RESOURCE || wire_type != wire_types::LEN {
            continue;
        }

        let mut resource_position = 0;

        while let Some((field, wire_type, scope_bytes)) =
            next_field(resource_bytes, &mut resource_position)?
        {
            if field != RESOURCE_LOGS_SCOPE_LOGS || wire_type != wire_types::LEN {
                continue;
            }

            let mut scope_position = 0;

            while let Some((field, wire_type, _log_record_bytes)) =
                next_field(scope_bytes, &mut scope_position)?
            {
                if field == SCOPE_LOGS_LOG_RECORDS && wire_type == wire_types::LEN {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

fn count_trace_spans(bytes: &[u8]) -> Result<usize, Error> {
    let mut count: usize = 0;
    let mut request_position = 0;

    while let Some((field, wire_type, resource_bytes)) = next_field(bytes, &mut request_position)? {
        if field != TRACES_DATA_RESOURCE_SPANS || wire_type != wire_types::LEN {
            continue;
        }

        let mut resource_position = 0;

        while let Some((field, wire_type, scope_bytes)) =
            next_field(resource_bytes, &mut resource_position)?
        {
            if field != RESOURCE_SPANS_SCOPE_SPANS || wire_type != wire_types::LEN {
                continue;
            }

            let mut scope_position = 0;

            while let Some((field, wire_type, _span_bytes)) =
                next_field(scope_bytes, &mut scope_position)?
            {
                if field == SCOPE_SPANS_SPANS && wire_type == wire_types::LEN {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

fn count_metrics_data_points(bytes: &[u8]) -> Result<usize, Error> {
    let mut count: usize = 0;
    let mut request_position = 0;
    let metric_fields = [
        METRIC_GAUGE,
        METRIC_SUM,
        METRIC_HISTOGRAM,
        METRIC_EXPONENTIAL_HISTOGRAM,
        METRIC_SUMMARY,
    ];

    while let Some((field, wire_type, resource_bytes)) = next_field(bytes, &mut request_position)? {
        if field != METRICS_DATA_RESOURCE_METRICS || wire_type != wire_types::LEN {
            continue;
        }

        let mut resource_position = 0;

        while let Some((field, wire_type, scope_bytes)) =
            next_field(resource_bytes, &mut resource_position)?
        {
            if field != RESOURCE_METRICS_SCOPE_METRICS || wire_type != wire_types::LEN {
                continue;
            }

            let mut metrics_position = 0;

            while let Some((field, wire_type, metrics_bytes)) =
                next_field(scope_bytes, &mut metrics_position)?
            {
                if field != SCOPE_METRICS_METRICS || wire_type != wire_types::LEN {
                    continue;
                }

                let mut data_position = 0;

                let mut data_count = 0;

                while let Some((field, wire_type, data_bytes)) =
                    next_field(metrics_bytes, &mut data_position)?
                {
                    if !metric_fields.contains(&field) || wire_type != wire_types::LEN {
                        continue;
                    }

                    // Reset count for each found `oneof` message (under the `data` field) to keep the last one only
                    data_count = 0;
                    let mut data_point_position = 0;

                    while let Some((field, wire_type, _data_point_bytes)) =
                        next_field(data_bytes, &mut data_point_position)?
                    {
                        // All metric data-point fields are field number 1 in the OTLP protobuf schema. This is validated by static assertions at the beginning of this file.
                        if field == 1 && wire_type == wire_types::LEN {
                            data_count += 1;
                        }
                    }
                }

                count += data_count;
            }
        }
    }

    Ok(count)
}

fn next_field<'a>(
    bytes: &'a [u8],
    position: &mut usize,
) -> Result<Option<(u64, u64, &'a [u8])>, Error> {
    if *position == bytes.len() {
        return Ok(None);
    }

    let (tag, after_tag) = read_varint(bytes, *position).ok_or(Error::InvalidProtobufWireFormat)?;

    let field_number = tag >> 3;
    let wire_type = tag & 7;

    if field_number == 0 {
        return Err(Error::InvalidProtobufWireFormat);
    }

    // Finds the value's boundaries for every supported wire type.
    // For LEN fields, the returned range excludes the length prefix.
    let (start, end) =
        field_value_range(bytes, wire_type, after_tag).ok_or(Error::InvalidProtobufWireFormat)?;

    *position = end;

    Ok(Some((field_number, wire_type, &bytes[start..end])))
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::otlp::BoundedBuf;
    use crate::otlp::common::EncodeFailure;
    use crate::proto::consts::field_num::metrics::{METRIC_NAME, METRIC_UNIT};
    use crate::proto::opentelemetry::metrics::v1::{
        Gauge, NumberDataPoint, Sum, Summary, SummaryDataPoint, number_data_point::Value,
    };
    use bytes::Bytes;
    use prost::Message;

    /// Scenario: A metrics request contains every metric kind across multiple resources.
    /// Guarantees: Byte-backed counting includes every data point exactly once.
    #[test]
    fn test_otlp_proto_bytes_metrics_num_items() {
        use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
        use crate::proto::opentelemetry::common::v1::InstrumentationScope;
        use crate::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
        use crate::proto::opentelemetry::metrics::v1::number_data_point::Value;
        use crate::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
        use crate::proto::opentelemetry::metrics::v1::{
            AggregationTemporality, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge,
            Histogram, HistogramDataPoint, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics,
            Sum, Summary, SummaryDataPoint, metric::Data,
        };
        use crate::proto::opentelemetry::resource::v1::Resource;
        use prost::Message;

        let metrics = ExportMetricsServiceRequest {
            resource_metrics: vec![
                ResourceMetrics {
                    resource: Some(Resource::default()),
                    scope_metrics: vec![ScopeMetrics {
                        scope: Some(InstrumentationScope::default()),
                        metrics: vec![
                            Metric {
                                name: "gauge_metric".into(),
                                data: Some(Data::Gauge(Gauge {
                                    data_points: vec![
                                        NumberDataPoint {
                                            value: Some(Value::AsDouble(1.0)),
                                            ..Default::default()
                                        },
                                        NumberDataPoint {
                                            value: Some(Value::AsDouble(2.0)),
                                            ..Default::default()
                                        },
                                    ],
                                })),
                                ..Default::default()
                            },
                            Metric {
                                name: "sum_metric".into(),
                                data: Some(Data::Sum(Sum {
                                    data_points: vec![
                                        NumberDataPoint {
                                            value: Some(Value::AsInt(100)),
                                            ..Default::default()
                                        },
                                        NumberDataPoint {
                                            value: Some(Value::AsInt(200)),
                                            ..Default::default()
                                        },
                                        NumberDataPoint {
                                            value: Some(Value::AsInt(300)),
                                            ..Default::default()
                                        },
                                    ],
                                    aggregation_temporality: AggregationTemporality::Cumulative
                                        .into(),
                                    is_monotonic: true,
                                })),
                                ..Default::default()
                            },
                            Metric {
                                name: "histogram_metric".into(),
                                data: Some(Data::Histogram(Histogram {
                                    data_points: vec![
                                        HistogramDataPoint {
                                            count: 10,
                                            sum: Some(100.0),
                                            bucket_counts: vec![2, 5, 3],
                                            explicit_bounds: vec![10.0, 50.0],
                                            ..Default::default()
                                        },
                                        HistogramDataPoint {
                                            count: 20,
                                            sum: Some(200.0),
                                            bucket_counts: vec![5, 10, 5],
                                            explicit_bounds: vec![10.0, 50.0],
                                            ..Default::default()
                                        },
                                    ],
                                    aggregation_temporality: AggregationTemporality::Cumulative
                                        .into(),
                                })),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ResourceMetrics {
                    resource: Some(Resource::default()),
                    scope_metrics: vec![ScopeMetrics {
                        scope: Some(InstrumentationScope::default()),
                        metrics: vec![
                            Metric {
                                name: "exp_histogram_metric".into(),
                                data: Some(Data::ExponentialHistogram(ExponentialHistogram {
                                    data_points: vec![
                                        ExponentialHistogramDataPoint {
                                            count: 15,
                                            sum: Some(150.0),
                                            scale: 1,
                                            zero_count: 1,
                                            positive: Some(Buckets {
                                                offset: 0,
                                                bucket_counts: vec![3, 5, 7],
                                            }),
                                            negative: Some(Buckets {
                                                offset: 0,
                                                bucket_counts: vec![1, 2],
                                            }),
                                            ..Default::default()
                                        },
                                        ExponentialHistogramDataPoint {
                                            count: 25,
                                            sum: Some(250.0),
                                            scale: 1,
                                            zero_count: 2,
                                            positive: Some(Buckets {
                                                offset: 0,
                                                bucket_counts: vec![5, 10, 8],
                                            }),
                                            ..Default::default()
                                        },
                                    ],
                                    aggregation_temporality: AggregationTemporality::Cumulative
                                        .into(),
                                })),
                                ..Default::default()
                            },
                            Metric {
                                name: "summary_metric".into(),
                                data: Some(Data::Summary(Summary {
                                    data_points: vec![
                                        SummaryDataPoint {
                                            count: 100,
                                            sum: 1000.0,
                                            quantile_values: vec![
                                                ValueAtQuantile {
                                                    quantile: 0.5,
                                                    value: 10.0,
                                                },
                                                ValueAtQuantile {
                                                    quantile: 0.95,
                                                    value: 50.0,
                                                },
                                            ],
                                            ..Default::default()
                                        },
                                        SummaryDataPoint {
                                            count: 200,
                                            sum: 2000.0,
                                            quantile_values: vec![ValueAtQuantile {
                                                quantile: 0.5,
                                                value: 20.0,
                                            }],
                                            ..Default::default()
                                        },
                                    ],
                                })),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
        };

        let mut buf = Vec::new();
        metrics.encode(&mut buf).unwrap();

        let otlp_bytes = OtlpProtoBytes::ExportMetricsRequest(Bytes::from(buf));

        assert_eq!(otlp_bytes.num_items(), 11);
    }

    fn proto_encode_sum(proto: &mut ProtoBuffer) -> Result<(), EncodeFailure> {
        proto.encode_len_delimited(METRIC_SUM, |proto| {
            let data = Sum {
                data_points: vec![NumberDataPoint {
                    value: Some(Value::AsInt(255)),
                    attributes: vec![],
                    exemplars: vec![],
                    start_time_unix_nano: 0,
                    time_unix_nano: 0,
                    flags: 0,
                }],
                aggregation_temporality: 0,
                is_monotonic: false,
            };
            let mut bytes_data = Vec::new();
            data.encode(&mut bytes_data).unwrap();
            proto.extend_from_slice(&bytes_data)
        })
    }

    fn proto_encode_gauge(proto: &mut ProtoBuffer) -> Result<(), EncodeFailure> {
        proto.encode_len_delimited(METRIC_GAUGE, |proto| {
            let data = Gauge {
                data_points: vec![
                    NumberDataPoint {
                        value: Some(Value::AsInt(12)),
                        attributes: vec![],
                        exemplars: vec![],
                        start_time_unix_nano: 0,
                        time_unix_nano: 0,
                        flags: 0,
                    },
                    NumberDataPoint {
                        value: Some(Value::AsInt(10)),
                        attributes: vec![],
                        exemplars: vec![],
                        start_time_unix_nano: 0,
                        time_unix_nano: 0,
                        flags: 0,
                    },
                    NumberDataPoint {
                        value: Some(Value::AsInt(15)),
                        attributes: vec![],
                        exemplars: vec![],
                        start_time_unix_nano: 0,
                        time_unix_nano: 0,
                        flags: 0,
                    },
                    NumberDataPoint {
                        value: Some(Value::AsInt(14)),
                        attributes: vec![],
                        exemplars: vec![],
                        start_time_unix_nano: 0,
                        time_unix_nano: 0,
                        flags: 0,
                    },
                ],
            };
            let mut bytes_data = Vec::new();
            data.encode(&mut bytes_data).unwrap();
            proto.extend_from_slice(&bytes_data)
        })
    }

    fn proto_encode_summary(proto: &mut ProtoBuffer) -> Result<(), EncodeFailure> {
        proto.encode_len_delimited(METRIC_SUMMARY, |proto| {
            let data = Summary {
                data_points: vec![
                    SummaryDataPoint {
                        count: 9,
                        sum: 33.0,
                        quantile_values: vec![],
                        attributes: vec![],
                        start_time_unix_nano: 0,
                        time_unix_nano: 0,
                        flags: 0,
                    },
                    SummaryDataPoint {
                        count: 9,
                        sum: 33.0,
                        quantile_values: vec![],
                        attributes: vec![],
                        start_time_unix_nano: 0,
                        time_unix_nano: 0,
                        flags: 0,
                    },
                ],
            };
            let mut bytes_data = Vec::new();
            data.encode(&mut bytes_data).unwrap();
            proto.extend_from_slice(&bytes_data)
        })
    }

    /// Scenario: Ill-formed metrics encode several ordered combinations of oneof fields.
    /// Guarantees: Standard decoding and byte-backed item counting use each case's final field.
    #[test]
    fn ill_formed_metric_uses_last_oneof_field() {
        let specs: [(fn(&mut ProtoBuffer) -> Result<(), EncodeFailure>, usize); 3] = [
            (
                |proto| {
                    proto_encode_sum(proto)?;
                    proto_encode_summary(proto)?;
                    proto_encode_gauge(proto)
                },
                4,
            ),
            (
                |proto| {
                    proto_encode_gauge(proto)?;
                    proto_encode_sum(proto)
                },
                1,
            ),
            (
                |proto| {
                    proto_encode_sum(proto)?;
                    proto_encode_gauge(proto)?;
                    proto_encode_summary(proto)
                },
                2,
            ),
        ];

        for (payload_fn, expected_num_items) in specs {
            let mut proto = ProtoBuffer::default();
            proto
                .encode_len_delimited(METRICS_DATA_RESOURCE_METRICS, |proto| {
                    proto.encode_len_delimited(RESOURCE_METRICS_SCOPE_METRICS, |proto| {
                        proto.encode_len_delimited(SCOPE_METRICS_METRICS, |proto| {
                            proto.encode_string(METRIC_NAME, "metric1")?;
                            proto.encode_string(METRIC_UNIT, "centimeters")?;
                            payload_fn(proto)
                        })
                    })
                })
                .unwrap();
            let parsed_bytes = OtlpProtoBytes::ExportMetricsRequest(proto.into_bytes());
            assert_eq!(parsed_bytes.num_items(), expected_num_items);
        }
    }
}
