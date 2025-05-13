// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    NullableArrayAccessor, StringArrayAccessor, StructColumnAccessor, get_required_array,
};
use crate::error;
use crate::proto::opentelemetry::common::v1::InstrumentationScope;
use crate::schema::consts;
use arrow::array::{Array, RecordBatch, StructArray, UInt16Array, UInt32Array};
use arrow::datatypes::{DataType, Field, Fields};
use snafu::OptionExt;
use std::sync::LazyLock;

pub(in crate::otlp) struct ResourceArrays<'a> {
    pub id: &'a UInt16Array,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
    pub schema_url: Option<StringArrayAccessor<'a>>,
}

static RESOURCE_ARRAY_DATA_TYPE: LazyLock<DataType> = LazyLock::new(|| {
    DataType::Struct(Fields::from(vec![
        Field::new(consts::ID, DataType::UInt16, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        Field::new(
            consts::SCHEMA_URL,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        ),
    ]))
});

impl ResourceArrays<'_> {
    fn data_type() -> &'static DataType {
        &RESOURCE_ARRAY_DATA_TYPE
    }
}

impl<'a> TryFrom<&'a RecordBatch> for ResourceArrays<'a> {
    type Error = error::Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self, Self::Error> {
        let struct_array = get_required_array(rb, consts::RESOURCE)?;
        let struct_array = struct_array
            .as_any()
            .downcast_ref::<StructArray>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: consts::RESOURCE,
                actual: struct_array.data_type().clone(),
                expect: Self::data_type().clone(),
            })?;

        let struct_col_accessor = StructColumnAccessor::new(struct_array);
        let id_array = struct_col_accessor.primitive_column(consts::ID)?;
        let dropped_attributes_count =
            struct_col_accessor.primitive_column_op(consts::DROPPED_ATTRIBUTES_COUNT)?;

        let schema_url = struct_array
            .column_by_name(consts::SCHEMA_URL)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        Ok(Self {
            id: id_array,
            dropped_attributes_count,
            schema_url,
        })
    }
}

pub(in crate::otlp) struct ScopeArrays<'a> {
    pub name: Option<StringArrayAccessor<'a>>,
    pub version: Option<StringArrayAccessor<'a>>,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
    pub id: Option<&'a UInt16Array>,
}

pub static SCOPE_ARRAY_DATA_TYPE: LazyLock<DataType> = LazyLock::new(|| {
    DataType::Struct(Fields::from(vec![
        Field::new(
            consts::NAME,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        ),
        Field::new(consts::VERSION, DataType::Utf8, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        Field::new(consts::ID, DataType::UInt16, true),
    ]))
});

impl ScopeArrays<'_> {
    fn data_type() -> &'static DataType {
        &SCOPE_ARRAY_DATA_TYPE
    }

    pub fn create_instrumentation_scope(&self, idx: usize) -> InstrumentationScope {
        InstrumentationScope {
            name: self.name.value_at(idx).unwrap_or_default(),
            version: self.version.value_at_or_default(idx),
            dropped_attributes_count: self.dropped_attributes_count.value_at_or_default(idx),
            attributes: vec![],
        }
    }
}

impl<'a> TryFrom<&'a RecordBatch> for ScopeArrays<'a> {
    type Error = error::Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self, Self::Error> {
        let struct_array = get_required_array(rb, consts::RESOURCE)?;
        let scope_array = struct_array
            .as_any()
            .downcast_ref::<StructArray>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: consts::RESOURCE,
                actual: struct_array.data_type().clone(),
                expect: Self::data_type().clone(),
            })?;

        let name = scope_array
            .column_by_name(consts::NAME)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        let version = scope_array
            .column_by_name(consts::VERSION)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        let struct_col_accessor = StructColumnAccessor::new(scope_array);
        let id = struct_col_accessor.primitive_column_op(consts::ID)?;
        let dropped_attributes_count =
            struct_col_accessor.primitive_column_op(consts::DROPPED_ATTRIBUTES_COUNT)?;

        Ok(Self {
            name,
            version,
            dropped_attributes_count,
            id,
        })
    }
}
