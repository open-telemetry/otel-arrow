// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Filelog configuration and source primitives, independent of receiver runtime.
//!
//! This module does not register a receiver.

pub mod config;
pub mod decoder;
pub mod framer;
pub mod multiline;
pub mod multiline_pattern;

/// Linux source-file access.
#[cfg(target_os = "linux")]
pub mod source;

/// Bounded Linux scanning and reconciliation without receiver wiring.
#[cfg(target_os = "linux")]
pub mod discovery;

// No production admission constructor until Task 12 supplies ReaderSeed.
#[cfg(target_os = "linux")]
#[allow(dead_code)]
mod scheduler;
