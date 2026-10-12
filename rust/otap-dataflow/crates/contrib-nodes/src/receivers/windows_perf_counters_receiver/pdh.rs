// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exact-counter query ownership and synchronous collection. No scheduling or recovery worker.
#![allow(unsafe_code)]

use super::config::CounterConfig;
use super::model::{
    Number, Sample, SampleFailure, SamplePoint, SampleValue, ScaleError, scale_double,
    scale_integer,
};
use super::native_type::{CounterKind, UnsupportedNativeType, classify_native_type};
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

/// A configured counter that could not be loaded; it is not retried later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StartupFailure {
    /// Original index into the configured counters.
    pub(super) counter_index: usize,
    /// Exact path that failed to add or inspect.
    pub(super) path: String,
    /// Add or inspection failure, plus any handle-removal error.
    pub(super) error: String,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum Error {
    #[error("{operation} for {path} failed with PDH status 0x{status:08X}")]
    Pdh {
        operation: &'static str,
        path: String,
        status: u32,
    },
    /// A counter's CStatus failure, separate from a PDH API return code.
    #[error("{operation} for {path} failed with counter data status 0x{status:08X}")]
    CounterData {
        operation: &'static str,
        path: String,
        status: u32,
    },
    #[error(transparent)]
    UnsupportedType(#[from] UnsupportedNativeType),
    #[error("no configured performance counter could be loaded; startup failures: {failures:?}")]
    NoCounters { failures: Vec<StartupFailure> },
    #[error("invalid performance-counter sample: {0}")]
    InvalidSample(&'static str),
    #[error("counter calculation for {path} failed: {message}")]
    Calculation { path: String, message: String },
    #[error("counter scaling for {path} failed: {source}")]
    Scale { path: String, source: ScaleError },
}

fn check(operation: &'static str, path: &str, status: u32) -> Result<(), Error> {
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

impl Error {
    fn is_no_observation(&self) -> bool {
        matches!(self, Self::Pdh { status, .. } if is_no_observation_status(*status))
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

/// The numeric representation requested from PDH.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ValueFormat {
    Integer,
    Double,
}

/// Native operations with metadata storage and formatted-value decoding owned by the adapter.
pub(super) trait PdhApi {
    fn open(&mut self) -> Result<PDH_HQUERY, Error>;
    fn add(&mut self, query: PDH_HQUERY, path: &str) -> Result<PDH_HCOUNTER, Error>;
    fn remove(&mut self, counter: PDH_HCOUNTER, path: &str) -> Result<(), Error>;
    fn counter_type(&mut self, counter: PDH_HCOUNTER, path: &str) -> Result<u32, Error>;
    fn collect(&mut self, query: PDH_HQUERY) -> Result<(), Error>;
    fn raw(&mut self, counter: PDH_HCOUNTER, path: &str) -> Result<PDH_RAW_COUNTER, Error>;
    /// Returns an unscaled value, an expected omission, or a validated failure.
    fn formatted(
        &mut self,
        counter: PDH_HCOUNTER,
        path: &str,
        format: ValueFormat,
    ) -> Result<Option<Number>, Error>;
    fn close(&mut self, query: PDH_HQUERY) -> u32;
    fn timestamp(&mut self) -> Result<i64, Error>;
}

pub(super) struct NativePdh;

impl PdhApi for NativePdh {
    fn open(&mut self) -> Result<PDH_HQUERY, Error> {
        let mut handle = null_mut();
        // SAFETY: Null selects live local data; the output handle is writable.
        check("PdhOpenQueryW", "<query>", unsafe {
            PdhOpenQueryW(null(), 0, &mut handle)
        })?;
        Ok(handle)
    }

    fn add(&mut self, query: PDH_HQUERY, path: &str) -> Result<PDH_HCOUNTER, Error> {
        let wide_path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let mut handle = null_mut();
        // SAFETY: Query is live, path is NUL-terminated, and output is writable.
        check("PdhAddEnglishCounterW", path, unsafe {
            PdhAddEnglishCounterW(query, wide_path.as_ptr(), 0, &mut handle)
        })?;
        Ok(handle)
    }

    fn remove(&mut self, counter: PDH_HCOUNTER, path: &str) -> Result<(), Error> {
        // SAFETY: Counter belongs to this query and is not used after successful removal.
        check("PdhRemoveCounter", path, unsafe {
            PdhRemoveCounter(counter)
        })
    }

    fn counter_type(&mut self, counter: PDH_HCOUNTER, path: &str) -> Result<u32, Error> {
        Self::counter_type_with(path, |size, buffer| {
            let info = if let Some(buffer) = buffer {
                if (*size as usize) > size_of_val(buffer) {
                    return Err(Error::InvalidSample(
                        "PdhGetCounterInfoW requested size exceeds buffer capacity",
                    ));
                }
                buffer.as_mut_ptr()
            } else {
                null_mut()
            };
            // SAFETY: The adapter provides either null for sizing or aligned
            // storage with capacity for the requested size. Counter is live.
            Ok(unsafe { PdhGetCounterInfoW(counter, false, size, info) })
        })
    }

    fn collect(&mut self, query: PDH_HQUERY) -> Result<(), Error> {
        // SAFETY: Query owns live counter handles and is borrowed exclusively.
        check("PdhCollectQueryData", "<query>", unsafe {
            PdhCollectQueryData(query)
        })
    }

    fn raw(&mut self, counter: PDH_HCOUNTER, path: &str) -> Result<PDH_RAW_COUNTER, Error> {
        let mut raw = PDH_RAW_COUNTER::default();
        // SAFETY: Counter is live and the initialized output is writable.
        check("PdhGetRawCounterValue", path, unsafe {
            PdhGetRawCounterValue(counter, null_mut(), &mut raw)
        })?;
        Ok(raw)
    }

    fn formatted(
        &mut self,
        counter: PDH_HCOUNTER,
        path: &str,
        format: ValueFormat,
    ) -> Result<Option<Number>, Error> {
        Self::formatted_with(path, format, |flags, value| {
            // SAFETY: Counter is live and the adapter supplies initialized,
            // writable storage for the format it will decode.
            unsafe { PdhGetFormattedCounterValue(counter, flags, null_mut(), value) }
        })
    }

    fn close(&mut self, query: PDH_HQUERY) -> u32 {
        // SAFETY: Query has one owner and no counter is used after closing.
        unsafe { PdhCloseQuery(query) }
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

impl NativePdh {
    fn counter_type_with(
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

    fn formatted_with(
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

struct CounterHandle {
    config_index: usize,
    path: String,
    handle: PDH_HCOUNTER,
    kind: CounterKind,
    scale_power10: i32,
    previous_base: Option<i64>,
}

impl CounterHandle {
    fn reset_readiness(&mut self) {
        self.previous_base = None;
    }

    fn read(&mut self, api: &mut impl PdhApi) -> Result<SampleValue, Error> {
        if self.kind != CounterKind::Direct {
            let raw = match api.raw(self.handle, &self.path) {
                Ok(raw) => raw,
                Err(error) => {
                    self.reset_readiness();
                    return if error.is_no_observation() {
                        Ok(SampleValue::NoObservation)
                    } else {
                        Err(error)
                    };
                }
            };
            if is_no_observation_data_status(raw.CStatus) {
                self.reset_readiness();
                return Ok(SampleValue::NoObservation);
            }
            if let Err(error) = check_data("raw counter CStatus", &self.path, raw.CStatus) {
                self.reset_readiness();
                return Err(error);
            }
            if self.kind == CounterKind::RawFraction {
                if raw.SecondValue <= 0 {
                    return Err(Error::Calculation {
                        path: self.path.clone(),
                        message: format!(
                            "base denominator must be positive, got {}",
                            raw.SecondValue
                        ),
                    });
                }
            } else {
                let Some(previous) = self.previous_base.replace(raw.SecondValue) else {
                    return Ok(SampleValue::NoObservation);
                };
                if self.kind == CounterKind::CalculatedTwoSampleWithBase {
                    match raw.SecondValue.cmp(&previous) {
                        std::cmp::Ordering::Less => {
                            self.reset_readiness();
                            return Ok(SampleValue::NoObservation);
                        }
                        std::cmp::Ordering::Equal => return Ok(SampleValue::NoObservation),
                        std::cmp::Ordering::Greater => {}
                    }
                }
            }
        }

        let format = if self.kind == CounterKind::Direct {
            ValueFormat::Integer
        } else {
            ValueFormat::Double
        };
        let number = match api.formatted(self.handle, &self.path, format) {
            Ok(Some(number)) => number,
            Ok(None) => {
                self.reset_readiness();
                return Ok(SampleValue::NoObservation);
            }
            Err(error) => {
                if matches!(error, Error::CounterData { .. }) {
                    self.reset_readiness();
                }
                return Err(error);
            }
        };
        let number = match number {
            Number::Integer(value) => scale_integer(value, self.scale_power10),
            Number::Double(value) => scale_double(value, self.scale_power10),
        }
        .map_err(|source| Error::Scale {
            path: self.path.clone(),
            source,
        })?;
        Ok(SampleValue::Value(number))
    }
}

/// Owns one query and its counters. Unusable counters are skipped during opening;
/// dropping closes every handle still owned by the query.
pub(super) struct Query<A: PdhApi = NativePdh> {
    api: A,
    handle: PDH_HQUERY,
    counters: Vec<CounterHandle>,
    node: String,
    start_time_unix_nano: i64,
    previous_timestamp_unix_nano: i64,
    /// Failed configurations, retained for runtime diagnostics without later recovery.
    pub(super) startup_failures: Vec<StartupFailure>,
}

impl Query {
    /// Loads usable exact counters without collecting or scheduling.
    pub(super) fn open(configs: &[CounterConfig], node: String) -> Result<Self, Error> {
        Self::open_with(NativePdh, configs, node)
    }
}

impl<A: PdhApi> Query<A> {
    fn open_with(mut api: A, configs: &[CounterConfig], node: String) -> Result<Self, Error> {
        let start = positive_timestamp(api.timestamp()?)?;
        let handle = api.open()?;
        let mut query = Self {
            api,
            handle,
            counters: Vec::with_capacity(configs.len()),
            node,
            start_time_unix_nano: start,
            previous_timestamp_unix_nano: start,
            startup_failures: Vec::new(),
        };
        for (config_index, config) in configs.iter().enumerate() {
            let handle = match query.api.add(query.handle, &config.path) {
                Ok(handle) => handle,
                Err(error) => {
                    query.startup_failures.push(StartupFailure {
                        counter_index: config_index,
                        path: config.path.clone(),
                        error: error.to_string(),
                    });
                    continue;
                }
            };
            let kind = match query
                .api
                .counter_type(handle, &config.path)
                .and_then(|native_type| {
                    classify_native_type(&config.path, native_type).map_err(Error::from)
                }) {
                Ok(kind) => kind,
                Err(error) => {
                    let mut diagnostic = error.to_string();
                    if let Err(cleanup) = query.api.remove(handle, &config.path) {
                        // A failed removal leaves the handle owned by the query,
                        // so dropping the query will still release it.
                        diagnostic.push_str(&format!("; {cleanup}"));
                    }
                    query.startup_failures.push(StartupFailure {
                        counter_index: config_index,
                        path: config.path.clone(),
                        error: diagnostic,
                    });
                    continue;
                }
            };
            query.counters.push(CounterHandle {
                config_index,
                path: config.path.clone(),
                handle,
                kind,
                scale_power10: config.scale_power10,
                previous_base: None,
            });
        }
        if query.counters.is_empty() {
            return Err(Error::NoCounters {
                failures: std::mem::take(&mut query.startup_failures),
            });
        }
        Ok(query)
    }

    /// Retries all existing handles on each call. The first valid raw sample
    /// warms calculated counters; no failure recreates handles or starts a sequence.
    pub(super) fn collect(&mut self) -> Result<Sample, Error> {
        let has_data = match self.api.collect(self.handle) {
            Ok(()) => true,
            Err(error) => {
                for counter in &mut self.counters {
                    counter.reset_readiness();
                }
                if !error.is_no_observation() {
                    return Err(error);
                }
                false
            }
        };
        let timestamp = match self.api.timestamp().and_then(positive_timestamp) {
            Ok(timestamp) => timestamp,
            Err(error) => {
                // PDH already advanced, but this sample cannot be observed. Do
                // not compare the next raw base with one from two scrapes ago.
                for counter in &mut self.counters {
                    counter.reset_readiness();
                }
                return Err(error);
            }
        };
        if timestamp < self.previous_timestamp_unix_nano {
            self.start_time_unix_nano = timestamp;
        }
        self.previous_timestamp_unix_nano = timestamp;
        let mut sample = Sample {
            start_time_unix_nano: self.start_time_unix_nano,
            timestamp_unix_nano: timestamp,
            points: Vec::with_capacity(self.counters.len()),
            failures: Vec::new(),
        };
        // A counter error must not short-circuit later reads: PDH advances all
        // counters together, so receiver-side base history must advance too.
        for counter in &mut self.counters {
            let value = if has_data {
                counter.read(&mut self.api)
            } else {
                Ok(SampleValue::NoObservation)
            };
            match value {
                Ok(value) => sample.points.push(SamplePoint {
                    counter_index: counter.config_index,
                    value,
                }),
                Err(error) => sample.failures.push(SampleFailure {
                    counter_index: counter.config_index,
                    error: error.to_string(),
                }),
            }
        }
        Ok(sample)
    }
}

fn positive_timestamp(timestamp: i64) -> Result<i64, Error> {
    if timestamp > 0 {
        Ok(timestamp)
    } else {
        Err(Error::InvalidSample("timestamp must be positive"))
    }
}

impl<A: PdhApi> Drop for Query<A> {
    fn drop(&mut self) {
        let status = self.api.close(self.handle);
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
mod tests;
