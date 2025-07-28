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
//! ```no_run
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

use otap_df_pdata_views::otlp::bytes::logs::RawLogsData;
use otel_arrow_rust::otap::{OtapArrowRecords, from_record_messages};
use otel_arrow_rust::otlp::{logs::logs_from, metrics::metrics_from, traces::traces_from};
use otel_arrow_rust::{Consumer, Producer};
use prost::{EncodeError, Message};

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

    fn try_from(otap_batch: OtapArrowRecords) -> Result<Self, Self::Error> {
        let mut producer = Producer::new();
        let bar = producer
            .produce_bar(&otap_batch)
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
mod test {
    use super::*;
    use otel_arrow_rust::{
        otap::OtapArrowRecords,
        proto::opentelemetry::{
            collector::logs::v1::ExportLogsServiceRequest,
            common::v1::{AnyValue, InstrumentationScope, KeyValue},
            logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
            resource::v1::Resource,
        },
    };

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

    // TODO add additional tests for converting between metrics & traces
    // once we have the ability to convert between OTLP bytes -> OTAP for
    // these signal types
    // https://github.com/open-telemetry/otel-arrow/issues/768
}
