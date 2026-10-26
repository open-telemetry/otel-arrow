// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! PDH collection on a dedicated worker thread.
//!
//! Each query and its counter handles stay on the thread that opened them.

mod native;
mod query;
mod worker;

pub(super) use native::NativePdh;
pub(super) use worker::{Stopped, Worker};

/// The PDH operations one query needs. Native statuses are returned unchanged.
pub(super) trait Pdh {
    type Counter;

    /// Adds an English counter path, which may wildcard its instance.
    fn add(&mut self, path: &str) -> Result<Self::Counter, u32>;
    /// Collects one sample for every counter in the query.
    fn collect(&mut self) -> Result<(), u32>;
    /// Replaces `items` with every instance in returned order, or returns the call status.
    /// A non-instanced counter returns one item with an empty name.
    fn array(
        &mut self,
        counter: &Self::Counter,
        items: &mut Vec<(String, Value)>,
    ) -> Result<(), u32>;
}

/// A formatted double and its PDH data status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Value {
    pub(super) status: u32,
    pub(super) value: f64,
}

/// One accepted, scaled value.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Point {
    /// Index of the configured counter.
    pub(super) counter: usize,
    /// Instance name after duplicate suffixing; `None` when empty or a lone aggregate.
    pub(super) instance: Option<String>,
    pub(super) value: f64,
    /// When the query was collected.
    pub(super) time_unix_nano: i64,
    /// When the query that produced the value was opened.
    pub(super) start_time_unix_nano: i64,
}

/// The outcome of query start or one scrape, reported to the receiver.
#[derive(Debug, Default)]
pub(super) struct Report {
    /// Accepted values from this scrape's collections.
    pub(super) points: Vec<Point>,
    /// Counter indexes whose query collected in this scrape; priming is not counted.
    pub(super) collected: Vec<usize>,
    /// Counter indexes whose query failed to open or collect, with the status; each is reopened
    /// on the next scrape.
    pub(super) collect_failures: Vec<(usize, u32)>,
    /// `recreate_query` counters whose replacement failed, with the status; the old query is kept.
    pub(super) recreate_failures: Vec<(usize, u32)>,
    /// `recreate_query` counters whose replacement was swapped in.
    pub(super) recreated: Vec<usize>,
    /// Counter indexes whose add failed, with the status.
    pub(super) add_failures: Vec<(usize, u32)>,
    /// Counter indexes added in this scrape.
    pub(super) added: Vec<usize>,
    /// Counter indexes whose array read failed with a non-ignorable status.
    pub(super) read_failures: Vec<(usize, u32)>,
    /// Counter indexes whose array read returned data or an ignorable status.
    pub(super) read: Vec<usize>,
    /// Counters currently in a query.
    pub(super) active_counters: usize,
}

#[cfg(test)]
mod tests;
