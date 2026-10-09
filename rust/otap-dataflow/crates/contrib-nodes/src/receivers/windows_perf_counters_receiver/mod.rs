// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Windows performance-counter receiver.

/// Configuration parsing, validation, and exact-path normalization.
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "consumed by the receiver runtime in a follow-up")
)]
mod config;

#[cfg_attr(
    not(test),
    allow(dead_code, reason = "consumed by PDH collection in a follow-up")
)]
mod model;

#[cfg_attr(
    not(test),
    allow(dead_code, reason = "consumed by the receiver runtime in a follow-up")
)]
mod otap_builder;
