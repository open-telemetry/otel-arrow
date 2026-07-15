// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end tests for the datapoint-level enum-attribute mechanism for metric
//! sets: the `AttributeEnum` derive, `#[attribute_set(dynamic)]`,
//! `#[metric_set(static_attributes = ..)]` / `#[metric_set(dynamic_attributes = ..)]`,
//! dense mixed-radix bucketing, and the export path that carries per-datapoint
//! attributes through the registry.

#![allow(missing_docs)]

use otap_df_telemetry::attributes::{AttributeEnum, DynamicAttributeSet};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::{
    DynamicMetricSet, DynamicMetricSetHandler, MetricSet, MetricSetRegistrar,
    StaticMetricSetHandler,
};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

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

/// Dynamic fields use their field names as keys without annotations.
#[attribute_set(name = "test.implicit.attrs", dynamic)]
#[derive(Debug, Clone, Copy)]
pub struct ImplicitDynamicAttributes {
    pub signal: Signal,
    pub outcome: Outcome,
}

/// An explicit key override changes the exported key from `outcome` to `result`.
#[attribute_set(name = "test.explicit.attrs", dynamic)]
#[derive(Debug, Clone, Copy)]
pub struct ExplicitDynamicAttributes {
    #[attribute_key = "result"]
    pub outcome: Outcome,
}

/// Static fields also use their field names as keys without annotations.
#[attribute_set(name = "test.static.attrs")]
#[derive(Debug, Clone, Copy)]
pub struct StaticAttributes {
    #[attribute_key = "static.signal"]
    pub signal: Signal,
}

#[attribute_set(name = "test.scope")]
#[derive(Debug, Clone)]
pub struct TestScopeAttributes {
    pub scope: String,
}

#[metric_set(
    name = "test.dynamic",
    dynamic_attributes = ImplicitDynamicAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct DynamicMetrics {
    #[metric(unit = "{items}")]
    pub items: Counter<u64>,
}

#[metric_set(name = "test.static", static_attributes = StaticAttributes)]
#[derive(Debug, Default, Clone)]
pub struct StaticMetrics {
    #[metric(unit = "{records}")]
    pub records: Counter<u64>,
}

#[metric_set(
    name = "test.mixed",
    static_attributes = StaticAttributes,
    dynamic_attributes = ExplicitDynamicAttributes
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
    fn register_static_metric_set<M: StaticMetricSetHandler + std::fmt::Debug + Send + Sync>(
        &self,
        static_attrs: &M::StaticAttributes,
    ) -> MetricSet<M> {
        self.registry
            .register_metric_set_with_static_attributes(Self::scope(), static_attrs)
    }

    fn register_dynamic_metric_set<M: DynamicMetricSetHandler + std::fmt::Debug + Send + Sync>(
        &self,
    ) -> DynamicMetricSet<M> {
        self.registry
            .register_metric_set_with_dynamic_attributes(Self::scope())
    }

    fn register_static_and_dynamic_metric_set<
        M: StaticMetricSetHandler + DynamicMetricSetHandler + std::fmt::Debug + Send + Sync,
    >(
        &self,
        static_attrs: &M::StaticAttributes,
    ) -> DynamicMetricSet<M> {
        self.registry
            .register_metric_set_with_static_and_dynamic_attributes(Self::scope(), static_attrs)
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
fn dynamic_attribute_set_descriptors_and_bucketing() {
    assert_eq!(ImplicitDynamicAttributes::CARDINALITY, 6);
    let descriptors = ImplicitDynamicAttributes::DESCRIPTORS;
    assert_eq!(descriptors.len(), 2);
    assert_eq!(descriptors[0].key, "signal");
    assert_eq!(
        descriptors[0].variants,
        &["log-records", "metrics", "traces"]
    );
    assert_eq!(descriptors[1].key, "outcome");
    assert_eq!(descriptors[1].variants, &["dropped", "expired"]);

    // First declared field (signal) is the low-order digit; radix 3 then 2.
    let idx = |signal, outcome| ImplicitDynamicAttributes { signal, outcome }.bucket_index();
    assert_eq!(idx(Signal::Logs, Outcome::Dropped), 0);
    assert_eq!(idx(Signal::Metrics, Outcome::Dropped), 1);
    assert_eq!(idx(Signal::Traces, Outcome::Dropped), 2);
    assert_eq!(idx(Signal::Logs, Outcome::Expired), 3);
    assert_eq!(idx(Signal::Metrics, Outcome::Expired), 4);
    assert_eq!(idx(Signal::Traces, Outcome::Expired), 5);
}

/// Scenario: a dynamic set uses an `attribute_value` override.
/// Guarantees: the dynamic datapoint carries the overridden enum wire value.
#[test]
fn dynamic_metric_set_export_carries_datapoint_attributes() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let scope = TestScopeAttributes {
        scope: "dynamic".to_string(),
    };
    let mut metrics = registry.register_metric_set_with_dynamic_attributes::<DynamicMetrics>(scope);

    // Record into two distinct buckets, one of them twice to check aggregation.
    metrics
        .with(ImplicitDynamicAttributes {
            signal: Signal::Metrics,
            outcome: Outcome::Expired,
        })
        .items
        .add(80);
    metrics
        .with(ImplicitDynamicAttributes {
            signal: Signal::Logs,
            outcome: Outcome::Dropped,
        })
        .items
        .add(5);
    metrics
        .with(ImplicitDynamicAttributes {
            signal: Signal::Logs,
            outcome: Outcome::Dropped,
        })
        .items
        .add(2);

    reporter.report_dynamic(&mut metrics).unwrap();

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

/// Scenario: a dynamic bucket is reported while its reporting channel is full.
/// Guarantees: its values remain pending, accumulate subsequent recordings, and
/// are sent after channel capacity returns.
#[test]
fn dynamic_metric_set_retries_deferred_bucket() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(1);
    let attrs = ImplicitDynamicAttributes {
        signal: Signal::Logs,
        outcome: Outcome::Dropped,
    };
    let mut metrics = registry.register_metric_set_with_dynamic_attributes::<DynamicMetrics>(
        TestScopeAttributes {
            scope: "deferred".to_string(),
        },
    );

    metrics.with(attrs).items.add(3);
    reporter.report_dynamic(&mut metrics).unwrap();

    metrics.with(attrs).items.add(4);
    reporter.report_dynamic(&mut metrics).unwrap();

    // The deferred bucket stays live, so later recordings join its retry snapshot.
    metrics.with(attrs).items.add(6);

    let first = rx
        .try_recv()
        .expect("first snapshot should fill the channel");
    assert_eq!(first.get_metrics()[0].to_u64_lossy(), 3);

    reporter.report_dynamic(&mut metrics).unwrap();
    let retry = rx
        .try_recv()
        .expect("deferred dynamic bucket should be retried");
    assert_eq!(retry.get_metrics()[0].to_u64_lossy(), 10);
}

/// Scenario: a static set uses `attribute_key` and `attribute_value` overrides.
/// Guarantees: the static datapoint carries both overridden key and enum wire value.
#[test]
fn static_metric_set_export_carries_fixed_attributes() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let scope = TestScopeAttributes {
        scope: "static".to_string(),
    };
    let static_attrs = StaticAttributes {
        signal: Signal::Logs,
    };
    let mut journald =
        registry.register_metric_set_with_static_attributes::<StaticMetrics>(scope, &static_attrs);

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
            vec![("static.signal".to_string(), "log-records".to_string())],
            42
        )]
    );
}

/// Scenario: a mixed set uses static and dynamic key/value overrides.
/// Guarantees: every emitted bucket includes the static and dynamic overridden keys.
#[test]
fn static_and_dynamic_metric_set_export_carries_both_attribute_kinds() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let static_attrs = StaticAttributes {
        signal: Signal::Logs,
    };
    let mut metrics = registry
        .register_metric_set_with_static_and_dynamic_attributes::<MixedMetrics>(
            TestScopeAttributes {
                scope: "mixed".to_string(),
            },
            &static_attrs,
        );
    metrics
        .with(ExplicitDynamicAttributes {
            outcome: Outcome::Dropped,
        })
        .records
        .add(4);
    metrics
        .with(ExplicitDynamicAttributes {
            outcome: Outcome::Expired,
        })
        .records
        .add(2);

    reporter.report_dynamic(&mut metrics).unwrap();
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
                    ("static.signal".to_string(), "log-records".to_string()),
                    ("result".to_string(), "dropped".to_string()),
                ],
                4,
            ),
            (
                vec![
                    ("static.signal".to_string(), "log-records".to_string()),
                    ("result".to_string(), "expired".to_string()),
                ],
                2,
            ),
        ]
    );
}

/// Scenario: generated metric-set registration methods receive a registrar.
/// Guarantees: static, dynamic, and combined declarations select their valid API.
#[test]
fn generated_metric_set_registration_methods_dispatch_to_registrar() {
    let registrar = TestRegistrar::new();
    let signal = StaticAttributes {
        signal: Signal::Logs,
    };

    let mut static_metrics = StaticMetrics::register(&registrar, &signal);
    static_metrics.records.add(1);

    let mut dynamic_metrics = DynamicMetrics::register(&registrar);
    dynamic_metrics
        .with(ImplicitDynamicAttributes {
            signal: Signal::Metrics,
            outcome: Outcome::Dropped,
        })
        .items
        .add(1);

    let mut combined_metrics = MixedMetrics::register(&registrar, &signal);
    combined_metrics
        .with(ExplicitDynamicAttributes {
            outcome: Outcome::Expired,
        })
        .records
        .add(1);
}

/// Scenario: static and dynamic metric sets share a registered entity.
/// Guarantees: both sets retain their datapoint attributes after reporting.
#[test]
fn register_metric_sets_for_existing_entity() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    // Register the owning entity once, then attach both a static and a dynamic
    // metric set to that same entity key.
    let entity = registry.register_entity(TestScopeAttributes {
        scope: "shared".to_string(),
    });

    let static_attrs = StaticAttributes {
        signal: Signal::Traces,
    };
    let mut journald = registry
        .register_metric_set_with_static_attributes_for_entity::<StaticMetrics>(
            entity,
            &static_attrs,
        );
    let mut loss =
        registry.register_metric_set_with_dynamic_attributes_for_entity::<DynamicMetrics>(entity);

    // Both metric sets should be bound to the entity we created.
    assert_eq!(loss.entity_key(), entity);

    journald.records.add(7);
    loss.with(ImplicitDynamicAttributes {
        signal: Signal::Traces,
        outcome: Outcome::Dropped,
    })
    .items
    .add(3);

    reporter.report(&mut journald).unwrap();
    reporter.report_dynamic(&mut loss).unwrap();

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
                vec![
                    ("signal".to_string(), "traces".to_string()),
                    ("outcome".to_string(), "dropped".to_string()),
                ],
                3,
            ),
            (vec![("static.signal".to_string(), "traces".to_string())], 7),
        ]
    );
}

/// Scenario: a dynamic datapoint is read from the registry without reset.
/// Guarantees: repeated reads return the same attributes and values.
#[test]
fn read_only_visit_preserves_datapoint_values() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let mut loss = registry.register_metric_set_with_dynamic_attributes::<DynamicMetrics>(
        TestScopeAttributes {
            scope: "readonly".to_string(),
        },
    );
    loss.with(ImplicitDynamicAttributes {
        signal: Signal::Metrics,
        outcome: Outcome::Expired,
    })
    .items
    .add(11);
    reporter.report_dynamic(&mut loss).unwrap();
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
