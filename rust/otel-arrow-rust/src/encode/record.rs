// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains builders for record batches and arrays of encoded OTAP data.

#![allow(clippy::new_without_default)]

pub mod attributes;
pub mod logs;
pub mod metrics;
pub mod traces;

mod array;
