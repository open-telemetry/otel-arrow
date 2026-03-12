// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::pdata::OtapPdata;
use bytes::BytesMut;
use otap_df_engine::error::Error;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::proto::OtlpProtoMessage;
use prost::Message;

impl TryFrom<OtlpProtoMessage> for OtapPdata {
    type Error = Error;

    fn try_from(value: OtlpProtoMessage) -> Result<Self, Self::Error> {
        let mut bytes = BytesMut::new();
        Ok(match value {
            OtlpProtoMessage::Logs(logs_data) => {
                logs_data.encode(&mut bytes)?;
                OtapPdata::new_todo_context(
                    OtlpProtoBytes::ExportLogsRequest(bytes.freeze()).into(),
                )
            }
            OtlpProtoMessage::Metrics(metrics_data) => {
                metrics_data.encode(&mut bytes)?;
                OtapPdata::new_todo_context(
                    OtlpProtoBytes::ExportMetricsRequest(bytes.freeze()).into(),
                )
            }
            OtlpProtoMessage::Traces(trace_data) => {
                trace_data.encode(&mut bytes)?;
                OtapPdata::new_todo_context(
                    OtlpProtoBytes::ExportTracesRequest(bytes.freeze()).into(),
                )
            }
        })
    }
}
