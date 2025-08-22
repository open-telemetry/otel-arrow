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
//! # use otel_arrow_rust::otap::{OtapArrowRecords, Logs};
//! # use otel_arrow_rust::proto::opentelemetry::{
//!     arrow::v1::ArrowPayloadType,
//!     collector::logs::v1::ExportLogsServiceRequest,
//!     common::v1::{AnyValue, InstrumentationScope, KeyValue},
//!     logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
//!     resource::v1::Resource    
//! };
//! # use otap_df_otap::{pdata::{OtapPdata, OtlpProtoBytes}, grpc::OtapArrowBytes};
//! # use prost::Message;
//! let otlp_service_req = ExportLogsServiceRequest::new(vec![
//!    ResourceLogs::build(Resource::default())
//!       .scope_logs(vec![
//!            ScopeLogs::build(InstrumentationScope::default())
//!                .log_records(vec![
//!                    LogRecord::build(2u64, SeverityNumber::Info, "event")
//!                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
//!                        .finish(),
//!                ])
//!                .finish(),
//!        ])
//!        .finish(),
//!  ]);
//! let mut buf = Vec::new();
//! otlp_service_req.encode(&mut buf).unwrap();
//!
//! let otap_pdata = OtapPdata::from(OtlpProtoBytes::ExportLogsRequest(buf));
//!
//! // convert to Otap Arrow Records
//! let otap_arrow_records: OtapArrowRecords = otap_pdata.try_into().unwrap();
//!
//! // convert to OTAP Arrow Bytes
//! let otap_pdata: OtapPdata = otap_arrow_records.into();
//! let otap_arrow_bytes: OtapArrowBytes = otap_pdata.try_into().unwrap();
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
//!    otap_df_otap::encoder::encode_<signal>_otap_batch    otel_arrow_rust::otlp::<signal>::<signal_>_from()
//!                                          │                 ▲                                              
//!                                          │                 │                                              
//!                                          │                 │                                              
//!                                          ▼                 │                                              
//!                                      ┌─────────────────────┴───┐                                          
//!                                      │                         │                                          
//!                                      │    OTAP Arrow Records   │                                          
//!                                      │                         │                                          
//!                                      └───┬─  ──────────────────┘                                          
//!                                          │                 ▲                                              
//!                                          │                 │                                              
//!                                          ▼                 │                                              
//!       otel_arrow_rust::Producer::produce_bar()    otel_arrow_rust::Consumer::consume_bar()                
//!                                          │                 ▲                                              
//!                                          │                 │                                              
//!                                          ▼                 │                                              
//!                                       ┌────────────────────┴───┐                                         
//!                                       │                        │                                         
//!                                       │    OTAP Arrow Bytes    │                                         
//!                                       │                        │                                         
//!                                       └────────────────────────┘                                         
//!                                                                           
//! ```

use otap_df_config::experimental::SignalType;
use otap_df_pdata_views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata_views::otlp::bytes::traces::RawTraceData;
use otel_arrow_rust::otap::{OtapArrowRecords, from_record_messages};
use otel_arrow_rust::otlp::{logs::logs_from, metrics::metrics_from, traces::traces_from};
use otel_arrow_rust::{Consumer, Producer};
use prost::{EncodeError, Message};

use crate::encoder::encode_spans_otap_batch;
use crate::{encoder::encode_logs_otap_batch, grpc::OtapArrowBytes};

/// module contains related to pdata
pub mod error {
    /// Errors related to pdata
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        /// Wrapper for error that occurred converting pdata
        #[error("An error occurred converting pdata: {error}")]
        ConversionError {
            /// The error that occurred
            error: String,
        },
    }

    impl<T> From<Error> for otap_df_engine::error::Error<T> {
        fn from(e: Error) -> Self {
            otap_df_engine::error::Error::PdataConversionError {
                error: format!("{e}"),
            }
        }
    }
}

/// Pipeline data represented as protobuf serialized OTLP request messages
#[derive(Clone, Debug)]
pub enum OtlpProtoBytes {
    /// protobuf serialized ExportLogsServiceRequest
    ExportLogsRequest(Vec<u8>),
    /// protobuf serialized ExportMetricsServiceRequest
    ExportMetricsRequest(Vec<u8>),
    /// protobuf serialized ExportTracesServiceRequest
    ExportTracesRequest(Vec<u8>),
}

impl OtlpProtoBytes {
    /// Get a borrowed reference to the serialized proto bytes slice
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            OtlpProtoBytes::ExportLogsRequest(bytes)
            | OtlpProtoBytes::ExportMetricsRequest(bytes)
            | OtlpProtoBytes::ExportTracesRequest(bytes) => bytes.as_slice(),
        }
    }
}

/// Container for the various representations of the telemetry data
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum OtapPdata {
    /// data is serialized as a protobuf service message for one of the OTLP GRPC services
    OtlpBytes(OtlpProtoBytes),

    /// data is contained in `BatchArrowRecords`, which contain ArrowIPC serialized
    OtapArrowBytes(OtapArrowBytes),

    /// data is contained in `OtapBatch` which contains Arrow `RecordBatches` for OTAP payload type
    OtapArrowRecords(OtapArrowRecords),
}

impl OtapPdata {
    /// Returns the type of signal represented by this `OtapPdata` instance.
    #[must_use]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::OtlpBytes(inner) => inner.signal_type(),
            Self::OtapArrowBytes(inner) => inner.signal_type(),
            Self::OtapArrowRecords(inner) => inner.signal_type(),
        }
    }
}

/* -------- Helper trait implementations -------- */

/// Helper methods that internal representations of OTAP PData should implement
trait OtapPdataHelpers {
    /// Returns the type of signal represented by this `OtapPdata` instance.
    fn signal_type(&self) -> SignalType;
}

impl OtapPdataHelpers for OtlpProtoBytes {
    fn signal_type(&self) -> SignalType {
        match self {
            Self::ExportLogsRequest(_) => SignalType::Logs,
            Self::ExportMetricsRequest(_) => SignalType::Metrics,
            Self::ExportTracesRequest(_) => SignalType::Traces,
        }
    }
}

impl OtapPdataHelpers for OtapArrowRecords {
    fn signal_type(&self) -> SignalType {
        match self {
            Self::Logs(_) => SignalType::Logs,
            Self::Metrics(_) => SignalType::Metrics,
            Self::Traces(_) => SignalType::Traces,
        }
    }
}

impl OtapPdataHelpers for OtapArrowBytes {
    fn signal_type(&self) -> SignalType {
        match self {
            Self::ArrowLogs(_) => SignalType::Logs,
            Self::ArrowMetrics(_) => SignalType::Metrics,
            Self::ArrowTraces(_) => SignalType::Traces,
        }
    }
}

/* -------- Conversion implementations -------- */

impl From<OtapArrowRecords> for OtapPdata {
    fn from(value: OtapArrowRecords) -> Self {
        Self::OtapArrowRecords(value)
    }
}

impl From<OtlpProtoBytes> for OtapPdata {
    fn from(value: OtlpProtoBytes) -> Self {
        Self::OtlpBytes(value)
    }
}

impl From<OtapArrowBytes> for OtapPdata {
    fn from(value: OtapArrowBytes) -> Self {
        Self::OtapArrowBytes(value)
    }
}

impl TryFrom<OtapPdata> for OtapArrowRecords {
    type Error = error::Error;

    fn try_from(value: OtapPdata) -> Result<Self, Self::Error> {
        match value {
            OtapPdata::OtapArrowBytes(otap_data) => otap_data.try_into(),
            OtapPdata::OtapArrowRecords(otap_batch) => Ok(otap_batch),
            OtapPdata::OtlpBytes(otlp_bytes) => otlp_bytes.try_into(),
        }
    }
}

impl TryFrom<OtapPdata> for OtlpProtoBytes {
    type Error = error::Error;

    fn try_from(value: OtapPdata) -> Result<Self, Self::Error> {
        match value {
            OtapPdata::OtapArrowBytes(otap_data) => otap_data.try_into(),
            OtapPdata::OtapArrowRecords(otap_batch) => otap_batch.try_into(),
            OtapPdata::OtlpBytes(otlp_bytes) => Ok(otlp_bytes),
        }
    }
}

impl TryFrom<OtapPdata> for OtapArrowBytes {
    type Error = error::Error;

    fn try_from(value: OtapPdata) -> Result<Self, Self::Error> {
        match value {
            OtapPdata::OtapArrowBytes(otap_data) => Ok(otap_data),
            OtapPdata::OtapArrowRecords(otap_batch) => otap_batch.try_into(),
            OtapPdata::OtlpBytes(otlp_bytes) => otlp_bytes.try_into(),
        }
    }
}

impl TryFrom<OtapArrowRecords> for OtlpProtoBytes {
    type Error = error::Error;

    fn try_from(value: OtapArrowRecords) -> Result<Self, Self::Error> {
        let map_otlp_conversion_error =
            |error: otel_arrow_rust::error::Error| error::Error::ConversionError {
                error: format!("error generating OTLP request: {error}"),
            };

        let map_prost_encode_error = |error: EncodeError| error::Error::ConversionError {
            error: format!("error encoding protobuf: {error}"),
        };

        match value {
            OtapArrowRecords::Logs(_) => {
                let export_logs_svc_req = logs_from(value).map_err(map_otlp_conversion_error)?;
                let mut bytes = vec![];
                export_logs_svc_req
                    .encode(&mut bytes)
                    .map_err(map_prost_encode_error)?;
                Ok(Self::ExportLogsRequest(bytes))
            }
            OtapArrowRecords::Metrics(_) => {
                let export_metrics_svc_req =
                    metrics_from(value).map_err(map_otlp_conversion_error)?;
                let mut bytes = vec![];
                export_metrics_svc_req
                    .encode(&mut bytes)
                    .map_err(map_prost_encode_error)?;
                Ok(Self::ExportMetricsRequest(bytes))
            }
            OtapArrowRecords::Traces(_) => {
                let export_traces_svc_req =
                    traces_from(value).map_err(map_otlp_conversion_error)?;
                let mut bytes = vec![];
                export_traces_svc_req
                    .encode(&mut bytes)
                    .map_err(map_prost_encode_error)?;
                Ok(Self::ExportTracesRequest(bytes))
            }
        }
    }
}

impl TryFrom<OtlpProtoBytes> for OtapArrowRecords {
    type Error = error::Error;

    fn try_from(value: OtlpProtoBytes) -> Result<Self, Self::Error> {
        let map_error = |error: crate::encoder::error::Error| error::Error::ConversionError {
            error: format!("error encoding OTAP Batch: {error}"),
        };
        match value {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                let logs_data_view = RawLogsData::new(&bytes);
                let otap_batch = encode_logs_otap_batch(&logs_data_view).map_err(map_error)?;

                Ok(otap_batch)
            }
            OtlpProtoBytes::ExportTracesRequest(bytes) => {
                let trace_data_view = RawTraceData::new(&bytes);
                let otap_batch = encode_spans_otap_batch(&trace_data_view).map_err(map_error)?;

                Ok(otap_batch)
            }
            _ => {
                // TODO add conversions when we support
                // https://github.com/open-telemetry/otel-arrow/issues/768
                Err(error::Error::ConversionError {
                    error: "converting from OTLP Bytes for this signal type not yet supported"
                        .to_string(),
                })
            }
        }
    }
}

impl TryFrom<OtlpProtoBytes> for OtapArrowBytes {
    type Error = error::Error;

    fn try_from(value: OtlpProtoBytes) -> Result<Self, Self::Error> {
        let otap_batch: OtapArrowRecords = value.try_into()?;
        otap_batch.try_into()
    }
}

impl TryFrom<OtapArrowBytes> for OtlpProtoBytes {
    type Error = error::Error;

    fn try_from(value: OtapArrowBytes) -> Result<Self, Self::Error> {
        let otap_batch: OtapArrowRecords = value.try_into()?;
        otap_batch.try_into()
    }
}

impl TryFrom<OtapArrowBytes> for OtapArrowRecords {
    type Error = error::Error;

    fn try_from(value: OtapArrowBytes) -> Result<Self, Self::Error> {
        let map_error = |error: otel_arrow_rust::error::Error| error::Error::ConversionError {
            error: format!("error decoding BatchArrowRecords: {error}"),
        };
        let mut consumer = Consumer::default();
        match value {
            OtapArrowBytes::ArrowLogs(mut bar) => {
                let record_messages = consumer.consume_bar(&mut bar).map_err(map_error)?;
                Ok(Self::Logs(from_record_messages(record_messages)))
            }
            OtapArrowBytes::ArrowMetrics(mut bar) => {
                let record_messages = consumer.consume_bar(&mut bar).map_err(map_error)?;
                Ok(Self::Metrics(from_record_messages(record_messages)))
            }
            OtapArrowBytes::ArrowTraces(mut bar) => {
                let record_messages = consumer.consume_bar(&mut bar).map_err(map_error)?;
                Ok(Self::Traces(from_record_messages(record_messages)))
            }
        }
    }
}

impl TryFrom<OtapArrowRecords> for OtapArrowBytes {
    type Error = error::Error;

    fn try_from(mut otap_batch: OtapArrowRecords) -> Result<Self, Self::Error> {
        let mut producer = Producer::new();
        let bar =
            producer
                .produce_bar(&mut otap_batch)
                .map_err(|e| error::Error::ConversionError {
                    error: format!("error encoding BatchArrowRecords: {e}"),
                })?;
        Ok(match otap_batch {
            OtapArrowRecords::Logs(_) => Self::ArrowLogs(bar),
            OtapArrowRecords::Metrics(_) => Self::ArrowMetrics(bar),
            OtapArrowRecords::Traces(_) => Self::ArrowTraces(bar),
        })
    }
}

#[cfg(test)]
/// Testing utilities for OtapPdata
pub mod test {
    use super::*;
    use otap_df_config::experimental::SignalType;
    use otel_arrow_rust::{
        otap::OtapArrowRecords,
        proto::opentelemetry::{
            collector::{logs::v1::ExportLogsServiceRequest, trace::v1::ExportTraceServiceRequest},
            common::v1::{AnyValue, InstrumentationScope, KeyValue},
            logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
            resource::v1::Resource,
            trace::v1::{
                ResourceSpans, ScopeSpans, Span, Status,
                span::{Event, Link},
                status::StatusCode,
            },
        },
    };

    /// Test helper function to count the number of primary signal items in OtapPdata.
    /// 
    /// This function extracts the main signal records (logs, spans, or metrics) and returns
    /// the number of rows, which represents the count of primary signal items:
    /// - For logs: number of log records
    /// - For traces: number of spans  
    /// - For metrics: number of metrics
    pub fn num_rows(pdata: &OtapPdata) -> usize {
        match pdata.signal_type() {
            SignalType::Logs => {
                let records: otel_arrow_rust::otap::OtapArrowRecords = pdata.clone().try_into().unwrap();
                records.get(otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs)
                    .map_or(0, |batch| batch.num_rows())
            }
            SignalType::Traces => {
                let records: otel_arrow_rust::otap::OtapArrowRecords = pdata.clone().try_into().unwrap();
                records.get(otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType::Spans)
                    .map_or(0, |batch| batch.num_rows())
            }
            SignalType::Metrics => {
                let records: otel_arrow_rust::otap::OtapArrowRecords = pdata.clone().try_into().unwrap();
                records.get(otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType::UnivariateMetrics)
                    .map_or(0, |batch| batch.num_rows())
            }
        }
    }

    #[test]
    fn test_conversion_logs() {
        let otlp_service_req = ExportLogsServiceRequest::new(vec![
            ResourceLogs::build(Resource::default())
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::default())
                        .log_records(vec![
                            LogRecord::build(2u64, SeverityNumber::Info, "event")
                                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                                .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ]);
        let mut otlp_bytes = vec![];
        otlp_service_req.encode(&mut otlp_bytes).unwrap();

        let pdata: OtapPdata = OtlpProtoBytes::ExportLogsRequest(otlp_bytes).into();

        // test can go OtlpProtoBytes -> OtapBatch & back
        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();
        assert!(matches!(otap_batch, OtapArrowRecords::Logs(_)));
        let pdata: OtapPdata = otap_batch.into();

        let otlp_bytes: OtlpProtoBytes = pdata.try_into().unwrap();
        assert!(matches!(otlp_bytes, OtlpProtoBytes::ExportLogsRequest(_)));
        let pdata: OtapPdata = otlp_bytes.into();

        // test can go OtlpProtoBytes -> OTAPData
        let otap_data: OtapArrowBytes = pdata.try_into().unwrap();
        assert!(matches!(otap_data, OtapArrowBytes::ArrowLogs(_)));
        let pdata: OtapPdata = otap_data.into();

        let otlp_bytes: OtlpProtoBytes = pdata.try_into().unwrap();
        assert!(matches!(otlp_bytes, OtlpProtoBytes::ExportLogsRequest(_)));
        let pdata: OtapPdata = otlp_bytes.into();

        // test can go otap_batch -> OTAPData
        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();
        let pdata: OtapPdata = otap_batch.into();
        let otap_data: OtapArrowBytes = pdata.try_into().unwrap();
        assert!(matches!(otap_data, OtapArrowBytes::ArrowLogs(_)));
        let pdata: OtapPdata = otap_data.into();

        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();
        assert!(matches!(otap_batch, OtapArrowRecords::Logs(_)));
    }

    // TODO add additional tests for converting between metrics once we have the ability to convert
    //  between OTLP bytes -> OTAP for this signal types
    // https://github.com/open-telemetry/otel-arrow/issues/768

    fn roundtrip_otlp_otap_logs(otlp_service_req: ExportLogsServiceRequest) {
        let mut otlp_bytes = vec![];
        otlp_service_req.encode(&mut otlp_bytes).unwrap();
        let pdata: OtapPdata = OtlpProtoBytes::ExportLogsRequest(otlp_bytes).into();

        // test can go OtlpProtoBytes -> OtapBatch & back
        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();
        assert!(matches!(otap_batch, OtapArrowRecords::Logs(_)));
        let pdata: OtapPdata = otap_batch.into();

        let otlp_bytes: OtlpProtoBytes = pdata.try_into().unwrap();
        let bytes = match otlp_bytes {
            OtlpProtoBytes::ExportLogsRequest(bytes) => bytes,
            _ => panic!("unexpected otlp bytes pdata variant"),
        };

        let result = ExportLogsServiceRequest::decode(bytes.as_ref()).unwrap();
        assert_eq!(otlp_service_req, result);
    }

    fn roundtrip_otlp_otap_traces(otlp_service_req: ExportTraceServiceRequest) {
        let mut otlp_bytes = vec![];
        otlp_service_req.encode(&mut otlp_bytes).unwrap();
        let pdata: OtapPdata = OtlpProtoBytes::ExportTracesRequest(otlp_bytes).into();

        // test can go OtlpBytes -> OtapBatch & back
        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();
        assert!(matches!(otap_batch, OtapArrowRecords::Traces(_)));
        let pdata: OtapPdata = otap_batch.into();

        let otlp_bytes: OtlpProtoBytes = pdata.try_into().unwrap();
        let bytes = match otlp_bytes {
            OtlpProtoBytes::ExportTracesRequest(bytes) => bytes,
            _ => panic!("unexpected otlp bytes pdata variant"),
        };

        let result = ExportTraceServiceRequest::decode(bytes.as_ref()).unwrap();
        assert_eq!(otlp_service_req, result);
    }

    #[test]
    fn test_otlp_otap_logs_roundtrip() {
        // test to ensure the correct attributes are assigned to the correct log message after
        // roundtrip encoding/decoding

        let otlp_service_req = ExportLogsServiceRequest::new(vec![
            ResourceLogs::build(Resource {
                attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val1"))],
                ..Default::default()
            })
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope {
                    attributes: vec![KeyValue::new("scope_key", AnyValue::new_string("val1"))],
                    ..Default::default()
                })
                .log_records(vec![
                    LogRecord::build(1u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                        .finish(),
                    LogRecord::build(2u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                        .finish(),
                    LogRecord::build(3u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val3"))])
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
            ResourceLogs::build(Resource {
                attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val2"))],
                ..Default::default()
            })
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope {
                    name: "Scope2".into(),
                    attributes: vec![KeyValue::new("scope_key", AnyValue::new_string("val2"))],
                    ..Default::default()
                })
                .log_records(vec![
                    LogRecord::build(4u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val4"))])
                        .finish(),
                    LogRecord::build(5u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val5"))])
                        .finish(),
                ])
                .finish(),
                ScopeLogs::build(InstrumentationScope {
                    attributes: vec![KeyValue::new("scope_key", AnyValue::new_string("val3"))],
                    ..Default::default()
                })
                .log_records(vec![
                    LogRecord::build(6u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val6"))])
                        .finish(),
                    LogRecord::build(7u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val7"))])
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
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
            ResourceLogs::build(Resource {
                attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val"))],
                ..Default::default()
            })
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope::build("scope1").attributes(vec![
                    KeyValue::new("scope_key", AnyValue::new_string("val")),
                ]))
                .log_records(vec![
                    // Add some logs with repeated attributes
                    LogRecord::build(1u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                        .finish(),
                    LogRecord::build(2u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                        .finish(),
                    LogRecord::build(3u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
            // also add some scopes and resources where the attributes repeat ...
            ResourceLogs::build(Resource {
                attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val"))],
                ..Default::default()
            })
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope::build("scope2").attributes(vec![
                    KeyValue::new("scope_key", AnyValue::new_string("val")),
                ]))
                .log_records(vec![
                    LogRecord::build(4u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
            ResourceLogs::build(Resource {
                attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val"))],
                ..Default::default()
            })
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope::build("scope2").attributes(vec![
                    KeyValue::new("scope_key", AnyValue::new_string("val")),
                ]))
                .log_records(vec![
                    LogRecord::build(7u64, SeverityNumber::Info, "")
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
        ]);

        roundtrip_otlp_otap_logs(otlp_service_req);
    }

    #[test]
    fn test_otlp_otap_traces_roundtrip() {
        let otlp_service_req = ExportTraceServiceRequest::new(vec![
            ResourceSpans::build(Resource {
                attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val1"))],
                ..Default::default()
            })
            .scope_spans(vec![
                ScopeSpans::build(InstrumentationScope {
                    attributes: vec![KeyValue::new("scope_key", AnyValue::new_string("val1"))],
                    ..Default::default()
                })
                .spans(vec![
                    Span::build(u128::to_be_bytes(1), u64::to_be_bytes(1), "albert", 1u64)
                        .end_time_unix_nano(4u64)
                        .status(Status::new("status1", StatusCode::Ok))
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                        .links(vec![
                            Link::build(u128::to_be_bytes(10), u64::to_be_bytes(10)).finish(),
                            // this Link's trace_id repeats with the next one. doing this to ensure
                            // their parent IDs don't get interpreted as delta encoded
                            Link::build(u128::to_be_bytes(11), u64::to_be_bytes(11))
                                .attributes(vec![
                                    KeyValue::new("link_key", AnyValue::new_string("val0")),
                                    // repeating the attr here with the next one for same reason
                                    // as repeating link with trace ID
                                    KeyValue::new("link_key_r", AnyValue::new_string("val1")),
                                ])
                                .finish(),
                        ])
                        .events(vec![
                            Event::build("event0", 0u64).finish(),
                            // this event has the repeating name with the next one. doing this to
                            // ensure their parent IDs don't get interpreted as delta encoded
                            Event::build("event1", 1u64)
                                .attributes(vec![
                                    KeyValue::new("evt_key", AnyValue::new_string("val0")),
                                    // repeating the attr here with the next one for same reason
                                    // as repeating link with trace ID
                                    KeyValue::new("evt_key_r", AnyValue::new_string("val1")),
                                ])
                                .finish(),
                        ])
                        .finish(),
                    Span::build(u128::to_be_bytes(2), u64::to_be_bytes(2), "terry", 2u64)
                        .end_time_unix_nano(3u64)
                        .status(Status::new("status1", StatusCode::Ok))
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                        .links(vec![
                            // this Link's trace_id repeats with the previous, and  next one.
                            // doing this to ensure their parent IDs don't get interpreted as
                            // delta encoded
                            Link::build(u128::to_be_bytes(11), u64::to_be_bytes(20))
                                .attributes(vec![
                                    KeyValue::new("link_key_r", AnyValue::new_string("val1")),
                                    KeyValue::new("link_key", AnyValue::new_string("val2")),
                                ])
                                .finish(),
                        ])
                        .events(vec![
                            // this event has the repeating name with the next one and previous one
                            // doing this to ensure their parent IDs don't get interpreted as
                            // delta encoded
                            Event::build("event1", 1u64)
                                .attributes(vec![
                                    KeyValue::new("evt_key_r", AnyValue::new_string("val1")),
                                    KeyValue::new("evt_key", AnyValue::new_string("val2")),
                                ])
                                .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
            ResourceSpans::build(Resource {
                attributes: vec![KeyValue::new("res_key", AnyValue::new_string("val2"))],
                ..Default::default()
            })
            .scope_spans(vec![
                ScopeSpans::build(InstrumentationScope {
                    attributes: vec![KeyValue::new("scope_key", AnyValue::new_string("val3"))],
                    ..Default::default()
                })
                .spans(vec![
                    Span::build(u128::to_be_bytes(3), u64::to_be_bytes(3), "albert", 3u64)
                        .end_time_unix_nano(4u64)
                        .status(Status::new("status1", StatusCode::Ok))
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                        .links(vec![
                            // this Link's trace_id repeats with the previous one. do this to ensure they
                            // don't get interpreted as delta encoded
                            Link::build(u128::to_be_bytes(11), u64::to_be_bytes(30)).finish(),
                            Link::build(u128::to_be_bytes(31), u64::to_be_bytes(31)).finish(),
                        ])
                        .events(vec![
                            // this event has the repeating name with the previous one. doing this
                            // to ensure their parent IDs don't get interpreted as delta encoded
                            Event::build("event1", 2u64).finish(),
                        ])
                        .finish(),
                    Span::build(u128::to_be_bytes(4), u64::to_be_bytes(4), "terry", 4u64)
                        .end_time_unix_nano(5u64)
                        .status(Status::new("status1", StatusCode::Ok))
                        .attributes(vec![KeyValue::new("key", AnyValue::new_string("val4"))])
                        .links(vec![
                            Link::build(u128::to_be_bytes(40), u64::to_be_bytes(40)).finish(),
                        ])
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
        ]);

        roundtrip_otlp_otap_traces(otlp_service_req);
    }

    #[test]
    fn test_signal_type() {
        // Test signal_type for OtlpProtoBytes variants
        let logs_bytes = OtlpProtoBytes::ExportLogsRequest(vec![]);
        let metrics_bytes = OtlpProtoBytes::ExportMetricsRequest(vec![]);
        let traces_bytes = OtlpProtoBytes::ExportTracesRequest(vec![]);

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

        // Test signal_type for OtapArrowBytes variants
        let logs_bytes = OtapArrowBytes::ArrowLogs(Default::default());
        let metrics_bytes = OtapArrowBytes::ArrowMetrics(Default::default());
        let traces_bytes = OtapArrowBytes::ArrowTraces(Default::default());

        assert_eq!(logs_bytes.signal_type(), SignalType::Logs);
        assert_eq!(metrics_bytes.signal_type(), SignalType::Metrics);
        assert_eq!(traces_bytes.signal_type(), SignalType::Traces);

        // Test signal_type for OtapPdata variants
        let pdata_logs = OtapPdata::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(vec![]));
        let pdata_metrics =
            OtapPdata::OtapArrowRecords(OtapArrowRecords::Metrics(Default::default()));
        let pdata_traces =
            OtapPdata::OtapArrowBytes(OtapArrowBytes::ArrowTraces(Default::default()));

        assert_eq!(pdata_logs.signal_type(), SignalType::Logs);
        assert_eq!(pdata_metrics.signal_type(), SignalType::Metrics);
        assert_eq!(pdata_traces.signal_type(), SignalType::Traces);
    }

    #[test]
    fn test_num_rows_helper() {
        // Test with logs data containing 2 log records
        let otlp_service_req = ExportLogsServiceRequest::new(vec![
            ResourceLogs::build(Resource::default())
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::default())
                        .log_records(vec![
                            LogRecord::build(1u64, SeverityNumber::Info, "event1")
                                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val1"))])
                                .finish(),
                            LogRecord::build(2u64, SeverityNumber::Info, "event2")
                                .attributes(vec![KeyValue::new("key", AnyValue::new_string("val2"))])
                                .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ]);
        let mut otlp_bytes = vec![];
        otlp_service_req.encode(&mut otlp_bytes).unwrap();
        
        let pdata: OtapPdata = OtlpProtoBytes::ExportLogsRequest(otlp_bytes).into();
        assert_eq!(num_rows(&pdata), 2);
    }
}
