// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared OTLP exporter utilities.

use futures::stream::{FuturesUnordered, StreamExt};
use std::future::Future;

/// Default maximum number of concurrent in-flight export requests.
#[must_use]
pub const fn default_max_in_flight() -> usize {
    5
}

/// FIFO-ish wrapper around in-flight export requests.
pub struct InFlightExports<Fut, Output>
where
    Fut: Future<Output = Output>,
{
    futures: FuturesUnordered<Fut>,
}

impl<Fut, Output> InFlightExports<Fut, Output>
where
    Fut: Future<Output = Output>,
{
    /// Creates an empty in-flight export queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            futures: FuturesUnordered::new(),
        }
    }

    /// Returns the number of in-flight exports.
    pub fn len(&self) -> usize {
        self.futures.len()
    }

    /// Returns whether there are no in-flight exports.
    pub fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    /// Adds an export request to the queue.
    pub fn push(&mut self, future: Fut) {
        self.futures.push(future);
    }

    /// Returns a future that resolves once the next export finishes.
    pub fn next_completion(&mut self) -> impl Future<Output = Option<Output>> + '_ {
        self.futures.next()
    }
}

impl<Fut, Output> Default for InFlightExports<Fut, Output>
where
    Fut: Future<Output = Output>,
{
    fn default() -> Self {
        Self::new()
    }
}
