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
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Placeholder emitted for type-owned secrets in config snapshots.
pub const REDACTED_VALUE: &str = "[REDACTED]";

thread_local! {
    static TRACKED_SECRET_COUNT: Cell<Option<usize>> = const { Cell::new(None) };
}

/// A string value that remains cleartext in memory but always serializes as the
/// redaction marker.
///
/// This wrapper is for config values, not map keys. Component owners declare
/// the corresponding raw fields in their exact [`ConfigRedactor`]
/// registration.
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

impl Drop for RedactedString {
    fn drop(&mut self) {
        TRACKED_SECRET_COUNT.with(|count| {
            if let Some(current) = count.get() {
                count.set(Some(current.saturating_sub(1)));
            }
        });
    }
}

impl RedactedString {
    /// Returns the cleartext value for an explicit runtime use.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
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
        serializer.serialize_str(REDACTED_VALUE)
    }
}

/// A sanitized failure from snapshot redaction.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RedactionError {
    /// The raw config no longer deserializes into its registered type.
    #[error("registered config could not be deserialized for snapshot redaction")]
    Deserialization,
    /// The typed config could not be inspected for declared secret fields.
    #[error("registered config could not be serialized for snapshot redaction")]
    Serialization,
    /// Typed secret declarations do not match the final typed value.
    #[error("registered config did not declare every deserialized secret")]
    SecretCountMismatch,
    /// A declared raw secret field did not match the original config shape.
    #[error("registered config secret fields did not match the raw config shape")]
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

/// Accessor binding a raw field declaration to its typed secret.
pub type SecretAccessor<T> = for<'a> fn(&'a T) -> Option<&'a RedactedString>;

/// A top-level raw config field owned by a typed component.
pub struct SecretField<T> {
    name: &'static str,
    required: bool,
    accessor: SecretAccessor<T>,
}

impl<T> Copy for SecretField<T> {}

impl<T> Clone for SecretField<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> SecretField<T> {
    /// Declares a required top-level secret field.
    #[must_use]
    pub const fn required(name: &'static str, accessor: SecretAccessor<T>) -> Self {
        Self {
            name,
            required: true,
            accessor,
        }
    }

    /// Declares an optional top-level secret field.
    #[must_use]
    pub const fn optional(name: &'static str, accessor: SecretAccessor<T>) -> Self {
        Self {
            name,
            required: false,
            accessor,
        }
    }
}

/// Declares a required top-level `RedactedString` from one Rust field token.
#[macro_export]
macro_rules! required_secret_field {
    ($config_type:ty, $field:ident) => {
        $crate::redaction::SecretField::required(stringify!($field), |config: &$config_type| {
            Some(&config.$field)
        })
    };
}

/// Declares an optional top-level `RedactedString` from one Rust field token.
#[macro_export]
macro_rules! optional_secret_field {
    ($config_type:ty, $field:ident) => {
        $crate::redaction::SecretField::optional(stringify!($field), |config: &$config_type| {
            config.$field.as_ref()
        })
    };
}

/// Function pointer registered by a component that owns a typed config.
pub type ConfigRedactorFn = fn(&mut Value) -> Result<(), RedactionError>;

/// Type-owned snapshot redactor registration.
///
/// The component's callback must deserialize the same config type used by its
/// validation/runtime path and declare every `RedactedString` raw field.
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

/// Applies the exact registered redactor, or returns `false` when the component
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

/// Returns a redacted copy of a typed component config.
pub fn redact_typed_config<T>(
    config: &Value,
    secret_fields: &[SecretField<T>],
) -> Result<Value, RedactionError>
where
    T: DeserializeOwned + Serialize,
{
    let mut redacted = config.clone();
    redact_typed_config_in_place::<T>(&mut redacted, secret_fields)?;
    Ok(redacted)
}

/// Redacts declared top-level `RedactedString` fields in place.
pub fn redact_typed_config_in_place<T>(
    config: &mut Value,
    secret_fields: &[SecretField<T>],
) -> Result<(), RedactionError>
where
    T: DeserializeOwned + Serialize,
{
    redact_typed_config_in_place_with_snapshot(config, secret_fields, |typed| {
        serde_json::to_value(typed).map_err(|_| RedactionError::Serialization)
    })
}

/// Redacts declared top-level `RedactedString` fields using a type-owned
/// snapshot view.
///
/// This variant lets a runtime config remain deserialize-only. The snapshot
/// function must serialize each declared secret under its raw config field
/// name through `RedactedString` so the marker-backed identity check remains
/// fail closed.
pub fn redact_typed_config_in_place_with_snapshot<T>(
    config: &mut Value,
    secret_fields: &[SecretField<T>],
    snapshot: impl FnOnce(&T) -> Result<Value, RedactionError>,
) -> Result<(), RedactionError>
where
    T: DeserializeOwned,
{
    let (typed, deserialized_secrets) = track_deserialized_secrets(|| {
        T::deserialize(&*config).map_err(|_| RedactionError::Deserialization)
    })?;
    let typed_json = snapshot(&typed)?;
    let typed_object = typed_json
        .as_object()
        .ok_or(RedactionError::ShapeMismatch)?;
    let object = config.as_object().ok_or(RedactionError::ShapeMismatch)?;
    let mut active_fields = Vec::with_capacity(secret_fields.len());
    let mut declared_names = HashSet::with_capacity(secret_fields.len());

    for field in secret_fields {
        if !declared_names.insert(field.name) {
            return Err(RedactionError::ShapeMismatch);
        }
        match object.get(field.name) {
            Some(Value::String(raw)) => {
                if !matches!(
                    typed_object.get(field.name),
                    Some(Value::String(value)) if value == REDACTED_VALUE
                ) {
                    return Err(RedactionError::ShapeMismatch);
                }
                let Some(secret) = (field.accessor)(&typed) else {
                    return Err(RedactionError::ShapeMismatch);
                };
                if secret.expose() != raw {
                    return Err(RedactionError::ShapeMismatch);
                }
                active_fields.push(field.name);
            }
            Some(Value::Null) | None if !field.required && (field.accessor)(&typed).is_none() => {}
            _ => return Err(RedactionError::ShapeMismatch),
        }
    }

    if active_fields.len() != deserialized_secrets {
        return Err(RedactionError::SecretCountMismatch);
    }

    let object = config
        .as_object_mut()
        .ok_or(RedactionError::ShapeMismatch)?;
    for field in active_fields {
        let value = object.get_mut(field).ok_or(RedactionError::ShapeMismatch)?;
        *value = Value::String(REDACTED_VALUE.to_owned());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[allow(dead_code)]
    struct TestConfig {
        password: RedactedString,
        optional_token: Option<RedactedString>,
        label: String,
    }

    #[derive(Deserialize)]
    struct DeserializeOnlyConfig {
        password: RedactedString,
        label: String,
    }

    #[derive(Serialize)]
    struct SecretSnapshot<'a> {
        password: &'a RedactedString,
    }

    const TEST_SECRET_FIELDS: &[SecretField<TestConfig>] = &[
        crate::required_secret_field!(TestConfig, password),
        crate::optional_secret_field!(TestConfig, optional_token),
    ];

    fn test_password(config: &TestConfig) -> Option<&RedactedString> {
        Some(&config.password)
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

    /// Scenario: required and optional typed secrets share a value with a
    /// public field.
    /// Guarantees: declared secret fields are redacted by raw path while the
    /// public value and source config remain unchanged.
    #[test]
    fn typed_redaction_preserves_shape_with_repeated_values() {
        let raw = serde_json::json!({
            "password": "same-value",
            "optional_token": "same-value",
            "label": "same-value"
        });

        let redacted = redact_typed_config::<TestConfig>(&raw, TEST_SECRET_FIELDS)
            .expect("declared secrets should redact");

        assert_eq!(redacted["password"], REDACTED_VALUE);
        assert_eq!(redacted["optional_token"], REDACTED_VALUE);
        assert_eq!(redacted["label"], "same-value");
        assert_eq!(raw["password"], "same-value");
    }

    /// Scenario: an optional secret is absent from raw config.
    /// Guarantees: redaction leaves the field omitted and preserves all
    /// non-secret values.
    #[test]
    fn typed_redaction_preserves_omitted_optional_secret() {
        let raw = serde_json::json!({
            "password": "required-secret",
            "label": "visible"
        });

        let redacted = redact_typed_config::<TestConfig>(&raw, TEST_SECRET_FIELDS)
            .expect("optional secret may be omitted");

        assert_eq!(redacted["password"], REDACTED_VALUE);
        assert!(redacted.get("optional_token").is_none());
        assert_eq!(redacted["label"], "visible");
    }

    /// Scenario: a runtime config intentionally implements only Deserialize.
    /// Guarantees: a private secret-only snapshot view preserves marker-backed
    /// field identity without exposing Serialize on the runtime config.
    #[test]
    fn typed_redaction_accepts_private_snapshot_view() {
        let mut raw = serde_json::json!({
            "password": "required-secret",
            "label": "visible"
        });

        redact_typed_config_in_place_with_snapshot::<DeserializeOnlyConfig>(
            &mut raw,
            &[crate::required_secret_field!(
                DeserializeOnlyConfig,
                password
            )],
            |typed| {
                assert_eq!(typed.label, "visible");
                serde_json::to_value(SecretSnapshot {
                    password: &typed.password,
                })
                .map_err(|_| RedactionError::Serialization)
            },
        )
        .expect("private snapshot view should support typed redaction");

        assert_eq!(raw["password"], REDACTED_VALUE);
        assert_eq!(raw["label"], "visible");
    }

    /// Scenario: a secret-bearing typed field is not declared by its component
    /// registration.
    /// Guarantees: the live typed-secret count fails closed before a partially
    /// redacted snapshot can escape.
    #[test]
    fn typed_redaction_rejects_undeclared_secret_field() {
        let raw = serde_json::json!({
            "password": "required-secret",
            "label": "visible"
        });

        let error = redact_typed_config::<TestConfig>(&raw, &[])
            .expect_err("undeclared typed secret must fail closed");

        assert_eq!(error, RedactionError::SecretCountMismatch);
    }

    /// Scenario: a component declares a public string field instead of its
    /// actual `RedactedString` field while the declaration count still matches.
    /// Guarantees: typed serialization rejects the wrong declaration even when
    /// the public field has the same bytes as the real secret.
    #[test]
    fn typed_redaction_rejects_misdeclared_public_field() {
        let raw = serde_json::json!({
            "password": "required-secret",
            "label": "required-secret"
        });

        let error = redact_typed_config::<TestConfig>(
            &raw,
            &[SecretField::required("label", test_password)],
        )
        .expect_err("public field declaration must fail closed");

        assert_eq!(error, RedactionError::ShapeMismatch);
        assert_eq!(raw["password"], "required-secret");
    }

    /// Scenario: two live typed secrets are paired with duplicate declarations
    /// for the same raw field.
    /// Guarantees: duplicate declaration names fail closed instead of masking
    /// one field twice and leaving its sibling secret exposed.
    #[test]
    fn typed_redaction_rejects_duplicate_declarations() {
        let raw = serde_json::json!({
            "password": "first-secret",
            "optional_token": "second-secret",
            "label": "visible"
        });

        let error = redact_typed_config::<TestConfig>(
            &raw,
            &[
                crate::required_secret_field!(TestConfig, password),
                crate::required_secret_field!(TestConfig, password),
            ],
        )
        .expect_err("duplicate declarations must fail closed");

        assert_eq!(error, RedactionError::ShapeMismatch);
    }

    /// Scenario: raw config cannot deserialize into its registered type and
    /// contains a sensitive value in the invalid field.
    /// Guarantees: the public error reports only the failure category and never
    /// embeds serde's value-bearing diagnostic.
    #[test]
    fn typed_redaction_sanitizes_deserialization_errors() {
        let error = redact_typed_config::<TestConfig>(
            &serde_json::json!({"password": {"secret": "diagnostic-secret"}}),
            TEST_SECRET_FIELDS,
        )
        .expect_err("invalid typed config must fail");

        assert_eq!(error, RedactionError::Deserialization);
        assert!(!error.to_string().contains("diagnostic-secret"));
    }

    #[derive(Deserialize, Serialize)]
    #[serde(untagged)]
    #[allow(dead_code)]
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
            "secret": "tentative-secret",
            "other": "selected"
        });

        let _ = redact_typed_config::<UntaggedConfig>(&raw, &[])
            .expect("backtracked secret must not remain live");
    }

    #[derive(Deserialize, Serialize, Clone)]
    #[allow(dead_code)]
    struct ClonedConfig {
        secret: RedactedString,
    }

    /// Scenario: a typed config clones a `RedactedString` while deserialization
    /// tracking is active.
    /// Guarantees: clone and drop lifecycle accounting leaves exactly one live
    /// secret in the final typed value.
    #[test]
    fn tracked_secret_clone_preserves_live_count() {
        let raw = serde_json::json!({"secret": "clone-secret"});
        let (_typed, count) = track_deserialized_secrets(|| {
            let typed = ClonedConfig::deserialize(&raw).expect("config should deserialize");
            let _cloned = typed.clone();
            Ok(typed)
        })
        .expect("tracking should succeed");

        assert_eq!(count, 1);
    }

    #[derive(Deserialize, Serialize)]
    #[allow(dead_code)]
    struct RegisteredConfig {
        secret: RedactedString,
    }

    fn redact_registered_test_config(config: &mut Value) -> Result<(), RedactionError> {
        redact_typed_config_in_place::<RegisteredConfig>(
            config,
            &[crate::required_secret_field!(RegisteredConfig, secret)],
        )
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

    /// Scenario: a mixed registry contains a duplicate URN and an unrelated
    /// unique URN.
    /// Guarantees: only the duplicate entry is poisoned; the unrelated entry
    /// remains callable.
    #[test]
    fn duplicate_redaction_registration_is_isolated() {
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
}
