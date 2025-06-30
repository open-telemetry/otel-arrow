// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the definitions of the traits for various OTLP messages.

pub mod common;
pub mod logs;
pub mod resource;

/// helpers for writing benchmarks against view implementations
#[cfg(feature = "bench")]
pub mod bench_helpers {
    use super::common::{AnyValueView, AttributeView, InstrumentationScopeView, ValueType};
    use super::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};
    use super::resource::ResourceView;
    use std::hint::black_box;

    /// noop visit `AnyValueView``
    pub fn visit_any_value<'a, T>(any_value_view_impl: T)
    where
        T: AnyValueView<'a>,
    {
        match any_value_view_impl.value_type() {
            ValueType::Array => {
                for val in any_value_view_impl.as_array().expect("value to be array") {
                    visit_any_value(val);
                }
            }
            ValueType::Bool => {
                let _ = black_box(any_value_view_impl.as_bool().expect("value to be bool"));
            }
            ValueType::Bytes => {
                let _ = black_box(any_value_view_impl.as_bytes().expect("value to be bytes"));
            }
            ValueType::Double => {
                let _ = black_box(any_value_view_impl.as_double().expect("value to be double"));
            }
            ValueType::Int64 => {
                let _ = black_box(any_value_view_impl.as_int64().expect("value to be int"));
            }
            ValueType::KeyValueList => {
                for kv in any_value_view_impl.as_kvlist().expect("value to be kvlist") {
                    visit_attribute(kv);
                }
            }
            ValueType::String => {
                let _ = black_box(any_value_view_impl.as_string().expect("value ot be string"));
            }
            ValueType::Empty => {}
        }
    }

    /// Noop visit `AttributeView``
    pub fn visit_attribute<T>(attribute_view_impl: T)
    where
        T: AttributeView,
    {
        let _ = black_box(attribute_view_impl.key());
        if let Some(val) = attribute_view_impl.value() {
            visit_any_value(val);
        }
    }

    /// noop visit every field of LogRecord
    pub fn visit_logs_record<T>(log_record: &T)
    where
        T: LogRecordView,
    {
        let _ = black_box(log_record.time_unix_nano());
        let _ = black_box(log_record.observed_time_unix_nano());
        let _ = black_box(log_record.severity_number());
        let _ = black_box(log_record.severity_text());

        let _ = black_box(log_record.trace_id());
        let _ = black_box(log_record.span_id());
        let _ = black_box(log_record.body().map(|b| b.value_type()));
        for kv in log_record.attributes() {
            visit_attribute(kv);
        }

        let _ = black_box(log_record.dropped_attributes_count());
        let _ = black_box(log_record.flags());
    }

    /// noop visit every field in the logs data
    pub fn visit_logs_data<T>(logs_view_impl: &T)
    where
        T: LogsDataView,
    {
        for resource_logs in logs_view_impl.resources() {
            for scope_logs in resource_logs.scopes() {
                for log_record in scope_logs.log_records() {
                    visit_logs_record(&log_record);
                }

                if let Some(scope) = scope_logs.scope() {
                    let _ = black_box(scope.name());
                    let _ = black_box(scope.version());
                    for kv in scope.attributes() {
                        visit_attribute(kv);
                    }
                    let _ = black_box(scope.dropped_attributes_count());
                }
                let _ = black_box(scope_logs.schema_url());
            }
            let _ = black_box(resource_logs.schema_url());
            if let Some(resource) = resource_logs.resource() {
                for kv in resource.attributes() {
                    visit_attribute(kv);
                }

                let _ = black_box(resource.dropped_attributes_count());
            }
        }
    }

    /// Noop visit LogVisit (in order of protobuf fields)
    pub fn visit_logs_record_ordered<T>(log_record: &T)
    where
        T: LogRecordView,
    {
        let _ = black_box(log_record.time_unix_nano());
        let _ = black_box(log_record.severity_number());
        let _ = black_box(log_record.severity_text());
        let _ = black_box(log_record.body().map(|b| b.value_type()));
        for kv in log_record.attributes() {
            visit_attribute(kv);
        }
        let _ = black_box(log_record.dropped_attributes_count());
        let _ = black_box(log_record.flags());
        let _ = black_box(log_record.trace_id());
        let _ = black_box(log_record.span_id());
        let _ = black_box(log_record.observed_time_unix_nano());
    }

    /// Noop visit visit every field in the logs data (in order of protobuf fields)
    pub fn visit_logs_data_ordered<T>(logs_view_impl: &T)
    where
        T: LogsDataView,
    {
        for resource_logs in logs_view_impl.resources() {
            if let Some(resource) = resource_logs.resource() {
                for kv in resource.attributes() {
                    visit_attribute(kv);
                }

                let _ = black_box(resource.dropped_attributes_count());
            }

            for scope_logs in resource_logs.scopes() {
                if let Some(scope) = scope_logs.scope() {
                    let _ = black_box(scope.name());
                    let _ = black_box(scope.version());
                    for kv in scope.attributes() {
                        visit_attribute(kv);
                    }
                    let _ = black_box(scope.dropped_attributes_count());
                }

                for log_record in scope_logs.log_records() {
                    visit_logs_record_ordered(&log_record);
                }

                let _ = black_box(scope_logs.schema_url());
            }
            let _ = black_box(resource_logs.schema_url());
        }
    }
}
