// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::io::Write;

use otap_df_pdata::views::common::{AnyValueView, AttributeView, ValueType};
use serde::ser::{Error as SerError, SerializeMap, SerializeSeq, Serializer};
use serde_cbor::ser::IoWrite;

use crate::encoder::error::{Error, Result};

/// Adapter for serializing AnyValueView using Serde
struct AnyValueSerializerWrapper<T>(pub T);

impl<'a, T> serde::Serialize for AnyValueSerializerWrapper<T>
where
    T: AnyValueView<'a> + 'a,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_any_value::<T, S>(&self.0, serializer)
    }
}

pub fn serialize_any_values<'a, I, T, W>(source: I, writer: W) -> Result<()>
where
    I: Iterator<Item = T>,
    T: AnyValueView<'a> + 'a,
    W: Write,
{
    let mut serializer = serde_cbor::Serializer::new(IoWrite::new(writer));
    let mut seq = serializer.serialize_seq(None)?;

    for val in source {
        seq.serialize_element(&AnyValueSerializerWrapper(val))?;
    }
    SerializeSeq::end(seq)?;

    Ok(())
}

/// serialize the passed `AnyValue` view as bytes in cbor representation
fn serialize_any_value<'a, T, S>(source: &T, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    T: AnyValueView<'a> + 'a,
    S: Serializer,
{
    match source.value_type() {
        ValueType::String => {
            let s_bytes = source.as_string().expect("expected string");
            let s_str = simdutf8::basic::from_utf8(s_bytes)
                .map_err(|e| S::Error::custom(format!("Invalid UTF-8: {e}")))?;
            serializer.serialize_str(s_str)
        }
        ValueType::Bool => {
            let b = source.as_bool().expect("expected bool");
            serializer.serialize_bool(b)
        }
        ValueType::Int64 => {
            let i = source.as_int64().expect("expected int64");
            serializer.serialize_i64(i)
        }
        ValueType::Double => {
            let f = source.as_double().expect("expected f64");
            serializer.serialize_f64(f)
        }
        ValueType::Bytes => {
            let b = source.as_bytes().expect("expected bytes");
            serializer.serialize_bytes(b)
        }
        ValueType::Empty => serializer.serialize_none(),
        ValueType::Array => {
            let children = source.as_array().expect("expected array");
            let mut seq = serializer.serialize_seq(None)?;
            for child in children {
                seq.serialize_element(&AnyValueSerializerWrapper(child))?;
            }
            seq.end()
        }
        ValueType::KeyValueList => {
            let kvlist = source.as_kvlist().expect("expected kvlist");
            let mut map = serializer.serialize_map(None)?;
            for kv in kvlist {
                let key = kv.key();
                let key_str = simdutf8::basic::from_utf8(key)
                    .map_err(|e| S::Error::custom(format!("Invalid UTF-8: {e}")))?;
                match kv.value() {
                    Some(v) => map.serialize_entry(&key_str, &AnyValueSerializerWrapper(v))?,
                    None => map.serialize_entry(&key_str, &Option::<()>::None)?,
                }
            }
            map.end()
        }
    }
}

/// serialize the passed list of key values as bytes in cbor representation
pub fn serialize_kv_list<I, T, W>(source: I, writer: W) -> Result<()>
where
    I: Iterator<Item = T>,
    T: AttributeView,
    W: Write,
{
    let mut serializer = serde_cbor::Serializer::new(IoWrite::new(writer));
    let mut map = serializer.serialize_map(None)?;
    for kv in source {
        let key = kv.key();
        let key_str = simdutf8::basic::from_utf8(key).map_err(|e| Error::CborError {
            error: format!("Invalid UTF-8: {e}"),
        })?;
        match kv.value() {
            Some(v) => map.serialize_entry(&key_str, &AnyValueSerializerWrapper(v))?,
            None => map.serialize_entry(&key_str, &Option::<()>::None)?,
        }
    }
    SerializeMap::end(map)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use otap_df_pdata::views::otlp::proto::common::{ObjAny, ObjKeyValue};
    use otap_df_pdata::views::otlp::proto::wrappers::Wraps;
    use otel_arrow_rust::otlp::ProtoBuffer;
    use otel_arrow_rust::{
        otlp::attributes::cbor::proto_encode_cbor_bytes,
        proto::opentelemetry::common::v1::{
            AnyValue, ArrayValue, KeyValue, KeyValueList, any_value,
        },
    };
    use prost::Message;

    #[test]
    fn test_round_trip_anyvals() {
        let test_cases = vec![
            (AnyValue::new_bool(true), any_value::Value::BoolValue(true)),
            (AnyValue::new_int(42), any_value::Value::IntValue(42)),
            (
                AnyValue::new_double(2.0),
                any_value::Value::DoubleValue(2.0),
            ),
            (
                AnyValue::new_string("hello"),
                any_value::Value::StringValue("hello".to_string()),
            ),
            (
                AnyValue::new_bytes(vec![1, 2, 3]),
                any_value::Value::BytesValue(vec![1, 2, 3]),
            ),
            (
                AnyValue::new_array(vec![AnyValue::new_bool(false), AnyValue::new_int(1)]),
                any_value::Value::ArrayValue(ArrayValue::new(vec![
                    AnyValue {
                        value: Some(any_value::Value::BoolValue(false)),
                    },
                    AnyValue {
                        value: Some(any_value::Value::IntValue(1)),
                    },
                ])),
            ),
            (
                AnyValue::new_kvlist(vec![
                    KeyValue::new("foo", AnyValue::new_string("bar")),
                    KeyValue::new("baz", AnyValue::new_int(123)),
                ]),
                any_value::Value::KvlistValue(KeyValueList {
                    values: vec![
                        KeyValue {
                            key: "foo".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue("bar".to_string())),
                            }),
                        },
                        KeyValue {
                            key: "baz".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::IntValue(123)),
                            }),
                        },
                    ],
                }),
            ),
        ];

        for (source, expected) in test_cases {
            let mut serialized_val = vec![];
            serialize_any_values(vec![ObjAny::new(&source)].into_iter(), &mut serialized_val)
                .unwrap();

            let mut proto_buffer = ProtoBuffer::new();
            proto_encode_cbor_bytes(&serialized_val, &mut proto_buffer).unwrap();
            let result = AnyValue::decode(proto_buffer.as_ref()).unwrap();
            assert_eq!(
                result.value,
                Some(any_value::Value::ArrayValue(ArrayValue::new(vec![
                    AnyValue {
                        value: Some(expected)
                    }
                ])))
            );
        }
    }

    #[test]
    fn test_round_trip_keyvalues() {
        let test_cases = vec![
            (
                vec![
                    KeyValue::new("foo", AnyValue::new_string("bar")),
                    KeyValue::new("baz", AnyValue::new_int(123)),
                ],
                vec![
                    KeyValue {
                        key: "foo".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("bar".to_string())),
                        }),
                    },
                    KeyValue {
                        key: "baz".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::IntValue(123)),
                        }),
                    },
                ],
            ),
            (
                vec![
                    KeyValue {
                        key: "empty".to_string(),
                        value: None,
                    },
                    KeyValue::new("bool", AnyValue::new_bool(false)),
                ],
                vec![
                    KeyValue {
                        key: "empty".to_string(),
                        value: Some(AnyValue { value: None }),
                    },
                    KeyValue {
                        key: "bool".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::BoolValue(false)),
                        }),
                    },
                ],
            ),
        ];

        for (source, expected) in test_cases {
            let mut serialized_val = vec![];
            serialize_kv_list(
                source.iter().map(|kv| {
                    ObjKeyValue::new(kv.key.as_str(), kv.value.as_ref().map(ObjAny::new))
                }),
                &mut serialized_val,
            )
            .unwrap();

            let mut proto_buffer = ProtoBuffer::new();
            proto_encode_cbor_bytes(&serialized_val, &mut proto_buffer).unwrap();
            let result = AnyValue::decode(proto_buffer.as_ref()).unwrap();
            assert_eq!(
                result.value,
                Some(any_value::Value::KvlistValue(KeyValueList {
                    values: expected.clone()
                }))
            );
        }
    }
}
