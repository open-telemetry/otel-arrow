// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Constants used by OpAMP Controller Extension

/// values used for component health status
#[allow(missing_docs)]
pub mod health_status {
    pub const RUNNING: &str = "running";
    pub const STARTING: &str = "starting";
    pub const STOPPED: &str = "stopped";
    pub const STOPPING: &str = "stopping";
    pub const FAILED: &str = "failed";
    pub const DEGRADED: &str = "degraded";
}
