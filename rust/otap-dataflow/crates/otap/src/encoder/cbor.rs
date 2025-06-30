// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::io::Write;

use ciborium::{Value, value::Integer};
use otap_df_pdata_views::views::common::{AnyValueView, AttributeView, ValueType};

use crate::encoder::error::Result;

/// serialize the passed `AnyValue` view as bytes in cbor representation
pub fn serialize_any_values<'a, I, T, W>(source: I, writer: W) -> Result<()>
where
    I: Iterator<Item = T>,
    T: AnyValueView<'a> + 'a,
    W: Write,
{
    let value = Value::Array(source.map(|val| cbor_value_from_any_value(&val)).collect());
    ciborium::into_writer(&value, writer)?;

    Ok(())
}

/// serialize the passed list of key values as bytes in cbor representation
pub fn serialize_kv_list<I, T, W>(source: I, writer: W) -> Result<()>
where
    I: Iterator<Item = T>,
    T: AttributeView,
    W: Write,
{
    let value = cbor_value_from_kvlist(source);
    ciborium::into_writer(&value, writer)?;

    Ok(())
}

/// convert the any value to cbor `Value`
fn cbor_value_from_any_value<'a, T>(source: &T) -> Value
where
    T: AnyValueView<'a>,
{
    match source.value_type() {
        ValueType::String => Value::Text(source.as_string().expect("body is string").to_string()),
        ValueType::Bool => Value::Bool(source.as_bool().expect("body is bool")),
        ValueType::Int64 => Value::Integer(Integer::from(source.as_int64().expect("body is int"))),
        ValueType::Double => Value::Float(source.as_double().expect("body is float")),
        ValueType::Bytes => Value::Bytes(source.as_bytes().expect("body is bytes").to_vec()),
        ValueType::Array => {
            let children = source
                .as_array()
                .expect("body is array")
                .map(|val| cbor_value_from_any_value(&val))
                .collect();
            Value::Array(children)
        }
        ValueType::KeyValueList => {
            cbor_value_from_kvlist(source.as_kvlist().expect("body is kvlist"))
        }
        ValueType::Empty => Value::Null,
    }
}

/// convert the key-value pairs into cbor `Value`
fn cbor_value_from_kvlist<I, T>(source: I) -> Value
where
    I: Iterator<Item = T>,
    T: AttributeView,
{
    let children = source
        .map(|kv| {
            (
                Value::Text(kv.key().to_string()),
                match kv.value() {
                    None => Value::Null,
                    Some(val) => cbor_value_from_any_value(&val),
                },
            )
        })
        .collect();

    Value::Map(children)
}

#[cfg(test)]
mod test {
    use super::*;
    use otap_df_pdata_views::otlp::proto::common::{ObjAny, ObjKeyValue};
    use otel_arrow_rust::{
        otlp::attributes::cbor::decode_pcommon_val,
        proto::opentelemetry::common::v1::{
            AnyValue, ArrayValue, KeyValue, KeyValueList, any_value,
        },
    };

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
                AnyValue::new_array(vec![AnyValue::new_bool(false), AnyValue::new_int(-1)]),
                any_value::Value::ArrayValue(ArrayValue::new(vec![
                    AnyValue {
                        value: Some(any_value::Value::BoolValue(false)),
                    },
                    AnyValue {
                        value: Some(any_value::Value::IntValue(-1)),
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
            serialize_any_values(vec![ObjAny(&source)].into_iter(), &mut serialized_val).unwrap();

            let result = decode_pcommon_val(&serialized_val).unwrap();
            assert_eq!(
                result,
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
                source
                    .iter()
                    .map(|kv| ObjKeyValue::new(kv.key.as_str(), kv.value.as_ref().map(ObjAny))),
                &mut serialized_val,
            )
            .unwrap();

            let result = decode_pcommon_val(&serialized_val).unwrap();
            assert_eq!(
                result,
                Some(any_value::Value::KvlistValue(KeyValueList {
                    values: expected.clone()
                }))
            );
        }
    }
}
