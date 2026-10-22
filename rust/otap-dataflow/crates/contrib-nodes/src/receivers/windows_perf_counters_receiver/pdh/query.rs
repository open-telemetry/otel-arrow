// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Query-owned counters, sampling histories, discovery reconciliation, and retry state.

use super::super::config::{CounterConfig, MetricKind};
use super::super::model::{
    ExpansionOverflow, InstanceIdentity, Number, PendingMetadata, Sample, SampleFailure,
    SampleMember, SamplePoint, SampleValue, scale_double, scale_integer,
};
use super::counter_type::{CounterKind, classify_native_type};
use super::error::{Error, WORKER_FAILURE_PATH, sample_failure};
use super::expansion::{
    CounterKey, DiscoveryBudget, ExpansionAllocation, InstanceGroupKey,
    cleanup_aggregation_instances, expansion_allocation, instance_groups, keyed_expanded_paths,
    refresh_changes_with_group_resets,
};
use super::native::NativePdh;
use super::timing::{cumulative_start_time, positive_timestamp};
use super::{CounterLanguage, PdhApi, ValueFormat};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use windows_sys::Win32::System::Performance::PDH_MAX_COUNTER_PATH;

/// The query owns every added counter; closing it releases all handles.
pub(super) struct Query<A: PdhApi = NativePdh> {
    api: A,
    pub(super) start_time_unix_nano: i64,
    pub(super) previous_timestamp_unix_nano: i64,
    configs: Vec<CounterConfig>,
    counters: BTreeMap<CounterKey, CounterHandle<A::Counter>>,
    pending_removals: BTreeMap<CounterKey, PendingRemoval<A::Counter>>,
    retry_states: BTreeMap<CounterKey, RetryState>,
    pub(super) pending: PendingMetadata,
    exact_retry_cap: Duration,
    wildcard_refresh_interval: Duration,
    max_instances_per_wildcard: usize,
    max_native_counters: usize,
    exact_counter_reservations: usize,
    next_wildcard_admission_order: u64,
    pub(super) last_wildcard_refresh: Instant,
    initial_wildcard_refresh: bool,
    pub(super) last_object_refresh: Instant,
    selector_failures: BTreeSet<usize>,
    object_refresh_selectors: BTreeSet<usize>,
    comparison_mappings: BTreeSet<usize>,
    identity_failures: BTreeSet<usize>,
    exact_identities: BTreeMap<CounterKey, usize>,
}

impl<A: PdhApi> Query<A> {
    pub(super) fn open_with_seed(
        api: A,
        configs: Vec<CounterConfig>,
        exact_retry_cap: Duration,
        wildcard_refresh_interval: Duration,
        max_instances_per_wildcard: usize,
        max_native_counters: usize,
        seed: Option<&QueryRecreationSeed>,
    ) -> Result<Self, Error> {
        let mut api = api;
        let exact_counter_reservations = configs
            .iter()
            .filter(|config| !config.instance_config.is_wildcard())
            .count();
        if exact_counter_reservations > max_native_counters {
            return Err(Error::InvalidSample(
                "query native-counter limit is below its exact-counter reservation",
            ));
        }
        let (start_time_unix_nano, previous_timestamp_unix_nano) = if let Some(seed) = seed {
            (seed.start_time_unix_nano, seed.previous_timestamp_unix_nano)
        } else {
            let timestamp = positive_timestamp(api.timestamp()?)?;
            (timestamp, timestamp)
        };
        let comparison_mappings = configs
            .iter()
            .filter(|config| config.instance_config.is_wildcard())
            .map(|config| config.source_mapping_index)
            .collect();
        let mut query = Self {
            api,
            start_time_unix_nano,
            previous_timestamp_unix_nano,
            configs,
            counters: BTreeMap::new(),
            pending_removals: BTreeMap::new(),
            retry_states: BTreeMap::new(),
            pending: PendingMetadata::default(),
            exact_retry_cap,
            wildcard_refresh_interval,
            max_instances_per_wildcard,
            max_native_counters,
            exact_counter_reservations,
            next_wildcard_admission_order: 0,
            last_wildcard_refresh: seed
                .map_or_else(Instant::now, |seed| seed.last_wildcard_refresh),
            initial_wildcard_refresh: seed.is_none(),
            last_object_refresh: seed.map_or_else(Instant::now, |seed| seed.last_object_refresh),
            selector_failures: seed
                .map_or_else(BTreeSet::new, |seed| seed.selector_failures.clone()),
            object_refresh_selectors: seed
                .map_or_else(BTreeSet::new, |seed| seed.object_refresh_selectors.clone()),
            comparison_mappings,
            identity_failures: seed
                .map_or_else(BTreeSet::new, |seed| seed.identity_failures.clone()),
            exact_identities: BTreeMap::new(),
        };
        query.initialize_exact_counters()?;
        query.ensure_available_or_recoverable(&query.pending)?;
        let mut metadata = PendingMetadata::default();
        query.refresh_exact_identities(&mut metadata)?;
        if let Some(seed) = seed {
            query.initialize_seeded_wildcards(seed, &mut metadata)?;
        }
        query.pending.merge(metadata);
        Ok(query)
    }

    fn has_wildcard_configs(&self) -> bool {
        self.configs
            .iter()
            .any(|config| config.instance_config.is_wildcard())
    }

    fn ensure_available_or_recoverable(&self, metadata: &PendingMetadata) -> Result<(), Error> {
        if self.counters.is_empty()
            && self.retry_states.is_empty()
            && self.pending_removals.is_empty()
            && !self.has_wildcard_configs()
        {
            let detail = metadata.failures.last().map_or_else(
                || "no installed counters, retries or wildcard discovery".to_owned(),
                |failure| {
                    format!("{} for {}: {}", failure.reason, failure.path, failure.error)
                        .chars()
                        .take(512)
                        .collect()
                },
            );
            return Err(Error::NoUsableCounters { detail });
        }
        Ok(())
    }

    fn initialize_seeded_wildcards(
        &mut self,
        seed: &QueryRecreationSeed,
        metadata: &mut PendingMetadata,
    ) -> Result<(), Error> {
        let now = Instant::now();
        for (key, target) in &seed.wildcard_targets {
            if let Some(admission_order) = target.admission_order {
                self.next_wildcard_admission_order = self
                    .next_wildcard_admission_order
                    .max(admission_order.saturating_add(1));
            }
            let _ = self.try_add_target(key.clone(), target.clone(), now, metadata)?;
        }
        Ok(())
    }

    pub(super) fn recreation_seed(&self) -> QueryRecreationSeed {
        let wildcard_targets = self
            .counter_targets()
            .filter_map(|(key, target)| {
                target.origin.is_wildcard().then_some((key.clone(), target))
            })
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect();
        QueryRecreationSeed {
            start_time_unix_nano: self.start_time_unix_nano,
            previous_timestamp_unix_nano: self.previous_timestamp_unix_nano,
            last_wildcard_refresh: self.last_wildcard_refresh,
            wildcard_targets,
            last_object_refresh: self.last_object_refresh,
            selector_failures: self.selector_failures.clone(),
            object_refresh_selectors: self.object_refresh_selectors.clone(),
            identity_failures: self.identity_failures.clone(),
        }
    }

    pub(super) fn admitted_members(&self) -> Vec<SampleMember> {
        let mut members = self
            .counter_targets()
            .map(|(key, target)| (key, target.member()))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect::<Vec<_>>();
        members.sort_by(|left, right| {
            left.counter_index
                .cmp(&right.counter_index)
                .then_with(|| left.path.cmp(&right.path))
        });
        members
    }

    fn counter_targets(&self) -> impl Iterator<Item = (&CounterKey, CounterTarget)> {
        self.counters
            .iter()
            .map(|(key, counter)| (key, counter.target()))
            .chain(
                self.retry_states
                    .iter()
                    .map(|(key, state)| (key, state.target.clone())),
            )
            .chain(self.pending_removals.iter().filter_map(|(key, removal)| {
                removal.retry_target.clone().map(|target| (key, target))
            }))
    }

    fn initialize_exact_counters(&mut self) -> Result<(), Error> {
        for config_index in 0..self.configs.len() {
            self.api.check_running()?;
            if self.configs[config_index].instance_config.is_wildcard() {
                continue;
            }
            let path = Arc::<str>::from(self.configs[config_index].path.as_str());
            let key = CounterKey::new(self.configs[config_index].source_mapping_index, &path);
            let instance = if self.configs[config_index].instance_config.has_instance() {
                match self.api.parse_instance(&path) {
                    Ok(instance) => Some(Arc::new(instance)),
                    Err(error) if error.is_stop() => return Err(error),
                    Err(error) => {
                        let mut failure = sample_failure(config_index, &error);
                        failure.diagnostic_only = true;
                        self.pending.failures.push(failure);
                        None
                    }
                }
            } else {
                None
            };
            let target = CounterTarget {
                config_index,
                path,
                origin: CounterOrigin::Exact,
                instance: instance.clone(),
                emit_instance_attribute: self.configs[config_index]
                    .instance_config
                    .emit_attribute()
                    && instance.is_some(),
                admission_order: None,
                removal_counted: false,
            };
            if let Err(error) = self.add_counter(target.clone()) {
                if error.is_stop() || error.requires_immediate_query_rebuild() {
                    return Err(error);
                }
                self.pending.diagnostics.counter_add_failures += 1;
                let failure = if !error.is_permanent_counter_error() {
                    self.record_selector_failure(config_index, &error)
                } else {
                    Some(sample_failure(config_index, &error))
                };
                self.pending.failures.extend(failure);
                if !error.is_permanent_counter_error() {
                    self.schedule_retry(key, target, Instant::now());
                }
            }
        }
        Ok(())
    }

    fn add_counter(&mut self, target: CounterTarget) -> Result<(), Error> {
        self.api.check_running()?;
        let language = if target.origin.is_wildcard() {
            CounterLanguage::Localized
        } else {
            CounterLanguage::English
        };
        let counter = self.api.add(&target.path, language)?;
        self.api.check_running()?;
        let kind = match self
            .api
            .counter_type(&counter, &target.path)
            .and_then(|native_type| {
                classify_native_type(&target.path, native_type).map_err(Error::from)
            }) {
            Ok(kind) => kind,
            Err(error) => {
                return Err(setup_error_with_cleanup(
                    &mut self.api,
                    &counter,
                    &target.path,
                    error,
                ));
            }
        };
        let key = CounterKey::new(
            self.configs[target.config_index].source_mapping_index,
            &target.path,
        );
        if !target.origin.is_wildcard() {
            let _ = self.selector_failures.remove(&target.config_index);
            let _ = self.object_refresh_selectors.remove(&target.config_index);
        }
        let _ = self.counters.insert(
            key,
            CounterHandle {
                config_index: target.config_index,
                path: target.path,
                handle: counter,
                kind,
                scale_power10: self.configs[target.config_index].scale_power10,
                history: CounterHistory::default(),
                origin: target.origin,
                instance: target.instance,
                emit_instance_attribute: target.emit_instance_attribute,
                admission_order: target.admission_order,
                removal_counted: target.removal_counted,
                native_identity: None,
            },
        );
        Ok(())
    }

    fn schedule_retry(&mut self, key: CounterKey, target: CounterTarget, now: Instant) {
        let cap = target
            .origin
            .retry_cap(self.exact_retry_cap, self.wildcard_refresh_interval);
        schedule_retry_state(&mut self.retry_states, key, target, now, cap);
    }

    fn try_add_target(
        &mut self,
        key: CounterKey,
        target: CounterTarget,
        now: Instant,
        metadata: &mut PendingMetadata,
    ) -> Result<bool, Error> {
        if target.origin.is_wildcard()
            && self.wildcard_matches_exact(target.config_index, &target.path, metadata)?
                == Some(true)
        {
            let _ = self.retry_states.remove(&key);
            return Ok(false);
        }
        match self.add_counter(target.clone()) {
            Ok(()) => {
                if self.retry_states.remove(&key).is_some() {
                    metadata.diagnostics.retry_recoveries += 1;
                }
                if !target.origin.is_wildcard()
                    && self.comparison_mappings.contains(&key.source_mapping_index)
                    && self.read_exact_identity(&key, metadata)?
                {
                    self.reconcile_exact_overlaps(metadata)?;
                }
                Ok(true)
            }
            Err(error) if error.is_stop() => Err(error),
            Err(error) => {
                metadata.diagnostics.counter_add_failures += 1;
                let failure = if !target.origin.is_wildcard() && !error.is_permanent_counter_error()
                {
                    self.record_selector_failure(target.config_index, &error)
                } else {
                    Some(sample_failure(target.config_index, &error))
                };
                metadata.failures.extend(failure);
                if error.requires_immediate_query_rebuild() {
                    return Err(error);
                }
                if error.is_permanent_counter_error() {
                    let _ = self.retry_states.remove(&key);
                } else {
                    self.schedule_retry(key, target, now);
                }
                Ok(false)
            }
        }
    }

    fn retry_counters(
        &mut self,
        now: Instant,
        metadata: &mut PendingMetadata,
    ) -> Result<(), Error> {
        let due = self
            .retry_states
            .iter()
            .filter(|(_, state)| state.next_attempt <= now)
            .map(|(key, state)| (key.clone(), state.target.clone()))
            .collect::<Vec<_>>();
        for (key, target) in due {
            metadata.diagnostics.retry_attempts += 1;
            let _ = self.try_add_target(key, target, now, metadata)?;
        }
        Ok(())
    }

    fn refresh_wildcards(&mut self, metadata: &mut PendingMetadata) -> Result<(), Error> {
        metadata.diagnostics.discovery_refreshes += 1;
        self.refresh_exact_identities(metadata)?;
        let refresh_interval = self.wildcard_refresh_interval.max(Duration::from_secs(600));
        if !self.object_refresh_selectors.is_empty()
            && self.last_object_refresh.elapsed() >= refresh_interval
        {
            self.last_object_refresh = Instant::now();
            match self.api.refresh_objects() {
                Ok(_) => {}
                Err(error) if error.is_stop() => return Err(error),
                Err(_) => metadata.diagnostics.discovery_failures += 1,
            }
        }
        let wildcard_configs = self
            .configs
            .iter()
            .enumerate()
            .filter_map(|(config_index, config)| {
                config.instance_config.is_wildcard().then_some(config_index)
            })
            .collect::<Vec<_>>();
        let mut discoveries = Vec::new();
        let mut desired = DesiredCounters::new();
        let mut budget = DiscoveryBudget::new();
        for config_index in wildcard_configs {
            self.api.check_running()?;
            match budget.reserve_selector().and_then(|()| {
                self.api
                    .expand(&self.configs[config_index].path, &mut budget)
            }) {
                Ok(paths) => {
                    let _ = self.selector_failures.remove(&config_index);
                    let _ = self.object_refresh_selectors.remove(&config_index);
                    discoveries.push(WildcardDiscovery {
                        config_index,
                        paths,
                    });
                }
                Err(error) if error.is_stop() => return Err(error),
                Err(error) => {
                    let reserved = desired.len();
                    self.reserve_failed_discovery(
                        config_index,
                        &mut desired,
                        reserved,
                        metadata,
                        &error,
                    );
                }
            }
        }
        let reserved = desired.len();
        self.prepare_successful_discoveries(discoveries, &mut desired, reserved, metadata)?;
        self.last_wildcard_refresh = Instant::now();
        self.initial_wildcard_refresh = false;
        Ok(())
    }

    pub(super) fn wildcard_refresh_due(&self) -> bool {
        (self.initial_wildcard_refresh && self.has_wildcard_configs())
            || ((self.has_wildcard_configs() || !self.object_refresh_selectors.is_empty())
                && self.last_wildcard_refresh.elapsed() >= self.wildcard_refresh_interval)
    }

    pub(super) fn refresh_all_wildcards_managed(&mut self) -> Result<(), Error> {
        if !self.wildcard_refresh_due() {
            return Ok(());
        }
        let mut metadata = PendingMetadata::default();
        let result = self.refresh_wildcards(&mut metadata);
        self.pending.merge(metadata);
        result
    }

    fn record_selector_failure(
        &mut self,
        config_index: usize,
        error: &Error,
    ) -> Option<SampleFailure> {
        if !matches!(
            error,
            Error::WildcardCleanup {
                lookup_failed: false,
                ..
            }
        ) {
            let _ = self.object_refresh_selectors.insert(config_index);
        }
        self.selector_failures
            .insert(config_index)
            .then(|| SampleFailure {
                counter_index: config_index,
                path: Arc::from(self.configs[config_index].path.as_str()),
                reason: "selector_unresolved",
                error: error.to_string(),
                diagnostic_only: true,
            })
    }

    fn record_identity_failure(
        &mut self,
        config_index: usize,
        error: &Error,
        metadata: &mut PendingMetadata,
    ) {
        metadata.diagnostics.discovery_failures += 1;
        if self.identity_failures.insert(config_index) {
            metadata.failures.push(SampleFailure {
                counter_index: config_index,
                path: Arc::from(self.configs[config_index].path.as_str()),
                reason: "native_identity_unavailable",
                error: error.bounded_detail(),
                diagnostic_only: true,
            });
        }
    }

    fn read_exact_identity(
        &mut self,
        key: &CounterKey,
        metadata: &mut PendingMetadata,
    ) -> Result<bool, Error> {
        self.api.check_running()?;
        let counter = &self.counters[key];
        let config_index = counter.config_index;
        let identity = self
            .api
            .full_path(&counter.handle, &counter.path)
            .and_then(|path| self.api.comparison_path(&path));
        self.api.check_running()?;
        match identity {
            Ok(path) => {
                let identity = CounterKey::new(key.source_mapping_index, &path);
                *self.exact_identities.entry(identity.clone()).or_default() += 1;
                self.counters
                    .get_mut(key)
                    .expect("identity read retains its counter")
                    .native_identity = Some(identity);
                let _ = self.identity_failures.remove(&config_index);
                Ok(true)
            }
            Err(error) if error.is_stop() || error.requires_immediate_query_rebuild() => Err(error),
            Err(error) => {
                self.record_identity_failure(config_index, &error, metadata);
                Ok(false)
            }
        }
    }

    fn refresh_exact_identities(&mut self, metadata: &mut PendingMetadata) -> Result<(), Error> {
        let missing = self
            .counters
            .iter()
            .filter(|(key, counter)| {
                !counter.origin.is_wildcard()
                    && counter.native_identity.is_none()
                    && self.comparison_mappings.contains(&key.source_mapping_index)
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        let mut changed = false;
        for key in missing {
            changed |= self.read_exact_identity(&key, metadata)?;
        }
        if changed {
            self.reconcile_exact_overlaps(metadata)?;
        }
        Ok(())
    }

    fn wildcard_matches_exact(
        &mut self,
        config_index: usize,
        path: &str,
        metadata: &mut PendingMetadata,
    ) -> Result<Option<bool>, Error> {
        let key = CounterKey::new(self.configs[config_index].source_mapping_index, path);
        if self
            .counters
            .get(&key)
            .is_some_and(|counter| !counter.origin.is_wildcard())
            || self
                .retry_states
                .get(&key)
                .is_some_and(|state| !state.target.origin.is_wildcard())
            || self
                .pending_removals
                .get(&key)
                .and_then(|removal| removal.retry_target.as_ref())
                .is_some_and(|target| !target.origin.is_wildcard())
        {
            return Ok(Some(true));
        }
        if !self
            .exact_identities
            .range(CounterKey::new(key.source_mapping_index, "")..)
            .next()
            .is_some_and(|(identity, _)| identity.source_mapping_index == key.source_mapping_index)
        {
            return Ok(Some(false));
        }
        self.api.check_running()?;
        let identity = self.api.comparison_path(path);
        self.api.check_running()?;
        match identity {
            Ok(path) => {
                let identity = CounterKey::new(key.source_mapping_index, &path);
                Ok(Some(self.exact_identities.contains_key(&identity)))
            }
            Err(error) if error.is_stop() || error.requires_immediate_query_rebuild() => Err(error),
            Err(error) => {
                self.record_identity_failure(config_index, &error, metadata);
                Ok(None)
            }
        }
    }

    fn reconcile_exact_overlaps(&mut self, metadata: &mut PendingMetadata) -> Result<(), Error> {
        let targets = self
            .counter_targets()
            .filter(|(_, target)| target.origin.is_wildcard())
            .map(|(key, target)| (key.clone(), target))
            .collect::<Vec<_>>();
        let mut wanted = BTreeSet::new();
        for (key, target) in targets {
            if self.wildcard_matches_exact(target.config_index, &target.path, metadata)?
                != Some(true)
            {
                let _ = wanted.insert(key);
            }
        }
        let removed = self
            .counters
            .iter()
            .filter(|(key, counter)| counter.origin.is_wildcard() && !wanted.contains(*key))
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        self.discard_unwanted_wildcard_retries(&wanted, metadata);
        self.remove_wildcard_counters(&removed, &wanted, metadata)
    }

    fn reserve_failed_discovery(
        &mut self,
        config_index: usize,
        desired: &mut DesiredCounters,
        expanded_total: usize,
        metadata: &mut PendingMetadata,
        error: &Error,
    ) {
        if !matches!(
            error,
            Error::WildcardCleanup {
                lookup_failed: false,
                ..
            }
        ) {
            metadata.diagnostics.discovery_failures += 1;
        }
        metadata
            .failures
            .extend(self.record_selector_failure(config_index, error));
        let mut retained = self
            .counter_targets()
            .filter(|(_, target)| {
                target.config_index == config_index && target.origin.is_wildcard()
            })
            .filter_map(|(key, target)| {
                target.instance.map(|instance| {
                    (
                        target.admission_order.unwrap_or(u64::MAX),
                        key.clone(),
                        DesiredCounter::new(
                            config_index,
                            target.path,
                            instance,
                            target.emit_instance_attribute,
                        ),
                    )
                })
            })
            .collect::<Vec<_>>();
        retained.sort_by_key(|(admission_order, _, _)| *admission_order);
        let discovered = retained.len();
        let allocation = expansion_allocation(
            discovered,
            self.max_instances_per_wildcard,
            self.remaining_expansion_capacity(expanded_total),
        );
        metadata.record_expansion_overflows(
            config_index,
            discovered,
            allocation,
            retained
                .iter()
                .skip(allocation.retained)
                .map(|(_, _, counter)| Arc::clone(&counter.path)),
        );
        for (_, key, counter) in retained.into_iter().take(allocation.retained) {
            let _ = desired.insert(key, counter);
        }
    }

    fn prepare_successful_discoveries(
        &mut self,
        discoveries: Vec<WildcardDiscovery>,
        desired: &mut DesiredCounters,
        expanded_total: usize,
        metadata: &mut PendingMetadata,
    ) -> Result<(), Error> {
        let mut prepared = Vec::new();
        for discovery in discoveries {
            self.api.check_running()?;
            let paths = discovery.paths;
            let config_index = discovery.config_index;
            let mut paths =
                keyed_expanded_paths(self.configs[config_index].source_mapping_index, paths)
                    .into_iter()
                    .map(|(_, path)| path)
                    .collect::<Vec<_>>();
            let aggregation_only = cleanup_aggregation_instances(
                &mut paths,
                self.configs[config_index]
                    .instance_config
                    .excluded_aggregation(),
                |path| {
                    self.api.check_running()?;
                    self.api.parse_instance(path)
                },
            )?;
            if paths.is_empty() && self.initial_wildcard_refresh {
                metadata.failures.push(SampleFailure {
                    counter_index: config_index,
                    path: Arc::from(self.configs[config_index].path.as_str()),
                    reason: "empty_expansion",
                    error: "startup wildcard expansion has no members".to_owned(),
                    diagnostic_only: true,
                });
            }
            let oversized = paths
                .iter()
                .filter(|path| path.encode_utf16().count() >= PDH_MAX_COUNTER_PATH as usize)
                .count();
            if oversized > 0 {
                metadata.diagnostics.counter_add_failures +=
                    u64::try_from(oversized).unwrap_or(u64::MAX);
                metadata.failures.push(SampleFailure {
                    counter_index: config_index,
                    path: Arc::from(self.configs[config_index].path.as_str()),
                    reason: "expanded_path_too_long",
                    error: format!(
                        "{oversized} expanded paths reached the PDH {PDH_MAX_COUNTER_PATH}-unit limit"
                    ),
                    diagnostic_only: false,
                });
                paths.retain(|path| path.encode_utf16().count() < PDH_MAX_COUNTER_PATH as usize);
            }
            let paths = paths
                .into_iter()
                .map(|path| {
                    (
                        CounterKey::new(self.configs[config_index].source_mapping_index, &path),
                        path,
                    )
                })
                .filter(|(key, _)| {
                    !self
                        .counters
                        .get(key)
                        .is_some_and(|counter| !counter.origin.is_wildcard())
                        && !self
                            .retry_states
                            .get(key)
                            .is_some_and(|state| !state.target.origin.is_wildcard())
                })
                .collect::<Vec<_>>();
            let mut distinct = Vec::with_capacity(paths.len());
            let mut identity_failed = false;
            for (key, path) in paths {
                match self.wildcard_matches_exact(config_index, &path, metadata)? {
                    Some(true) => {}
                    result => {
                        identity_failed |= result.is_none();
                        distinct.push((key, path));
                    }
                }
            }
            if !identity_failed {
                let _ = self.identity_failures.remove(&config_index);
            }
            let paths = distinct;
            let discovered_keys = paths
                .iter()
                .map(|(key, _)| key.clone())
                .collect::<BTreeSet<_>>();
            self.restore_rediscovered_pending_removals(&discovered_keys);
            prepared.push(PreparedDiscovery {
                config_index,
                aggregation_only,
                paths,
            });
        }
        let known_groups = self.wildcard_instance_groups();
        let discovered_keys = desired
            .keys()
            .cloned()
            .chain(
                prepared
                    .iter()
                    .flat_map(|discovery| discovery.paths.iter().map(|(key, _)| key.clone())),
            )
            .collect::<BTreeSet<_>>();
        let disappeared = self
            .counters
            .iter()
            .filter_map(|(key, counter)| {
                (counter.origin.is_wildcard() && !discovered_keys.contains(key))
                    .then_some(key.clone())
            })
            .collect::<Vec<_>>();
        self.discard_unwanted_wildcard_retries(&discovered_keys, metadata);
        self.remove_wildcard_counters(&disappeared, &discovered_keys, metadata)?;
        let admitted = self
            .counter_targets()
            .filter_map(|(key, target)| {
                target.origin.is_wildcard().then_some((
                    key,
                    (
                        target.admission_order.unwrap_or(u64::MAX),
                        target.config_index,
                    ),
                ))
            })
            .collect::<BTreeMap<_, _>>();
        let mut candidates = prepared
            .iter()
            .enumerate()
            .flat_map(|(discovery_index, discovery)| {
                let admitted = &admitted;
                let mapping_index = self.configs[discovery.config_index].source_mapping_index;
                discovery
                    .paths
                    .iter()
                    .enumerate()
                    .map(move |(path_index, (key, path))| {
                        let admission = admitted.get(key).copied();
                        AdmissionCandidate {
                            priority: AdmissionPriority {
                                new_member: admission.is_none(),
                                admission_order: admission.map_or(0, |(order, _)| order),
                                mapping_index,
                                owner_changed: admission
                                    .is_some_and(|(_, owner)| owner != discovery.config_index),
                                key,
                                path,
                            },
                            discovery_index,
                            path_index,
                        }
                    })
            })
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        let mut results = vec![AdmissionResult::default(); prepared.len()];
        let mut remaining = self.remaining_expansion_capacity(expanded_total);
        let mut owned = desired.keys().cloned().collect::<BTreeSet<_>>();
        for candidate in candidates {
            let result = &mut results[candidate.discovery_index];
            let (key, path) = &prepared[candidate.discovery_index].paths[candidate.path_index];
            if owned.contains(key) {
                result.duplicates += 1;
                continue;
            }
            if result.selected.len() < self.max_instances_per_wildcard
                && remaining > 0
                && owned.insert(key.clone())
            {
                let _ = result.selected.insert(candidate.path_index);
                remaining -= 1;
            } else if result.omitted_examples.len() < 3 {
                result.omitted_examples.push(Arc::from(path.as_str()));
            }
        }
        for (discovery, result) in prepared.into_iter().zip(results) {
            let config_index = discovery.config_index;
            let discovered = discovery.paths.len() - result.duplicates;
            let per_wildcard_retained = discovered.min(self.max_instances_per_wildcard);
            metadata.record_expansion_overflows(
                config_index,
                discovered,
                ExpansionAllocation {
                    retained: result.selected.len(),
                    per_wildcard_omitted: discovered - per_wildcard_retained,
                    total_omitted: per_wildcard_retained.saturating_sub(result.selected.len()),
                },
                result.omitted_examples,
            );
            for (path_index, (key, path)) in discovery.paths.into_iter().enumerate() {
                if !result.selected.contains(&path_index) {
                    continue;
                }
                self.api.check_running()?;
                match self.api.parse_instance(&path) {
                    Ok(instance) => {
                        let emit_instance_attribute =
                            self.configs[config_index].instance_config.emit_attribute()
                                && !discovery.aggregation_only;
                        let _ = desired.insert(
                            key,
                            DesiredCounter::new(
                                config_index,
                                path,
                                Arc::new(instance),
                                emit_instance_attribute,
                            ),
                        );
                    }
                    Err(error) if error.is_stop() => return Err(error),
                    Err(error) => {
                        metadata.diagnostics.counter_add_failures += 1;
                        metadata.failures.push(sample_failure(config_index, &error));
                    }
                }
            }
        }
        self.reconcile_wildcards_with_groups(std::mem::take(desired), &known_groups, metadata)
    }

    fn restore_rediscovered_pending_removals(&mut self, discovered: &BTreeSet<CounterKey>) {
        let restored = self
            .pending_removals
            .iter()
            .filter(|(key, removal)| !removal.replacement_required && discovered.contains(*key))
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        for key in restored {
            let Some(mut removal) = self.pending_removals.remove(&key) else {
                continue;
            };
            removal.counter.reset_readiness();
            let _ = self.counters.insert(key, removal.counter);
        }
    }

    fn reconcile_wildcards_with_groups(
        &mut self,
        mut desired: DesiredCounters,
        known_groups: &BTreeMap<InstanceGroupKey, BTreeSet<CounterKey>>,
        metadata: &mut PendingMetadata,
    ) -> Result<(), Error> {
        for (key, target) in &desired {
            if let Some(counter) = self.counters.get_mut(key)
                && counter.origin.is_wildcard()
            {
                counter.config_index = target.config_index;
                counter.instance = Some(Arc::clone(&target.instance));
                counter.emit_instance_attribute = target.emit_instance_attribute;
            }
            if let Some(state) = self.retry_states.get_mut(key)
                && state.target.origin.is_wildcard()
            {
                state.target.config_index = target.config_index;
                state.target.instance = Some(Arc::clone(&target.instance));
                state.target.emit_instance_attribute = target.emit_instance_attribute;
            }
            if let Some(pending_target) = self
                .pending_removals
                .get_mut(key)
                .and_then(|removal| removal.retry_target.as_mut())
                && pending_target.origin.is_wildcard()
            {
                pending_target.config_index = target.config_index;
                pending_target.instance = Some(Arc::clone(&target.instance));
                pending_target.emit_instance_attribute = target.emit_instance_attribute;
            }
        }
        let changes = self.plan_wildcard_reconciliation(&desired, known_groups);
        self.discard_unwanted_wildcard_retries(&changes.wanted, metadata);
        self.remove_wildcard_counters(&changes.removed, &changes.wanted, metadata)?;
        self.prune_pending_failures(&desired);
        self.add_wildcard_counters(&changes.added, &changes.live, &mut desired, metadata)?;
        Ok(())
    }

    fn wildcard_instance_groups(&self) -> BTreeMap<InstanceGroupKey, BTreeSet<CounterKey>> {
        let instances = self
            .counter_targets()
            .filter(|(_, target)| target.origin.is_wildcard())
            .filter_map(|(key, target)| target.instance.map(|instance| (key, instance)))
            .collect::<Vec<_>>();
        instance_groups(
            instances
                .iter()
                .map(|(key, instance)| (*key, instance.as_ref())),
        )
    }

    fn plan_wildcard_reconciliation(
        &self,
        desired: &DesiredCounters,
        known_groups: &BTreeMap<InstanceGroupKey, BTreeSet<CounterKey>>,
    ) -> WildcardReconciliation {
        let live = self
            .counters
            .iter()
            .filter_map(|(key, counter)| counter.origin.is_wildcard().then_some(key.clone()))
            .collect::<BTreeSet<_>>();
        let retrying = self
            .retry_states
            .iter()
            .filter_map(|(key, state)| state.target.origin.is_wildcard().then_some(key.clone()))
            .collect::<BTreeSet<_>>();
        let wanted = desired.keys().cloned().collect::<BTreeSet<_>>();
        let wanted_groups = instance_groups(
            desired
                .iter()
                .map(|(key, counter)| (key, counter.instance.as_ref())),
        );
        let (removed, mut added) = refresh_changes_with_group_resets(
            &live,
            &retrying,
            &wanted,
            known_groups,
            &wanted_groups,
        );
        added.retain(|key| {
            !self
                .counters
                .get(key)
                .is_some_and(|counter| !counter.origin.is_wildcard())
                && !self
                    .retry_states
                    .get(key)
                    .is_some_and(|state| !state.target.origin.is_wildcard())
        });
        WildcardReconciliation {
            live,
            wanted,
            removed,
            added,
        }
    }

    fn discard_unwanted_wildcard_retries(
        &mut self,
        wanted: &BTreeSet<CounterKey>,
        metadata: &mut PendingMetadata,
    ) {
        let removed_retry_instances = self
            .retry_states
            .iter()
            .filter(|(key, state)| state.target.origin.is_wildcard() && !wanted.contains(*key))
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        metadata.diagnostics.instances_removed +=
            u64::try_from(removed_retry_instances.len()).unwrap_or(u64::MAX);
        for key in removed_retry_instances {
            let _ = self.retry_states.remove(&key);
        }
        for (key, removal) in &mut self.pending_removals {
            if removal.counter.origin.is_wildcard()
                && !wanted.contains(key)
                && removal.retry_target.take().is_some()
                && !removal.counter.removal_counted
            {
                metadata.diagnostics.instances_removed += 1;
                removal.counter.removal_counted = true;
            }
        }
    }

    fn remove_wildcard_counters(
        &mut self,
        removed: &[CounterKey],
        wanted: &BTreeSet<CounterKey>,
        metadata: &mut PendingMetadata,
    ) -> Result<(), Error> {
        for key in removed {
            self.api.check_running()?;
            let _ = self.retry_states.remove(key);
            let mut counter = self.counters.remove(key).ok_or(Error::InvalidSample(
                "wildcard refresh lost an existing counter",
            ))?;
            if !wanted.contains(key) && !counter.removal_counted {
                metadata.diagnostics.instances_removed += 1;
                counter.removal_counted = true;
            }
            if let Err(error) = self.api.remove(&counter.handle, &counter.path) {
                if !error.is_stop() {
                    metadata
                        .failures
                        .push(sample_failure(counter.config_index, &error));
                }
                let retry_target = wanted.contains(key).then(|| counter.target());
                let _ = self.pending_removals.insert(
                    key.clone(),
                    PendingRemoval::new(
                        counter,
                        retry_target,
                        Instant::now(),
                        self.exact_retry_cap,
                    ),
                );
                if error.is_stop() {
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    fn add_wildcard_counters(
        &mut self,
        added: &[CounterKey],
        live: &BTreeSet<CounterKey>,
        desired: &mut DesiredCounters,
        metadata: &mut PendingMetadata,
    ) -> Result<(), Error> {
        let now = Instant::now();
        for key in added {
            if self
                .pending_removals
                .get(key)
                .is_some_and(|removal| removal.retry_target.is_some())
            {
                continue;
            }
            let Some(DesiredCounter {
                config_index,
                path,
                instance,
                emit_instance_attribute,
            }) = desired.remove(key)
            else {
                return Err(Error::InvalidSample(
                    "wildcard admission lost a desired counter",
                ));
            };
            let admission_order = self.next_wildcard_admission_order;
            self.next_wildcard_admission_order =
                self.next_wildcard_admission_order.saturating_add(1);
            let target = CounterTarget {
                config_index,
                path,
                origin: CounterOrigin::Wildcard,
                instance: Some(instance),
                emit_instance_attribute,
                admission_order: Some(admission_order),
                removal_counted: false,
            };
            if let Some(removal) = self.pending_removals.get_mut(key) {
                removal.retry_target = Some(target);
                continue;
            }
            let was_retry = self.retry_states.contains_key(key);
            if self.try_add_target(key.clone(), target, now, metadata)?
                && !live.contains(key)
                && !was_retry
            {
                metadata.diagnostics.instances_added += 1;
            }
        }
        Ok(())
    }

    fn restore_pending_metadata(&mut self, metadata: PendingMetadata) {
        self.pending.merge(metadata);
    }

    fn prune_pending_failures(&mut self, desired: &DesiredCounters) {
        let retained = self
            .configs
            .iter()
            .enumerate()
            .map(|(index, config)| (index, config.path.as_str()))
            .chain(
                desired
                    .values()
                    .map(|counter| (counter.config_index, counter.path.as_ref())),
            )
            .chain(
                self.counters
                    .values()
                    .map(|counter| (counter.config_index, counter.path.as_ref())),
            )
            .chain(
                self.retry_states
                    .values()
                    .map(|state| (state.target.config_index, state.target.path.as_ref())),
            )
            .chain(
                self.pending_removals
                    .values()
                    .map(|removal| (removal.counter.config_index, removal.counter.path.as_ref())),
            )
            .chain(self.pending_removals.values().filter_map(|removal| {
                removal
                    .retry_target
                    .as_ref()
                    .map(|target| (target.config_index, target.path.as_ref()))
            }))
            .collect::<BTreeSet<_>>();
        self.pending.failures.retain(|failure| {
            failure.path.as_ref() == WORKER_FAILURE_PATH
                || self.selector_failures.contains(&failure.counter_index)
                || retained.contains(&(failure.counter_index, failure.path.as_ref()))
        });
    }

    fn restore_collection_error(&mut self, metadata: PendingMetadata, error: Error) -> Error {
        self.restore_pending_metadata(metadata);
        error
    }

    fn restart_sampling(&mut self) {
        restart_query_pdh_sampling(
            self.counters
                .values_mut()
                .map(|counter| &mut counter.history),
        );
    }

    fn restart_sampling_and_restore_collection_error(
        &mut self,
        metadata: PendingMetadata,
        error: Error,
    ) -> Error {
        self.restart_sampling();
        self.restore_collection_error(metadata, error)
    }

    pub(super) fn active_counter_count(&self) -> usize {
        self.counters.len()
    }

    pub(super) fn active_expanded_count(&self) -> usize {
        self.counters
            .values()
            .filter(|counter| counter.origin.is_wildcard())
            .count()
    }

    pub(super) fn prime(&mut self) -> Result<(), Error> {
        let result = self.collect().map(|sample| {
            self.pending.merge(sample.metadata);
        });
        self.pending.diagnostics.any_query_collected = false;
        for failure in &mut self.pending.failures {
            failure.diagnostic_only = true;
        }
        result
    }

    fn remaining_expansion_capacity(&self, allocated_expanded: usize) -> usize {
        self.max_native_counters
            .saturating_sub(self.exact_counter_reservations)
            .saturating_sub(
                self.pending_removals
                    .values()
                    .filter(|removal| {
                        removal.counter.origin.is_wildcard() && removal.retry_target.is_none()
                    })
                    .count(),
            )
            .saturating_sub(allocated_expanded)
    }

    fn retry_pending_removals(
        &mut self,
        now: Instant,
        metadata: &mut PendingMetadata,
    ) -> Result<(), Error> {
        let keys = self
            .pending_removals
            .iter()
            .filter(|(_, removal)| now >= removal.next_attempt)
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        for key in keys {
            self.api.check_running()?;
            let Some(mut removal) = self.pending_removals.remove(&key) else {
                continue;
            };
            match self
                .api
                .remove(&removal.counter.handle, &removal.counter.path)
            {
                Ok(()) => {
                    if let Some(target) = removal.retry_target {
                        self.schedule_retry(key, target, now);
                    }
                }
                Err(error) => {
                    if !error.is_stop() {
                        metadata
                            .failures
                            .push(sample_failure(removal.counter.config_index, &error));
                        removal.attempts = removal.attempts.saturating_add(1);
                        removal.next_attempt =
                            now + retry_delay(removal.attempts, self.exact_retry_cap);
                    }
                    let _ = self.pending_removals.insert(key, removal);
                    if error.is_stop() {
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn collect(&mut self) -> Result<Sample, Error> {
        self.collect_at(Instant::now())
    }

    pub(super) fn collect_at(&mut self, now: Instant) -> Result<Sample, Error> {
        self.api.check_running()?;
        self.refresh_all_wildcards_managed()?;
        let mut metadata = std::mem::take(&mut self.pending);
        metadata.diagnostics.any_query_collected = false;
        if let Err(error) = self.retry_pending_removals(now, &mut metadata) {
            return Err(self.restore_collection_error(metadata, error));
        }
        if let Err(error) = self.retry_counters(now, &mut metadata) {
            return Err(self.restore_collection_error(metadata, error));
        }
        if self.counters.is_empty() {
            if let Err(error) = self.ensure_available_or_recoverable(&metadata) {
                return Err(self.restore_collection_error(metadata, error));
            }
            let timestamp_unix_nano = match self.api.timestamp().and_then(positive_timestamp) {
                Ok(timestamp) => timestamp,
                Err(error) => {
                    return Err(self.restore_collection_error(metadata, error));
                }
            };
            let _ = cumulative_start_time(
                &mut self.start_time_unix_nano,
                &mut self.previous_timestamp_unix_nano,
                timestamp_unix_nano,
            );
            return Ok(Sample {
                timestamp_unix_nano,
                members: self.admitted_members(),
                points: Vec::new(),
                metadata,
            });
        }
        let has_data = match self.api.collect() {
            Ok(has_data) => has_data,
            Err(error) => {
                return Err(self.restart_sampling_and_restore_collection_error(metadata, error));
            }
        };
        if !has_data {
            self.restart_sampling();
        }
        let timestamp_unix_nano = match self.api.timestamp().and_then(positive_timestamp) {
            Ok(timestamp) => timestamp,
            Err(error) => {
                return Err(self.restart_sampling_and_restore_collection_error(metadata, error));
            }
        };
        let _ = cumulative_start_time(
            &mut self.start_time_unix_nano,
            &mut self.previous_timestamp_unix_nano,
            timestamp_unix_nano,
        );
        metadata.diagnostics.any_query_collected = true;
        let points = match self.collect_points(has_data, timestamp_unix_nano, &mut metadata) {
            Ok(points) => points,
            Err(error) => return Err(self.restore_collection_error(metadata, error)),
        };
        metadata.diagnostics.active_expanded_counters = self.active_expanded_count();
        Ok(Sample {
            timestamp_unix_nano,
            members: self.admitted_members(),
            points,
            metadata,
        })
    }

    fn collect_points(
        &mut self,
        has_data: bool,
        timestamp_unix_nano: i64,
        metadata: &mut PendingMetadata,
    ) -> Result<Vec<SamplePoint>, Error> {
        let mut points = Vec::with_capacity(self.counters.len());
        let mut retired = Vec::new();
        for (key, counter) in &mut self.counters {
            let metric_kind = self.configs[counter.config_index].metric_kind;
            let result = if has_data {
                counter.read(&mut self.api, metric_kind, self.start_time_unix_nano)
            } else {
                Ok((SampleValue::NoObservation, None))
            };
            let (value, point_start_time) = match result {
                Ok(result) => result,
                Err(error) if error.is_stop() => return Err(error),
                Err(error) => {
                    metadata
                        .failures
                        .push(sample_failure(counter.config_index, &error));
                    if error.is_invalid_counter_handle_read() {
                        retired.push((key.clone(), counter.target()));
                    }
                    continue;
                }
            };
            if matches!(value, SampleValue::Warming) {
                metadata.diagnostics.warmup_omissions += 1;
            }
            points.push(SamplePoint {
                counter_index: counter.config_index,
                path: counter.path.clone(),
                instance: counter.instance.clone(),
                emit_instance_attribute: counter.emit_instance_attribute,
                timestamp_unix_nano,
                start_time_unix_nano: point_start_time,
                value,
            });
        }
        points.sort_by(|left, right| {
            left.counter_index
                .cmp(&right.counter_index)
                .then_with(|| left.path.cmp(&right.path))
        });
        let now = Instant::now();
        for (key, target) in retired {
            let counter = self.counters.remove(&key).ok_or(Error::InvalidSample(
                "invalid-handle recovery lost an existing counter",
            ))?;
            if let Some(identity) = &counter.native_identity {
                let owners = self
                    .exact_identities
                    .get_mut(identity)
                    .expect("installed exact identity is indexed");
                *owners -= 1;
                if *owners == 0 {
                    let _ = self.exact_identities.remove(identity);
                }
            }
            if let Err(error) = self.api.remove(&counter.handle, &counter.path) {
                metadata
                    .failures
                    .push(sample_failure(counter.config_index, &error));
                let _ = self.pending_removals.insert(
                    key,
                    PendingRemoval::new(counter, Some(target), now, self.exact_retry_cap),
                );
                if error.is_stop() {
                    return Err(error);
                }
                continue;
            }
            self.schedule_retry(key, target, now);
        }
        Ok(points)
    }
}

fn setup_error_with_cleanup<A: PdhApi>(
    api: &mut A,
    counter: &A::Counter,
    path: &str,
    error: Error,
) -> Error {
    match api.remove(counter, path) {
        Ok(()) => error,
        Err(cleanup) => Error::with_cleanup_failure(path, error, cleanup),
    }
}

fn validate_positive_multi_base(path: &str, base: u32) -> Result<(), Error> {
    if base > 0 {
        Ok(())
    } else {
        Err(Error::Calculation {
            path: path.to_owned(),
            message: "multi-counter base denominator must be positive, got 0".to_owned(),
        })
    }
}

struct DesiredCounter {
    config_index: usize,
    path: Arc<str>,
    instance: Arc<InstanceIdentity>,
    emit_instance_attribute: bool,
}

impl DesiredCounter {
    fn new(
        config_index: usize,
        path: impl Into<Arc<str>>,
        instance: Arc<InstanceIdentity>,
        emit_instance_attribute: bool,
    ) -> Self {
        Self {
            config_index,
            path: path.into(),
            instance,
            emit_instance_attribute,
        }
    }
}

type DesiredCounters = BTreeMap<CounterKey, DesiredCounter>;

pub(super) struct QueryRecreationSeed {
    start_time_unix_nano: i64,
    pub(super) previous_timestamp_unix_nano: i64,
    last_wildcard_refresh: Instant,
    wildcard_targets: Vec<(CounterKey, CounterTarget)>,
    last_object_refresh: Instant,
    selector_failures: BTreeSet<usize>,
    object_refresh_selectors: BTreeSet<usize>,
    identity_failures: BTreeSet<usize>,
}

struct WildcardReconciliation {
    live: BTreeSet<CounterKey>,
    wanted: BTreeSet<CounterKey>,
    removed: Vec<CounterKey>,
    added: Vec<CounterKey>,
}

struct WildcardDiscovery {
    config_index: usize,
    paths: Vec<String>,
}

struct PreparedDiscovery {
    config_index: usize,
    aggregation_only: bool,
    paths: Vec<(CounterKey, String)>,
}

/// Field order defines retained age, mapping ownership and path tie-break priority.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct AdmissionPriority<'a> {
    new_member: bool,
    admission_order: u64,
    mapping_index: usize,
    owner_changed: bool,
    key: &'a CounterKey,
    path: &'a str,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct AdmissionCandidate<'a> {
    priority: AdmissionPriority<'a>,
    discovery_index: usize,
    path_index: usize,
}

#[derive(Clone, Default)]
struct AdmissionResult {
    selected: BTreeSet<usize>,
    duplicates: usize,
    omitted_examples: Vec<Arc<str>>,
}

#[derive(Clone)]
struct CounterTarget {
    config_index: usize,
    path: Arc<str>,
    origin: CounterOrigin,
    instance: Option<Arc<InstanceIdentity>>,
    emit_instance_attribute: bool,
    admission_order: Option<u64>,
    removal_counted: bool,
}

#[derive(Clone, Copy)]
enum CounterOrigin {
    Exact,
    Wildcard,
}

impl CounterOrigin {
    fn is_wildcard(self) -> bool {
        matches!(self, Self::Wildcard)
    }

    fn retry_cap(self, exact_retry_cap: Duration, wildcard_retry_cap: Duration) -> Duration {
        match self {
            Self::Exact => exact_retry_cap,
            Self::Wildcard => wildcard_retry_cap,
        }
    }
}

#[derive(Clone)]
struct RetryState {
    target: CounterTarget,
    attempts: u32,
    next_attempt: Instant,
}

pub(super) fn retry_delay(attempts: u32, cap: Duration) -> Duration {
    let exponent = attempts.saturating_sub(1).min(20);
    Duration::from_secs(1_u64 << exponent).min(cap)
}

fn schedule_retry_state(
    states: &mut BTreeMap<CounterKey, RetryState>,
    key: CounterKey,
    target: CounterTarget,
    now: Instant,
    cap: Duration,
) {
    let attempts = states
        .get(&key)
        .map_or(1, |state| state.attempts.saturating_add(1));
    let _ = states.insert(
        key,
        RetryState {
            target,
            attempts,
            next_attempt: now + retry_delay(attempts, cap),
        },
    );
}

impl PendingMetadata {
    fn record_expansion_overflows(
        &mut self,
        counter_index: usize,
        discovered: usize,
        allocation: ExpansionAllocation,
        examples: impl IntoIterator<Item = Arc<str>>,
    ) {
        let omitted = allocation
            .per_wildcard_omitted
            .saturating_add(allocation.total_omitted);
        if omitted > 0 {
            self.overflows.push(ExpansionOverflow {
                counter_index,
                reason: "admission_limit",
                discovered,
                retained: allocation.retained,
                omitted,
                per_wildcard_omitted: allocation.per_wildcard_omitted,
                total_omitted: allocation.total_omitted,
                examples: examples.into_iter().take(3).collect(),
            });
        }
        self.diagnostics.instances_omitted_over_limit += u64::try_from(omitted).unwrap_or(u64::MAX);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CounterHistory {
    previous_base: Option<i64>,
}

impl CounterHistory {
    fn restart_pdh_sampling(&mut self) {
        self.previous_base = None;
    }
}

impl CounterTarget {
    fn member(&self) -> SampleMember {
        SampleMember {
            counter_index: self.config_index,
            path: Arc::clone(&self.path),
            instance: self.instance.clone(),
            emit_instance_attribute: self.emit_instance_attribute,
        }
    }
}

fn restart_query_pdh_sampling<'a>(histories: impl Iterator<Item = &'a mut CounterHistory>) {
    for history in histories {
        history.restart_pdh_sampling();
    }
}

struct CounterHandle<C> {
    config_index: usize,
    path: Arc<str>,
    handle: C,
    kind: CounterKind,
    scale_power10: i32,
    history: CounterHistory,
    origin: CounterOrigin,
    instance: Option<Arc<InstanceIdentity>>,
    emit_instance_attribute: bool,
    admission_order: Option<u64>,
    removal_counted: bool,
    native_identity: Option<CounterKey>,
}

struct PendingRemoval<C> {
    counter: CounterHandle<C>,
    retry_target: Option<CounterTarget>,
    // Rediscovery cannot cancel required retirement, even after the target disappears.
    replacement_required: bool,
    attempts: u32,
    next_attempt: Instant,
}

impl<C> PendingRemoval<C> {
    fn new(
        counter: CounterHandle<C>,
        retry_target: Option<CounterTarget>,
        now: Instant,
        retry_cap: Duration,
    ) -> Self {
        Self {
            counter,
            replacement_required: retry_target.is_some(),
            retry_target,
            attempts: 1,
            next_attempt: now + retry_delay(1, retry_cap),
        }
    }
}

impl<C> CounterHandle<C> {
    fn target(&self) -> CounterTarget {
        CounterTarget {
            config_index: self.config_index,
            path: self.path.clone(),
            origin: self.origin,
            instance: self.instance.clone(),
            emit_instance_attribute: self.emit_instance_attribute,
            admission_order: self.admission_order,
            removal_counted: self.removal_counted,
        }
    }

    fn reset_readiness(&mut self) {
        self.history.restart_pdh_sampling();
    }

    fn read(
        &mut self,
        api: &mut impl PdhApi<Counter = C>,
        metric_kind: MetricKind,
        query_start_time_unix_nano: i64,
    ) -> Result<(SampleValue, Option<i64>), Error> {
        if let Some(value) = self.prepare_raw_sample(api)? {
            return Ok((value, None));
        }

        let Some(number) = self.read_formatted_number(api)? else {
            return Ok((SampleValue::NoObservation, None));
        };
        let scaled = self.scale_number(number)?;
        let point_start_time = self.point_start_time(metric_kind, query_start_time_unix_nano);
        Ok((SampleValue::Value(scaled), point_start_time))
    }

    fn prepare_raw_sample(
        &mut self,
        api: &mut impl PdhApi<Counter = C>,
    ) -> Result<Option<SampleValue>, Error> {
        if matches!(
            self.kind,
            CounterKind::Direct | CounterKind::FormattedSingleSample
        ) {
            return Ok(None);
        }
        let raw = match api.raw(&self.handle, &self.path) {
            Ok(Some(raw)) => raw,
            Ok(None) => {
                self.reset_readiness();
                return Ok(Some(SampleValue::NoObservation));
            }
            Err(error) => {
                self.reset_readiness();
                return Err(error);
            }
        };
        let previous = self.history.previous_base.replace(raw.second_value);
        if self.kind == CounterKind::CalculatedTwoSampleWithMultiBase {
            validate_positive_multi_base(&self.path, raw.multi_count)?;
        }
        let Some(previous) = previous else {
            return Ok(Some(SampleValue::Warming));
        };
        if self.kind == CounterKind::CalculatedTwoSampleWithBase && raw.second_value <= previous {
            return Ok(Some(SampleValue::NoObservation));
        }
        Ok(None)
    }

    fn read_formatted_number(
        &mut self,
        api: &mut impl PdhApi<Counter = C>,
    ) -> Result<Option<Number>, Error> {
        let format = if self.kind == CounterKind::Direct {
            ValueFormat::Integer
        } else {
            ValueFormat::Double
        };
        match api.formatted(&self.handle, &self.path, format) {
            Ok(Some(number)) => Ok(Some(number)),
            Ok(None) => {
                self.reset_readiness();
                Ok(None)
            }
            Err(error) => {
                if self.kind == CounterKind::Delta || matches!(error, Error::CounterData { .. }) {
                    self.reset_readiness();
                }
                Err(error)
            }
        }
    }

    fn scale_number(&self, number: Number) -> Result<Number, Error> {
        match number {
            Number::Integer(value) => scale_integer(value, self.scale_power10),
            Number::Double(value) => scale_double(value, self.scale_power10),
        }
        .map_err(|source| Error::Scale {
            path: self.path.to_string(),
            source,
        })
    }

    fn point_start_time(
        &self,
        metric_kind: MetricKind,
        query_start_time_unix_nano: i64,
    ) -> Option<i64> {
        match metric_kind {
            MetricKind::Gauge => None,
            MetricKind::Sum => Some(query_start_time_unix_nano),
        }
    }
}

#[cfg(test)]
#[path = "tests/query_internal.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/query_support.rs"]
mod test_support;
