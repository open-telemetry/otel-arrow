// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the debug processor

use serde::Deserialize;
use std::collections::HashSet;

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
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    /// output the whole batch at once
    Batch,
    /// output per signal
    Signal,
}
/// Enum that defines which signals to debug for
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Hash, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalActive {
    Metrics,
    Logs,
    Spans,
}

/// Defines the settings of the debug processor, controls the level of verbosity the processor outputs
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_verbosity")]
    verbosity: Verbosity,
    #[serde(default = "default_output_mode")]
    mode: OutputMode,
    #[serde(default = "default_active_signal")]
    signals: HashSet<SignalActive>,
}

fn default_verbosity() -> Verbosity {
    Verbosity::Normal
}

fn default_active_signal() -> HashSet<SignalActive> {
    HashSet::from([
        SignalActive::Metrics,
        SignalActive::Logs,
        SignalActive::Spans,
    ])
}

fn default_output_mode() -> OutputMode {
    OutputMode::Batch
}

impl Config {
    /// Create a new Config object
    #[must_use]
    pub fn new(verbosity: Verbosity, mode: OutputMode, signals: HashSet<SignalActive>) -> Self {
        Self {
            verbosity,
            mode,
            signals,
        }
    }
    /// get the verbosity level
    #[must_use]
    pub const fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    /// get a set of active signals
    #[must_use]
    pub const fn signals(&self) -> &HashSet<SignalActive> {
        &self.signals
    }

    #[must_use]
    pub const fn mode(&self) -> OutputMode {
        self.mode
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbosity: default_verbosity(),
            mode: default_output_mode(),
            signals: default_active_signal(),
        }
    }
}
