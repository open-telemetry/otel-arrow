// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exact-counter query ownership and synchronous collection. No scheduling or recovery worker.
#![allow(unsafe_code)]

use super::config::CounterConfig;
use super::model::{
    Sample, SampleFailure, SamplePoint, SampleValue, ScaleError, scale_double, scale_integer,
};
use super::native_type::{CounterKind, UnsupportedNativeType, classify_native_type};
use std::mem::size_of;
use std::ptr::{null, null_mut};
use std::time::{SystemTime, UNIX_EPOCH};
use windows_sys::Win32::System::Performance::{
    PDH_CALC_NEGATIVE_DENOMINATOR, PDH_CALC_NEGATIVE_TIMEBASE, PDH_CALC_NEGATIVE_VALUE,
    PDH_COUNTER_INFO_W, PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE,
    PDH_FMT_DOUBLE, PDH_FMT_LARGE, PDH_HCOUNTER, PDH_HQUERY, PDH_INVALID_DATA, PDH_MORE_DATA,
    PDH_NO_DATA, PDH_RAW_COUNTER, PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData,
    PdhGetCounterInfoW, PdhGetFormattedCounterValue, PdhGetRawCounterValue, PdhOpenQueryW,
    PdhRemoveCounter,
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
        Err(Error::Pdh {
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

/// Private call seam for deterministic Windows status and ownership tests.
pub(super) trait PdhApi {
    fn open(&mut self) -> Result<PDH_HQUERY, Error>;
    fn add(&mut self, query: PDH_HQUERY, path: &str) -> Result<PDH_HCOUNTER, Error>;
    fn remove(&mut self, counter: PDH_HCOUNTER, path: &str) -> Result<(), Error>;
    fn info(&mut self, counter: PDH_HCOUNTER, size: &mut u32, info: *mut PDH_COUNTER_INFO_W)
    -> u32;
    fn collect(&mut self, query: PDH_HQUERY) -> Result<(), Error>;
    fn raw(&mut self, counter: PDH_HCOUNTER, path: &str) -> Result<PDH_RAW_COUNTER, Error>;
    fn formatted(&mut self, counter: PDH_HCOUNTER, format: u32) -> (u32, PDH_FMT_COUNTERVALUE);
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

    fn info(
        &mut self,
        counter: PDH_HCOUNTER,
        size: &mut u32,
        info: *mut PDH_COUNTER_INFO_W,
    ) -> u32 {
        // SAFETY: The caller supplies a live counter and either null for sizing
        // or an aligned buffer of at least `size` bytes.
        unsafe { PdhGetCounterInfoW(counter, false, size, info) }
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

    fn formatted(&mut self, counter: PDH_HCOUNTER, format: u32) -> (u32, PDH_FMT_COUNTERVALUE) {
        let mut value = PDH_FMT_COUNTERVALUE::default();
        // SAFETY: Counter is live and output storage accommodates either requested format.
        let status =
            unsafe { PdhGetFormattedCounterValue(counter, format, null_mut(), &mut value) };
        // Keep CStatus even when PDH_INVALID_DATA reports a failed calculation.
        (status, value)
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

fn inspect_counter(
    api: &mut impl PdhApi,
    path: &str,
    counter: PDH_HCOUNTER,
) -> Result<CounterKind, Error> {
    let mut size = 0;
    let status = api.info(counter, &mut size, null_mut());
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
    let mut buffer = vec![0usize; (size as usize).div_ceil(size_of::<usize>())];
    let info = buffer.as_mut_ptr().cast::<PDH_COUNTER_INFO_W>();
    check(
        "PdhGetCounterInfoW",
        path,
        api.info(counter, &mut size, info),
    )?;
    // SAFETY: Only a successful fill reaches classification. The aligned buffer
    // contains the fixed header; zero is a valid native type, not a failure sentinel.
    classify_native_type(path, unsafe { (*info).dwType }).map_err(Error::from)
}

struct CounterHandle {
    config_index: usize,
    path: String,
    handle: PDH_HCOUNTER,
    kind: CounterKind,
    scale_power10: i32,
    ready: bool,
    previous_base: Option<i64>,
}

impl CounterHandle {
    fn reset_readiness(&mut self) {
        self.ready = false;
        self.previous_base = None;
    }

    fn read(&mut self, api: &mut impl PdhApi) -> Result<SampleValue, Error> {
        if self.kind != CounterKind::Direct {
            let raw = match api.raw(self.handle, &self.path).and_then(|raw| {
                check_data("raw counter CStatus", &self.path, raw.CStatus)?;
                Ok(raw)
            }) {
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
                let previous_base = self.previous_base.replace(raw.SecondValue);
                if !std::mem::replace(&mut self.ready, true) {
                    return Ok(SampleValue::NoObservation);
                }
                if self.kind == CounterKind::CalculatedTwoSampleWithBase {
                    let Some(previous) = previous_base else {
                        return Ok(SampleValue::NoObservation);
                    };
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
            PDH_FMT_LARGE | PDH_FMT_NOSCALE
        } else {
            PDH_FMT_DOUBLE | PDH_FMT_NOSCALE | PDH_FMT_NOCAP100
        };
        let (status, value) = api.formatted(self.handle, format);
        if is_no_observation_status(status)
            || (status == 0 && is_no_observation_status(value.CStatus))
        {
            self.reset_readiness();
            return Ok(SampleValue::NoObservation);
        }
        check("PdhGetFormattedCounterValue", &self.path, status)?;
        check_data("formatted counter CStatus", &self.path, value.CStatus)?;
        let number = if self.kind == CounterKind::Direct {
            // SAFETY: A successful PDH_FMT_LARGE request initialized largeValue.
            scale_integer(unsafe { value.Anonymous.largeValue }, self.scale_power10)
        } else {
            // SAFETY: A successful PDH_FMT_DOUBLE request initialized doubleValue.
            scale_double(unsafe { value.Anonymous.doubleValue }, self.scale_power10)
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
            let kind = match inspect_counter(&mut query.api, &config.path, handle) {
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
                ready: false,
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
