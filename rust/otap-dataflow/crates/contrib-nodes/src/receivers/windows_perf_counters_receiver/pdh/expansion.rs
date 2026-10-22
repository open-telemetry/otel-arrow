// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::model::InstanceIdentity;
use super::error::Error;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const DISCOVERY_REFRESH_BYTES: usize = 128 * 1024 * 1024;

/// Conservative allocation credits shared by every selector in one refresh.
///
/// Raw attempts are charged at capacity. Path credits bound retained candidates
/// to at most 16,384 and their decoded UTF-8 capacity to at most 8 MiB, reserving
/// additional copies and bookkeeping for admission. This bounds discovery work,
/// not process RSS or the separately bounded configured/admitted counter state.
pub(in super::super) struct DiscoveryBudget {
    remaining: usize,
}

impl DiscoveryBudget {
    pub(super) fn new() -> Self {
        Self {
            remaining: DISCOVERY_REFRESH_BYTES,
        }
    }

    #[cfg(test)]
    pub(super) fn limited(bytes: usize) -> Self {
        Self { remaining: bytes }
    }

    pub(super) fn reserve(&mut self, bytes: usize) -> Result<(), Error> {
        self.remaining = self
            .remaining
            .checked_sub(bytes)
            .ok_or(Error::DiscoveryMemoryLimit)?;
        Ok(())
    }

    pub(super) fn reserve_selector(&mut self) -> Result<(), Error> {
        self.reserve(4096)
    }

    pub(super) fn reserve_path(&mut self, utf16_units: usize) -> Result<usize, Error> {
        let utf8_capacity = utf16_units
            .checked_mul(3)
            .ok_or(Error::DiscoveryMemoryLimit)?;
        // Include decoded storage and headroom for the path/key copies made by
        // normalization, ownership sets, desired membership and admission.
        // The fixed charge covers vector growth, candidate records and tree
        // nodes. Credits are never refunded, including failed/raw allocations,
        // so earlier results and transient growth cannot reuse the same budget.
        let bytes = utf8_capacity
            .checked_mul(16)
            .and_then(|bytes| bytes.checked_add(8192))
            .ok_or(Error::DiscoveryMemoryLimit)?;
        self.reserve(bytes)?;
        Ok(utf8_capacity)
    }
}

#[cfg(test)]
pub(super) use test_support::parse_instance;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CounterKey {
    pub(super) source_mapping_index: usize,
    normalized_path: String,
}

impl CounterKey {
    pub(super) fn new(source_mapping_index: usize, path: &str) -> Self {
        Self {
            source_mapping_index,
            normalized_path: local_counter_path(path).to_lowercase(),
        }
    }
}

fn local_counter_path(path: &str) -> &str {
    let Some(rest) = path.strip_prefix(r"\\") else {
        return path;
    };
    match rest.find('\\') {
        Some(separator) if separator > 0 => &rest[separator..],
        _ => path,
    }
}

pub(super) fn normalize_expanded_path(path: String) -> String {
    let local_path = local_counter_path(&path);
    if local_path.len() == path.len() {
        path
    } else {
        local_path.to_owned()
    }
}

pub(super) fn keyed_expanded_paths(
    source_mapping_index: usize,
    paths: Vec<String>,
) -> Vec<(CounterKey, String)> {
    let mut keyed = paths
        .into_iter()
        .map(|path| {
            let path = normalize_expanded_path(path);
            (CounterKey::new(source_mapping_index, &path), path)
        })
        .collect::<Vec<_>>();
    keyed.sort_by(|(left_key, left_path), (right_key, right_path)| {
        left_key
            .cmp(right_key)
            .then_with(|| left_path.cmp(right_path))
    });
    keyed.dedup_by(|(left_key, _), (right_key, _)| left_key == right_key);
    keyed
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ExpansionAllocation {
    pub(super) retained: usize,
    pub(super) per_wildcard_omitted: usize,
    pub(super) total_omitted: usize,
}

pub(super) fn expansion_allocation(
    discovered: usize,
    per_wildcard_limit: usize,
    total_remaining: usize,
) -> ExpansionAllocation {
    let per_wildcard_retained = discovered.min(per_wildcard_limit);
    let per_wildcard_omitted = discovered - per_wildcard_retained;
    let retained = per_wildcard_retained.min(total_remaining);
    let total_omitted = per_wildcard_retained - retained;
    ExpansionAllocation {
        retained,
        per_wildcard_omitted,
        total_omitted,
    }
}

fn matches_aggregation(instance: InstanceIdentity, aggregation_name: &str) -> bool {
    let name = if instance.index == 0 {
        instance.name
    } else {
        format!("{}#{}", instance.name, instance.index)
    };
    instance.parent.map_or(name == aggregation_name, |parent| {
        format!("{parent}/{name}") == aggregation_name
    })
}

#[cfg(test)]
#[path = "tests/expansion_support.rs"]
mod test_support;

pub(super) fn cleanup_aggregation_instances(
    paths: &mut Vec<String>,
    aggregation_name: Option<&str>,
    mut parse: impl FnMut(&str) -> Result<InstanceIdentity, Error>,
) -> Result<bool, Error> {
    let Some(aggregation_name) = aggregation_name else {
        return Ok(false);
    };
    let single = paths.len() == 1;
    let mut retained = Vec::with_capacity(paths.len());
    let mut aggregation_only = false;
    for path in paths.drain(..) {
        let aggregate = match parse(&path) {
            Ok(instance) => matches_aggregation(instance, aggregation_name),
            Err(error) if error.is_stop() => return Err(error),
            // Invalid paths remain available for ordinary admission diagnostics.
            Err(_) => false,
        };
        if !aggregate || single {
            retained.push(path);
            aggregation_only = aggregate;
        }
    }
    *paths = retained;
    Ok(single && aggregation_only)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct InstanceGroupKey {
    config_index: usize,
    normalized_name: String,
    normalized_parent: Option<String>,
}

impl InstanceGroupKey {
    fn new(config_index: usize, instance: &InstanceIdentity) -> Self {
        Self {
            config_index,
            normalized_name: instance.name.to_lowercase(),
            normalized_parent: instance.parent.as_ref().map(|value| value.to_lowercase()),
        }
    }
}

pub(super) fn instance_groups<'a>(
    counters: impl Iterator<Item = (&'a CounterKey, &'a InstanceIdentity)>,
) -> BTreeMap<InstanceGroupKey, BTreeSet<CounterKey>> {
    let mut groups = BTreeMap::new();
    for (key, instance) in counters {
        let _ = groups
            .entry(InstanceGroupKey::new(key.source_mapping_index, instance))
            .or_insert_with(BTreeSet::new)
            .insert(key.clone());
    }
    groups
}

pub(super) fn changed_instance_groups(
    existing: &BTreeMap<InstanceGroupKey, BTreeSet<CounterKey>>,
    wanted: &BTreeMap<InstanceGroupKey, BTreeSet<CounterKey>>,
) -> BTreeSet<InstanceGroupKey> {
    existing
        .keys()
        .chain(wanted.keys())
        .filter(|group| existing.get(*group) != wanted.get(*group))
        .cloned()
        .collect()
}

pub(super) fn refresh_changes_with_group_resets(
    live: &BTreeSet<CounterKey>,
    retrying: &BTreeSet<CounterKey>,
    wanted: &BTreeSet<CounterKey>,
    known_groups: &BTreeMap<InstanceGroupKey, BTreeSet<CounterKey>>,
    wanted_groups: &BTreeMap<InstanceGroupKey, BTreeSet<CounterKey>>,
) -> (Vec<CounterKey>, Vec<CounterKey>) {
    let changed_groups = changed_instance_groups(known_groups, wanted_groups);
    let reset_live = changed_groups
        .iter()
        .filter_map(|group| known_groups.get(group))
        .flatten()
        .filter(|key| live.contains(*key))
        .cloned()
        .collect::<BTreeSet<_>>();
    let reset_wanted = changed_groups
        .iter()
        .filter_map(|group| wanted_groups.get(group))
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut removed = live.difference(wanted).cloned().collect::<Vec<_>>();
    let mut added = wanted
        .difference(live)
        .filter(|key| !retrying.contains(*key))
        .cloned()
        .collect::<Vec<_>>();
    removed.extend(reset_live);
    removed.sort();
    removed.dedup();
    added.extend(reset_wanted);
    added.sort();
    added.dedup();
    (removed, added)
}
