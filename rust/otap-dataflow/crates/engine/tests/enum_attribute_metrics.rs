// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end tests for the datapoint-level enum-attribute mechanism for metric
//! sets: the `AttributeEnum` derive, `#[attribute_set(measurement)]`,
//! `#[metric_set(registration_attributes = ..)]` / `#[metric_set(measurement_attributes = ..)]`,
//! dense mixed-radix bucketing, and the export path that carries per-datapoint
//! attributes through the registry.

#![allow(missing_docs)]

use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otap_df_pdata::proto::opentelemetry::logs::v1::ResourceLogs;
use otap_df_pdata::proto::opentelemetry::metrics::v1::{metric, number_data_point};
use otap_df_telemetry::attributes::{AttributeEnum, MeasurementAttributeSet};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::otlp::MetricsOtlpEncoder;
use otap_df_telemetry::metrics::{
    MeasurementMetricSet, MeasurementMetricSetHandler, MetricSet, MetricSetRegistrar,
    RegistrationMetricSetHandler,
};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use prost::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum Signal {
    #[attribute_value = "log-records"]
    Logs,
    Metrics,
    Traces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum Outcome {
    Dropped,
    Expired,
}

/// Measurement fields use their field names as keys without annotations.
#[attribute_set(name = "test.implicit.attrs", measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ImplicitMeasurementAttributes {
    pub signal: Signal,
    pub outcome: Outcome,
}

/// An explicit key override changes the exported key from `outcome` to `result`.
#[attribute_set(name = "test.explicit.attrs", measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ExplicitMeasurementAttributes {
    #[attribute_key = "result"]
    pub outcome: Outcome,
}

/// Registration fields also use their field names as keys without annotations.
#[attribute_set(name = "test.registration.attrs")]
#[derive(Debug, Clone, Copy)]
pub struct RegistrationAttributes {
    #[attribute_key = "registration.signal"]
    pub signal: Signal,
}

#[attribute_set(name = "test.scope")]
#[derive(Debug, Clone)]
pub struct TestScopeAttributes {
    pub scope: String,
}

#[metric_set(
    name = "test.measurement",
    measurement_attributes = ImplicitMeasurementAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct MeasurementMetrics {
    #[metric(unit = "{items}")]
    pub items: Counter<u64>,
}

#[metric_set(name = "test.plain")]
#[derive(Debug, Default, Clone)]
pub struct PlainMetrics {
    #[metric(unit = "{items}")]
    pub items: Counter<u64>,
}

#[metric_set(
    name = "test.registration",
    registration_attributes = RegistrationAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct RegistrationMetrics {
    #[metric(unit = "{records}")]
    pub records: Counter<u64>,
}

#[metric_set(
    name = "test.mixed",
    registration_attributes = RegistrationAttributes,
    measurement_attributes = ExplicitMeasurementAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct MixedMetrics {
    #[metric(unit = "{records}")]
    pub records: Counter<u64>,
}

struct TestRegistrar {
    registry: TelemetryRegistryHandle,
}

impl TestRegistrar {
    fn new() -> Self {
        Self {
            registry: TelemetryRegistryHandle::new(),
        }
    }

    fn scope() -> TestScopeAttributes {
        TestScopeAttributes {
            scope: "test".to_string(),
        }
    }
}

impl MetricSetRegistrar for TestRegistrar {
    fn register_metric_set<
        M: otap_df_telemetry::metrics::MetricSetHandler + Default + std::fmt::Debug + Send + Sync,
    >(
        &self,
    ) -> MetricSet<M> {
        self.registry.register_metric_set(Self::scope())
    }

    fn register_registration_metric_set<
        M: RegistrationMetricSetHandler + std::fmt::Debug + Send + Sync,
    >(
        &self,
        registration_attrs: &M::RegistrationAttributes,
    ) -> MetricSet<M> {
        self.registry
            .register_metric_set_with_registration_attributes(Self::scope(), registration_attrs)
    }

    fn register_measurement_metric_set<
        M: MeasurementMetricSetHandler + std::fmt::Debug + Send + Sync,
    >(
        &self,
    ) -> MeasurementMetricSet<M> {
        self.registry
            .register_metric_set_with_measurement_attributes(Self::scope())
    }

    fn register_registration_and_measurement_metric_set<
        M: RegistrationMetricSetHandler + MeasurementMetricSetHandler + std::fmt::Debug + Send + Sync,
    >(
        &self,
        registration_attrs: &M::RegistrationAttributes,
    ) -> MeasurementMetricSet<M> {
        self.registry
            .register_metric_set_with_registration_and_measurement_attributes(
                Self::scope(),
                registration_attrs,
            )
    }
}

/// Scenario: enum variants use default and explicit exported strings.
/// Guarantees: descriptors expose stable strings, indexes, and cardinality.
#[test]
fn attribute_enum_uses_default_and_override_values() {
    assert_eq!(Signal::Logs.as_str(), "log-records");
    assert_eq!(Signal::Metrics.as_str(), "metrics");
    assert_eq!(Signal::Traces.as_str(), "traces");
    assert_eq!(Signal::CARDINALITY, 3);
    assert_eq!(Signal::VARIANTS, &["log-records", "metrics", "traces"]);
    assert_eq!(Signal::Logs.variant_index(), 0);
    assert_eq!(Signal::Traces.variant_index(), 2);
}

/// Scenario: a two-field enum attribute set selects datapoint buckets.
/// Guarantees: descriptors and mixed-radix indexes match declaration order.
#[test]
fn measurement_attribute_set_descriptors_and_bucketing() {
    assert_eq!(ImplicitMeasurementAttributes::CARDINALITY, 6);
    let descriptors = ImplicitMeasurementAttributes::DESCRIPTORS;
    assert_eq!(descriptors.len(), 2);
    assert_eq!(descriptors[0].key, "signal");
    assert_eq!(
        descriptors[0].variants,
        &["log-records", "metrics", "traces"]
    );
    assert_eq!(descriptors[1].key, "outcome");
    assert_eq!(descriptors[1].variants, &["dropped", "expired"]);

    // First declared field (signal) is the low-order digit; radix 3 then 2.
    let idx = |signal, outcome| ImplicitMeasurementAttributes { signal, outcome }.bucket_index();
    assert_eq!(idx(Signal::Logs, Outcome::Dropped), 0);
    assert_eq!(idx(Signal::Metrics, Outcome::Dropped), 1);
    assert_eq!(idx(Signal::Traces, Outcome::Dropped), 2);
    assert_eq!(idx(Signal::Logs, Outcome::Expired), 3);
    assert_eq!(idx(Signal::Metrics, Outcome::Expired), 4);
    assert_eq!(idx(Signal::Traces, Outcome::Expired), 5);
}

/// Scenario: a measurement set uses an `attribute_value` override.
/// Guarantees: the measurement datapoint carries the overridden enum wire value.
#[test]
fn measurement_metric_set_export_carries_datapoint_attributes() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let scope = TestScopeAttributes {
        scope: "measurement".to_string(),
    };
    let mut metrics =
        registry.register_metric_set_with_measurement_attributes::<MeasurementMetrics>(scope);

    // Record into two distinct buckets, one of them twice to check aggregation.
    metrics
        .with(ImplicitMeasurementAttributes {
            signal: Signal::Metrics,
            outcome: Outcome::Expired,
        })
        .items
        .add(80);
    metrics
        .with(ImplicitMeasurementAttributes {
            signal: Signal::Logs,
            outcome: Outcome::Dropped,
        })
        .items
        .add(5);
    metrics
        .with(ImplicitMeasurementAttributes {
            signal: Signal::Logs,
            outcome: Outcome::Dropped,
        })
        .items
        .add(2);

    reporter.report_measurement(&mut metrics).unwrap();

    // Drain the channel into the registry (one snapshot per touched bucket).
    let mut snapshot_count = 0;
    while let Ok(snapshot) = rx.try_recv() {
        registry.accumulate_metric_set_snapshot(
            snapshot.key(),
            snapshot.bucket(),
            snapshot.get_metrics(),
        );
        snapshot_count += 1;
    }
    assert_eq!(snapshot_count, 2, "two touched buckets => two snapshots");

    // Visit and verify the decoded per-datapoint attributes and aggregated values.
    let mut seen: Vec<(Vec<(String, String)>, u64)> = Vec::new();
    registry.visit_metrics_and_reset_with_datapoint_attrs(
        |_desc, _scope, dp_attrs, iter| {
            let attrs = dp_attrs
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect::<Vec<_>>();
            let total = iter.map(|(_, v)| v.to_u64_lossy()).sum();
            seen.push((attrs, total));
        },
        false,
    );

    seen.sort();
    assert_eq!(
        seen,
        vec![
            (
                vec![
                    ("signal".to_string(), "log-records".to_string()),
                    ("outcome".to_string(), "dropped".to_string()),
                ],
                7, // 5 + 2 aggregated in the same bucket
            ),
            (
                vec![
                    ("signal".to_string(), "metrics".to_string()),
                    ("outcome".to_string(), "expired".to_string()),
                ],
                80,
            ),
        ]
    );
}

/// Scenario: measurement buckets are drained through the ITS export bridge.
/// Guarantees: OTLP emits one stream containing distinct attributed points,
/// rather than combining bucket values or duplicating the metric stream.
#[test]
fn measurement_metric_set_its_export_preserves_bucket_points() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);
    let mut metrics = registry
        .register_metric_set_with_measurement_attributes::<MeasurementMetrics>(
            TestScopeAttributes {
                scope: "its".to_string(),
            },
        );

    metrics
        .with(ImplicitMeasurementAttributes {
            signal: Signal::Logs,
            outcome: Outcome::Dropped,
        })
        .items
        .add(7);
    metrics
        .with(ImplicitMeasurementAttributes {
            signal: Signal::Metrics,
            outcome: Outcome::Expired,
        })
        .items
        .add(80);
    reporter.report_measurement(&mut metrics).unwrap();
    while let Ok(snapshot) = rx.try_recv() {
        registry.accumulate_metric_set_snapshot(
            snapshot.key(),
            snapshot.bucket(),
            snapshot.get_metrics(),
        );
    }

    let batch = registry.drain_metric_export_batch();
    assert_eq!(batch.metric_sets.len(), 2);
    let encoder = MetricsOtlpEncoder::new(&ResourceLogs::default().encode_to_vec()).unwrap();
    let encoded = encoder.encode(&batch).unwrap().expect("non-empty metrics");
    let OtlpProtoBytes::ExportMetricsRequest(bytes) = encoded else {
        panic!("expected an OTLP metrics request");
    };
    let request = ExportMetricsServiceRequest::decode(bytes).unwrap();
    let [resource] = request.resource_metrics.as_slice() else {
        panic!("expected one resource metrics message");
    };
    let [scope] = resource.scope_metrics.as_slice() else {
        panic!("measurement buckets must share one instrumentation scope");
    };
    let [metric] = scope.metrics.as_slice() else {
        panic!("measurement buckets must share one metric stream");
    };
    let Some(metric::Data::Sum(sum)) = metric.data.as_ref() else {
        panic!("measurement counter must be an OTLP sum");
    };

    let mut points = sum
        .data_points
        .iter()
        .map(|point| {
            let attributes = point
                .attributes
                .iter()
                .map(|attribute| {
                    let value = attribute
                        .value
                        .as_ref()
                        .and_then(|value| value.value.as_ref())
                        .map(|value| match value {
                            otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(value) => value.as_str(),
                            _ => panic!("expected string datapoint attribute"),
                        })
                        .expect("datapoint attribute value");
                    (attribute.key.as_str(), value)
                })
                .collect::<Vec<_>>();
            let Some(number_data_point::Value::AsInt(value)) = point.value else {
                panic!("expected an integer datapoint");
            };
            (attributes, value)
        })
        .collect::<Vec<_>>();
    points.sort_by_key(|(_, value)| *value);
    assert_eq!(
        points,
        vec![
            (vec![("signal", "log-records"), ("outcome", "dropped")], 7),
            (vec![("signal", "metrics"), ("outcome", "expired")], 80),
        ]
    );
}

/// Scenario: a measurement bucket is reported while its reporting channel is full.
/// Guarantees: its values remain pending, accumulate subsequent recordings, and
/// are sent after channel capacity returns.
#[test]
fn measurement_metric_set_retries_deferred_bucket() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(1);
    let attrs = ImplicitMeasurementAttributes {
        signal: Signal::Logs,
        outcome: Outcome::Dropped,
    };
    let mut metrics = registry
        .register_metric_set_with_measurement_attributes::<MeasurementMetrics>(
            TestScopeAttributes {
                scope: "deferred".to_string(),
            },
        );

    metrics.with(attrs).items.add(3);
    reporter.report_measurement(&mut metrics).unwrap();

    metrics.with(attrs).items.add(4);
    reporter.report_measurement(&mut metrics).unwrap();

    // The deferred bucket stays live, so later recordings join its retry snapshot.
    metrics.with(attrs).items.add(6);

    let first = rx
        .try_recv()
        .expect("first snapshot should fill the channel");
    assert_eq!(first.get_metrics()[0].to_u64_lossy(), 3);

    reporter.report_measurement(&mut metrics).unwrap();
    let retry = rx
        .try_recv()
        .expect("deferred measurement bucket should be retried");
    assert_eq!(retry.get_metrics()[0].to_u64_lossy(), 10);
}

/// Scenario: a registration set uses `attribute_key` and `attribute_value` overrides.
/// Guarantees: the registration datapoint carries both overridden key and enum wire value.
#[test]
fn registration_metric_set_export_carries_fixed_attributes() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let scope = TestScopeAttributes {
        scope: "registration".to_string(),
    };
    let static_attrs = RegistrationAttributes {
        signal: Signal::Logs,
    };
    let mut journald = registry
        .register_metric_set_with_registration_attributes::<RegistrationMetrics>(
            scope,
            &static_attrs,
        );

    journald.records.add(42);
    reporter.report(&mut journald).unwrap();

    while let Ok(snapshot) = rx.try_recv() {
        registry.accumulate_metric_set_snapshot(
            snapshot.key(),
            snapshot.bucket(),
            snapshot.get_metrics(),
        );
    }

    let mut seen: Vec<(Vec<(String, String)>, u64)> = Vec::new();
    registry.visit_metrics_and_reset_with_datapoint_attrs(
        |_desc, _scope, dp_attrs, iter| {
            let attrs = dp_attrs
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect::<Vec<_>>();
            let total = iter.map(|(_, v)| v.to_u64_lossy()).sum();
            seen.push((attrs, total));
        },
        false,
    );

    assert_eq!(
        seen,
        vec![(
            vec![("registration.signal".to_string(), "log-records".to_string())],
            42
        )]
    );
}

/// Scenario: a mixed set uses registration and measurement key/value overrides.
/// Guarantees: every emitted bucket includes the registration and measurement overridden keys.
#[test]
fn registration_and_measurement_metric_set_export_carries_both_attribute_kinds() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let static_attrs = RegistrationAttributes {
        signal: Signal::Logs,
    };
    let mut metrics = registry
        .register_metric_set_with_registration_and_measurement_attributes::<MixedMetrics>(
            TestScopeAttributes {
                scope: "mixed".to_string(),
            },
            &static_attrs,
        );
    metrics
        .with(ExplicitMeasurementAttributes {
            outcome: Outcome::Dropped,
        })
        .records
        .add(4);
    metrics
        .with(ExplicitMeasurementAttributes {
            outcome: Outcome::Expired,
        })
        .records
        .add(2);

    reporter.report_measurement(&mut metrics).unwrap();
    while let Ok(snapshot) = rx.try_recv() {
        registry.accumulate_metric_set_snapshot(
            snapshot.key(),
            snapshot.bucket(),
            snapshot.get_metrics(),
        );
    }

    let mut seen: Vec<(Vec<(String, String)>, u64)> = Vec::new();
    registry.visit_metrics_and_reset_with_datapoint_attrs(
        |_desc, _scope, dp_attrs, iter| {
            let attrs = dp_attrs
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                .collect::<Vec<_>>();
            let total = iter.map(|(_, value)| value.to_u64_lossy()).sum();
            seen.push((attrs, total));
        },
        false,
    );

    seen.sort();
    assert_eq!(
        seen,
        vec![
            (
                vec![
                    ("registration.signal".to_string(), "log-records".to_string()),
                    ("result".to_string(), "dropped".to_string()),
                ],
                4,
            ),
            (
                vec![
                    ("registration.signal".to_string(), "log-records".to_string()),
                    ("result".to_string(), "expired".to_string()),
                ],
                2,
            ),
        ]
    );
}

/// Scenario: generated metric-set registration methods receive a registrar.
/// Guarantees: plain, registration, measurement, and combined declarations select their valid API.
#[test]
fn generated_metric_set_registration_methods_dispatch_to_registrar() {
    let registrar = TestRegistrar::new();
    let signal = RegistrationAttributes {
        signal: Signal::Logs,
    };

    let mut plain_metrics = PlainMetrics::register(&registrar);
    plain_metrics.items.add(1);

    let mut registration_metrics = RegistrationMetrics::register(&registrar, &signal);
    registration_metrics.records.add(1);

    let mut measurement_metrics = MeasurementMetrics::register(&registrar);
    measurement_metrics
        .with(ImplicitMeasurementAttributes {
            signal: Signal::Metrics,
            outcome: Outcome::Dropped,
        })
        .items
        .add(1);

    let mut combined_metrics = MixedMetrics::register(&registrar, &signal);
    combined_metrics
        .with(ExplicitMeasurementAttributes {
            outcome: Outcome::Expired,
        })
        .records
        .add(1);
}

/// Scenario: registration and measurement metric sets share a registered entity.
/// Guarantees: both sets retain their datapoint attributes after reporting.
#[test]
fn register_metric_sets_for_existing_entity() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    // Register the owning entity once, then attach both a registration and a measurement
    // metric set to that same entity key.
    let entity = registry.register_entity(TestScopeAttributes {
        scope: "shared".to_string(),
    });

    let static_attrs = RegistrationAttributes {
        signal: Signal::Traces,
    };
    let mut journald = registry
        .register_metric_set_with_registration_attributes_for_entity::<RegistrationMetrics>(
            entity,
            &static_attrs,
        );
    let mut loss = registry
        .register_metric_set_with_measurement_attributes_for_entity::<MeasurementMetrics>(entity);

    // Both metric sets should be bound to the entity we created.
    assert_eq!(loss.entity_key(), entity);

    journald.records.add(7);
    loss.with(ImplicitMeasurementAttributes {
        signal: Signal::Traces,
        outcome: Outcome::Dropped,
    })
    .items
    .add(3);

    reporter.report(&mut journald).unwrap();
    reporter.report_measurement(&mut loss).unwrap();

    while let Ok(snapshot) = rx.try_recv() {
        registry.accumulate_metric_set_snapshot(
            snapshot.key(),
            snapshot.bucket(),
            snapshot.get_metrics(),
        );
    }

    let mut seen: Vec<(Vec<(String, String)>, u64)> = Vec::new();
    registry.visit_metrics_and_reset_with_datapoint_attrs(
        |_desc, _scope, dp_attrs, iter| {
            let attrs = dp_attrs
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect::<Vec<_>>();
            let total = iter.map(|(_, v)| v.to_u64_lossy()).sum();
            seen.push((attrs, total));
        },
        false,
    );
    seen.sort();
    assert_eq!(
        seen,
        vec![
            (
                vec![("registration.signal".to_string(), "traces".to_string())],
                7,
            ),
            (
                vec![
                    ("signal".to_string(), "traces".to_string()),
                    ("outcome".to_string(), "dropped".to_string()),
                ],
                3,
            ),
        ]
    );
}

/// Scenario: a measurement datapoint is read from the registry without reset.
/// Guarantees: repeated reads return the same attributes and values.
#[test]
fn read_only_visit_preserves_datapoint_values() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let mut loss = registry.register_metric_set_with_measurement_attributes::<MeasurementMetrics>(
        TestScopeAttributes {
            scope: "readonly".to_string(),
        },
    );
    loss.with(ImplicitMeasurementAttributes {
        signal: Signal::Metrics,
        outcome: Outcome::Expired,
    })
    .items
    .add(11);
    reporter.report_measurement(&mut loss).unwrap();
    while let Ok(snapshot) = rx.try_recv() {
        registry.accumulate_metric_set_snapshot(
            snapshot.key(),
            snapshot.bucket(),
            snapshot.get_metrics(),
        );
    }

    // Visiting without reset must return the same value on repeated calls.
    let read = || {
        let mut seen: Vec<(Vec<(String, String)>, u64)> = Vec::new();
        registry.visit_current_metrics_with_datapoint_attrs(
            |_desc, _scope, dp_attrs, iter| {
                let attrs = dp_attrs
                    .iter()
                    .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                    .collect::<Vec<_>>();
                let total = iter.map(|(_, v)| v.to_u64_lossy()).sum();
                seen.push((attrs, total));
            },
            false,
        );
        seen
    };
    let first = read();
    let second = read();
    assert_eq!(first, second);
    assert_eq!(
        first,
        vec![(
            vec![
                ("signal".to_string(), "metrics".to_string()),
                ("outcome".to_string(), "expired".to_string()),
            ],
            11,
        )]
    );
}
