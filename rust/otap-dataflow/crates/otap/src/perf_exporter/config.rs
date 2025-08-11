// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the perf exporter
//!

use serde::Deserialize;

/// Defines the settings of the perf exporter such as what to track
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Time duration after which a perf trace is displayed (default = 1000ms).
    #[serde(default = "default_frequency")]
    frequency: u64,

    // smoothing factor for the exponential moving average for the latency.
    #[serde(default = "default_smoothing_factor")]
    smoothing_factor: f32,

    #[serde(default = "default_self_usage")]
    self_usage: bool,

    #[serde(default = "default_cpu_usage")]
    cpu_usage: bool,

    #[serde(default = "default_mem_usage")]
    mem_usage: bool,

    #[serde(default = "default_disk_usage")]
    disk_usage: bool,

    #[serde(default = "default_io_usage")]
    io_usage: bool,
}

fn default_frequency() -> u64 {
    1000
}

fn default_self_usage() -> bool {
    true
}

fn default_cpu_usage() -> bool {
    true
}

fn default_mem_usage() -> bool {
    true
}
fn default_disk_usage() -> bool {
    true
}
fn default_io_usage() -> bool {
    true
}

fn default_smoothing_factor() -> f32 {
    0.3
}

impl Config {
    /// Create a new Config object
    #[must_use]
    pub fn new(
        frequency: u64,
        smoothing_factor: f32,
        self_usage: bool,
        cpu_usage: bool,
        mem_usage: bool,
        disk_usage: bool,
        io_usage: bool,
    ) -> Self {
        Self {
            frequency,
            smoothing_factor,
            self_usage,
            cpu_usage,
            mem_usage,
            disk_usage,
            io_usage,
        }
    }
    /// check the frequency interval
    #[must_use]
    pub const fn frequency(&self) -> u64 {
        self.frequency
    }
    /// check if self_usage is enabled
    #[must_use]
    pub const fn self_usage(&self) -> bool {
        self.self_usage
    }
    /// check if cpu_usage is enabled
    #[must_use]
    pub const fn cpu_usage(&self) -> bool {
        self.cpu_usage
    }
    /// check if mem_usage is enabled
    #[must_use]
    pub const fn mem_usage(&self) -> bool {
        self.mem_usage
    }
    /// check if disk_usage is enabled
    #[must_use]
    pub const fn disk_usage(&self) -> bool {
        self.disk_usage
    }
    /// check if io_usage is enabled
    #[must_use]
    pub const fn io_usage(&self) -> bool {
        self.io_usage
    }
    /// get the smoothing factor
    #[must_use]
    pub const fn smoothing_factor(&self) -> f32 {
        self.smoothing_factor
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: default_frequency(),
            smoothing_factor: default_smoothing_factor(),
            self_usage: default_self_usage(),
            cpu_usage: default_cpu_usage(),
            mem_usage: default_mem_usage(),
            disk_usage: default_disk_usage(),
            io_usage: default_io_usage(),
        }
    }
}
