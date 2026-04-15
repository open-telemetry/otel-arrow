// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Contrib nodes (receiver, exporter, processor).

#[cfg(any(target_os = "linux", target_os = "windows"))]
mod collection;

/// Exporter implementations for contrib nodes.
pub mod exporters;

/// Receiver implementations for contrib nodes.
pub mod receivers;

/// Processor implementations for contrib nodes.
pub mod processors;
