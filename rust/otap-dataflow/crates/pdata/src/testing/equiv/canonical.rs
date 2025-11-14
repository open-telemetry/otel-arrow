// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Canonical representation of OTLP AnyValue for comparison.

use crate::proto::opentelemetry::common::v1::any_value;
use prost::Message;
use std::collections::BTreeSet;

// Helper to recursively canonicalize AnyValue.
pub(crate) fn canonicalize_any_value(av: &mut crate::proto::opentelemetry::common::v1::AnyValue) {
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

/// Canonicalize a messqage into bytes.  The data must have been
/// canonicalized in-place before calling this method.
pub(crate) fn canonicalize_message<T>(msg: T) -> Vec<u8>
where
    T: Message,
{
    let mut buf = Vec::new();
    msg.encode(&mut buf).expect("encoding should not fail");
    buf
}

/// Generic equivalence assertion for any OTLP signal type.
pub(crate) fn assert_equivalent<T, F, G>(
    left: &[T],
    right: &[T],
    split_fn: F,
    canonicalize_fn: G,
    message_name: &str,
) where
    T: Message + Clone,
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
    let left_set: BTreeSet<Vec<u8>> = left_singletons
        .into_iter()
        .map(canonicalize_message)
        .collect();
    let right_set: BTreeSet<Vec<u8>> = right_singletons
        .into_iter()
        .map(canonicalize_message)
        .collect();

    // Use pretty_assertions for nice diff output
    pretty_assertions::assert_eq!(left_set, right_set, "{} not equivalent", message_name);
}
