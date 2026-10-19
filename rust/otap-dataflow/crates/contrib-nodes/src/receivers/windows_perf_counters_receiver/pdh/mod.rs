// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exact-counter query ownership and synchronous collection. No scheduling or recovery worker.

use super::config::CounterConfig;
use super::model::{
    Number, Sample, SampleFailure, SamplePoint, SampleValue, ScaleError, scale_double,
    scale_integer,
};

mod counter_type;
mod native;
use counter_type::{CounterKind, UnsupportedNativeType, classify_native_type};
use native::NativePdh;

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
    #[error(
        "no configured performance counter could be loaded; startup failures: {}",
        format_startup_failures(.failures)
    )]
    NoCounters { failures: Vec<StartupFailure> },
    #[error("invalid performance-counter token for {path}")]
    InvalidCounterToken { path: String },
    #[error("invalid performance-counter sample: {0}")]
    InvalidSample(&'static str),
    #[error("counter calculation for {path} failed: {message}")]
    Calculation { path: String, message: String },
    #[error("counter scaling for {path} failed: {source}")]
    Scale { path: String, source: ScaleError },
}

fn format_startup_failures(failures: &[StartupFailure]) -> String {
    failures
        .iter()
        .map(|failure| format!("{}: {}", failure.path, failure.error))
        .collect::<Vec<_>>()
        .join("; ")
}

/// The numeric representation requested from PDH.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ValueFormat {
    Integer,
    Double,
}

/// Checked native fields used to validate receiver-side sampling state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RawSample {
    second_value: i64,
    multi_count: u32,
}

/// Owns an open query and its counters; dropping the adapter closes the query.
/// No raw native handle crosses this boundary.
pub(super) trait PdhApi {
    type Counter;

    fn add(&mut self, path: &str) -> Result<Self::Counter, Error>;
    fn remove(&mut self, counter: &Self::Counter, path: &str) -> Result<(), Error>;
    fn counter_type(&mut self, counter: &Self::Counter, path: &str) -> Result<u32, Error>;
    /// Returns whether collection has data; an expected omission returns false.
    fn collect(&mut self) -> Result<bool, Error>;
    /// Returns checked raw sampling fields, an expected omission, or a validated failure.
    fn raw(&mut self, counter: &Self::Counter, path: &str) -> Result<Option<RawSample>, Error>;
    /// Returns an unscaled value, an expected omission, or a validated failure.
    fn formatted(
        &mut self,
        counter: &Self::Counter,
        path: &str,
        format: ValueFormat,
    ) -> Result<Option<Number>, Error>;
    fn timestamp(&mut self) -> Result<i64, Error>;
}

struct CounterHandle<C> {
    config_index: usize,
    path: String,
    handle: C,
    kind: CounterKind,
    scale_power10: i32,
    previous_base: Option<i64>,
}

impl<C> CounterHandle<C> {
    fn reset_readiness(&mut self) {
        self.previous_base = None;
    }

    fn read(&mut self, api: &mut impl PdhApi<Counter = C>) -> Result<SampleValue, Error> {
        if !matches!(
            self.kind,
            CounterKind::Direct | CounterKind::FormattedSingleSample
        ) {
            let raw = match api.raw(&self.handle, &self.path) {
                Ok(Some(raw)) => raw,
                Ok(None) => {
                    self.reset_readiness();
                    return Ok(SampleValue::NoObservation);
                }
                Err(error) => {
                    self.reset_readiness();
                    return Err(error);
                }
            };
            if self.kind == CounterKind::RawFraction {
                let base = raw.second_value;
                if base <= 0 {
                    return Err(Error::Calculation {
                        path: self.path.clone(),
                        message: format!("base denominator must be positive, got {base}"),
                    });
                }
            } else {
                // The next interval uses its own current MultiCount, so even
                // a zero-base sample must advance the raw baseline.
                let previous = self.previous_base.replace(raw.second_value);
                if self.kind == CounterKind::CalculatedTwoSampleWithMultiBase
                    && raw.multi_count == 0
                {
                    return Err(Error::Calculation {
                        path: self.path.clone(),
                        message: "multi-counter base denominator must be positive, got 0"
                            .to_owned(),
                    });
                }
                let Some(previous) = previous else {
                    return Ok(SampleValue::NoObservation);
                };
                if self.kind == CounterKind::CalculatedTwoSampleWithBase {
                    match raw.second_value.cmp(&previous) {
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
        let number = match api.formatted(&self.handle, &self.path, format) {
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

/// Collects one adapter's counters. Unusable counters are skipped during opening;
/// dropping the adapter releases its query and remaining counter handles.
pub(super) struct Query<A: PdhApi = NativePdh> {
    api: A,
    counters: Vec<CounterHandle<A::Counter>>,
    start_time_unix_nano: i64,
    previous_timestamp_unix_nano: i64,
    /// Failed configurations, retained for runtime diagnostics without later recovery.
    pub(super) startup_failures: Vec<StartupFailure>,
}

impl Query {
    /// Loads usable exact counters without collecting or scheduling.
    pub(super) fn open(configs: &[CounterConfig], node: String) -> Result<Self, Error> {
        Self::open_with(NativePdh::open(node)?, configs)
    }
}

impl<A: PdhApi> Query<A> {
    fn open_with(mut api: A, configs: &[CounterConfig]) -> Result<Self, Error> {
        let start = positive_timestamp(api.timestamp()?)?;
        let mut query = Self {
            api,
            counters: Vec::with_capacity(configs.len()),
            start_time_unix_nano: start,
            previous_timestamp_unix_nano: start,
            startup_failures: Vec::new(),
        };
        for (config_index, config) in configs.iter().enumerate() {
            let handle = match query.api.add(&config.path) {
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
                .counter_type(&handle, &config.path)
                .and_then(|native_type| {
                    classify_native_type(&config.path, native_type).map_err(Error::from)
                }) {
                Ok(kind) => kind,
                Err(error) => {
                    let mut diagnostic = error.to_string();
                    if let Err(cleanup) = query.api.remove(&handle, &config.path) {
                        // A failed removal leaves the adapter's slot owning the
                        // handle, so dropping the adapter will still release it.
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
        let has_data = match self.api.collect() {
            Ok(true) => true,
            other => {
                for counter in &mut self.counters {
                    counter.reset_readiness();
                }
                other?
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

#[cfg(test)]
mod tests;
