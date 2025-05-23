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

use arrow::array::RecordBatch;
use arrow::datatypes::Schema;
use std::sync::Arc;

pub mod consts;

/// Returns a new record batch with the new key/value updated in the schema metadata.
pub fn update_schema_metadata(
    record_batch: RecordBatch,
    key: String,
    value: String,
) -> RecordBatch {
    let schema = record_batch.schema_ref();
    let mut schema_metadata = schema.metadata.clone();
    let _ = schema_metadata.insert(key, value);

    let new_schema = schema.as_ref().clone().with_metadata(schema_metadata);

    // TODO expect, safety, etc
    record_batch.with_schema(Arc::new(new_schema)).unwrap()
}

/// Returns a new record batch with the new key/value updated in the field metadata.
pub fn update_field_metadata(schema: &Schema, column_name: &str, key: &str, value: &str) -> Schema {
    // find the column index
    let column_index = schema.index_of(column_name);
    if !column_index.is_ok() {
        // nothing to do, column doesn't exist
        return schema.clone();
    }
    let column_index = column_index.unwrap();

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
pub fn get_schema_metadata<'a>(schema: &'a Schema, key: &'a str) -> Option<&'a str> {
    // get the schema metadata
    let schema_metadata = schema.metadata();
    schema_metadata.get(key).map(|s| s.as_str())
}

/// Get the value of the field metadata for a given column and key.
pub fn get_field_metadata<'a>(
    schema: &'a Schema,
    column_name: &str,
    key: &'a str,
) -> Option<&'a str> {
    // find the column index
    let column_index = schema.index_of(column_name);
    if !column_index.is_ok() {
        // nothing to do, column doesn't exist
        return None;
    }
    let column_index = column_index.unwrap();

    // get the field metadata
    let field = &schema.fields[column_index];
    let field_metadata = field.metadata();
    field_metadata.get(key).map(|s| s.as_str())
}
