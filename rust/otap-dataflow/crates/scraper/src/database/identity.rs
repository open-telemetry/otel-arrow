// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Logical source identity, deliberately separate from checkpoint storage.

use super::{CompiledWatermark, DatabaseSystem, ScalarValue};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

/// Operator-declared execution semantics, not credentials or session commands.
///
/// Password rotation with unchanged visibility does not change this context.
/// Change it when a login, default schema/search path, role, or row-level policy
/// changes the meaning or visibility of the query. Existing checkpoints then
/// fail closed. These declarations do not configure or verify server sessions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceContext {
    /// Effective database principal whose visibility the query relies on.
    #[serde(deserialize_with = "context_value")]
    pub principal: String,
    /// Effective default schema or ordered search path, spelled exactly.
    #[serde(deserialize_with = "context_value")]
    pub schema: String,
    /// Operator version for semantic changes not represented by principal/schema.
    #[serde(default = "default_revision", deserialize_with = "context_value")]
    pub revision: String,
}

fn default_revision() -> String {
    "1".into()
}

fn context_value<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = String::deserialize(deserializer)?;
    if value.trim().is_empty() || value.len() > 1024 || value.chars().any(char::is_control) {
        return Err(D::Error::custom(
            "source context values must contain 1..1024 bytes without control characters",
        ));
    }
    Ok(value)
}

impl SourceContext {
    /// Binds an adapter's existing compatibility contract to execution semantics.
    ///
    /// Call only for an explicitly configured context. With no context adapters
    /// retain their original fingerprint byte for byte, including SQL spelling,
    /// initial cursor, timestamp precision, and adapter-version rules.
    #[must_use]
    pub fn checkpoint_fingerprint(&self, legacy: &str) -> String {
        let mut hash = IdentityHash::new("otel-arrow/database-checkpoint-context/v1");
        hash.add(legacy);
        self.hash_into(&mut hash);
        hash.finish()
    }

    fn hash_into(&self, hash: &mut IdentityHash) {
        hash.add(&self.principal);
        hash.add(&self.schema);
        hash.add(&self.revision);
    }
}

/// Configured database address; no DNS, catalog lookup, or credential resolution.
#[derive(Clone, Copy, Debug)]
pub enum DatabaseEndpoint<'a> {
    /// Adapter-normalized network host, port, and case-sensitive database name.
    Network {
        /// Host spelling after the adapter's existing normalization.
        host: &'a str,
        /// Explicit port, including the adapter's default.
        port: u16,
        /// Exact database name.
        database: &'a str,
    },
    /// Exact Oracle service/descriptor/alias spelling, excluding client paths.
    Oracle {
        /// Configured connect string, treated as opaque.
        connect_string: &'a str,
    },
}

/// Versioned, path-independent identity of a configured database query range.
///
/// Pipeline/receiver names, output `source_id`, checkpoint paths, credentials,
/// and operational budgets are deliberately absent. SQL uses the adapter's
/// existing `SourceQuery::fingerprint_sql` contract. Static predicates identify
/// the range; an initial cursor is only a starting position and cannot establish
/// disjoint ownership, so it is excluded here (but retained in checkpoints).
///
/// Equality detects only identical configured sources, not overlapping SQL,
/// aliases, views, or equivalent statements. This value is not a lease and
/// provides neither process-local nor cross-process coordination.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LogicalSourceId(String);

impl LogicalSourceId {
    /// Identifies validated adapter SQL and its cursor interpretation.
    #[must_use]
    pub fn new(
        system: DatabaseSystem,
        endpoint: DatabaseEndpoint<'_>,
        fingerprint_sql: &str,
        watermark: &CompiledWatermark,
        context: Option<&SourceContext>,
    ) -> Self {
        let mut hash = IdentityHash::new("otel-arrow/database-source/v1");
        hash.add(system.as_str());
        match endpoint {
            DatabaseEndpoint::Network {
                host,
                port,
                database,
            } => {
                hash.add("network");
                hash.add(host);
                hash.add(&port.to_string());
                hash.add(database);
            }
            DatabaseEndpoint::Oracle { connect_string } => {
                hash.add("oracle-connect-string");
                hash.add(connect_string);
            }
        }
        hash.add(fingerprint_sql);
        match watermark {
            CompiledWatermark::Snapshot => hash.add("snapshot"),
            CompiledWatermark::Scalar(cursor) => {
                hash.add("scalar");
                hash.add(&cursor.column);
                hash.add(&cursor.bind);
                hash.add(match &cursor.initial {
                    ScalarValue::Int64(_) => "int64",
                    ScalarValue::UInt64(_) => "uint64",
                    ScalarValue::String(_) => "string",
                    ScalarValue::Timestamp(_) => "timestamp",
                });
            }
            CompiledWatermark::Composite(cursor) => {
                hash.add("composite");
                hash.add(&cursor.timestamp_column);
                hash.add(&cursor.timestamp_bind);
                hash.add(&cursor.tie_breaker_column);
                hash.add(&cursor.tie_breaker_bind);
            }
        }
        if let Some(context) = context {
            hash.add("explicit-context");
            context.hash_into(&mut hash);
        } else {
            hash.add("unspecified-context");
        }
        Self(hash.finish())
    }

    /// Returns the opaque BLAKE3 digest; no endpoint, SQL, or principal is exposed.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

struct IdentityHash(blake3::Hasher);

impl IdentityHash {
    fn new(domain: &str) -> Self {
        let mut hash = Self(blake3::Hasher::new());
        hash.add(domain);
        hash
    }

    fn add(&mut self, value: &str) {
        // Length framing prevents ambiguous concatenations across identity fields.
        _ = self.0.update(&(value.len() as u64).to_le_bytes());
        _ = self.0.update(value.as_bytes());
    }

    fn finish(self) -> String {
        self.0.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
#[path = "identity_tests.rs"]
mod tests;
