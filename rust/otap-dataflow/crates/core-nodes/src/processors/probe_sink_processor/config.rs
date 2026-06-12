// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the probe-sink processor.

use serde::{Deserialize, Serialize};

/// Configuration for the `probe_sink` processor.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// When `true` (default), probe log records are dropped after their
    /// latency is recorded so they never reach downstream exporters. When
    /// `false`, probes are passed through unchanged (useful for debugging
    /// or for inspecting probe records in test sinks).
    #[serde(default = "default_drop_probes")]
    pub drop_probes: bool,
}

fn default_drop_probes() -> bool {
    true
}
