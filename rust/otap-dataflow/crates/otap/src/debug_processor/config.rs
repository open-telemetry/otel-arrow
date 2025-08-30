// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the debug processor

use serde::Deserialize;

/// Enum that allows the user to specify how much information they want displayed
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verbosity {
    /// display the most detailed information available
    Detailed,
    /// display the basic amount of information available and some detail about each request
    Normal,
    /// just display number of logs, metrics, traces, profiles received with some additional detail about samples/datapoints
    Basic,
}

/// Defines the settings of the debug processor, controls the level of verbosity the processor outputs
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Config {
    #[serde(default = "default_verbosity")]
    verbosity: Verbosity,
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
