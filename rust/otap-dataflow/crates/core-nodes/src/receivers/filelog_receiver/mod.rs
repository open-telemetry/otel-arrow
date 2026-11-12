// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Filelog configuration and source primitives, independent of receiver runtime.
//!
//! This module does not register a receiver.

pub mod config;
pub mod decoder;
pub mod framer;
pub mod metadata;
pub mod multiline;
pub mod multiline_pattern;

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod partial_state;

/// Linux source-file access.
#[cfg(target_os = "linux")]
pub mod source;

/// Bounded Linux scanning and reconciliation without receiver wiring.
#[cfg(target_os = "linux")]
pub mod discovery;

// Scheduler composition remains separate from receiver registration.
#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub(crate) mod scheduler;

/// Linux file identity and local ownership admission.
#[cfg(target_os = "linux")]
pub mod identity;
