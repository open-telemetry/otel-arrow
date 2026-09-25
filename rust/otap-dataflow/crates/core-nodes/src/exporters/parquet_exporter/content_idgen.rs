// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Content-hash ID generator for the parquet exporter.
//!
//! Instead of minting sequential per-partition IDs (see [`super::idgen`]), this generator
//! derives IDs from the *content* of attribute sets, following the design of ClickHouse's
//! TimeSeries engine (id = hash of metric + label set; see ClickHouse#100906/#112799):
//!
//! - attribute-set id: hash over the canonicalized (sorted) key/value pairs of one parent's
//!   attribute rows. Rows in the attrs tables are keyed by this hash, so the attrs tables
//!   become dimension tables with one logical row per *distinct* set, not per record.
//! - metrics hub id: hash over (resource set, scope set, name, type, unit, temporality,
//!   metric attr set) — one id per series group, stable across batches and process restarts.
//! - data point id: series id = hash(hub id, dp attr set). All points of one series share it,
//!   giving the (parent_id, id) column pair the locality-vs-identity structure of ClickHouse's
//!   multi-component TimeSeries ids.
//!
//! Because ids are content-derived they are globally stable: cross-table joins no longer
//! need `_part_id` scoping, and replays regenerate identical dimension rows. Attribute-set
//! and hub rows already emitted are deduplicated via a `seen` cache that resets on UTC day
//! change, so every `date=` partition remains self-contained (safe under prefix lifecycle
//! expiry) while repeated sets within a day are written once.
//!
//! ID columns are widened to `UInt64` (including the nested `resource.id`/`scope.id` struct
//! fields, which the sequence generator leaves untouched).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use arrow::array::{
    Array, ArrayRef, BinaryArray, BooleanArray, Float64Array, Int32Array, Int64Array,
    RecordBatch, StringArray, StructArray, UInt8Array, UInt32Array, UInt64Array,
};
use arrow::compute::{cast, filter_record_batch};
use arrow::datatypes::{DataType, Field, Schema};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{consts, update_schema_metadata};
use uuid::Uuid;

use super::error::ParquetExporterError;
use super::idgen::{PARTITION_METADATA_KEY, PartitionSequenceIdGenerator};
use super::records::OtapParquetRecords;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

// domain tags keep hashes from different id spaces from colliding/deduping across tables
const DOMAIN_ATTR_SET: u8 = 1;
const DOMAIN_METRICS_HUB: u8 = 2;
const DOMAIN_SERIES: u8 = 3;

// safety valve: if an attrs stream turns out to be per-record (unbounded sets), cap the
// dedup cache; clearing only costs re-emitting duplicates, never correctness
const MAX_SEEN_PER_TABLE: usize = 4_000_000;

struct Fnv(u64);

impl Fnv {
    fn new(domain: u8) -> Self {
        let mut f = Fnv(FNV_OFFSET);
        f.write(&[domain]);
        f
    }

    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 ^= u64::from(*b);
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }

    fn write_u64(&mut self, v: u64) {
        self.write(&v.to_le_bytes());
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

pub struct ContentHashIdGenerator {
    part_id: Uuid,
    seen: HashMap<ArrowPayloadType, HashSet<u64>>,
    seen_day: u64,
    // traces are not content-addressed (no attr-set-per-record repetition to exploit);
    // fall back to the sequence generator for them
    traces_fallback: PartitionSequenceIdGenerator,
}

impl Default for ContentHashIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentHashIdGenerator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            part_id: Uuid::new_v4(),
            seen: HashMap::new(),
            seen_day: current_utc_day(),
            traces_fallback: PartitionSequenceIdGenerator::new(),
        }
    }

    pub fn generate_unique_ids(
        &mut self,
        otap_batch: &mut OtapParquetRecords,
    ) -> Result<(), ParquetExporterError> {
        // reset dedup caches on UTC day change so each date= partition carries a full copy
        // of its active dimension rows (self-contained under retention expiry)
        let day = current_utc_day();
        if day != self.seen_day {
            self.seen.clear();
            self.seen_day = day;
        }

        match otap_batch {
            OtapParquetRecords::Logs(_) => {
                self.generate_logs(otap_batch)?;
                self.stamp_partition_key(otap_batch, ArrowPayloadType::Logs);
            }
            OtapParquetRecords::Metrics(_) => {
                self.generate_metrics(otap_batch)?;
                self.stamp_partition_key(otap_batch, ArrowPayloadType::UnivariateMetrics);
            }
            OtapParquetRecords::Traces(_) => {
                return self.traces_fallback.generate_unique_ids(otap_batch);
            }
        }

        Ok(())
    }

    /// Stamp the writer-run partition key like the sequence generator does; with content
    /// ids it only namespaces files on disk — joins no longer depend on it.
    fn stamp_partition_key(
        &self,
        otap_batch: &mut OtapParquetRecords,
        root: ArrowPayloadType,
    ) {
        if let Some(main_rb) = otap_batch.get(root) {
            let stamped = update_schema_metadata(
                main_rb,
                PARTITION_METADATA_KEY.into(),
                format!("{}", self.part_id),
            );
            otap_batch.set(root, stamped);
        }
    }

    fn generate_logs(
        &mut self,
        otap_batch: &mut OtapParquetRecords,
    ) -> Result<(), ParquetExporterError> {
        let r_map = attr_set_hashes(otap_batch.get(ArrowPayloadType::ResourceAttrs))?;
        let s_map = attr_set_hashes(otap_batch.get(ArrowPayloadType::ScopeAttrs))?;
        let l_map = attr_set_hashes(otap_batch.get(ArrowPayloadType::LogAttrs))?;

        if let Some(rb) = otap_batch.get(ArrowPayloadType::Logs) {
            let rb = map_struct_id(rb, consts::RESOURCE, &r_map)?;
            let rb = map_struct_id(&rb, consts::SCOPE, &s_map)?;
            let rb = map_id_column(&rb, consts::ID, &l_map)?;
            otap_batch.set(ArrowPayloadType::Logs, rb);
        }

        for (payload_type, map) in [
            (ArrowPayloadType::ResourceAttrs, &r_map),
            (ArrowPayloadType::ScopeAttrs, &s_map),
            (ArrowPayloadType::LogAttrs, &l_map),
        ] {
            self.remap_and_dedup_attrs(otap_batch, payload_type, map)?;
        }

        Ok(())
    }

    fn generate_metrics(
        &mut self,
        otap_batch: &mut OtapParquetRecords,
    ) -> Result<(), ParquetExporterError> {
        let r_map = attr_set_hashes(otap_batch.get(ArrowPayloadType::ResourceAttrs))?;
        let s_map = attr_set_hashes(otap_batch.get(ArrowPayloadType::ScopeAttrs))?;
        let m_map = attr_set_hashes(otap_batch.get(ArrowPayloadType::MetricAttrs))?;

        let hub_type = if otap_batch.get(ArrowPayloadType::UnivariateMetrics).is_some() {
            ArrowPayloadType::UnivariateMetrics
        } else {
            ArrowPayloadType::MultivariateMetrics
        };

        // old hub id -> content-derived hub id
        let hub_map = match otap_batch.get(hub_type) {
            Some(rb) => hub_id_map(rb, &r_map, &s_map, &m_map)?,
            None => HashMap::new(),
        };

        if let Some(rb) = otap_batch.get(hub_type) {
            let rb = map_struct_id(rb, consts::RESOURCE, &r_map)?;
            let rb = map_struct_id(&rb, consts::SCOPE, &s_map)?;
            let rb = map_id_column(&rb, consts::ID, &hub_map)?;
            let rb = self.dedup_by_u64_column(hub_type, &rb, consts::ID)?;
            otap_batch.set(hub_type, rb);
        }

        // metric_attrs reference the hub row (its new id already folds in the attr set)
        self.remap_and_dedup_attrs(otap_batch, ArrowPayloadType::MetricAttrs, &hub_map)?;
        // resource/scope attrs are keyed by their set hashes, matching the struct ids
        self.remap_and_dedup_attrs(otap_batch, ArrowPayloadType::ResourceAttrs, &r_map)?;
        self.remap_and_dedup_attrs(otap_batch, ArrowPayloadType::ScopeAttrs, &s_map)?;

        for (dp_type, dp_attrs_type, exemplar_type) in [
            (
                ArrowPayloadType::NumberDataPoints,
                ArrowPayloadType::NumberDpAttrs,
                Some(ArrowPayloadType::NumberDpExemplars),
            ),
            (
                ArrowPayloadType::SummaryDataPoints,
                ArrowPayloadType::SummaryDpAttrs,
                None,
            ),
            (
                ArrowPayloadType::HistogramDataPoints,
                ArrowPayloadType::HistogramDpAttrs,
                Some(ArrowPayloadType::HistogramDpExemplars),
            ),
            (
                ArrowPayloadType::ExpHistogramDataPoints,
                ArrowPayloadType::ExpHistogramDpAttrs,
                Some(ArrowPayloadType::ExpHistogramDpExemplars),
            ),
        ] {
            let dpa_map = attr_set_hashes(otap_batch.get(dp_attrs_type))?;

            // old dp id -> series id (hash of hub id + dp attr set)
            let mut series_map: HashMap<u32, u64> = HashMap::new();
            if let Some(rb) = otap_batch.get(dp_type) {
                let ids = col_as_u32(rb, consts::ID)?;
                let parents = col_as_u32(rb, consts::PARENT_ID)?;
                if let (Some(ids), Some(parents)) = (&ids, &parents) {
                    for i in 0..rb.num_rows() {
                        if ids.is_null(i) || parents.is_null(i) {
                            continue;
                        }
                        let hub_id = match hub_map.get(&parents.value(i)) {
                            Some(h) => *h,
                            None => continue,
                        };
                        let mut f = Fnv::new(DOMAIN_SERIES);
                        f.write_u64(hub_id);
                        f.write_u64(dpa_map.get(&ids.value(i)).copied().unwrap_or(0));
                        let _ = series_map.insert(ids.value(i), f.finish());
                    }
                }

                let rb = map_id_column(rb, consts::PARENT_ID, &hub_map)?;
                let rb = map_id_column(&rb, consts::ID, &series_map)?;
                otap_batch.set(dp_type, rb);
            }

            self.remap_and_dedup_attrs(otap_batch, dp_attrs_type, &series_map)?;

            // exemplars are facts, not dimensions: remap the parent, never dedup. They
            // attach to the series rather than the individual point.
            if let Some(exemplar_type) = exemplar_type {
                if let Some(rb) = otap_batch.get(exemplar_type) {
                    let rb = map_id_column(rb, consts::PARENT_ID, &series_map)?;
                    otap_batch.set(exemplar_type, rb);
                }
            }
        }

        Ok(())
    }

    fn remap_and_dedup_attrs(
        &mut self,
        otap_batch: &mut OtapParquetRecords,
        payload_type: ArrowPayloadType,
        map: &HashMap<u32, u64>,
    ) -> Result<(), ParquetExporterError> {
        if let Some(rb) = otap_batch.get(payload_type) {
            let rb = map_id_column(rb, consts::PARENT_ID, map)?;
            let rb = self.dedup_by_u64_column(payload_type, &rb, consts::PARENT_ID)?;
            otap_batch.set(payload_type, rb);
        }
        Ok(())
    }

    /// Drop rows whose u64 key column value was already emitted for this payload type
    /// (since the last daily cache reset). Rows with null keys are kept.
    fn dedup_by_u64_column(
        &mut self,
        payload_type: ArrowPayloadType,
        rb: &RecordBatch,
        column_name: &str,
    ) -> Result<RecordBatch, ParquetExporterError> {
        let column = match rb.column_by_name(column_name) {
            Some(c) => c,
            None => return Ok(rb.clone()),
        };
        let keys = column
            .as_any()
            .downcast_ref::<UInt64Array>()
            .ok_or_else(|| ParquetExporterError::InvalidRecordBatch {
                error: format!("expected UInt64 column '{column_name}' for dedup"),
            })?;

        let seen = self.seen.entry(payload_type).or_default();
        if seen.len() > MAX_SEEN_PER_TABLE {
            seen.clear();
        }

        let mut newly: HashSet<u64> = HashSet::new();
        let mut any_dropped = false;
        let mask = BooleanArray::from_iter((0..keys.len()).map(|i| {
            if keys.is_null(i) {
                return Some(true);
            }
            let k = keys.value(i);
            if seen.contains(&k) {
                any_dropped = true;
                Some(false)
            } else {
                let _ = newly.insert(k);
                Some(true)
            }
        }));
        seen.extend(newly);

        if !any_dropped {
            return Ok(rb.clone());
        }
        filter_record_batch(rb, &mask).map_err(|e| ParquetExporterError::InvalidRecordBatch {
            error: format!("failed to filter deduplicated rows: {e}"),
        })
    }
}

fn current_utc_day() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0)
}

/// Compute per-parent content hashes for an attrs record batch: group rows by parent_id,
/// canonicalize each group (sorted row encodings) and hash it.
fn attr_set_hashes(
    rb: Option<&RecordBatch>,
) -> Result<HashMap<u32, u64>, ParquetExporterError> {
    let rb = match rb {
        Some(rb) if rb.num_rows() > 0 => rb,
        _ => return Ok(HashMap::new()),
    };

    let parents = col_as_u32(rb, consts::PARENT_ID)?.ok_or_else(|| {
        ParquetExporterError::InvalidRecordBatch {
            error: "attrs batch is missing the parent_id column".to_string(),
        }
    })?;

    let keys = col_as_string(rb, consts::ATTRIBUTE_KEY)?;
    let types = col_as_u8(rb, consts::ATTRIBUTE_TYPE)?;
    let strs = col_as_string(rb, consts::ATTRIBUTE_STR)?;
    let ints = col_as::<Int64Array>(rb, consts::ATTRIBUTE_INT, &DataType::Int64)?;
    let doubles = col_as::<Float64Array>(rb, consts::ATTRIBUTE_DOUBLE, &DataType::Float64)?;
    let bools = col_as::<BooleanArray>(rb, consts::ATTRIBUTE_BOOL, &DataType::Boolean)?;
    let bytes = col_as::<BinaryArray>(rb, consts::ATTRIBUTE_BYTES, &DataType::Binary)?;
    let sers = col_as::<BinaryArray>(rb, consts::ATTRIBUTE_SER, &DataType::Binary)?;

    let mut groups: HashMap<u32, Vec<Vec<u8>>> = HashMap::new();
    for i in 0..rb.num_rows() {
        if parents.is_null(i) {
            continue;
        }

        let mut enc: Vec<u8> = Vec::with_capacity(32);
        if let Some(keys) = &keys {
            if keys.is_valid(i) {
                enc.extend_from_slice(keys.value(i).as_bytes());
            }
        }
        enc.push(0xfe);
        if let Some(types) = &types {
            if types.is_valid(i) {
                enc.push(types.value(i));
            }
        }
        enc.push(0xfe);
        if let Some(strs) = &strs {
            if strs.is_valid(i) {
                enc.push(1);
                enc.extend_from_slice(strs.value(i).as_bytes());
            }
        }
        enc.push(0xfe);
        if let Some(ints) = &ints {
            if ints.is_valid(i) {
                enc.push(1);
                enc.extend_from_slice(&ints.value(i).to_le_bytes());
            }
        }
        enc.push(0xfe);
        if let Some(doubles) = &doubles {
            if doubles.is_valid(i) {
                enc.push(1);
                enc.extend_from_slice(&doubles.value(i).to_bits().to_le_bytes());
            }
        }
        enc.push(0xfe);
        if let Some(bools) = &bools {
            if bools.is_valid(i) {
                enc.push(if bools.value(i) { 2 } else { 1 });
            }
        }
        enc.push(0xfe);
        if let Some(bytes) = &bytes {
            if bytes.is_valid(i) {
                enc.push(1);
                enc.extend_from_slice(bytes.value(i));
            }
        }
        enc.push(0xfe);
        if let Some(sers) = &sers {
            if sers.is_valid(i) {
                enc.push(1);
                enc.extend_from_slice(sers.value(i));
            }
        }

        groups.entry(parents.value(i)).or_default().push(enc);
    }

    let mut result = HashMap::with_capacity(groups.len());
    for (parent, mut rows) in groups {
        rows.sort_unstable();
        let mut f = Fnv::new(DOMAIN_ATTR_SET);
        for row in &rows {
            f.write(row);
            f.write(&[0xff]);
        }
        let _ = result.insert(parent, f.finish());
    }
    Ok(result)
}

/// Build the map from old hub (metrics) ids to content-derived hub ids.
fn hub_id_map(
    rb: &RecordBatch,
    r_map: &HashMap<u32, u64>,
    s_map: &HashMap<u32, u64>,
    m_map: &HashMap<u32, u64>,
) -> Result<HashMap<u32, u64>, ParquetExporterError> {
    let ids = match col_as_u32(rb, consts::ID)? {
        Some(ids) => ids,
        None => return Ok(HashMap::new()),
    };

    let resource_ids = struct_child_as_u32(rb, consts::RESOURCE)?;
    let scope_ids = struct_child_as_u32(rb, consts::SCOPE)?;
    let metric_types = col_as_u8(rb, consts::METRIC_TYPE)?;
    let names = col_as_string(rb, consts::NAME)?;
    let units = col_as_string(rb, consts::UNIT)?;
    let temporalities = col_as::<Int32Array>(
        rb,
        consts::AGGREGATION_TEMPORALITY,
        &DataType::Int32,
    )?;

    let mut result = HashMap::with_capacity(rb.num_rows());
    for i in 0..rb.num_rows() {
        if ids.is_null(i) {
            continue;
        }

        let mut f = Fnv::new(DOMAIN_METRICS_HUB);
        let resource_hash = resource_ids
            .as_ref()
            .filter(|a| a.is_valid(i))
            .and_then(|a| r_map.get(&a.value(i)))
            .copied()
            .unwrap_or(0);
        f.write_u64(resource_hash);
        let scope_hash = scope_ids
            .as_ref()
            .filter(|a| a.is_valid(i))
            .and_then(|a| s_map.get(&a.value(i)))
            .copied()
            .unwrap_or(0);
        f.write_u64(scope_hash);
        if let Some(metric_types) = &metric_types {
            if metric_types.is_valid(i) {
                f.write(&[metric_types.value(i)]);
            }
        }
        f.write(&[0xfe]);
        if let Some(names) = &names {
            if names.is_valid(i) {
                f.write(names.value(i).as_bytes());
            }
        }
        f.write(&[0xfe]);
        if let Some(units) = &units {
            if units.is_valid(i) {
                f.write(units.value(i).as_bytes());
            }
        }
        f.write(&[0xfe]);
        if let Some(temporalities) = &temporalities {
            if temporalities.is_valid(i) {
                f.write(&temporalities.value(i).to_le_bytes());
            }
        }
        f.write_u64(m_map.get(&ids.value(i)).copied().unwrap_or(0));

        let _ = result.insert(ids.value(i), f.finish());
    }
    Ok(result)
}

/// Replace a top-level id column with the UInt64 values it maps to (null when the source
/// value is null or unmapped). Batches without the column pass through unchanged.
fn map_id_column(
    rb: &RecordBatch,
    column_name: &str,
    map: &HashMap<u32, u64>,
) -> Result<RecordBatch, ParquetExporterError> {
    let index = match rb.schema_ref().index_of(column_name) {
        Ok(index) => index,
        Err(_) => return Ok(rb.clone()),
    };

    let old = col_as_u32(rb, column_name)?.ok_or_else(|| {
        ParquetExporterError::InvalidRecordBatch {
            error: format!("could not read id column '{column_name}' as UInt32"),
        }
    })?;

    let new_column: ArrayRef = Arc::new(UInt64Array::from_iter((0..old.len()).map(|i| {
        if old.is_null(i) {
            None
        } else {
            map.get(&old.value(i)).copied()
        }
    })));

    replace_column(rb, index, column_name, new_column)
}

/// Replace the `id` child inside a struct column (resource/scope) with mapped UInt64 values.
fn map_struct_id(
    rb: &RecordBatch,
    struct_name: &str,
    map: &HashMap<u32, u64>,
) -> Result<RecordBatch, ParquetExporterError> {
    let index = match rb.schema_ref().index_of(struct_name) {
        Ok(index) => index,
        Err(_) => return Ok(rb.clone()),
    };
    let struct_array = match rb.column(index).as_any().downcast_ref::<StructArray>() {
        Some(s) => s,
        None => return Ok(rb.clone()),
    };
    let (child_index, _) = match struct_array.fields().find(consts::ID) {
        Some(found) => found,
        None => return Ok(rb.clone()),
    };

    let old = cast(struct_array.column(child_index), &DataType::UInt32).map_err(|e| {
        ParquetExporterError::InvalidRecordBatch {
            error: format!("could not cast {struct_name}.id to UInt32: {e}"),
        }
    })?;
    let old = old
        .as_any()
        .downcast_ref::<UInt32Array>()
        .expect("cast to UInt32 yields UInt32Array");

    let new_child: ArrayRef = Arc::new(UInt64Array::from_iter((0..old.len()).map(|i| {
        if old.is_null(i) {
            None
        } else {
            map.get(&old.value(i)).copied()
        }
    })));

    let mut new_fields = Vec::with_capacity(struct_array.fields().len());
    let mut new_children: Vec<ArrayRef> = Vec::with_capacity(struct_array.fields().len());
    for (i, field) in struct_array.fields().iter().enumerate() {
        if i == child_index {
            new_fields.push(Arc::new(Field::new(consts::ID, DataType::UInt64, true)));
            new_children.push(new_child.clone());
        } else {
            new_fields.push(field.clone());
            new_children.push(struct_array.column(i).clone());
        }
    }
    let new_struct = StructArray::new(
        new_fields.into(),
        new_children,
        struct_array.nulls().cloned(),
    );

    replace_column(rb, index, struct_name, Arc::new(new_struct))
}

fn replace_column(
    rb: &RecordBatch,
    index: usize,
    name: &str,
    new_column: ArrayRef,
) -> Result<RecordBatch, ParquetExporterError> {
    let schema = rb.schema_ref();
    let new_columns = rb
        .columns()
        .iter()
        .enumerate()
        .map(|(i, col)| {
            if i == index {
                new_column.clone()
            } else {
                col.clone()
            }
        })
        .collect::<Vec<_>>();
    let new_fields = schema
        .fields()
        .iter()
        .enumerate()
        .map(|(i, field)| {
            if i == index {
                Arc::new(Field::new(name, new_column.data_type().clone(), true))
            } else {
                field.clone()
            }
        })
        .collect::<Vec<_>>();
    let new_schema =
        Arc::new(Schema::new(new_fields).with_metadata(schema.metadata().clone()));
    RecordBatch::try_new(new_schema, new_columns).map_err(|e| {
        ParquetExporterError::InvalidRecordBatch {
            error: format!("failed to rebuild record batch with mapped '{name}': {e}"),
        }
    })
}

fn col_as_u32(
    rb: &RecordBatch,
    name: &str,
) -> Result<Option<UInt32Array>, ParquetExporterError> {
    match rb.column_by_name(name) {
        Some(col) => {
            let col = cast(col, &DataType::UInt32).map_err(|e| {
                ParquetExporterError::InvalidRecordBatch {
                    error: format!("could not cast column '{name}' to UInt32: {e}"),
                }
            })?;
            Ok(Some(
                col.as_any()
                    .downcast_ref::<UInt32Array>()
                    .expect("cast to UInt32 yields UInt32Array")
                    .clone(),
            ))
        }
        None => Ok(None),
    }
}

fn struct_child_as_u32(
    rb: &RecordBatch,
    struct_name: &str,
) -> Result<Option<UInt32Array>, ParquetExporterError> {
    let struct_array = match rb
        .column_by_name(struct_name)
        .and_then(|c| c.as_any().downcast_ref::<StructArray>())
    {
        Some(s) => s,
        None => return Ok(None),
    };
    let child = match struct_array.column_by_name(consts::ID) {
        Some(c) => c,
        None => return Ok(None),
    };
    let col = cast(child, &DataType::UInt32).map_err(|e| {
        ParquetExporterError::InvalidRecordBatch {
            error: format!("could not cast {struct_name}.id to UInt32: {e}"),
        }
    })?;
    Ok(Some(
        col.as_any()
            .downcast_ref::<UInt32Array>()
            .expect("cast to UInt32 yields UInt32Array")
            .clone(),
    ))
}

fn col_as_string(
    rb: &RecordBatch,
    name: &str,
) -> Result<Option<StringArray>, ParquetExporterError> {
    match rb.column_by_name(name) {
        Some(col) => {
            let col = cast(col, &DataType::Utf8).map_err(|e| {
                ParquetExporterError::InvalidRecordBatch {
                    error: format!("could not cast column '{name}' to Utf8: {e}"),
                }
            })?;
            Ok(Some(
                col.as_any()
                    .downcast_ref::<StringArray>()
                    .expect("cast to Utf8 yields StringArray")
                    .clone(),
            ))
        }
        None => Ok(None),
    }
}

fn col_as_u8(
    rb: &RecordBatch,
    name: &str,
) -> Result<Option<UInt8Array>, ParquetExporterError> {
    match rb.column_by_name(name) {
        Some(col) => {
            let col = cast(col, &DataType::UInt8).map_err(|e| {
                ParquetExporterError::InvalidRecordBatch {
                    error: format!("could not cast column '{name}' to UInt8: {e}"),
                }
            })?;
            Ok(Some(
                col.as_any()
                    .downcast_ref::<UInt8Array>()
                    .expect("cast to UInt8 yields UInt8Array")
                    .clone(),
            ))
        }
        None => Ok(None),
    }
}

fn col_as<T: Clone + 'static>(
    rb: &RecordBatch,
    name: &str,
    data_type: &DataType,
) -> Result<Option<T>, ParquetExporterError> {
    match rb.column_by_name(name) {
        Some(col) => {
            let col = cast(col, data_type).map_err(|e| {
                ParquetExporterError::InvalidRecordBatch {
                    error: format!("could not cast column '{name}' to {data_type}: {e}"),
                }
            })?;
            Ok(Some(
                col.as_any()
                    .downcast_ref::<T>()
                    .expect("cast yields requested array type")
                    .clone(),
            ))
        }
        None => Ok(None),
    }
}
