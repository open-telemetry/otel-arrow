// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Preserve portable protocol types without a mandatory database dependency.

use crate::{Error, Grant, Limits, OperationId, Outcome, Progress, ProgressUpdate};

/// No storage instance can be constructed when the local backend is unavailable.
pub enum Store {}
impl Store {
    /// Reject an unsupported build/platform before accessing the filesystem.
    pub fn open(_root: &std::path::Path, _namespace: &str, _limits: Limits) -> Result<Self, Error> {
        Err(Error::Invalid(
            "durable authority requires the SQLite feature on Linux",
        ))
    }
    /// A disabled backend never provides an epoch.
    #[must_use]
    pub fn epoch(&self) -> u64 {
        match *self {}
    }
    /// A disabled backend never activates authority.
    pub fn activate(&mut self, _operation: OperationId, _grant: &Grant) -> Result<Outcome, Error> {
        match *self {}
    }
    /// A disabled backend never retires authority.
    pub fn retire(&mut self, _operation: OperationId, _grant: &Grant) -> Result<Outcome, Error> {
        match *self {}
    }
    /// A disabled backend never commits progress.
    pub fn commit(
        &mut self,
        _operation: OperationId,
        _updates: &[ProgressUpdate],
    ) -> Result<Outcome, Error> {
        match *self {}
    }
    /// A disabled backend never certifies an outcome.
    pub fn resolve(&mut self, _operation: OperationId) -> Result<Outcome, Error> {
        match *self {}
    }
    /// A disabled backend never returns source state.
    pub fn progress(&mut self, _source: &str) -> Result<Option<Progress>, Error> {
        match *self {}
    }
    /// Test callbacks cannot enable an unsupported backend.
    #[cfg(feature = "test-utils")]
    pub fn set_fault_hook(&mut self, _hook: impl FnMut(crate::FaultPoint) + Send + 'static) {
        match *self {}
    }
}
