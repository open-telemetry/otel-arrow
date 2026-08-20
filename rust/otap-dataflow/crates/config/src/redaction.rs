// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Type-owned secret redaction for raw configuration snapshots.

use linkme::distributed_slice;
use schemars::JsonSchema;
use secrecy::{ExposeSecret, SecretString};
use serde::de::{DeserializeOwned, Error as _};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Placeholder emitted for type-owned secrets in config snapshots.
pub const REDACTED_VALUE: &str = "[REDACTED]";

const PRIVATE_MARKER_A: &str = "\u{001e}otap-secret-marker-a\u{001f}";
const PRIVATE_MARKER_B: &str = "\u{001e}otap-secret-marker-b\u{001f}";

thread_local! {
    static TRACKED_SECRET_COUNT: Cell<Option<usize>> = const { Cell::new(None) };
    static SERIALIZED_SECRET_MARKER: Cell<Option<&'static str>> = const { Cell::new(None) };
}

/// A string value that remains cleartext in memory but always serializes as
/// redacted.
///
/// This wrapper is for config values, not map keys. Secret-bearing config types
/// must use symmetric serde field mappings and register an exact
/// [`ConfigRedactor`].
#[derive(Debug, JsonSchema)]
pub struct RedactedString(#[schemars(with = "String")] SecretString);

impl Clone for RedactedString {
    fn clone(&self) -> Self {
        TRACKED_SECRET_COUNT.with(|count| {
            if let Some(current) = count.get() {
                count.set(Some(current.saturating_add(1)));
            }
        });
        Self(self.0.clone())
    }
}

impl RedactedString {
    /// Returns the cleartext value for an explicit runtime use.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl Drop for RedactedString {
    fn drop(&mut self) {
        TRACKED_SECRET_COUNT.with(|count| {
            if let Some(current) = count.get() {
                count.set(Some(current.saturating_sub(1)));
            }
        });
    }
}

impl<'de> Deserialize<'de> for RedactedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == REDACTED_VALUE {
            return Err(D::Error::custom(
                "the redaction placeholder cannot be used as a secret value; provide the secret again",
            ));
        }
        TRACKED_SECRET_COUNT.with(|count| {
            if let Some(current) = count.get() {
                count.set(Some(current.saturating_add(1)));
            }
        });
        Ok(Self(SecretString::from(value)))
    }
}

impl Serialize for RedactedString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let marker = SERIALIZED_SECRET_MARKER.with(Cell::get);
        match marker {
            Some(prefix) => serializer.serialize_str(&format!("{prefix}{}", self.expose())),
            None => serializer.serialize_str(REDACTED_VALUE),
        }
    }
}

/// A sanitized failure from snapshot redaction.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RedactionError {
    /// The raw config no longer deserializes into its registered type.
    #[error("registered config could not be deserialized for snapshot redaction")]
    Deserialization,
    /// The typed config was not deterministic across marker serializations.
    #[error("registered config could not be serialized deterministically for snapshot redaction")]
    Serialization,
    /// Deserialization and serialization observed different numbers of secrets.
    #[error("registered config did not serialize every deserialized secret")]
    SecretCountMismatch,
    /// A typed secret path did not exist in the original raw config.
    #[error("registered config secret paths did not match the raw config shape")]
    ShapeMismatch,
    /// More than one component registered the same exact type URN.
    #[error("multiple snapshot redactors registered for component type `{component_type}`")]
    DuplicateRegistration {
        /// Exact component type URN with conflicting registrations.
        component_type: String,
    },
    /// Structural location added while a snapshot tree propagates a failure.
    #[error("{context}: {source}")]
    Context {
        /// Group, pipeline, node, or extension location.
        context: String,
        /// Sanitized underlying redaction failure.
        #[source]
        source: Box<RedactionError>,
    },
}

impl RedactionError {
    /// Adds a non-secret structural location to this failure.
    #[must_use]
    pub fn at(self, context: impl Into<String>) -> Self {
        Self::Context {
            context: context.into(),
            source: Box::new(self),
        }
    }
}

/// Function pointer registered by a component that owns a typed config.
pub type ConfigRedactorFn = fn(&mut Value) -> Result<(), RedactionError>;

/// Type-owned snapshot redactor registration.
///
/// The registered type must use deterministic, symmetric serde mappings:
/// secrets must serialize at the same raw paths from which they deserialize.
/// `RedactedString` fields must originate in the raw config and must not be
/// omitted by serialization. Violations fail closed instead of returning a
/// partially redacted snapshot.
#[derive(Clone, Copy)]
pub struct ConfigRedactor {
    /// Exact component type URN.
    pub component_type: &'static str,
    /// Redacts the component's raw config without changing its shape.
    pub redact: ConfigRedactorFn,
}

impl ConfigRedactor {
    /// Creates a static component redactor registration.
    #[must_use]
    pub const fn new(component_type: &'static str, redact: ConfigRedactorFn) -> Self {
        Self {
            component_type,
            redact,
        }
    }
}

/// Statically linked component config redactors.
#[allow(unsafe_code)]
#[distributed_slice]
pub static CONFIG_REDACTORS: [ConfigRedactor] = [..];

static CONFIG_REDACTOR_INDEX: OnceLock<HashMap<&'static str, RedactorEntry>> = OnceLock::new();

#[derive(Clone, Copy)]
enum RedactorEntry {
    Redactor(ConfigRedactorFn),
    Duplicate,
}

/// Applies the exact registered redactor, or returns `None` when the component
/// has no type-owned registration.
pub fn redact_registered_config(
    component_type: &str,
    config: &mut Value,
) -> Result<bool, RedactionError> {
    let index = CONFIG_REDACTOR_INDEX.get_or_init(|| build_redactor_index(&CONFIG_REDACTORS));
    let Some(entry) = index.get(component_type) else {
        return Ok(false);
    };
    let redact = match entry {
        RedactorEntry::Redactor(redact) => redact,
        RedactorEntry::Duplicate => {
            return Err(RedactionError::DuplicateRegistration {
                component_type: component_type.to_owned(),
            });
        }
    };
    redact(config).map_err(|error| error.at(format!("component type `{component_type}`")))?;
    Ok(true)
}

fn build_redactor_index(registrations: &[ConfigRedactor]) -> HashMap<&'static str, RedactorEntry> {
    let mut index = HashMap::with_capacity(registrations.len());
    for registration in registrations {
        match index.entry(registration.component_type) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                let _ = entry.insert(RedactorEntry::Redactor(registration.redact));
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let _ = entry.insert(RedactorEntry::Duplicate);
            }
        }
    }
    index
}

/// Redacts every [`RedactedString`] in typed config `T` while preserving the
/// exact keys, values, and omissions in the original raw JSON.
pub fn redact_typed_config<T>(config: &Value) -> Result<Value, RedactionError>
where
    T: DeserializeOwned + Serialize,
{
    let mut redacted = config.clone();
    redact_typed_config_in_place::<T>(&mut redacted)?;
    Ok(redacted)
}

/// Redacts every [`RedactedString`] in typed config `T` in place.
pub fn redact_typed_config_in_place<T>(config: &mut Value) -> Result<(), RedactionError>
where
    T: DeserializeOwned + Serialize,
{
    let (typed, deserialized_secrets) = track_deserialized_secrets(|| {
        T::deserialize(&*config).map_err(|_| RedactionError::Deserialization)
    })?;
    let first = serialize_with_marker(&typed, PRIVATE_MARKER_A)?;
    let second = serialize_with_marker(&typed, PRIVATE_MARKER_B)?;
    let counts = compare_and_apply(Some(config), &first, &second)?;

    if counts.serialized != deserialized_secrets {
        return Err(RedactionError::SecretCountMismatch);
    }
    if counts.applied != counts.serialized {
        return Err(RedactionError::ShapeMismatch);
    }
    Ok(())
}

fn track_deserialized_secrets<T>(
    operation: impl FnOnce() -> Result<T, RedactionError>,
) -> Result<(T, usize), RedactionError> {
    struct TrackingGuard(Option<usize>);

    impl Drop for TrackingGuard {
        fn drop(&mut self) {
            TRACKED_SECRET_COUNT.with(|count| count.set(self.0));
        }
    }

    let previous = TRACKED_SECRET_COUNT.with(|count| count.replace(Some(0)));
    let guard = TrackingGuard(previous);
    let result = operation();
    let count = TRACKED_SECRET_COUNT.with(|tracked| tracked.get().unwrap_or(0));
    drop(guard);
    result.map(|value| (value, count))
}

fn serialize_with_marker<T>(value: &T, marker: &'static str) -> Result<Value, RedactionError>
where
    T: Serialize,
{
    struct MarkerGuard(Option<&'static str>);

    impl Drop for MarkerGuard {
        fn drop(&mut self) {
            SERIALIZED_SECRET_MARKER.with(|current| current.set(self.0));
        }
    }

    let previous = SERIALIZED_SECRET_MARKER.with(|current| current.replace(Some(marker)));
    let guard = MarkerGuard(previous);
    let serialized = serde_json::to_value(value).map_err(|_| RedactionError::Serialization);
    drop(guard);
    serialized
}

#[derive(Default)]
struct SecretCounts {
    serialized: usize,
    applied: usize,
}

impl SecretCounts {
    fn add(&mut self, other: Self) {
        self.serialized = self.serialized.saturating_add(other.serialized);
        self.applied = self.applied.saturating_add(other.applied);
    }
}

fn compare_and_apply(
    raw: Option<&mut Value>,
    first: &Value,
    second: &Value,
) -> Result<SecretCounts, RedactionError> {
    if let Some(secret) = private_secret_value(first, second) {
        let applied = if let Some(raw) = raw {
            match raw {
                Value::String(value) if value == secret => {
                    *raw = Value::String(REDACTED_VALUE.to_owned());
                    1
                }
                _ => return Err(RedactionError::ShapeMismatch),
            }
        } else {
            0
        };
        return Ok(SecretCounts {
            serialized: 1,
            applied,
        });
    }

    match (first, second) {
        (Value::Array(first_values), Value::Array(second_values)) => {
            if first_values.len() != second_values.len() {
                return Err(RedactionError::Serialization);
            }
            let mut raw_values = match raw {
                Some(Value::Array(values)) => Some(values),
                _ => None,
            };
            let mut counts = SecretCounts::default();
            for (index, (first_value, second_value)) in
                first_values.iter().zip(second_values).enumerate()
            {
                let raw_value = raw_values
                    .as_deref_mut()
                    .and_then(|values| values.get_mut(index));
                counts.add(compare_and_apply(raw_value, first_value, second_value)?);
            }
            Ok(counts)
        }
        (Value::Object(first_values), Value::Object(second_values)) => {
            if first_values.len() != second_values.len()
                || first_values
                    .keys()
                    .any(|key| !second_values.contains_key(key))
            {
                return Err(RedactionError::Serialization);
            }
            let mut raw_values = match raw {
                Some(Value::Object(values)) => Some(values),
                _ => None,
            };
            let mut counts = SecretCounts::default();
            for (key, first_value) in first_values {
                let second_value = second_values
                    .get(key)
                    .ok_or(RedactionError::Serialization)?;
                let raw_value = raw_values
                    .as_deref_mut()
                    .and_then(|values| values.get_mut(key));
                counts.add(compare_and_apply(raw_value, first_value, second_value)?);
            }
            Ok(counts)
        }
        (a, b) if a == b => Ok(SecretCounts::default()),
        _ => Err(RedactionError::Serialization),
    }
}

fn private_secret_value<'a>(first: &'a Value, second: &'a Value) -> Option<&'a str> {
    let first = first.as_str()?.strip_prefix(PRIVATE_MARKER_A)?;
    let second = second.as_str()?.strip_prefix(PRIVATE_MARKER_B)?;
    (first == second).then_some(first)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Deserialize, Serialize)]
    struct NestedConfig {
        token: RedactedString,
        label: String,
    }

    #[derive(Deserialize, Serialize)]
    struct TestConfig {
        password: RedactedString,
        nested: Vec<NestedConfig>,
        tokens: HashMap<String, RedactedString>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        omitted: Option<String>,
        literal_marker: String,
    }

    /// Scenario: a cleartext value is loaded into a `RedactedString`.
    /// Guarantees: explicit access returns cleartext while Debug and JSON
    /// serialization never expose it.
    #[test]
    fn redacted_string_exposes_only_explicitly() {
        let secret: RedactedString =
            serde_json::from_str("\"cleartext-value\"").expect("secret should deserialize");

        assert_eq!(secret.expose(), "cleartext-value");
        assert!(!format!("{secret:?}").contains("cleartext-value"));
        assert_eq!(
            serde_json::to_value(&secret).expect("secret should serialize"),
            REDACTED_VALUE
        );
    }

    /// Scenario: a client submits a snapshot's redaction marker as a secret.
    /// Guarantees: deserialization rejects the display-only placeholder before
    /// a runtime can authenticate with the marker as if it were cleartext.
    #[test]
    fn redacted_string_rejects_snapshot_placeholder() {
        let error =
            serde_json::from_value::<RedactedString>(Value::String(REDACTED_VALUE.to_owned()))
                .expect_err("snapshot placeholder must not become a runtime secret");

        assert!(error.to_string().contains("provide the secret again"));
    }

    /// Scenario: a typed config contains required, nested, sequence, and map
    /// secrets plus omitted and ordinary values.
    /// Guarantees: every typed secret is redacted while raw JSON shape and all
    /// non-secret values remain byte-for-byte equivalent as JSON values.
    #[test]
    fn typed_redaction_preserves_raw_shape() {
        let raw = serde_json::json!({
            "password": "top-secret",
            "nested": [
                {"token": "nested-secret", "label": "first"}
            ],
            "tokens": {
                "primary": "map-secret"
            },
            "literal_marker": REDACTED_VALUE
        });

        let redacted =
            redact_typed_config::<TestConfig>(&raw).expect("typed redaction should succeed");

        assert_eq!(
            redacted,
            serde_json::json!({
                "password": REDACTED_VALUE,
                "nested": [
                    {"token": REDACTED_VALUE, "label": "first"}
                ],
                "tokens": {
                    "primary": REDACTED_VALUE
                },
                "literal_marker": REDACTED_VALUE
            })
        );
        assert!(redacted.get("omitted").is_none());
        assert_eq!(raw["password"], "top-secret");
    }

    #[derive(Deserialize, Serialize)]
    struct AliasedConfig {
        #[serde(alias = "token")]
        secret: RedactedString,
    }

    /// Scenario: deserialization accepts an alias whose serialized field path
    /// differs from the original raw key.
    /// Guarantees: redaction fails closed instead of returning a config with an
    /// unmatched cleartext secret.
    #[test]
    fn typed_redaction_rejects_unmatched_alias_path() {
        let error = redact_typed_config::<AliasedConfig>(&serde_json::json!({
            "token": "alias-secret"
        }))
        .expect_err("alias path must not be guessed");

        assert_eq!(error, RedactionError::ShapeMismatch);
        assert!(!error.to_string().contains("alias-secret"));
    }

    #[derive(Deserialize, Serialize)]
    struct SkippedSecretConfig {
        #[serde(rename = "secret", skip_serializing)]
        _secret: RedactedString,
    }

    /// Scenario: a typed secret is deliberately omitted by `Serialize`.
    /// Guarantees: the deserialize/serialize count invariant refuses the result
    /// before a cleartext secret can survive.
    #[test]
    fn typed_redaction_rejects_skipped_secret() {
        let error = redact_typed_config::<SkippedSecretConfig>(&serde_json::json!({
            "secret": "skipped-secret"
        }))
        .expect_err("skipped secret must fail closed");

        assert_eq!(error, RedactionError::SecretCountMismatch);
        assert!(!error.to_string().contains("skipped-secret"));
    }

    /// Scenario: raw config cannot deserialize into its registered type and
    /// contains a sensitive value in the invalid field.
    /// Guarantees: the public error reports only the failure category and never
    /// embeds serde's value-bearing diagnostic.
    #[test]
    fn typed_redaction_sanitizes_deserialization_errors() {
        let error = redact_typed_config::<TestConfig>(&serde_json::json!({
            "password": {"secret": "diagnostic-secret"}
        }))
        .expect_err("invalid typed config must fail");

        assert_eq!(error, RedactionError::Deserialization);
        assert!(!error.to_string().contains("diagnostic-secret"));
    }

    #[derive(Deserialize, Serialize)]
    struct RegisteredConfig {
        secret: RedactedString,
    }

    fn redact_registered_test_config(config: &mut Value) -> Result<(), RedactionError> {
        redact_typed_config_in_place::<RegisteredConfig>(config)
    }

    #[allow(unsafe_code)]
    #[distributed_slice(CONFIG_REDACTORS)]
    static TEST_REDACTOR: ConfigRedactor = ConfigRedactor::new(
        "urn:test:exporter:registered-redaction",
        redact_registered_test_config,
    );

    /// Scenario: a component type has one exact redactor registration.
    /// Guarantees: exact URN dispatch invokes that component-owned redactor.
    #[test]
    fn registered_redaction_uses_exact_component_urn() {
        let mut redacted = serde_json::json!({"secret": "registered-secret"});
        let registered =
            redact_registered_config("urn:test:exporter:registered-redaction", &mut redacted)
                .expect("registered redaction should succeed");

        assert!(registered);
        assert_eq!(redacted["secret"], REDACTED_VALUE);
    }

    /// Scenario: a component type has no redactor registration.
    /// Guarantees: dispatch preserves the raw config without fuzzy matching.
    #[test]
    fn unregistered_redaction_preserves_config() {
        let mut raw = serde_json::json!({"secret": "unregistered-secret"});
        let registered = redact_registered_config("urn:test:exporter:unregistered", &mut raw)
            .expect("unregistered config should be preserved");

        assert!(!registered);
        assert_eq!(raw["secret"], "unregistered-secret");
    }

    /// Scenario: two components register the same exact type URN.
    /// Guarantees: dispatch rejects the ambiguity rather than choosing by link
    /// order.
    #[test]
    fn duplicate_redaction_registration_fails() {
        let registrations = [
            ConfigRedactor::new(
                "urn:test:exporter:duplicate-redaction",
                redact_registered_test_config,
            ),
            ConfigRedactor::new(
                "urn:test:exporter:duplicate-redaction",
                redact_registered_test_config,
            ),
            ConfigRedactor::new(
                "urn:test:exporter:unique-redaction",
                redact_registered_test_config,
            ),
        ];
        let index = build_redactor_index(&registrations);

        assert!(matches!(
            index.get("urn:test:exporter:duplicate-redaction"),
            Some(RedactorEntry::Duplicate)
        ));
        assert!(matches!(
            index.get("urn:test:exporter:unique-redaction"),
            Some(RedactorEntry::Redactor(_))
        ));
    }

    #[derive(Deserialize, Serialize)]
    struct AsymmetricConfig {
        #[serde(rename(serialize = "public", deserialize = "secret"))]
        secret: RedactedString,
        #[serde(rename(serialize = "secret", deserialize = "public"))]
        public: String,
    }

    /// Scenario: asymmetric serde names swap a secret and non-secret field onto
    /// existing raw paths.
    /// Guarantees: a nonmatching raw value prevents redacting the public field
    /// while leaving the real secret exposed.
    #[test]
    fn typed_redaction_rejects_asymmetric_path_collision() {
        let raw = serde_json::json!({
            "secret": "identity-bound-secret",
            "public": "visible-label"
        });

        let error = redact_typed_config::<AsymmetricConfig>(&raw)
            .expect_err("asymmetric secret mapping must fail closed");

        assert_eq!(error, RedactionError::ShapeMismatch);
        assert_eq!(raw["secret"], "identity-bound-secret");
    }

    #[derive(Deserialize, Serialize)]
    struct RepeatedValueConfig {
        primary: RedactedString,
        secondary: RedactedString,
        label: String,
    }

    /// Scenario: multiple symmetric secret fields and a public field share the
    /// same string value.
    /// Guarantees: both declared secret paths are redacted while the public
    /// value remains unchanged.
    #[test]
    fn typed_redaction_accepts_repeated_values_on_symmetric_paths() {
        let raw = serde_json::json!({
            "primary": "same-value",
            "secondary": "same-value",
            "label": "same-value"
        });

        let redacted = redact_typed_config::<RepeatedValueConfig>(&raw)
            .expect("symmetric repeated values should redact");

        assert_eq!(redacted["primary"], REDACTED_VALUE);
        assert_eq!(redacted["secondary"], REDACTED_VALUE);
        assert_eq!(redacted["label"], "same-value");
    }

    #[derive(Deserialize, Serialize)]
    #[serde(untagged)]
    enum UntaggedConfig {
        VariantA {
            secret: RedactedString,
            required_field: String,
        },
        VariantB {
            other: String,
        },
    }

    /// Scenario: an untagged enum tentatively deserializes a secret-bearing
    /// variant before backtracking to a different variant.
    /// Guarantees: dropped tentative secrets do not inflate the final secret
    /// count or make a valid config unsnapshotable.
    #[test]
    fn typed_redaction_handles_untagged_enum_backtracking() {
        let raw = serde_json::json!({
            "secret": "my-secret",
            "other": "val"
        });

        let result = redact_typed_config::<UntaggedConfig>(&raw);
        assert!(
            result.is_ok(),
            "Backtracking should not cause a mismatch: {:?}",
            result
        );
    }

    #[derive(Deserialize, Serialize, Clone)]
    struct ClonedConfig {
        secret: RedactedString,
    }

    /// Scenario: a typed config clones a `RedactedString` while deserialization
    /// tracking is active.
    /// Guarantees: clone and drop lifecycle accounting leaves exactly one live
    /// secret in the final typed value.
    #[test]
    fn tracked_secret_clone_preserves_live_count() {
        let raw = serde_json::json!({
            "secret": "my-secret"
        });
        let (_typed, count) = track_deserialized_secrets(|| {
            let typed = ClonedConfig::deserialize(&raw).unwrap();
            let _cloned = typed.clone();
            Ok(typed)
        })
        .unwrap();
        assert_eq!(count, 1);
    }
}
