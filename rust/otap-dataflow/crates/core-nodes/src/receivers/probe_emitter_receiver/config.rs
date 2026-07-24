// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the probe-emitter receiver.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default emission interval (1 probe per second).
pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(1);

/// Well-known log body value that backends can use for fast O(1) probe
/// detection. Starts with `_` so a single-byte prefix check short-circuits
/// for every non-probe log.
pub const PROBE_BODY: &str = "_OTAP_PROBE";

/// Reserved attribute key carrying the probe's unique id.
pub const PROBE_ID_ATTR: &str = "_otap_internal.probe.id";

/// Reserved attribute key carrying the probe's source timestamp (Unix nanos).
pub const PROBE_EMITTED_AT_ATTR: &str = "_otap_internal.probe.emitted_at_unix_nanos";

/// Reserved attribute key carrying the probe's source name.
pub const PROBE_SOURCE_ATTR: &str = "_otap_internal.probe.source";

/// Configuration for the `probe_emitter` receiver.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// How often to emit a single-row probe log batch.
    /// Format: humantime (e.g., "500ms", "1s", "5s"). Defaults to `1s`.
    #[serde(default = "default_interval", with = "humantime_serde")]
    pub interval: Duration,

    /// Logical identifier of this probe source, propagated as an attribute so
    /// the sink can group latency by source. Defaults to `"default"`.
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_interval() -> Duration {
    DEFAULT_INTERVAL
}

fn default_source() -> String {
    "default".to_owned()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval: DEFAULT_INTERVAL,
            source: default_source(),
        }
    }
}
