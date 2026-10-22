// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Native PDH access, query-owned sampling state, discovery, and retries.
//!
//! Windows handles remain on the synchronous thread that owns the query.

use super::model::{InstanceIdentity, Number};
pub(super) use expansion::DiscoveryBudget;
mod counter_type;
mod error;
mod expansion;
mod native;
mod query;
mod timing;

pub(super) use error::{Error, PdhOperation};

#[cfg(test)]
static QUERY_OPENS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[cfg(test)]
static QUERY_CLOSES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// The numeric representation requested from PDH.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ValueFormat {
    Integer,
    Double,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CounterLanguage {
    English,
    Localized,
}

/// Checked native fields used to validate receiver-side sampling state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RawSample {
    pub(super) second_value: i64,
    pub(super) multi_count: u32,
}

/// Owns an open query and its counters; raw handles and unchecked statuses stay native.
pub(super) trait PdhApi {
    type Counter;

    /// Observe stop between native operations without attempting native cancellation.
    fn check_running(&self) -> Result<(), Error> {
        Ok(())
    }

    fn add(&mut self, path: &str, language: CounterLanguage) -> Result<Self::Counter, Error>;
    fn remove(&mut self, counter: &Self::Counter, path: &str) -> Result<(), Error>;
    fn counter_type(&mut self, counter: &Self::Counter, path: &str) -> Result<u32, Error>;
    /// Reads the native display-language path without changing the configured path.
    fn full_path(&mut self, counter: &Self::Counter, path: &str) -> Result<String, Error>;
    /// Parses a comparison-only local path with canonical instance suffix spelling.
    fn comparison_path(&mut self, path: &str) -> Result<String, Error>;
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
    fn timestamp(&mut self) -> Result<i64, Error> {
        Self::current_timestamp()
    }
    /// Reads the same wall clock even when no query could be opened.
    fn current_timestamp() -> Result<i64, Error> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::Timestamp("system clock precedes Unix epoch"))?
            .as_nanos();
        i64::try_from(timestamp).map_err(|_| Error::Timestamp("timestamp exceeds i64 nanoseconds"))
    }
    /// Resolves an English template and returns concrete host-language paths.
    /// Charge the shared budget before allocating raw buffers or decoded paths.
    fn expand(&mut self, path: &str, budget: &mut DiscoveryBudget) -> Result<Vec<String>, Error>;
    /// Refreshes the local object list and returns its object count.
    fn refresh_objects(&mut self) -> Result<usize, Error>;
    /// Parses the instance identity from one concrete path.
    fn parse_instance(&mut self, path: &str) -> Result<InstanceIdentity, Error>;
}

#[cfg(test)]
mod tests;
