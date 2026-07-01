// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Nested flattened-Parquet contender.
//!
//! The flat table keeps the entire root `Logs` record batch unchanged (so the
//! decode side can carry the scalar/struct columns straight back without
//! re-walking them, matching what the IPC path gets "for free") and appends
//! three `List<Struct{key,type,str,int,double,bool,bytes,ser}>` columns holding
//! each log row's denormalized resource, scope, and log attributes.
//!
//! On decode the `Logs` batch is the flat table minus those three columns; the
//! attribute batches are rebuilt from the list columns, re-normalizing the
//! resource/scope sets via the `resource.id` / `scope.id` join keys that the
//! `Logs` batch still carries.

use std::sync::Arc;

use arrow::array::{ArrayRef, RecordBatch};
use arrow::datatypes::{Field, Schema};

use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use super::attrs::{
    self, LOG_ATTRS_COL, RESOURCE_ATTRS_COL, SCOPE_ATTRS_COL, attr_list_column_field,
    build_attr_list_column, empty_attr_list_column, entries_dedup, entries_per_row,
    gather_by_parent, logs_batch, logs_id, logs_resource_id, logs_scope_id, rebuild_attr_batch,
};
use super::{Codec, Compressor, StudyResult, parquet_io};

/// Contender that flattens OTAP logs into a single Parquet file using
/// `List<Struct>` attribute columns.
pub struct NestedParquetCodec {
    /// Parquet compression codec.
    pub compressor: Compressor,
}

/// Flatten an OTAP logs batch into the nested flat record batch.
pub fn flatten(otap: &OtapArrowRecords) -> StudyResult<RecordBatch> {
    let logs = logs_batch(otap)?;
    let num_rows = logs.num_rows();
    let resource_id = logs_resource_id(logs)?;
    let scope_id = logs_scope_id(logs)?;
    let log_id = logs_id(logs)?;

    let resource_col = match otap.get(ArrowPayloadType::ResourceAttrs) {
        Some(b) => build_attr_list_column(b, &gather_by_parent(b, &resource_id)?)?,
        None => empty_attr_list_column(num_rows),
    };
    let scope_col = match otap.get(ArrowPayloadType::ScopeAttrs) {
        Some(b) => build_attr_list_column(b, &gather_by_parent(b, &scope_id)?)?,
        None => empty_attr_list_column(num_rows),
    };
    let log_col = match otap.get(ArrowPayloadType::LogAttrs) {
        Some(b) => build_attr_list_column(b, &gather_by_parent(b, &log_id)?)?,
        None => empty_attr_list_column(num_rows),
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
        fields.push(attr_list_column_field(name));
        columns.push(col);
    }

    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}

/// Reconstruct an OTAP logs batch from the nested flat record batch.
pub fn unflatten(flat: &RecordBatch) -> StudyResult<OtapArrowRecords> {
    let container = [RESOURCE_ATTRS_COL, SCOPE_ATTRS_COL, LOG_ATTRS_COL];

    // The Logs batch is the flat table minus the three attribute columns.
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

    let resource_attrs = rebuild_attr_batch(
        attrs::as_list(flat, RESOURCE_ATTRS_COL)?,
        &entries_dedup(&resource_id),
    )?;
    let scope_attrs = rebuild_attr_batch(
        attrs::as_list(flat, SCOPE_ATTRS_COL)?,
        &entries_dedup(&scope_id),
    )?;
    let log_attrs = rebuild_attr_batch(
        attrs::as_list(flat, LOG_ATTRS_COL)?,
        &entries_per_row(&log_id),
    )?;

    let mut otap = OtapArrowRecords::Logs(Default::default());
    otap.set(ArrowPayloadType::Logs, logs)?;
    otap.set(ArrowPayloadType::ResourceAttrs, resource_attrs)?;
    otap.set(ArrowPayloadType::ScopeAttrs, scope_attrs)?;
    otap.set(ArrowPayloadType::LogAttrs, log_attrs)?;
    Ok(otap)
}

impl Codec for NestedParquetCodec {
    fn name(&self) -> &'static str {
        "parquet-nested"
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
    fn nested_round_trip_preserves_structure() {
        let params = LogsGenParams {
            num_resources: 3,
            num_scopes: 2,
            num_logs: 5,
        };
        let (otap, _) = gen_logs_otap(&params);

        for compressor in Compressor::ALL {
            let codec = NestedParquetCodec { compressor };
            let bytes = codec.write(otap.clone()).expect("write");
            let decoded = codec.read(&bytes).expect("read");
            assert_logs_equivalent(&otap, &decoded, codec.name(), compressor.label());
        }
    }
}
