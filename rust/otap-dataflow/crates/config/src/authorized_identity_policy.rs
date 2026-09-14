// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Authorized identity claim projection policy.

use crate::ContextEntryName;
use crate::error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Policy selecting verified authorization claims for pdata context storage.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(transparent)]
pub struct AuthorizedIdentityPolicy {
    entries: Vec<AuthorizedIdentityClaim>,
}

impl AuthorizedIdentityPolicy {
    /// Returns the configured claim projections.
    pub fn iter(&self) -> impl Iterator<Item = &AuthorizedIdentityClaim> {
        self.entries.iter()
    }

    /// Returns the number of configured claim projections.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether no claims are configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Validates claim names and destination uniqueness.
    pub fn validate(&self) -> Result<(), Error> {
        let mut destinations = HashSet::with_capacity(self.entries.len());
        for entry in &self.entries {
            if entry.claim.is_empty() {
                return Err(Error::InvalidUserConfig {
                    error: "authorized_identity claim names must not be empty".to_string(),
                });
            }
            if !destinations.insert(&entry.store_as) {
                return Err(Error::InvalidUserConfig {
                    error: format!(
                        "authorized_identity destination `{}` is configured more than once",
                        entry.store_as
                    ),
                });
            }
        }
        Ok(())
    }
}

/// One verified claim projected into a named pdata context entry.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(deny_unknown_fields)]
pub struct AuthorizedIdentityClaim {
    /// Verified claim name. Use `sub` for the authorized subject.
    pub claim: Box<str>,
    /// Destination pdata context entry name.
    pub store_as: ContextEntryName,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a policy selects subject and a multi-valued groups claim.
    /// Guarantees: the RFC-minimal list syntax preserves claim and destination names.
    #[test]
    fn policy_deserializes_claim_projections() {
        let policy: AuthorizedIdentityPolicy = serde_yaml::from_str(
            r#"
- claim: sub
  store_as: customer_id
- claim: groups
  store_as: access_groups
"#,
        )
        .expect("valid policy");

        let entries = policy.iter().collect::<Vec<_>>();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].claim.as_ref(), "sub");
        assert_eq!(entries[0].store_as, "customer_id");
        assert_eq!(entries[1].claim.as_ref(), "groups");
        assert_eq!(entries[1].store_as, "access_groups");
        assert!(policy.validate().is_ok());
    }

    /// Scenario: two claims target the same context entry.
    /// Guarantees: ambiguous destination collisions are rejected during policy validation.
    #[test]
    fn policy_rejects_duplicate_destinations() {
        let policy: AuthorizedIdentityPolicy = serde_yaml::from_str(
            r#"
- claim: sub
  store_as: tenant
- claim: groups
  store_as: tenant
"#,
        )
        .expect("policy parses before semantic validation");

        assert!(policy.validate().is_err());
    }

    /// Scenario: a projection contains an empty claim name.
    /// Guarantees: a policy cannot project an unspecified authorization field.
    #[test]
    fn policy_rejects_empty_claim_name() {
        let policy: AuthorizedIdentityPolicy = serde_yaml::from_str(
            r#"
- claim: ""
  store_as: tenant
"#,
        )
        .expect("policy parses before semantic validation");

        assert!(policy.validate().is_err());
    }
}
