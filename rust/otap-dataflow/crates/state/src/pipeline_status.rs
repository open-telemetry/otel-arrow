// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Observed pipeline status and aggregation logic per core.

use std::collections::HashMap;
use std::time::SystemTime;
use serde::Serialize;
use crate::core_status::CoreStatus;
use crate::CoreId;
use crate::phase::PipelinePhase;
use crate::store::ts_to_rfc3339;

/// Aggregated, controller-synthesized view for a pipeline across all targeted
/// cores. This is what external APIs will return for `status`.
#[derive(Debug, Serialize, Clone)]
pub struct PipelineStatus {
    /// Coarse phase synthesized from all per-core phases.
    phase: PipelinePhase,

    /// Timestamp of the last phase transition.
    #[serde(serialize_with = "ts_to_rfc3339")]
    phase_since: SystemTime,

    /// Per-core details to aid debugging and aggregation.
    pub(crate) per_core: HashMap<CoreId, CoreStatus>,
}

impl PipelineStatus {
    pub(crate) fn new(now: SystemTime) -> Self {
        Self {
            phase: PipelinePhase::Pending,
            phase_since: now,
            per_core: HashMap::new(),
        }
    }

    /// Returns the current aggregated phase of the pipeline.
    pub fn phase(&self) -> PipelinePhase {
        self.phase
    }

    /// Returns the timestamp corresponding to the last phase transition.
    pub fn phase_since(&self) -> SystemTime {
        self.phase_since
    }

    /// Returns the current per-core status map.
    pub fn per_core(&self) -> &HashMap<CoreId, CoreStatus> {
        &self.per_core
    }
}