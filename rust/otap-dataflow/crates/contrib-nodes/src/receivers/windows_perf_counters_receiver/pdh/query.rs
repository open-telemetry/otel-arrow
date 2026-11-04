// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! PDH queries: start, add retry, collection, reads, reopen after failure, and
//! per-scrape replacement for `recreate_query` counters.

use super::super::config::{CounterConfig, emitted_instances};
use super::{Pdh, Point, Report, Value};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use windows_sys::Win32::System::Performance::{
    PDH_CALC_NEGATIVE_DENOMINATOR, PDH_CSTATUS_INVALID_DATA, PDH_CSTATUS_NEW_DATA,
    PDH_CSTATUS_VALID_DATA, PDH_INSUFFICIENT_BUFFER, PDH_INVALID_DATA, PDH_NO_DATA,
};

/// How long to wait before collecting again after a negative denominator.
const NEGATIVE_DENOMINATOR_RETRY: Duration = Duration::from_secs(1);

/// Most instances one scrape keeps as points, across all counters in configuration order. A
/// counter whose array would exceed the remainder is read but skipped whole for that scrape as a
/// read failure, never truncated, since PDH's instance order is not stable. Splitting counters
/// across receivers raises the total.
const MAX_INSTANCES_PER_SCRAPE: usize = 100_000;

/// One counter's open query and, once its add succeeds, the counter handle.
struct Query<P: Pdh> {
    query: P,
    handle: Option<P::Counter>,
}

/// Owns every query: one per configured counter, so a failed query stops only its counter.
pub(super) struct Collector<P: Pdh, F> {
    open: F,
    counters: Vec<CounterConfig>,
    /// `None` after the counter's query failed to open, prime or collect, until a scrape reopens
    /// it; `Some` is an open query, whose add may still await a retry.
    queries: Vec<Option<Query<P>>>,
    report: Report,
    items: Vec<(String, Value)>,
    /// Set on shutdown: no further counter work starts.
    stop: Arc<AtomicBool>,
    /// Cumulative start time: when the collector started. Reopening or recreating a query does not
    /// reset the OS counter, so neither changes it; only a read at an earlier time, as after a
    /// backward clock step, moves it back.
    pub(super) start_time_unix_nano: i64,
}

impl<P: Pdh, F: FnMut() -> Result<P, u32>> Collector<P, F> {
    /// Opens, adds every path, and primes each query until `stop` is set; returns the start
    /// report.
    pub(super) fn start(
        counters: Vec<CounterConfig>,
        mut open: F,
        stop: Arc<AtomicBool>,
    ) -> (Self, Report) {
        let mut report = Report::default();
        let queries = counters
            .iter()
            .enumerate()
            .map(|(index, counter)| {
                if stop.load(Ordering::Relaxed) {
                    return None;
                }
                open_query(&mut open, index, counter, &mut report)
            })
            .collect();
        let mut collector = Self {
            open,
            counters,
            queries,
            report,
            items: Vec::new(),
            stop,
            start_time_unix_nano: now_unix_nano(),
        };
        let report = collector.take_report();
        (collector, report)
    }

    /// Runs one scrape until `stop` is set. A query that failed is reopened and primed instead of
    /// read.
    pub(super) fn scrape(&mut self) -> Report {
        let report = &mut self.report;
        for (index, (slot, counter)) in self.queries.iter_mut().zip(&self.counters).enumerate() {
            if self.stop.load(Ordering::Relaxed) {
                break;
            }
            let Some(query) = slot else {
                *slot = open_query(&mut self.open, index, counter, report);
                continue;
            };
            if query.handle.is_none() {
                query.handle = add(&mut query.query, index, counter, report);
            }
            let Some(handle) = &query.handle else {
                continue;
            };
            match collect(&mut query.query, &self.stop) {
                Ok(true) => read(
                    &mut query.query,
                    handle,
                    index,
                    counter,
                    &mut self.items,
                    &mut self.start_time_unix_nano,
                    report,
                ),
                // No counter has data yet; this is not a failure.
                Ok(false) => {}
                Err(status) => {
                    // Dropping the query closes it; the next scrape reopens it.
                    *slot = None;
                    report.collect_failures.push((index, status));
                    continue;
                }
            }
            report.collected.push(index);
            if counter.recreate_query && !self.stop.load(Ordering::Relaxed) {
                // Swaps in a primed replacement; on failure keeps the current query.
                let replacement = (self.open)().and_then(|mut query| {
                    let handle = query.add(&counter.path)?;
                    prime(&mut query)?;
                    Ok(Query {
                        query,
                        handle: Some(handle),
                    })
                });
                match replacement {
                    Ok(replacement) => {
                        *slot = Some(replacement);
                        report.recreated.push(index);
                    }
                    Err(status) => report.recreate_failures.push((index, status)),
                }
            }
        }
        self.take_report()
    }

    fn take_report(&mut self) -> Report {
        self.report.active_counters = self
            .queries
            .iter()
            .flatten()
            .filter(|query| query.handle.is_some())
            .count();
        std::mem::take(&mut self.report)
    }
}

/// Opens a fresh query, adds the counter, and primes it; `None` reports a collect failure.
fn open_query<P: Pdh>(
    open: &mut impl FnMut() -> Result<P, u32>,
    index: usize,
    counter: &CounterConfig,
    report: &mut Report,
) -> Option<Query<P>> {
    let opened = open().and_then(|mut query| {
        let handle = add(&mut query, index, counter, report);
        if handle.is_some() {
            prime(&mut query)?;
        }
        Ok(Query { query, handle })
    });
    opened
        .map_err(|status| report.collect_failures.push((index, status)))
        .ok()
}

fn add<P: Pdh>(
    query: &mut P,
    index: usize,
    counter: &CounterConfig,
    report: &mut Report,
) -> Option<P::Counter> {
    match query.add(&counter.path) {
        Ok(handle) => {
            report.added.push(index);
            Some(handle)
        }
        Err(status) => {
            report.add_failures.push((index, status));
            None
        }
    }
}

/// Reads the counter as an array within the scrape's instance budget, and names its instances with
/// [`emitted_instances`].
fn read<P: Pdh>(
    query: &mut P,
    handle: &P::Counter,
    index: usize,
    counter: &CounterConfig,
    items: &mut Vec<(String, Value)>,
    start_time_unix_nano: &mut i64,
    report: &mut Report,
) {
    let time_unix_nano = now_unix_nano();
    *start_time_unix_nano = (*start_time_unix_nano).min(time_unix_nano);
    match query.array(handle, items) {
        Ok(()) => {}
        // No observation, for example a rate counter's first sample.
        Err(
            PDH_CSTATUS_INVALID_DATA
            | PDH_INVALID_DATA
            | PDH_NO_DATA
            | PDH_CALC_NEGATIVE_DENOMINATOR,
        ) => return report.read.push(index),
        Err(status) => return report.read_failures.push((index, status)),
    }
    if report.points.len() + items.len() > MAX_INSTANCES_PER_SCRAPE {
        return report.read_failures.push((index, PDH_INSUFFICIENT_BUFFER));
    }
    report.read.push(index);
    let names = items
        .iter()
        .map(|(name, value)| (name.as_str(), accepted(value)));
    for (position, instance) in emitted_instances(names, counter.aggregate.as_deref()) {
        report.points.push(Point {
            counter: index,
            instance,
            value: scale(items[position].1.value, counter.scale_power10),
            time_unix_nano,
            start_time_unix_nano: *start_time_unix_nano,
        });
    }
}

/// Collects and returns whether there is data. A negative denominator collects once more after
/// a pause, unless `stop` is set before or during the pause; any error from that retry, including
/// no data, is a collect failure.
fn collect<P: Pdh>(query: &mut P, stop: &AtomicBool) -> Result<bool, u32> {
    match query.collect() {
        Ok(()) => Ok(true),
        Err(PDH_NO_DATA) => Ok(false),
        Err(PDH_CALC_NEGATIVE_DENOMINATOR) => {
            if !stop.load(Ordering::Relaxed) {
                std::thread::sleep(NEGATIVE_DENOMINATOR_RETRY);
            }
            if stop.load(Ordering::Relaxed) {
                return Ok(false);
            }
            query.collect().map(|()| true)
        }
        Err(status) => Err(status),
    }
}

/// The first collection, which only establishes a baseline for rate counters.
fn prime<P: Pdh>(query: &mut P) -> Result<(), u32> {
    match query.collect() {
        Err(PDH_NO_DATA) => Ok(()),
        result => result,
    }
}

fn accepted(value: &Value) -> bool {
    matches!(value.status, PDH_CSTATUS_VALID_DATA | PDH_CSTATUS_NEW_DATA)
}

fn scale(value: f64, scale_power10: i32) -> f64 {
    let factor = 10_f64.powi(scale_power10.abs());
    if scale_power10 < 0 {
        value / factor
    } else {
        value * factor
    }
}

fn now_unix_nano() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_nanos()).unwrap_or(i64::MAX)
        })
}
