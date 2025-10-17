// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the filter processor
//!

use otel_arrow_rust::otap::filter::LogFilter;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // ToDo: add metrics and spans
    logs: LogFilter,
}

impl Config {
    pub fn new(logs: LogFilter) -> Self {
        Self { logs }
    }

    #[must_use]
    pub fn log_filters(&self) -> &LogFilter {
        &self.logs
    }
}

#[cfg(test)]
mod tests {

    use crate::pdata::{OtapPayload, OtlpProtoBytes};
    use otel_arrow_rust::otap::OtapArrowRecords;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_rust::proto::opentelemetry::{
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
    };
    use prost::Message as _;
    #[test]
    fn test_create_otap_batch() {
        let logs_data = LogsData::new(vec![
            // ResourceLogs for prod/checkout-service
            ResourceLogs::build(Resource::build(vec![
                KeyValue::new("version", AnyValue::new_string("2.0")),
                KeyValue::new("service.name", AnyValue::new_string("checkout-service")),
                KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
                KeyValue::new("service.instance.number", AnyValue::new_int(42)),
                KeyValue::new("deployment.environment", AnyValue::new_string("prod")),
                KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
                KeyValue::new("host.uptime_sec", AnyValue::new_int(86_400)),
                KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
                KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
                KeyValue::new("process.pid", AnyValue::new_int(12345)),
                KeyValue::new("team", AnyValue::new_string("payments")),
                KeyValue::new("telemetry.sdk.language", AnyValue::new_string("rust")),
            ]))
            .scope_logs(vec![
                ScopeLogs::build(
                    InstrumentationScope::build("library")
                        .version("scopev1")
                        .finish(),
                )
                .log_records(vec![
                    // WARN: included by severity, then excluded (retryable/body)
                    LogRecord::build(2_200_000_000u64, SeverityNumber::Warn, "event")
                        .attributes(vec![
                            KeyValue::new("http.method", AnyValue::new_string("POST")),
                            KeyValue::new("http.route", AnyValue::new_string("/api/checkout")),
                            KeyValue::new("http.status_code", AnyValue::new_int(429)),
                            KeyValue::new("retryable", AnyValue::new_bool(true)),
                            KeyValue::new("backoff_ms", AnyValue::new_int(200)),
                            KeyValue::new("throttle_factor", AnyValue::new_double(1.5)),
                        ])
                        .severity_text("WARN")
                        .body(AnyValue::new_string("rate limited"))
                        .finish(),
                    // ERROR: included (prod, severity >= WARN), not excluded
                    LogRecord::build(2_300_000_000u64, SeverityNumber::Error, "event")
                        .attributes(vec![
                            KeyValue::new("exception.type", AnyValue::new_string("io::Error")),
                            KeyValue::new(
                                "exception.message",
                                AnyValue::new_string("connection reset by peer"),
                            ),
                            KeyValue::new(
                                "exception.stacktrace",
                                AnyValue::new_string("...stack..."),
                            ),
                            KeyValue::new("peer.address", AnyValue::new_string("10.42.0.7:5432")),
                            KeyValue::new("peer.port", AnyValue::new_int(5432)),
                            KeyValue::new("peer.tls", AnyValue::new_bool(true)),
                            KeyValue::new("bytes_sent", AnyValue::new_int(0)),
                            KeyValue::new("jitter", AnyValue::new_double(0.003)),
                        ])
                        .severity_text("ERROR")
                        .body(AnyValue::new_string("failed to write to socket"))
                        .finish(),
                    // WARN: included by severity, then excluded (component=db)
                    LogRecord::build(2_600_000_000u64, SeverityNumber::Warn, "event")
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("db")),
                            KeyValue::new(
                                "query",
                                AnyValue::new_string(
                                    "UPDATE inventory SET count = count - 1 WHERE sku='ABC123'",
                                ),
                            ),
                            KeyValue::new("rows_affected", AnyValue::new_int(1)),
                            KeyValue::new("success", AnyValue::new_bool(true)),
                        ])
                        .severity_text("WARN")
                        .body(AnyValue::new_string("inventory updated"))
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
            // ResourceLogs for staging/inventory-service (will not pass include)
            ResourceLogs::build(Resource::build(vec![
                KeyValue::new("version", AnyValue::new_string("2.0")),
                KeyValue::new("service.name", AnyValue::new_string("inventory-service")),
                KeyValue::new("service.version", AnyValue::new_string("0.9.1")),
                KeyValue::new("service.instance.number", AnyValue::new_int(7)),
                KeyValue::new("deployment.environment", AnyValue::new_string("staging")),
                KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                KeyValue::new("host.cpu_cores", AnyValue::new_int(4)),
                KeyValue::new("host.uptime_sec", AnyValue::new_int(12_345)),
                KeyValue::new("sampling.rate", AnyValue::new_double(0.10)),
                KeyValue::new("debug.enabled", AnyValue::new_bool(true)),
                KeyValue::new("process.pid", AnyValue::new_int(22222)),
                KeyValue::new("team", AnyValue::new_string("inventory")),
            ]))
            .scope_logs(vec![
                ScopeLogs::build(
                    InstrumentationScope::build("library")
                        .version("scopev2")
                        .finish(),
                )
                .log_records(vec![
                    LogRecord::build(3_000_000_000u64, SeverityNumber::Warn, "event")
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("rest")),
                            KeyValue::new(
                                "http.route",
                                AnyValue::new_string("/api/internal/cache_warm"),
                            ),
                            KeyValue::new("http.status_code", AnyValue::new_int(200)),
                            KeyValue::new("success", AnyValue::new_bool(true)),
                        ])
                        .severity_text("WARN")
                        .body(AnyValue::new_string("warmup complete"))
                        .finish(),
                    LogRecord::build(3_100_000_000u64, SeverityNumber::Info, "event")
                        .attributes(vec![
                            KeyValue::new("event.domain", AnyValue::new_string("ops")),
                            KeyValue::new("message", AnyValue::new_string("heartbeat")),
                            KeyValue::new("uptime_sec", AnyValue::new_int(100)),
                        ])
                        .severity_text("INFO")
                        .body(AnyValue::new_string("heartbeat"))
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
        ]);
        let mut bytes = vec![];
        logs_data.encode(&mut bytes).unwrap();
        let pdata = OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes));
        let otap_batch: OtapArrowRecords = pdata.try_into().unwrap();

        let log_record = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        // println!("{:?}", log_attrs);
        // println!("{:?}", log_attrs.schema());

        let table = arrow::util::pretty::pretty_format_batches_with_schema(
            log_record.schema(),
            &[log_record.clone()],
        )
        .unwrap();
        println!("{table}");
        // let attr_mask = arrow::compute::kernels::cmp::eq(log_attrs.column_by_name("str").unwrap(), &StringArray::new_scalar("keep")).unwrap();
        // let attr_mask_4 = arrow::compute::kernels::cmp::eq(log_attrs.column_by_name("key").unwrap(), &StringArray::new_scalar("b")).unwrap();

        // let attr_mask_2 = arrow::compute::kernels::cmp::eq(log_attrs.column_by_name("key").unwrap(), &StringArray::new_scalar("d")).unwrap();
        // let attr_msk_3 = arrow::compute::kernels::cmp::eq(log_attrs.column_by_name("int").unwrap(), &Int64Array::new_scalar(120)).unwrap();

        // let combined_attr_mask = arrow::compute::and(&attr_mask_2, &attr_msk_3).unwrap();
        // let combined_attr_mask_2 = arrow::compute::and(&attr_mask_4, &attr_mask).unwrap();
        // let combined_attr_mask_3 = arrow::compute::or_kleene(&combined_attr_mask, &combined_attr_mask_2).expect("failed to or");

        // let new_attrs = filter_record_batch(log_attrs, &combined_attr_mask_3).expect("failed to filter");
        // arrow::util::pretty::print_batches(&[new_attrs.clone()]).unwrap();
    }
}
