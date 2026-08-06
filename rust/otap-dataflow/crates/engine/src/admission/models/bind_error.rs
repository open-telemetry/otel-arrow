// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Failure modes of binding an admission gate at startup.

use super::{AdmissionDimension, AdmissionDimensionSet};

/// Why the engine could not bind an admission gate.
///
/// Every variant is a **startup** failure. Binding happens once, while the
/// component is being constructed, so a misconfiguration is reported as a
/// pipeline build error attributed to the offending node rather than surfacing
/// as an opaque client error on the first request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionBindError {
    /// The component asked to meter a dimension the configured policy does not use.
    ///
    /// For example, a byte-metered limiter bound to a receiver that can only
    /// count framed messages.
    UnsupportedDimension {
        /// Dimension the component requires at its admission point.
        requested: AdmissionDimension,
        /// Dimensions supported by the configured admission policy.
        supported: AdmissionDimensionSet,
    },
    /// The node already claimed this admission binding.
    AlreadyBound,
    /// An explicit node binding was not consumed by its component factory.
    ExplicitBindingNotConsumed {
        /// Configured limiter name.
        limiter: String,
        /// Component URN that did not consume the binding.
        component: String,
    },
}

impl std::fmt::Display for AdmissionBindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedDimension {
                requested,
                supported,
            } => write!(
                f,
                "configured ingress admission does not support the `{}` dimension \
                 required at this admission point (supported: {})",
                requested.as_str(),
                supported.to_display_string()
            ),
            Self::AlreadyBound => f.write_str("ingress admission binding was already consumed"),
            Self::ExplicitBindingNotConsumed { limiter, component } => write!(
                f,
                "component `{component}` explicitly binds rate limiter `{limiter}` but does not consume ingress admission during construction"
            ),
        }
    }
}

impl std::error::Error for AdmissionBindError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a byte-metered component binds against a message policy.
    /// Guarantees: the startup diagnostic names both the required dimension and the
    /// policy's supported set, so an operator can fix the config without reading code.
    #[test]
    fn dimension_mismatch_message_names_required_and_supported() {
        let err = AdmissionBindError::UnsupportedDimension {
            requested: AdmissionDimension::Bytes,
            supported: AdmissionDimensionSet::single(AdmissionDimension::Messages),
        };

        let text = err.to_string();
        assert!(text.contains("`bytes`"), "unexpected message: {text}");
        assert!(text.contains("[messages]"), "unexpected message: {text}");
    }
}
