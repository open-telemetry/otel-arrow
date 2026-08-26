// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared ingress-admission data types.
//!
//! The vocabulary exchanged by the engine admission binder and components, one
//! concern per file: the metering [`AdmissionDimension`] and its
//! [`AdmissionDimensionSet`], the per-request [`AdmissionContext`], the
//! [`AdmissionDecision`] outcome, and the startup [`AdmissionBindError`].
//!
//! The submodules are private; each type is re-exported here and again from
//! [`super`], so consumers reach them at `admission::<Type>`.

mod bind_error;
mod context;
mod decision;
mod dimension;
pub use bind_error::AdmissionBindError;
pub use context::AdmissionContext;
pub use decision::AdmissionDecision;
pub use dimension::{AdmissionDimension, AdmissionDimensionSet};
