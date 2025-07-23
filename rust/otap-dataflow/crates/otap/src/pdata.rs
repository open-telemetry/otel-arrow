// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// TODO revisit whether we actually need to allow this ...
#![allow(dead_code)]

use std::cell::RefCell;

use otap_df_pdata_views::otlp::bytes::logs::RawLogsData;
use otel_arrow_rust::otap::OtapBatch;

use crate::{encoder::encode_logs_otap_batch, grpc::OTAPData};

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

    /// 
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

impl From<OtapPdata> for OtapBatch {
    fn from(value: OtapPdata) -> Self {
        match value {
            OtapPdata::OTAPData(otap_data) => {
                todo!()
            },
            OtapPdata::OtapBatch(otap_batch) => {
                otap_batch
            },
            OtapPdata::OtlpBytes(otlp_bytes) => {
                otlp_bytes.into()
            }
        }
    }
}

impl From<OtapPdata> for OtlpProtoBytes {
    fn from(value: OtapPdata) -> Self {
        match value {
            OtapPdata::OTAPData(otap_data) => {
                todo!()
            },
            OtapPdata::OtapBatch(otap_batch) => {
                todo!()
            },
            OtapPdata::OtlpBytes(otlp_bytes) => {
                otlp_bytes
            }
        }
    }
}

impl From<OtapPdata> for OTAPData {
    fn from(value: OtapPdata) -> Self {
        match value {
            OtapPdata::OTAPData(otap_data) => {
                otap_data
            },
            OtapPdata::OtapBatch(otap_batch) => {
                todo!()
            },
            OtapPdata::OtlpBytes(otlp_bytes) => {
                todo!()
            }
        }
    }
}


impl From<OtlpProtoBytes> for OtapBatch {
    fn from(value: OtlpProtoBytes) -> Self {
        match value {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                let logs_data_view = RawLogsData::new(&bytes);
                let otap_batch = encode_logs_otap_batch(&logs_data_view).unwrap();
                otap_batch
            }
            _ => {
                // TODO add conversions when we support 
                todo!()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use otel_arrow_rust::otap::OtapBatch;

    use crate::pdata::OtapPdata;

    #[test]
    fn test_conversions() {
        let otap_batch: OtapBatch = {
            todo!()
        };

        let otap_pdata: OtapPdata = otap_batch.into();
    }
}
