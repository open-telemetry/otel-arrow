// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{ArrowPrimitiveType, PrimitiveArray, RecordBatch, StringArray};
use arrow::datatypes::{UInt16Type, UInt32Type};
use num_enum::TryFromPrimitive;
use prost::encoding::{encoded_len_varint, key_len};

use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor, get_required_array};
use crate::error::{Error, Result};
use crate::otlp::{
    attributes::cbor::proto_encode_cbor_bytes,
    common::{AnyValueArrays, BoundedBuf, ProtoBuffer},
};
use crate::proto::consts::field_num::common::{
    ANY_VALUE_BOOL_VALUE, ANY_VALUE_BYTES_VALUE, ANY_VALUE_DOUBLE_VALUE, ANY_VALUE_INT_VALUE,
    ANY_VALUE_STRING_VALUE, KEY_VALUE_KEY, KEY_VALUE_VALUE,
};
use crate::proto::consts::wire_types;
use crate::schema::consts;

/// Common methods for Key-Value list and Array values.
pub mod cbor;

/// Common methods for decoding OTAP/OTLP data.
pub mod decoder;

/// Common methods for OTAP parent ID columns encoding/optimization.
pub mod parent_id;

/// OTAP conventional numbering for fields.
#[derive(Copy, Clone, Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum AttributeValueType {
    Empty = 0,
    Str = 1,
    Int = 2,
    Double = 3,
    Bool = 4,
    Map = 5,
    Slice = 6,
    Bytes = 7,
}

pub(crate) type Attribute16Arrays<'a> = AttributeArrays<'a, UInt16Type>;
pub(crate) type Attribute32Arrays<'a> = AttributeArrays<'a, UInt32Type>;

pub(crate) struct AttributeArrays<'a, T: ArrowPrimitiveType> {
    pub parent_id: MaybeDictArrayAccessor<'a, PrimitiveArray<T>>,
    pub attr_key: MaybeDictArrayAccessor<'a, StringArray>,
    pub anyval_arrays: AnyValueArrays<'a>,
}

impl<'a, T> TryFrom<&'a RecordBatch> for AttributeArrays<'a, T>
where
    T: ArrowPrimitiveType,
{
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let parent_id = MaybeDictArrayAccessor::<PrimitiveArray<T>>::try_new(get_required_array(
            rb,
            consts::PARENT_ID,
        )?)?;

        let key =
            rb.column_by_name(consts::ATTRIBUTE_KEY)
                .ok_or_else(|| Error::ColumnNotFound {
                    name: consts::ATTRIBUTE_KEY.into(),
                })?;
        let attr_key = MaybeDictArrayAccessor::<StringArray>::try_new(key)?;

        let anyval_arrays = AnyValueArrays::try_from(rb)?;

        Ok(Self {
            parent_id,
            attr_key,
            anyval_arrays,
        })
    }
}

/// Encode the attribute at `index` as a length-delimited `KeyValue` field with
/// the given tag.
///
/// When the value is not a CBOR serialized map/slice, the size of the KeyValue
/// is computed up front so both the KeyValue and the nested AnyValue get
/// canonical (minimal width) length prefixes instead of padded placeholders.
pub(crate) fn encode_attribute<T: ArrowPrimitiveType>(
    tag: u64,
    attr_arrays: &AttributeArrays<'_, T>,
    index: usize,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    let key = attr_arrays.attr_key.str_at(index);
    let value = attr_arrays
        .anyval_arrays
        .attr_type
        .value_at(index)
        .and_then(|t| AttributeValueType::try_from(t).ok())
        .map(|t| ResolvedAnyValue::resolve(&attr_arrays.anyval_arrays, index, t));

    let value_body_len = match &value {
        Some(value) => match value.encoded_len() {
            Some(len) => Some(len),
            None => return encode_attribute_unsized(tag, key, value, result_buf),
        },
        None => None,
    };

    let value_field_len = value_body_len.map_or(0, |len| {
        key_len(KEY_VALUE_VALUE as u32) + encoded_len_varint(len as u64) + len
    });
    let key_field_len = key.map_or(0, |key| {
        key_len(KEY_VALUE_KEY as u32) + encoded_len_varint(key.len() as u64) + key.len()
    });
    let kv_len = value_field_len + key_field_len;

    result_buf.encode_field_tag(tag, wire_types::LEN)?;
    result_buf.encode_varint(kv_len as u64)?;
    let body_start = result_buf.len();

    if let (Some(value), Some(len)) = (&value, value_body_len) {
        result_buf.encode_field_tag(KEY_VALUE_VALUE, wire_types::LEN)?;
        result_buf.encode_varint(len as u64)?;
        value.encode_body(result_buf)?;
    }
    if let Some(key) = key {
        result_buf.encode_string(KEY_VALUE_KEY, key)?;
    }

    debug_assert_eq!(
        result_buf.len() - body_start,
        kv_len,
        "predicted KeyValue length must match encoded length"
    );
    Ok(())
}

/// Encode the value at `index` as a length-delimited `AnyValue` field with
/// the given tag, using a canonical length prefix when the size is known up
/// front.
pub(crate) fn encode_any_value_field(
    tag: u64,
    arrays: &AnyValueArrays<'_>,
    index: usize,
    value_type: AttributeValueType,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    let value = ResolvedAnyValue::resolve(arrays, index, value_type);
    let Some(len) = value.encoded_len() else {
        return result_buf.encode_len_delimited(tag, |result_buf| value.encode_body(result_buf));
    };

    result_buf.encode_field_tag(tag, wire_types::LEN)?;
    result_buf.encode_varint(len as u64)?;
    let body_start = result_buf.len();
    value.encode_body(result_buf)?;

    debug_assert_eq!(
        result_buf.len() - body_start,
        len,
        "predicted AnyValue length must match encoded length"
    );
    Ok(())
}

/// Fallback for attributes whose value size is not known before encoding (CBOR
/// serialized maps/slices). Uses padded length placeholders patched after the
/// fact.
fn encode_attribute_unsized(
    tag: u64,
    key: Option<&str>,
    value: &ResolvedAnyValue<'_>,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    result_buf.encode_len_delimited(tag, |result_buf| {
        result_buf
            .encode_len_delimited(KEY_VALUE_VALUE, |result_buf| value.encode_body(result_buf))?;
        if let Some(key) = key {
            result_buf.encode_string(KEY_VALUE_KEY, key)?;
        }
        Ok(())
    })
}

/// An AnyValue read out of the OTAP columns, borrowing from the Arrow arrays.
/// Resolving once lets callers compute the encoded size before writing without
/// reading the columns twice.
enum ResolvedAnyValue<'a> {
    Str(&'a str),
    Bytes(&'a [u8]),
    Int(i64),
    Double(f64),
    Bool(bool),
    /// Encodes as an AnyValue with no fields set.
    Empty,
    /// CBOR serialized map or slice. The proto size is unknown until decoded.
    Serialized(&'a [u8]),
}

impl<'a> ResolvedAnyValue<'a> {
    /// Read the value at `index`. Missing columns or null values resolve to the
    /// default for the type, except bool and map/slice which resolve to
    /// [`ResolvedAnyValue::Empty`].
    fn resolve(
        arrays: &'a AnyValueArrays<'_>,
        index: usize,
        value_type: AttributeValueType,
    ) -> Self {
        match value_type {
            AttributeValueType::Str => Self::Str(
                arrays
                    .attr_str
                    .as_ref()
                    .and_then(|col| col.str_at(index))
                    .unwrap_or_default(),
            ),
            AttributeValueType::Bool => {
                // TODO handle case when bool column is missing we correct the default value handling
                // https://github.com/open-telemetry/otel-arrow/issues/1449
                arrays
                    .attr_bool
                    .and_then(|col| col.value_at(index))
                    .map_or(Self::Empty, Self::Bool)
            }
            AttributeValueType::Int => Self::Int(
                arrays
                    .attr_int
                    .as_ref()
                    .and_then(|col| col.value_at(index))
                    .unwrap_or_default(),
            ),
            AttributeValueType::Double => Self::Double(
                arrays
                    .attr_double
                    .and_then(|col| col.value_at(index))
                    .unwrap_or_default(),
            ),
            AttributeValueType::Bytes => Self::Bytes(
                arrays
                    .attr_bytes
                    .as_ref()
                    .and_then(|col| col.slice_at(index))
                    .unwrap_or_default(),
            ),
            AttributeValueType::Map | AttributeValueType::Slice => arrays
                .attr_ser
                .as_ref()
                .and_then(|col| col.slice_at(index))
                .map_or(Self::Empty, Self::Serialized),
            AttributeValueType::Empty => Self::Empty,
        }
    }

    /// Length of the encoded AnyValue message body, or `None` if it can only
    /// be determined by encoding.
    fn encoded_len(&self) -> Option<usize> {
        let len = match self {
            Self::Str(val) => {
                key_len(ANY_VALUE_STRING_VALUE as u32)
                    + encoded_len_varint(val.len() as u64)
                    + val.len()
            }
            Self::Bytes(val) => {
                key_len(ANY_VALUE_BYTES_VALUE as u32)
                    + encoded_len_varint(val.len() as u64)
                    + val.len()
            }
            Self::Int(val) => key_len(ANY_VALUE_INT_VALUE as u32) + encoded_len_varint(*val as u64),
            Self::Double(_) => key_len(ANY_VALUE_DOUBLE_VALUE as u32) + 8,
            Self::Bool(val) => {
                key_len(ANY_VALUE_BOOL_VALUE as u32) + encoded_len_varint(*val as u64)
            }
            Self::Empty => 0,
            Self::Serialized(_) => return None,
        };
        Some(len)
    }

    /// Write the AnyValue message body (no tag or length prefix).
    fn encode_body(&self, result_buf: &mut ProtoBuffer) -> Result<()> {
        match self {
            Self::Str(val) => result_buf.encode_string(ANY_VALUE_STRING_VALUE, val)?,
            Self::Bytes(val) => result_buf.encode_bytes(ANY_VALUE_BYTES_VALUE, val)?,
            Self::Int(val) => {
                result_buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT)?;
                result_buf.encode_varint(*val as u64)?;
            }
            Self::Double(val) => {
                result_buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64)?;
                result_buf.extend_from_slice(&val.to_le_bytes())?;
            }
            Self::Bool(val) => {
                result_buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT)?;
                result_buf.encode_varint(*val as u64)?;
            }
            Self::Empty => {}
            Self::Serialized(val) => proto_encode_cbor_bytes(val, result_buf)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{Array, RecordBatch, StructArray};
    use arrow::datatypes::Schema;
    use prost::Message;
    use std::sync::Arc;

    use crate::encode::record::attributes::AnyValuesRecordsBuilder;
    use crate::otlp::ProtoBuffer;
    use crate::otlp::common::AnyValueArrays;
    use crate::otlp::logs::LogBodyArrays;
    use crate::proto::consts::field_num::logs::LOG_RECORD_BODY;
    use crate::proto::consts::field_num::resource::RESOURCE_ATTRIBUTES;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::proto::opentelemetry::common::v1::{AnyValue, ArrayValue, KeyValue};
    use crate::proto::opentelemetry::logs::v1::LogRecord;
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::testing::round_trip::{encode_logs, to_logs_data};

    #[test]
    fn test_default_anyvalue_encoded_when_column_missing() -> Result<()> {
        // append a bunch of "default" values
        let mut rb_builder = AnyValuesRecordsBuilder::new();
        rb_builder.append_str(b"");
        rb_builder.append_bytes(b"");
        rb_builder.append_double(0.0);
        rb_builder.append_int(0);

        // TODO include test cases for bool once we've corrected the default value handling:
        // https://github.com/open-telemetry/otel-arrow/issues/1449

        let mut fields = vec![];
        let mut columns = vec![];
        rb_builder.finish(&mut columns, &mut fields).unwrap();
        let schema = Arc::new(Schema::new(fields));
        let rb = RecordBatch::try_new(schema, columns).unwrap();

        // assert the columns are not present
        assert!(
            rb.column_by_name(consts::ATTRIBUTE_BYTES).is_none(),
            "Bytes column should be omitted"
        );
        assert!(
            rb.column_by_name(consts::ATTRIBUTE_STR).is_none(),
            "Str column should be omitted"
        );
        assert!(
            rb.column_by_name(consts::ATTRIBUTE_INT).is_none(),
            "Int column should be omitted"
        );
        assert!(
            rb.column_by_name(consts::ATTRIBUTE_DOUBLE).is_none(),
            "Double column should be omitted"
        );

        let any_val_arrays = AnyValueArrays::try_from(&rb).unwrap();
        let mut protobuf = ProtoBuffer::default();

        for i in 0..rb.num_rows() {
            if let Some(value_type) = any_val_arrays.attr_type.value_at(i)
                && let Ok(value_type) = AttributeValueType::try_from(value_type)
            {
                // tag 1 is the values field in ArrayValue message
                encode_any_value_field(1, &any_val_arrays, i, value_type, &mut protobuf).unwrap();
            }
        }

        let results = ArrayValue::decode(protobuf.as_ref()).unwrap().values;
        let expected = vec![
            AnyValue::new_string(""),
            AnyValue::new_bytes(b""),
            AnyValue::new_double(0.0),
            AnyValue::new_int(0),
        ];

        assert_eq!(results, expected);

        Ok(())
    }

    /// Values covering every AnyValue type plus varint length boundaries. The
    /// bool flags whether the value is serialized as CBOR in OTAP (map/slice),
    /// in which case the encoded length is not required to be canonical.
    fn any_value_cases() -> Vec<(AnyValue, bool)> {
        vec![
            (AnyValue::new_string(""), false),
            (AnyValue::new_string("GET"), false),
            // 200 bytes -> 2 byte length varint
            (AnyValue::new_string("x".repeat(200)), false),
            // 20000 bytes -> 3 byte length varint
            (AnyValue::new_string("y".repeat(20_000)), false),
            (AnyValue::new_bytes(&[1u8, 2, 3][..]), false),
            (AnyValue::new_int(42), false),
            (AnyValue::new_int(0), false),
            // negative ints encode as 10 byte varints
            (AnyValue::new_int(-1), false),
            (AnyValue::new_double(3.5), false),
            (AnyValue::new_bool(true), false),
            (AnyValue::new_bool(false), false),
            (AnyValue { value: None }, false),
            (
                AnyValue::new_kvlist(vec![KeyValue::new("inner", AnyValue::new_int(1))]),
                true,
            ),
            (
                AnyValue::new_array(vec![AnyValue::new_string("a"), AnyValue::new_double(2.0)]),
                true,
            ),
        ]
    }

    /// Scenario: attributes of every value type, including keys and values that
    /// cross varint length boundaries, are encoded from OTAP to OTLP.
    /// Guarantees: each KeyValue decodes to the input, and for non-CBOR values
    /// the encoding is exactly as long as prost's canonical encoding (no padded
    /// length prefixes).
    #[test]
    fn test_encode_attribute_canonical_len() {
        let mut attributes: Vec<(KeyValue, bool)> = any_value_cases()
            .into_iter()
            .enumerate()
            .map(|(i, (v, is_ser))| (KeyValue::new(format!("k{i}"), v), is_ser))
            .collect();
        // a key long enough that the KeyValue length needs a 2 byte varint
        attributes.push((
            KeyValue::new("k".repeat(200), AnyValue::new_string("v")),
            false,
        ));

        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(
                    attributes
                        .iter()
                        .map(|(kv, _)| kv.clone())
                        .collect::<Vec<_>>(),
                )
                .finish(),
        ]);
        let otap = encode_logs(&logs);
        let attrs_rb = otap.get(ArrowPayloadType::LogAttrs).unwrap();
        let attr_arrays = Attribute16Arrays::try_from(attrs_rb).unwrap();
        assert_eq!(attrs_rb.num_rows(), attributes.len());

        let mut buf = ProtoBuffer::default();
        for i in 0..attrs_rb.num_rows() {
            buf.clear();
            encode_attribute(RESOURCE_ATTRIBUTES, &attr_arrays, i, &mut buf).unwrap();

            let decoded = Resource::decode(buf.as_ref()).unwrap();
            assert_eq!(decoded.attributes.len(), 1);
            let decoded_kv = &decoded.attributes[0];
            let (expected_kv, is_ser) = attributes
                .iter()
                .find(|(kv, _)| kv.key == decoded_kv.key)
                .unwrap();
            assert_eq!(decoded_kv, expected_kv);

            if !is_ser {
                let canonical = Resource {
                    attributes: vec![expected_kv.clone()],
                    ..Default::default()
                };
                assert_eq!(
                    buf.len(),
                    canonical.encoded_len(),
                    "non-canonical length for attribute {}",
                    expected_kv.key
                );
            }
        }
    }

    /// Scenario: log bodies of every value type are encoded from OTAP to OTLP.
    /// Guarantees: each body decodes to the input, and for non-CBOR values the
    /// encoding is exactly as long as prost's canonical encoding.
    #[test]
    fn test_encode_body_canonical_len() {
        let cases = any_value_cases();
        let logs = to_logs_data(
            cases
                .iter()
                .map(|(v, _)| LogRecord::build().body(v.clone()).finish())
                .collect(),
        );
        let otap = encode_logs(&logs);
        let logs_rb = otap.get(ArrowPayloadType::Logs).unwrap();
        let body = logs_rb
            .column_by_name(consts::BODY)
            .unwrap()
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap();
        let body_arrays = LogBodyArrays::try_from(body).unwrap();
        let anyval_arrays = &body_arrays.anyval_arrays;
        assert_eq!(logs_rb.num_rows(), cases.len());

        let mut buf = ProtoBuffer::default();
        for (i, (expected, is_ser)) in cases.iter().enumerate() {
            buf.clear();
            let value_type =
                AttributeValueType::try_from(anyval_arrays.attr_type.value(i)).unwrap();
            encode_any_value_field(LOG_RECORD_BODY, anyval_arrays, i, value_type, &mut buf)
                .unwrap();

            let decoded = LogRecord::decode(buf.as_ref()).unwrap();
            assert_eq!(decoded.body.as_ref(), Some(expected), "case {i}");

            if !is_ser {
                let canonical = LogRecord {
                    body: Some(expected.clone()),
                    ..Default::default()
                };
                assert_eq!(
                    buf.len(),
                    canonical.encoded_len(),
                    "non-canonical length for body case {i}"
                );
            }
        }
    }
}
