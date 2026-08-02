// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine-owned ingress admission contracts and pressure-aware rate gate.

mod binder;
mod bucket;
mod clock;
mod gate;
pub(crate) mod metrics;
mod models;

pub use binder::{AdmissionBinder, AdmissionBindingProvenance};
pub use gate::{LocalAdmissionGate, SharedAdmissionGate};
pub use models::{
    AdmissionBindError, AdmissionContext, AdmissionDecision, AdmissionDimension,
    AdmissionDimensionSet,
};
