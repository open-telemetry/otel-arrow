// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Command runners and shared helpers for the non-interactive CLI flows.

pub(crate) mod catalog;
pub(crate) mod completions;
pub(crate) mod config;
pub(crate) mod engine;
pub(crate) mod fetch;
pub(crate) mod filters;
pub(crate) mod groups;
pub(crate) mod output;
pub(crate) mod pipelines;
pub(crate) mod schemas;
pub(crate) mod telemetry;
pub(crate) mod watch;

#[cfg(test)]
mod tests;
