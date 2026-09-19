// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Filelog source primitives, independent of receiver configuration and runtime.
//!
//! This module does not register a receiver.

/// Linux source-file access.
#[cfg(target_os = "linux")]
pub mod source;
