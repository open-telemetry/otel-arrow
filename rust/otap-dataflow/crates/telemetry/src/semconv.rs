// SPDX-License-Identifier: Apache-2.0

//! Semantic convention file.

use crate::descriptor::{AttributeField, MetricsDescriptor};
use serde::Serialize;
use std::borrow::Cow;

/// A semantic convention registry containing attributes and metrics definitions.
#[derive(Serialize)]
pub struct SemConvRegistry {
    /// The semantic convention version (version 2).
    pub version: Cow<'static, str>,

    /// Contains definitions of semantic attributes which may be applicable to all OpenTelemetry
    /// signals.
    pub attributes: Vec<&'static AttributeField>,

    /// Contains definitions of metric sets.
    pub metric_sets: Vec<&'static MetricsDescriptor>,
}

impl SemConvRegistry {
    /// Creates a new SemConvRegistry with the specified version.
    pub fn new(version: Cow<'static, str>) -> Self {
        Self {
            version,
            attributes: Vec::new(),
            metric_sets: Vec::new(),
        }
    }

    /// Returns a YAML representation of the semantic convention registry.
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(self).expect("Failed to serialize SemConvRegistry to YAML")
    }
}
