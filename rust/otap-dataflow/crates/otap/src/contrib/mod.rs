// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Contributed exporters and processors
//!
//! This module contains components that are not fully supported but are related
//! to project goals mentioned in the [OTel-Arrow Project Phases](../../../../../../docs/project-phases.md).
//!
//! These components were previously located in the `experimental` module and have
//! been reorganized into a structure that matches the core components.

pub mod exporter;
pub mod processor;
pub mod receiver;
