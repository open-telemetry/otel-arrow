// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Query engine language parsers for OTAP dataflow.
//!
//! This crate centralizes OTel-specific language parsers used by the
//! transform processor and other query-engine consumers.

pub mod opl;
pub mod ottl;
