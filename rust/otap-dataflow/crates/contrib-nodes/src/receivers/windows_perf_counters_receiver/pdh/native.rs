// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Owns native handles, aligned metadata storage, and PDH status/value decoding.
#![allow(unsafe_code)]

use super::{
    CounterLanguage, Error, InstanceIdentity, Number, PdhApi, PdhOperation, RawSample, ValueFormat,
};
use std::collections::BTreeMap;
use std::mem::{size_of, size_of_val};
use std::ptr::{null, null_mut};
use std::slice;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use windows_sys::Win32::Globalization::GetThreadUILanguage;
use windows_sys::Win32::System::Performance::{
    PDH_CALC_NEGATIVE_DENOMINATOR, PDH_CALC_NEGATIVE_TIMEBASE, PDH_CALC_NEGATIVE_VALUE,
    PDH_COUNTER_INFO_W, PDH_COUNTER_PATH_ELEMENTS_W, PDH_CSTATUS_INVALID_DATA,
    PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_NO_INSTANCE, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE,
    PDH_FMT_DOUBLE, PDH_FMT_LARGE, PDH_HCOUNTER, PDH_HQUERY, PDH_INVALID_ARGUMENT,
    PDH_INVALID_DATA, PDH_MORE_DATA, PDH_NO_DATA, PDH_RAW_COUNTER, PDH_REFRESHCOUNTERS,
    PERF_DETAIL_WIZARD, PdhAddCounterW, PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData,
    PdhEnumObjectsW, PdhExpandWildCardPathW, PdhGetCounterInfoW, PdhGetFormattedCounterValue,
    PdhGetRawCounterValue, PdhOpenQueryW, PdhParseCounterPathW, PdhRemoveCounter,
};

// These flags are not exposed by windows-sys 0.61.2.
const PDH_FMT_NOSCALE: u32 = 0x0000_1000;
const PDH_FMT_NOCAP100: u32 = 0x0000_8000;
// Discovery receives the whole list before admission can select individual members.
const MAX_DISCOVERY_BUFFER_BYTES: usize = 64 * 1024 * 1024;
const MAX_DISCOVERY_BUFFER_UNITS: usize = MAX_DISCOVERY_BUFFER_BYTES / size_of::<u16>();

pub(super) fn initialize_worker(node: &str, shutdown: &AtomicBool) -> Result<(), Error> {
    check_running(shutdown)?;
    // SAFETY: This reads only the calling worker thread's UI language.
    let language = unsafe { GetThreadUILanguage() };
    otel_arrow_dfe_telemetry::otel_info!(
        "windows_perf_counters_receiver.worker.startup",
        node = node,
        thread_ui_language_id = language as u64
    );
    Ok(())
}

fn refresh_objects_with(
    mut enumerate_objects: impl FnMut(bool, &mut u32, Option<&mut [u16]>) -> Result<u32, Error>,
) -> Result<usize, Error> {
    let mut size = 0;
    let status = enumerate_objects(true, &mut size, None)?;
    if !matches!(status, 0 | PDH_MORE_DATA) || (status == PDH_MORE_DATA && size == 0) {
        Err(Error::WorkerObjectRefresh {
            stage: "refresh sizing",
            status,
        })
    } else if size == 0 {
        Ok(0)
    } else {
        let mut buffer = vec![0_u16; refresh_size(size, "refresh size conversion", status)?];
        size = refresh_size(buffer.len(), "enumeration buffer size", status)?;
        let mut status = enumerate_objects(false, &mut size, Some(&mut buffer))?;
        if status == PDH_MORE_DATA {
            let capacity: u32 = refresh_size(buffer.len(), "enumeration buffer size", status)?;
            size = size.max(capacity.checked_mul(2).ok_or(Error::WorkerObjectRefresh {
                stage: "enumeration buffer growth",
                status,
            })?);
            buffer.resize(
                refresh_size(size, "enumeration size conversion", status)?,
                0,
            );
            status = enumerate_objects(false, &mut size, Some(&mut buffer))?;
        }
        if status == 0 {
            let filled = refresh_size(size, "enumeration filled size", status)?;
            let objects = buffer.get(..filled).ok_or(Error::WorkerObjectRefresh {
                stage: "enumeration filled size",
                status,
            })?;
            Ok(NativePdh::parse_multi_sz(objects)?.len())
        } else {
            Err(Error::WorkerObjectRefresh {
                stage: "enumeration fill",
                status,
            })
        }
    }
}

fn refresh_size<T, U>(value: T, stage: &'static str, status: u32) -> Result<U, Error>
where
    U: TryFrom<T>,
{
    U::try_from(value).map_err(|_| Error::WorkerObjectRefresh { stage, status })
}

fn discovery_size(size: u32) -> Result<usize, Error> {
    let size = usize::try_from(size)
        .map_err(|_| Error::InvalidSample("wildcard path list size exceeds usize"))?;
    if size > MAX_DISCOVERY_BUFFER_UNITS {
        return Err(Error::DiscoveryBufferLimit);
    }
    Ok(size)
}

pub(super) fn check(operation: PdhOperation, path: &str, status: u32) -> Result<(), Error> {
    if status == 0 {
        Ok(())
    } else {
        Err(Error::Pdh {
            operation,
            path: path.to_owned(),
            status,
        })
    }
}

fn check_data(operation: PdhOperation, path: &str, status: u32) -> Result<(), Error> {
    if matches!(status, PDH_CSTATUS_VALID_DATA | PDH_CSTATUS_NEW_DATA) {
        Ok(())
    } else {
        Err(Error::CounterData {
            operation,
            path: path.to_owned(),
            status,
        })
    }
}

#[derive(Debug)]
struct NativeBuffer<T> {
    storage: Vec<T>,
}

impl<T: Clone + Default> NativeBuffer<T> {
    fn with_byte_capacity(size: u32, capacity_error: &'static str) -> Result<Self, Error> {
        let elements = (size as usize).div_ceil(size_of::<T>());
        let bytes = elements
            .checked_mul(size_of::<T>())
            .ok_or(Error::InvalidSample(capacity_error))?;
        let _ = u32::try_from(bytes).map_err(|_| Error::InvalidSample(capacity_error))?;
        let storage = vec![T::default(); elements];
        let buffer = Self { storage };
        let _ = buffer.byte_capacity_u32(capacity_error)?;
        Ok(buffer)
    }

    fn byte_capacity(&self) -> usize {
        size_of_val(self.storage.as_slice())
    }

    fn byte_capacity_u32(&self, error: &'static str) -> Result<u32, Error> {
        u32::try_from(self.byte_capacity()).map_err(|_| Error::InvalidSample(error))
    }

    fn as_ptr<U>(&self) -> *const U {
        self.storage.as_ptr().cast::<U>()
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.storage
    }
}

struct FixedHeaderBufferSpec {
    minimum_size: usize,
    size_operation: PdhOperation,
    fill_operation: PdhOperation,
    size_error: &'static str,
    capacity_error: &'static str,
    fill_error: &'static str,
}

const COUNTER_INFO_BUFFER: FixedHeaderBufferSpec = FixedHeaderBufferSpec {
    minimum_size: size_of::<PDH_COUNTER_INFO_W>(),
    size_operation: PdhOperation::GetCounterInfoSize,
    fill_operation: PdhOperation::GetCounterInfo,
    size_error: "PdhGetCounterInfoW returned an undersized buffer",
    capacity_error: "PdhGetCounterInfoW buffer capacity exceeds u32",
    fill_error: "PdhGetCounterInfoW returned an invalid filled size",
};

const COUNTER_PATH_BUFFER: FixedHeaderBufferSpec = FixedHeaderBufferSpec {
    minimum_size: size_of::<PDH_COUNTER_PATH_ELEMENTS_W>(),
    size_operation: PdhOperation::ParseCounterPathSize,
    fill_operation: PdhOperation::ParseCounterPath,
    size_error: "PdhParseCounterPathW returned an undersized buffer",
    capacity_error: "PdhParseCounterPathW buffer capacity exceeds u32",
    fill_error: "PdhParseCounterPathW returned an invalid filled size",
};

fn fixed_header_buffer_with<T: Clone + Default>(
    path: &str,
    spec: &FixedHeaderBufferSpec,
    mut read: impl FnMut(&mut u32, Option<&mut [T]>) -> Result<u32, Error>,
) -> Result<(NativeBuffer<T>, usize), Error> {
    let mut size = 0;
    let status = read(&mut size, None)?;
    if status != PDH_MORE_DATA {
        return Err(Error::Pdh {
            operation: spec.size_operation,
            path: path.to_owned(),
            status,
        });
    }
    if (size as usize) < spec.minimum_size {
        return Err(Error::InvalidSample(spec.size_error));
    }
    let mut buffer = NativeBuffer::with_byte_capacity(size, spec.capacity_error)?;
    check(
        spec.fill_operation,
        path,
        read(&mut size, Some(buffer.as_mut_slice()))?,
    )?;
    if (size as usize) < spec.minimum_size || (size as usize) > buffer.byte_capacity() {
        return Err(Error::InvalidSample(spec.fill_error));
    }
    Ok((buffer, size as usize))
}

fn is_no_observation_status(status: u32) -> bool {
    matches!(
        status,
        PDH_INVALID_DATA
            | PDH_NO_DATA
            | PDH_CSTATUS_NO_INSTANCE
            | PDH_CALC_NEGATIVE_DENOMINATOR
            | PDH_CALC_NEGATIVE_TIMEBASE
            | PDH_CALC_NEGATIVE_VALUE
    )
}

fn is_no_observation_data_status(status: u32) -> bool {
    matches!(
        status,
        PDH_CSTATUS_INVALID_DATA
            | PDH_CSTATUS_NO_INSTANCE
            | PDH_NO_DATA
            | PDH_CALC_NEGATIVE_DENOMINATOR
            | PDH_CALC_NEGATIVE_TIMEBASE
            | PDH_CALC_NEGATIVE_VALUE
    )
}

/// Opaque generational slot in an adapter's query.
pub(in super::super) struct CounterToken {
    index: usize,
    generation: u64,
}

struct CounterSlot {
    generation: u64,
    handle: Option<PDH_HCOUNTER>,
}

struct CounterHandles {
    slots: Vec<CounterSlot>,
    free_slots: Vec<usize>,
    next_generation: u64,
}

impl CounterHandles {
    const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_slots: Vec::new(),
            next_generation: 0,
        }
    }

    fn take_generation(&mut self, path: &str) -> Result<u64, Error> {
        let generation = self.next_generation;
        self.next_generation =
            self.next_generation
                .checked_add(1)
                .ok_or_else(|| Error::CounterSetup {
                    path: path.to_owned(),
                    message: "counter token generation exhausted".to_owned(),
                    permanent: true,
                    cleanup_failed: false,
                })?;
        Ok(generation)
    }

    fn insert(&mut self, generation: u64, handle: PDH_HCOUNTER) -> CounterToken {
        let index = if let Some(index) = self.free_slots.pop() {
            let slot = &mut self.slots[index];
            debug_assert!(slot.handle.is_none());
            slot.generation = generation;
            slot.handle = Some(handle);
            index
        } else {
            self.slots.push(CounterSlot {
                generation,
                handle: Some(handle),
            });
            self.slots.len() - 1
        };
        CounterToken { index, generation }
    }

    fn handle(&self, counter: &CounterToken, path: &str) -> Result<PDH_HCOUNTER, Error> {
        self.slots
            .get(counter.index)
            .filter(|slot| slot.generation == counter.generation)
            .and_then(|slot| slot.handle)
            .ok_or_else(|| Error::InvalidCounterToken {
                path: path.to_owned(),
            })
    }

    fn remove_with(
        &mut self,
        counter: &CounterToken,
        path: &str,
        remove: impl FnOnce(PDH_HCOUNTER) -> u32,
    ) -> Result<(), Error> {
        let handle = self.handle(counter, path)?;
        check(PdhOperation::RemoveCounter, path, remove(handle))?;
        let slot = &mut self.slots[counter.index];
        debug_assert_eq!(slot.generation, counter.generation);
        debug_assert!(slot.handle.is_some());
        slot.handle = None;
        self.free_slots.push(counter.index);
        Ok(())
    }
}

pub(in super::super) struct NativePdh {
    query: PDH_HQUERY,
    counters: CounterHandles,
    node: String,
    shutdown: Arc<AtomicBool>,
    lookup_queries: BTreeMap<String, LookupQuery>,
}

struct LookupQuery {
    // A template awaiting removal must never join serving-query collection.
    api: NativePdh,
    pending: Option<CounterToken>,
}

pub(super) fn resolve_template<A: PdhApi>(
    api: &mut A,
    pending: &mut Option<A::Counter>,
    path: &str,
    full_path: impl FnOnce(&mut A, &A::Counter, &str) -> Result<String, Error>,
) -> Result<String, Error> {
    if let Some(counter) = pending.as_ref() {
        api.remove(counter, path)
            .map_err(|error| Error::WildcardCleanup {
                path: path.to_owned(),
                message: error.to_string(),
                lookup_failed: false,
            })?;
        *pending = None;
    }
    api.check_running()?;
    let counter = api.add(path, CounterLanguage::English)?;
    let resolved = full_path(api, &counter, path);
    let cleanup = api.remove(&counter, path);
    if cleanup.is_err() {
        *pending = Some(counter);
    }
    if resolved.as_ref().is_err_and(Error::is_stop) {
        return resolved;
    }
    match cleanup {
        Ok(()) => resolved,
        Err(cleanup) => Err(Error::WildcardCleanup {
            path: path.to_owned(),
            message: resolved.as_ref().err().map_or_else(
                || cleanup.to_string(),
                |lookup| format!("{lookup}; {cleanup}"),
            ),
            lookup_failed: resolved.is_err(),
        }),
    }
}

impl NativePdh {
    pub(super) fn open_controlled(node: String, shutdown: Arc<AtomicBool>) -> Result<Self, Error> {
        check_running(&shutdown)?;
        let mut query = null_mut();
        // SAFETY: Null selects live local data; the output handle is writable.
        check(PdhOperation::OpenQuery, "<query>", unsafe {
            PdhOpenQueryW(null(), 0, &mut query)
        })?;
        #[cfg(test)]
        let _ = super::QUERY_OPENS.fetch_add(1, Ordering::Relaxed);
        Ok(Self {
            query,
            counters: CounterHandles::new(),
            node,
            shutdown,
            lookup_queries: BTreeMap::new(),
        })
    }

    fn wide_string(
        pointer: *const u16,
        buffer: &NativeBuffer<usize>,
        filled_size: usize,
    ) -> Result<Option<String>, Error> {
        if pointer.is_null() {
            return Ok(None);
        }
        if filled_size > buffer.byte_capacity() {
            return Err(Error::InvalidSample(
                "PDH metadata returned an invalid filled size",
            ));
        }
        let start = buffer.as_ptr::<usize>() as usize;
        let bytes = filled_size;
        let end = start
            .checked_add(bytes)
            .ok_or(Error::InvalidSample("PDH metadata buffer overflowed"))?;
        let address = pointer as usize;
        if address < start || address >= end || !(address - start).is_multiple_of(size_of::<u16>())
        {
            return Err(Error::InvalidSample(
                "PDH metadata returned an out-of-buffer string",
            ));
        }
        let available = (end - address) / size_of::<u16>();
        // SAFETY: Bounds and alignment were validated against the live buffer.
        let units = unsafe { slice::from_raw_parts(pointer, available) };
        let Some(length) = units.iter().position(|unit| *unit == 0) else {
            return Err(Error::InvalidSample(
                "PDH metadata returned an unterminated string",
            ));
        };
        Ok(Some(String::from_utf16_lossy(&units[..length])))
    }

    fn full_path(&mut self, counter: &CounterToken, path: &str) -> Result<String, Error> {
        let handle = self.counter_handle(counter, path)?;
        let (buffer, filled) =
            fixed_header_buffer_with::<usize>(path, &COUNTER_INFO_BUFFER, |size, buffer| {
                self.check_running()?;
                // SAFETY: The counter belongs to this query. The helper
                // supplies aligned writable storage or null for sizing.
                Ok(unsafe {
                    PdhGetCounterInfoW(
                        handle,
                        false,
                        size,
                        buffer.map_or(null_mut(), |buffer| buffer.as_mut_ptr().cast()),
                    )
                })
            })?;
        // SAFETY: A successful fill initialized the validated fixed header.
        let info = unsafe { &*buffer.as_ptr::<PDH_COUNTER_INFO_W>() };
        Self::wide_string(info.szFullPath, &buffer, filled)?
            .filter(|path| !path.is_empty())
            .ok_or(Error::InvalidSample("counter info returned no full path"))
    }

    fn parse_multi_sz(buffer: &[u16]) -> Result<Vec<String>, Error> {
        Self::parse_multi_sz_with(buffer, |units| Ok(String::from_utf16_lossy(units)))
    }

    fn parse_multi_sz_with(
        buffer: &[u16],
        mut decode: impl FnMut(&[u16]) -> Result<String, Error>,
    ) -> Result<Vec<String>, Error> {
        let mut values = Vec::new();
        let mut start = 0;
        while start < buffer.len() {
            let Some(relative_end) = buffer[start..].iter().position(|unit| *unit == 0) else {
                return Err(Error::InvalidSample(
                    "PDH multi-string returned an unterminated entry",
                ));
            };
            if relative_end == 0 {
                return Ok(values);
            }
            let end = start + relative_end;
            values.push(decode(&buffer[start..end])?);
            start = end + 1;
        }
        Err(Error::InvalidSample(
            "PDH multi-string returned no list terminator",
        ))
    }

    fn parse_discovery_paths(
        buffer: &[u16],
        budget: &mut super::expansion::DiscoveryBudget,
        mut check: impl FnMut() -> Result<(), Error>,
    ) -> Result<Vec<String>, Error> {
        Self::parse_multi_sz_with(buffer, |units| {
            check()?;
            let capacity = budget.reserve_path(units.len())?;
            let mut path = String::with_capacity(capacity);
            path.extend(
                char::decode_utf16(units.iter().copied())
                    .map(|value| value.unwrap_or(char::REPLACEMENT_CHARACTER)),
            );
            Ok(path)
        })
    }

    #[cfg(test)]
    fn expanded_path_buffer_with(
        path: &str,
        expand: impl FnMut(&mut u32, Option<&mut [u16]>) -> Result<u32, Error>,
    ) -> Result<Vec<u16>, Error> {
        Self::expanded_path_buffer_budgeted_with(
            path,
            &mut super::expansion::DiscoveryBudget::new(),
            expand,
        )
    }

    fn expanded_path_buffer_budgeted_with(
        path: &str,
        budget: &mut super::expansion::DiscoveryBudget,
        mut expand: impl FnMut(&mut u32, Option<&mut [u16]>) -> Result<u32, Error>,
    ) -> Result<Vec<u16>, Error> {
        let mut size = 0;
        let size_status = expand(&mut size, None)?;
        if size_status == PDH_CSTATUS_NO_INSTANCE || (size_status == 0 && size == 0) {
            return Ok(Vec::new());
        }
        if size_status != PDH_MORE_DATA {
            return Err(Error::Pdh {
                operation: PdhOperation::ExpandWildcardPathSize,
                path: path.to_owned(),
                status: size_status,
            });
        }
        let measured = discovery_size(size)?;
        let headroom = measured / 4;
        let mut capacity = (measured + headroom.max(64)).min(MAX_DISCOVERY_BUFFER_UNITS);
        let mut last_status = PDH_MORE_DATA;
        for _ in 0..8 {
            size = u32::try_from(capacity)
                .map_err(|_| Error::InvalidSample("wildcard path list size exceeds u32"))?;
            budget.reserve(capacity * size_of::<u16>())?;
            let mut buffer = vec![0_u16; capacity];
            let status = expand(&mut size, Some(&mut buffer))?;
            if matches!(status, 0 | PDH_CSTATUS_NO_INSTANCE) {
                if status == PDH_CSTATUS_NO_INSTANCE || size == 0 {
                    return Ok(Vec::new());
                }
                let used = usize::try_from(size)
                    .map_err(|_| Error::InvalidSample("wildcard path list size exceeds usize"))?;
                if used > buffer.len() {
                    return Err(Error::InvalidSample(
                        "PdhExpandWildCardPathW reported an oversized path list",
                    ));
                }
                buffer.truncate(used);
                return Ok(buffer);
            }
            last_status = status;
            if !matches!(status, PDH_MORE_DATA | PDH_INVALID_ARGUMENT) {
                break;
            }
            let reported = discovery_size(size)?;
            let geometric = capacity + capacity / 2 + 64;
            capacity = reported.max(geometric).min(MAX_DISCOVERY_BUFFER_UNITS);
        }
        Err(Error::Pdh {
            operation: PdhOperation::ExpandWildcardPath,
            path: path.to_owned(),
            status: last_status,
        })
    }

    fn expand_wildcard(
        &mut self,
        path: &str,
        budget: &mut super::expansion::DiscoveryBudget,
    ) -> Result<Vec<String>, Error> {
        let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let buffer = Self::expanded_path_buffer_budgeted_with(path, budget, |size, buffer| {
            self.check_running()?;
            // SAFETY: The path is NUL-terminated. The optional buffer, when present,
            // is writable for the size supplied to PDH.
            Ok(unsafe {
                PdhExpandWildCardPathW(
                    null(),
                    wide.as_ptr(),
                    buffer.map_or(null_mut(), |buffer| buffer.as_mut_ptr()),
                    size,
                    PDH_REFRESHCOUNTERS,
                )
            })
        });
        self.check_running()?;
        let buffer = buffer?;
        if buffer.is_empty() {
            Ok(Vec::new())
        } else {
            Self::parse_discovery_paths(&buffer, budget, || self.check_running())
        }
    }

    fn parse_instance_controlled(
        path: &str,
        shutdown: &AtomicBool,
    ) -> Result<InstanceIdentity, Error> {
        Self::parse_path_controlled(path, shutdown, Self::instance_from_buffer)
    }

    fn parse_path_controlled<T>(
        path: &str,
        shutdown: &AtomicBool,
        decode: impl FnOnce(&NativeBuffer<usize>, usize) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let wide_path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let result =
            fixed_header_buffer_with::<usize>(path, &COUNTER_PATH_BUFFER, |size, buffer| {
                check_running(shutdown)?;
                // SAFETY: The path is NUL-terminated. Sizing uses null and zero;
                // filling uses an aligned writable slice.
                Ok(unsafe {
                    PdhParseCounterPathW(
                        wide_path.as_ptr(),
                        buffer.map_or(null_mut(), |buffer| buffer.as_mut_ptr().cast()),
                        size,
                        0,
                    )
                })
            });
        check_running(shutdown)?;
        let (buffer, filled_size) = result?;
        decode(&buffer, filled_size)
    }

    #[cfg(test)]
    fn parse_instance_identity_with(
        path: &str,
        parse: impl FnMut(&mut u32, Option<&mut [usize]>) -> Result<u32, Error>,
    ) -> Result<InstanceIdentity, Error> {
        let (buffer, filled_size) = fixed_header_buffer_with(path, &COUNTER_PATH_BUFFER, parse)?;
        Self::instance_from_buffer(&buffer, filled_size)
    }

    fn instance_from_buffer(
        buffer: &NativeBuffer<usize>,
        filled_size: usize,
    ) -> Result<InstanceIdentity, Error> {
        // SAFETY: Successful parsing initialized the fixed pointer fields.
        let elements = unsafe { &*buffer.as_ptr::<PDH_COUNTER_PATH_ELEMENTS_W>() };
        let name = Self::wide_string(elements.szInstanceName, buffer, filled_size)?.ok_or(
            Error::InvalidSample("expanded path has no instance identity"),
        )?;
        let parent = Self::wide_string(elements.szParentInstance, buffer, filled_size)?;
        Ok(InstanceIdentity {
            name,
            parent: parent.filter(|value| !value.is_empty()),
            index: elements.dwInstanceIndex,
        })
    }

    fn comparison_path_controlled(path: &str, shutdown: &AtomicBool) -> Result<String, Error> {
        Self::parse_path_controlled(path, shutdown, |buffer, filled| {
            // SAFETY: Successful parsing initialized the validated fixed header.
            let elements = unsafe { &*buffer.as_ptr::<PDH_COUNTER_PATH_ELEMENTS_W>() };
            let object = Self::wide_string(elements.szObjectName, buffer, filled)?
                .ok_or(Error::InvalidSample("native path has no object"))?;
            let counter = Self::wide_string(elements.szCounterName, buffer, filled)?
                .ok_or(Error::InvalidSample("native path has no counter"))?;
            if elements.szInstanceName.is_null() {
                return Ok(format!(r"\{object}\{counter}"));
            }
            let instance = Self::instance_from_buffer(buffer, filled)?;
            let name = if instance.index == 0 {
                instance.name
            } else {
                format!("{}#{}", instance.name, instance.index)
            };
            let name = super::super::config::instance_comparison_key(&name);
            let component = instance
                .parent
                .map_or_else(|| name.clone(), |parent| format!("{parent}/{name}"));
            Ok(format!(r"\{object}({component})\{counter}"))
        })
    }

    fn counter_handle(&self, counter: &CounterToken, path: &str) -> Result<PDH_HCOUNTER, Error> {
        self.counters.handle(counter, path)
    }

    pub(super) fn collect_status(status: u32) -> Result<bool, Error> {
        if is_no_observation_status(status) {
            return Ok(false);
        }
        check(PdhOperation::CollectQueryData, "<query>", status)?;
        Ok(true)
    }

    pub(super) fn raw_with(
        path: &str,
        read: impl FnOnce(&mut PDH_RAW_COUNTER) -> u32,
    ) -> Result<Option<RawSample>, Error> {
        let mut raw = PDH_RAW_COUNTER::default();
        let status = read(&mut raw);
        if is_no_observation_status(status) {
            return Ok(None);
        }
        check(PdhOperation::GetRawCounterValue, path, status)?;
        if is_no_observation_data_status(raw.CStatus) {
            return Ok(None);
        }
        check_data(PdhOperation::RawCounterDataStatus, path, raw.CStatus)?;
        Ok(Some(RawSample {
            second_value: raw.SecondValue,
            multi_count: raw.MultiCount,
        }))
    }

    pub(super) fn counter_type_with(
        path: &str,
        info: impl FnMut(&mut u32, Option<&mut [PDH_COUNTER_INFO_W]>) -> Result<u32, Error>,
    ) -> Result<u32, Error> {
        // Typed storage guarantees the header's alignment and initializes the
        // trailing buffer; PDH fills both the header and variable-sized data.
        let (mut buffer, _) = fixed_header_buffer_with(path, &COUNTER_INFO_BUFFER, info)?;
        // Only a successful fill reaches classification; zero is a valid type.
        Ok(buffer.as_mut_slice()[0].dwType)
    }

    pub(super) fn formatted_with(
        path: &str,
        format: ValueFormat,
        formatted: impl FnOnce(u32, &mut PDH_FMT_COUNTERVALUE) -> u32,
    ) -> Result<Option<Number>, Error> {
        let flags = match format {
            ValueFormat::Integer => PDH_FMT_LARGE | PDH_FMT_NOSCALE,
            ValueFormat::Double => PDH_FMT_DOUBLE | PDH_FMT_NOSCALE | PDH_FMT_NOCAP100,
        };
        let mut value = PDH_FMT_COUNTERVALUE::default();
        let status = formatted(flags, &mut value);
        if status != 0 && status != PDH_INVALID_DATA {
            if is_no_observation_status(status) {
                return Ok(None);
            }
            check(PdhOperation::GetFormattedCounterValue, path, status)?;
        }
        // PDH_INVALID_DATA can carry a more specific failure in CStatus.
        if is_no_observation_data_status(value.CStatus) {
            return Ok(None);
        }
        check_data(
            PdhOperation::FormattedCounterDataStatus,
            path,
            value.CStatus,
        )?;
        if status == PDH_INVALID_DATA {
            return Ok(None);
        }
        let number = match format {
            // SAFETY: A successful PDH_FMT_LARGE call initialized largeValue.
            ValueFormat::Integer => Number::Integer(unsafe { value.Anonymous.largeValue }),
            // SAFETY: A successful PDH_FMT_DOUBLE call initialized doubleValue.
            ValueFormat::Double => Number::Double(unsafe { value.Anonymous.doubleValue }),
        };
        Ok(Some(number))
    }
}

impl PdhApi for NativePdh {
    type Counter = CounterToken;

    fn check_running(&self) -> Result<(), Error> {
        check_running(&self.shutdown)
    }

    fn add(&mut self, path: &str, language: CounterLanguage) -> Result<CounterToken, Error> {
        self.check_running()?;
        let generation = self.counters.take_generation(path)?;
        let wide_path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let mut handle = null_mut();
        // SAFETY: The adapter owns the live query, path is NUL-terminated,
        // and the output handle is writable.
        let (status, operation) = unsafe {
            match language {
                CounterLanguage::English => (
                    PdhAddEnglishCounterW(self.query, wide_path.as_ptr(), 0, &mut handle),
                    PdhOperation::AddEnglishCounter,
                ),
                CounterLanguage::Localized => (
                    PdhAddCounterW(self.query, wide_path.as_ptr(), 0, &mut handle),
                    PdhOperation::AddCounter,
                ),
            }
        };
        check(operation, path, status)?;
        Ok(self.counters.insert(generation, handle))
    }

    fn remove(&mut self, counter: &CounterToken, path: &str) -> Result<(), Error> {
        self.counters.remove_with(counter, path, |handle| {
            // SAFETY: Lookup returns an owned, live counter; only a successful
            // native removal releases its generational slot.
            unsafe { PdhRemoveCounter(handle) }
        })
    }

    fn counter_type(&mut self, counter: &CounterToken, path: &str) -> Result<u32, Error> {
        let handle = self.counter_handle(counter, path)?;
        Self::counter_type_with(path, |size, buffer| {
            self.check_running()?;
            let info = if let Some(buffer) = buffer {
                *size = u32::try_from(size_of_val(buffer)).map_err(|_| {
                    Error::InvalidSample("PdhGetCounterInfoW buffer capacity exceeds u32")
                })?;
                buffer.as_mut_ptr()
            } else {
                *size = 0;
                null_mut()
            };
            // SAFETY: Lookup returned an owned, live counter. For filling, size
            // and the aligned pointer come from the same writable slice above;
            // sizing uses null and zero. The size output is writable.
            Ok(unsafe { PdhGetCounterInfoW(handle, false, size, info) })
        })
    }

    fn full_path(&mut self, counter: &CounterToken, path: &str) -> Result<String, Error> {
        Self::full_path(self, counter, path)
    }

    fn comparison_path(&mut self, path: &str) -> Result<String, Error> {
        Self::comparison_path_controlled(path, &self.shutdown)
    }

    fn collect(&mut self) -> Result<bool, Error> {
        self.check_running()?;
        // SAFETY: The adapter owns the live query and is borrowed exclusively.
        Self::collect_status(unsafe { PdhCollectQueryData(self.query) })
    }

    fn raw(&mut self, counter: &CounterToken, path: &str) -> Result<Option<RawSample>, Error> {
        self.check_running()?;
        let handle = self.counter_handle(counter, path)?;
        Self::raw_with(path, |raw| {
            // SAFETY: Lookup returned an owned, live counter and the adapter
            // supplies initialized, writable output.
            unsafe { PdhGetRawCounterValue(handle, null_mut(), raw) }
        })
    }

    fn formatted(
        &mut self,
        counter: &CounterToken,
        path: &str,
        format: ValueFormat,
    ) -> Result<Option<Number>, Error> {
        self.check_running()?;
        let handle = self.counter_handle(counter, path)?;
        Self::formatted_with(path, format, |flags, value| {
            // SAFETY: Lookup returned an owned, live counter and the adapter
            // supplies initialized, writable storage for the format it decodes.
            unsafe { PdhGetFormattedCounterValue(handle, flags, null_mut(), value) }
        })
    }

    fn expand(
        &mut self,
        path: &str,
        budget: &mut super::expansion::DiscoveryBudget,
    ) -> Result<Vec<String>, Error> {
        self.check_running()?;
        if !self.lookup_queries.contains_key(path) {
            let api = Self::open_controlled(self.node.clone(), self.shutdown.clone())?;
            let _ = self
                .lookup_queries
                .insert(path.to_owned(), LookupQuery { api, pending: None });
        }
        let lookup = self
            .lookup_queries
            .get_mut(path)
            .expect("lookup query was inserted");
        let localized =
            resolve_template(&mut lookup.api, &mut lookup.pending, path, Self::full_path)?;
        self.expand_wildcard(&localized, budget)
    }

    fn refresh_objects(&mut self) -> Result<usize, Error> {
        let started = Instant::now();
        let result = refresh_objects_with(|refresh, size, buffer| {
            self.check_running()?;
            // SAFETY: Null selects the live local source and machine. The output
            // buffer is writable for the supplied UTF-16 element count.
            Ok(unsafe {
                PdhEnumObjectsW(
                    null(),
                    null(),
                    buffer.map_or(null_mut(), |buffer| buffer.as_mut_ptr()),
                    size,
                    PERF_DETAIL_WIZARD,
                    refresh.into(),
                )
            })
        });
        otel_arrow_dfe_telemetry::otel_info!(
            "windows_perf_counters_receiver.objects.refresh",
            node = self.node.as_str(),
            duration_ms = started.elapsed().as_secs_f64() * 1000.0,
            object_count = ?result.as_ref().ok(),
            error = ?result.as_ref().err()
        );
        result
    }

    fn parse_instance(&mut self, path: &str) -> Result<InstanceIdentity, Error> {
        Self::parse_instance_controlled(path, &self.shutdown)
    }
}

fn check_running(shutdown: &AtomicBool) -> Result<(), Error> {
    if shutdown.load(Ordering::Acquire) {
        Err(Error::WorkerStopRequested)
    } else {
        Ok(())
    }
}

impl Drop for NativePdh {
    fn drop(&mut self) {
        // SAFETY: Construction opened this exclusively owned query. Drop is
        // its only close path; no counter can be read through this adapter afterward.
        let status = unsafe { PdhCloseQuery(self.query) };
        #[cfg(test)]
        let _ = super::QUERY_CLOSES.fetch_add(1, Ordering::Relaxed);
        if status != 0 {
            otel_arrow_dfe_telemetry::otel_warn!(
                "windows_perf_counters_receiver.query.close.fail",
                node = self.node.as_str(),
                status = status as u64
            );
        }
    }
}

#[cfg(test)]
#[path = "tests/native.rs"]
mod tests;
