// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared operation options for long-running admin actions.

use serde::{Deserialize, Serialize};

/// Generic options for long-running admin operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationOptions {
    /// Whether to wait for completion.
    #[serde(default)]
    pub wait: bool,
    /// Wait timeout in seconds.
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for OperationOptions {
    fn default() -> Self {
        Self {
            wait: false,
            timeout_secs: default_timeout_secs(),
        }
    }
}

const fn default_timeout_secs() -> u64 {
    60
}

impl OperationOptions {
    /// Converts this request into URL query pairs.
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("wait", self.wait.to_string()),
            ("timeout_secs", self.timeout_secs.to_string()),
        ]
    }
}
