// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// A string wrapper that redacts its value in Debug/Display output.
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct OpaqueString(String);

impl Deref for OpaqueString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for OpaqueString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Debug for OpaqueString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpaqueString(\"***\")")
    }
}

impl Display for OpaqueString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "***")
    }
}

impl From<String> for OpaqueString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for OpaqueString {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_debug_is_redacted() {
        let value: OpaqueString = "super-secret".into();
        assert_eq!(format!("{value:?}"), "OpaqueString(\"***\")");
    }

    #[test]
    fn test_display_is_redacted() {
        let value: OpaqueString = "super-secret".into();
        assert_eq!(value.to_string(), "***");
    }

    #[test]
    fn test_serde_roundtrip() {
        let value: OpaqueString = "super-secret".into();
        let serialized = serde_json::to_string(&value).expect("serialize");
        assert_eq!(serialized, "\"super-secret\"");

        let deserialized: OpaqueString = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(&*deserialized, "super-secret");
    }

    #[test]
    fn test_deref_to_str() {
        let value: OpaqueString = "super-secret".into();
        let as_str: &str = &value;
        assert_eq!(as_str, "super-secret");
    }
}
