// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Live Rust heap samples collected above the snmalloc global allocator.

use otel_arrow_dfe_controller::HeapProfileError;
use rustfs_mimalloc_profiling::{MimallocConfig, Profiler};
use std::sync::Mutex;

static PROFILER: Mutex<Option<Profiler>> = Mutex::new(None);
const MAX_PROFILE_SIZE: usize = 16 * 1024 * 1024;

pub(crate) fn initialize() -> Result<(), HeapProfileError> {
    if std::env::var_os("SNMALLOC_PROFILE").is_some_and(|value| value == "1") {
        start(MimallocConfig {
            live_heap_tracking: true,
            ..MimallocConfig::default()
        })?;
    }
    Ok(())
}

fn start(config: MimallocConfig) -> Result<(), HeapProfileError> {
    let mut profiler = PROFILER.lock().map_err(|_| profiler_lock_error())?;
    *profiler = Some(Profiler::start(config).map_err(|error| {
        HeapProfileError::Failed(format!("Could not start snmalloc heap profiling: {error}"))
    })?);
    Ok(())
}

fn profiler_lock_error() -> HeapProfileError {
    HeapProfileError::Failed("Could not lock the snmalloc heap profiler".into())
}

pub(crate) fn snapshot() -> Result<Vec<u8>, HeapProfileError> {
    let profile = PROFILER
        .lock()
        .map_err(|_| profiler_lock_error())?
        .as_mut()
        .ok_or(HeapProfileError::Unavailable(
            "Heap profiling is inactive. Start the engine with SNMALLOC_PROFILE=1.",
        ))?
        .report()
        .map_err(|error| {
            HeapProfileError::Failed(format!("Could not dump snmalloc heap pprof: {error}"))
        })?;

    if profile.len() > MAX_PROFILE_SIZE {
        return Err(profile_too_large());
    }

    #[cfg(target_os = "linux")]
    let profile = linux::with_executable_mappings(profile)?;

    Ok(profile)
}

fn profile_too_large() -> HeapProfileError {
    HeapProfileError::Failed(
        "Could not encode the heap profile within the 16 MiB response limit".into(),
    )
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{HeapProfileError, MAX_PROFILE_SIZE, profile_too_large};
    use prost::Message;
    use rustfs_mimalloc_profiling::pprof::{Mapping, Profile};
    use std::collections::{BTreeMap, HashMap};

    struct ExecutableMapping<'a> {
        start: u64,
        end: u64,
        offset: u64,
        path: &'a str,
    }

    fn parse_mapping(line: &str) -> Option<ExecutableMapping<'_>> {
        // Linux pads the pathname column and paths themselves can contain spaces.
        let mut columns = line.splitn(6, ' ');
        let (start, end) = columns.next()?.split_once('-')?;
        let mode = columns.next()?;
        let offset = columns.next()?;
        let _device = columns.next()?;
        let _inode = columns.next()?;
        let path = columns.next()?.trim_start();
        if mode.as_bytes().get(2) != Some(&b'x') || !path.starts_with('/') {
            return None;
        }
        Some(ExecutableMapping {
            start: u64::from_str_radix(start, 16).ok()?,
            end: u64::from_str_radix(end, 16).ok()?,
            offset: u64::from_str_radix(offset, 16).ok()?,
            path,
        })
    }

    pub(super) fn with_executable_mappings(bytes: Vec<u8>) -> Result<Vec<u8>, HeapProfileError> {
        // The wrapper can omit mappings if it cannot read the in-memory ELF
        // build ID (observed with mold). Linux's mappings also record the
        // page-aligned file offsets that pprof needs for PIE binaries.
        let mut profile = Profile::decode(bytes.as_slice()).map_err(|error| {
            HeapProfileError::Failed(format!("Could not parse snmalloc heap pprof: {error}"))
        })?;
        if profile.location.is_empty() {
            return Ok(bytes);
        }
        let maps = std::fs::read("/proc/self/maps").map_err(|error| {
            HeapProfileError::Failed(format!("Could not read the executable mappings: {error}"))
        })?;
        let maps = String::from_utf8_lossy(&maps);
        let executable = std::env::current_exe().map_err(|error| {
            HeapProfileError::Failed(format!("Could not identify the engine executable: {error}"))
        })?;
        let executable = executable.to_string_lossy();

        // Preserve build IDs from the original encoder where it found them.
        let build_ids: HashMap<_, _> = profile
            .mapping
            .iter()
            .filter(|mapping| mapping.build_id != 0)
            .filter_map(|mapping| {
                let file_name = profile.string_table.get(mapping.filename as usize)?;
                Some((file_name.clone(), mapping.build_id))
            })
            .collect();
        profile.mapping.clear();
        let mut by_start = BTreeMap::new();
        let mut addresses: Vec<_> = profile
            .location
            .iter()
            .map(|location| location.address)
            .collect();
        addresses.sort_unstable();
        let mut mappings: Vec<_> = maps
            .lines()
            .filter_map(parse_mapping)
            .filter(|mapping| {
                let first = addresses.partition_point(|address| *address < mapping.start);
                mapping.path == executable
                    || addresses
                        .get(first)
                        .is_some_and(|address| *address < mapping.end)
            })
            .collect();
        // pprof treats the first mapping as the main executable.
        mappings.sort_by_key(|mapping| mapping.path != executable);
        for mapping in mappings {
            let id = profile.mapping.len() as u64 + 1;
            let filename = profile.string_table.len() as i64;
            profile.string_table.push(mapping.path.to_owned());
            profile.mapping.push(Mapping {
                id,
                memory_start: mapping.start,
                memory_limit: mapping.end,
                file_offset: mapping.offset,
                filename,
                build_id: build_ids.get(mapping.path).copied().unwrap_or_default(),
                ..Default::default()
            });
            _ = by_start.insert(mapping.start, (mapping.end, id));
        }
        for location in &mut profile.location {
            location.mapping_id = by_start
                .range(..=location.address)
                .next_back()
                .filter(|(_, (end, _))| location.address < *end)
                .map_or(0, |(_, (_, id))| *id);
        }
        if profile.encoded_len() > MAX_PROFILE_SIZE {
            return Err(profile_too_large());
        }
        Ok(profile.encode_to_vec())
    }
}

#[cfg(all(
    test,
    target_os = "linux",
    not(any(feature = "mimalloc", feature = "jemalloc", feature = "dhat-heap"))
))]
mod tests {
    use super::*;
    use prost::Message;
    use rustfs_mimalloc_profiling::pprof::Profile;
    use std::process::Command;

    const CHILD: &str = "DF_ENGINE_TEST_SNMALLOC_HEAP_CHILD";

    #[inline(never)]
    fn retained_allocation() -> Vec<u8> {
        let allocation = vec![42_u8; 4 * 1024 * 1024];
        std::hint::black_box(&allocation);
        allocation
    }

    fn retained_frame_addresses(profile: &Profile) -> Vec<u64> {
        let sample_index = profile
            .sample_type
            .iter()
            .position(|kind| profile.string_table[kind.r#type as usize] == "inuse_space")
            .unwrap();
        profile
            .sample
            .iter()
            .filter(|sample| sample.value[sample_index] > 0)
            .flat_map(|sample| &sample.location_id)
            .filter_map(|id| profile.location.iter().find(|location| location.id == *id))
            .filter(|location| {
                location.line.iter().any(|line| {
                    profile.function.iter().any(|function| {
                        function.id == line.function_id
                            && profile.string_table[function.name as usize]
                                .contains("::retained_allocation")
                    })
                })
            })
            .map(|location| location.address)
            .collect()
    }

    /// Scenario: An isolated snmalloc process profiles a live buffer and frees it on another thread.
    /// Guarantees: The live protobuf identifies the allocation site with a correct executable
    /// mapping and removes it from live samples after the cross-thread free.
    #[test]
    fn snmalloc_heap_snapshot_maps_and_tracks_live_allocations() {
        if std::env::var_os(CHILD).is_none() {
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "snmalloc_profile::tests::snmalloc_heap_snapshot_maps_and_tracks_live_allocations",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .env_remove("SNMALLOC_PROFILE")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        initialize().unwrap();
        assert!(matches!(snapshot(), Err(HeapProfileError::Unavailable(_))));
        start(MimallocConfig {
            sample_interval_bytes: 1,
            live_heap_tracking: true,
            ..MimallocConfig::default()
        })
        .unwrap();
        let allocation = retained_allocation();
        let live = Profile::decode(snapshot().unwrap().as_slice()).unwrap();
        let addresses = retained_frame_addresses(&live);
        assert!(!addresses.is_empty(), "live sample must name the caller");
        let maps = std::fs::read_to_string("/proc/self/maps").unwrap();
        for address in addresses {
            let location = live
                .location
                .iter()
                .find(|location| location.address == address)
                .unwrap();
            let mapping = live
                .mapping
                .iter()
                .find(|mapping| mapping.id == location.mapping_id)
                .unwrap();
            let actual = maps
                .lines()
                .find_map(|line| {
                    let mut fields = line.split_whitespace();
                    let (start, end) = fields.next()?.split_once('-')?;
                    let start = u64::from_str_radix(start, 16).ok()?;
                    let end = u64::from_str_radix(end, 16).ok()?;
                    let _mode = fields.next()?;
                    let offset = u64::from_str_radix(fields.next()?, 16).ok()?;
                    (start..end)
                        .contains(&address)
                        .then_some((start, end, offset))
                })
                .unwrap();
            assert_eq!(
                (
                    mapping.memory_start,
                    mapping.memory_limit,
                    mapping.file_offset
                ),
                actual
            );
        }
        std::thread::spawn(move || drop(allocation)).join().unwrap();
        let freed = Profile::decode(snapshot().unwrap().as_slice()).unwrap();
        assert!(retained_frame_addresses(&freed).is_empty());
    }
}
