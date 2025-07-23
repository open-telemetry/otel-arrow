// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// TODO revisit whether we actually need to allow this ...
#![allow(dead_code)]

use otap_df_pdata_views::otlp::bytes::logs::RawLogsData;
use otel_arrow_rust::otap::{OtapBatch, from_record_messages};
use otel_arrow_rust::otlp::{logs::logs_from, metrics::metrics_from, traces::traces_from};
use otel_arrow_rust::{Consumer, Producer};
use prost::{EncodeError, Message};

use crate::{encoder::encode_logs_otap_batch, grpc::OTAPData};

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
}

enum OtlpProtoBytes {
    // TODO revisit the names of these
    ExportLogsRequest(Vec<u8>),
    ExportMetricsRequest(Vec<u8>),
    ExportTracesRequest(Vec<u8>),
}

/// Container for the various representations of the telemetry data
enum OtapPdata {
    /// data is serialized as a protobuf service message for one of the OTLP GRPC services
    OtlpBytes(OtlpProtoBytes),

    /// data is contained in `BatchArrowRecords`, which contain ArrowIPC serialized
    OTAPData(OTAPData),

    /// data is contained in `OtapBatch` which contains Arrow `RecordBatches` for OTAP payload type
    OtapBatch(OtapBatch),
}

impl From<OtapBatch> for OtapPdata {
    fn from(value: OtapBatch) -> Self {
        Self::OtapBatch(value)
    }
}

impl From<OtlpProtoBytes> for OtapPdata {
    fn from(value: OtlpProtoBytes) -> Self {
        Self::OtlpBytes(value)
    }
}

impl From<OTAPData> for OtapPdata {
    fn from(value: OTAPData) -> Self {
        Self::OTAPData(value)
    }
}

impl TryFrom<OtapPdata> for OtapBatch {
    type Error = error::Error;

    fn try_from(value: OtapPdata) -> Result<Self, Self::Error> {
        match value {
            OtapPdata::OTAPData(otap_data) => otap_data.try_into(),
            OtapPdata::OtapBatch(otap_batch) => Ok(otap_batch),
            OtapPdata::OtlpBytes(otlp_bytes) => otlp_bytes.try_into(),
        }
    }
}

impl TryFrom<OtapPdata> for OtlpProtoBytes {
    type Error = error::Error;

    fn try_from(value: OtapPdata) -> Result<Self, Self::Error> {
        match value {
            OtapPdata::OTAPData(otap_data) => otap_data.try_into(),
            OtapPdata::OtapBatch(otap_batch) => otap_batch.try_into(),
            OtapPdata::OtlpBytes(otlp_bytes) => Ok(otlp_bytes),
        }
    }
}

impl TryFrom<OtapPdata> for OTAPData {
    type Error = error::Error;

    fn try_from(value: OtapPdata) -> Result<Self, Self::Error> {
        match value {
            OtapPdata::OTAPData(otap_data) => Ok(otap_data),
            OtapPdata::OtapBatch(otap_batch) => otap_batch.try_into(),
            OtapPdata::OtlpBytes(otlp_bytes) => otlp_bytes.try_into(),
        }
    }
}

impl TryFrom<OtapBatch> for OtlpProtoBytes {
    type Error = error::Error;

    fn try_from(value: OtapBatch) -> Result<Self, Self::Error> {
        let map_otlp_conversion_error =
            |error: otel_arrow_rust::error::Error| error::Error::ConversionError {
                error: format!("error generating OTLP request: {error}"),
            };

        let map_prost_encode_error = |error: EncodeError| error::Error::ConversionError {
            error: format!("error encoding protobuf: {error}"),
        };

        match value {
            OtapBatch::Logs(_) => {
                let export_logs_svc_req = logs_from(value).map_err(map_otlp_conversion_error)?;
                let mut bytes = vec![];
                export_logs_svc_req
                    .encode(&mut bytes)
                    .map_err(map_prost_encode_error)?;
                Ok(Self::ExportLogsRequest(bytes))
            }
            OtapBatch::Metrics(_) => {
                let export_metrics_svc_req =
                    metrics_from(value).map_err(map_otlp_conversion_error)?;
                let mut bytes = vec![];
                export_metrics_svc_req
                    .encode(&mut bytes)
                    .map_err(map_prost_encode_error)?;
                Ok(Self::ExportMetricsRequest(bytes))
            }
            OtapBatch::Traces(_) => {
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

impl TryFrom<OtlpProtoBytes> for OtapBatch {
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
                todo!()
            }
        }
    }
}

impl TryFrom<OtlpProtoBytes> for OTAPData {
    type Error = error::Error;

    fn try_from(value: OtlpProtoBytes) -> Result<Self, Self::Error> {
        let otap_batch: OtapBatch = value.try_into()?;
        otap_batch.try_into()
    }
}

impl TryFrom<OTAPData> for OtlpProtoBytes {
    type Error = error::Error;

    fn try_from(value: OTAPData) -> Result<Self, Self::Error> {
        let otap_batch: OtapBatch = value.try_into()?;
        otap_batch.try_into()
    }
}

impl TryFrom<OTAPData> for OtapBatch {
    type Error = error::Error;

    fn try_from(value: OTAPData) -> Result<Self, Self::Error> {
        let map_error = |error: otel_arrow_rust::error::Error| error::Error::ConversionError {
            error: format!("error decoding BatchArrowRecords: {error}"),
        };
        let mut consumer = Consumer::default();
        match value {
            OTAPData::ArrowLogs(mut bar) => {
                let record_messages = consumer.consume_bar(&mut bar).map_err(map_error)?;
                Ok(Self::Logs(from_record_messages(record_messages)))
            }
            OTAPData::ArrowMetrics(mut bar) => {
                let record_messages = consumer.consume_bar(&mut bar).map_err(map_error)?;
                Ok(Self::Metrics(from_record_messages(record_messages)))
            }
            OTAPData::ArrowTraces(mut bar) => {
                let record_messages = consumer.consume_bar(&mut bar).map_err(map_error)?;
                Ok(Self::Traces(from_record_messages(record_messages)))
            }
        }
    }
}

impl TryFrom<OtapBatch> for OTAPData {
    type Error = error::Error;

    fn try_from(otap_batch: OtapBatch) -> Result<Self, Self::Error> {
        let mut producer = Producer::new();
        let bar = producer.produce_bar(&otap_batch).unwrap();
        Ok(match otap_batch {
            OtapBatch::Logs(_) => Self::ArrowLogs(bar),
            OtapBatch::Metrics(_) => Self::ArrowMetrics(bar),
            OtapBatch::Traces(_) => Self::ArrowTraces(bar),
        })
    }
}

#[cfg(test)]
mod test {
    use otel_arrow_rust::otap::OtapBatch;

    use crate::pdata::OtapPdata;

    #[test]
    fn test_conversions() {
        let otap_batch: OtapBatch = { todo!() };

        let otap_pdata: OtapPdata = otap_batch.into();
    }
}
