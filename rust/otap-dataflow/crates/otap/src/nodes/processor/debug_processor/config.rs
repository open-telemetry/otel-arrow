// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the debug processor

use super::filter::FilterRules;
use super::output::OutputMode;
use super::sampling::SamplingConfig;
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
pub enum DisplayMode {
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
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_verbosity")]
    verbosity: Verbosity,
    #[serde(default = "default_display_mode")]
    mode: DisplayMode,
    #[serde(default = "default_active_signal")]
    signals: HashSet<SignalActive>,
    #[serde(default = "default_output_mode")]
    output: OutputMode,
    #[serde(default = "default_filters")]
    filters: Vec<FilterRules>,
    #[serde(default = "default_sampling")]
    sampling: SamplingConfig,
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

fn default_display_mode() -> DisplayMode {
    DisplayMode::Batch
}
fn default_filters() -> Vec<FilterRules> {
    Vec::new()
}

fn default_output_mode() -> OutputMode {
    OutputMode::Console
}

fn default_sampling() -> SamplingConfig {
    SamplingConfig::NoSampling
}

impl Config {
    /// Create a new Config object
    #[must_use]
    pub fn new(
        verbosity: Verbosity,
        mode: DisplayMode,
        signals: HashSet<SignalActive>,
        output: OutputMode,
        filters: Vec<FilterRules>,
        sampling: SamplingConfig,
    ) -> Self {
        Self {
            verbosity,
            mode,
            signals,
            output,
            filters,
            sampling,
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

    /// get display mode
    #[must_use]
    pub const fn mode(&self) -> DisplayMode {
        self.mode
    }

    /// get output mode
    #[must_use]
    pub fn output(&self) -> OutputMode {
        self.output.clone()
    }
    #[must_use]
    pub const fn filters(&self) -> &Vec<FilterRules> {
        &self.filters
    }

    #[must_use]
    pub const fn sampling(&self) -> SamplingConfig {
        self.sampling
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbosity: default_verbosity(),
            mode: default_display_mode(),
            signals: default_active_signal(),
            output: default_output_mode(),
            filters: default_filters(),
            sampling: default_sampling(),
        }
    }
}
