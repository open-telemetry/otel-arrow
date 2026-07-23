// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Resource detection for self-telemetry.
//!
//! A registry mapping detector names to their OpenTelemetry SDK and contrib `ResourceDetector`
//! implementations. Config (`engine.telemetry.detectors`) selects which run. Detected
//! attributes are converted to typed [`AttributeValue`]s and merged.
//!
//! `env` and `service_instance` run by default (see `default_detectors` in config); the
//! detectors that probe the host/OS/process/container/k8s environment are opt-in.

use std::collections::BTreeMap;

use opentelemetry_resource_detectors::{
    ContainerResourceDetector, HostResourceDetector, K8sResourceDetector, OsResourceDetector,
    ProcessResourceDetector, ServiceInstanceIdResourceDetector,
};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::resource::{EnvResourceDetector, ResourceDetector};
use otap_df_config::pipeline::telemetry::{AttributeValue, AttributeValueArray};

/// Selectable detectors, keyed by config name.
const REGISTRY: &[(&str, fn() -> Box<dyn ResourceDetector>)] = &[
    ("env", || Box::new(EnvResourceDetector::default())),
    ("host", || Box::new(HostResourceDetector::default())),
    ("os", || Box::new(OsResourceDetector)),
    ("process", || Box::new(ProcessResourceDetector)),
    ("container", || Box::new(ContainerResourceDetector)),
    ("k8s", || Box::new(K8sResourceDetector)),
    ("service_instance", || {
        Box::new(ServiceInstanceIdResourceDetector)
    }),
];

/// Error raised for an unrecognized detector name in configuration.
#[derive(thiserror::Error, Debug)]
pub enum DetectorError {
    /// A configured detector name has no registered detector.
    #[error(
        "unknown resource detector `{0}` (known: {known})",
        known = known_detector_names().collect::<Vec<_>>().join(", ")
    )]
    Unknown(String),
}

/// Detector names known to the registry.
fn known_detector_names() -> impl Iterator<Item = &'static str> {
    REGISTRY.iter().map(|(name, _)| *name)
}

/// Look up a detector by its config name.
fn detector_by_name(name: &str) -> Result<Box<dyn ResourceDetector>, DetectorError> {
    REGISTRY
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, factory)| factory())
        .ok_or_else(|| DetectorError::Unknown(name.to_owned()))
}

/// Convert an SDK [`opentelemetry::Value`] into a config [`AttributeValue`], preserving
/// the scalar/array type rather than flattening to a string. `opentelemetry::Value` and
/// `opentelemetry::Array` are `#[non_exhaustive]`; unknown variants fall back to `Display`.
fn value_to_attr(value: &opentelemetry::Value) -> AttributeValue {
    use opentelemetry::{Array, Value};
    match value {
        Value::Bool(b) => AttributeValue::Bool(*b),
        Value::I64(i) => AttributeValue::I64(*i),
        Value::F64(f) => AttributeValue::F64(*f),
        Value::String(s) => AttributeValue::String(s.as_str().to_owned()),
        Value::Array(Array::Bool(v)) => AttributeValue::Array(AttributeValueArray::Bool(v.clone())),
        Value::Array(Array::I64(v)) => AttributeValue::Array(AttributeValueArray::I64(v.clone())),
        Value::Array(Array::F64(v)) => AttributeValue::Array(AttributeValueArray::F64(v.clone())),
        Value::Array(Array::String(v)) => AttributeValue::Array(AttributeValueArray::String(
            v.iter().map(|s| s.as_str().to_owned()).collect(),
        )),
        other => AttributeValue::String(other.to_string()),
    }
}

/// Convert a detected SDK [`Resource`] into typed key/value attributes.
fn resource_to_attrs(resource: &Resource) -> Vec<(String, AttributeValue)> {
    resource
        .iter()
        .map(|(k, v)| (k.to_string(), value_to_attr(v)))
        .collect()
}

/// Run the named detectors and merge their attributes into one set (later detector wins
/// on key conflicts).
///
/// # Errors
/// Returns [`DetectorError::Unknown`] if any name has no registered detector.
pub fn detect(names: &[String]) -> Result<Vec<(String, AttributeValue)>, DetectorError> {
    let merged: BTreeMap<String, AttributeValue> = names
        .iter()
        .map(|n| detector_by_name(n))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .flat_map(|d| resource_to_attrs(&d.detect()))
        .collect();

    Ok(merged.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn detect_map(names: &[&str]) -> HashMap<String, AttributeValue> {
        let names: Vec<String> = names.iter().map(|s| s.to_string()).collect();
        detect(&names).unwrap().into_iter().collect()
    }

    /// Scenario: each `opentelemetry::Value` variant is converted via `value_to_attr`.
    /// Guarantees: scalars and arrays map to the matching `AttributeValue` variant rather
    /// than being flattened to a string.
    #[test]
    fn value_to_attr_maps_each_variant() {
        use opentelemetry::{Array, Value};
        assert_eq!(
            value_to_attr(&Value::Bool(true)),
            AttributeValue::Bool(true)
        );
        assert_eq!(value_to_attr(&Value::I64(42)), AttributeValue::I64(42));
        assert_eq!(value_to_attr(&Value::F64(1.5)), AttributeValue::F64(1.5));
        assert_eq!(
            value_to_attr(&Value::String("x".into())),
            AttributeValue::String("x".into())
        );
        assert_eq!(
            value_to_attr(&Value::Array(Array::I64(vec![1, 2, 3]))),
            AttributeValue::Array(AttributeValueArray::I64(vec![1, 2, 3]))
        );
    }

    /// Scenario: `detect` is given a name with no registered detector.
    /// Guarantees: it returns `DetectorError::Unknown` carrying the missing name.
    #[test]
    fn unknown_detector_names_the_culprit() {
        assert!(matches!(
            detect(&["bogus".to_string()]),
            Err(DetectorError::Unknown(name)) if name == "bogus"
        ));
    }

    /// Scenario: `detect` is called with an empty detector list.
    /// Guarantees: no attributes are produced.
    #[test]
    fn empty_list_detects_nothing() {
        assert!(detect(&[]).unwrap().is_empty());
    }

    /// Scenario: every name in the registry is run through `detect`.
    /// Guarantees: each row's factory builds and its detector runs without error.
    #[test]
    fn every_registered_detector_resolves_and_runs() {
        for name in known_detector_names() {
            let _ = detect(&[name.to_string()]).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }

    /// Scenario: the `env` detector runs with `OTEL_RESOURCE_ATTRIBUTES` set.
    /// Guarantees: each `key=value` pair is parsed into a string `AttributeValue`.
    #[test]
    fn env_detector_parses_otel_resource_attributes() {
        temp_env::with_var("OTEL_RESOURCE_ATTRIBUTES", Some("foo=bar,baz=qux"), || {
            let attrs = detect_map(&["env"]);
            assert_eq!(
                attrs.get("foo"),
                Some(&AttributeValue::String("bar".into()))
            );
            assert_eq!(
                attrs.get("baz"),
                Some(&AttributeValue::String("qux".into()))
            );
        });
    }

    /// Scenario: the `service_instance` detector runs.
    /// Guarantees: it emits `service.instance.id` as a 36-char hyphenated UUID string.
    #[test]
    fn service_instance_detector_emits_uuid() {
        let attrs = detect_map(&["service_instance"]);
        match attrs.get("service.instance.id") {
            Some(AttributeValue::String(id)) => {
                assert_eq!(id.len(), 36, "expected valid UUID, got {id:?}");
            }
            other => panic!("expected a string service.instance.id, got {other:?}"),
        }
    }

    /// Scenario: a `Resource` holding a typed (I64) attribute is converted.
    /// Guarantees: `resource_to_attrs` preserves the I64 type instead of flattening to a string.
    #[test]
    fn resource_to_attrs_preserves_typed_values() {
        let resource = Resource::builder_empty()
            .with_attribute(opentelemetry::KeyValue::new("process.pid", 4321_i64))
            .build();
        assert_eq!(
            resource_to_attrs(&resource),
            vec![("process.pid".to_string(), AttributeValue::I64(4321))]
        );
    }

    /// Scenario: two detectors emit the same key, with `env` listed last.
    /// Guarantees: the later detector's value wins the merge.
    #[test]
    fn later_detector_wins_on_key_conflict() {
        temp_env::with_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            Some("service.instance.id=from-env"),
            || {
                let attrs = detect_map(&["service_instance", "env"]);
                assert_eq!(
                    attrs.get("service.instance.id"),
                    Some(&AttributeValue::String("from-env".into()))
                );
            },
        );
    }
}
