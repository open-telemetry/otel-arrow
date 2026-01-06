// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Canonical comparison for OTLP messages.
//!
//! Note! When encoding OTLP -> OTAP -> OTLP, None-valued fields may
//! end up as default-valued fields. We canonicalize default to None.
//! We are explicitly ignoring the proto3 syntax recommendations about
//! field presence for protobuf implementations. In the OpenTelemetry
//! framing, this is an irrelevant distinction.

use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, any_value};
use crate::proto::opentelemetry::resource::v1::Resource;
use prost::Message;
use std::collections::BTreeSet;

/// Helper to recursively canonicalize AnyValue.
pub(crate) fn canonicalize_any_value(av: &mut AnyValue) {
    if let Some(value) = &mut av.value {
        match value {
            any_value::Value::ArrayValue(arr) => {
                // Arrays preserve order but canonicalize elements
                for v in &mut arr.values {
                    canonicalize_any_value(v);
                }
            }
            any_value::Value::KvlistValue(kvlist) => {
                // Canonicalize the key-value list using the generic canonicalizer
                canonicalize_vec(&mut kvlist.values, |kv| {
                    if let Some(v) = &mut kv.value {
                        canonicalize_any_value(v);
                    }
                });
            }
            _ => {}
        }
    }
}

/// Canonicalize a slice of protobuf messages by encoding them,
/// sorting, and reconstructing in the canonical order.
pub(crate) fn canonicalize_vec<T, F>(vec: &mut Vec<T>, mut canonicalize_fn: F)
where
    T: Message + Default + Clone,
    F: FnMut(&mut T),
{
    if vec.is_empty() {
        return;
    }

    // Clone, canonicalize, and encode each element
    let encoded_set: BTreeSet<Vec<u8>> = vec
        .iter()
        .map(|item| {
            let mut cloned = item.clone();
            canonicalize_fn(&mut cloned);
            let mut buf = Vec::new();
            cloned.encode(&mut buf).expect("encoding should not fail");
            buf
        })
        .collect();

    // Decode back in canonical order
    vec.clear();
    for bytes in encoded_set {
        let decoded = T::decode(bytes.as_slice()).expect("decoding should not fail");
        vec.push(decoded);
    }
}

/// Replaces all zeros with None.
pub(crate) fn canonicalize_idvec(id: &mut Vec<u8>) {
    if id.iter().all(|&b| b == 0) {
        id.clear();
    }
}

/// Canonicalize a messqage into bytes.  The data must have been
/// canonicalized in-place before calling this method.
pub(crate) fn canonicalize_message<T>(msg: &T) -> Vec<u8>
where
    T: Message,
{
    let mut buf = Vec::new();
    msg.encode(&mut buf).expect("encoding should not fail");
    buf
}

/// Canonicalizes a resource, sets it to None if equivalent to the default.
pub(crate) fn canonicalize_resource(resource_opt: &mut Option<Resource>) {
    let erase = match resource_opt {
        None => true,
        Some(resource) => {
            canonicalize_vec(&mut resource.attributes, |attr| {
                if let Some(value) = &mut attr.value {
                    canonicalize_any_value(value);
                }
            });
            *resource == Resource::default()
        }
    };
    if erase {
        *resource_opt = None;
    }
}

/// Canonicalizes a scope, sets it to None if equivalent to the default.
pub(crate) fn canonicalize_scope(scope_opt: &mut Option<InstrumentationScope>) {
    let erase = match scope_opt {
        None => true,
        Some(scope) => {
            canonicalize_vec(&mut scope.attributes, |attr| {
                if let Some(value) = &mut attr.value {
                    canonicalize_any_value(value);
                }
            });
            *scope == InstrumentationScope::default()
        }
    };
    if erase {
        *scope_opt = None;
    }
}

/// Generic equivalence assertion for any OTLP signal type.
pub(crate) fn assert_equivalent<T, F, G>(
    left: &[T],
    right: &[T],
    split_fn: F,
    canonicalize_fn: G,
    message_name: &str,
) where
    T: Message + Clone + std::fmt::Debug + PartialEq,
    F: Fn(&T) -> Vec<T>,
    G: Fn(&mut T),
{
    // Split into singletons from all messages in the slices
    let mut left_singletons: Vec<T> = left.iter().flat_map(&split_fn).collect();
    let mut right_singletons: Vec<T> = right.iter().flat_map(&split_fn).collect();

    // Canonicalize each singleton
    for singleton in &mut left_singletons {
        canonicalize_fn(singleton);
    }
    for singleton in &mut right_singletons {
        canonicalize_fn(singleton);
    }

    // Encode to bytes and collect into BTreeSets
    let left_set: BTreeSet<Vec<u8>> = left_singletons.iter().map(canonicalize_message).collect();
    let right_set: BTreeSet<Vec<u8>> = right_singletons.iter().map(canonicalize_message).collect();

    // Use pretty_assertions for nice diff output
    if left_set == right_set {
        return;
    }
    pretty_assertions::assert_eq!(
        left_singletons,
        right_singletons,
        "{} not equivalent",
        message_name
    );
}
