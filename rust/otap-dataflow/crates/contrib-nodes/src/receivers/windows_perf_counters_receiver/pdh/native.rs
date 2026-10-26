// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Native PDH calls with checked statuses and buffers.
#![allow(unsafe_code)]

use super::{Pdh, Value};
use std::mem::size_of;
use std::ptr::{null, null_mut};
use windows_sys::Win32::System::Performance::{
    PDH_CSTATUS_BAD_COUNTERNAME, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER,
    PDH_HQUERY, PDH_INVALID_BUFFER, PDH_MORE_DATA, PdhAddEnglishCounterW, PdhCloseQuery,
    PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
};

// Not exposed by windows-sys 0.61.2.
const PDH_FMT_NOCAP100: u32 = 0x0000_8000;
const FORMAT: u32 = PDH_FMT_DOUBLE | PDH_FMT_NOCAP100;
// Instances can appear between the sizing and fill calls; give up after a few tries.
const MAX_ARRAY_READS: usize = 4;
const ITEM_SIZE: usize = size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>();

fn check(status: u32) -> Result<(), u32> {
    if status == 0 { Ok(()) } else { Err(status) }
}

/// An open live-data query. Dropping it closes the query and all its counters.
pub(in super::super) struct NativePdh {
    query: PDH_HQUERY,
}

impl NativePdh {
    pub(in super::super) fn open() -> Result<Self, u32> {
        let mut query = null_mut();
        // SAFETY: Null selects live local data; the output handle is writable.
        check(unsafe { PdhOpenQueryW(null(), 0, &mut query) })?;
        Ok(Self { query })
    }
}

impl Drop for NativePdh {
    fn drop(&mut self) {
        // SAFETY: This value exclusively owns the open query; no counter outlives it.
        let _ = unsafe { PdhCloseQuery(self.query) };
    }
}

impl Pdh for NativePdh {
    type Counter = PDH_HCOUNTER;

    fn add(&mut self, path: &str) -> Result<PDH_HCOUNTER, u32> {
        let path = wide_path(path)?;
        let mut counter = null_mut();
        // SAFETY: The query is open, the path is NUL-terminated, and the output is writable.
        check(unsafe { PdhAddEnglishCounterW(self.query, path.as_ptr(), 0, &mut counter) })?;
        Ok(counter)
    }

    fn collect(&mut self) -> Result<(), u32> {
        // SAFETY: The query is open and exclusively borrowed.
        check(unsafe { PdhCollectQueryData(self.query) })
    }

    fn array(
        &mut self,
        counter: &PDH_HCOUNTER,
        items: &mut Vec<(String, Value)>,
    ) -> Result<(), u32> {
        items.clear();
        for _ in 0..MAX_ARRAY_READS {
            // Size with a null buffer every time: after PDH_MORE_DATA from a fill, the
            // returned size is not reliable.
            let mut size = 0_u32;
            let mut count = 0_u32;
            // SAFETY: The counter belongs to this open query; a null buffer with zero size
            // only asks for the size, and both outputs are writable.
            let status = unsafe {
                PdhGetFormattedCounterArrayW(*counter, FORMAT, &mut size, &mut count, null_mut())
            };
            match status {
                PDH_MORE_DATA => {}
                0 => return decode_items(&[], count as usize, items),
                status => return Err(status),
            }
            // u64 storage is zero-initialized and aligned for the item structs.
            let mut buffer: Vec<u64> = vec![0; (size as usize).div_ceil(size_of::<u64>())];
            let mut size =
                u32::try_from(size_of_val(buffer.as_slice())).map_err(|_| PDH_INVALID_BUFFER)?;
            // SAFETY: As above, and the buffer is writable for `size` bytes.
            let status = unsafe {
                PdhGetFormattedCounterArrayW(
                    *counter,
                    FORMAT,
                    &mut size,
                    &mut count,
                    buffer.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>(),
                )
            };
            match status {
                // Instances appeared between the calls; size again.
                PDH_MORE_DATA => {}
                0 => return decode_items(&buffer, count as usize, items),
                status => return Err(status),
            }
        }
        Err(PDH_MORE_DATA)
    }
}

/// Encodes a NUL-terminated path; an interior NUL would silently truncate it, so it fails to add.
pub(super) fn wide_path(path: &str) -> Result<Vec<u16>, u32> {
    if path.contains('\0') {
        return Err(PDH_CSTATUS_BAD_COUNTERNAME);
    }
    Ok(path.encode_utf16().chain(Some(0)).collect())
}

/// Copies `count` items out of a filled buffer, rejecting any out-of-bounds data.
fn decode_items(buffer: &[u64], count: usize, items: &mut Vec<(String, Value)>) -> Result<(), u32> {
    let start = buffer.as_ptr() as usize;
    let end = start + size_of_val(buffer);
    if count
        .checked_mul(ITEM_SIZE)
        .is_none_or(|bytes| bytes > end - start)
    {
        return Err(PDH_INVALID_BUFFER);
    }
    // SAFETY: The buffer is initialized, aligned for items, and holds `count` of them.
    let raw = unsafe {
        std::slice::from_raw_parts(buffer.as_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>(), count)
    };
    for item in raw {
        let Some(name) = wide_string(item.szName, start, end) else {
            items.clear();
            return Err(PDH_INVALID_BUFFER);
        };
        // SAFETY: The union is plain data, and PDH_FMT_DOUBLE selects doubleValue.
        let value = Value {
            status: item.FmtValue.CStatus,
            value: unsafe { item.FmtValue.Anonymous.doubleValue },
        };
        items.push((name, value));
    }
    Ok(())
}

/// Reads a NUL-terminated UTF-16 string that must lie inside `[start, end)`; null is empty.
fn wide_string(pointer: *const u16, start: usize, end: usize) -> Option<String> {
    if pointer.is_null() {
        return Some(String::new());
    }
    let address = pointer as usize;
    if address < start || address >= end || !(address - start).is_multiple_of(size_of::<u16>()) {
        return None;
    }
    // SAFETY: The range lies inside the initialized buffer and is u16-aligned.
    let units = unsafe { std::slice::from_raw_parts(pointer, (end - address) / size_of::<u16>()) };
    let length = units.iter().position(|unit| *unit == 0)?;
    Some(String::from_utf16_lossy(&units[..length]))
}
