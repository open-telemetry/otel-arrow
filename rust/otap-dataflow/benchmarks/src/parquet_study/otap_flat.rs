// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Efficient "OTAP-flat": present the four OTAP logs record batches as a single
//! columnar view at minimal conversion cost.
//!
//! The flattened-Parquet contenders in [`nested`](super::nested) pay a large
//! "flatten tax" because [`gather_by_parent`](super::attrs::gather_by_parent)
//! builds a `HashMap<u16, Vec<u32>>` per attribute batch and materializes every
//! value column with a random-access `take`. That work is avoidable: the OTAP
//! encoder emits each attribute batch already **grouped by `parent_id` in
//! ascending, contiguous runs** (`LogAttrs.parent_id = [0..,1..,2..]`,
//! `ResourceAttrs.parent_id = [0,1,2]`, `Logs.id` sequential). This module
//! exploits that ordering.
//!
//! All three variants share one move: the per-row **log attributes** become a
//! `List<Struct>` whose struct children are the *existing* `LogAttrs` value
//! columns (zero-copy) and whose offsets come from a single linear scan of the
//! sorted `parent_id`. No hash join, no `take`.
//!
//! They differ only in how the *shared* resource and scope attributes (one set
//! per resource/scope, spanning a contiguous run of log rows) are presented:
//!
//! - [`Layout::Materialized`] repeats each set physically per row (a plain
//!   `List<Struct>`, the same shape [`nested`](super::nested) produces, so it is
//!   directly Parquet-writable).
//! - [`Layout::RunEndEncoded`] stores each set once as `RunEndEncoded<i32,
//!   List<Struct>>` (logical per-row, physical one-per-resource). In-memory /
//!   query form only: arrow-rs cannot write REE to Parquet.
//! - [`Layout::Dictionary`] stores each set once as `Dictionary<u16,
//!   List<Struct>>` (per-row u16 index + one list per resource). Also in-memory
//!   only: arrow-rs cannot write a dictionary of `List<Struct>` to Parquet.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, DictionaryArray, Int32Array, ListArray, RecordBatch, RunArray, StructArray,
    UInt16Array, UInt32Array,
};
use arrow::buffer::{OffsetBuffer, ScalarBuffer};
use arrow::compute::take;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, UInt16Type};

use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use super::StudyResult;
use super::attrs::{
    LOG_ATTRS_COL, RESOURCE_ATTRS_COL, SCOPE_ATTRS_COL, as_list, attr_list_element_field,
    attr_struct_fields, empty_attr_list_column, entries_dedup, entries_per_row,
    extract_attr_value_arrays, logs_batch, logs_id, logs_resource_id, logs_scope_id,
    rebuild_attr_batch,
};

/// How the shared resource/scope attribute sets are presented in the flat view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Physically repeat each set per log row (`List<Struct>`). Parquet-writable.
    Materialized,
    /// `RunEndEncoded<i32, List<Struct>>`: one physical set per resource/scope.
    RunEndEncoded,
    /// `Dictionary<u16, List<Struct>>`: per-row index + one set per resource.
    Dictionary,
}

impl Layout {
    /// Stable contender name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Layout::Materialized => "otap-flat-materialized",
            Layout::RunEndEncoded => "otap-flat-ree",
            Layout::Dictionary => "otap-flat-dict",
        }
    }

    /// Whether the arrow-rs Parquet writer can serialize this layout directly.
    /// Only [`Layout::Materialized`] can; REE and dictionary-of-`List<Struct>`
    /// are in-memory / query representations (see module docs).
    #[must_use]
    pub fn parquet_writable(self) -> bool {
        matches!(self, Layout::Materialized)
    }
}

/// Downcast an attribute batch's `parent_id` column to `UInt16Array`.
fn parent_id_u16(attr_batch: &RecordBatch) -> StudyResult<UInt16Array> {
    let col = attr_batch
        .column_by_name(otap_df_pdata::schema::consts::PARENT_ID)
        .ok_or("attribute batch missing `parent_id`")?;
    Ok(col
        .as_any()
        .downcast_ref::<UInt16Array>()
        .ok_or("`parent_id` is not UInt16")?
        .clone())
}

/// The eight-field attribute `Struct` over *all* rows of an attribute batch,
/// built zero-copy from its value columns (no `take`).
fn attr_struct_all(attr_batch: &RecordBatch) -> StudyResult<StructArray> {
    let values = extract_attr_value_arrays(attr_batch)?;
    Ok(StructArray::new(attr_struct_fields(), values, None))
}

/// Given an attribute batch sorted by `parent_id`, return a `List<Struct>` with
/// one row per distinct parent (its contiguous run, zero-copy) plus the parent
/// id of each row. This is the compact "one set per resource/scope" form.
fn per_parent_lists(attr_batch: &RecordBatch) -> StudyResult<(ListArray, Vec<u16>)> {
    let struct_all = attr_struct_all(attr_batch)?;
    let parent = parent_id_u16(attr_batch)?;
    let n = parent.len();

    let mut offsets: Vec<i32> = vec![0];
    let mut parent_ids: Vec<u16> = Vec::new();
    if n > 0 {
        let mut start = 0usize;
        for i in 1..n {
            if parent.value(i) != parent.value(i - 1) {
                offsets.push(i32::try_from(i).expect("offset fits i32"));
                parent_ids.push(parent.value(start));
                start = i;
            }
        }
        offsets.push(i32::try_from(n).expect("offset fits i32"));
        parent_ids.push(parent.value(start));
    }

    let list = ListArray::new(
        attr_list_element_field(),
        OffsetBuffer::new(ScalarBuffer::from(offsets)),
        Arc::new(struct_all),
        None,
    );
    Ok((list, parent_ids))
}

/// The zero-copy per-row log-attribute `List<Struct>` column. Struct children
/// are the `LogAttrs` value columns unchanged; offsets come from one linear scan
/// aligning the sorted `LogAttrs.parent_id` runs with the sequential, nullable
/// `Logs.id` column. No hash join, no `take`.
fn log_attr_list(otap: &OtapArrowRecords, logs: &RecordBatch) -> StudyResult<ArrayRef> {
    let num_rows = logs.num_rows();
    let Some(attr_batch) = otap.get(ArrowPayloadType::LogAttrs) else {
        return Ok(empty_attr_list_column(num_rows));
    };
    let struct_all = attr_struct_all(attr_batch)?;
    let parent = parent_id_u16(attr_batch)?;
    let log_id = logs_id(logs)?;

    // Run length of each distinct parent, in ascending parent order.
    let n = parent.len();
    let mut run_lengths: Vec<i32> = Vec::new();
    if n > 0 {
        let mut start = 0usize;
        for i in 1..n {
            if parent.value(i) != parent.value(i - 1) {
                run_lengths.push(i32::try_from(i - start).expect("len fits i32"));
                start = i;
            }
        }
        run_lengths.push(i32::try_from(n - start).expect("len fits i32"));
    }

    // Each log row with a valid id consumes the next run, in order (the encoder
    // assigns ids sequentially in the same order it emits the runs).
    let mut offsets: Vec<i32> = Vec::with_capacity(num_rows + 1);
    offsets.push(0);
    let mut acc = 0i32;
    let mut next_run = 0usize;
    for i in 0..num_rows {
        if log_id.is_valid(i) {
            acc += run_lengths.get(next_run).copied().unwrap_or(0);
            next_run += 1;
        }
        offsets.push(acc);
    }
    debug_assert_eq!(
        next_run,
        run_lengths.len(),
        "every LogAttrs run should map to exactly one log row with an id"
    );

    Ok(Arc::new(ListArray::new(
        attr_list_element_field(),
        OffsetBuffer::new(ScalarBuffer::from(offsets)),
        Arc::new(struct_all),
        None,
    )))
}

/// The contiguous runs of a dense (non-null) run-structured id column: the id of
/// each run and the exclusive end row index of each run.
fn runs(id_arr: &UInt16Array) -> (Vec<u16>, Vec<i32>) {
    let n = id_arr.len();
    let mut ids = Vec::new();
    let mut ends = Vec::new();
    if n > 0 {
        for i in 1..n {
            if id_arr.value(i) != id_arr.value(i - 1) {
                ids.push(id_arr.value(i - 1));
                ends.push(i32::try_from(i).expect("end fits i32"));
            }
        }
        ids.push(id_arr.value(n - 1));
        ends.push(i32::try_from(n).expect("end fits i32"));
    }
    (ids, ends)
}

/// Byte span (start,end) of each parent's run in the flattened struct, keyed by
/// parent id. The map is sized by the number of distinct resources/scopes (tiny)
/// -- not by the attribute-row count -- so it is not the join `HashMap` the
/// baseline flatten pays.
fn spans_by_id(list: &ListArray, parent_ids: &[u16]) -> HashMap<u16, (i32, i32)> {
    let offs = list.value_offsets();
    parent_ids
        .iter()
        .enumerate()
        .map(|(j, &pid)| (pid, (offs[j], offs[j + 1])))
        .collect()
}

/// Build the shared resource/scope attribute column for a given [`Layout`] from
/// the per-parent lists and the log rows' run structure.
fn shared_column(
    layout: Layout,
    per_parent: &ListArray,
    parent_ids: &[u16],
    id_arr: &UInt16Array,
) -> StudyResult<ArrayRef> {
    let (run_ids, run_ends) = runs(id_arr);
    let spans = spans_by_id(per_parent, parent_ids);
    let base_struct = per_parent
        .values()
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or("per-parent values are not a struct")?;

    match layout {
        Layout::Materialized => {
            // Repeat each run's set across its rows: gather indices row by row.
            let num_rows = id_arr.len();
            let mut idx: Vec<u32> = Vec::new();
            let mut offsets: Vec<i32> = Vec::with_capacity(num_rows + 1);
            offsets.push(0);
            let mut run = 0usize;
            for i in 0..num_rows {
                if run + 1 < run_ends.len() && i32::try_from(i).expect("fits") >= run_ends[run] {
                    run += 1;
                }
                let (s, e) = spans.get(&run_ids[run]).copied().unwrap_or((0, 0));
                for k in s..e {
                    idx.push(u32::try_from(k).expect("index fits u32"));
                }
                offsets.push(i32::try_from(idx.len()).expect("offset fits i32"));
            }
            let taken = take_struct(base_struct, &idx)?;
            Ok(Arc::new(ListArray::new(
                attr_list_element_field(),
                OffsetBuffer::new(ScalarBuffer::from(offsets)),
                Arc::new(taken),
                None,
            )))
        }
        Layout::RunEndEncoded => {
            // One list per run; run_ends give the logical row spans.
            let values = align_runs(base_struct, &spans, &run_ids)?;
            let run_ends = Int32Array::from(run_ends);
            let ree = RunArray::<Int32Type>::try_new(&run_ends, &values)?;
            Ok(Arc::new(ree))
        }
        Layout::Dictionary => {
            // keys index into the per-parent lists by position; values are those
            // lists unchanged (zero-copy).
            let pos_of_id: HashMap<u16, i32> = parent_ids
                .iter()
                .enumerate()
                .map(|(j, &pid)| (pid, i32::try_from(j).expect("fits")))
                .collect();
            let keys: UInt16Array = (0..id_arr.len())
                .map(|i| {
                    let id = id_arr.value(i);
                    u16::try_from(*pos_of_id.get(&id).unwrap_or(&0)).expect("key fits u16")
                })
                .collect();
            let dict = DictionaryArray::<UInt16Type>::try_new(
                keys,
                Arc::new(per_parent.clone()) as ArrayRef,
            )?;
            Ok(Arc::new(dict))
        }
    }
}

/// One `List<Struct>` per run (in run order), each holding that run's resource
/// set (or empty if the resource carried no attributes).
fn align_runs(
    base_struct: &StructArray,
    spans: &HashMap<u16, (i32, i32)>,
    run_ids: &[u16],
) -> StudyResult<ArrayRef> {
    let mut idx: Vec<u32> = Vec::new();
    let mut offsets: Vec<i32> = vec![0];
    for rid in run_ids {
        let (s, e) = spans.get(rid).copied().unwrap_or((0, 0));
        for k in s..e {
            idx.push(u32::try_from(k).expect("index fits u32"));
        }
        offsets.push(i32::try_from(idx.len()).expect("offset fits i32"));
    }
    let taken = take_struct(base_struct, &idx)?;
    Ok(Arc::new(ListArray::new(
        attr_list_element_field(),
        OffsetBuffer::new(ScalarBuffer::from(offsets)),
        Arc::new(taken),
        None,
    )))
}

/// `take` each field of a struct by the given indices (used only for the tiny
/// resource/scope sets, never for the per-row log attributes).
fn take_struct(base: &StructArray, idx: &[u32]) -> StudyResult<StructArray> {
    let indices = UInt32Array::from(idx.to_vec());
    let cols: Vec<ArrayRef> = base
        .columns()
        .iter()
        .map(|c| take(c, &indices, None))
        .collect::<Result<_, _>>()?;
    Ok(StructArray::new(attr_struct_fields(), cols, None))
}

/// The flat-table field for a shared attribute column of the given layout.
fn shared_field(name: &str, array: &ArrayRef) -> Field {
    Field::new(name, array.data_type().clone(), array.is_nullable())
}

/// Flatten an OTAP logs batch into a single columnar view with the given layout.
pub fn flatten(otap: &OtapArrowRecords, layout: Layout) -> StudyResult<RecordBatch> {
    let logs = logs_batch(otap)?;
    let resource_id = logs_resource_id(logs)?;
    let scope_id = logs_scope_id(logs)?;

    let mut fields: Vec<Field> = logs
        .schema()
        .fields()
        .iter()
        .map(|f| f.as_ref().clone())
        .collect();
    let mut columns: Vec<ArrayRef> = logs.columns().to_vec();

    // Shared resource/scope columns.
    for (name, payload, id_arr) in [
        (
            RESOURCE_ATTRS_COL,
            ArrowPayloadType::ResourceAttrs,
            &resource_id,
        ),
        (SCOPE_ATTRS_COL, ArrowPayloadType::ScopeAttrs, &scope_id),
    ] {
        let col: ArrayRef = match otap.get(payload) {
            Some(attr_batch) => {
                let (per_parent, parent_ids) = per_parent_lists(attr_batch)?;
                shared_column(layout, &per_parent, &parent_ids, id_arr)?
            }
            None => empty_attr_list_column(logs.num_rows()),
        };
        fields.push(shared_field(name, &col));
        columns.push(col);
    }

    // Per-row log attributes (zero-copy for every layout).
    let log_col = log_attr_list(otap, logs)?;
    fields.push(shared_field(LOG_ATTRS_COL, &log_col));
    columns.push(log_col);

    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}

/// Read a shared attribute column back to a `List<Struct>` of one row per
/// distinct resource/scope plus the id to stamp on each, regardless of layout.
fn shared_lists(flat: &RecordBatch, name: &str, id_arr: &UInt16Array) -> StudyResult<RecordBatch> {
    let column = flat
        .column_by_name(name)
        .ok_or_else(|| format!("flat batch missing `{name}`"))?;

    match column.data_type() {
        DataType::List(_) => {
            // Materialized: per-row lists; dedup to first row per id.
            rebuild_attr_batch(as_list(flat, name)?, &entries_dedup(id_arr))
        }
        DataType::RunEndEncoded(_, _) => {
            let ree = column
                .as_any()
                .downcast_ref::<RunArray<Int32Type>>()
                .ok_or("expected RunEndEncoded<i32>")?;
            let list = ree
                .values()
                .as_any()
                .downcast_ref::<ListArray>()
                .ok_or("REE values are not a list")?;
            let (run_ids, _) = runs(id_arr);
            let entries: Vec<(usize, u16)> =
                run_ids.iter().enumerate().map(|(j, &id)| (j, id)).collect();
            rebuild_attr_batch(list, &entries)
        }
        DataType::Dictionary(_, _) => {
            let dict = column
                .as_any()
                .downcast_ref::<DictionaryArray<UInt16Type>>()
                .ok_or("expected Dictionary<u16>")?;
            let list = dict
                .values()
                .as_any()
                .downcast_ref::<ListArray>()
                .ok_or("dictionary values are not a list")?;
            // The per-parent lists are in ascending id order; distinct ids in
            // ascending order recover the stamp for each list row.
            let (mut ids, _) = runs(id_arr);
            ids.sort_unstable();
            ids.dedup();
            let entries: Vec<(usize, u16)> =
                ids.iter().enumerate().map(|(j, &id)| (j, id)).collect();
            rebuild_attr_batch(list, &entries)
        }
        other => Err(format!("unexpected shared column type {other:?}").into()),
    }
}

/// Reconstruct an OTAP logs batch from a flat view (any layout).
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

    let resource_attrs = shared_lists(flat, RESOURCE_ATTRS_COL, &resource_id)?;
    let scope_attrs = shared_lists(flat, SCOPE_ATTRS_COL, &scope_id)?;
    let log_attrs = rebuild_attr_batch(as_list(flat, LOG_ATTRS_COL)?, &entries_per_row(&log_id))?;

    let mut otap = OtapArrowRecords::Logs(Default::default());
    otap.set(ArrowPayloadType::Logs, logs)?;
    otap.set(ArrowPayloadType::ResourceAttrs, resource_attrs)?;
    otap.set(ArrowPayloadType::ScopeAttrs, scope_attrs)?;
    otap.set(ArrowPayloadType::LogAttrs, log_attrs)?;
    Ok(otap)
}

/// Sum of the in-memory footprint of every column in the flat view.
#[must_use]
pub fn in_memory_bytes(flat: &RecordBatch) -> usize {
    flat.columns()
        .iter()
        .map(|c| c.get_array_memory_size())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parquet_study::attrs::assert_logs_equivalent;
    use crate::parquet_study::datagen::{LogsGenParams, gen_logs_otap};

    #[test]
    fn all_layouts_round_trip() {
        let params = LogsGenParams {
            num_resources: 3,
            num_scopes: 2,
            num_logs: 7,
        };
        let (otap, _) = gen_logs_otap(&params);

        for layout in [
            Layout::Materialized,
            Layout::RunEndEncoded,
            Layout::Dictionary,
        ] {
            let flat = flatten(&otap, layout).expect("flatten");
            assert_eq!(flat.num_rows(), params.total_logs(), "{}", layout.name());
            let decoded = unflatten(&flat).expect("unflatten");
            assert_logs_equivalent(&otap, &decoded, layout.name(), "n/a");
        }
    }

    #[test]
    fn resource_heavy_round_trips() {
        use crate::parquet_study::datagen::{RichGenParams, gen_logs_otap_rich};

        let params = RichGenParams {
            label: "test",
            num_resources: 40,
            num_scopes: 3,
            num_logs: 5,
            num_resource_attrs: 12,
            num_scope_attrs: 4,
            num_log_attrs: 3,
        };
        let otap = gen_logs_otap_rich(&params);

        for layout in [
            Layout::Materialized,
            Layout::RunEndEncoded,
            Layout::Dictionary,
        ] {
            let flat = flatten(&otap, layout).expect("flatten");
            assert_eq!(flat.num_rows(), params.total_logs(), "{}", layout.name());
            let decoded = unflatten(&flat).expect("unflatten");
            assert_logs_equivalent(&otap, &decoded, layout.name(), "n/a");
        }
    }

    #[test]
    fn ree_and_dict_are_smaller_in_memory_than_materialized() {
        let params = LogsGenParams {
            num_resources: 1,
            num_scopes: 1,
            num_logs: 20_000,
        };
        let (otap, _) = gen_logs_otap(&params);

        let mat = in_memory_bytes(&flatten(&otap, Layout::Materialized).expect("mat"));
        let ree = in_memory_bytes(&flatten(&otap, Layout::RunEndEncoded).expect("ree"));
        let dict = in_memory_bytes(&flatten(&otap, Layout::Dictionary).expect("dict"));

        assert!(ree < mat, "REE {ree} not smaller than materialized {mat}");
        assert!(
            dict < mat,
            "dict {dict} not smaller than materialized {mat}"
        );
    }
}
