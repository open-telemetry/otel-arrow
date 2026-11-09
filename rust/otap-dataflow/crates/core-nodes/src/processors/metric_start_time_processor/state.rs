// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-series state cache and zero-allocation attribute multiset hashing for
//! the `metric_start_time` processor (`subtract_initial_point` strategy).

use std::time::{Duration, Instant};

use arrow::array::{
    Array, BooleanArray, Float64Array, Int64Array, RecordBatch, StringArray, UInt8Array,
    UInt16Array, UInt32Array,
};
use hashbrown::HashMap;
use hashbrown::hash_map::EntryRef;
use otel_arrow_dfe_pdata::arrays::{
    ByteArrayAccessor, MaybeDictArrayAccessor, NullableArrayAccessor, get_required_array,
};
use otel_arrow_dfe_pdata::error::Error as PdataError;
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::schema::consts;
use xxhash_rust::xxh3::xxh3_128;

/// Upper bound for direct flat-vector indexing of `parent_id` values before falling
/// back to the sparse hash map. OTAP batches assign dense IDs starting at 0, so
/// normal batches remain entirely within the flat vector.
const DENSE_ID_LIMIT: usize = 65_536;

/// Order-independent 128-bit multiset attribute hasher for OTAP `Attributes` record batches.
///
/// Each attribute row `(key, type, value)` is hashed with `xxh3_128` into a pair of 64-bit words
/// `(lo, hi)` and accumulated into the slot for `parent_id` via wrapping 64-bit addition
/// `(acc_lo.wrapping_add(lo), acc_hi.wrapping_add(hi))`. Because 64-bit addition is commutative
/// and associative, attributes for an entity can appear in any key order or row order without
/// sorting or per-entity allocations.
#[derive(Debug, Default)]
pub struct AttrHasher {
    dense: Vec<(u64, u64)>,
    max_dense_id: Option<usize>,
    sparse: HashMap<u32, (u64, u64)>,
    scratch_buf: Vec<u8>,
}

impl AttrHasher {
    /// Creates a new reusable attribute hasher.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears accumulated hashes from the previous table while retaining allocated capacity.
    pub fn clear(&mut self) {
        if let Some(max_id) = self.max_dense_id.take() {
            let end = (max_id + 1).min(self.dense.len());
            self.dense[..end].fill((0, 0));
        }
        self.sparse.clear();
    }

    /// Returns the 128-bit attribute multiset hash for `parent_id`, or `0` if `parent_id` has no
    /// attributes in the populated batch.
    #[inline]
    #[must_use]
    pub fn get(&self, parent_id: u32) -> u128 {
        let idx = parent_id as usize;
        let (lo, hi) = if idx < DENSE_ID_LIMIT {
            self.dense.get(idx).copied().unwrap_or((0, 0))
        } else {
            self.sparse.get(&parent_id).copied().unwrap_or((0, 0))
        };
        ((hi as u128) << 64) | (lo as u128)
    }

    #[inline]
    fn add_hash(&mut self, parent_id: u32, lo: u64, hi: u64) {
        let idx = parent_id as usize;
        if idx < DENSE_ID_LIMIT {
            if idx >= self.dense.len() {
                self.dense.resize(idx + 1, (0, 0));
            }
            self.max_dense_id = Some(self.max_dense_id.map_or(idx, |cur| cur.max(idx)));
            let slot = &mut self.dense[idx];
            slot.0 = slot.0.wrapping_add(lo);
            slot.1 = slot.1.wrapping_add(hi);
        } else {
            let slot = self.sparse.entry(parent_id).or_insert((0, 0));
            slot.0 = slot.0.wrapping_add(lo);
            slot.1 = slot.1.wrapping_add(hi);
        }
    }

    /// Populates attribute hashes from a 16-bit `parent_id` attribute batch (`ResourceAttrs`).
    pub fn populate_u16(&mut self, attrs_rb: Option<&RecordBatch>) -> Result<(), PdataError> {
        self.clear();
        let Some(rb) = attrs_rb else {
            return Ok(());
        };
        if rb.num_rows() == 0 {
            return Ok(());
        }
        let parent_id_arr = get_required_array(rb, consts::PARENT_ID)?;
        let parent_ids = MaybeDictArrayAccessor::<UInt16Array>::try_new(parent_id_arr)?;
        let cols = AttrColumns::try_new(rb)?;

        for row in 0..rb.num_rows() {
            let Some(parent_id) = parent_ids.value_at(row) else {
                continue;
            };
            let (lo, hi) = cols.hash_row(row, &mut self.scratch_buf);
            self.add_hash(u32::from(parent_id), lo, hi);
        }
        Ok(())
    }

    /// Populates attribute hashes from a 32-bit `parent_id` attribute batch (`*DpAttrs`).
    pub fn populate_u32(&mut self, attrs_rb: Option<&RecordBatch>) -> Result<(), PdataError> {
        self.clear();
        let Some(rb) = attrs_rb else {
            return Ok(());
        };
        if rb.num_rows() == 0 {
            return Ok(());
        }
        let parent_id_arr = get_required_array(rb, consts::PARENT_ID)?;
        let parent_ids = MaybeDictArrayAccessor::<UInt32Array>::try_new(parent_id_arr)?;
        let cols = AttrColumns::try_new(rb)?;

        for row in 0..rb.num_rows() {
            let Some(parent_id) = parent_ids.value_at(row) else {
                continue;
            };
            let (lo, hi) = cols.hash_row(row, &mut self.scratch_buf);
            self.add_hash(parent_id, lo, hi);
        }
        Ok(())
    }
}

struct AttrColumns<'a> {
    keys: MaybeDictArrayAccessor<'a, StringArray>,
    types: &'a UInt8Array,
    strs: Option<MaybeDictArrayAccessor<'a, StringArray>>,
    ints: Option<MaybeDictArrayAccessor<'a, Int64Array>>,
    doubles: Option<&'a Float64Array>,
    bools: Option<&'a BooleanArray>,
    bytes: Option<ByteArrayAccessor<'a>>,
    sers: Option<ByteArrayAccessor<'a>>,
}

impl<'a> AttrColumns<'a> {
    fn try_new(rb: &'a RecordBatch) -> Result<Self, PdataError> {
        let key_arr = get_required_array(rb, consts::ATTRIBUTE_KEY)?;
        let keys = MaybeDictArrayAccessor::<StringArray>::try_new(key_arr)?;
        let type_arr = get_required_array(rb, consts::ATTRIBUTE_TYPE)?;
        let types = type_arr
            .as_any()
            .downcast_ref::<UInt8Array>()
            .ok_or_else(|| PdataError::ColumnDataTypeMismatch {
                name: consts::ATTRIBUTE_TYPE.into(),
                actual: type_arr.data_type().clone(),
                expect: arrow::datatypes::DataType::UInt8,
            })?;

        let strs = rb
            .column_by_name(consts::ATTRIBUTE_STR)
            .map(MaybeDictArrayAccessor::<StringArray>::try_new)
            .transpose()?;
        let ints = rb
            .column_by_name(consts::ATTRIBUTE_INT)
            .map(MaybeDictArrayAccessor::<Int64Array>::try_new)
            .transpose()?;
        let doubles = rb
            .column_by_name(consts::ATTRIBUTE_DOUBLE)
            .and_then(|c| c.as_any().downcast_ref::<Float64Array>());
        let bools = rb
            .column_by_name(consts::ATTRIBUTE_BOOL)
            .and_then(|c| c.as_any().downcast_ref::<BooleanArray>());
        let bytes = rb
            .column_by_name(consts::ATTRIBUTE_BYTES)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;
        let sers = rb
            .column_by_name(consts::ATTRIBUTE_SER)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;

        Ok(Self {
            keys,
            types,
            strs,
            ints,
            doubles,
            bools,
            bytes,
            sers,
        })
    }

    #[inline]
    fn hash_row(&self, row: usize, buf: &mut Vec<u8>) -> (u64, u64) {
        buf.clear();
        let val_type = if self.types.is_valid(row) {
            self.types.value(row)
        } else {
            0
        };
        let key = self.keys.str_at(row).unwrap_or("");
        buf.push(val_type);
        buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
        buf.extend_from_slice(key.as_bytes());

        match AttributeValueType::try_from(val_type) {
            Ok(AttributeValueType::Str) => {
                if let Some(s) = self.strs.as_ref().and_then(|c| c.str_at(row)) {
                    buf.extend_from_slice(s.as_bytes());
                }
            }
            Ok(AttributeValueType::Int) => {
                let v = self
                    .ints
                    .as_ref()
                    .and_then(|c| c.value_at(row))
                    .unwrap_or(0);
                buf.extend_from_slice(&v.to_le_bytes());
            }
            Ok(AttributeValueType::Double) => {
                let v = self
                    .doubles
                    .and_then(|c| c.is_valid(row).then(|| c.value(row)))
                    .unwrap_or(0.0);
                buf.extend_from_slice(&v.to_bits().to_le_bytes());
            }
            Ok(AttributeValueType::Bool) => {
                let v = self
                    .bools
                    .and_then(|c| c.is_valid(row).then(|| c.value(row)))
                    .unwrap_or(false);
                buf.push(u8::from(v));
            }
            Ok(AttributeValueType::Bytes) => {
                if let Some(b) = self.bytes.as_ref().and_then(|c| c.slice_at(row)) {
                    buf.extend_from_slice(b);
                }
            }
            Ok(AttributeValueType::Map | AttributeValueType::Slice) => {
                if let Some(b) = self.sers.as_ref().and_then(|c| c.slice_at(row)) {
                    buf.extend_from_slice(b);
                }
            }
            Ok(AttributeValueType::Empty) | Err(_) => {}
        }

        let h = xxh3_128(buf);
        let mut lo = h as u64;
        let hi = (h >> 64) as u64;
        if lo == 0 && hi == 0 {
            lo = 1;
        }
        (lo, hi)
    }
}

/// Cached reference and previous values for a cumulative `Sum` number series.
#[derive(Debug, Clone, Copy)]
pub struct NumberSeriesState {
    pub mark: bool,
    pub start_time_ns: i64,
    pub ref_double: f64,
    pub ref_int: i64,
    pub prev_double: f64,
    pub prev_int: i64,
}

/// Cached reference and previous values for a `Summary` series.
#[derive(Debug, Clone, Copy)]
pub struct SummarySeriesState {
    pub mark: bool,
    pub start_time_ns: i64,
    pub ref_count: u64,
    pub ref_sum: f64,
    pub prev_count: u64,
    pub prev_sum: f64,
}

/// Cached reference and previous values for a cumulative `Histogram` series.
#[derive(Debug, Clone)]
pub struct HistogramSeriesState {
    pub mark: bool,
    pub start_time_ns: i64,
    pub ref_count: u64,
    pub ref_sum: f64,
    pub prev_count: u64,
    pub prev_sum: f64,
    /// Bit representations (`f64::to_bits`) of the explicit bucket bounds.
    pub explicit_bounds_bits: Vec<u64>,
    pub ref_buckets: Vec<u64>,
    pub prev_buckets: Vec<u64>,
}

/// Cached bucket offset and counts for one side (positive or negative) of an exponential histogram.
#[derive(Debug, Clone, Default)]
pub struct ExpBucketsState {
    pub offset: i32,
    pub counts: Vec<u64>,
}

/// Cached reference and previous values for a cumulative `ExponentialHistogram` series.
#[derive(Debug, Clone)]
pub struct ExpHistogramSeriesState {
    pub mark: bool,
    pub start_time_ns: i64,
    pub scale: i32,
    pub ref_count: u64,
    pub ref_sum: f64,
    pub ref_zero_count: u64,
    pub prev_count: u64,
    pub prev_sum: f64,
    pub prev_zero_count: u64,
    pub ref_positive: ExpBucketsState,
    pub prev_positive: ExpBucketsState,
    pub ref_negative: ExpBucketsState,
    pub prev_negative: ExpBucketsState,
}

/// Per-resource series state maps partitioned by data point table type so each inner hash table
/// stores compact, unboxed state structs.
#[derive(Debug, Default)]
pub struct ResourceSeriesMap {
    pub mark: bool,
    pub numbers: HashMap<String, HashMap<u128, NumberSeriesState>>,
    pub summaries: HashMap<String, HashMap<u128, SummarySeriesState>>,
    pub histograms: HashMap<String, HashMap<u128, HistogramSeriesState>>,
    pub exp_histograms: HashMap<String, HashMap<u128, ExpHistogramSeriesState>>,
}

impl ResourceSeriesMap {
    fn sweep(&mut self) -> bool {
        Self::sweep_table(&mut self.numbers, |s| &mut s.mark);
        Self::sweep_table(&mut self.summaries, |s| &mut s.mark);
        Self::sweep_table(&mut self.histograms, |s| &mut s.mark);
        Self::sweep_table(&mut self.exp_histograms, |s| &mut s.mark);

        !self.numbers.is_empty()
            || !self.summaries.is_empty()
            || !self.histograms.is_empty()
            || !self.exp_histograms.is_empty()
    }

    fn sweep_table<S>(
        table: &mut HashMap<String, HashMap<u128, S>>,
        mut mark_mut: impl FnMut(&mut S) -> &mut bool,
    ) {
        table.retain(|_, series_map| {
            series_map.retain(|_, state| std::mem::replace(mark_mut(state), false));
            !series_map.is_empty()
        });
    }
}

/// Two-level (`resource_hash -> metric_name -> dp_attr_hash`) state cache with mark-and-sweep GC.
#[derive(Debug)]
pub struct SubtractInitialCache {
    gc_interval: Duration,
    last_gc: Instant,
    resources: HashMap<u128, ResourceSeriesMap>,
}

impl SubtractInitialCache {
    /// Creates a new cache with the given mark-and-sweep garbage collection interval.
    #[must_use]
    pub fn new(gc_interval: Duration, now: Instant) -> Self {
        Self {
            gc_interval,
            last_gc: now,
            resources: HashMap::new(),
        }
    }

    /// Returns a mutable reference to the per-series state map for a cumulative `Sum` metric.
    #[inline]
    pub fn number_series_mut(
        &mut self,
        resource_hash: u128,
        metric_name: &str,
    ) -> &mut HashMap<u128, NumberSeriesState> {
        let res = self.resources.entry(resource_hash).or_default();
        res.mark = true;
        match res.numbers.entry_ref(metric_name) {
            EntryRef::Occupied(entry) => entry.into_mut(),
            EntryRef::Vacant(entry) => entry.insert(HashMap::new()),
        }
    }

    /// Returns a mutable reference to the per-series state map for a `Summary` metric.
    #[inline]
    pub fn summary_series_mut(
        &mut self,
        resource_hash: u128,
        metric_name: &str,
    ) -> &mut HashMap<u128, SummarySeriesState> {
        let res = self.resources.entry(resource_hash).or_default();
        res.mark = true;
        match res.summaries.entry_ref(metric_name) {
            EntryRef::Occupied(entry) => entry.into_mut(),
            EntryRef::Vacant(entry) => entry.insert(HashMap::new()),
        }
    }

    /// Returns a mutable reference to the per-series state map for a cumulative `Histogram` metric.
    #[inline]
    pub fn histogram_series_mut(
        &mut self,
        resource_hash: u128,
        metric_name: &str,
    ) -> &mut HashMap<u128, HistogramSeriesState> {
        let res = self.resources.entry(resource_hash).or_default();
        res.mark = true;
        match res.histograms.entry_ref(metric_name) {
            EntryRef::Occupied(entry) => entry.into_mut(),
            EntryRef::Vacant(entry) => entry.insert(HashMap::new()),
        }
    }

    /// Returns a mutable reference to the per-series state map for a cumulative
    /// `ExponentialHistogram` metric.
    #[inline]
    pub fn exp_histogram_series_mut(
        &mut self,
        resource_hash: u128,
        metric_name: &str,
    ) -> &mut HashMap<u128, ExpHistogramSeriesState> {
        let res = self.resources.entry(resource_hash).or_default();
        res.mark = true;
        match res.exp_histograms.entry_ref(metric_name) {
            EntryRef::Occupied(entry) => entry.into_mut(),
            EntryRef::Vacant(entry) => entry.insert(HashMap::new()),
        }
    }

    /// Runs a mark-and-sweep garbage collection pass if `gc_interval` has elapsed since `last_gc`.
    pub fn maybe_gc(&mut self, now: Instant) {
        if now.saturating_duration_since(self.last_gc) <= self.gc_interval {
            return;
        }
        self.resources.retain(|_, res_map| {
            if !res_map.mark {
                return false;
            }
            res_map.mark = false;
            res_map.sweep()
        });
        self.last_gc = now;
    }

    #[cfg(test)]
    pub(crate) fn resource_count(&self) -> usize {
        self.resources.len()
    }
}
