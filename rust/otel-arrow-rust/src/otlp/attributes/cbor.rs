// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::error::{self, Error, Result};
use crate::proto::opentelemetry::common::v1::any_value::Value;
use crate::proto::opentelemetry::common::v1::{AnyValue, ArrayValue, KeyValue, KeyValueList};
use snafu::ResultExt;

/// Decode bytes from a serialized attribute into pcommon value.
/// This should be used for values in the `ser` column of attributes
/// and Log bodies.
pub fn decode_pcommon_val(input: &[u8]) -> Result<Option<Value>> {
    let decoded_val = ciborium::from_reader::<ciborium::Value, &[u8]>(input)
        .context(error::InvalidSerializedAttributeBytesSnafu)?;

    MaybeValue::try_from(decoded_val).map(Into::into)
}

// `MaybeValue` is a thin wrapper around `Option<Value>`. We use this so we to
// avoid violating the coherence rule when implementing TryFrom.
struct MaybeValue(Option<Value>);

impl From<MaybeValue> for Option<Value> {
    fn from(value: MaybeValue) -> Self {
        value.0
    }
}

impl TryFrom<ciborium::Value> for MaybeValue {
    type Error = Error;

    fn try_from(value: ciborium::Value) -> Result<Self> {
        let val = match value {
            ciborium::Value::Null => None,
            ciborium::Value::Text(string_val) => Some(Value::StringValue(string_val)),
            ciborium::Value::Float(double_val) => Some(Value::DoubleValue(double_val)),
            ciborium::Value::Bytes(bytes_val) => Some(Value::BytesValue(bytes_val)),
            ciborium::Value::Bool(bool_val) => Some(Value::BoolValue(bool_val)),
            ciborium::Value::Integer(int_val) => Some(Value::IntValue(
                int_val
                    .try_into()
                    .context(error::InvalidSerializedIntAttributeValueSnafu)?,
            )),
            ciborium::Value::Array(array_vals) => {
                let vals: Result<Vec<_>> = array_vals
                    .into_iter()
                    .map(|element| match Self::try_from(element) {
                        Ok(val) => Ok(AnyValue { value: val.into() }),
                        Err(e) => Err(e),
                    })
                    .collect();

                Some(Value::ArrayValue(ArrayValue { values: vals? }))
            }
            ciborium::Value::Map(map_vals) => {
                let kvs: Result<Vec<_>> = map_vals
                    .into_iter()
                    .map(|(k, v)| {
                        if let ciborium::Value::Text(key) = k {
                            match Self::try_from(v) {
                                Ok(val) => Ok(KeyValue::new(key, AnyValue { value: val.into() })),
                                Err(e) => Err(e),
                            }
                        } else {
                            error::InvalidSerializedMapKeyTypeSnafu { actual: k }.fail()
                        }
                    })
                    .collect();

                Some(Value::KvlistValue(KeyValueList::new(kvs?)))
            }

            other => {
                return error::UnsupportedSerializedAttributeValueSnafu { actual: other }.fail();
            }
        };

        Ok(Self(val))
    }
}
