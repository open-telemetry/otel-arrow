// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Receiver-side helpers for turning request metadata into a tenant context.
//!
//! Resolution is on the per-request path of every receiver, so the two costs
//! that would otherwise dominate are removed here rather than at each call
//! site:
//!
//! - The scratch buffer is thread-local. A gRPC service object is built per
//!   call, so owning the scratch there would allocate once per request, which
//!   is exactly what the packed context exists to avoid. Pipelines are
//!   thread-per-core and resolution never yields, so one scratch per thread is
//!   both sufficient and uncontended.
//! - Header values are produced lazily. tonic stores `-bin` values base64
//!   encoded and decoding them allocates, so the decode is deferred until the
//!   registry confirms some token actually declared that header name.

use otap_df_config::tenant::compiled::{HeaderValue, TenantTokenRegistry, TokenInputs};
use std::borrow::Cow;
use std::cell::RefCell;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::metadata::{KeyAndValueRef, MetadataMap};

thread_local! {
    /// Per-thread resolution scratch, resized on first use by `reset`.
    static SCRATCH: RefCell<otap_df_config::tenant::compiled::TokenScratch> =
        RefCell::new(otap_df_config::tenant::compiled::TokenScratch::new());
}

/// A tonic metadata value that has not been decoded yet.
///
/// ASCII values borrow straight out of the request; binary values are base64
/// on the wire and only pay for the decode if the name matched.
enum LazyMetadataValue<'a> {
    /// Value already stored as bytes; borrowing is free.
    Ascii(&'a tonic::metadata::MetadataValue<tonic::metadata::Ascii>),
    /// Value stored base64 encoded; reading it decodes.
    Binary(&'a tonic::metadata::MetadataValue<tonic::metadata::Binary>),
}

impl HeaderValue for LazyMetadataValue<'_> {
    fn bytes(&self) -> Cow<'_, [u8]> {
        match self {
            Self::Ascii(value) => Cow::Borrowed(value.as_bytes()),
            // A malformed base64 value resolves as empty rather than as the
            // raw encoded bytes: admitting the encoded form would let a
            // corrupt header masquerade as a legitimate token value.
            Self::Binary(value) => value.to_bytes().map_or(Cow::Borrowed(&[][..]), |decoded| {
                Cow::Owned(decoded.to_vec())
            }),
        }
    }
}

/// Resolves the tenant context from any header source that already holds
/// decoded bytes, such as HTTP.
///
/// Returns `None` when no token resolved, in which case the request carries no
/// tenant context and no allocation was made.
#[must_use]
pub fn resolve_pairs<'a, I, V>(
    registry: &TenantTokenRegistry,
    headers: I,
    peer_addr: Option<SocketAddr>,
) -> Option<Arc<[u64]>>
where
    I: IntoIterator<Item = (&'a str, V)>,
    V: HeaderValue,
{
    if registry.is_empty() {
        return None;
    }
    SCRATCH.with(|scratch| {
        registry.resolve(
            &mut scratch.borrow_mut(),
            TokenInputs::new(headers).with_peer_addr(peer_addr),
        )
    })
}

/// Resolves the tenant context for one gRPC request.
///
/// Returns `None` when no token resolved, in which case the request carries no
/// tenant context and no allocation was made.
#[must_use]
pub fn resolve_grpc(
    registry: &TenantTokenRegistry,
    metadata: &MetadataMap,
    peer_addr: Option<SocketAddr>,
) -> Option<Arc<[u64]>> {
    if registry.is_empty() {
        return None;
    }
    let headers = metadata.iter().map(|kv| match kv {
        KeyAndValueRef::Ascii(key, value) => (key.as_str(), LazyMetadataValue::Ascii(value)),
        KeyAndValueRef::Binary(key, value) => (key.as_str(), LazyMetadataValue::Binary(value)),
    });
    SCRATCH.with(|scratch| {
        registry.resolve(
            &mut scratch.borrow_mut(),
            TokenInputs::new(headers).with_peer_addr(peer_addr),
        )
    })
}
