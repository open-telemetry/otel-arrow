// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Collection diagnostics extracted from the MVP runtime, without receiver activation.

use super::super::config::CounterConfig;
use super::super::model::{Sample, SampleValue};
use super::Error;
use std::collections::BTreeSet;
use std::sync::Arc;

pub(super) struct SampleLogState {
    node: String,
    configs: Vec<CounterConfig>,
    active_counter_failures: BTreeSet<(usize, Arc<str>, &'static str)>,
}

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod tests;

impl SampleLogState {
    pub(super) fn new(node: String, configs: Vec<CounterConfig>) -> Self {
        Self {
            node,
            configs,
            active_counter_failures: BTreeSet::new(),
        }
    }

    pub(super) fn log(&mut self, sample: &Sample) -> Result<(), Error> {
        let retained = sample
            .members
            .iter()
            .map(|member| (member.counter_index, member.path.as_ref()))
            .chain(
                sample
                    .metadata
                    .failures
                    .iter()
                    .filter(|failure| failure.reason == "PdhRemoveCounter")
                    .map(|failure| (failure.counter_index, failure.path.as_ref())),
            )
            .collect::<BTreeSet<_>>();
        self.active_counter_failures.retain(|(index, path, _)| {
            retained.contains(&(*index, path.as_ref()))
                || self
                    .configs
                    .get(*index)
                    .is_some_and(|config| path.as_ref() == config.path)
        });
        for point in &sample.points {
            let counter = self
                .configs
                .get(point.counter_index)
                .ok_or(Error::InvalidSample(
                    "diagnostic point references missing configured counter",
                ))?;
            if matches!(point.value, SampleValue::Value(_)) {
                self.active_counter_failures.retain(|(index, path, _)| {
                    *index != point.counter_index
                        || (path.as_ref() != point.path.as_ref()
                            && path.as_ref() != counter.path.as_str())
                });
            }
        }
        for failure in &sample.metadata.failures {
            let counter = self
                .configs
                .get(failure.counter_index)
                .ok_or(Error::InvalidSample(
                    "diagnostic failure references missing configured counter",
                ))?;
            // Selector episodes are owned by Query and survive retained values and seeded rebuilds.
            let selector_failure = matches!(
                failure.reason,
                "selector_unresolved" | "native_identity_unavailable"
            );
            let key = (
                failure.counter_index,
                Arc::clone(&failure.path),
                failure.reason,
            );
            if selector_failure || !self.active_counter_failures.contains(&key) {
                otel_arrow_dfe_telemetry::otel_warn!(
                    "windows_perf_counters_receiver.counter.fail",
                    node = self.node.as_str(),
                    path_template = counter.path.as_str(),
                    reason = failure.reason,
                    error = failure.error.as_str()
                );
            }
            if !selector_failure {
                let _ = self.active_counter_failures.insert(key);
            }
        }
        for overflow in &sample.metadata.overflows {
            let counter = self
                .configs
                .get(overflow.counter_index)
                .ok_or(Error::InvalidSample(
                    "diagnostic overflow references missing configured counter",
                ))?;
            otel_arrow_dfe_telemetry::otel_warn!(
                "windows_perf_counters_receiver.instance_limit.exceeded",
                node = self.node.as_str(),
                path_template = counter.path.as_str(),
                reason = overflow.reason,
                discovered = overflow.discovered as u64,
                retained = overflow.retained as u64,
                omitted = overflow.omitted as u64,
                per_wildcard_omitted = overflow.per_wildcard_omitted as u64,
                total_omitted = overflow.total_omitted as u64,
                examples = ?overflow.examples
            );
        }
        let diagnostics = &sample.metadata.diagnostics;
        if diagnostics.instances_added > 0 || diagnostics.instances_removed > 0 {
            otel_arrow_dfe_telemetry::otel_info!(
                "windows_perf_counters_receiver.instances.change",
                node = self.node.as_str(),
                added = diagnostics.instances_added,
                removed = diagnostics.instances_removed,
                active = diagnostics.active_expanded_counters as u64,
                timestamp_unix_nano = sample.timestamp_unix_nano
            );
        }
        if diagnostics.retry_attempts > 0
            || diagnostics.retry_recoveries > 0
            || diagnostics.query_rebuild_attempts > 0
            || diagnostics.query_rebuild_recoveries > 0
        {
            otel_arrow_dfe_telemetry::otel_info!(
                "windows_perf_counters_receiver.recovery.change",
                node = self.node.as_str(),
                retry_attempts = diagnostics.retry_attempts,
                retry_recoveries = diagnostics.retry_recoveries,
                query_rebuild_attempts = diagnostics.query_rebuild_attempts,
                query_rebuild_recoveries = diagnostics.query_rebuild_recoveries
            );
        }
        Ok(())
    }
}
