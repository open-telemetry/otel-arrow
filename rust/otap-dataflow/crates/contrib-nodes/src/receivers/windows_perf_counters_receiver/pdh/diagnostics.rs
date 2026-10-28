// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Native collection and recovery diagnostics, without receiver activation.

use super::super::config::CounterConfig;
use super::super::model::{PendingMetadata, Sample, SampleFailure, SampleValue};
use super::Error;
use std::collections::BTreeSet;
use std::sync::Arc;

pub(super) struct SampleLogState {
    node: String,
    configs: Vec<CounterConfig>,
    active_counter_failures: BTreeSet<(usize, Arc<str>, &'static str)>,
    query_failure_active: bool,
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
            query_failure_active: false,
        }
    }

    /// Logs counter-local diagnostics before query-failure metadata is merged.
    /// Returns the number of counter warnings selected by episode suppression.
    pub(super) fn log(&mut self, sample: &Sample) -> Result<usize, Error> {
        if sample.metadata.diagnostics.any_query_collected {
            self.query_failure_active = false;
        }
        let mut warnings = 0;
        let retained = sample
            .members
            .iter()
            .map(|member| (member.counter_index, member.path.as_ref()))
            .chain(
                sample
                    .metadata
                    .pending_cleanup
                    .iter()
                    .flatten()
                    .map(|(index, path)| (*index, path.as_ref())),
            )
            .collect::<BTreeSet<_>>();
        let mut recovered = BTreeSet::new();
        for point in &sample.points {
            let counter = self
                .configs
                .get(point.counter_index)
                .ok_or(Error::InvalidSample(
                    "diagnostic point references missing configured counter",
                ))?;
            if matches!(point.value, SampleValue::Value(_)) {
                let _ = recovered.insert((point.counter_index, point.path.as_ref()));
                let _ = recovered.insert((point.counter_index, counter.path.as_str()));
            }
        }
        self.active_counter_failures
            .retain(|(index, path, _)| !recovered.contains(&(*index, path.as_ref())));
        for failure in &sample.metadata.failures {
            warnings += usize::from(self.log_failure(failure)?);
        }
        // Buffered failures can predate successful cleanup; suppress them before pruning.
        self.active_counter_failures.retain(|(index, path, _)| {
            retained.contains(&(*index, path.as_ref()))
                || self
                    .configs
                    .get(*index)
                    .is_some_and(|config| path.as_ref() == config.path)
        });
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
        Ok(warnings)
    }

    pub(super) fn log_error(&mut self, error: &Error) -> Result<usize, Error> {
        Ok(usize::from(self.log_query_failure(
            error.reason(),
            &error.bounded_detail(),
        )))
    }

    pub(super) fn log_query_failures(
        &mut self,
        metadata: &PendingMetadata,
    ) -> Result<usize, Error> {
        let mut warnings = 0;
        for failure in &metadata.failures {
            let _ = self
                .configs
                .get(failure.counter_index)
                .ok_or(Error::InvalidSample(
                    "query diagnostic references missing configured counter",
                ))?;
            warnings += usize::from(self.log_query_failure(failure.reason, &failure.error));
        }
        Ok(warnings)
    }

    fn log_query_failure(&mut self, reason: &str, error: &str) -> bool {
        if reason == "query_recovery_pending" || self.query_failure_active {
            return false;
        }
        otel_arrow_dfe_telemetry::otel_warn!(
            "windows_perf_counters_receiver.query.fail",
            node = self.node.as_str(),
            configured_counters = self.configs.len() as u64,
            reason,
            error
        );
        self.query_failure_active = true;
        true
    }

    fn log_failure(&mut self, failure: &SampleFailure) -> Result<bool, Error> {
        let counter = self
            .configs
            .get(failure.counter_index)
            .ok_or(Error::InvalidSample(
                "diagnostic failure references missing configured counter",
            ))?;
        if failure.reason == "query_recovery_pending" {
            return Ok(false);
        }
        // Query owns selector and comparison-identity warning episodes.
        let query_owned = matches!(
            failure.reason,
            "selector_unresolved" | "native_identity_unavailable"
        );
        let key = (
            failure.counter_index,
            Arc::clone(&failure.path),
            failure.reason,
        );
        let emitted = query_owned || !self.active_counter_failures.contains(&key);
        if emitted {
            otel_arrow_dfe_telemetry::otel_warn!(
                "windows_perf_counters_receiver.counter.fail",
                node = self.node.as_str(),
                path_template = counter.path.as_str(),
                reason = failure.reason,
                error = failure.error.as_str()
            );
        }
        if !query_owned {
            let _ = self.active_counter_failures.insert(key);
        }
        Ok(emitted)
    }
}
