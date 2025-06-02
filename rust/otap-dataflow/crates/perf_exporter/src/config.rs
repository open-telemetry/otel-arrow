// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the perf exporter
//!

/// Defines the settings of the perf exporter such as what to track
#[derive(Debug)]
pub struct Config {
    /// Time duration after which a perf trace is displayed (default = 1000ms).
    timeout: u64,
    self_usage: bool,
    cpu_usage: bool,
    mem_usage: bool,
    disk_usage: bool,
    io_usage: bool,
}

impl Config {
    /// check the timeout interval
    pub fn timeout(&self) -> u64 {
        self.timeout
    }
    /// check if self_usage is enabled
    pub fn self_usage(&self) -> bool {
        self.self_usage
    }
    /// check if cpu_usage is enabled
    pub fn cpu_usage(&self) -> bool {
        self.cpu_usage
    }
    /// check if mem_usage is enabled
    pub fn mem_usage(&self) -> bool {
        self.mem_usage
    }
    /// check if disk_usage is enabled
    pub fn disk_usage(&self) -> bool {
        self.disk_usage
    }
    /// check if io_usage is enabled
    pub fn io_usage(&self) -> bool {
        self.io_usage
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: 1000,
            self_usage: true,
            cpu_usage: true,
            mem_usage: true,
            disk_usage: true,
            io_usage: true,
        }
    }
}
