// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Registry-derived native inputs selected by benchmark cases, outside timing.

use std::sync::OnceLock;

use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::{
    config::{Config, TrafficConfig},
    semconv_signal::semconv_otlp_logs,
};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::encode::encode_logs_otap_batch;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
use weaver_common::vdir::VirtualDirectoryPath;
use weaver_semconv::v1::{attribute::Examples, group::GroupType};

pub(super) const SYSLOG_REGISTRY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../crates/pdata-codec/src/codecs/syslog/registry"
);

/// Registry-derived input shared by syslog/CEF and the OTLP comparison case.
pub fn syslog_logs(count: usize) -> OtapArrowRecords {
    static TEMPLATES: OnceLock<LogsData> = OnceLock::new();
    let mut logs = TEMPLATES
        .get_or_init(|| load_templates(SYSLOG_REGISTRY))
        .clone();
    let records = &mut logs.resource_logs[0].scope_logs[0].log_records;
    *records = records.iter().cycle().take(count).cloned().collect();
    // Cycle January, April, July, and October after expanding the templates.
    for (index, record) in records.iter_mut().enumerate() {
        const NANOS_PER_DAY: u64 = 86_400_000_000_000;
        record.time_unix_nano += [0, 91, 182, 274][index % 4] * NANOS_PER_DAY;
        record.observed_time_unix_nano = record.time_unix_nano;
    }
    encode_logs_otap_batch(&logs).expect("construct native logs")
}

/// Resolve a local event registry into repeatable log templates, without caching.
pub(super) fn load_templates(registry_path: impl Into<String>) -> LogsData {
    let registry_path = VirtualDirectoryPath::LocalFolder {
        path: registry_path.into(),
    };
    let config = Config::new(TrafficConfig::new(None, None, 1, 0, 0, 1), registry_path);
    let mut registry = config
        .get_registry()
        .expect("resolve local benchmark registry with Weaver")
        .expect("semantic-convention registry");
    registry.groups.sort_by(|a, b| a.name.cmp(&b.name));
    let mut event_count = 0;
    for (index, group) in registry
        .groups
        .iter_mut()
        .filter(|group| group.r#type == GroupType::Event)
        .enumerate()
    {
        group.attributes.sort_by(|a, b| a.name.cmp(&b.name));
        // Singleton examples keep trafficgen's random choice deterministic.
        for attribute in &mut group.attributes {
            match &mut attribute.examples {
                Some(Examples::Strings(values)) => select_example(values, index),
                Some(Examples::Ints(values)) => select_example(values, index),
                Some(Examples::String(_) | Examples::Int(_)) => {}
                _ => panic!(
                    "benchmark registry attribute {} requires string or integer examples",
                    attribute.name
                ),
            }
        }
        event_count += 1;
    }
    assert!(event_count > 0, "benchmark registry must define events");
    let mut logs = semconv_otlp_logs(event_count, &registry);
    for resource in &mut logs.resource_logs {
        for scope in &mut resource.scope_logs {
            for record in &mut scope.log_records {
                record.time_unix_nano = 1_704_164_645_123_456_000;
                record.observed_time_unix_nano = record.time_unix_nano;
            }
        }
    }
    logs
}

fn select_example<T>(values: &mut Vec<T>, index: usize) {
    assert!(!values.is_empty(), "benchmark examples must not be empty");
    let offset = index % values.len();
    values.rotate_left(offset);
    values.truncate(1);
}
