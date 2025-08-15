// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for adding transport optimized encodings to the various columns in
//! OTAP record batches when converting to BatchArrowRecords. These types of encodings include
//! delta encoding for ID columns, and quasi-delta encoding for attribute parent IDs.
//!
//! The motivation behind adding these encodings to the columns is for better compression when,
//! for example, transmitting OTAP data via gRPC.

use arrow::{
    array::{ArrayRef, RecordBatch, StructArray},
    datatypes::{DataType, Field, Schema},
};

use crate::{
    error::Result,
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
    schema::{consts, get_field_metadata},
};

/// identifier for column encoding
#[derive(Clone, Copy)]
enum Encoding {
    Delta,

    /// this is the transport optimized encoding that is applied to attribute's parent_id column
    /// where subsequent rows of matching type/key/value will have their parent_id field delta
    /// encoded
    AttributeQuasiDelta,
}

// for the majority of columns, we'll be able to identify the path within the record batch as
// the column name directly, but Resource ID and Scope ID, they're typically nested within a
// struct on the root record so we treat these as special cases.
const RESOURCE_ID_COL_PATH: &str = "resource.id";
const SCOPE_ID_COL_PATH: &str = "scope.id";

/// specification for encoding that should be applied to the column before it is IPC serialized
struct ColumnEncoding<'a> {
    /// path to the column within the record batch
    path: &'a str,

    /// the expected data type of the column
    data_type: DataType,

    /// identifier for how the column should be encoded
    encoding: Encoding,
}

impl<'a> ColumnEncoding<'a> {
    /// access the column associated with this [`ColumnEncoding`]
    fn access_column(&self, schema: &Schema, columns: &[ArrayRef]) -> Option<ArrayRef> {
        // handle special case of accessing either the resource ID or scope ID which are nested
        // within a struct
        if let Some(struct_col_name) = self.struct_column_name() {
            let struct_col_idx = schema.index_of(struct_col_name).ok()?;
            let struct_col = columns
                .get(struct_col_idx)?
                .as_any()
                .downcast_ref::<StructArray>()?;
            return struct_col.column_by_name(consts::ID).cloned();
        }

        // otherwise just return column by nname
        let column_idx = schema.index_of(self.path).ok()?;
        columns.get(column_idx).cloned()
    }

    /// checks if the column associated with this [`ColumnEncoding`] has already had the column
    /// encoding applied.
    ///
    /// this is done by inspecting the field metadata, and specifically checking that the column
    /// encoding is not 'plain'. if there is no field metadata, we assume the column is already
    /// encoded. we make this assumption because it probably means we received this OTAP data from
    /// the golang OTAP exporter, which always encodes the columns and never adds metadata
    ///
    /// returns `None` if the field associated with `self.path` isn't found in passed schema
    fn is_column_encoded(&self, schema: &Schema) -> Option<bool> {
        let field = if let Some(struct_col_name) = self.struct_column_name() {
            // get the ID field out of the struct column
            let struct_col = schema.field_with_name(struct_col_name).ok()?;
            if let DataType::Struct(fields) = struct_col.data_type() {
                fields.find(consts::ID).map(|(_, field)| field)?
            } else {
                return None;
            }
        } else {
            // otherwise just look at field with path == name
            schema.field_with_name(self.path).ok()?
        };

        // check the field metadata to determine if field is encoded
        let field_metadata = field.metadata();
        let is_encoded = match field_metadata.get(consts::metadata::COLUMN_ENCODING) {
            Some(encoding) => encoding != consts::metadata::encodings::PLAIN,

            // assume if there is no metadata, then the column is already encoded
            None => true,
        };

        Some(is_encoded)
    }

    /// if configured to encode the ID column in the nested resource/scope struct array, this
    /// helper function simply returns the name of the struct column, and otherwise returns `None`
    fn struct_column_name(&self) -> Option<&'static str> {
        if self.path == RESOURCE_ID_COL_PATH {
            return Some(consts::RESOURCE);
        }

        if self.path == SCOPE_ID_COL_PATH {
            return Some(consts::SCOPE);
        }

        None
    }
}

/// helper for initializing [`ColumnEncoding`] as it will be initialized many times in the function
/// below, so this helps w/ brevity
macro_rules! col_encoding {
    ($path:expr, $dtype:ident, $enc:ident) => {
        ColumnEncoding {
            path: $path,
            data_type: DataType::$dtype,
            encoding: Encoding::$enc,
        }
    };
}

/// returns the list of transport-optimized encoding that should be applied to OTAP batches of a
/// given payload type
fn get_column_encodings(payload_type: &ArrowPayloadType) -> &'static [ColumnEncoding<'static>] {
    match payload_type {
        ArrowPayloadType::LogAttrs => &[
            col_encoding!(consts::ID, UInt16, Delta),
            col_encoding!(RESOURCE_ID_COL_PATH, UInt16, Delta),
            col_encoding!(SCOPE_ID_COL_PATH, UInt16, Delta),
        ],
        _ => &[],
    }
}

/// apply transport-optimized encodings to the record batch's columns
pub fn apply_column_encodings(
    payload_type: &ArrowPayloadType,
    record_batch: &RecordBatch,
) -> Result<RecordBatch> {
    let column_encodings = get_column_encodings(payload_type);

    // TODO revisit this check after filling in get_column_encodings b/c there might not be any
    // payload types that have no encoded columns in practice, in which case this check is useless
    // as it'll never be true.
    if column_encodings.is_empty() {
        // nothing to do
        return Ok(record_batch.clone());
    }

    let mut schema = record_batch.schema().as_ref().clone();

    // determine which columns need to be encoded
    let mut to_apply = vec![];
    for column_encoding in column_encodings {
        let is_encoded = column_encoding.is_column_encoded(&schema);
        if is_encoded == Some(false) {
            to_apply.push(column_encoding)
        }
    }

    if to_apply.len() == 0 {
        // nothing to do
        return Ok(record_batch.clone());
    }

    todo!();
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use arrow::{
        array::{StringArray, StructArray, UInt8Array, UInt16Array},
        datatypes::{Field, Fields},
    };

    use crate::{encode::column_encoding, schema::FieldExt};

    use super::*;

    #[test]
    fn test_access_column_basic() {
        let schema = Schema::new(vec![Field::new("a", DataType::UInt8, true)]);
        let columns = vec![Arc::new(UInt8Array::from_iter_values([1, 2])) as ArrayRef];

        let mut column_encoding = ColumnEncoding {
            path: "a",
            data_type: DataType::UInt8,
            encoding: Encoding::Delta,
        };

        let column = column_encoding
            .access_column(&schema, columns.as_ref())
            .unwrap();
        assert_eq!(*column, *columns[0]);

        // assert what happens if the column isn't present
        column_encoding.path = "b";
        assert!(
            column_encoding
                .access_column(&schema, columns.as_ref())
                .is_none()
        )
    }

    #[test]
    fn test_access_column_nested() {
        let struct_fields: Fields = vec![Field::new("id", DataType::UInt16, true)].into();
        let schema = Schema::new(vec![
            Field::new("resource", DataType::Struct(struct_fields.clone()), true),
            Field::new("scope", DataType::Struct(struct_fields.clone()), true),
        ]);

        let resource_ids = UInt16Array::from_iter_values(vec![1]);
        let scope_ids = UInt16Array::from_iter_values(vec![2]);

        let columns = vec![
            Arc::new(StructArray::new(
                struct_fields.clone(),
                vec![Arc::new(resource_ids.clone())],
                None,
            )) as ArrayRef,
            Arc::new(StructArray::new(
                struct_fields.clone(),
                vec![Arc::new(scope_ids.clone())],
                None,
            )) as ArrayRef,
        ];

        let mut column_encoding = ColumnEncoding {
            path: RESOURCE_ID_COL_PATH,
            data_type: DataType::UInt8,
            encoding: Encoding::Delta,
        };

        let column = column_encoding
            .access_column(&schema, columns.as_ref())
            .unwrap();
        assert_eq!(*column, resource_ids);

        column_encoding.path = SCOPE_ID_COL_PATH;
        let column = column_encoding
            .access_column(&schema, columns.as_ref())
            .unwrap();
        assert_eq!(*column, scope_ids)
    }

    #[test]
    fn test_access_column_not_present_nested() {
        let schema = Schema::new(vec![Field::new(
            "scope",
            DataType::Struct(Vec::<Field>::new().into()),
            true,
        )]);

        let columns = vec![Arc::new(StructArray::new_empty_fields(1, None)) as ArrayRef];

        let mut column_encoding = ColumnEncoding {
            path: RESOURCE_ID_COL_PATH,
            data_type: DataType::UInt8,
            encoding: Encoding::Delta,
        };

        // assert what happens if the struct isn't present
        assert!(
            column_encoding
                .access_column(&schema, columns.as_ref())
                .is_none()
        );

        // assert what happens if the struct is present, but the ID field isn't present
        column_encoding.path = SCOPE_ID_COL_PATH;
        assert!(
            column_encoding
                .access_column(&schema, columns.as_ref())
                .is_none()
        );
    }

    #[test]
    fn test_is_column_encoded_basic() {
        let schema = Schema::new(vec![
            Field::new("a", DataType::Utf8, false).with_plain_encoding(),
            Field::new("b", DataType::UInt16, false),
        ]);

        let mut column_encoding = ColumnEncoding {
            path: "a",
            data_type: DataType::UInt16,
            encoding: Encoding::Delta,
        };

        assert!(!column_encoding.is_column_encoded(&schema).unwrap());

        // ensure that no metadata means column is encoded
        column_encoding.path = "b";
        assert!(column_encoding.is_column_encoded(&schema).unwrap());
    }

    #[test]
    fn test_is_column_encoded_nested() {
        let schema = Schema::new(vec![
            Field::new(
                "resource",
                DataType::Struct(
                    vec![Field::new("id", DataType::UInt16, true).with_plain_encoding()].into(),
                ),
                true,
            ),
            Field::new(
                "scope",
                DataType::Struct(vec![Field::new("id", DataType::UInt16, true)].into()),
                true,
            ),
        ]);

        let mut column_encoding = ColumnEncoding {
            path: RESOURCE_ID_COL_PATH,
            data_type: DataType::UInt16,
            encoding: Encoding::Delta,
        };

        assert!(!column_encoding.is_column_encoded(&schema).unwrap());
        column_encoding.path = SCOPE_ID_COL_PATH;
        assert!(column_encoding.is_column_encoded(&schema).unwrap());
    }

    #[test]
    fn test_apply_column_encodings_is_noop_if_all_columns_already_encoded() {
        let struct_fields: Fields = vec![Field::new(consts::ID, DataType::UInt16, true)].into();
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, false),
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(struct_fields.clone()),
                    true,
                ),
                Field::new(consts::SCOPE, DataType::Struct(struct_fields.clone()), true),
            ])),
            vec![
                // id:
                Arc::new(UInt16Array::from_iter_values(vec![1])),
                // resource:
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values(vec![1]))],
                    None,
                )),
                // scope:
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values(vec![1]))],
                    None,
                )),
            ],
        )
        .unwrap();

        let result = apply_column_encodings(&ArrowPayloadType::Logs, &input).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_apply_column_encodings_is_noop_if_all_columns_not_present() {
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::SCHEMA_URL,
                DataType::Utf8,
                false,
            )])),
            vec![Arc::new(StringArray::from_iter_values(vec!["a"]))],
        )
        .unwrap();

        let result = apply_column_encodings(&ArrowPayloadType::Logs, &input).unwrap();
        assert_eq!(result, input);
    }
}
