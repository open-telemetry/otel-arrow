// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error type for the fallible test-suite surfaces.
//!
//! Setup/build helpers panic with rich context (a setup failure is a
//! test-environment bug), while `send`/`recv`-style operations return this so
//! negative tests can assert failures.

use std::fmt;

/// Errors surfaced by fallible test-suite operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TestError {
    /// The mock cluster or a topic could not be set up.
    ClusterSetup(String),
    /// A produce (send) operation failed or timed out.
    Produce(String),
    /// A consume (receive) operation failed.
    Consume(String),
    /// An operation timed out.
    Timeout(String),
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClusterSetup(m) => write!(f, "cluster setup failed: {m}"),
            Self::Produce(m) => write!(f, "produce failed: {m}"),
            Self::Consume(m) => write!(f, "consume failed: {m}"),
            Self::Timeout(m) => write!(f, "operation timed out: {m}"),
        }
    }
}

impl std::error::Error for TestError {}
