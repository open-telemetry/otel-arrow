// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, Result};
use crate::otlp::ProtoBuffer;
use crate::otlp::common::BoundedBuf;
use crate::proto::consts::field_num::common::{
    ANY_VALUE_ARRAY_VALUE, ANY_VALUE_BOOL_VALUE, ANY_VALUE_BYTES_VALUE, ANY_VALUE_DOUBLE_VALUE,
    ANY_VALUE_INT_VALUE, ANY_VALUE_KVLIST_VALUE, ANY_VALUE_STRING_VALUE, ARRAY_VALUE_VALUES,
    KEY_VALUE_KEY, KEY_VALUE_LIST_VALUES, KEY_VALUE_VALUE,
};
use crate::proto::consts::wire_types;

/// A path segment into a serialized attribute value.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SerializedValuePathElement {
    /// A map key.
    Key(String),
    /// An array index.
    Index(usize),
}

/// Scalar value that can be written into a serialized attribute.
#[derive(Clone, Debug, PartialEq)]
pub enum SerializedAttributeScalarValue {
    /// Empty value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Bytes value.
    Bytes(Vec<u8>),
    /// Floating point value.
    Float(f64),
    /// Text value.
    Text(String),
    /// Integer value.
    Integer(i64),
}

/// Mutation to apply at a serialized attribute value path.
#[derive(Clone, Debug, PartialEq)]
pub enum SerializedValueMutation {
    /// Set the path to the supplied value.
    Set(SerializedAttributeScalarValue),
    /// Remove the value at the path.
    Remove,
}

/// A mutation paired with the serialized attribute path it targets.
#[derive(Clone, Debug, PartialEq)]
pub struct SerializedValuePathMutation<'a> {
    /// Path to mutate.
    pub path: &'a [SerializedValuePathElement],
    /// Mutation to apply at `path`.
    pub mutation: SerializedValueMutation,
}

/// Decode bytes from a serialized attribute into protobuf bytes value.
///
/// This should be used for values in the `ser` column of attributes and Log bodies.
pub fn proto_encode_cbor_bytes(input: &[u8], result_buf: &mut ProtoBuffer) -> Result<()> {
    let value = ciborium::from_reader::<ciborium::Value, &[u8]>(input)
        .map_err(|e| Error::InvalidSerializedAttributeBytes { source: e })?;

    proto_encode_cbor_value(&value, result_buf)?;

    Ok(())
}

/// Mutate serialized CBOR bytes and return the updated bytes.
///
/// Missing paths and type mismatches are treated as no-ops. For `Set`, missing map keys are
/// created when all parent containers exist. Array indices must already exist.
pub fn mutate_cbor_bytes(
    input: &[u8],
    path: &[SerializedValuePathElement],
    mutation: SerializedValueMutation,
) -> Result<Vec<u8>> {
    Ok(
        mutate_cbor_bytes_many(input, &[SerializedValuePathMutation { path, mutation }])?
            .unwrap_or_else(|| input.to_vec()),
    )
}

/// Mutate serialized CBOR bytes with multiple path mutations in one decode/encode pass.
///
/// Returns `Ok(Some(bytes))` when at least one mutation changed the value, and `Ok(None)` when all
/// mutations were no-ops.
pub fn mutate_cbor_bytes_many(
    input: &[u8],
    mutations: &[SerializedValuePathMutation<'_>],
) -> Result<Option<Vec<u8>>> {
    let mut value = ciborium::from_reader::<ciborium::Value, &[u8]>(input)
        .map_err(|e| Error::InvalidSerializedAttributeBytes { source: e })?;

    let mut changed = false;
    for mutation in mutations {
        changed |= mutate_cbor_value(&mut value, mutation.path, mutation.mutation.clone());
    }

    if changed {
        let mut output = Vec::new();
        ciborium::into_writer(&value, &mut output).map_err(|e| {
            Error::UnexpectedRecordBatchState {
                reason: format!("failed to encode serialized attribute bytes: {e}"),
            }
        })?;
        Ok(Some(output))
    } else {
        Ok(None)
    }
}

fn mutate_cbor_value(
    value: &mut ciborium::Value,
    path: &[SerializedValuePathElement],
    mutation: SerializedValueMutation,
) -> bool {
    let Some((head, tail)) = path.split_first() else {
        *value = match mutation {
            SerializedValueMutation::Set(value) => value.into(),
            SerializedValueMutation::Remove => ciborium::Value::Null,
        };
        return true;
    };

    if tail.is_empty() {
        return mutate_cbor_child(value, head, mutation);
    }

    match (value, head) {
        (ciborium::Value::Map(entries), SerializedValuePathElement::Key(key)) => entries
            .iter_mut()
            .find_map(|(entry_key, entry_value)| match entry_key {
                ciborium::Value::Text(entry_key) if entry_key == key => Some(entry_value),
                _ => None,
            })
            .is_some_and(|entry_value| mutate_cbor_value(entry_value, tail, mutation)),
        (ciborium::Value::Array(values), SerializedValuePathElement::Index(index)) => values
            .get_mut(*index)
            .is_some_and(|entry_value| mutate_cbor_value(entry_value, tail, mutation)),
        _ => false,
    }
}

fn mutate_cbor_child(
    value: &mut ciborium::Value,
    path_element: &SerializedValuePathElement,
    mutation: SerializedValueMutation,
) -> bool {
    match (value, path_element, mutation) {
        (
            ciborium::Value::Map(entries),
            SerializedValuePathElement::Key(key),
            SerializedValueMutation::Set(new_value),
        ) => {
            let new_value = new_value.into();
            if let Some((_, value)) = entries.iter_mut().find(|(entry_key, _)| {
                matches!(entry_key, ciborium::Value::Text(entry_key) if entry_key == key)
            }) {
                *value = new_value;
            } else {
                entries.push((ciborium::Value::Text(key.clone()), new_value));
            }
            true
        }
        (
            ciborium::Value::Map(entries),
            SerializedValuePathElement::Key(key),
            SerializedValueMutation::Remove,
        ) => {
            let before = entries.len();
            entries.retain(|(entry_key, _)| {
                !matches!(entry_key, ciborium::Value::Text(entry_key) if entry_key == key)
            });
            entries.len() != before
        }
        (
            ciborium::Value::Array(values),
            SerializedValuePathElement::Index(index),
            SerializedValueMutation::Set(new_value),
        ) => {
            if let Some(value) = values.get_mut(*index) {
                *value = new_value.into();
                true
            } else {
                false
            }
        }
        (
            ciborium::Value::Array(values),
            SerializedValuePathElement::Index(index),
            SerializedValueMutation::Remove,
        ) if *index < values.len() => {
            _ = values.remove(*index);
            true
        }
        (
            ciborium::Value::Array(_),
            SerializedValuePathElement::Index(_),
            SerializedValueMutation::Remove,
        ) => false,
        _ => false,
    }
}

impl From<SerializedAttributeScalarValue> for ciborium::Value {
    fn from(value: SerializedAttributeScalarValue) -> Self {
        match value {
            SerializedAttributeScalarValue::Null => Self::Null,
            SerializedAttributeScalarValue::Bool(value) => Self::Bool(value),
            SerializedAttributeScalarValue::Bytes(value) => Self::Bytes(value),
            SerializedAttributeScalarValue::Float(value) => Self::Float(value),
            SerializedAttributeScalarValue::Text(value) => Self::Text(value),
            SerializedAttributeScalarValue::Integer(value) => Self::Integer(value.into()),
        }
    }
}

fn proto_encode_cbor_value(value: &ciborium::Value, result_buf: &mut ProtoBuffer) -> Result<()> {
    match value {
        ciborium::Value::Null => {
            // do nothing, it's an empty value
        }
        ciborium::Value::Bool(bool_val) => {
            result_buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT)?;
            result_buf.encode_varint(*bool_val as u64)?;
        }
        ciborium::Value::Bytes(bytes_val) => {
            result_buf.encode_bytes(ANY_VALUE_BYTES_VALUE, bytes_val)?;
        }
        ciborium::Value::Float(float_val) => {
            result_buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64)?;
            result_buf.extend_from_slice(&float_val.to_le_bytes())?;
        }
        ciborium::Value::Text(str_val) => {
            result_buf.encode_string(ANY_VALUE_STRING_VALUE, str_val)?;
        }
        ciborium::Value::Integer(int_val) => {
            let int_val: i64 = (*int_val)
                .try_into()
                .map_err(|e| Error::InvalidSerializedIntAttributeValue { source: e })?;
            result_buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT)?;
            result_buf.encode_varint(int_val as u64)?;
        }
        ciborium::Value::Array(list_val) => {
            result_buf.encode_len_delimited(ANY_VALUE_ARRAY_VALUE, |result_buf| {
                proto_encode_array_values(list_val, result_buf)
            })?;
        }
        ciborium::Value::Map(kv_list_val) => {
            result_buf.encode_len_delimited(ANY_VALUE_KVLIST_VALUE, |result_buf| {
                proto_encode_cbor_kv_list(kv_list_val, result_buf)
            })?;
        }
        other => {
            return Err(Error::UnsupportedSerializedAttributeValue {
                actual: other.clone(),
            });
        }
    }

    Ok(())
}

fn proto_encode_array_values(
    list_val: &[ciborium::Value],
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    for v in list_val {
        result_buf.encode_len_delimited(ARRAY_VALUE_VALUES, |result_buf| {
            proto_encode_cbor_value(v, result_buf)
        })?;
    }

    Ok(())
}

fn proto_encode_cbor_kv_list(
    kv_list: &[(ciborium::Value, ciborium::Value)],
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    for (k, v) in kv_list {
        result_buf.encode_len_delimited(KEY_VALUE_LIST_VALUES, |result_buf| {
            proto_encode_cbor_kv(k, v, result_buf)
        })?;
    }

    Ok(())
}

fn proto_encode_cbor_kv(
    key: &ciborium::Value,
    value: &ciborium::Value,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    match key {
        ciborium::Value::Text(key_str) => {
            result_buf.encode_string(KEY_VALUE_KEY, key_str)?;
        }
        ciborium::Value::Null => {
            // empty key
        }
        other => {
            return Err(Error::InvalidSerializedMapKeyType {
                actual: other.clone(),
            });
        }
    }

    result_buf.encode_len_delimited(KEY_VALUE_VALUE, |result_buf| {
        proto_encode_cbor_value(value, result_buf)
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(value: &ciborium::Value) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::into_writer(value, &mut bytes).expect("encode cbor");
        bytes
    }

    fn decode(bytes: &[u8]) -> ciborium::Value {
        ciborium::from_reader(bytes).expect("decode cbor")
    }

    #[test]
    fn set_creates_missing_leaf_map_key() {
        let input = encode(&ciborium::Value::Map(vec![(
            ciborium::Value::Text("child".into()),
            ciborium::Value::Map(vec![]),
        )]));

        let output = mutate_cbor_bytes(
            &input,
            &[
                SerializedValuePathElement::Key("child".into()),
                SerializedValuePathElement::Key("name".into()),
            ],
            SerializedValueMutation::Set(SerializedAttributeScalarValue::Text("after".into())),
        )
        .expect("mutate");

        assert_eq!(
            decode(&output),
            ciborium::Value::Map(vec![(
                ciborium::Value::Text("child".into()),
                ciborium::Value::Map(vec![(
                    ciborium::Value::Text("name".into()),
                    ciborium::Value::Text("after".into()),
                )]),
            )])
        );
    }

    #[test]
    fn missing_intermediate_path_is_noop() {
        let input = encode(&ciborium::Value::Map(vec![]));
        let output = mutate_cbor_bytes_many(
            &input,
            &[SerializedValuePathMutation {
                path: &[
                    SerializedValuePathElement::Key("missing".into()),
                    SerializedValuePathElement::Key("name".into()),
                ],
                mutation: SerializedValueMutation::Set(SerializedAttributeScalarValue::Text(
                    "after".into(),
                )),
            }],
        )
        .expect("mutate");

        assert_eq!(output, None);
    }

    #[test]
    fn wrong_container_and_array_oob_are_noops() {
        let input = encode(&ciborium::Value::Map(vec![(
            ciborium::Value::Text("items".into()),
            ciborium::Value::Array(vec![]),
        )]));

        assert_eq!(
            mutate_cbor_bytes_many(
                &input,
                &[SerializedValuePathMutation {
                    path: &[SerializedValuePathElement::Index(0)],
                    mutation: SerializedValueMutation::Set(SerializedAttributeScalarValue::Text(
                        "after".into(),
                    )),
                }],
            )
            .expect("mutate"),
            None
        );
        assert_eq!(
            mutate_cbor_bytes_many(
                &input,
                &[SerializedValuePathMutation {
                    path: &[
                        SerializedValuePathElement::Key("items".into()),
                        SerializedValuePathElement::Index(1),
                    ],
                    mutation: SerializedValueMutation::Set(SerializedAttributeScalarValue::Text(
                        "after".into(),
                    )),
                }],
            )
            .expect("mutate"),
            None
        );
    }

    #[test]
    fn remove_map_key_and_array_element() {
        let input = encode(&ciborium::Value::Map(vec![
            (
                ciborium::Value::Text("drop".into()),
                ciborium::Value::Text("value".into()),
            ),
            (
                ciborium::Value::Text("items".into()),
                ciborium::Value::Array(vec![
                    ciborium::Value::Integer(1.into()),
                    ciborium::Value::Integer(2.into()),
                ]),
            ),
        ]));

        let output = mutate_cbor_bytes_many(
            &input,
            &[
                SerializedValuePathMutation {
                    path: &[SerializedValuePathElement::Key("drop".into())],
                    mutation: SerializedValueMutation::Remove,
                },
                SerializedValuePathMutation {
                    path: &[
                        SerializedValuePathElement::Key("items".into()),
                        SerializedValuePathElement::Index(0),
                    ],
                    mutation: SerializedValueMutation::Remove,
                },
            ],
        )
        .expect("mutate")
        .expect("changed");

        assert_eq!(
            decode(&output),
            ciborium::Value::Map(vec![(
                ciborium::Value::Text("items".into()),
                ciborium::Value::Array(vec![ciborium::Value::Integer(2.into())]),
            )])
        );
    }

    #[test]
    fn corrupt_cbor_returns_error() {
        let err = mutate_cbor_bytes(
            b"not cbor",
            &[SerializedValuePathElement::Key("name".into())],
            SerializedValueMutation::Set(SerializedAttributeScalarValue::Text("after".into())),
        )
        .expect_err("invalid cbor");

        assert!(matches!(err, Error::InvalidSerializedAttributeBytes { .. }));
    }

    #[test]
    fn proto_encode_accepts_negative_cbor_integer() {
        let input = encode(&ciborium::Value::Integer((-1).into()));
        let mut output = ProtoBuffer::default();

        proto_encode_cbor_bytes(&input, &mut output).expect("negative int encodes");
        assert_eq!(
            output.as_ref(),
            &[
                0x18, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01
            ]
        );
    }
}
