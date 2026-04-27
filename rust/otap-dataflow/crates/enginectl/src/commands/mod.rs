// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Command runners and shared helpers for the non-interactive CLI flows.
//!
//! This module is the dispatch boundary between parsed CLI arguments and the
//! admin SDK operations that implement `dfctl` commands. Submodules keep each
//! resource area isolated while sharing common output, filtering, fetch, and
//! watch helpers so command behavior stays consistent across human, script, and
//! agent-facing modes.

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
