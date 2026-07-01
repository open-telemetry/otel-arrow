// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Wide / exploded flattened-Parquet contender.
//!
//! Each distinct attribute key becomes its own top-level, typed Parquet column,
//! prefixed by its group (`resource.<key>`, `scope.<key>`, `log.<key>`). The
//! column's Arrow type is chosen from the key's first-seen OTAP value type
//! (string -> Utf8, int -> Int64, double -> Float64, bool -> Boolean,
//! bytes/map/slice -> Binary, empty -> a Boolean presence marker). A row is null
//! in a column when it lacks that attribute. This is the "analytics-flat" layout
//! that compresses and queries well column-by-column.
//!
//! To stay lossless and keep round-trip counts exact even when a key appears
//! with more than one value type (which the single typed column cannot hold),
//! any attribute that does not fit its typed column is spilled into a per-group
//! `List<Struct{key,type,...}>` overflow column. With type-consistent keys (the
//! common case, and what this study generates) the overflow columns are empty.
//!
//! Every added column carries field metadata describing its group, key, and
//! OTAP type, so decode is fully self-describing after a Parquet round-trip.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BinaryArray, BinaryBuilder, BooleanArray, BooleanBuilder, Float64Array,
    Float64Builder, Int64Array, Int64Builder, ListArray, RecordBatch, StringArray, StringBuilder,
    StructArray, UInt8Array,
};
use arrow::datatypes::{Field, Schema};

use otap_df_pdata::encode::record::attributes::AttributesRecordBatchBuilder;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use super::attrs::{
    Gathered, build_attr_list_column, extract_attr_value_arrays, gather_by_parent, logs_batch,
    logs_id, logs_resource_id, logs_scope_id,
};
use super::{Codec, Compressor, StudyResult, parquet_io};

// OTAP attribute value type discriminators (see AttributeValueType).
const T_STR: u8 = 1;
const T_INT: u8 = 2;
const T_DOUBLE: u8 = 3;
const T_BOOL: u8 = 4;
const T_MAP: u8 = 5;
const T_SLICE: u8 = 6;
const T_BYTES: u8 = 7;

const MD_GROUP: &str = "wide_group";
const MD_KEY: &str = "wide_key";
const MD_TYPE: &str = "wide_otap_type";
const MD_OVERFLOW: &str = "wide_overflow";

/// Contender that flattens OTAP logs into a single Parquet file using one typed
/// column per distinct attribute key.
pub struct WideParquetCodec {
    /// Parquet compression codec.
    pub compressor: Compressor,
}

/// Group identifier used in column prefixes and metadata.
#[derive(Clone, Copy)]
struct Group {
    name: &'static str,
    prefix: &'static str,
    payload: ArrowPayloadType,
    overflow_col: &'static str,
}

const GROUPS: [Group; 3] = [
    Group {
        name: "resource",
        prefix: "resource.",
        payload: ArrowPayloadType::ResourceAttrs,
        overflow_col: "resource_overflow",
    },
    Group {
        name: "scope",
        prefix: "scope.",
        payload: ArrowPayloadType::ScopeAttrs,
        overflow_col: "scope_overflow",
    },
    Group {
        name: "log",
        prefix: "log.",
        payload: ArrowPayloadType::LogAttrs,
        overflow_col: "log_overflow",
    },
];

/// Borrowed, typed view over the eight value columns of a source attribute
/// batch (`key, type, str, int, double, bool, bytes, ser`).
struct SourceAttrs {
    key: ArrayRef,
    atype: ArrayRef,
    values: Vec<ArrayRef>, // str,int,double,bool,bytes,ser in canonical order
}

impl SourceAttrs {
    fn new(attr_batch: &RecordBatch) -> StudyResult<Self> {
        let arrays = extract_attr_value_arrays(attr_batch)?;
        Ok(Self {
            key: arrays[0].clone(),
            atype: arrays[1].clone(),
            values: arrays[2..].to_vec(),
        })
    }

    fn key_at(&self, row: usize) -> &str {
        self.key
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("key is Utf8")
            .value(row)
    }

    fn type_at(&self, row: usize) -> u8 {
        self.atype
            .as_any()
            .downcast_ref::<UInt8Array>()
            .expect("type is UInt8")
            .value(row)
    }

    fn str(&self) -> &StringArray {
        self.values[0].as_any().downcast_ref().expect("str Utf8")
    }
    fn int(&self) -> &Int64Array {
        self.values[1].as_any().downcast_ref().expect("int Int64")
    }
    fn double(&self) -> &Float64Array {
        self.values[2]
            .as_any()
            .downcast_ref()
            .expect("double Float64")
    }
    fn boolean(&self) -> &BooleanArray {
        self.values[3]
            .as_any()
            .downcast_ref()
            .expect("bool Boolean")
    }
    fn bytes(&self) -> &BinaryArray {
        self.values[4]
            .as_any()
            .downcast_ref()
            .expect("bytes Binary")
    }
    fn ser(&self) -> &BinaryArray {
        self.values[5].as_any().downcast_ref().expect("ser Binary")
    }
}

/// Build the typed Arrow column for one exploded key. `row_src[i]` is the source
/// attribute row that supplies row `i`'s value, or `None` if absent.
fn build_typed_column(otap_type: u8, src: &SourceAttrs, row_src: &[Option<usize>]) -> ArrayRef {
    match otap_type {
        T_STR => {
            let a = src.str();
            let mut b = StringBuilder::new();
            for r in row_src {
                match r {
                    Some(i) => b.append_value(a.value(*i)),
                    None => b.append_null(),
                }
            }
            Arc::new(b.finish())
        }
        T_INT => {
            let a = src.int();
            let mut b = Int64Builder::new();
            for r in row_src {
                match r {
                    Some(i) => b.append_value(a.value(*i)),
                    None => b.append_null(),
                }
            }
            Arc::new(b.finish())
        }
        T_DOUBLE => {
            let a = src.double();
            let mut b = Float64Builder::new();
            for r in row_src {
                match r {
                    Some(i) => b.append_value(a.value(*i)),
                    None => b.append_null(),
                }
            }
            Arc::new(b.finish())
        }
        T_BOOL => {
            let a = src.boolean();
            let mut b = BooleanBuilder::new();
            for r in row_src {
                match r {
                    Some(i) => b.append_value(a.value(*i)),
                    None => b.append_null(),
                }
            }
            Arc::new(b.finish())
        }
        T_BYTES => binary_column(src.bytes(), row_src),
        T_MAP | T_SLICE => binary_column(src.ser(), row_src),
        // Empty: a Boolean presence marker (value is meaningless).
        _ => {
            let mut b = BooleanBuilder::new();
            for r in row_src {
                match r {
                    Some(_) => b.append_value(true),
                    None => b.append_null(),
                }
            }
            Arc::new(b.finish())
        }
    }
}

fn binary_column(src: &BinaryArray, row_src: &[Option<usize>]) -> ArrayRef {
    let mut b = BinaryBuilder::new();
    for r in row_src {
        match r {
            Some(i) => b.append_value(src.value(*i)),
            None => b.append_null(),
        }
    }
    Arc::new(b.finish())
}

fn field_with_md(name: String, array: &ArrayRef, md: HashMap<String, String>) -> Field {
    Field::new(name, array.data_type().clone(), true).with_metadata(md)
}

/// Explode one attribute group into typed columns plus an overflow column.
fn explode_group(
    otap: &OtapArrowRecords,
    group: Group,
    parents: &arrow::array::UInt16Array,
    num_rows: usize,
) -> StudyResult<Vec<(Field, ArrayRef)>> {
    let Some(attr_batch) = otap.get(group.payload) else {
        // No attributes for this group: emit only an empty overflow column.
        let overflow = super::attrs::empty_attr_list_column(num_rows);
        let mut md = HashMap::new();
        let _ = md.insert(MD_OVERFLOW.to_string(), group.name.to_string());
        return Ok(vec![(
            field_with_md(group.overflow_col.to_string(), &overflow, md),
            overflow,
        )]);
    };

    let src = SourceAttrs::new(attr_batch)?;
    let gathered = gather_by_parent(attr_batch, parents)?;

    // Discover columns: first-seen OTAP type per key, ordered deterministically.
    let mut col_type: BTreeMap<String, u8> = BTreeMap::new();
    for row in 0..attr_batch.num_rows() {
        let key = src.key_at(row).to_string();
        let _ = col_type.entry(key).or_insert_with(|| src.type_at(row));
    }
    let keys: Vec<String> = col_type.keys().cloned().collect();
    let key_ordinal: HashMap<&str, usize> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (k.as_str(), i))
        .collect();

    // For each column, the source row supplying each log row's value.
    let mut per_col_row_src: Vec<Vec<Option<usize>>> = vec![vec![None; num_rows]; keys.len()];
    let mut overflow_indices: Vec<u32> = Vec::new();
    let mut overflow_offsets: Vec<i32> = Vec::with_capacity(num_rows + 1);
    overflow_offsets.push(0);

    // `row` indexes per_col_row_src, gathered.offsets, and overflow tracking.
    #[allow(clippy::needless_range_loop)]
    for row in 0..num_rows {
        let start = gathered.offsets[row] as usize;
        let end = gathered.offsets[row + 1] as usize;
        for slot in start..end {
            let s = gathered.indices.value(slot) as usize;
            let key = src.key_at(s);
            let t = src.type_at(s);
            let ord = key_ordinal[key];
            if col_type[key] == t && per_col_row_src[ord][row].is_none() {
                per_col_row_src[ord][row] = Some(s);
            } else {
                overflow_indices.push(u32::try_from(s).expect("index fits u32"));
            }
        }
        overflow_offsets.push(i32::try_from(overflow_indices.len()).expect("offset fits i32"));
    }

    let mut out: Vec<(Field, ArrayRef)> = Vec::with_capacity(keys.len() + 1);
    for (ord, key) in keys.iter().enumerate() {
        let t = col_type[key];
        let array = build_typed_column(t, &src, &per_col_row_src[ord]);
        let mut md = HashMap::new();
        let _ = md.insert(MD_GROUP.to_string(), group.name.to_string());
        let _ = md.insert(MD_KEY.to_string(), key.clone());
        let _ = md.insert(MD_TYPE.to_string(), t.to_string());
        let name = format!("{}{}", group.prefix, key);
        out.push((field_with_md(name, &array, md), array));
    }

    // Overflow column (empty for type-consistent keys).
    let overflow_gathered = Gathered {
        indices: arrow::array::UInt32Array::from(overflow_indices),
        offsets: overflow_offsets,
    };
    let overflow = build_attr_list_column(attr_batch, &overflow_gathered)?;
    let mut md = HashMap::new();
    let _ = md.insert(MD_OVERFLOW.to_string(), group.name.to_string());
    out.push((
        field_with_md(group.overflow_col.to_string(), &overflow, md),
        overflow,
    ));

    Ok(out)
}

/// Flatten an OTAP logs batch into the wide flat record batch.
pub fn flatten(otap: &OtapArrowRecords) -> StudyResult<RecordBatch> {
    let logs = logs_batch(otap)?;
    let num_rows = logs.num_rows();
    let resource_id = logs_resource_id(logs)?;
    let scope_id = logs_scope_id(logs)?;
    let log_id = logs_id(logs)?;
    let parents = [resource_id, scope_id, log_id];

    let mut fields: Vec<Field> = logs
        .schema()
        .fields()
        .iter()
        .map(|f| f.as_ref().clone())
        .collect();
    let mut columns: Vec<ArrayRef> = logs.columns().to_vec();

    for (group, parent) in GROUPS.iter().zip(parents.iter()) {
        for (field, array) in explode_group(otap, *group, parent, num_rows)? {
            fields.push(field);
            columns.push(array);
        }
    }

    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}

fn append_typed(
    builder: &mut AttributesRecordBatchBuilder<u16>,
    otap_type: u8,
    array: &ArrayRef,
    row: usize,
) {
    match otap_type {
        T_STR => builder
            .any_values_builder
            .append_str(downcast::<StringArray>(array).value(row).as_bytes()),
        T_INT => builder
            .any_values_builder
            .append_int(downcast::<Int64Array>(array).value(row)),
        T_DOUBLE => builder
            .any_values_builder
            .append_double(downcast::<Float64Array>(array).value(row)),
        T_BOOL => builder
            .any_values_builder
            .append_bool(downcast::<BooleanArray>(array).value(row)),
        T_BYTES => builder
            .any_values_builder
            .append_bytes(downcast::<BinaryArray>(array).value(row)),
        T_MAP => builder
            .any_values_builder
            .append_map(downcast::<BinaryArray>(array).value(row)),
        T_SLICE => builder
            .any_values_builder
            .append_slice(downcast::<BinaryArray>(array).value(row)),
        _ => builder.any_values_builder.append_empty(),
    }
}

fn downcast<T: 'static>(array: &ArrayRef) -> &T {
    array
        .as_any()
        .downcast_ref::<T>()
        .expect("wide column has expected type")
}

fn append_overflow_row(
    builder: &mut AttributesRecordBatchBuilder<u16>,
    parent_id: u16,
    values: &StructArray,
    elem: usize,
) {
    let key = downcast::<StringArray>(values.column(0)).value(elem);
    let t = downcast::<UInt8Array>(values.column(1)).value(elem);
    builder.append_parent_id(&parent_id);
    builder.append_key(key.as_bytes());
    // struct columns: 2=str,3=int,4=double,5=bool,6=bytes,7=ser
    match t {
        T_STR => builder.any_values_builder.append_str(
            downcast::<StringArray>(values.column(2))
                .value(elem)
                .as_bytes(),
        ),
        T_INT => builder
            .any_values_builder
            .append_int(downcast::<Int64Array>(values.column(3)).value(elem)),
        T_DOUBLE => builder
            .any_values_builder
            .append_double(downcast::<Float64Array>(values.column(4)).value(elem)),
        T_BOOL => builder
            .any_values_builder
            .append_bool(downcast::<BooleanArray>(values.column(5)).value(elem)),
        T_BYTES => builder
            .any_values_builder
            .append_bytes(downcast::<BinaryArray>(values.column(6)).value(elem)),
        T_MAP => builder
            .any_values_builder
            .append_map(downcast::<BinaryArray>(values.column(7)).value(elem)),
        T_SLICE => builder
            .any_values_builder
            .append_slice(downcast::<BinaryArray>(values.column(7)).value(elem)),
        _ => builder.any_values_builder.append_empty(),
    }
}

struct WideColumn {
    key: String,
    otap_type: u8,
    array: ArrayRef,
}

/// Reconstruct an OTAP logs batch from the wide flat record batch.
pub fn unflatten(flat: &RecordBatch) -> StudyResult<OtapArrowRecords> {
    // Partition columns: plain Logs columns vs. wide attribute / overflow columns.
    let mut logs_fields: Vec<Field> = Vec::new();
    let mut logs_columns: Vec<ArrayRef> = Vec::new();
    let mut group_columns: HashMap<String, Vec<WideColumn>> = HashMap::new();
    let mut overflow_columns: HashMap<String, ListArray> = HashMap::new();

    for (field, column) in flat.schema().fields().iter().zip(flat.columns()) {
        let md = field.metadata();
        if let Some(group) = md.get(MD_OVERFLOW) {
            let list = column
                .as_any()
                .downcast_ref::<ListArray>()
                .ok_or("overflow column is not a list")?
                .clone();
            let _ = overflow_columns.insert(group.clone(), list);
        } else if let (Some(group), Some(key), Some(t)) =
            (md.get(MD_GROUP), md.get(MD_KEY), md.get(MD_TYPE))
        {
            group_columns
                .entry(group.clone())
                .or_default()
                .push(WideColumn {
                    key: key.clone(),
                    otap_type: t.parse().map_err(|_| "bad wide_otap_type")?,
                    array: column.clone(),
                });
        } else {
            logs_fields.push(field.as_ref().clone());
            logs_columns.push(column.clone());
        }
    }

    let logs = RecordBatch::try_new(Arc::new(Schema::new(logs_fields)), logs_columns)?;
    let resource_id = logs_resource_id(&logs)?;
    let scope_id = logs_scope_id(&logs)?;
    let log_id = logs_id(&logs)?;
    let ids = [resource_id, scope_id, log_id];

    let mut otap = OtapArrowRecords::Logs(Default::default());
    otap.set(ArrowPayloadType::Logs, logs)?;

    for (group, id_arr) in GROUPS.iter().zip(ids.iter()) {
        let entries = if group.name == "log" {
            super::attrs::entries_per_row(id_arr)
        } else {
            super::attrs::entries_dedup(id_arr)
        };
        let cols = group_columns.get(group.name);
        let overflow = overflow_columns.get(group.name);

        let mut builder = AttributesRecordBatchBuilder::<u16>::new();
        for (row, pid) in &entries {
            if let Some(cols) = cols {
                for col in cols {
                    if col.array.is_valid(*row) {
                        builder.append_parent_id(pid);
                        builder.append_key(col.key.as_bytes());
                        append_typed(&mut builder, col.otap_type, &col.array, *row);
                    }
                }
            }
            if let Some(list) = overflow {
                let values = list
                    .values()
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .ok_or("overflow values are not a struct")?;
                let offs = list.value_offsets();
                let (start, end) = (offs[*row] as usize, offs[*row + 1] as usize);
                for elem in start..end {
                    append_overflow_row(&mut builder, *pid, values, elem);
                }
            }
        }
        otap.set(group.payload, builder.finish()?)?;
    }

    Ok(otap)
}

impl Codec for WideParquetCodec {
    fn name(&self) -> &'static str {
        "parquet-wide"
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
    fn wide_round_trip_preserves_structure() {
        let params = LogsGenParams {
            num_resources: 3,
            num_scopes: 2,
            num_logs: 5,
        };
        let (otap, _) = gen_logs_otap(&params);

        for compressor in Compressor::ALL {
            let codec = WideParquetCodec { compressor };
            let bytes = codec.write(otap.clone()).expect("write");
            let decoded = codec.read(&bytes).expect("read");
            assert_logs_equivalent(&otap, &decoded, codec.name(), compressor.label());
        }
    }
}
