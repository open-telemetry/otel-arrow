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
    Logs,
    Metrics,
    Traces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum LossOutcome {
    Dropped,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum HttpMethod {
    #[attribute_value = "GET"]
    Get,
    #[attribute_value = "POST"]
    Post,
    Patch,
}

// A dynamic attribute set: every field is a per-record attribute.
#[attribute_set(name = "test.loss.attrs", dynamic)]
#[derive(Debug, Clone, Copy)]
pub struct LossAttributes {
    pub signal: Signal,
    pub outcome: LossOutcome,
}

// A dynamic attribute set that can be combined with static signal attributes.
#[attribute_set(name = "test.outcome.attrs", dynamic)]
#[derive(Debug, Clone, Copy)]
pub struct OutcomeAttributes {
    #[attribute(key = "outcome")]
    pub outcome: LossOutcome,
}

// A static attribute set: value fixed once at registration.
#[attribute_set(name = "test.signal.attrs")]
#[derive(Debug, Clone, Copy)]
pub struct SignalAttributes {
    #[attribute(key = "signal")]
    pub signal: Signal,
}

// Scope attributes for the owning entity.
#[attribute_set(name = "test.scope")]
#[derive(Debug, Clone)]
pub struct ScopeAttributes {
    #[attribute(key = "node")]
    pub node: String,
}

#[metric_set(name = "test.loss", dynamic_attributes = LossAttributes)]
#[derive(Debug, Default, Clone)]
pub struct LossMetrics {
    /// Number of items lost.
    #[metric(unit = "{items}")]
    pub lost_items: Counter<u64>,
}

#[metric_set(name = "test.journald", static_attributes = SignalAttributes)]
#[derive(Debug, Default, Clone)]
pub struct JournaldMetrics {
    /// Number of records read.
    #[metric(unit = "{records}")]
    pub records: Counter<u64>,
}

#[metric_set(
    name = "test.journald.outcomes",
    static_attributes = SignalAttributes,
    dynamic_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct JournaldOutcomeMetrics {
    /// Number of records read by outcome.
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

    fn scope() -> ScopeAttributes {
        ScopeAttributes {
            node: "test".to_string(),
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
fn attribute_enum_snake_case_and_override() {
    assert_eq!(Signal::Logs.as_str(), "logs");
    assert_eq!(Signal::Metrics.as_str(), "metrics");
    assert_eq!(Signal::Traces.as_str(), "traces");
    assert_eq!(Signal::CARDINALITY, 3);
    assert_eq!(Signal::VARIANTS, &["logs", "metrics", "traces"]);
    assert_eq!(Signal::Logs.variant_index(), 0);
    assert_eq!(Signal::Traces.variant_index(), 2);

    // Explicit overrides win; un-annotated variants fall back to snake_case.
    assert_eq!(HttpMethod::Get.as_str(), "GET");
    assert_eq!(HttpMethod::Post.as_str(), "POST");
    assert_eq!(HttpMethod::Patch.as_str(), "patch");
    assert_eq!(HttpMethod::CARDINALITY, 3);
}

/// Scenario: a two-field enum attribute set selects datapoint buckets.
/// Guarantees: descriptors and mixed-radix indexes match declaration order.
#[test]
fn dynamic_attribute_set_descriptors_and_bucketing() {
    assert_eq!(LossAttributes::CARDINALITY, 6);
    let descriptors = LossAttributes::DESCRIPTORS;
    assert_eq!(descriptors.len(), 2);
    assert_eq!(descriptors[0].key, "signal");
    assert_eq!(descriptors[0].variants, &["logs", "metrics", "traces"]);
    assert_eq!(descriptors[1].key, "outcome");
    assert_eq!(descriptors[1].variants, &["dropped", "expired"]);

    // First declared field (signal) is the low-order digit; radix 3 then 2.
    let idx = |signal, outcome| LossAttributes { signal, outcome }.bucket_index();
    assert_eq!(idx(Signal::Logs, LossOutcome::Dropped), 0);
    assert_eq!(idx(Signal::Metrics, LossOutcome::Dropped), 1);
    assert_eq!(idx(Signal::Traces, LossOutcome::Dropped), 2);
    assert_eq!(idx(Signal::Logs, LossOutcome::Expired), 3);
    assert_eq!(idx(Signal::Metrics, LossOutcome::Expired), 4);
    assert_eq!(idx(Signal::Traces, LossOutcome::Expired), 5);
}

/// Scenario: two dynamic buckets report through the telemetry registry.
/// Guarantees: each bucket retains attributes and aggregates its own values.
#[test]
fn dynamic_metric_set_export_carries_datapoint_attributes() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let scope = ScopeAttributes {
        node: "durable_buffer".to_string(),
    };
    let mut loss = registry.register_metric_set_with_dynamic_attributes::<LossMetrics>(scope);

    // Record into two distinct buckets, one of them twice to check aggregation.
    loss.with(LossAttributes {
        signal: Signal::Metrics,
        outcome: LossOutcome::Expired,
    })
    .lost_items
    .add(80);
    loss.with(LossAttributes {
        signal: Signal::Logs,
        outcome: LossOutcome::Dropped,
    })
    .lost_items
    .add(5);
    loss.with(LossAttributes {
        signal: Signal::Logs,
        outcome: LossOutcome::Dropped,
    })
    .lost_items
    .add(2);

    reporter.report_dynamic(&mut loss).unwrap();

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
                    ("signal".to_string(), "logs".to_string()),
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

/// Scenario: a static attribute is supplied when a metric set is registered.
/// Guarantees: the fixed attribute is exported with the recorded datapoint.
#[test]
fn static_metric_set_export_carries_fixed_attributes() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let scope = ScopeAttributes {
        node: "journald".to_string(),
    };
    let static_attrs = SignalAttributes {
        signal: Signal::Logs,
    };
    let mut journald = registry
        .register_metric_set_with_static_attributes::<JournaldMetrics>(scope, &static_attrs);

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
        vec![(vec![("signal".to_string(), "logs".to_string())], 42)]
    );
}

/// Scenario: fixed and per-record attributes are bound to one metric set.
/// Guarantees: every emitted bucket includes the registration-time attribute.
#[test]
fn static_and_dynamic_metric_set_export_carries_both_attribute_kinds() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let static_attrs = SignalAttributes {
        signal: Signal::Logs,
    };
    let mut metrics = registry
        .register_metric_set_with_static_and_dynamic_attributes::<JournaldOutcomeMetrics>(
            ScopeAttributes {
                node: "journald".to_string(),
            },
            &static_attrs,
        );
    metrics
        .with(OutcomeAttributes {
            outcome: LossOutcome::Dropped,
        })
        .records
        .add(4);
    metrics
        .with(OutcomeAttributes {
            outcome: LossOutcome::Expired,
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
                    ("signal".to_string(), "logs".to_string()),
                    ("outcome".to_string(), "dropped".to_string()),
                ],
                4,
            ),
            (
                vec![
                    ("signal".to_string(), "logs".to_string()),
                    ("outcome".to_string(), "expired".to_string()),
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
    let signal = SignalAttributes {
        signal: Signal::Logs,
    };

    let mut static_metrics = JournaldMetrics::register(&registrar, &signal);
    static_metrics.records.add(1);

    let mut dynamic_metrics = LossMetrics::register(&registrar);
    dynamic_metrics
        .with(LossAttributes {
            signal: Signal::Metrics,
            outcome: LossOutcome::Dropped,
        })
        .lost_items
        .add(1);

    let mut combined_metrics = JournaldOutcomeMetrics::register(&registrar, &signal);
    combined_metrics
        .with(OutcomeAttributes {
            outcome: LossOutcome::Expired,
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
    let entity = registry.register_entity(ScopeAttributes {
        node: "shared_entity".to_string(),
    });

    let static_attrs = SignalAttributes {
        signal: Signal::Traces,
    };
    let mut journald = registry
        .register_metric_set_with_static_attributes_for_entity::<JournaldMetrics>(
            entity,
            &static_attrs,
        );
    let mut loss =
        registry.register_metric_set_with_dynamic_attributes_for_entity::<LossMetrics>(entity);

    // Both metric sets should be bound to the entity we created.
    assert_eq!(loss.entity_key(), entity);

    journald.records.add(7);
    loss.with(LossAttributes {
        signal: Signal::Traces,
        outcome: LossOutcome::Dropped,
    })
    .lost_items
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
            (vec![("signal".to_string(), "traces".to_string())], 7),
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

/// Scenario: a dynamic datapoint is read from the registry without reset.
/// Guarantees: repeated reads return the same attributes and values.
#[test]
fn read_only_visit_preserves_datapoint_values() {
    let registry = TelemetryRegistryHandle::new();
    let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(64);

    let mut loss =
        registry.register_metric_set_with_dynamic_attributes::<LossMetrics>(ScopeAttributes {
            node: "readonly".to_string(),
        });
    loss.with(LossAttributes {
        signal: Signal::Metrics,
        outcome: LossOutcome::Expired,
    })
    .lost_items
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
