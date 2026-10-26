// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! PDH queries: start, add retry, collection, reads, reopen after failure, and
//! per-scrape replacement for `recreate_query` counters.

use super::super::config::CounterConfig;
use super::{Pdh, Point, Report, Value};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use windows_sys::Win32::System::Performance::{
    PDH_CALC_NEGATIVE_DENOMINATOR, PDH_CSTATUS_INVALID_DATA, PDH_CSTATUS_NEW_DATA,
    PDH_CSTATUS_VALID_DATA, PDH_INVALID_DATA, PDH_NO_DATA,
};

/// Go waits this long before collecting again after a negative denominator.
const NEGATIVE_DENOMINATOR_RETRY: Duration = Duration::from_secs(1);

/// One query and the counters added to it.
struct Group<P: Pdh> {
    /// Configured counter indexes.
    counters: Vec<usize>,
    /// Replace the query after every successful scrape.
    recreate: bool,
    query: Option<P>,
    handles: Vec<Option<P::Counter>>,
    start_time_unix_nano: i64,
}

/// Owns every query: one per configured counter, as in Go, so a failed query stops only its counter.
pub(super) struct Collector<P: Pdh, F> {
    open: F,
    counters: Vec<CounterConfig>,
    groups: Vec<Group<P>>,
    report: Report,
    items: Vec<(String, Value)>,
}

impl<P: Pdh, F: FnMut() -> Result<P, u32>> Collector<P, F> {
    /// Opens, adds every path, and primes each query; returns the start report.
    pub(super) fn start(counters: Vec<CounterConfig>, open: F) -> (Self, Report) {
        let groups = counters
            .iter()
            .enumerate()
            .map(|(index, counter)| Group {
                counters: vec![index],
                recreate: counter.recreate_query,
                query: None,
                handles: vec![None],
                start_time_unix_nano: 0,
            })
            .collect();
        let mut collector = Self {
            open,
            counters,
            groups,
            report: Report::default(),
            items: Vec::new(),
        };
        for group in &mut collector.groups {
            group.open(
                &mut collector.open,
                &collector.counters,
                &mut collector.report,
            );
        }
        let report = collector.take_report();
        (collector, report)
    }

    /// Runs one scrape. A query that failed is reopened and primed instead of read.
    pub(super) fn scrape(&mut self) -> Report {
        for group in &mut self.groups {
            let Some(query) = group.query.as_mut() else {
                group.open(&mut self.open, &self.counters, &mut self.report);
                continue;
            };
            add_missing(
                query,
                &group.counters,
                &mut group.handles,
                &self.counters,
                &mut self.report,
            );
            if group.handles.iter().all(Option::is_none) {
                continue;
            }
            match collect(query) {
                Ok(true) => group.read(&self.counters, &mut self.items, &mut self.report),
                // No counter has data yet; this is not a failure.
                Ok(false) => {}
                Err(status) => {
                    group.fail(status, &mut self.report);
                    continue;
                }
            }
            self.report.collected.extend(&group.counters);
            if group.recreate {
                group.replace(&mut self.open, &self.counters, &mut self.report);
            }
        }
        self.take_report()
    }

    fn take_report(&mut self) -> Report {
        self.report.active_counters = self
            .groups
            .iter()
            .map(|group| group.handles.iter().flatten().count())
            .sum();
        std::mem::take(&mut self.report)
    }
}

impl<P: Pdh> Group<P> {
    /// Opens a fresh query, adds every counter, and primes it.
    fn open<F: FnMut() -> Result<P, u32>>(
        &mut self,
        open: &mut F,
        counters: &[CounterConfig],
        report: &mut Report,
    ) {
        self.query = None;
        self.handles.iter_mut().for_each(|handle| *handle = None);
        self.start_time_unix_nano = now_unix_nano();
        let mut query = match open() {
            Ok(query) => query,
            Err(status) => return self.fail(status, report),
        };
        add_missing(
            &mut query,
            &self.counters,
            &mut self.handles,
            counters,
            report,
        );
        if self.handles.iter().any(Option::is_some)
            && let Err(status) = prime(&mut query)
        {
            return self.fail(status, report);
        }
        self.query = Some(query);
    }

    /// Swaps in a primed replacement query; on failure keeps the current one.
    fn replace<F: FnMut() -> Result<P, u32>>(
        &mut self,
        open: &mut F,
        counters: &[CounterConfig],
        report: &mut Report,
    ) {
        let start_time_unix_nano = now_unix_nano();
        let replacement = open().and_then(|mut query| {
            let handles = self
                .counters
                .iter()
                .map(|&index| query.add(&counters[index].path).map(Some))
                .collect::<Result<Vec<_>, u32>>()?;
            prime(&mut query)?;
            Ok((query, handles))
        });
        match replacement {
            Ok((query, handles)) => {
                // The old query closes when it is dropped here.
                self.query = Some(query);
                self.handles = handles;
                self.start_time_unix_nano = start_time_unix_nano;
                report.recreated.extend(&self.counters);
            }
            Err(status) => report
                .recreate_failures
                .extend(self.counters.iter().map(|&index| (index, status))),
        }
    }

    /// Drops the query so the next scrape reopens it, and reports the failure for its counters.
    fn fail(&mut self, status: u32, report: &mut Report) {
        self.query = None;
        self.handles.iter_mut().for_each(|handle| *handle = None);
        report
            .collect_failures
            .extend(self.counters.iter().map(|&index| (index, status)));
    }

    /// Reads every added counter as an array and applies Go's instance naming.
    fn read(
        &mut self,
        counters: &[CounterConfig],
        items: &mut Vec<(String, Value)>,
        report: &mut Report,
    ) {
        let Some(query) = self.query.as_mut() else {
            return;
        };
        let time_unix_nano = now_unix_nano();
        for (&index, handle) in self.counters.iter().zip(&self.handles) {
            let Some(handle) = handle else {
                continue;
            };
            match query.array(handle, items) {
                Ok(()) => report.read.push(index),
                // No observation, for example a rate counter's first sample.
                Err(
                    PDH_CSTATUS_INVALID_DATA
                    | PDH_INVALID_DATA
                    | PDH_NO_DATA
                    | PDH_CALC_NEGATIVE_DENOMINATOR,
                ) => {
                    report.read.push(index);
                    continue;
                }
                Err(status) => {
                    report.read_failures.push((index, status));
                    continue;
                }
            }
            let counter = &counters[index];
            let aggregate = counter.aggregate.as_deref();
            items.retain(|(_, value)| accepted(value));
            // Go order: clear a lone aggregate's name, suffix duplicates, then drop the aggregate.
            if let [(name, _)] = items.as_mut_slice()
                && aggregate == Some(name.as_str())
            {
                name.clear();
            }
            let mut seen = HashMap::<&str, u32>::new();
            for (name, value) in items.iter() {
                let occurrence = seen.entry(name).or_default();
                let instance = match *occurrence {
                    0 => name.clone(),
                    n => format!("{name}#{n}"),
                };
                *occurrence += 1;
                if aggregate == Some(instance.as_str()) {
                    continue;
                }
                report.points.push(Point {
                    counter: index,
                    instance: (!instance.is_empty()).then_some(instance),
                    value: scale(value.value, counter.scale_power10),
                    time_unix_nano,
                    start_time_unix_nano: self.start_time_unix_nano,
                });
            }
        }
    }
}

/// Collects and returns whether there is data. As in Go, a negative denominator collects once
/// more after a pause; any error from that retry, including no data, is a collect failure.
fn collect<P: Pdh>(query: &mut P) -> Result<bool, u32> {
    match query.collect() {
        Ok(()) => Ok(true),
        Err(PDH_NO_DATA) => Ok(false),
        Err(PDH_CALC_NEGATIVE_DENOMINATOR) => {
            std::thread::sleep(NEGATIVE_DENOMINATOR_RETRY);
            query.collect().map(|()| true)
        }
        Err(status) => Err(status),
    }
}

fn add_missing<P: Pdh>(
    query: &mut P,
    indexes: &[usize],
    handles: &mut [Option<P::Counter>],
    counters: &[CounterConfig],
    report: &mut Report,
) {
    for (&index, handle) in indexes.iter().zip(handles) {
        if handle.is_none() {
            match query.add(&counters[index].path) {
                Ok(added) => {
                    *handle = Some(added);
                    report.added.push(index);
                }
                Err(status) => report.add_failures.push((index, status)),
            }
        }
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
