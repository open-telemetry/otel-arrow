// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

mod create_array;

pub(crate) use create_array::{create_record_batch, create_test_schema};

use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::{
    common::v1::{AnyValue, InstrumentationScope, KeyValue},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
    resource::v1::Resource,
};

/// Create minimal test data
#[must_use]
pub fn create_test_logs() -> ExportLogsServiceRequest {
    ExportLogsServiceRequest::new(vec![ResourceLogs::new(
        Resource::default(),
        vec![ScopeLogs::new(
            InstrumentationScope::default(),
            vec![
                LogRecord::build()
                    .time_unix_nano(2u64)
                    .severity_number(SeverityNumber::Info)
                    .event_name("event")
                    .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                    .finish(),
            ],
        )],
    )])
}
