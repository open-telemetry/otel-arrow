// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experimental exporters
//!
//! This module contains exporters that are not fully supported
//! but are related to project goals mentioned
//! in [OTel-Arrow Project Phases](../../../../../../docs/project-phases.md).

/// Geneva exporter for Microsoft telemetry backend
#[cfg(feature = "geneva-exporter")]
pub mod geneva_exporter;
