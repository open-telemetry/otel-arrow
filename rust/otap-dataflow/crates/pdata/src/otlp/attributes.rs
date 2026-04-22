// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{ArrowPrimitiveType, PrimitiveArray, RecordBatch, StringArray};
use arrow::datatypes::{UInt16Type, UInt32Type};
use num_enum::TryFromPrimitive;

use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor, get_required_array};
use crate::error::{Error, Result};
use crate::otlp::attributes::cbor::proto_encode_cbor_bytes;
use crate::otlp::common::{AnyValueArrays, ProtoBuffer};
use crate::proto::consts::field_num::common::{
    ANY_VALUE_BOOL_VALUE, ANY_VALUE_BYTES_VALUE, ANY_VALUE_DOUBLE_VALUE, ANY_VALUE_INT_VALUE,
    ANY_VALUE_STRING_VALUE, KEY_VALUE_KEY, KEY_VALUE_VALUE,
};
use crate::proto::consts::wire_types;
use crate::proto_encode_len_delimited_unknown_size;
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

pub(crate) fn encode_key_value<T: ArrowPrimitiveType>(
    attr_arrays: &AttributeArrays<'_, T>,
    index: usize,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(key) = attr_arrays.attr_key.str_at(index) {
        result_buf.encode_string(KEY_VALUE_KEY, key);
    }

    if let Some(value_type) = attr_arrays.anyval_arrays.attr_type.value_at(index) {
        if let Ok(value_type) = AttributeValueType::try_from(value_type) {
            // TODO try to compute the length of the value here. This would probably be
            // straight forward for most types for all cases except map/slice, and even then
            // we could maybe guess order of magnitude by looking at the CBOR representation
            proto_encode_len_delimited_unknown_size!(
                KEY_VALUE_VALUE,
                encode_any_value(&attr_arrays.anyval_arrays, index, value_type, result_buf)?,
                result_buf
            );
        }
    }

    Ok(())
}

pub(crate) fn encode_any_value(
    attr_arrays: &AnyValueArrays<'_>,
    index: usize,
    value_type: AttributeValueType,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    match value_type {
        AttributeValueType::Str => {
            let val = attr_arrays
                .attr_str
                .as_ref()
                .and_then(|col| col.str_at(index))
                .unwrap_or_default();
            result_buf.encode_string(ANY_VALUE_STRING_VALUE, val);
        }
        AttributeValueType::Bool => {
            // TODO handle case when bool column is missing we correct the default value handling
            // https://github.com/open-telemetry/otel-arrow/issues/1449
            if let Some(attr_bool) = &attr_arrays.attr_bool {
                if let Some(val) = attr_bool.value_at(index) {
                    result_buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT);
                    result_buf.encode_varint(val as u64)
                }
            }
        }
        AttributeValueType::Int => {
            let val = attr_arrays
                .attr_int
                .as_ref()
                .and_then(|col| col.value_at(index))
                .unwrap_or_default();
            result_buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
            result_buf.encode_varint(val as u64);
        }
        AttributeValueType::Double => {
            let val = attr_arrays
                .attr_double
                .as_ref()
                .and_then(|col| col.value_at(index))
                .unwrap_or_default();
            result_buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
        AttributeValueType::Bytes => {
            let val = attr_arrays
                .attr_bytes
                .as_ref()
                .and_then(|col| col.slice_at(index))
                .unwrap_or_default();
            result_buf.encode_bytes(ANY_VALUE_BYTES_VALUE, val);
        }
        AttributeValueType::Map | AttributeValueType::Slice => {
            if let Some(ser_bytes) = &attr_arrays.attr_ser {
                if let Some(val) = ser_bytes.slice_at(index) {
                    proto_encode_cbor_bytes(val, result_buf)?;
                }
            }
        }
        AttributeValueType::Empty => {
            // nothing to do
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::RecordBatch;
    use arrow::datatypes::Schema;
    use prost::Message;
    use std::sync::Arc;

    use crate::encode::record::attributes::AnyValuesRecordsBuilder;
    use crate::otlp::ProtoBuffer;
    use crate::otlp::common::AnyValueArrays;
    use crate::proto::opentelemetry::common::v1::{AnyValue, ArrayValue};

    #[test]
    fn test_default_anyvalue_encoded_when_column_missing() {
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
        let mut protobuf = ProtoBuffer::new();

        for i in 0..rb.num_rows() {
            if let Some(value_type) = any_val_arrays.attr_type.value_at(i) {
                if let Ok(value_type) = AttributeValueType::try_from(value_type) {
                    proto_encode_len_delimited_unknown_size!(
                        1, // the values field in ArrayValue message
                        encode_any_value(&any_val_arrays, i, value_type, &mut protobuf).unwrap(),
                        &mut protobuf
                    );
                }
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
    }
}
