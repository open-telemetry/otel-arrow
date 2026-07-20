// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The [`AuthorizedIdentity`] an authorizer emits when it admits a request.

/// The authenticated principal an authorizer identified while allowing a
/// request.
///
/// Deliberately scheme-agnostic: `subject` is the principal the credential
/// represents and `audience` is the audience it was accepted for. For a
/// Kubernetes service-account-token authorizer, `subject` is the SA username
/// (`system:serviceaccount:<namespace>:<name>`) and `audience` is the matched
/// audience; other authorizers populate them from their own token model. Both
/// are optional so an authorizer that allows without a meaningful identity (e.g.
/// a static allow-list) can return an empty identity.
///
/// Marked `#[non_exhaustive]`: as the input to downstream per-tenant/route
/// authorization it is expected to grow (e.g. OIDC groups/claims), and this
/// keeps such additions non-breaking.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct AuthorizedIdentity {
    subject: Option<String>,
    audience: Option<String>,
}

impl AuthorizedIdentity {
    /// Creates an empty identity (no subject, no audience).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            subject: None,
            audience: None,
        }
    }

    /// Sets the subject (the principal the credential represents).
    #[must_use]
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Sets the audience the credential was accepted for.
    #[must_use]
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }

    /// The principal the credential represents, if known.
    #[must_use]
    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    /// The audience the credential was accepted for, if known.
    #[must_use]
    pub fn audience(&self) -> Option<&str> {
        self.audience.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: build an identity via the `with_subject`/`with_audience`
    /// builders, and separately construct an empty one with `new`.
    /// Guarantees: the builder stores and returns exactly the set subject and
    /// audience, while a `new` identity reports `None` for both.
    #[test]
    fn builder_sets_and_reads_fields() {
        let identity = AuthorizedIdentity::new()
            .with_subject("system:serviceaccount:default:my-sa")
            .with_audience("https://my-service.example");
        assert_eq!(
            identity.subject(),
            Some("system:serviceaccount:default:my-sa")
        );
        assert_eq!(identity.audience(), Some("https://my-service.example"));

        let empty = AuthorizedIdentity::new();
        assert_eq!(empty.subject(), None);
        assert_eq!(empty.audience(), None);
    }
}
