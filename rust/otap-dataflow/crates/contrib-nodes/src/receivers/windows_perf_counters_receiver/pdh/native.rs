// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Owns native handles, aligned metadata storage, and PDH status/value decoding.
#![allow(unsafe_code)]

use super::{Error, Number, PdhApi, ValueFormat};
use std::mem::{size_of, size_of_val};
use std::ptr::{null, null_mut};
use std::time::{SystemTime, UNIX_EPOCH};
use windows_sys::Win32::System::Performance::{
    PDH_CALC_NEGATIVE_DENOMINATOR, PDH_CALC_NEGATIVE_TIMEBASE, PDH_CALC_NEGATIVE_VALUE,
    PDH_COUNTER_INFO_W, PDH_CSTATUS_INVALID_DATA, PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA,
    PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE, PDH_FMT_LARGE, PDH_HCOUNTER, PDH_HQUERY,
    PDH_INVALID_DATA, PDH_MORE_DATA, PDH_NO_DATA, PDH_RAW_COUNTER, PdhAddEnglishCounterW,
    PdhCloseQuery, PdhCollectQueryData, PdhGetCounterInfoW, PdhGetFormattedCounterValue,
    PdhGetRawCounterValue, PdhOpenQueryW, PdhRemoveCounter,
};

// These flags are not exposed by windows-sys 0.61.2.
const PDH_FMT_NOSCALE: u32 = 0x0000_1000;
const PDH_FMT_NOCAP100: u32 = 0x0000_8000;

pub(super) fn check(operation: &'static str, path: &str, status: u32) -> Result<(), Error> {
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

fn check_data(operation: &'static str, path: &str, status: u32) -> Result<(), Error> {
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

fn is_no_observation_status(status: u32) -> bool {
    matches!(
        status,
        PDH_INVALID_DATA
            | PDH_NO_DATA
            | PDH_CALC_NEGATIVE_DENOMINATOR
            | PDH_CALC_NEGATIVE_TIMEBASE
            | PDH_CALC_NEGATIVE_VALUE
    )
}

fn is_no_observation_data_status(status: u32) -> bool {
    matches!(
        status,
        PDH_CSTATUS_INVALID_DATA
            | PDH_NO_DATA
            | PDH_CALC_NEGATIVE_DENOMINATOR
            | PDH_CALC_NEGATIVE_TIMEBASE
            | PDH_CALC_NEGATIVE_VALUE
    )
}

/// Opaque slot in an adapter's query. Slots are never reused.
pub(in super::super) struct CounterToken {
    index: usize,
}

pub(in super::super) struct NativePdh {
    query: PDH_HQUERY,
    counters: Vec<Option<PDH_HCOUNTER>>,
    node: String,
}

impl NativePdh {
    pub(super) fn open(node: String) -> Result<Self, Error> {
        let mut query = null_mut();
        // SAFETY: Null selects live local data; the output handle is writable.
        check("PdhOpenQueryW", "<query>", unsafe {
            PdhOpenQueryW(null(), 0, &mut query)
        })?;
        Ok(Self {
            query,
            counters: Vec::new(),
            node,
        })
    }

    fn counter_handle(&self, counter: &CounterToken, path: &str) -> Result<PDH_HCOUNTER, Error> {
        self.counters
            .get(counter.index)
            .copied()
            .flatten()
            .ok_or_else(|| Error::InvalidCounterToken {
                path: path.to_owned(),
            })
    }

    pub(super) fn collect_status(status: u32) -> Result<bool, Error> {
        if is_no_observation_status(status) {
            return Ok(false);
        }
        check("PdhCollectQueryData", "<query>", status)?;
        Ok(true)
    }

    pub(super) fn raw_with(
        path: &str,
        read: impl FnOnce(&mut PDH_RAW_COUNTER) -> u32,
    ) -> Result<Option<i64>, Error> {
        let mut raw = PDH_RAW_COUNTER::default();
        let status = read(&mut raw);
        if is_no_observation_status(status) {
            return Ok(None);
        }
        check("PdhGetRawCounterValue", path, status)?;
        if is_no_observation_data_status(raw.CStatus) {
            return Ok(None);
        }
        check_data("raw counter CStatus", path, raw.CStatus)?;
        Ok(Some(raw.SecondValue))
    }

    pub(super) fn counter_type_with(
        path: &str,
        mut info: impl FnMut(&mut u32, Option<&mut [PDH_COUNTER_INFO_W]>) -> Result<u32, Error>,
    ) -> Result<u32, Error> {
        let mut size = 0;
        let status = info(&mut size, None)?;
        if status != PDH_MORE_DATA {
            return Err(Error::Pdh {
                operation: "PdhGetCounterInfoW(size)",
                path: path.to_owned(),
                status,
            });
        }
        if (size as usize) < size_of::<PDH_COUNTER_INFO_W>() {
            return Err(Error::InvalidSample(
                "PdhGetCounterInfoW returned an undersized buffer",
            ));
        }
        // Typed storage guarantees the header's alignment and initializes the
        // trailing buffer; PDH fills both the header and variable-sized data.
        let mut buffer = vec![
            PDH_COUNTER_INFO_W::default();
            (size as usize).div_ceil(size_of::<PDH_COUNTER_INFO_W>())
        ];
        check(
            "PdhGetCounterInfoW",
            path,
            info(&mut size, Some(&mut buffer))?,
        )?;
        if (size as usize) < size_of::<PDH_COUNTER_INFO_W>()
            || (size as usize) > size_of_val(buffer.as_slice())
        {
            return Err(Error::InvalidSample(
                "PdhGetCounterInfoW returned an invalid filled size",
            ));
        }
        // Only a successful fill reaches classification; zero is a valid type.
        Ok(buffer[0].dwType)
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
            check("PdhGetFormattedCounterValue", path, status)?;
        }
        // PDH_INVALID_DATA can carry a more specific failure in CStatus.
        if is_no_observation_data_status(value.CStatus) {
            return Ok(None);
        }
        check_data("formatted counter CStatus", path, value.CStatus)?;
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

    fn add(&mut self, path: &str) -> Result<CounterToken, Error> {
        let wide_path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let mut handle = null_mut();
        // SAFETY: The adapter owns the live query, path is NUL-terminated,
        // and the output handle is writable.
        check("PdhAddEnglishCounterW", path, unsafe {
            PdhAddEnglishCounterW(self.query, wide_path.as_ptr(), 0, &mut handle)
        })?;
        let token = CounterToken {
            index: self.counters.len(),
        };
        self.counters.push(Some(handle));
        Ok(token)
    }

    fn remove(&mut self, counter: &CounterToken, path: &str) -> Result<(), Error> {
        let handle = self.counter_handle(counter, path)?;
        // SAFETY: Lookup returns an owned, live counter; successful removal
        // invalidates its slot before another operation can borrow the adapter.
        check("PdhRemoveCounter", path, unsafe {
            PdhRemoveCounter(handle)
        })?;
        self.counters[counter.index] = None;
        Ok(())
    }

    fn counter_type(&mut self, counter: &CounterToken, path: &str) -> Result<u32, Error> {
        let handle = self.counter_handle(counter, path)?;
        Self::counter_type_with(path, |size, buffer| {
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

    fn collect(&mut self) -> Result<bool, Error> {
        // SAFETY: The adapter owns the live query and is borrowed exclusively.
        Self::collect_status(unsafe { PdhCollectQueryData(self.query) })
    }

    fn raw(&mut self, counter: &CounterToken, path: &str) -> Result<Option<i64>, Error> {
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
        let handle = self.counter_handle(counter, path)?;
        Self::formatted_with(path, format, |flags, value| {
            // SAFETY: Lookup returned an owned, live counter and the adapter
            // supplies initialized, writable storage for the format it decodes.
            unsafe { PdhGetFormattedCounterValue(handle, flags, null_mut(), value) }
        })
    }

    fn timestamp(&mut self) -> Result<i64, Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::InvalidSample("system clock precedes Unix epoch"))?
            .as_nanos();
        i64::try_from(timestamp)
            .map_err(|_| Error::InvalidSample("timestamp exceeds i64 nanoseconds"))
    }
}

impl Drop for NativePdh {
    fn drop(&mut self) {
        // SAFETY: Construction opened this exclusively owned query. Drop is
        // its only close path; no counter can be read through this adapter afterward.
        let status = unsafe { PdhCloseQuery(self.query) };
        if status != 0 {
            otel_arrow_dfe_telemetry::otel_warn!(
                "otelcol.node.windows_perf_counters.close.fail",
                node = self.node.as_str(),
                status = status as u64
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Performance::PdhFormatFromRawValue;

    /// Scenario: Native formatting is requested for an exact large integer and a double above 100.
    /// Guarantees: PDH display scaling is disabled, doubles are not capped, and the matching union field preserves each value.
    #[test]
    fn formatting_requests_unscaled_uncapped_values() {
        for (format, flags, number) in [
            (
                ValueFormat::Integer,
                PDH_FMT_LARGE | PDH_FMT_NOSCALE,
                Number::Integer(i64::MAX),
            ),
            (
                ValueFormat::Double,
                PDH_FMT_DOUBLE | PDH_FMT_NOSCALE | PDH_FMT_NOCAP100,
                Number::Double(250.0),
            ),
        ] {
            let result = NativePdh::formatted_with("fixture", format, |requested, value| {
                assert_eq!(requested, flags);
                value.CStatus = PDH_CSTATUS_VALID_DATA;
                match number {
                    Number::Integer(number) => value.Anonymous.largeValue = number,
                    Number::Double(number) => value.Anonymous.doubleValue = number,
                }
                0
            })
            .unwrap();
            assert_eq!(result, Some(number));
        }
    }

    /// Scenario: Windows formats rate, percentage, fraction and average fixtures from successive raw samples.
    /// Guarantees: Native formulas remain finite, preserve CPU above 100 percent, and use no display scaling.
    #[test]
    fn native_formulas_from_raw_fixtures() {
        for (native_type, first, second, previous_first, previous_second, time_base, expected) in [
            (0x1041_0400, 160, 12, 100, 10, 10, 300.0),
            (0x2051_0500, 150, 200, 100, 180, 10_000_000, 250.0),
            (0x2002_0400, 25, 100, 0, 0, 1, 25.0),
            (0x20C2_0400, 160, 200, 100, 100, 1, 60.0),
            (0x3002_0400, 200, 5, 100, 3, 100, 0.5),
            (0x4002_0500, 200, 5, 100, 3, 1, 50.0),
        ] {
            let current = PDH_RAW_COUNTER {
                CStatus: PDH_CSTATUS_VALID_DATA,
                FirstValue: first,
                SecondValue: second,
                ..Default::default()
            };
            let previous = PDH_RAW_COUNTER {
                CStatus: PDH_CSTATUS_VALID_DATA,
                FirstValue: previous_first,
                SecondValue: previous_second,
                ..Default::default()
            };
            let value =
                NativePdh::formatted_with("fixture", ValueFormat::Double, |flags, value| {
                    // SAFETY: All pointers reference initialized fixture values and writable output.
                    unsafe {
                        PdhFormatFromRawValue(
                            native_type,
                            flags,
                            &time_base,
                            &current,
                            &previous,
                            value,
                        )
                    }
                })
                .unwrap();
            let Some(Number::Double(actual)) = value else {
                panic!("successful native double formatting must return a double");
            };
            assert!((actual - expected).abs() < 1e-9);
        }
    }
}
