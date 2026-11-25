// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Filelog receiver and source primitives.
//!
//! Enable the experimental-filelog feature to register the single-file Linux receiver.

pub mod batch;
pub mod config;
pub mod decoder;
pub mod framer;
pub mod metadata;
pub mod multiline;
pub mod multiline_pattern;

// Worker lifecycle and transport, independent of receiver registration.
#[allow(dead_code)]
pub(crate) mod workers;

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod partial_state;

/// Linux source-file access.
#[cfg(target_os = "linux")]
pub mod source;

/// Bounded Linux scanning and reconciliation.
#[cfg(target_os = "linux")]
pub mod discovery;

// Scheduler composition remains separate from receiver registration.
#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub(crate) mod scheduler;

/// Linux file identity and local ownership admission.
#[cfg(target_os = "linux")]
pub mod identity;

// Internal delivery primitives.
#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub(crate) mod delivery;

#[cfg(target_os = "linux")]
#[cfg_attr(not(feature = "experimental-filelog"), allow(dead_code))]
mod startup;

#[cfg(all(target_os = "linux", feature = "experimental-filelog"))]
mod runtime;
#[cfg(all(target_os = "linux", feature = "experimental-filelog"))]
mod runtime_worker;

#[cfg(all(test, target_os = "linux", feature = "experimental-filelog"))]
mod runtime_tests;
