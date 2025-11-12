// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Canonical representation of OTLP AnyValue for comparison.

use crate::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value};
use std::collections::BTreeMap;

/// A canonical, comparable representation of an OTLP AnyValue.
///
/// This type implements total ordering and can be used in sorted collections
/// like BTreeMap and BTreeSet. Arrays preserve their order (user data),
/// but maps (KvList) are sorted by key for canonical comparison.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalValue {
    /// No value
    Empty,
    /// String value
    String(String),
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Double value - represented as bits for total ordering
    /// (NaN, -0.0, +0.0 are all distinct)
    Double(u64),
    /// Bytes value
    Bytes(Vec<u8>),
    /// Array value - preserves order (user data)
    Array(Vec<CanonicalValue>),
    /// Key-value list - sorted by key for canonical comparison
    KvList(BTreeMap<String, CanonicalValue>),
}

impl From<&AnyValue> for CanonicalValue {
    fn from(av: &AnyValue) -> Self {
        match &av.value {
            Some(any_value::Value::StringValue(s)) => CanonicalValue::String(s.clone()),
            Some(any_value::Value::BoolValue(b)) => CanonicalValue::Bool(*b),
            Some(any_value::Value::IntValue(i)) => CanonicalValue::Int(*i),
            Some(any_value::Value::DoubleValue(d)) => CanonicalValue::Double(d.to_bits()),
            Some(any_value::Value::BytesValue(b)) => CanonicalValue::Bytes(b.clone()),
            Some(any_value::Value::ArrayValue(arr)) => {
                CanonicalValue::Array(arr.values.iter().map(CanonicalValue::from).collect())
            }
            Some(any_value::Value::KvlistValue(kvlist)) => CanonicalValue::KvList(
                kvlist
                    .values
                    .iter()
                    .map(|kv| (kv.key.clone(), CanonicalValue::from(kv.value.as_ref())))
                    .collect(),
            ),
            None => CanonicalValue::Empty,
        }
    }
}

impl From<Option<&AnyValue>> for CanonicalValue {
    fn from(av: Option<&AnyValue>) -> Self {
        match av {
            Some(v) => CanonicalValue::from(v),
            None => CanonicalValue::Empty,
        }
    }
}

/// Convert a list of KeyValue pairs to a canonical sorted map
#[allow(dead_code)] // Used in future trace/metric implementations
pub fn canonical_attributes(attrs: &[KeyValue]) -> BTreeMap<String, CanonicalValue> {
    attrs
        .iter()
        .map(|kv| (kv.key.clone(), CanonicalValue::from(kv.value.as_ref())))
        .collect()
}

/// Compare two attribute lists in canonical order (sorted by key)
pub fn compare_attributes(a: &[KeyValue], b: &[KeyValue]) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    // Sort both by key and compare
    let mut a_sorted: Vec<_> = a.iter().collect();
    let mut b_sorted: Vec<_> = b.iter().collect();
    a_sorted.sort_by(|x, y| x.key.cmp(&y.key));
    b_sorted.sort_by(|x, y| x.key.cmp(&y.key));

    // Compare lengths first
    match a_sorted.len().cmp(&b_sorted.len()) {
        Ordering::Equal => {}
        other => return other,
    }

    // Compare each key-value pair
    for (x, y) in a_sorted.iter().zip(b_sorted.iter()) {
        match x.key.cmp(&y.key) {
            Ordering::Equal => {}
            other => return other,
        }
        // Compare values using CanonicalValue
        let x_val = CanonicalValue::from(x.value.as_ref());
        let y_val = CanonicalValue::from(y.value.as_ref());
        match x_val.cmp(&y_val) {
            Ordering::Equal => {}
            other => return other,
        }
    }

    Ordering::Equal
}
