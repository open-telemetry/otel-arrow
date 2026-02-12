// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Attribute validation helpers.
//!
//! Given `&[OtlpProtoMessage]`, verify that certain attribute keys (or key/value
//! pairs) appear for each attribute list or do **not** appear within configured domains (resource,
//! scope, or the signal itself).

use otap_df_pdata::proto::opentelemetry::common::v1::{
    any_value::Value as ProtoAnyValue, AnyValue as ProtoValue, KeyValue as ProtoKeyValue,
};
use otap_df_pdata::proto::OtlpProtoMessage;
use serde::{Deserialize, Serialize};

use super::{AnyValue, KeyValue};

/// Domains where attribute assertions can be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttributeDomain {
    /// Resource attributes.
    Resource,
    /// Scope / instrumentation scope attributes.
    Scope,
    /// Signal-specific attributes (logs, spans, data points, etc.).
    Signal,
}

fn default_domains() -> Vec<AttributeDomain> {
    vec![AttributeDomain::Signal]
}

/// Declarative attribute validation.
///
/// - `require_keys`: every attribute list in the selected domains must contain
///   these keys (value is ignored).
/// - `require`: required key/value pairs that must appear in each attribute list.
/// - `forbid_keys`: keys that must **not** appear in each attribute list.
/// - `forbid`: key/value pairs that must **not** appear in each attribute list.
/// - `domains`: which domains to inspect; defaults to [`AttributeDomain::Signal`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AttributeCheck {
    /// Domains to inspect; defaults to only the signal domain.
    #[serde(default = "default_domains")]
    pub domains: Vec<AttributeDomain>,
    /// Keys that must exist in every inspected attribute list (value ignored).
    #[serde(default)]
    pub require_keys: Vec<String>,
    /// Key/value pairs that must exist in every inspected attribute list.
    #[serde(default)]
    pub require: Vec<KeyValue>,
    /// Keys that must not appear in any inspected attribute list.
    #[serde(default)]
    pub forbid_keys: Vec<String>,
}

impl AttributeCheck {
    /// Run the configured checks against a list of OTLP proto messages.
    pub fn check(&self, messages: &[OtlpProtoMessage]) -> bool {
        if messages.is_empty() {
            return false;
        }

        // iter through messages and call check_attributes immediately return if check_attributes returns false
    }

    /// takes a OtlpProtoMessage and extracts teh attributes then iterates through the Vec<[ProtoKeyValue]> checking if the attribute lists are valid
    /// that is if self.forbid_keys doesn't show up in any of the keys, and that require and require_keys are present in the lists
    pub fn check_attributes()


}

/// extract attributes will get the signal type and extract the attributes from that signal
fn extract_attributes() -> Vec<[ProtoKeyValue]>

/// extract span attributes will extract attribute lists from TracesData based on domain specified
fn extract_span_attributes() -> Vec<[ProtoKeyValue]>

/// extract log attributes will extract attribute lists from LogsData based on domain specified
fn extract_log_attributes() -> Vec<[ProtoKeyValue]>

/// extract metric attributes will extract attribute lists from MetricsData based on domain specified
fn extract_metrics_attributes() -> Vec<[ProtoKeyValue]>