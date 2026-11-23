// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit and conformance tests for `PolicyEngine` (`open-telemetry/otel-arrow#4726`).

#![allow(clippy::unwrap_used)]

mod bitset;
mod columnar;
mod engine;
mod helpers;
mod model;
mod otap_ops;
mod transform;
