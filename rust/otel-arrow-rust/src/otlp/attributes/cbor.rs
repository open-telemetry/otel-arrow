// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::error::{self, Result};
use crate::otlp::ProtoBuffer;
use crate::proto::consts::field_num::common::{
    ANY_VALUE_ARRAY_VALUE, ANY_VALUE_BOOL_VALUE, ANY_VALUE_BYTES_VALUE, ANY_VALUE_DOUBLE_VALUE,
    ANY_VALUE_INT_VALUE, ANY_VALUE_KVLIST_VALUE, ANY_VALUE_STRING_VALUE, ARRAY_VALUE_VALUES,
    KEY_VALUE_KEY, KEY_VALUE_LIST_VALUES, KEY_VALUE_VALUE,
};
use crate::proto::consts::wire_types;
use crate::proto_encode_len_delimited_unknown_size;
use snafu::ResultExt;

/// Decode bytes from a serialized attribute into protobuf bytes value.
///
/// This should be used for values in the `ser` column of attributes and Log bodies.
pub fn proto_encode_cbor_bytes(input: &[u8], result_buf: &mut ProtoBuffer) -> Result<()> {
    let value = ciborium::from_reader::<ciborium::Value, &[u8]>(input)
        .context(error::InvalidSerializedAttributeBytesSnafu)?;

    proto_encode_cbor_value(&value, result_buf)?;

    Ok(())
}

fn proto_encode_cbor_value(value: &ciborium::Value, result_buf: &mut ProtoBuffer) -> Result<()> {
    match value {
        ciborium::Value::Null => {
            // do nothing, it's an empty value
        }
        ciborium::Value::Bool(bool_val) => {
            result_buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT);
            result_buf.encode_varint(*bool_val as u64);
        }
        ciborium::Value::Bytes(bytes_val) => {
            result_buf.encode_bytes(ANY_VALUE_BYTES_VALUE, bytes_val);
        }
        ciborium::Value::Float(float_val) => {
            result_buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64);
            result_buf.extend_from_slice(&float_val.to_le_bytes());
        }
        ciborium::Value::Text(str_val) => {
            result_buf.encode_string(ANY_VALUE_STRING_VALUE, str_val);
        }
        ciborium::Value::Integer(int_val) => {
            let int_val: u64 = (*int_val)
                .try_into()
                .context(error::InvalidSerializedIntAttributeValueSnafu)?;
            result_buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
            result_buf.encode_varint(int_val);
        }
        ciborium::Value::Array(list_val) => {
            proto_encode_len_delimited_unknown_size!(
                ANY_VALUE_ARRAY_VALUE,
                proto_encode_array_values(list_val, result_buf)?,
                result_buf
            );
        }
        ciborium::Value::Map(kv_list_val) => {
            proto_encode_len_delimited_unknown_size!(
                ANY_VALUE_KVLIST_VALUE,
                proto_encode_cbor_kv_list(kv_list_val, result_buf)?,
                result_buf
            );
        }
        other => {
            return error::UnsupportedSerializedAttributeValueSnafu {
                actual: other.clone(),
            }
            .fail();
        }
    }

    Ok(())
}

fn proto_encode_array_values(
    list_val: &[ciborium::Value],
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    for v in list_val {
        proto_encode_len_delimited_unknown_size!(
            ARRAY_VALUE_VALUES,
            proto_encode_cbor_value(v, result_buf)?,
            result_buf
        );
    }

    Ok(())
}

fn proto_encode_cbor_kv_list(
    kv_list: &[(ciborium::Value, ciborium::Value)],
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    for (k, v) in kv_list {
        proto_encode_len_delimited_unknown_size!(
            KEY_VALUE_LIST_VALUES,
            proto_encode_cbor_kv(k, v, result_buf)?,
            result_buf
        );
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
            result_buf.encode_string(KEY_VALUE_KEY, key_str);
        }
        ciborium::Value::Null => {
            // empty key
        }
        other => {
            return error::InvalidSerializedMapKeyTypeSnafu {
                actual: other.clone(),
            }
            .fail();
        }
    }

    proto_encode_len_delimited_unknown_size!(
        KEY_VALUE_VALUE,
        proto_encode_cbor_value(value, result_buf)?,
        result_buf
    );

    Ok(())
}
