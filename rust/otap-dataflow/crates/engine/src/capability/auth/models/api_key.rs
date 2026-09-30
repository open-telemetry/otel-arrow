// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The shared [`ApiKey`] credential.

use http::HeaderName;
use secrecy::{ExposeSecret, SecretString};
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::Instant;

/// An API Key error.
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug)]
pub enum ApiKeyAttributeError {
    #[error("HTTP header name is invalid: {0}")]
    InvalidHttpHeaderName(&'static str),

    #[error("HTTP header scheme is invalid: {0}")]
    InvalidHttpHeaderScheme(&'static str),
}

/// API Key attributes.
#[derive(Clone, Debug)]
pub struct ApiKeyAttributes {
    attributes: Arc<Map<String, Value>>,
}

impl ApiKeyAttributes {
    /// Attribute key containing the HTTP header name.
    const HTTP_HEADER_NAME_ATTRIBUTE: &str = "http.header_name";

    /// Attribute key containing the HTTP header scheme.
    const HTTP_HEADER_SCHEME_ATTRIBUTE: &str = "http.header_scheme";

    /// Create API Key attributes.
    #[must_use]
    pub fn new() -> ApiKeyAttributes {
        Self {
            attributes: Arc::new(Map::new()),
        }
    }

    /// Create API Key attributes from a map.
    pub fn from_map(attributes: Map<String, Value>) -> Result<Self, ApiKeyAttributeError> {
        let mut a = Self::new();
        for (key, value) in attributes {
            a = a.with_attribute(key, value)?;
        }
        Ok(a)
    }

    /// Adds an attribute to an API Key.
    pub fn with_attribute(
        mut self,
        name: impl Into<String>,
        value: impl Into<Value>,
    ) -> Result<Self, ApiKeyAttributeError> {
        let name: String = name.into();
        let value: Value = value.into();
        match name.as_str() {
            Self::HTTP_HEADER_NAME_ATTRIBUTE => {
                if let Value::String(value) = value {
                    self.with_http_header_name_attribute(value)
                } else {
                    Err(ApiKeyAttributeError::InvalidHttpHeaderName(
                        "String value expected",
                    ))
                }
            }
            Self::HTTP_HEADER_SCHEME_ATTRIBUTE => {
                if let Value::String(value) = value {
                    self.with_http_header_scheme_attribute(value)
                } else {
                    Err(ApiKeyAttributeError::InvalidHttpHeaderScheme(
                        "String value expected",
                    ))
                }
            }
            _ => {
                let mut attributes = Arc::unwrap_or_clone(self.attributes);
                _ = attributes.insert(name, value);
                self.attributes = Arc::new(attributes);
                Ok(self)
            }
        }
    }

    /// Adds `http.header_name` attribute to an API Key.
    pub fn with_http_header_name_attribute(
        mut self,
        header_name: impl Into<String>,
    ) -> Result<Self, ApiKeyAttributeError> {
        let header_name: String = header_name.into();
        Self::validate_http_header_name_attribute(&header_name)?;
        let mut attributes = Arc::unwrap_or_clone(self.attributes);
        _ = attributes.insert(
            Self::HTTP_HEADER_NAME_ATTRIBUTE.into(),
            Value::String(header_name),
        );
        self.attributes = Arc::new(attributes);
        Ok(self)
    }

    /// Adds `http.header_scheme` attribute to an API Key.
    pub fn with_http_header_scheme_attribute(
        mut self,
        header_scheme: impl Into<String>,
    ) -> Result<Self, ApiKeyAttributeError> {
        let header_scheme: String = header_scheme.into();
        Self::validate_http_header_scheme_attribute(&header_scheme)?;
        let mut attributes = Arc::unwrap_or_clone(self.attributes);
        _ = attributes.insert(
            Self::HTTP_HEADER_SCHEME_ATTRIBUTE.into(),
            Value::String(header_scheme),
        );
        self.attributes = Arc::new(attributes);
        Ok(self)
    }

    /// Validates an `http.header_name` attribute value.
    fn validate_http_header_name_attribute(header_name: &str) -> Result<(), ApiKeyAttributeError> {
        HeaderName::from_bytes(header_name.as_bytes())
            .map(|_| ())
            .map_err(|_| {
                ApiKeyAttributeError::InvalidHttpHeaderName("Header name could not be parsed")
            })
    }

    /// Validates an `http.header_scheme` attribute value.
    fn validate_http_header_scheme_attribute(
        header_scheme: &str,
    ) -> Result<(), ApiKeyAttributeError> {
        if header_scheme.is_empty()
            || !header_scheme.bytes().all(|byte| {
                byte.is_ascii_alphanumeric()
                    || matches!(
                        byte,
                        b'!' | b'#'
                            | b'$'
                            | b'%'
                            | b'&'
                            | b'\''
                            | b'*'
                            | b'+'
                            | b'-'
                            | b'.'
                            | b'^'
                            | b'_'
                            | b'`'
                            | b'|'
                            | b'~'
                    )
            })
        {
            return Err(ApiKeyAttributeError::InvalidHttpHeaderScheme(
                "Header scheme could not be parsed",
            ));
        }

        Ok(())
    }
}

impl Default for ApiKeyAttributes {
    fn default() -> Self {
        Self::new()
    }
}

/// An API Key.
///
/// The value is wrapped in [`SecretString`], which zeroizes on drop and masks
/// itself in [`Debug`] output, so it cannot leak into logs or telemetry. The
/// `SecretString` sits behind an [`Arc`] so cloning an API Key (handing it to
/// multiple subscribers, or returning it from `get_api_key` on the hot path) is
/// a cheap refcount bump that shares one plaintext allocation rather than
/// copying the secret bytes.
///
/// `expires_on` is a monotonic [`Instant`] -- an absolute wall-clock expiry is
/// converted to an `Instant` once, so the value is immune to wall-clock jumps
/// thereafter. `None` means no known expiry. The API Key value is opaque to
/// this type: an expiry is only ever what a caller supplies from the issuer's
/// response metadata, never parsed out of the API Key itself.
///
/// `attributes` is a map for attaching metadata to the value consumers may use
/// when handling the API Key. [`Debug`] output renders `attributes` verbatim so
/// hosts must keep secrets out of the attribute map.
#[derive(Clone, Debug)]
pub struct ApiKey {
    value: Arc<SecretString>,
    attributes: Option<Arc<Map<String, Value>>>,
    expires_on: Option<Instant>,
}

impl ApiKey {
    /// Creates an API Key from its value.
    #[must_use]
    pub fn new(value: impl Into<SecretString>) -> Self {
        Self {
            value: Arc::new(value.into()),
            attributes: None,
            expires_on: None,
        }
    }

    /// Adds attributes to an API Key.
    #[must_use]
    pub fn with_attributes(mut self, attributes: ApiKeyAttributes) -> Self {
        self.attributes = Some(attributes.attributes.clone());
        self
    }

    /// Adds expiry to an API Key.
    #[must_use]
    pub const fn with_expiry(mut self, expires_on: Instant) -> Self {
        self.expires_on = Some(expires_on);
        self
    }

    /// Exposes the API Key value secret.
    ///
    /// Named `expose_value` (rather than a plain getter) so every plaintext
    /// access is explicit and greppable.
    #[must_use]
    pub fn expose_value(&self) -> &str {
        self.value.expose_secret()
    }

    /// Gets API Key attributes.
    #[must_use]
    pub fn get_attributes(&self) -> Option<&Map<String, Value>> {
        self.attributes.as_deref()
    }

    /// Gets the API Key expiry.
    #[must_use]
    pub const fn get_expires_on(&self) -> Option<Instant> {
        self.expires_on
    }

    /// Gets the API Key `http.header_name` attribute.
    #[must_use]
    pub fn get_http_header_name_attribute(&self) -> Option<&str> {
        if let Some(header_value) = self
            .attributes
            .as_ref()
            .and_then(|v| v.get(ApiKeyAttributes::HTTP_HEADER_NAME_ATTRIBUTE))
            && let Value::String(header_value) = header_value
        {
            return Some(header_value.as_str());
        }

        None
    }

    /// Gets the API Key `http.header_scheme` attribute.
    #[must_use]
    pub fn get_http_header_scheme_attribute(&self) -> Option<&str> {
        if let Some(scheme_value) = self
            .attributes
            .as_ref()
            .and_then(|v| v.get(ApiKeyAttributes::HTTP_HEADER_SCHEME_ATTRIBUTE))
            && let Value::String(scheme_value) = scheme_value
        {
            return Some(scheme_value.as_str());
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: A valid HTTP header name is added with the API key builder.
    /// Guarantees: The builder preserves the valid field name as an attribute.
    #[test]
    fn http_header_name_builder_accepts_valid_value() {
        let key = ApiKey::new("secret").with_attributes(
            ApiKeyAttributes::new()
                .with_http_header_name_attribute("x-api-key")
                .expect("header name is valid"),
        );

        assert_eq!(key.get_http_header_name_attribute(), Some("x-api-key"));
    }

    /// Scenario: A valid HTTP authentication scheme is added with the API key builder.
    /// Guarantees: The builder preserves the case-sensitive token as an attribute.
    #[test]
    fn http_header_scheme_builder_accepts_valid_value() {
        let key = ApiKey::new("secret").with_attributes(
            ApiKeyAttributes::new()
                .with_http_header_scheme_attribute("ApiKey")
                .expect("header scheme is valid"),
        );

        assert_eq!(key.get_http_header_scheme_attribute(), Some("ApiKey"));
    }

    /// Scenario: HTTP header metadata contains empty or malformed token values.
    /// Guarantees: The public typed builders reject invalid values.
    #[test]
    fn http_attribute_builders_reject_malformed_values() {
        for header_name in ["", "invalid header", "x-api-key\n"] {
            assert!(matches!(
                ApiKeyAttributes::new().with_http_header_name_attribute(header_name),
                Err(ApiKeyAttributeError::InvalidHttpHeaderName(_))
            ));
        }

        for header_scheme in [
            "",
            "invalid scheme",
            "ApiKey\n",
            "Api:Key",
            "Api/Key",
            "Api,Key",
            "Api\tKey",
            "ApiK\u{e9}y",
        ] {
            assert!(matches!(
                ApiKeyAttributes::new().with_http_header_scheme_attribute(header_scheme),
                Err(ApiKeyAttributeError::InvalidHttpHeaderScheme(_))
            ));
        }
    }

    /// Scenario: HTTP authentication schemes contain every permitted token character.
    /// Guarantees: The scheme builder accepts the complete RFC token character set.
    #[test]
    fn http_header_scheme_builder_accepts_token_characters() {
        for header_scheme in ["ApiKey", "A!#$%&'*+-.^_`|~9"] {
            assert!(
                ApiKeyAttributes::new()
                    .with_http_header_scheme_attribute(header_scheme)
                    .is_ok(),
                "expected valid scheme: {header_scheme}"
            );
        }
    }

    /// Scenario: Reserved HTTP attributes are supplied through the generic builder.
    /// Guarantees: The generic path enforces both value types and syntax.
    #[test]
    fn generic_attribute_builder_validates_reserved_attributes() {
        for (name, value) in [
            (
                ApiKeyAttributes::HTTP_HEADER_NAME_ATTRIBUTE,
                Value::Number(42.into()),
            ),
            (
                ApiKeyAttributes::HTTP_HEADER_SCHEME_ATTRIBUTE,
                Value::Bool(false),
            ),
            (
                ApiKeyAttributes::HTTP_HEADER_NAME_ATTRIBUTE,
                Value::String("invalid header".into()),
            ),
            (
                ApiKeyAttributes::HTTP_HEADER_SCHEME_ATTRIBUTE,
                Value::String("invalid scheme".into()),
            ),
        ] {
            assert!(ApiKeyAttributes::new().with_attribute(name, value).is_err());
        }
    }

    /// Scenario: A map contains custom metadata and valid reserved HTTP attributes.
    /// Guarantees: Map conversion preserves every value for API key consumers.
    #[test]
    fn from_map_preserves_valid_attributes() {
        let attributes = serde_json::from_value(serde_json::json!({
            "tenant.id": 42,
            "http.header_name": "x-api-key",
            "http.header_scheme": "ApiKey"
        }))
        .expect("attributes are an object");
        let key = ApiKey::new("secret")
            .with_attributes(ApiKeyAttributes::from_map(attributes).expect("attributes are valid"));

        assert_eq!(key.get_http_header_name_attribute(), Some("x-api-key"));
        assert_eq!(key.get_http_header_scheme_attribute(), Some("ApiKey"));
        assert_eq!(
            key.get_attributes()
                .and_then(|values| values.get("tenant.id")),
            Some(&Value::Number(42.into()))
        );
    }

    /// Scenario: An attribute collection is cloned and one copy replaces an attribute.
    /// Guarantees: Replacement affects only the modified copy and leaves the clone isolated.
    #[test]
    fn cloned_attributes_replace_values_independently() {
        let original = ApiKeyAttributes::new()
            .with_attribute("tenant.id", 42)
            .expect("custom attribute is valid");
        let updated = original
            .clone()
            .with_attribute("tenant.id", 43)
            .expect("replacement attribute is valid");
        let original_key = ApiKey::new("original").with_attributes(original);
        let updated_key = ApiKey::new("updated").with_attributes(updated);

        assert_eq!(
            original_key
                .get_attributes()
                .and_then(|values| values.get("tenant.id")),
            Some(&Value::Number(42.into()))
        );
        assert_eq!(
            updated_key
                .get_attributes()
                .and_then(|values| values.get("tenant.id")),
            Some(&Value::Number(43.into()))
        );
    }
}
