// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otap_df_otap::encoder::encode_logs_otap_batch;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otel_arrow_rust::proto::opentelemetry::logs::v1::{
    LogRecord, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
use prost::Message;

/// Generate a batch of logs for testing, development and examples
pub fn generate_logs_batch(batch_size: usize, offset: usize) -> OtapArrowRecords {
    let logs = ((0 + offset)..(batch_size + offset))
        .map(|i| {
            let severity_number = SeverityNumber::try_from(((i % 4) * 4 + 1) as i32).unwrap();
            let severity_text = severity_number
                .as_str_name()
                .split("_")
                .skip(2)
                .next()
                .unwrap();
            let event_name = format!("event {}", i);

            let attrs = vec![
                KeyValue::new("k8s.pod", AnyValue::new_string(format!("my-app-{}", i % 4))),
                KeyValue::new(
                    "k8s.ns",
                    AnyValue::new_string(format!(
                        "{}",
                        match i % 3 {
                            0 => "dev",
                            1 => "staging",
                            _ => "prod",
                        }
                    )),
                ),
                KeyValue::new(
                    "region",
                    AnyValue::new_string(if i > batch_size / 2 {
                        "us-east-1"
                    } else {
                        "us-west-1"
                    }),
                ),
            ];

            LogRecord::build(i as u64, severity_number, event_name)
                .severity_text(severity_text)
                .attributes(attrs)
                .finish()
        })
        .collect::<Vec<_>>();

    let log_req = ExportLogsServiceRequest::new(vec![
        ResourceLogs::build(Resource::default())
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope::default())
                    .log_records(logs)
                    .finish(),
            ])
            .finish(),
    ]);

    let mut bytes = vec![];
    log_req.encode(&mut bytes).expect("can encode to vec");
    let logs_view = RawLogsData::new(&bytes);
    let otap_batch = encode_logs_otap_batch(&logs_view).expect("can convert to OTAP");

    otap_batch
}
