// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The shared [`ApiKey`] credential.

use secrecy::{ExposeSecret, SecretString};
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::Instant;

static HTTP_HEADER_NAME_ATTRIBUTE: &str = "http.header_name";
static HTTP_HEADER_SCHEME_ATTRIBUTE: &str = "http.header_scheme";

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
    pub fn with_attributes(mut self, attributes: Map<String, Value>) -> Self {
        self.attributes = Some(Arc::new(attributes));
        self
    }

    /// Adds expiry to an API Key.
    #[must_use]
    pub const fn with_expiry(mut self, expires_on: Instant) -> Self {
        self.expires_on = Some(expires_on);
        self
    }

    /// Adds `http.header_name` attribute to an API Key.
    #[must_use]
    pub fn with_http_header_name_attribute(mut self, header_name: &str) -> Self {
        let mut attributes = self
            .attributes
            .map(Arc::unwrap_or_clone)
            .unwrap_or_default();
        attributes[HTTP_HEADER_NAME_ATTRIBUTE] = Value::String(header_name.into());
        self.attributes = Some(Arc::new(attributes));
        self
    }

    /// Adds `http.header_scheme` attribute to an API Key.
    #[must_use]
    pub fn with_http_header_scheme_attribute(mut self, header_scheme: &str) -> Self {
        let mut attributes = self
            .attributes
            .map(Arc::unwrap_or_clone)
            .unwrap_or_default();
        attributes[HTTP_HEADER_SCHEME_ATTRIBUTE] = Value::String(header_scheme.into());
        self.attributes = Some(Arc::new(attributes));
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
            .and_then(|v| v.get(HTTP_HEADER_NAME_ATTRIBUTE))
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
            .and_then(|v| v.get(HTTP_HEADER_SCHEME_ATTRIBUTE))
            && let Value::String(scheme_value) = scheme_value
        {
            return Some(scheme_value.as_str());
        }

        None
    }
}
