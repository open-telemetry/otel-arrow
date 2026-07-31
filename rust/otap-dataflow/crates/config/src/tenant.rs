// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tenant token configuration types.
//!
//! A *tenant token* is a small vector of `key: value` identifiers describing
//! the tenant that a request belongs to. Tokens are produced by receivers from
//! request-scoped material (transport headers, peer address, and eventually
//! authorization data) and consumed by downstream nodes that evaluate
//! first-match-wins *conditions* over them.
//!
//! This module holds only the user-facing configuration shapes. The compiled,
//! hot-path representation lives in [`crate::tenant::compiled`].
//!
//! Extracted values may optionally be *retained*, which is what allows tenant
//! tokens to subsume the general-purpose transport header map: instead of
//! carrying every captured header as an owned string pair, the engine carries
//! only the configured token keys.
//!
//! A token deliberately says nothing about the wire name a retained value is
//! re-emitted under. The token is the portable identity; how it appears on the
//! wire is a site-specific decision belonging to the node that does the
//! emitting. Exporters therefore map `key -> outbound header name` themselves,
//! and the same token can be emitted under different names by two exporters.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod compiled;

/// Identifier of a tenant token definition, as used in engine configuration.
pub type TenantTokenId = String;

/// A single rule that resolves one token key from the request context.
///
/// Variants are untagged and disambiguated by their distinguishing field, so
/// an extractor reads as a flat map in YAML.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Extractor {
    /// Copy a transport header value into the token key.
    TransportHeader {
        /// Token key this extractor resolves.
        key: String,
        /// Transport header name to read, matched case-insensitively.
        transport_header: String,
        /// Retain the value in the request context. Retained values can be
        /// re-emitted by an exporter under a name that exporter chooses, and
        /// offered to a downstream pipeline by a boundary policy. Keys that
        /// are not retained participate in matching only and cost no bytes.
        #[serde(default)]
        retain: bool,
        /// Also carry the key's name, so the pair can be appended to telemetry
        /// attributes without re-encoding. This is the only way a name travels
        /// with a request; every other use of a key name is compiled out.
        /// Implies `retain`.
        #[serde(default)]
        bag: bool,
    },
    /// Resolve the token key to a static, configured value.
    ///
    /// This is how a pipeline mints an identity of its own, such as the tenant
    /// a dedicated pipeline is dedicated to.
    GenericKey {
        /// Token key this extractor resolves.
        key: String,
        /// Static value assigned to the key.
        generic_key: String,
        /// Retain the value in the request context. Retained values can be
        /// re-emitted by an exporter under a name that exporter chooses, and
        /// offered to a downstream pipeline by a boundary policy. Keys that
        /// are not retained participate in matching only and cost no bytes.
        #[serde(default)]
        retain: bool,
        /// Also carry the key's name, so the pair can be appended to telemetry
        /// attributes without re-encoding. This is the only way a name travels
        /// with a request; every other use of a key name is compiled out.
        /// Implies `retain`.
        #[serde(default)]
        bag: bool,
    },
    /// Resolve the token key to the network peer's address.
    RemoteAddress {
        /// Token key this extractor resolves.
        key: String,
        /// Must be `true`; selects this extractor kind.
        remote_address: bool,
        /// Retain the value in the request context. Retained values can be
        /// re-emitted by an exporter under a name that exporter chooses, and
        /// offered to a downstream pipeline by a boundary policy. Keys that
        /// are not retained participate in matching only and cost no bytes.
        #[serde(default)]
        retain: bool,
        /// Also carry the key's name, so the pair can be appended to telemetry
        /// attributes without re-encoding. This is the only way a name travels
        /// with a request; every other use of a key name is compiled out.
        /// Implies `retain`.
        #[serde(default)]
        bag: bool,
    },
    /// Resolve the token key from a value retained by an upstream pipeline
    /// and carried across a pipeline or group boundary.
    ///
    /// This is the import half of cross-boundary propagation: the upstream
    /// pipeline retains a key, the boundary policy admits it, and this
    /// extractor binds it into a token belonging to the downstream pipeline.
    ImportedKey {
        /// Token key this extractor resolves.
        key: String,
        /// Key name to read from the inbound cross-boundary context.
        imported_key: String,
        /// Retain the value in the request context. Retained values can be
        /// re-emitted by an exporter under a name that exporter chooses, and
        /// offered to a downstream pipeline by a boundary policy. Keys that
        /// are not retained participate in matching only and cost no bytes.
        #[serde(default)]
        retain: bool,
        /// Also carry the key's name, so the pair can be appended to telemetry
        /// attributes without re-encoding. This is the only way a name travels
        /// with a request; every other use of a key name is compiled out.
        /// Implies `retain`.
        #[serde(default)]
        bag: bool,
    },
}

impl Extractor {
    /// Token key resolved by this extractor.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            Self::TransportHeader { key, .. }
            | Self::GenericKey { key, .. }
            | Self::RemoteAddress { key, .. }
            | Self::ImportedKey { key, .. } => key,
        }
    }

    /// Whether this extractor's value is retained in the request context.
    #[must_use]
    pub fn retain(&self) -> bool {
        match self {
            Self::TransportHeader { retain, bag, .. }
            | Self::GenericKey { retain, bag, .. }
            | Self::RemoteAddress { retain, bag, .. }
            | Self::ImportedKey { retain, bag, .. } => *retain || *bag,
        }
    }

    /// Whether this extractor's key name travels alongside its value.
    #[must_use]
    pub fn bag(&self) -> bool {
        match self {
            Self::TransportHeader { bag, .. }
            | Self::GenericKey { bag, .. }
            | Self::RemoteAddress { bag, .. }
            | Self::ImportedKey { bag, .. } => *bag,
        }
    }
}

/// A named tenant token definition: the list of extractors that must all
/// resolve for the token to be present on a request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TenantTokenSpec {
    /// Extractors that must all resolve for this token to be resolved.
    pub extractors: Vec<Extractor>,
}

/// One `{ key, value }` term of a condition. A missing `value` is a wildcard:
/// the key must be present with any value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// Token key tested by this entry.
    pub key: String,
    /// Required value, or `None` to accept any value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// An ordered list of entries selecting a destination. All entries must match
/// for the condition to match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    /// Entries that must all match.
    pub entries: Vec<Entry>,
}

/// Engine-level map of tenant token definitions, shared across pipeline groups.
pub type TenantTokens = HashMap<TenantTokenId, TenantTokenSpec>;

/// One route of a tenant-token router: a condition plus the destination topic
/// selected when that condition is the first match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TenantRoute {
    /// Entries that must all match for this route to be selected.
    pub entries: Vec<Entry>,
    /// Destination topic name.
    pub topic: String,
}

/// Tenant-token routing configuration shared by the controller (which must
/// know every declared condition before nodes are built) and the routing node
/// itself.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TenantRouting {
    /// Tenant tokens this router binds. Empty binds every declared token.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tenant_tokens: Vec<TenantTokenId>,
    /// Routes evaluated first-match-wins.
    pub routes: Vec<TenantRoute>,
    /// Keys allowed to cross the topic boundary with the published data.
    #[serde(default)]
    pub export: TenantBoundaryPolicy,
    /// Topic used when no route matches. Without it, unmatched data is nacked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_topic: Option<String>,
}

/// Names an outbound header and the token key whose retained value fills it.
///
/// This is where a retained token key acquires a wire name. Keeping the name
/// here rather than in the token definition means the same portable token can
/// be emitted as `x-acme-customer` by one exporter and `x-customer-id` by
/// another, and that a token carries no assumptions about any backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TenantHeader {
    /// Token key supplying the value.
    pub key: String,
    /// Outbound header name.
    pub header: String,
    /// Emit the value as binary metadata (gRPC `-bin` keys).
    #[serde(default)]
    pub binary: bool,
}

/// Policy limiting which retained token keys may cross a pipeline or group
/// boundary.
///
/// Boundaries are the only places tenant material can leak between tenants, so
/// both sides of every boundary carry an explicit allowlist and everything not
/// named is dropped. An empty or absent policy propagates nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TenantBoundaryPolicy {
    /// Token keys admitted across the boundary. Anything else is dropped.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow_keys: Vec<String>,
}

impl TenantBoundaryPolicy {
    /// True when the policy admits nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.allow_keys.is_empty()
    }
}

/// Rules a boundary-crossing receiver uses to build a fresh request context.
///
/// The inbound context from the upstream pipeline is never adopted as-is. The
/// receiver admits the keys its `import` policy names, then resolves its own
/// tokens over the admitted values plus any locally minted generic keys, so
/// the downstream pipeline evaluates conditions against identities it declared
/// itself.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TenantContextRules {
    /// Keys admitted from the inbound cross-boundary context.
    #[serde(default)]
    pub import: TenantBoundaryPolicy,
    /// Tokens resolved after import. Empty resolves every declared token.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<TenantTokenId>,
}

impl TenantContextRules {
    /// The bound token names, or `None` to bind every declared token.
    #[must_use]
    pub fn bound_tokens(&self) -> Option<&[TenantTokenId]> {
        (!self.tokens.is_empty()).then_some(self.tokens.as_slice())
    }
}

/// Groups work by tenant condition so that a merged unit never mixes tenants.
///
/// Used by the batch processor: each condition is one partition, so every
/// output batch carries a single tenant context and the retained values
/// survive the merge intact.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TenantPartitioning {
    /// Tenant tokens this node binds. Empty binds every declared token.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tenant_tokens: Vec<TenantTokenId>,
    /// Partitions, evaluated first-match-wins. Data matching no partition
    /// falls into a catch-all partition that carries no tenant context.
    pub partitions: Vec<Condition>,
}

impl TenantPartitioning {
    /// The bound token names, or `None` to bind every declared token.
    #[must_use]
    pub fn bound_tokens(&self) -> Option<&[TenantTokenId]> {
        (!self.tenant_tokens.is_empty()).then_some(self.tenant_tokens.as_slice())
    }
}

impl TenantRouting {
    /// The bound token names, or `None` to bind every declared token.
    #[must_use]
    pub fn bound_tokens(&self) -> Option<&[TenantTokenId]> {
        (!self.tenant_tokens.is_empty()).then_some(self.tenant_tokens.as_slice())
    }

    /// The routes' conditions, in route order.
    #[must_use]
    pub fn conditions(&self) -> Vec<Condition> {
        self.routes
            .iter()
            .map(|route| Condition {
                entries: route.entries.clone(),
            })
            .collect()
    }
}
