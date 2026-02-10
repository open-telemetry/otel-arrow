// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline and engine settings.
//!
//! This module contains configuration types for pipeline-internal telemetry
//! and logging settings that control the behavior of the dataflow engine.

pub mod telemetry;

pub use telemetry::TelemetrySettings;
