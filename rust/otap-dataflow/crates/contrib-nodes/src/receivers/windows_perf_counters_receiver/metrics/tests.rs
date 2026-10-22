// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_engine::testing::test_pipeline_ctx;
use std::collections::BTreeSet;

/// Scenario: Collection inventory changes are applied and receiver metric sets produce snapshots.
/// Guarantees: Pending/active state and scrape outcomes retain the settled public metric-set identities.
#[test]
fn collection_metrics_preserve_inventory_and_names() {
    let (pipeline, _) = test_pipeline_ctx();
    let mut metrics = WindowsPerfCountersReceiverMetrics::register(&pipeline);
    metrics.health.apply(&SampleDiagnostics {
        active_expanded_counters: 2,
        query_pending: true,
        discovery_refreshes: 1,
        ..Default::default()
    });
    metrics.record_scrape(Outcome::Failure);
    assert_eq!(metrics.health.active_expanded.get(), 2);
    assert_eq!(metrics.health.query_pending.get(), 1);
    assert_eq!(metrics.health.discovery_refreshes.get(), 1);
    assert_eq!(metrics.scrape_attempts(Outcome::Failure), 1);
    let snapshots = metrics.terminal_snapshots();
    let names = snapshots
        .iter()
        .map(|snapshot| snapshot.descriptor().name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        names,
        BTreeSet::from([
            "receiver.windows_perf_counters",
            "receiver.windows_perf_counters.scrapes"
        ])
    );
}

impl WindowsPerfCountersReceiverMetrics {
    pub(in super::super) fn scrape_attempts(&self, outcome: Outcome) -> u64 {
        self.scrapes
            .get(OutcomeAttributes { outcome })
            .attempts
            .get()
    }
}
