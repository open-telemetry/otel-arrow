// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Protocol-neutral Azure Key Vault acquisition shared by credential providers.

pub mod config;
pub mod error;
mod source;

pub use source::Source;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod testing;
