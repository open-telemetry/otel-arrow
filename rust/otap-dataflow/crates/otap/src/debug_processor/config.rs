// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the debug processor

use serde::Deserialize;

/// Enum that allows the user to specify how much information they want displayed
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verbosity {
    /// displays the number of received signals + extracts all of the fields in the signal object
    Detailed,
    /// displays the number of received signals + extracts a few of the fields in the signal object such as attributes, trace_id, body
    Normal,
    /// just display number of logs, metrics, traces
    Basic,
}

/// Enum that describes how the output should be handled
pub enum OutputMode {
    /// output the whole batch at once
    Batch,
    /// output per signal
    Signal,
}

/// Defines the settings of the debug processor, controls the level of verbosity the processor outputs
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Config {
    #[serde(default = "default_verbosity")]
    verbosity: Verbosity,
    mode: OutputMode,
}

fn default_verbosity() -> Verbosity {
    Verbosity::Normal
}

impl Config {
    /// Create a new Config object
    #[must_use]
    pub fn new(verbosity: Verbosity) -> Self {
        Self { verbosity }
    }
    /// check the frequency interval
    #[must_use]
    pub const fn verbosity(&self) -> Verbosity {
        self.verbosity
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbosity: default_verbosity(),
        }
    }
}
