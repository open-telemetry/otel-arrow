// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opentelemetry SDK logs exporter of internal telemetry logs.

use std::time::SystemTime;

use opentelemetry::logs::Severity as SdkSeverity;
use opentelemetry_proto::tonic::common::v1::KeyValue as OtlpKeyValue;
use opentelemetry_proto::tonic::common::v1::{AnyValue, InstrumentationScope};
use opentelemetry_proto::tonic::logs::v1::ResourceLogs as OtlpResourceLogs;
use opentelemetry_proto::tonic::logs::v1::ScopeLogs as OtlpScopeLogs;
use opentelemetry_proto::tonic::logs::v1::{LogRecord, LogsData};
use opentelemetry_proto::tonic::resource::v1::Resource as OtlpResource;
use opentelemetry_sdk::Resource as SdkResource;
use opentelemetry_sdk::logs::LogBatch as SdkLogBatch;
use opentelemetry_sdk::{error::OTelSdkResult, logs::LogExporter};
use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
use prost::Message;
use prost::bytes::BytesMut;

use crate::opentelemetry_client::InternalLogSender;

/// An OpenTelemetry log exporter that sends internal logs to the pipeline engine.
#[derive(Debug)]
pub struct InternalLogsExporter {
    internal_logs_sender: InternalLogSender,
    sdk_resource: Option<SdkResource>,
}

impl LogExporter for InternalLogsExporter {
    fn export(&self, batch: SdkLogBatch<'_>) -> impl Future<Output = OTelSdkResult> + Send {
        let otap_data = self.convert_sdk_logs_batch_to_otap_data(batch);
        let internal_logs_sender = self.internal_logs_sender.clone();

        async move {
            // Push the logs_data to the internal telemetry receiver though its channel.
            // It can be a different object to be sent instead of the proto LogsData.
            let _ = internal_logs_sender.try_send(otap_data);
            // Ignore if there is an error as there might not be any receiver configured to receive internal telemetry data.
            Ok(())
        }
    }

    fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource) {
        self.sdk_resource = Some(resource.clone());
    }
}

impl InternalLogsExporter {
    /// Creates a new instance of the InternalLogsExporter.
    /// internal_logs_sender: The channel sender to send internal logs to the pipeline engine.
    #[must_use]
    pub fn new(internal_logs_sender: InternalLogSender) -> Self {
        InternalLogsExporter {
            internal_logs_sender,
            sdk_resource: None,
        }
    }

    /// Converts an SDK LogBatch into OTLP LogsData format.
    fn to_otlp_logs_data(&self, batch: SdkLogBatch<'_>) -> LogsData {
        let mut scope_logs = Vec::new();

        for (log_record, instrumentation_scope) in batch.iter() {
            let time_unix_nano: u64 = log_record
                .timestamp()
                .unwrap_or_else(SystemTime::now)
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            let observed_time_unix_nano: u64 = log_record
                .observed_timestamp()
                .unwrap_or_else(SystemTime::now)
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;

            let severity_number = log_record.severity_number().unwrap_or(SdkSeverity::Info) as i32;
            let severity_text = log_record.severity_text().unwrap_or("INFO").to_string();
            let body: Option<AnyValue> =
                log_record.body().map(Self::convert_sdk_any_value_to_proto);

            let event_name: String = log_record.event_name().unwrap_or("").to_string();

            let attributes: Vec<OtlpKeyValue> =
                Self::convert_sdk_attributes_to_proto(log_record.attributes_iter());

            // Conversion logic from SdkLogRecord to LogRecord:
            let scope_logs_instance = OtlpScopeLogs {
                scope: Some(InstrumentationScope {
                    name: instrumentation_scope.name().into(),
                    version: instrumentation_scope.version().unwrap_or_default().into(),
                    attributes: attributes.clone(),
                    dropped_attributes_count: 0,
                }),
                log_records: vec![LogRecord {
                    time_unix_nano,
                    observed_time_unix_nano,
                    severity_number,
                    severity_text,
                    body,
                    attributes,
                    dropped_attributes_count: 0,
                    flags: 0,
                    trace_id: vec![],
                    span_id: vec![],
                    event_name,
                }],
                schema_url: String::new(),
            };
            scope_logs.push(scope_logs_instance)
        }

        let otlp_resource: Option<OtlpResource> =
            Self::to_otlp_resource(self.sdk_resource.as_ref());

        let resource_logs = OtlpResourceLogs {
            resource: otlp_resource,
            scope_logs,
            schema_url: String::new(),
        };
        LogsData {
            resource_logs: vec![resource_logs],
        }
    }

    // Helper function to convert OpenTelemetry AnyValue to protobuf AnyValue
    fn convert_sdk_any_value_to_proto(value: &opentelemetry::logs::AnyValue) -> AnyValue {
        use opentelemetry_proto::tonic::common::v1::any_value::Value;

        let proto_value = match value {
            opentelemetry::logs::AnyValue::String(s) => Value::StringValue(s.to_string()),
            opentelemetry::logs::AnyValue::Int(i) => Value::IntValue(*i),
            opentelemetry::logs::AnyValue::Double(d) => Value::DoubleValue(*d),
            opentelemetry::logs::AnyValue::Boolean(b) => Value::BoolValue(*b),
            opentelemetry::logs::AnyValue::Bytes(bytes) => Value::BytesValue(*bytes.clone()),
            opentelemetry::logs::AnyValue::ListAny(list) => {
                Value::ArrayValue(opentelemetry_proto::tonic::common::v1::ArrayValue {
                    values: list
                        .iter()
                        .map(Self::convert_sdk_any_value_to_proto)
                        .collect(),
                })
            }
            _ => {
                // TODO: Complete.
                // Handle any other variants by defaulting to an empty string
                Value::StringValue(String::new())
            }
        };

        AnyValue {
            value: Some(proto_value),
        }
    }

    /// Helper function to convert SDK OpenTelemetry Value to protobuf AnyValue
    fn convert_value_to_proto(value: &opentelemetry::Value) -> AnyValue {
        use opentelemetry_proto::tonic::common::v1::any_value::Value as ProtoValue;

        let proto_value = match value {
            opentelemetry::Value::Bool(b) => ProtoValue::BoolValue(*b),
            opentelemetry::Value::I64(i) => ProtoValue::IntValue(*i),
            opentelemetry::Value::F64(f) => ProtoValue::DoubleValue(*f),
            opentelemetry::Value::String(s) => ProtoValue::StringValue(s.to_string()),
            opentelemetry::Value::Array(arr) => {
                let values = match arr {
                    opentelemetry::Array::Bool(v) => v
                        .iter()
                        .map(|b| AnyValue {
                            value: Some(ProtoValue::BoolValue(*b)),
                        })
                        .collect(),
                    opentelemetry::Array::I64(v) => v
                        .iter()
                        .map(|i| AnyValue {
                            value: Some(ProtoValue::IntValue(*i)),
                        })
                        .collect(),
                    opentelemetry::Array::F64(v) => v
                        .iter()
                        .map(|f| AnyValue {
                            value: Some(ProtoValue::DoubleValue(*f)),
                        })
                        .collect(),
                    opentelemetry::Array::String(v) => v
                        .iter()
                        .map(|s| AnyValue {
                            value: Some(ProtoValue::StringValue(s.to_string())),
                        })
                        .collect(),
                    _ => vec![],
                };
                ProtoValue::ArrayValue(opentelemetry_proto::tonic::common::v1::ArrayValue {
                    values,
                })
            }
            _ => ProtoValue::StringValue(String::new()),
        };

        AnyValue {
            value: Some(proto_value),
        }
    }

    /// Converts an SDK Resource into OTLP Resource format.
    fn to_otlp_resource(resource: Option<&SdkResource>) -> Option<OtlpResource> {
        resource.map(|res| {
            let attributes = res
                .iter()
                .map(
                    |(key, value)| opentelemetry_proto::tonic::common::v1::KeyValue {
                        key: key.as_str().to_string(),
                        value: Some(Self::convert_value_to_proto(value)),
                    },
                )
                .collect();

            OtlpResource {
                attributes,
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }
        })
    }

    /// Helper function to convert SDK attributes iterator to protobuf KeyValue vector
    fn convert_sdk_attributes_to_proto<'a, I>(attributes_iter: I) -> Vec<OtlpKeyValue>
    where
        I: Iterator<Item = &'a (opentelemetry::Key, opentelemetry::logs::AnyValue)>,
    {
        attributes_iter
            .map(|(key, value)| OtlpKeyValue {
                key: key.as_str().to_string(),
                value: Some(Self::convert_sdk_any_value_to_proto(value)),
            })
            .collect()
    }

    /// Converts an SDK LogBatch into OTAP payload format.
    fn convert_sdk_logs_batch_to_otap_data(&self, batch: SdkLogBatch<'_>) -> OtapPayload {
        let logs_data = self.to_otlp_logs_data(batch);

        let mut bytes = BytesMut::new();
        logs_data
            .encode(&mut bytes)
            .expect("Failed to encode LogsData");

        let otlp_bytes = OtlpProtoBytes::ExportLogsRequest(bytes.into());
        OtapPayload::OtlpBytes(otlp_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use opentelemetry::KeyValue as SdkKeyValue;
    use opentelemetry::logs::{AnyValue, LogRecord, Logger, LoggerProvider, Severity};
    use opentelemetry_sdk::Resource as SdkResource;
    use opentelemetry_sdk::logs::SdkLoggerProvider;

    #[test]
    fn test_internal_logs_exporter_export() {
        let (internal_logs_sender, internal_logs_receiver) = flume::unbounded();
        let mut exporter = InternalLogsExporter::new(internal_logs_sender);

        let sdk_resource = SdkResource::builder()
            .with_attributes(vec![
                SdkKeyValue::new("service.name", "test-service"),
                SdkKeyValue::new("service.version", "1.0.0"),
            ])
            .build();

        exporter.set_resource(&sdk_resource);

        // Create a logger provider and logger
        let logger_provider = SdkLoggerProvider::builder()
            .with_resource(sdk_resource.clone())
            .with_batch_exporter(exporter)
            .build();

        let logger = logger_provider.logger("test-logger");

        // Emit a log record
        let mut log_record = logger.create_log_record();
        log_record.set_body(AnyValue::from("Test log message"));
        log_record.set_severity_number(Severity::Info);
        log_record.set_severity_text("INFO");
        log_record.set_event_name("test_event");
        logger.emit(log_record);

        // Receive the internal log payload
        let received_payload = internal_logs_receiver
            .recv()
            .expect("Failed to receive internal log payload");
        match received_payload {
            OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) => {
                let logs_data =
                    LogsData::decode(bytes.as_ref()).expect("Failed to decode LogsData");
                assert_eq!(logs_data.resource_logs.len(), 1);
                let resource_logs = &logs_data.resource_logs[0];
                assert_eq!(resource_logs.scope_logs.len(), 1);
                let scope_logs = &resource_logs.scope_logs[0];
                assert_eq!(scope_logs.log_records.len(), 1);
                let log_record = &scope_logs.log_records[0];
                assert_eq!(log_record.severity_text, "INFO");
                assert_eq!(log_record.event_name, "test_event");
                match log_record.body.as_ref() {
                    Some(any_value) => match any_value.value.as_ref() {
                        Some(
                            opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(
                                s,
                            ),
                        ) => {
                            assert_eq!(s, "Test log message");
                        }
                        _ => panic!("Unexpected body value type"),
                    },
                    _ => panic!("Unexpected body value type"),
                }
            }
            _ => panic!("Unexpected OtapPayload variant"),
        }
    }
}
