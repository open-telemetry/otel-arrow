// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared, cross-platform Windows event semantics.
//!
//! The `model` submodule defines an owned representation of a decoded Windows
//! event that is independent of transport and output encoding. The `parse`
//! submodule builds that representation from a bounded XML document; native
//! Windows Event Log APIs can populate the same structure without XML. Attribute
//! naming, severity mapping, and OTAP encoding are caller concerns and live with
//! the consuming receiver.

pub mod model;
pub mod parse;
