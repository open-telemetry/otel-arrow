// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! SQL Server preparation and driver integration with an injected connector.
//!
//! The independently opt-in `sql-server` feature exposes a local adapter for the
//! shared scraper. No production connector or receiver factory is registered.

mod adapter;
mod config;
mod data;
mod error;
mod session;
mod sql;

pub use adapter::{SqlServerAdapter, SqlServerCancellation, SqlServerConnector};
pub use config::{SqlServerConfigError, SqlServerConnectionConfig, SqlServerReceiverConfig};
pub use error::SqlServerError;
pub use sql::{SqlServerParameter, SqlServerStatement};

#[cfg(test)]
mod adapter_tests;

#[cfg(test)]
mod tests;
