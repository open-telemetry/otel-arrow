// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//todo: support schema transformation if we need to implement the encoding part.

// TODO write documentation for this crate
#![allow(missing_docs)]

use arrow::array::{LargeListArray, RecordBatch};
use arrow::datatypes::{Field, Schema};
use std::collections::HashMap;
use std::sync::Arc;

pub mod consts;

/// Trace IDs are 16 binary bytes.
pub type TraceId = [u8; 16];

/// Span IDs are 8 binary bytes.
pub type SpanId = [u8; 8];

/// Returns a new record batch with the new key/value updated in the schema metadata.
#[must_use]
pub fn update_schema_metadata(
    record_batch: &RecordBatch,
    key: String,
    value: String,
) -> RecordBatch {
    let schema = record_batch.schema_ref();
    let mut schema_metadata = schema.metadata.clone();
    let _ = schema_metadata.insert(key, value);

    let new_schema = schema.as_ref().clone().with_metadata(schema_metadata);

    // safety: this should not fail, as we haven't changed the fields in the schema,
    // just the metadata, so the schema should be compatible with the columns
    record_batch
        .clone()
        .with_schema(Arc::new(new_schema))
        .expect("can create record batch with same schema.")
}

/// Returns a new record batch with the new key/value updated in the field metadata.
#[must_use]
pub fn update_field_metadata(schema: &Schema, column_name: &str, key: &str, value: &str) -> Schema {
    // find the column index
    let column_index = schema.index_of(column_name);
    if column_index.is_err() {
        // nothing to do, column doesn't exist
        return schema.clone();
    }
    // safety: we have already returned if column_id is Err
    let column_index = column_index.expect("expect column_id is Ok");

    // create a new field with updated metadata
    let field = &schema.fields[column_index];
    let mut field_metadata = field.metadata().clone();
    let _ = field_metadata.insert(key.to_string(), value.to_string());
    let new_field = field.as_ref().clone().with_metadata(field_metadata);

    let new_fields = schema
        .fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            if i == column_index {
                Arc::new(new_field.clone())
            } else {
                f.clone()
            }
        })
        .collect::<Vec<_>>();

    // create a new schema with the updated field
    Schema::new(new_fields).with_metadata(schema.metadata().clone())
}

/// Get the value of the schema metadata for a given key.
#[must_use]
pub fn get_schema_metadata<'a>(schema: &'a Schema, key: &'a str) -> Option<&'a str> {
    // get the schema metadata
    let schema_metadata = schema.metadata();
    schema_metadata.get(key).map(|s| s.as_str())
}

/// Get the value of the field metadata for a given column and key.
#[must_use]
pub fn get_field_metadata<'a>(
    schema: &'a Schema,
    column_name: &str,
    key: &'a str,
) -> Option<&'a str> {
    // find the column index
    let column_index = schema.index_of(column_name);
    if column_index.is_err() {
        // nothing to do, column doesn't exist
        return None;
    }
    // safety: we've already returned if column_index is error
    let column_index = column_index.expect("column_index to be Ok");

    // get the field metadata
    let field = &schema.fields[column_index];
    let field_metadata = field.metadata();
    field_metadata.get(key).map(|s| s.as_str())
}

/// Make a `LargeListArray` into an array whose item field is not nullable.
///
/// When you use `GenericListBuilder`, you'll get a list array where list elements are
/// nullable. This is often not what we want, so this little function converts `LargeListArray`s
/// that don't have any nulls into an equivalent form whose item field type is not nullable. This
/// function panics if the input contains any nulls at all.
#[must_use]
pub fn no_nulls(values: LargeListArray) -> LargeListArray {
    let (mut field, offsets, values, nulls) = values.into_parts();
    assert_eq!(0, nulls.map(|n| n.null_count()).unwrap_or(0));
    Arc::make_mut(&mut field).set_nullable(false);
    LargeListArray::new(field, offsets, values, None)
}

/// Checks the Arrow schema field metadata to determine if the "id" field in this record batch is
/// plain encoded.
///
/// It is assumed that if the ID column is plain encoded, the schema metadata will have this
/// the encoding identified explicitly. If the encoding metadata is absent, then the batch may
/// have been produced by the golang exporter, which adds no metadata and generally uses delta
/// encoding for the ID column by default.
#[must_use]
pub fn is_id_plain_encoded(record_batch: &RecordBatch) -> bool {
    let schema = record_batch.schema_ref();
    let encoding = get_field_metadata(
        schema.as_ref(),
        consts::ID,
        consts::metadata::COLUMN_ENCODING,
    );
    encoding == Some(consts::metadata::encodings::PLAIN)
}

/// Checks the Arrow schema field metadata to determine if the "parent_id" field in this record
/// batch is plain encoded.
///
/// It is assumed that if the Parent ID column is plain encoded, the schema metadata will have the
/// encoding identified explicitly. If the encoding metadata is absent, then the batch may have
/// been produced by the golang exporter, which adds no metadata and generally uses the transport
/// optimized/quasi-delta encoding for the Parent ID column by default.
#[must_use]
pub fn is_parent_id_plain_encoded(record_batch: &RecordBatch) -> bool {
    let schema = record_batch.schema_ref();
    let encoding = get_field_metadata(
        schema.as_ref(),
        consts::PARENT_ID,
        consts::metadata::COLUMN_ENCODING,
    );
    encoding == Some(consts::metadata::encodings::PLAIN)
}

/// Additional helper methods for Arrow Field
pub trait FieldExt {
    /// Sets the encoding column metadata key to "plain".
    ///
    /// This can be used as an indicator for various utilities that need to process the record
    /// batch of how the column is encoded. Commonly, this is used for ID columns that may or
    /// may not use some alternate encoding (like delta encoding)
    fn with_plain_encoding(self) -> Self;
}

impl FieldExt for Field {
    fn with_plain_encoding(self) -> Self {
        self.with_metadata(HashMap::from_iter([(
            consts::metadata::COLUMN_ENCODING.into(),
            consts::metadata::encodings::PLAIN.into(),
        )]))
    }
}
