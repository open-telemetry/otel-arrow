// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Filelog source primitives, independent of receiver configuration and runtime.
//!
//! This module does not register a receiver.

pub mod decoder;
pub mod framer;
pub mod multiline;
pub mod multiline_pattern;

/// Linux source-file access.
#[cfg(target_os = "linux")]
pub mod source;
