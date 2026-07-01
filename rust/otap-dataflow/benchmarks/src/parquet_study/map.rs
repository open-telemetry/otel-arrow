// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Map flattened-Parquet contender.
//!
//! Identical to the [`nested`](super::nested) layout except the three attribute
//! containers are encoded as Arrow `Map<Utf8, Struct{type,str,int,double,bool,
//! bytes,ser}>` instead of `List<Struct{key,...}>`. The key is pulled out of the
//! attribute struct into the map key; the remaining seven value columns form the
//! map value struct. This exercises Parquet's map encoding versus the
//! list-of-struct encoding for the exact same logical attributes.

use std::sync::Arc;

use arrow::array::{ArrayRef, MapArray, RecordBatch, StructArray};
use arrow::buffer::{OffsetBuffer, ScalarBuffer};
use arrow::datatypes::{DataType, Field, Fields, Schema};

use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use super::attrs::{
    Gathered, LOG_ATTRS_COL, RESOURCE_ATTRS_COL, SCOPE_ATTRS_COL, attr_struct_fields,
    attr_value_struct_fields, entries_dedup, entries_per_row, gather_by_parent, logs_batch,
    logs_id, logs_resource_id, logs_scope_id, rebuild_attr_batch_from_parts, taken_attr_struct,
};
use super::{Codec, Compressor, StudyResult, parquet_io};

/// Contender that flattens OTAP logs into a single Parquet file using `Map`
/// attribute columns.
pub struct MapParquetCodec {
    /// Parquet compression codec.
    pub compressor: Compressor,
}

fn map_entries_fields() -> Fields {
    Fields::from(vec![
        Field::new("keys", DataType::Utf8, false),
        Field::new(
            "values",
            DataType::Struct(attr_value_struct_fields()),
            false,
        ),
    ])
}

fn map_entries_field() -> Arc<Field> {
    Arc::new(Field::new(
        "entries",
        DataType::Struct(map_entries_fields()),
        false,
    ))
}

fn map_column_field(name: &str) -> Field {
    Field::new(name, DataType::Map(map_entries_field(), false), false)
}

fn build_attr_map_column(attr_batch: &RecordBatch, gathered: &Gathered) -> StudyResult<ArrayRef> {
    let full = taken_attr_struct(attr_batch, gathered)?;
    let keys = full.column(0).clone();
    let value_struct = StructArray::new(
        attr_value_struct_fields(),
        full.columns()[1..].to_vec(),
        None,
    );
    let entries = StructArray::new(
        map_entries_fields(),
        vec![keys, Arc::new(value_struct)],
        None,
    );
    let offsets = OffsetBuffer::new(ScalarBuffer::from(gathered.offsets.clone()));
    let map = MapArray::try_new(map_entries_field(), offsets, entries, None, false)?;
    Ok(Arc::new(map))
}

fn empty_map_column(num_rows: usize) -> StudyResult<ArrayRef> {
    let keys = arrow::array::new_empty_array(&DataType::Utf8);
    let value_children: Vec<ArrayRef> = attr_value_struct_fields()
        .iter()
        .map(|f| arrow::array::new_empty_array(f.data_type()))
        .collect();
    let value_struct = StructArray::new(attr_value_struct_fields(), value_children, None);
    let entries = StructArray::new(
        map_entries_fields(),
        vec![keys, Arc::new(value_struct)],
        None,
    );
    let offsets = OffsetBuffer::new(ScalarBuffer::from(vec![0i32; num_rows + 1]));
    let map = MapArray::try_new(map_entries_field(), offsets, entries, None, false)?;
    Ok(Arc::new(map))
}

/// Flatten an OTAP logs batch into the map flat record batch.
pub fn flatten(otap: &OtapArrowRecords) -> StudyResult<RecordBatch> {
    let logs = logs_batch(otap)?;
    let num_rows = logs.num_rows();
    let resource_id = logs_resource_id(logs)?;
    let scope_id = logs_scope_id(logs)?;
    let log_id = logs_id(logs)?;

    let resource_col = match otap.get(ArrowPayloadType::ResourceAttrs) {
        Some(b) => build_attr_map_column(b, &gather_by_parent(b, &resource_id)?)?,
        None => empty_map_column(num_rows)?,
    };
    let scope_col = match otap.get(ArrowPayloadType::ScopeAttrs) {
        Some(b) => build_attr_map_column(b, &gather_by_parent(b, &scope_id)?)?,
        None => empty_map_column(num_rows)?,
    };
    let log_col = match otap.get(ArrowPayloadType::LogAttrs) {
        Some(b) => build_attr_map_column(b, &gather_by_parent(b, &log_id)?)?,
        None => empty_map_column(num_rows)?,
    };

    let mut fields: Vec<Field> = logs
        .schema()
        .fields()
        .iter()
        .map(|f| f.as_ref().clone())
        .collect();
    let mut columns: Vec<ArrayRef> = logs.columns().to_vec();
    for (name, col) in [
        (RESOURCE_ATTRS_COL, resource_col),
        (SCOPE_ATTRS_COL, scope_col),
        (LOG_ATTRS_COL, log_col),
    ] {
        fields.push(map_column_field(name));
        columns.push(col);
    }

    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}

fn map_column<'a>(flat: &'a RecordBatch, name: &str) -> StudyResult<&'a MapArray> {
    flat.column_by_name(name)
        .ok_or_else(|| format!("flat batch missing `{name}` column"))?
        .as_any()
        .downcast_ref::<MapArray>()
        .ok_or_else(|| format!("`{name}` is not a map").into())
}

fn rebuild_from_map(map: &MapArray, entries: &[(usize, u16)]) -> StudyResult<RecordBatch> {
    let keys = map.keys().clone();
    let value_struct = map
        .values()
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or("map values are not a struct")?;
    let mut cols: Vec<ArrayRef> = Vec::with_capacity(8);
    cols.push(keys);
    cols.extend(value_struct.columns().iter().cloned());
    let full = StructArray::new(attr_struct_fields(), cols, None);
    rebuild_attr_batch_from_parts(&full, map.value_offsets(), entries)
}

/// Reconstruct an OTAP logs batch from the map flat record batch.
pub fn unflatten(flat: &RecordBatch) -> StudyResult<OtapArrowRecords> {
    let container = [RESOURCE_ATTRS_COL, SCOPE_ATTRS_COL, LOG_ATTRS_COL];

    let mut fields: Vec<Field> = Vec::new();
    let mut columns: Vec<ArrayRef> = Vec::new();
    for (field, column) in flat.schema().fields().iter().zip(flat.columns()) {
        if !container.contains(&field.name().as_str()) {
            fields.push(field.as_ref().clone());
            columns.push(column.clone());
        }
    }
    let logs = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)?;

    let resource_id = logs_resource_id(&logs)?;
    let scope_id = logs_scope_id(&logs)?;
    let log_id = logs_id(&logs)?;

    let resource_attrs = rebuild_from_map(
        map_column(flat, RESOURCE_ATTRS_COL)?,
        &entries_dedup(&resource_id),
    )?;
    let scope_attrs = rebuild_from_map(
        map_column(flat, SCOPE_ATTRS_COL)?,
        &entries_dedup(&scope_id),
    )?;
    let log_attrs = rebuild_from_map(map_column(flat, LOG_ATTRS_COL)?, &entries_per_row(&log_id))?;

    let mut otap = OtapArrowRecords::Logs(Default::default());
    otap.set(ArrowPayloadType::Logs, logs)?;
    otap.set(ArrowPayloadType::ResourceAttrs, resource_attrs)?;
    otap.set(ArrowPayloadType::ScopeAttrs, scope_attrs)?;
    otap.set(ArrowPayloadType::LogAttrs, log_attrs)?;
    Ok(otap)
}

impl Codec for MapParquetCodec {
    fn name(&self) -> &'static str {
        "parquet-map"
    }

    fn write(&self, logs: OtapArrowRecords) -> StudyResult<Vec<u8>> {
        let flat = flatten(&logs)?;
        parquet_io::write_parquet(&flat, self.compressor.parquet())
    }

    fn read(&self, bytes: &[u8]) -> StudyResult<OtapArrowRecords> {
        let flat = parquet_io::read_parquet(bytes)?;
        unflatten(&flat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parquet_study::Compressor;
    use crate::parquet_study::attrs::assert_logs_equivalent;
    use crate::parquet_study::datagen::{LogsGenParams, gen_logs_otap};

    #[test]
    fn map_round_trip_preserves_structure() {
        let params = LogsGenParams {
            num_resources: 3,
            num_scopes: 2,
            num_logs: 5,
        };
        let (otap, _) = gen_logs_otap(&params);

        for compressor in Compressor::ALL {
            let codec = MapParquetCodec { compressor };
            let bytes = codec.write(otap.clone()).expect("write");
            let decoded = codec.read(&bytes).expect("read");
            assert_logs_equivalent(&otap, &decoded, codec.name(), compressor.label());
        }
    }
}
