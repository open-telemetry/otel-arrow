// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration parsing, validation, and defaults tests, split by concern.

use super::*;
use serde_json::json;

mod compatibility;
mod construction_and_configuration;
mod dlq;
mod operational;
mod rebalancing;
mod routing;
