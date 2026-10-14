// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! SQL Server configuration and T-SQL query preparation only.
//!
//! The independently opt-in `sql-server` feature exposes validated plans for a
//! future adapter. It does not register a receiver or perform database I/O.

mod config;
mod sql;

pub use config::{SqlServerConfigError, SqlServerConnectionConfig, SqlServerReceiverConfig};
pub use sql::{SqlServerParameter, SqlServerStatement};

#[cfg(test)]
mod tests;
