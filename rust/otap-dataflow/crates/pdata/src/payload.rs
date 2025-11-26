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
//! # use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
//! # use prost::Message;
//! # use bytes::Bytes;
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
//! // Create a new OtapPayload from OTLP bytes
//! let payload: OtapPayload = OtlpProtoBytes::ExportLogsRequest(Bytes::from(buf)).into();
//!
//! // Convert to OTAP records
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
use otap_df_config::SignalType;

/// Container for the various representations of the telemetry data
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum OtapPayload {
    /// data is serialized as a protobuf service message for one of the OTLP GRPC services
    OtlpBytes(OtlpProtoBytes),

    /// data is contained in `OtapBatch` which contains Arrow `RecordBatches` for OTAP payload type
    OtapArrowRecords(OtapArrowRecords),
}

impl OtapPayload {
    /// Returns the type of signal represented by this `OtapPdata` instance.
    #[must_use]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::OtlpBytes(value) => value.signal_type(),
            Self::OtapArrowRecords(value) => value.signal_type(),
        }
    }

    /// True if the payload is empty. By definition, we can skip sending an
    /// empty request.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::OtlpBytes(value) => value.is_empty(),
            Self::OtapArrowRecords(value) => value.is_empty(),
        }
    }

    /// Removes the payload from this request, leaving an empty request.
    #[must_use]
    pub fn take_payload(&mut self) -> Self {
        match self {
            Self::OtlpBytes(value) => Self::OtlpBytes(value.take_payload()),
            Self::OtapArrowRecords(value) => Self::OtapArrowRecords(value.take_payload()),
        }
    }

    /// Returns the number of items of the primary signal (spans, data
    /// points, log records).
    #[must_use]
    pub fn num_items(&self) -> usize {
        match self {
            Self::OtlpBytes(value) => value.num_items(),
            Self::OtapArrowRecords(value) => value.num_items(),
        }
    }
}

/* -------- Trait implementations -------- */

/// Helper methods that internal representations of OTAP PData should implement
pub trait OtapPayloadHelpers {
    /// Returns the type of signal represented by this `OtapPdata` instance.
    fn signal_type(&self) -> SignalType;

    /// Number of items.
    fn num_items(&self) -> usize;

    /// Return true if there is no data.
    fn is_empty(&self) -> bool;

    /// Takes the payload, leaving an empty payload behind.
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

    fn take_payload(&mut self) -> Self {
        match self {
            Self::Logs(value) => Self::Logs(std::mem::take(value)),
            Self::Metrics(value) => Self::Metrics(std::mem::take(value)),
            Self::Traces(value) => Self::Traces(std::mem::take(value)),
        }
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

    fn num_items(&self) -> usize {
        match self {
            Self::Logs(records) => records.batch_length(),
            Self::Traces(records) => records.batch_length(),
            Self::Metrics(records) => records.batch_length(),
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

    fn is_empty(&self) -> bool {
        match self {
            Self::ExportLogsRequest(bytes) => bytes.is_empty(),
            Self::ExportMetricsRequest(bytes) => bytes.is_empty(),
            Self::ExportTracesRequest(bytes) => bytes.is_empty(),
        }
    }

    fn take_payload(&mut self) -> Self {
        match self {
            Self::ExportLogsRequest(value) => Self::ExportLogsRequest(std::mem::take(value)),
            Self::ExportMetricsRequest(value) => Self::ExportMetricsRequest(std::mem::take(value)),
            Self::ExportTracesRequest(value) => Self::ExportTracesRequest(std::mem::take(value)),
        }
    }

    fn num_items(&self) -> usize {
        match self {
            Self::ExportLogsRequest(bytes) => {
                let logs_data_view = RawLogsData::new(bytes.as_ref());
                use crate::views::logs::{LogsDataView, ResourceLogsView, ScopeLogsView};
                logs_data_view
                    .resources()
                    .map(|rl| {
                        rl.scopes()
                            .map(|sl| sl.log_records().count())
                            .sum::<usize>()
                    })
                    .sum()
            }
            Self::ExportTracesRequest(bytes) => {
                let traces_data_view = RawTraceData::new(bytes.as_ref());
                use crate::views::trace::{ResourceSpansView, ScopeSpansView, TracesView};
                traces_data_view
                    .resources()
                    .map(|rs| rs.scopes().map(|ss| ss.spans().count()).sum::<usize>())
                    .sum()
            }
            Self::ExportMetricsRequest(_bytes) => {
                // Metrics view is not implemented yet
                panic!("ToDo")
            }
        }
    }
}

/* -------- Conversion implementations -------- */

impl From<OtapArrowRecords> for OtapPayload {
    fn from(value: OtapArrowRecords) -> Self {
        Self::OtapArrowRecords(value)
    }
}

impl From<OtlpProtoBytes> for OtapPayload {
    fn from(value: OtlpProtoBytes) -> Self {
        Self::OtlpBytes(value)
    }
}

impl TryFrom<OtapPayload> for OtapArrowRecords {
    type Error = Error;

    fn try_from(value: OtapPayload) -> Result<Self, Self::Error> {
        match value {
            OtapPayload::OtapArrowRecords(value) => Ok(value),
            OtapPayload::OtlpBytes(value) => value.try_into(),
        }
    }
}

impl TryFrom<OtapPayload> for OtlpProtoBytes {
    type Error = Error;

    fn try_from(value: OtapPayload) -> Result<Self, Self::Error> {
        match value {
            OtapPayload::OtapArrowRecords(value) => value.try_into(),
            OtapPayload::OtlpBytes(value) => Ok(value),
        }
    }
}

impl TryFrom<OtapArrowRecords> for OtlpProtoBytes {
    type Error = Error;

    fn try_from(mut value: OtapArrowRecords) -> Result<Self, Self::Error> {
        match value {
            OtapArrowRecords::Logs(_) => {
                // TODO it'd be nice to expose a better API where we can make it easier to pass the encoder
                // and the buffer, a these structures can be used between requests
                let mut logs_encoder = LogsProtoBytesEncoder::new();
                let mut buffer = ProtoBuffer::new();

                logs_encoder.encode(&mut value, &mut buffer)?;
                Ok(Self::ExportLogsRequest(buffer.into_bytes()))
            }
            OtapArrowRecords::Metrics(_) => {
                let mut metrics_encoder = MetricsProtoBytesEncoder::new();
                let mut buffer = ProtoBuffer::new();
                metrics_encoder.encode(&mut value, &mut buffer)?;

                Ok(Self::ExportMetricsRequest(buffer.into_bytes()))
            }
            OtapArrowRecords::Traces(_) => {
                let mut traces_encoder = TracesProtoBytesEncoder::new();
                let mut buffer = ProtoBuffer::new();
                traces_encoder.encode(&mut value, &mut buffer)?;
                Ok(Self::ExportTracesRequest(buffer.into_bytes()))
            }
        }
    }
}

impl TryFrom<OtlpProtoBytes> for OtapArrowRecords {
    type Error = Error;

    fn try_from(value: OtlpProtoBytes) -> Result<Self, Self::Error> {
        match value {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                let logs_data_view = RawLogsData::new(bytes.as_ref());
                let otap_batch = encode_logs_otap_batch(&logs_data_view)?;

                Ok(otap_batch)
            }
            OtlpProtoBytes::ExportTracesRequest(bytes) => {
                let trace_data_view = RawTraceData::new(bytes.as_ref());
                let otap_batch = encode_spans_otap_batch(&trace_data_view)?;

                Ok(otap_batch)
            }
            OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                let metrics_data_view = RawMetricsData::new(bytes.as_ref());
                let otap_batch = encode_metrics_otap_batch(&metrics_data_view)?;

                Ok(otap_batch)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::fixtures::logs_with_full_resource_and_scope;
    use crate::{
        otap::OtapArrowRecords,
        proto::opentelemetry::{
            collector::{
                logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
                trace::v1::ExportTraceServiceRequest,
            },
            common::v1::{AnyValue, InstrumentationScope, KeyValue},
            logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
            metrics::v1::{
                AggregationTemporality, Exemplar, ExponentialHistogram,
                ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint, Metric,
                NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
                exemplar, exponential_histogram_data_point::Buckets, metric::Data,
                number_data_point::Value, summary_data_point::ValueAtQuantile,
            },
            resource::v1::Resource,
            trace::v1::{
                ResourceSpans, ScopeSpans, Span, SpanFlags, Status,
                span::{Event, Link},
                status::StatusCode,
            },
        },
    };
    use bytes::Bytes;
    use pretty_assertions::assert_eq;
    use prost::Message;

    #[test]
    fn test_conversion_logs() {
        let mut otlp_bytes = vec![];
        let otlp_service_req = logs_with_full_resource_and_scope();
        otlp_service_req.encode(&mut otlp_bytes).unwrap();

        let pdata: OtapPayload = OtlpProtoBytes::ExportLogsRequest(otlp_bytes.into()).into();

        // test can go OtlpProtoBytes -> OtapBatch & back
        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();
        assert!(matches!(otap_batch, OtapArrowRecords::Logs(_)));
        let pdata: OtapPayload = otap_batch.into();

        let otlp_bytes: OtlpProtoBytes = pdata.try_into().unwrap();
        assert!(matches!(otlp_bytes, OtlpProtoBytes::ExportLogsRequest(_)));
        let pdata: OtapPayload = otlp_bytes.into();

        let otlp_bytes: OtlpProtoBytes = pdata.try_into().unwrap();
        assert!(matches!(otlp_bytes, OtlpProtoBytes::ExportLogsRequest(_)));
        let pdata: OtapPayload = otlp_bytes.into();

        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();
        assert!(matches!(otap_batch, OtapArrowRecords::Logs(_)));
    }

    // TODO add additional tests for converting between metrics once we have the ability to convert
    //  between OTLP bytes -> OTAP for this signal types
    // https://github.com/open-telemetry/otel-arrow/issues/768

    fn roundtrip_otlp_otap_logs(otlp_service_req: ExportLogsServiceRequest) {
        let mut otlp_bytes = vec![];
        otlp_service_req.encode(&mut otlp_bytes).unwrap();
        let pdata: OtapPayload = OtlpProtoBytes::ExportLogsRequest(otlp_bytes.into()).into();

        // test can go OtlpProtoBytes (written by prost) -> OtapBatch & back (using prost)
        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();
        assert!(matches!(otap_batch, OtapArrowRecords::Logs(_)));
        let pdata: OtapPayload = otap_batch.clone().into();

        let otlp_bytes: OtlpProtoBytes = pdata.try_into().unwrap();
        let bytes = match &otlp_bytes {
            OtlpProtoBytes::ExportLogsRequest(bytes) => bytes.clone(),
            _ => panic!("unexpected otlp bytes pdata variant"),
        };

        let result = ExportLogsServiceRequest::decode(bytes.as_ref()).unwrap();
        assert_eq!(otlp_service_req, result);

        // check that we can also re-decode the OTLP proto bytes that we encode directly
        // from OTAP, that we get the same result
        let pdata: OtapPayload = otlp_bytes.into();
        let otap_batch2: OtapArrowRecords = pdata.try_into().unwrap();
        assert_eq!(otap_batch, otap_batch2);
    }

    fn roundtrip_otlp_otap_traces(otlp_service_req: ExportTraceServiceRequest) {
        let mut otlp_bytes = vec![];
        otlp_service_req.encode(&mut otlp_bytes).unwrap();
        let pdata: OtapPayload = OtlpProtoBytes::ExportTracesRequest(otlp_bytes.into()).into();

        // test can go OtlpBytes (written by prost) -> OtapBatch & back (using prost)
        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();
        assert!(matches!(otap_batch, OtapArrowRecords::Traces(_)));
        let pdata: OtapPayload = otap_batch.clone().into();

        let otlp_bytes: OtlpProtoBytes = pdata.try_into().unwrap();
        let bytes = match &otlp_bytes {
            OtlpProtoBytes::ExportTracesRequest(bytes) => bytes.clone(),
            _ => panic!("unexpected otlp bytes pdata variant"),
        };

        let result = ExportTraceServiceRequest::decode(bytes.as_ref()).unwrap();
        assert_eq!(otlp_service_req, result);

        // check that we can also re-decode the OTLP proto bytes that we encode directly
        // from OTAP, that we get the same result
        let pdata: OtapPayload = otlp_bytes.into();
        let otap_batch2: OtapArrowRecords = pdata.try_into().unwrap();
        assert_eq!(otap_batch, otap_batch2);
    }

    fn roundtrip_otlp_otap_metrics(otlp_service_request: ExportMetricsServiceRequest) {
        let mut otlp_bytes = vec![];
        otlp_service_request.encode(&mut otlp_bytes).unwrap();
        let pdata: OtapPayload = OtlpProtoBytes::ExportMetricsRequest(otlp_bytes.into()).into();

        // test can go OtlpBytes (written by prost) to OTAP & back (using prost)
        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();
        assert!(matches!(otap_batch, OtapArrowRecords::Metrics(_)));
        let pdata: OtapPayload = otap_batch.clone().into();

        let otlp_bytes: OtlpProtoBytes = pdata.try_into().unwrap();
        let bytes = match &otlp_bytes {
            OtlpProtoBytes::ExportMetricsRequest(bytes) => bytes.clone(),
            _ => panic!("unexpected otlp bytes pdata variant"),
        };

        let result = ExportMetricsServiceRequest::decode(bytes.as_ref()).unwrap();
        assert_eq!(otlp_service_request, result);

        // check that we can also re-decode the OTLP proto bytes that we encode directly
        // from OTAP, that we get the same result
        let pdata: OtapPayload = otlp_bytes.into();
        let otap_batch2: OtapArrowRecords = pdata.try_into().unwrap();
        assert_eq!(otap_batch, otap_batch2);
    }

    #[test]
    fn test_otlp_otap_logs_roundtrip() {
        // test to ensure the correct attributes are assigned to the correct log message after
        // roundtrip encoding/decoding

        let otlp_service_req = ExportLogsServiceRequest::new(vec![
            ResourceLogs::new(
                Resource {
                    attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val1"))],
                    ..Default::default()
                },
                vec![ScopeLogs::new(
                    InstrumentationScope {
                        attributes: vec![KeyValue::new("scope_key", AnyValue::new_string("val1"))],
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build()
                            .time_unix_nano(1u64)
                            .severity_number(SeverityNumber::Info)
                            .event_name("event1")
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                            .finish(),
                        LogRecord::build()
                            .time_unix_nano(2u64)
                            .severity_number(SeverityNumber::Info)
                            .event_name("event1")
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                            .finish(),
                        LogRecord::build()
                            .time_unix_nano(3u64)
                            .severity_number(SeverityNumber::Info)
                            .event_name("event2")
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val3"))])
                            .finish(),
                    ],
                )],
            ),
            ResourceLogs::new(
                Resource {
                    attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val2"))],
                    ..Default::default()
                },
                vec![
                    ScopeLogs::new(
                        InstrumentationScope {
                            name: "Scope2".into(),
                            attributes: vec![KeyValue::new(
                                "scope_key",
                                AnyValue::new_string("val2"),
                            )],
                            ..Default::default()
                        },
                        vec![
                            LogRecord::build()
                                .time_unix_nano(4u64)
                                .severity_number(SeverityNumber::Info)
                                .event_name("event3")
                                .attributes(vec![KeyValue::new(
                                    "key",
                                    AnyValue::new_string("val4"),
                                )])
                                .finish(),
                            LogRecord::build()
                                .time_unix_nano(5u64)
                                .severity_number(SeverityNumber::Info)
                                .event_name("event1")
                                .attributes(vec![KeyValue::new(
                                    "key",
                                    AnyValue::new_string("val5"),
                                )])
                                .finish(),
                        ],
                    ),
                    ScopeLogs::new(
                        InstrumentationScope {
                            attributes: vec![KeyValue::new(
                                "scope_key",
                                AnyValue::new_string("val3"),
                            )],
                            ..Default::default()
                        },
                        vec![
                            LogRecord::build()
                                .time_unix_nano(6u64)
                                .severity_number(SeverityNumber::Info)
                                .event_name("event1")
                                .attributes(vec![KeyValue::new(
                                    "key",
                                    AnyValue::new_string("val6"),
                                )])
                                .finish(),
                            LogRecord::build()
                                .time_unix_nano(7u64)
                                .severity_number(SeverityNumber::Info)
                                .event_name("")
                                .attributes(vec![KeyValue::new(
                                    "key",
                                    AnyValue::new_string("val7"),
                                )])
                                .finish(),
                        ],
                    ),
                ],
            ),
        ]);

        roundtrip_otlp_otap_logs(otlp_service_req);
    }

    #[test]
    fn test_otlp_otap_logs_repeated_attributes() {
        // check to ensure attributes that are repeated are correctly encoded and decoding when
        // doing round-trip between OTLP and OTAP. This test is needed because OTAP attributes'
        // parent IDs can be in multiple formats: plain encoded, and quasi-delta encoded (where
        // delta encoding is used for sequential runs of some key-value pairs).

        let otlp_service_req = ExportLogsServiceRequest::new(vec![
            ResourceLogs::new(
                Resource {
                    attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val"))],
                    ..Default::default()
                },
                vec![ScopeLogs::new(
                    InstrumentationScope::build()
                        .name("scope1")
                        .attributes(vec![KeyValue::new(
                            "scope_key",
                            AnyValue::new_string("val"),
                        )])
                        .finish(),
                    vec![
                        // Add some logs with repeated attributes
                        LogRecord::build()
                            .time_unix_nano(1u64)
                            .severity_number(SeverityNumber::Info)
                            .event_name("")
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                            .finish(),
                        LogRecord::build()
                            .time_unix_nano(2u64)
                            .severity_number(SeverityNumber::Info)
                            .event_name("")
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                            .finish(),
                        LogRecord::build()
                            .time_unix_nano(3u64)
                            .severity_number(SeverityNumber::Info)
                            .event_name("")
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                            .finish(),
                    ],
                )],
            ),
            // also add some scopes and resources where the attributes repeat ...
            ResourceLogs::new(
                Resource {
                    attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val"))],
                    ..Default::default()
                },
                vec![ScopeLogs::new(
                    InstrumentationScope::build()
                        .name("scope2")
                        .attributes(vec![KeyValue::new(
                            "scope_key",
                            AnyValue::new_string("val"),
                        )])
                        .finish(),
                    vec![
                        LogRecord::build()
                            .time_unix_nano(4u64)
                            .severity_number(SeverityNumber::Info)
                            .event_name("")
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                            .finish(),
                    ],
                )],
            ),
            ResourceLogs::new(
                Resource {
                    attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val"))],
                    ..Default::default()
                },
                vec![ScopeLogs::new(
                    InstrumentationScope::build()
                        .name("scope2")
                        .attributes(vec![KeyValue::new(
                            "scope_key",
                            AnyValue::new_string("val"),
                        )])
                        .finish(),
                    vec![
                        LogRecord::build()
                            .time_unix_nano(7u64)
                            .severity_number(SeverityNumber::Info)
                            .event_name("")
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                            .finish(),
                    ],
                )],
            ),
        ]);

        roundtrip_otlp_otap_logs(otlp_service_req);
    }

    #[test]
    fn test_otlp_otap_traces_roundtrip() {
        let otlp_service_req = ExportTraceServiceRequest::new(vec![
            ResourceSpans::new(
                Resource::build()
                    .attributes(vec![KeyValue::new("res_key", AnyValue::new_string("val1"))])
                    .finish(),
                vec![ScopeSpans::new(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new(
                            "scope_key",
                            AnyValue::new_string("val1"),
                        )])
                        .finish(),
                    vec![
                        Span::build()
                            .trace_id(u128::to_be_bytes(1).to_vec())
                            .span_id(u64::to_be_bytes(1).to_vec())
                            .name("albert")
                            .start_time_unix_nano(1u64)
                            .end_time_unix_nano(4u64)
                            .status(Status::new(StatusCode::Ok, "status1"))
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                            .links(vec![
                                Link::build()
                                    .trace_id(u128::to_be_bytes(10))
                                    .span_id(u64::to_be_bytes(10))
                                    .finish(),
                                // this Link's trace_id repeats with the next one. doing this to ensure
                                // their parent IDs don't get interpreted as delta encoded
                                Link::build()
                                    .trace_id(u128::to_be_bytes(11))
                                    .span_id(u64::to_be_bytes(11))
                                    .attributes(vec![
                                        KeyValue::new("link_key", AnyValue::new_string("val0")),
                                        // repeating the attr here with the next one for same reason
                                        // as repeating link with trace ID
                                        KeyValue::new("link_key_r", AnyValue::new_string("val1")),
                                    ])
                                    .finish(),
                            ])
                            .events(vec![
                                Event::build().name("event0").time_unix_nano(0u64).finish(),
                                // this event has the repeating name with the next one. doing this to
                                // ensure their parent IDs don't get interpreted as delta encoded
                                Event::build()
                                    .name("event1")
                                    .time_unix_nano(1u64)
                                    .attributes(vec![
                                        KeyValue::new("evt_key", AnyValue::new_string("val0")),
                                        // repeating the attr here with the next one for same reason
                                        // as repeating link with trace ID
                                        KeyValue::new("evt_key_r", AnyValue::new_string("val1")),
                                    ])
                                    .finish(),
                            ])
                            .finish(),
                        Span::build()
                            .trace_id(u128::to_be_bytes(2))
                            .span_id(u64::to_be_bytes(2))
                            .name("terry")
                            .start_time_unix_nano(2u64)
                            .flags(SpanFlags::TraceFlagsMask)
                            .end_time_unix_nano(3u64)
                            .status(Status::new(StatusCode::Ok, "status1"))
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                            .links(vec![
                                // this Link's trace_id repeats with the previous, and  next one.
                                // doing this to ensure their parent IDs don't get interpreted as
                                // delta encoded
                                Link::build()
                                    .trace_id(u128::to_be_bytes(11))
                                    .span_id(u64::to_be_bytes(20))
                                    .attributes(vec![
                                        KeyValue::new("link_key_r", AnyValue::new_string("val1")),
                                        KeyValue::new("link_key", AnyValue::new_string("val2")),
                                    ])
                                    .flags(255u32)
                                    .finish(),
                            ])
                            .events(vec![
                                // this event has the repeating name with the next one and previous one
                                // doing this to ensure their parent IDs don't get interpreted as
                                // delta encoded
                                Event::build()
                                    .name("event1")
                                    .time_unix_nano(1u64)
                                    .attributes(vec![
                                        KeyValue::new("evt_key_r", AnyValue::new_string("val1")),
                                        KeyValue::new("evt_key", AnyValue::new_string("val2")),
                                    ])
                                    .finish(),
                            ])
                            .finish(),
                    ],
                )],
            ),
            ResourceSpans::new(
                Resource::build()
                    .attributes(vec![KeyValue::new("res_key", AnyValue::new_string("val2"))])
                    .finish(),
                vec![ScopeSpans::new(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new(
                            "scope_key",
                            AnyValue::new_string("val3"),
                        )])
                        .finish(),
                    vec![
                        Span::build()
                            .trace_id(u128::to_be_bytes(3))
                            .span_id(u64::to_be_bytes(3))
                            .name("albert")
                            .start_time_unix_nano(3u64)
                            .end_time_unix_nano(4u64)
                            .status(Status::new(StatusCode::Ok, "status1"))
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                            .links(vec![
                                // this Link's trace_id repeats with the previous one. do this to ensure they
                                // don't get interpreted as delta encoded
                                Link::build()
                                    .trace_id(u128::to_be_bytes(11))
                                    .span_id(u64::to_be_bytes(30))
                                    .finish(),
                                Link::build()
                                    .trace_id(u128::to_be_bytes(31))
                                    .span_id(u64::to_be_bytes(31))
                                    .finish(),
                            ])
                            .events(vec![
                                // this event has the repeating name with the previous one. doing this
                                // to ensure their parent IDs don't get interpreted as delta encoded
                                Event::build().name("event1").time_unix_nano(2u64).finish(),
                            ])
                            .finish(),
                        Span::build()
                            .trace_id(u128::to_be_bytes(4))
                            .span_id(u64::to_be_bytes(4))
                            .name("terry")
                            .start_time_unix_nano(4u64)
                            .end_time_unix_nano(5u64)
                            .status(Status::new(StatusCode::Ok, "status1"))
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val4"))])
                            .links(vec![
                                Link::build()
                                    .trace_id(u128::to_be_bytes(40))
                                    .span_id(u64::to_be_bytes(40))
                                    .finish(),
                            ])
                            .finish(),
                    ],
                )],
            ),
        ]);

        roundtrip_otlp_otap_traces(otlp_service_req);
    }

    #[test]
    fn test_otlp_otap_metrics_roundtrip() {
        let otlp_service_req = ExportMetricsServiceRequest::new(vec![ResourceMetrics {
            schema_url: "resource1 schema url".into(),
            resource: Some(Resource {
                dropped_attributes_count: 1,
                attributes: vec![KeyValue::new("res_attr1", AnyValue::new_string("res_val1"))],
                // TODO support entity refs
                entity_refs: Default::default(),
            }),

            scope_metrics: vec![ScopeMetrics {
                schema_url: "scope1 schema url".into(),
                scope: Some(InstrumentationScope {
                    name: "scope1 name".into(),
                    version: "scope1 version".into(),
                    attributes: vec![KeyValue::new("scp_attr1", AnyValue::new_string("scp_val1"))],
                    dropped_attributes_count: 2,
                }),
                metrics: vec![
                    Metric {
                        name: "metric1".into(),
                        description: "metric1 desc".into(),
                        unit: "m1 unit".into(),
                        // Test empty data
                        data: None,
                        metadata: vec![KeyValue::new(
                            "met_attr1",
                            AnyValue::new_string("met_val1"),
                        )],
                    },
                    Metric {
                        name: "metric2".into(),
                        description: "metric2 desc".into(),
                        unit: "m2 unit".into(),
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "attr1",
                                        AnyValue::new_string("val1"),
                                    )],
                                    start_time_unix_nano: 5,
                                    time_unix_nano: 6,
                                    exemplars: vec![
                                        Exemplar {
                                            time_unix_nano: 56,
                                            span_id: 4u64.to_le_bytes().to_vec(),
                                            trace_id: 999u128.to_le_bytes().to_vec(),
                                            value: Some(exemplar::Value::AsDouble(-3.0)),
                                            filtered_attributes: vec![
                                                KeyValue::new(
                                                    "attr1",
                                                    AnyValue::new_string("val1"),
                                                ),
                                                KeyValue::new(
                                                    "attr2",
                                                    AnyValue::new_string("val1"),
                                                ),
                                            ],
                                        },
                                        Exemplar {
                                            time_unix_nano: 56,
                                            span_id: 2u64.to_le_bytes().to_vec(),
                                            trace_id: 9393939u128.to_le_bytes().to_vec(),
                                            value: Some(exemplar::Value::AsInt(-10)),
                                            filtered_attributes: vec![KeyValue::new(
                                                "attr3",
                                                AnyValue::new_string("val3"),
                                            )],
                                        },
                                    ],
                                    flags: 8,
                                    value: None, // Test None Value
                                },
                                NumberDataPoint {
                                    attributes: vec![
                                        KeyValue::new("attr1", AnyValue::new_string("val1")),
                                        KeyValue::new("attr2", AnyValue::new_string("val1")),
                                    ],
                                    start_time_unix_nano: 6,
                                    time_unix_nano: 7,
                                    exemplars: vec![Exemplar {
                                        time_unix_nano: 156,
                                        span_id: 12u64.to_le_bytes().to_vec(),
                                        trace_id: 932339u128.to_le_bytes().to_vec(),
                                        value: Some(exemplar::Value::AsInt(10)),
                                        filtered_attributes: vec![KeyValue::new(
                                            "attr4",
                                            AnyValue::new_string("val40"),
                                        )],
                                    }],
                                    flags: 9,
                                    value: Some(Value::AsDouble(11.0)), // Test double value
                                },
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "attr2",
                                        AnyValue::new_string("val1"),
                                    )],
                                    start_time_unix_nano: 6,
                                    time_unix_nano: 8,
                                    exemplars: vec![],
                                    flags: 9,
                                    value: Some(Value::AsInt(14)), // Test int value
                                },
                            ],
                        })),
                        metadata: vec![
                            KeyValue::new("met_attr1", AnyValue::new_string("met_val2")),
                            KeyValue::new("met_attr2", AnyValue::new_string("met_val2")),
                        ],
                    },
                    Metric {
                        name: "metric3".into(),
                        description: "metric3 desc".into(),
                        unit: "m3 unit".into(),
                        metadata: vec![KeyValue::new(
                            "met_attr2",
                            AnyValue::new_string("met_val1"),
                        )],
                        data: Some(Data::Sum(Sum {
                            aggregation_temporality: AggregationTemporality::Cumulative as i32,
                            is_monotonic: true,
                            data_points: vec![
                                NumberDataPoint {
                                    attributes: vec![
                                        KeyValue::new("attr2", AnyValue::new_string("val1")),
                                        KeyValue::new("attr4", AnyValue::new_string("val1")),
                                    ],
                                    start_time_unix_nano: 16,
                                    time_unix_nano: 18,
                                    exemplars: vec![],
                                    flags: 19,
                                    value: Some(Value::AsInt(14)),
                                },
                                NumberDataPoint {
                                    attributes: vec![KeyValue::new(
                                        "attr",
                                        AnyValue::new_string("val1"),
                                    )],
                                    start_time_unix_nano: 17,
                                    time_unix_nano: 18,
                                    exemplars: vec![Exemplar {
                                        time_unix_nano: 1,
                                        span_id: 2u64.to_le_bytes().to_vec(),
                                        trace_id: 3u128.to_le_bytes().to_vec(),
                                        value: Some(exemplar::Value::AsDouble(-4.0)),
                                        filtered_attributes: vec![KeyValue::new(
                                            "attr5",
                                            AnyValue::new_string("val6"),
                                        )],
                                    }],
                                    flags: 0,
                                    value: Some(Value::AsInt(14)),
                                },
                            ],
                        })),
                    },
                    Metric {
                        name: "metric4".into(),
                        description: "metric4 desc".into(),
                        unit: "m4 unit".into(),
                        metadata: vec![
                            KeyValue::new("met_attr1", AnyValue::new_string("met_val2")),
                            KeyValue::new("met_attr2", AnyValue::new_string("met_val2")),
                            KeyValue::new("met_attr3", AnyValue::new_string("met_val1")),
                        ],
                        data: Some(Data::Histogram(Histogram {
                            aggregation_temporality: AggregationTemporality::Delta as i32,
                            data_points: vec![
                                HistogramDataPoint {
                                    time_unix_nano: 1,
                                    start_time_unix_nano: 2,
                                    attributes: vec![
                                        KeyValue::new("attr1", AnyValue::new_string("val1")),
                                        KeyValue::new("attr2", AnyValue::new_string("val2")),
                                    ],
                                    count: 3,
                                    sum: Some(4.0),
                                    bucket_counts: vec![1, 2],
                                    exemplars: vec![Exemplar {
                                        time_unix_nano: 56,
                                        span_id: 78u64.to_le_bytes().to_vec(),
                                        trace_id: 1011u128.to_le_bytes().to_vec(),
                                        value: Some(exemplar::Value::AsInt(-10)),
                                        filtered_attributes: vec![KeyValue::new(
                                            "attr4",
                                            AnyValue::new_string("terry"),
                                        )],
                                    }],
                                    explicit_bounds: vec![3.0, 4.0, 5.0],
                                    flags: 6,
                                    min: Some(7.0),
                                    max: Some(8.0),
                                },
                                HistogramDataPoint {
                                    time_unix_nano: 3,
                                    start_time_unix_nano: 4,
                                    attributes: vec![KeyValue::new(
                                        "attr1",
                                        AnyValue::new_string("val1"),
                                    )],
                                    count: 2,
                                    sum: Some(5.0),
                                    bucket_counts: vec![6, 7, 8],
                                    exemplars: vec![],
                                    explicit_bounds: vec![9.0, 10.0],
                                    flags: 16,
                                    min: Some(17.0),
                                    max: Some(18.0),
                                },
                            ],
                        })),
                    },
                    Metric {
                        name: "metric5".into(),
                        description: "metric5 desc".into(),
                        unit: "m5 unit".into(),
                        metadata: vec![
                            KeyValue::new("attr1", AnyValue::new_string("val6")),
                            KeyValue::new("attr2", AnyValue::new_string("val7")),
                        ],
                        data: Some(Data::ExponentialHistogram(ExponentialHistogram {
                            aggregation_temporality: AggregationTemporality::Cumulative as i32,
                            data_points: vec![
                                ExponentialHistogramDataPoint {
                                    start_time_unix_nano: 8,
                                    time_unix_nano: 3,
                                    count: 99,
                                    sum: Some(94.4),
                                    scale: 76,
                                    zero_count: 324,
                                    positive: Some(Buckets {
                                        offset: -3,
                                        bucket_counts: vec![1, 2, 2345435235, 2, 443434],
                                    }),
                                    negative: Some(Buckets {
                                        offset: 5,
                                        bucket_counts: vec![1, 2, 4, 0, 1, 3, 9999, 3],
                                    }),
                                    flags: 48,
                                    min: Some(9.4),
                                    max: Some(99.5),
                                    zero_threshold: 4.9,
                                    attributes: vec![
                                        KeyValue::new("attr1", AnyValue::new_string("val6")),
                                        KeyValue::new("attr2", AnyValue::new_string("val7")),
                                    ],
                                    exemplars: vec![Exemplar {
                                        time_unix_nano: 9,
                                        span_id: 78u64.to_le_bytes().to_vec(),
                                        trace_id: 1011u128.to_le_bytes().to_vec(),
                                        value: Some(exemplar::Value::AsInt(-999)),
                                        filtered_attributes: vec![KeyValue::new(
                                            "attr4",
                                            AnyValue::new_string("lance"),
                                        )],
                                    }],
                                },
                                ExponentialHistogramDataPoint {
                                    positive: Some(Buckets {
                                        offset: -3,
                                        bucket_counts: vec![4, 4, 5, 3],
                                    }),
                                    negative: Some(Buckets {
                                        offset: 5,
                                        bucket_counts: vec![1, 2, 3],
                                    }),
                                    attributes: vec![KeyValue::new(
                                        "attr1",
                                        AnyValue::new_string("val6"),
                                    )],
                                    exemplars: vec![], // TODO
                                    ..Default::default()
                                },
                            ],
                        })),
                    },
                    Metric {
                        name: "metric6".into(),
                        description: "metric 6 desc".into(),
                        unit: "metric6 unit".into(),
                        metadata: vec![
                            KeyValue::new("attr1", AnyValue::new_string("val99")),
                            KeyValue::new("attr2", AnyValue::new_string("val007")),
                        ],
                        data: Some(Data::Summary(Summary {
                            data_points: vec![
                                SummaryDataPoint {
                                    count: 1,
                                    sum: 2.0,
                                    attributes: vec![
                                        KeyValue::new("dp_attr1", AnyValue::new_string("val99")),
                                        KeyValue::new("dp_attr2", AnyValue::new_string("val007")),
                                    ],
                                    start_time_unix_nano: 8383,
                                    time_unix_nano: 9873,
                                    quantile_values: vec![
                                        ValueAtQuantile {
                                            quantile: 1.0,
                                            value: 2.0,
                                        },
                                        ValueAtQuantile {
                                            quantile: 8.0,
                                            value: 4.0,
                                        },
                                        ValueAtQuantile {
                                            quantile: 9.0,
                                            value: 5.0,
                                        },
                                    ],
                                    flags: 256,
                                },
                                SummaryDataPoint {
                                    count: 11,
                                    sum: 21.0,
                                    attributes: vec![KeyValue::new(
                                        "dp_attr11",
                                        AnyValue::new_string("val99"),
                                    )],
                                    start_time_unix_nano: 333,
                                    time_unix_nano: 444,
                                    quantile_values: vec![
                                        ValueAtQuantile {
                                            quantile: 11.0,
                                            value: 20.0,
                                        },
                                        ValueAtQuantile {
                                            quantile: 81.0,
                                            value: 40.0,
                                        },
                                        ValueAtQuantile {
                                            quantile: 91.0,
                                            value: 59.0,
                                        },
                                    ],
                                    flags: 200,
                                },
                            ],
                        })),
                    },
                ],
            }],
        }]);

        roundtrip_otlp_otap_metrics(otlp_service_req);
    }

    #[test]
    fn test_signal_type() {
        // Test signal_type for OtlpProtoBytes variants
        let logs_bytes = OtlpProtoBytes::ExportLogsRequest(Bytes::new());
        let metrics_bytes = OtlpProtoBytes::ExportMetricsRequest(Bytes::new());
        let traces_bytes = OtlpProtoBytes::ExportTracesRequest(Bytes::new());

        assert_eq!(logs_bytes.signal_type(), SignalType::Logs);
        assert_eq!(metrics_bytes.signal_type(), SignalType::Metrics);
        assert_eq!(traces_bytes.signal_type(), SignalType::Traces);

        // Test signal_type for OtapArrowRecords variants
        let logs_records = OtapArrowRecords::Logs(Default::default());
        let metrics_records = OtapArrowRecords::Metrics(Default::default());
        let traces_records = OtapArrowRecords::Traces(Default::default());

        assert_eq!(logs_records.signal_type(), SignalType::Logs);
        assert_eq!(metrics_records.signal_type(), SignalType::Metrics);
        assert_eq!(traces_records.signal_type(), SignalType::Traces);

        // Test signal_type for OtapPdata variants
        let pdata_logs: OtapPayload = OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into();
        let pdata_metrics: OtapPayload = OtapArrowRecords::Metrics(Default::default()).into();
        assert_eq!(pdata_logs.signal_type(), SignalType::Logs);
        assert_eq!(pdata_metrics.signal_type(), SignalType::Metrics);
    }
}
