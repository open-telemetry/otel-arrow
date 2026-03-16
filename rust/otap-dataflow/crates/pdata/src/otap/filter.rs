// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{
    Array, ArrayRef, BooleanArray, BooleanBuilder, DictionaryArray, Float64Array, Int64Array,
    PrimitiveArray, RecordBatch, StringArray, UInt16Array, UInt32Array,
};
use arrow::datatypes::{ArrowPrimitiveType, DataType, UInt8Type, UInt16Type};
use roaring::RoaringBitmap;

use crate::arrays::get_required_array;
use crate::otap::OtapArrowRecords;
use crate::otap::error::{Error, Result};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use arrow::buffer::BooleanBuffer;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
pub mod logs;
pub mod traces;
// threshold numbers to determine which method to use for building id filter
// ToDo: determine optimimal numbers
const ID_COLUMN_LENGTH_MIN_THRESHOLD: usize = 2000;
const IDS_PERCENTAGE_MAX_THRESHOLD: f64 = 0.05;
const ID_SET_MAX_LENGTH_THRESHOLD: usize = 20;

// default boolean array length to use for filter if there is no record batch set
// when attempting to build a filter for a optional record batch
const NO_RECORD_BATCH_FILTER_SIZE: usize = 1;

/// Number of u64 words per page. Each page covers 65,536 IDs (one full u16 range).
const ID_BITMAP_PAGE_WORDS: usize = 1024;

/// Number of [`clear`](IdBitmap::clear) cycles a page can remain unused before being evicted
/// (deallocated). A threshold of 16 means a page that hasn't been touched in 16 consecutive
/// `clear()` calls will be freed, preventing unbounded memory growth from adversarial inputs
/// while avoiding thrashing for pages that are used intermittently.
const ID_BITMAP_PAGE_EVICTION_THRESHOLD: u64 = 16;

/// A single page of the [`IdBitmap`], covering 65,536 IDs (8 KiB of bitmap data).
///
/// Each page tracks the generation in which it was last written, enabling the bitmap to evict
/// pages that haven't been touched in several cycles.
struct IdBitmapPage {
    words: [u64; ID_BITMAP_PAGE_WORDS],
    last_used_generation: u64,
}

impl IdBitmapPage {
    /// Creates a new zeroed page stamped with the given generation.
    fn new(generation: u64) -> Self {
        Self {
            words: [0u64; ID_BITMAP_PAGE_WORDS],
            last_used_generation: generation,
        }
    }
}

/// A paged bitmap for fast membership testing of ID values.
///
/// The underlying bitmap data is heap allocated, and the intention of this type is that it
/// can be reused between batches by calling the `clear` method. This method is also called
/// automatically by the `populate` method, allowing the bitmap to be rewritten from some
/// input ID column.
///
/// The ID space is partitioned into pages of 65,536 IDs each (8 KiB per page). For the common
/// case of dense IDs starting near 0 (typical of OTAP batches), there few pages are allocated.
///
/// The motivation for the paged bitmap is to protect against adversarial situations where we
/// receive batches containing few, sparse IDs.
///
/// ## Page lifecycle
///
/// Each page tracks the generation (batch cycle) in which it was last written. On
/// [`clear`](IdBitmap::clear), the generation counter is incremented and pages are evaluated:
///
/// - Pages used within the last [`PAGE_EVICTION_THRESHOLD`] generations are zeroed and retained.
/// - Pages that haven't been used in more than [`PAGE_EVICTION_THRESHOLD`] generations are
///   deallocated, preventing unbounded memory growth from adversarial or unusual input patterns.
///
/// This means pages that are used regularly (even intermittently) stay allocated, while pages
/// from one-off anomalous batches are eventually freed.
pub struct IdBitmap {
    pages: Vec<Option<Box<IdBitmapPage>>>,
    generation: u64,
}

impl IdBitmap {
    /// Creates a new empty `IdBitmap`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            generation: 0,
        }
    }

    /// Clears all bits in the bitmap, evicting stale pages.
    pub fn clear(&mut self) {
        self.generation += 1;
        for page_slot in &mut self.pages {
            if let Some(page) = page_slot {
                if self.generation - page.last_used_generation > ID_BITMAP_PAGE_EVICTION_THRESHOLD {
                    *page_slot = None;
                } else {
                    page.words.fill(0);
                }
            }
        }
        // Trim trailing None slots to avoid unbounded growth of the outer vec
        while self.pages.last().is_some_and(|p| p.is_none()) {
            let _ = self.pages.pop();
        }
    }

    /// Returns the page index and bit position within the page for the given ID.
    #[inline]
    const fn page_and_bit(id: u32) -> (usize, usize) {
        let page_idx = (id >> 16) as usize;
        let bit_idx = (id & 0xFFFF) as usize;
        (page_idx, bit_idx)
    }

    /// Ensures the page for the given page index exists, allocating it if necessary,
    /// and stamps it with the current generation.
    #[inline]
    fn ensure_page(&mut self, page_idx: usize) -> &mut IdBitmapPage {
        if page_idx >= self.pages.len() {
            self.pages.resize_with(page_idx + 1, || None);
        }
        let generation = self.generation;
        let page =
            self.pages[page_idx].get_or_insert_with(|| Box::new(IdBitmapPage::new(generation)));
        page.last_used_generation = generation;
        page
    }

    /// Inserts an ID into the bitmap.
    #[inline]
    pub fn insert(&mut self, id: u32) {
        let (page_idx, bit_idx) = Self::page_and_bit(id);
        let page = self.ensure_page(page_idx);
        page.words[bit_idx / 64] |= 1 << (bit_idx % 64);
    }

    /// Returns `true` if the bitmap contains the given ID.
    #[inline]
    #[must_use]
    pub fn contains(&self, id: u32) -> bool {
        let (page_idx, bit_idx) = Self::page_and_bit(id);
        match self.pages.get(page_idx) {
            Some(Some(page)) => page.words[bit_idx / 64] & (1 << (bit_idx % 64)) != 0,
            _ => false,
        }
    }

    /// Returns the number of IDs stored in the bitmap (popcount).
    #[must_use]
    pub fn len(&self) -> u64 {
        self.pages
            .iter()
            .filter_map(|p| p.as_ref())
            .flat_map(|p| p.words.iter())
            .map(|w| w.count_ones() as u64)
            .sum()
    }

    /// Returns `true` if the bitmap contains no IDs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pages
            .iter()
            .filter_map(|p| p.as_ref())
            .all(|p| p.words.iter().all(|&w| w == 0))
    }

    /// Clears the bitmap and repopulates it from the given iterator.
    ///
    /// This reuses existing page allocations when possible. New pages are allocated as needed,
    /// and stale pages are evicted per the generation-based eviction policy.
    pub fn populate(&mut self, iter: impl Iterator<Item = u32>) {
        self.clear();
        for id in iter {
            self.insert(id);
        }
    }

    /// In-place union: `self |= other`.
    pub fn union_with(&mut self, other: &Self) {
        if other.pages.len() > self.pages.len() {
            self.pages.resize_with(other.pages.len(), || None);
        }
        for (i, other_page) in other.pages.iter().enumerate() {
            if let Some(op) = other_page {
                let sp = self.ensure_page(i);
                for (sw, ow) in sp.words.iter_mut().zip(op.words.iter()) {
                    *sw |= ow;
                }
            }
        }
    }

    /// In-place intersection: `self &= other`.
    pub fn intersect_with(&mut self, other: &Self) {
        for (i, self_page) in self.pages.iter_mut().enumerate() {
            if let Some(sp) = self_page {
                match other.pages.get(i) {
                    Some(Some(op)) => {
                        sp.last_used_generation = self.generation;
                        for (sw, ow) in sp.words.iter_mut().zip(op.words.iter()) {
                            *sw &= ow;
                        }
                    }
                    _ => {
                        // No corresponding page in other — zero this page
                        sp.last_used_generation = self.generation;
                        sp.words.fill(0);
                    }
                }
            }
        }
    }

    /// In-place difference: `self &= !other`.
    pub fn difference_with(&mut self, other: &Self) {
        for (i, self_page) in self.pages.iter_mut().enumerate() {
            if let Some(sp) = self_page {
                if let Some(Some(op)) = other.pages.get(i) {
                    sp.last_used_generation = self.generation;
                    for (sw, ow) in sp.words.iter_mut().zip(op.words.iter()) {
                        *sw &= !ow;
                    }
                }
            }
        }
    }
}

impl Default for IdBitmap {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for IdBitmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let allocated_pages = self.pages.iter().filter(|p| p.is_some()).count();
        f.debug_struct("IdBitmap")
            .field("total_pages", &self.pages.len())
            .field("allocated_pages", &allocated_pages)
            .field("generation", &self.generation)
            .field("len", &self.len())
            .finish()
    }
}

impl PartialEq for IdBitmap {
    fn eq(&self, other: &Self) -> bool {
        // Compare the logical content only (not generation counters): for each page index,
        // both bitmaps must have the same bits set. Missing or None pages are treated as
        // all-zeros.
        let max_pages = self.pages.len().max(other.pages.len());
        for i in 0..max_pages {
            let self_words = self
                .pages
                .get(i)
                .and_then(|p| p.as_ref())
                .map(|p| &p.words[..]);
            let other_words = other
                .pages
                .get(i)
                .and_then(|p| p.as_ref())
                .map(|p| &p.words[..]);
            match (self_words, other_words) {
                (Some(sw), Some(ow)) => {
                    if sw != ow {
                        return false;
                    }
                }
                (Some(w), None) | (None, Some(w)) => {
                    if w.iter().any(|&v| v != 0) {
                        return false;
                    }
                }
                (None, None) => {}
            }
        }
        true
    }
}

/// A pool of reusable [`IdBitmap`] instances to avoid repeated heap allocations.
///
/// When acquiring a bitmap, the pool returns a previously released bitmap (already cleared) or
/// creates a new one if the pool is empty. When releasing a bitmap, it is returned to the pool
/// for future reuse. The bitmaps retain their page allocations across reuse cycles.
pub struct IdBitmapPool {
    bitmaps: Vec<IdBitmap>,
}

impl IdBitmapPool {
    /// Creates a new empty pool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bitmaps: Vec::new(),
        }
    }

    /// Acquires an [`IdBitmap`] from the pool, or creates a new empty one if the pool is empty.
    ///
    /// The returned bitmap is cleared and ready for use.
    pub fn acquire(&mut self) -> IdBitmap {
        match self.bitmaps.pop() {
            Some(mut bm) => {
                bm.clear();
                bm
            }
            None => IdBitmap::new(),
        }
    }

    /// Returns an [`IdBitmap`] to the pool for future reuse.
    pub fn release(&mut self, bitmap: IdBitmap) {
        self.bitmaps.push(bitmap);
    }
}

impl Default for IdBitmapPool {
    fn default() -> Self {
        Self::new()
    }
}

/// MatchType describes how we should match the String values provided
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// match on the string values exactly how they are defined
    Strict,
    /// apply string values as a regexp
    Regexp,
}

const fn default_match_type() -> MatchType {
    MatchType::Strict
}

/// enum that allows a field to have any type
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum AnyValue {
    /// string type
    String(String),
    /// int type
    Int(i64),
    /// double type
    Double(f64),
    /// boolean type
    Boolean(bool),
    /// array of any type
    Array(Vec<AnyValue>),
    /// keyvalue type
    KeyValue(Vec<KeyValue>),
}

/// struct that represents attributes and other key value pairs
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct KeyValue {
    /// Attribute key.
    pub key: String,
    /// Attribute value.
    pub value: AnyValue,
}

impl KeyValue {
    /// create a new key value pair
    #[must_use]
    pub const fn new(key: String, value: AnyValue) -> Self {
        Self { key, value }
    }
}

/// Finds all nulls and converts them to false values
/// null values affect the filter computation as when
/// we perform the not operation nothing happens to
/// the null values
#[must_use]
fn nulls_to_false(a: &BooleanArray) -> BooleanArray {
    // is_not_null doesn't error see https://docs.rs/arrow/latest/arrow/compute/fn.is_not_null.html
    let valid = arrow::compute::is_not_null(a).expect("is_not_null doesn't error"); // BooleanArray with no nulls
    // the result of boolean array will be a boolean array of equal length so we can guarantee that these two columns have the same length
    arrow::compute::and_kleene(a, &valid).expect("can combine two columns with equal length") // nulls become false; trues stay true
}

enum IdSet {
    U16(RoaringBitmap),
    U32(RoaringBitmap),
}

/// regex_match_column() takes a string column and a regex expression. The function
/// determines what type the string column is either string array or a dictionary
/// array and then applys a regex expression onto it, returns the corresponding boolean
/// array.
/// Returns an error if string column is not a utf8, dictionary(uint8, utf8), or dictionary(uint16, utf8)
fn regex_match_column(src: &ArrayRef, regex: &str) -> Result<BooleanArray> {
    match src.data_type() {
        DataType::Utf8 => {
            let string_array = src
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("array can be downcast to StringArray");

            Ok(
                arrow::compute::regexp_is_match_scalar(string_array, regex, None)
                    .expect("can apply match string column with regexp scalar"),
            )
        }

        DataType::Dictionary(key, val) => {
            match (key.as_ref(), val.as_ref()) {
                (&DataType::UInt8, &DataType::Utf8) => {
                    let dict_arr = src
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt8Type>>()
                        .expect("can cast to dictionary array uint8type");

                    // get string from values
                    // safety: we've checked the type
                    let string_values = dict_arr
                        .values()
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .expect("can cast to string type");
                    // regex check against the values
                    let val_filt =
                        arrow::compute::regexp_is_match_scalar(string_values, regex, None)
                            .expect("can compare string value column to string regex scalar");
                    // now we need to map to the keys
                    let mut key_filt = BooleanBuilder::with_capacity(dict_arr.len());
                    for key in dict_arr.keys() {
                        if let Some(k) = key {
                            key_filt.append_value(val_filt.value(k as usize));
                        } else {
                            key_filt.append_value(false);
                        }
                    }
                    Ok(key_filt.finish())
                }
                (&DataType::UInt16, &DataType::Utf8) => {
                    let dict_arr = src
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .expect("can cast to dictionary array uint16type");

                    // get string from values
                    // safety: we've checked the type
                    let string_values = dict_arr
                        .values()
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .expect("can cast to string type");
                    // since we use a scalar here we don't have to worry a column length mismatch when we compare string values to regexp
                    let val_filt =
                        arrow::compute::regexp_is_match_scalar(string_values, regex, None)
                            .expect("can compare string value column to string regex scalar");
                    // now we need to map to the keys
                    let mut key_filt = BooleanBuilder::with_capacity(dict_arr.len());
                    for key in dict_arr.keys() {
                        if let Some(k) = key {
                            key_filt.append_value(val_filt.value(k as usize));
                        } else {
                            key_filt.append_value(false);
                        }
                    }
                    Ok(key_filt.finish())
                }
                _ => {
                    // return error not correct column type
                    Err(Error::UnsupportedStringDictKeyType {
                        data_type: *key.clone(),
                    })
                }
            }
        }
        _ => {
            // return error not correct column type
            Err(Error::UnsupportedStringColumnType {
                data_type: src.data_type().clone(),
            })
        }
    }
}

/// build_uint16_id_filter() takes a id_set which contains ids of u16 we want to remove and the id_column that
/// the set of id's should map to. The function then iterates through the ids and builds a filter
/// that matches those ids
/// This will return an error if the column is not DataType::UInt16
pub fn build_uint16_id_filter(
    id_column: &Arc<dyn Array>,
    id_set: &RoaringBitmap,
) -> Result<BooleanArray> {
    if (id_column.len() >= ID_COLUMN_LENGTH_MIN_THRESHOLD)
        && ((id_set.len() as f64 / id_column.len() as f64) <= IDS_PERCENTAGE_MAX_THRESHOLD)
        && (id_set.len() as usize) <= ID_SET_MAX_LENGTH_THRESHOLD
    {
        let mut combined_id_filter = BooleanArray::new_null(id_column.len());
        // build id filter using the id hashset
        for id in id_set {
            let id_scalar = UInt16Array::new_scalar(id as u16);
            // since we use a scalar here we don't have to worry a column length mismatch when we compare
            let id_filter =
                arrow::compute::kernels::cmp::eq(id_column, &id_scalar).map_err(|_| {
                    Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        actual: id_column.data_type().clone(),
                        expect: DataType::UInt16,
                    }
                })?;
            combined_id_filter = arrow::compute::or_kleene(&combined_id_filter, &id_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        }

        Ok(combined_id_filter)
    } else {
        // convert id to something we can iterate through
        // iterate through and check if id is in the id_set if so then we append true to boolean builder if not then false
        let uint16_id_array = id_column
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                name: consts::ID.into(),
                actual: id_column.data_type().clone(),
                expect: DataType::UInt16,
            })?;

        let mut id_filter = BooleanBuilder::new();

        // we'll build up the filter by appending to it in contiguous segments of selected IDs
        // as this is usually faster way to append to BooleanBuilder
        let mut segment_validity = false;
        let mut segment_len = 0usize;

        for uint16_id in uint16_id_array {
            let row_validity = match uint16_id {
                Some(uint16) => id_set.contains(uint16 as u32),
                None => false,
            };

            if segment_validity != row_validity {
                if segment_len > 0 {
                    id_filter.append_n(segment_len, segment_validity);
                }
                segment_validity = row_validity;
                segment_len = 0;
            }

            segment_len += 1;
        }

        // append the final segment
        if segment_len > 0 {
            id_filter.append_n(segment_len, segment_validity);
        }

        Ok(id_filter.finish())
    }
}

/// build_uint32_id_filter() takes a id_set of u32 which contains ids we want to remove and the id_column that
/// the set of id's should map to. The function then iterates through the ids and builds a filter
/// that matches those ids and inverts it so the returned BooleanArray when applied will remove rows
/// that contain those ids
/// This will return an error if the column is not DataType::UInt32, DataType::Dictionary(UInt8, UInt32)
/// or DataType::Dictionary(UInt16, UInt32)
pub fn build_uint32_id_filter(
    id_column: &Arc<dyn Array>,
    id_set: &RoaringBitmap,
) -> Result<BooleanArray> {
    if (id_column.len() >= ID_COLUMN_LENGTH_MIN_THRESHOLD)
        && ((id_set.len() as f64 / id_column.len() as f64) <= IDS_PERCENTAGE_MAX_THRESHOLD)
    {
        let mut combined_id_filter = BooleanArray::new_null(id_column.len());
        // build id filter using the id hashset
        for id in id_set {
            let id_scalar = UInt32Array::new_scalar(id);
            // since we use a scalar here we don't have to worry a column length mismatch when we compare
            let id_filter =
                arrow::compute::kernels::cmp::eq(id_column, &id_scalar).map_err(|_| {
                    Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        actual: id_column.data_type().clone(),
                        expect: DataType::UInt32,
                    }
                })?;
            combined_id_filter = arrow::compute::or_kleene(&combined_id_filter, &id_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        }

        Ok(combined_id_filter)
    } else {
        // convert id to something we can iterate through
        // iterate through and check if id is in the id_set if so then we append true to boolean builder if not then false
        match id_column.data_type() {
            DataType::UInt32 => {
                // convert id to something we can iterate through
                // iterate through and check if id is in the id_set if so then we append true to boolean builder if not then false
                let uint32_id_array = id_column
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        actual: id_column.data_type().clone(),
                        expect: DataType::UInt32,
                    })?;

                let mut id_filter = BooleanBuilder::with_capacity(uint32_id_array.len());
                for uint32_id in uint32_id_array {
                    match uint32_id {
                        Some(uint32) => {
                            id_filter.append_value(id_set.contains(uint32));
                        }
                        None => {
                            id_filter.append_value(false);
                        }
                    }
                }

                Ok(id_filter.finish())
            }
            DataType::Dictionary(key, val) => match (key.as_ref(), val.as_ref()) {
                (&DataType::UInt8, &DataType::UInt32) => {
                    let uint32_id_dict_array = id_column
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt8Type>>()
                        .ok_or_else(|| Error::ColumnDataTypeMismatch {
                            name: consts::ID.into(),
                            actual: id_column.data_type().clone(),
                            expect: DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::UInt32),
                            ),
                        })?;

                    let uint32_id_array = uint32_id_dict_array
                        .values()
                        .as_any()
                        .downcast_ref::<UInt32Array>()
                        .ok_or_else(|| Error::ColumnDataTypeMismatch {
                            name: consts::ID.into(),
                            actual: uint32_id_dict_array.data_type().clone(),
                            expect: DataType::UInt32,
                        })?;

                    let mut id_filter = BooleanBuilder::with_capacity(uint32_id_dict_array.len());
                    for key in uint32_id_dict_array.keys() {
                        if let Some(k) = key {
                            id_filter
                                .append_value(id_set.contains(uint32_id_array.value(k as usize)));
                        } else {
                            id_filter.append_value(false);
                        }
                    }

                    Ok(id_filter.finish())
                }
                (&DataType::UInt16, &DataType::UInt32) => {
                    let uint32_id_dict_array = id_column
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .ok_or_else(|| Error::ColumnDataTypeMismatch {
                            name: consts::ID.into(),
                            actual: id_column.data_type().clone(),
                            expect: DataType::Dictionary(
                                Box::new(DataType::UInt16),
                                Box::new(DataType::UInt32),
                            ),
                        })?;

                    let uint32_id_array = uint32_id_dict_array
                        .values()
                        .as_any()
                        .downcast_ref::<UInt32Array>()
                        .ok_or_else(|| Error::ColumnDataTypeMismatch {
                            name: consts::ID.into(),
                            actual: uint32_id_dict_array.data_type().clone(),
                            expect: DataType::UInt32,
                        })?;

                    let mut id_filter = BooleanBuilder::with_capacity(uint32_id_dict_array.len());
                    for key in uint32_id_dict_array.keys() {
                        if let Some(k) = key {
                            id_filter
                                .append_value(id_set.contains(uint32_id_array.value(k as usize)));
                        } else {
                            id_filter.append_value(false);
                        }
                    }

                    Ok(id_filter.finish())
                }
                _ => Err(Error::InvalidListArray {
                    expect_oneof: vec![
                        DataType::UInt32,
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                        DataType::Dictionary(
                            Box::new(DataType::UInt16),
                            Box::new(DataType::UInt32),
                        ),
                    ],
                    actual: (id_column.data_type().clone()),
                }),
            },
            _ => Err(Error::InvalidListArray {
                expect_oneof: vec![
                    DataType::UInt32,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
                ],
                actual: (id_column.data_type().clone()),
            }),
        }
    }
}

/// build_id_filter() takes a set of ids either u16 or u32 and maps it it's corresponding method
/// and then returns a boolean array that masks a column based on the set of ids provided
fn build_id_filter(id_column: &Arc<dyn Array>, id_set: IdSet) -> Result<BooleanArray> {
    match id_set {
        IdSet::U16(u16_ids) => build_uint16_id_filter(id_column, &u16_ids),
        IdSet::U32(u32_ids) => build_uint32_id_filter(id_column, &u32_ids),
    }
}

/// Builds a selection [`BooleanArray`] for a native (non-dictionary) [`PrimitiveArray`] by checking
/// each value against the [`IdBitmap`].
#[must_use]
pub fn build_native_selection_vec<T: ArrowPrimitiveType>(
    array: &PrimitiveArray<T>,
    id_bitmap: &IdBitmap,
) -> BooleanArray
where
    T::Native: Into<u32>,
{
    let mut builder = BooleanBuilder::with_capacity(array.len());
    let mut seg_val = false;
    let mut seg_len = 0usize;

    for val in array {
        let valid = match val {
            Some(v) => id_bitmap.contains(v.into()),
            None => false,
        };

        if valid != seg_val {
            if seg_len > 0 {
                builder.append_n(seg_len, seg_val);
            }
            seg_val = valid;
            seg_len = 0;
        }

        seg_len += 1;
    }

    if seg_len > 0 {
        builder.append_n(seg_len, seg_val);
    }

    builder.finish()
}

/// Builds a selection [`BooleanArray`] for a dictionary-encoded u32 ID column by checking each
/// resolved value against the [`IdBitmap`].
///
/// Note - only supports Dictionary ID types used for in OTAP (`Dictionary(UInt8, UInt32)`
/// and `Dictionary(UInt16, UInt32)`, which are used as parent_id in attributes record batches.
/// This returns an error if the passed id_column is not one of these types.
pub fn build_dict_u32_selection_vec(
    id_column: &ArrayRef,
    id_bitmap: &IdBitmap,
) -> Result<BooleanArray> {
    match id_column.data_type() {
        DataType::Dictionary(key, val) => match (key.as_ref(), val.as_ref()) {
            (&DataType::UInt8, &DataType::UInt32) => {
                let dict_array = id_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        actual: id_column.data_type().clone(),
                        expect: DataType::Dictionary(
                            Box::new(DataType::UInt8),
                            Box::new(DataType::UInt32),
                        ),
                    })?;

                let values = dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        actual: dict_array.data_type().clone(),
                        expect: DataType::UInt32,
                    })?;

                let mut builder = BooleanBuilder::with_capacity(dict_array.len());
                for key in dict_array.keys() {
                    match key {
                        Some(k) => {
                            builder.append_value(id_bitmap.contains(values.value(k as usize)));
                        }
                        None => {
                            builder.append_value(false);
                        }
                    }
                }

                Ok(builder.finish())
            }
            (&DataType::UInt16, &DataType::UInt32) => {
                let dict_array = id_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        actual: id_column.data_type().clone(),
                        expect: DataType::Dictionary(
                            Box::new(DataType::UInt16),
                            Box::new(DataType::UInt32),
                        ),
                    })?;

                let values = dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        actual: dict_array.data_type().clone(),
                        expect: DataType::UInt32,
                    })?;

                let mut builder = BooleanBuilder::with_capacity(dict_array.len());
                for key in dict_array.keys() {
                    match key {
                        Some(k) => {
                            builder.append_value(id_bitmap.contains(values.value(k as usize)));
                        }
                        None => {
                            builder.append_value(false);
                        }
                    }
                }

                Ok(builder.finish())
            }
            _ => Err(Error::InvalidListArray {
                expect_oneof: vec![
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
                ],
                actual: id_column.data_type().clone(),
            }),
        },
        _ => Err(Error::InvalidListArray {
            expect_oneof: vec![
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
            ],
            actual: id_column.data_type().clone(),
        }),
    }
}

/// get_ids() takes the id_column from a record batch and the corresponding filter
/// and applies it to extract all ids that match and then returns the set of ids.
/// This will return an error if the column is not DataType::UInt16, DataType::UInt32,
/// DataType::Dictionary(UInt8, UInt32), or DataType::Dictionary(UInt16, UInt32)
fn get_ids(id_column: &Arc<dyn Array>, filter: &BooleanArray) -> Result<IdSet> {
    // get ids being removed
    // error out herre
    let filtered_ids = arrow::compute::filter(id_column, filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;

    match filtered_ids.data_type() {
        DataType::UInt16 => {
            let filtered_ids = filtered_ids
                .as_any()
                .downcast_ref::<UInt16Array>()
                .expect("Data type is uint16 so we can safely downcast");
            Ok(IdSet::U16(
                filtered_ids.iter().flatten().map(|i| i as u32).collect(),
            ))
        }
        DataType::UInt32 => {
            let filtered_ids = filtered_ids
                .as_any()
                .downcast_ref::<UInt32Array>()
                .expect("Data type is uint32 so we can safely downcast");
            Ok(IdSet::U32(filtered_ids.iter().flatten().collect()))
        }
        DataType::Dictionary(key, val) => match (key.as_ref(), val.as_ref()) {
            (&DataType::UInt8, &DataType::UInt32) => {
                let filtered_ids_dictionary = filtered_ids
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect(
                        "Data type is dictionary with key type uint8, so we can safely downcast",
                    );

                let filtered_ids_value = filtered_ids_dictionary
                    .values()
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .expect("Data type of values is Uint32, so we can safely downcast");

                Ok(IdSet::U32(
                    filtered_ids_dictionary
                        .keys()
                        .into_iter()
                        .flatten()
                        .map(|k| filtered_ids_value.value(k as usize))
                        .collect(),
                ))
            }
            (&DataType::UInt16, &DataType::UInt32) => {
                let filtered_ids_dictionary = filtered_ids
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect(
                        "Data type is dictionary with key type uint8, so we can safely downcast",
                    );

                let filtered_ids_value = filtered_ids_dictionary
                    .values()
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .expect("Data type of values is Uint32, so we can safely downcast");

                Ok(IdSet::U32(
                    filtered_ids_dictionary
                        .keys()
                        .into_iter()
                        .flatten()
                        .map(|k| filtered_ids_value.value(k as usize))
                        .collect(),
                ))
            }
            _ => Err(Error::InvalidListArray {
                expect_oneof: vec![
                    DataType::UInt16,
                    DataType::UInt32,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
                ],
                actual: (filtered_ids.data_type().clone()),
            }),
        },
        _ => Err(Error::InvalidListArray {
            expect_oneof: vec![
                DataType::UInt16,
                DataType::UInt32,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
            ],
            actual: (filtered_ids.data_type().clone()),
        }),
    }
}

/// apply_filter() takes a payload, payload_type, and filter and uses the payload type
/// to extract the record_batch then applys the filter and updates the payload with the
/// new record batch.
/// This function will error out if the record batch doesn't exist or the filter column length
/// doesn't match the record batch column length
fn apply_filter(
    payload: &mut OtapArrowRecords,
    payload_type: ArrowPayloadType,
    filter: &BooleanArray,
) -> Result<(u64, u64)> {
    let record_batch = payload
        .get(payload_type)
        .ok_or_else(|| Error::RecordBatchNotFound { payload_type })?;
    let num_rows_before = record_batch.num_rows() as u64;
    let filtered_record_batch = arrow::compute::filter_record_batch(record_batch, filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
    let num_rows_removed = num_rows_before - (filtered_record_batch.num_rows() as u64);
    payload.set(payload_type, filtered_record_batch);
    Ok((num_rows_before, num_rows_removed))
}

/// update_child_record_batch_filter() takes an child record batch, with it's respective filter
/// id column from the parent record batch, and the parent record batch filter.
/// This function extracts the masked id from the parent record batch and uses these ids to update
/// the optional record batch
/// This function will return an error if the id column is not DataType::UInt16 or the filter
/// column length doesn't match the record batch column length
fn update_child_record_batch_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    child_filter: &BooleanArray,
    parent_filter: &BooleanArray,
) -> Result<BooleanArray> {
    let parent_id_column = get_required_array(record_batch, consts::PARENT_ID)?;
    let ids_filtered = get_ids(id_column, parent_filter)?;
    let parent_id_filter = build_id_filter(parent_id_column, ids_filtered)?;

    arrow::compute::and_kleene(child_filter, &parent_id_filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })
}

/// update_parent_record_batch_filter() takes an child record batch, with it's respective filter
/// id column from the parent record batch, and the parent record batch filter.
/// This function extracts the masked id from the optional record batch and uses these ids to update
/// the parent record batch
/// This function will return an error if the id column is not DataType::UInt16 or the filter
/// column length doesn't match the record batch column length
fn update_parent_record_batch_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    child_filter: &BooleanArray,
    parent_filter: &BooleanArray,
) -> Result<BooleanArray> {
    // starting with the resource_attr
    // -> get ids of filtered attributes
    // -> map ids to resource_ids in span
    // -> create filter to require these resource_ids
    // -> update span filter
    let parent_ids_column = get_required_array(record_batch, consts::PARENT_ID)?;

    // let ids_filter = match parent_ids_column.data_type() {
    //     DataType::UInt16 => {
    let parent_ids_filtered = get_ids(parent_ids_column, child_filter)?;

    // create filter to remove these ids from span
    let ids_filter = build_id_filter(id_column, parent_ids_filtered)?;

    arrow::compute::and_kleene(parent_filter, &ids_filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })
}

/// new_child_record_batch_filter() takes an child record batch,
/// id column from the parent record batch, and the parent record batch filter.
/// This function extracts the masked id from the parent record batch and uses these ids to
/// create a filter for the provided child record batch
/// This function will return an error if the id column is not DataType::UInt16, DataType::UInt32,
/// DataType::Dictionary(UInt8, UInt32) or the filter column length doesn't match the record batch column length
fn new_child_record_batch_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    parent_filter: &BooleanArray,
) -> Result<BooleanArray> {
    let parent_id_column = get_required_array(record_batch, consts::PARENT_ID)?;
    let ids_filtered = get_ids(id_column, parent_filter)?;
    build_id_filter(parent_id_column, ids_filtered)
}

/// new_parent_record_batch_filter() takes an child record batch,
/// id column from the parent record batch, and the parent record batch filter.
/// This function extracts the masked id from the parent record batch and uses these ids to
/// create a filter for the provided child record batch
/// This function will return an error if the id column is not DataType::UInt16, DataType::UInt32,
/// DataType::Dictionary(UInt8, UInt32) or the filter column length doesn't match the record batch column length
fn new_parent_record_batch_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    child_filter: &BooleanArray,
) -> Result<BooleanArray> {
    let parent_ids_column = get_required_array(record_batch, consts::PARENT_ID)?;

    let parent_ids_filtered = get_ids(parent_ids_column, child_filter)?;

    // create filter to remove these ids from span
    build_id_filter(id_column, parent_ids_filtered)
}

/// get_resource_attr_filter() takes a payload a vec of keyvalue pairs defining attributes to filter on, and the match type
/// and creates a booleanarray that will filter a resource_attribute record batch based on the
/// defined resource attributes we want to match
fn get_resource_attr_filter(
    payload: &OtapArrowRecords,
    attributes: &Vec<KeyValue>,
    match_type: &MatchType,
) -> Result<BooleanArray> {
    // get resource_attrs record batch
    let resource_attrs = match payload.get(ArrowPayloadType::ResourceAttrs) {
        Some(record_batch) => {
            if attributes.is_empty() {
                return Ok(BooleanArray::from(BooleanBuffer::new_set(
                    record_batch.num_rows(),
                )));
            }
            record_batch
        }
        None => {
            // if there is no record batch then
            // if we didn't plan to match any resource attributes -> allow all values through
            if attributes.is_empty() {
                return Ok(BooleanArray::from(BooleanBuffer::new_set(
                    NO_RECORD_BATCH_FILTER_SIZE,
                )));
            } else {
                // if we did match on resource attributes then there are no attributes to match
                return Ok(BooleanArray::from(BooleanBuffer::new_unset(
                    NO_RECORD_BATCH_FILTER_SIZE,
                )));
            }
        }
    };

    let num_rows = resource_attrs.num_rows();
    let mut attributes_filter = BooleanArray::new_null(num_rows);
    let key_column = get_required_array(resource_attrs, consts::ATTRIBUTE_KEY)?;

    // generate the filter for this record_batch
    for attribute in attributes {
        // match on key
        let key_scalar = StringArray::new_scalar(attribute.key.clone());
        // since we use a scalar here we don't have to worry a column length mismatch when we compare
        let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
            .expect("can compare string key column to string scalar");
        // and match on value
        let value_filter = match &attribute.value {
            AnyValue::String(value) => {
                // get string column
                let string_column = get_required_array(resource_attrs, consts::ATTRIBUTE_STR)?;
                match match_type {
                    MatchType::Regexp => regex_match_column(string_column, value)?,
                    MatchType::Strict => {
                        let value_scalar = StringArray::new_scalar(value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare
                        arrow::compute::kernels::cmp::eq(&string_column, &value_scalar)
                            .expect("can compare string value column to string scalar")
                    }
                }
            }
            AnyValue::Int(value) => {
                let int_column = resource_attrs.column_by_name(consts::ATTRIBUTE_INT);

                // check if column exists if not then there is no resource that has this attribute so we can return a all false boolean array
                match int_column {
                    Some(column) => {
                        let value_scalar = Int64Array::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare

                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare i64 value column to i64 scalar")
                    }
                    None => {
                        return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                    }
                }
            }
            AnyValue::Double(value) => {
                let double_column = resource_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
                match double_column {
                    Some(column) => {
                        let value_scalar = Float64Array::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare

                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare f64 value column to f64 scalar")
                    }
                    None => {
                        return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                    }
                }
            }
            AnyValue::Boolean(value) => {
                let bool_column = resource_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
                match bool_column {
                    Some(column) => {
                        let value_scalar = BooleanArray::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare

                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare bool value column to bool scalar")
                    }
                    None => {
                        return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                    }
                }
            }
            _ => {
                // ToDo add keyvalue, array, and bytes
                return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
            }
        };
        // build filter that checks for both matching key and value filter
        let attribute_filter = arrow::compute::and_kleene(&key_filter, &value_filter)
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        // combine with overrall filter
        attributes_filter = arrow::compute::or_kleene(&attributes_filter, &attribute_filter)
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
    }

    // using the attribute filter we need to get the ids of the rows that match and use that to build our final filter
    // this is to make sure we don't drop attributes that belong to a resource that matched the resource_attributes that
    // were defined

    // we get the id column and apply filter to get the ids we should keep
    let parent_id_column = get_required_array(resource_attrs, consts::PARENT_ID)?;
    // the ids should show up self.resource_attr.len() times otherwise they don't have all the required attributes
    let ids = arrow::compute::filter(&parent_id_column, &attributes_filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
    // extract correct ids
    let ids = ids
        .as_any()
        .downcast_ref::<UInt16Array>()
        .expect("array can be downcast to UInt16Array");
    // remove null values
    let mut ids_counted: HashMap<u16, usize> = HashMap::with_capacity(ids.len());
    // since we require that all the resource attributes match we use the count of the ids extracted to determine a full match
    // a full match should meant that the amount of times a id appears is equal the number of resource attributes we want to match on
    for id in ids.iter().flatten() {
        *ids_counted.entry(id).or_default() += 1;
    }

    let required_ids_count = attributes.len();
    // filter out ids that don't fully match
    ids_counted.retain(|_key, value| *value >= required_ids_count);

    // return filter built with the ids
    build_id_filter(
        parent_id_column,
        IdSet::U16(ids_counted.into_keys().map(|i| i as u32).collect()),
    )
}

/// get_attr_filter() takes a payload a vec of keyvalue pairs defining attributes to filter on, the match type
/// and the specific attribute payload type to filter on and creates a booleanarray that will filter for that
/// record batch based on the provided attributes
fn get_attr_filter(
    payload: &OtapArrowRecords,
    attributes: &Vec<KeyValue>,
    match_type: &MatchType,
    attribute_payload_type: ArrowPayloadType,
) -> Result<BooleanArray> {
    // get record batch containing attributes
    let log_attrs = match payload.get(attribute_payload_type) {
        Some(record_batch) => {
            if attributes.is_empty() {
                return Ok(BooleanArray::from(BooleanBuffer::new_set(
                    record_batch.num_rows(),
                )));
            }
            record_batch
        }
        None => {
            // if there is no record batch then
            // if we didn't plan to match any record attributes -> allow all values through
            if attributes.is_empty() {
                return Ok(BooleanArray::from(BooleanBuffer::new_set(
                    NO_RECORD_BATCH_FILTER_SIZE,
                )));
            } else {
                // if we did match on record attributes then there are no attributes to match
                return Ok(BooleanArray::from(BooleanBuffer::new_unset(
                    NO_RECORD_BATCH_FILTER_SIZE,
                )));
            }
        }
    };

    let num_rows = log_attrs.num_rows();
    // if there is nothing to filter we return all true
    let mut attributes_filter = BooleanArray::from(BooleanBuffer::new_unset(num_rows));

    let key_column = get_required_array(log_attrs, consts::ATTRIBUTE_KEY)?;

    // generate the filter for this record_batch
    for attribute in attributes {
        // match on key
        let key_scalar = StringArray::new_scalar(attribute.key.clone());
        // since we use a scalar here we don't have to worry a column length mismatch when we compare

        let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
            .expect("can compare string key column to string scalar");
        // match on value
        let value_filter = match &attribute.value {
            AnyValue::String(value) => {
                // get string column
                let string_column = get_required_array(log_attrs, consts::ATTRIBUTE_STR)?;

                match match_type {
                    MatchType::Regexp => regex_match_column(string_column, value)?,
                    MatchType::Strict => {
                        let value_scalar = StringArray::new_scalar(value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare
                        arrow::compute::kernels::cmp::eq(&string_column, &value_scalar)
                            .expect("can compare string value column to string scalar")
                    }
                }
            }
            AnyValue::Int(value) => {
                let int_column = log_attrs.column_by_name(consts::ATTRIBUTE_INT);
                match int_column {
                    Some(column) => {
                        let value_scalar = Int64Array::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare
                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare i64 value column to i64 scalar")
                    }
                    None => {
                        continue;
                    }
                }
            }
            AnyValue::Double(value) => {
                let double_column = log_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
                match double_column {
                    Some(column) => {
                        let value_scalar = Float64Array::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare
                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare f64 value column to f64 scalar")
                    }
                    None => {
                        continue;
                    }
                }
            }
            AnyValue::Boolean(value) => {
                let bool_column = log_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
                match bool_column {
                    Some(column) => {
                        let value_scalar = BooleanArray::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare
                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare bool value column to bool scalar")
                    }
                    None => {
                        continue;
                    }
                }
            }
            _ => {
                // ToDo add keyvalue, array, and bytes
                continue;
            }
        };
        // build filter that checks for both matching key and value filter
        let attribute_filter = arrow::compute::and_kleene(&key_filter, &value_filter)
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        // combine with rest of filters
        attributes_filter = arrow::compute::or_kleene(&attributes_filter, &attribute_filter)
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
    }

    // now we get ids of filtered attributes to make sure we don't drop any attributes that belong to the log record
    let parent_id_column = get_required_array(log_attrs, consts::PARENT_ID)?;

    let ids = get_ids(parent_id_column, &attributes_filter)?;
    // build filter around the ids and return the filter
    build_id_filter(parent_id_column, ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, BooleanArray, UInt16Array};
    use roaring::RoaringBitmap;
    use std::sync::Arc;

    #[test]
    fn test_build_uint16_id_filter_basic() {
        // Test with basic filtering
        let id_column: Arc<dyn Array> = Arc::new(UInt16Array::from(vec![1, 2, 3, 4, 5]));
        let mut id_set = RoaringBitmap::new();
        _ = id_set.insert(2);
        _ = id_set.insert(4);

        let result = build_uint16_id_filter(&id_column, &id_set).unwrap();
        let expected = BooleanArray::from(vec![false, true, false, true, false]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_uint16_id_filter_with_nulls() {
        // Test with null values in the column
        let id_column: Arc<dyn Array> = Arc::new(UInt16Array::from(vec![
            Some(1),
            None,
            Some(3),
            None,
            Some(5),
        ]));
        let mut id_set = RoaringBitmap::new();
        _ = id_set.insert(1);
        _ = id_set.insert(3);
        _ = id_set.insert(5);

        let result = build_uint16_id_filter(&id_column, &id_set).unwrap();
        let expected = BooleanArray::from(vec![true, false, true, false, true]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_uint16_id_filter_all_nulls() {
        // Test with all null values
        let id_column: Arc<dyn Array> = Arc::new(UInt16Array::from(vec![None, None, None]));
        let mut id_set = RoaringBitmap::new();
        _ = id_set.insert(1);
        _ = id_set.insert(2);
        _ = id_set.insert(3);

        let result = build_uint16_id_filter(&id_column, &id_set).unwrap();
        let expected = BooleanArray::from(vec![false, false, false]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_uint16_id_filter_contiguous_runs() {
        // Test with contiguous runs of the same value
        let id_column: Arc<dyn Array> =
            Arc::new(UInt16Array::from(vec![1, 1, 1, 2, 2, 3, 3, 3, 3, 4]));
        let mut id_set = RoaringBitmap::new();
        _ = id_set.insert(1);
        _ = id_set.insert(3);

        let result = build_uint16_id_filter(&id_column, &id_set).unwrap();
        let expected = BooleanArray::from(vec![
            true, true, true, false, false, true, true, true, true, false,
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_uint16_id_filter_contiguous_with_nulls() {
        // Test with contiguous runs including nulls
        let id_column: Arc<dyn Array> = Arc::new(UInt16Array::from(vec![
            Some(5),
            Some(5),
            None,
            None,
            Some(5),
            Some(10),
            Some(10),
        ]));
        let mut id_set = RoaringBitmap::new();
        _ = id_set.insert(5);

        let result = build_uint16_id_filter(&id_column, &id_set).unwrap();
        let expected = BooleanArray::from(vec![true, true, false, false, true, false, false]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_uint16_id_filter_empty_set() {
        // Test with empty id_set (should filter out everything)
        let id_column: Arc<dyn Array> = Arc::new(UInt16Array::from(vec![1, 2, 3, 4]));
        let id_set = RoaringBitmap::new();

        let result = build_uint16_id_filter(&id_column, &id_set).unwrap();
        let expected = BooleanArray::from(vec![false, false, false, false]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_uint16_id_filter_no_matches() {
        // Test where no IDs in column match the set
        let id_column: Arc<dyn Array> = Arc::new(UInt16Array::from(vec![1, 2, 3]));
        let mut id_set = RoaringBitmap::new();
        _ = id_set.insert(10);
        _ = id_set.insert(20);
        _ = id_set.insert(30);

        let result = build_uint16_id_filter(&id_column, &id_set).unwrap();
        let expected = BooleanArray::from(vec![false, false, false]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_uint16_id_filter_all_match() {
        // Test where all IDs match
        let id_column: Arc<dyn Array> = Arc::new(UInt16Array::from(vec![7, 8, 9]));
        let mut id_set = RoaringBitmap::new();
        _ = id_set.insert(7);
        _ = id_set.insert(8);
        _ = id_set.insert(9);

        let result = build_uint16_id_filter(&id_column, &id_set).unwrap();
        let expected = BooleanArray::from(vec![true, true, true]);

        assert_eq!(result, expected);
    }

    // --- IdBitmap tests ---

    #[test]
    fn test_id_bitmap_empty() {
        let bitmap = IdBitmap::new();
        assert!(bitmap.is_empty());
        assert_eq!(bitmap.len(), 0);
        assert!(!bitmap.contains(0));
        assert!(!bitmap.contains(100));
    }

    #[test]
    fn test_id_bitmap_insert_and_contains() {
        let mut bitmap = IdBitmap::new();
        bitmap.insert(0);
        bitmap.insert(5);
        bitmap.insert(63);
        bitmap.insert(64);
        bitmap.insert(1000);

        assert!(bitmap.contains(0));
        assert!(bitmap.contains(5));
        assert!(bitmap.contains(63));
        assert!(bitmap.contains(64));
        assert!(bitmap.contains(1000));

        assert!(!bitmap.contains(1));
        assert!(!bitmap.contains(62));
        assert!(!bitmap.contains(65));
        assert!(!bitmap.contains(999));

        assert_eq!(bitmap.len(), 5);
        assert!(!bitmap.is_empty());
    }

    #[test]
    fn test_id_bitmap_clear_and_reuse() {
        let mut bitmap = IdBitmap::new();
        bitmap.insert(10);
        bitmap.insert(20);
        assert_eq!(bitmap.len(), 2);

        bitmap.clear();
        assert!(bitmap.is_empty());
        assert!(!bitmap.contains(10));
        assert!(!bitmap.contains(20));

        // reuse after clear - allocation should still be there
        bitmap.insert(30);
        assert!(bitmap.contains(30));
        assert!(!bitmap.contains(10));
        assert_eq!(bitmap.len(), 1);
    }

    #[test]
    fn test_id_bitmap_populate() {
        let mut bitmap = IdBitmap::new();
        bitmap.populate([1u32, 3, 5, 7].into_iter());

        assert!(bitmap.contains(1));
        assert!(!bitmap.contains(2));
        assert!(bitmap.contains(3));
        assert!(!bitmap.contains(4));
        assert!(bitmap.contains(5));
        assert!(bitmap.contains(7));
        assert_eq!(bitmap.len(), 4);

        // repopulate with different values
        bitmap.populate([2u32, 4].into_iter());
        assert!(!bitmap.contains(1));
        assert!(bitmap.contains(2));
        assert!(!bitmap.contains(3));
        assert!(bitmap.contains(4));
        assert!(!bitmap.contains(5));
        assert_eq!(bitmap.len(), 2);
    }

    #[test]
    fn test_id_bitmap_contains_out_of_range() {
        let mut bitmap = IdBitmap::new();
        bitmap.insert(5);
        // ID well beyond the allocated range should return false, not panic
        assert!(!bitmap.contains(1_000_000));
    }

    #[test]
    fn test_id_bitmap_word_boundary() {
        // Test IDs right at word boundaries (multiples of 64)
        let mut bitmap = IdBitmap::new();
        for i in (0..256).step_by(64) {
            bitmap.insert(i);
        }
        for i in (0..256).step_by(64) {
            assert!(bitmap.contains(i), "expected {} to be present", i);
        }
        for i in (1..256).step_by(64) {
            assert!(!bitmap.contains(i), "expected {} to be absent", i);
        }
    }

    // --- build_native_selection_vec tests ---

    #[test]
    fn test_build_native_selection_vec_u16_basic() {
        let array = UInt16Array::from(vec![1, 2, 3, 4, 5]);
        let mut bitmap = IdBitmap::new();
        bitmap.populate([2u32, 4].into_iter());

        let result = build_native_selection_vec(&array, &bitmap);
        let expected = BooleanArray::from(vec![false, true, false, true, false]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_native_selection_vec_u16_with_nulls() {
        let array = UInt16Array::from(vec![Some(1), None, Some(3), None, Some(5)]);
        let mut bitmap = IdBitmap::new();
        bitmap.populate([1u32, 3, 5].into_iter());

        let result = build_native_selection_vec(&array, &bitmap);
        let expected = BooleanArray::from(vec![true, false, true, false, true]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_native_selection_vec_u32_basic() {
        let array = UInt32Array::from(vec![10, 20, 30, 40, 50]);
        let mut bitmap = IdBitmap::new();
        bitmap.populate([20u32, 30, 50].into_iter());

        let result = build_native_selection_vec(&array, &bitmap);
        let expected = BooleanArray::from(vec![false, true, true, false, true]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_native_selection_vec_empty_bitmap() {
        let array = UInt16Array::from(vec![1, 2, 3]);
        let bitmap = IdBitmap::new();

        let result = build_native_selection_vec(&array, &bitmap);
        let expected = BooleanArray::from(vec![false, false, false]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_native_selection_vec_all_match() {
        let array = UInt16Array::from(vec![7, 8, 9]);
        let mut bitmap = IdBitmap::new();
        bitmap.populate([7u32, 8, 9].into_iter());

        let result = build_native_selection_vec(&array, &bitmap);
        let expected = BooleanArray::from(vec![true, true, true]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_native_selection_vec_no_match() {
        let array = UInt16Array::from(vec![1, 2, 3]);
        let mut bitmap = IdBitmap::new();
        bitmap.populate([10u32, 20, 30].into_iter());

        let result = build_native_selection_vec(&array, &bitmap);
        let expected = BooleanArray::from(vec![false, false, false]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_native_selection_vec_contiguous_runs() {
        // Tests the segment batching optimization with contiguous selected/unselected runs
        let array = UInt16Array::from(vec![1, 1, 1, 2, 2, 2, 1, 1]);
        let mut bitmap = IdBitmap::new();
        bitmap.insert(1);

        let result = build_native_selection_vec(&array, &bitmap);
        let expected = BooleanArray::from(vec![true, true, true, false, false, false, true, true]);
        assert_eq!(result, expected);
    }

    // --- IdBitmap set operations tests ---

    fn bitmap_from(ids: &[u32]) -> IdBitmap {
        let mut bm = IdBitmap::new();
        for &id in ids {
            bm.insert(id);
        }
        bm
    }

    #[test]
    fn test_id_bitmap_union_with() {
        let mut a = bitmap_from(&[1, 2, 3]);
        let b = bitmap_from(&[3, 4, 5]);
        a.union_with(&b);

        for id in [1, 2, 3, 4, 5] {
            assert!(a.contains(id), "expected {} to be present", id);
        }
        assert!(!a.contains(0));
        assert!(!a.contains(6));
    }

    #[test]
    fn test_id_bitmap_union_with_extends() {
        // other is longer than self — self should grow
        let mut a = bitmap_from(&[1]);
        let b = bitmap_from(&[1000]);
        a.union_with(&b);

        assert!(a.contains(1));
        assert!(a.contains(1000));
    }

    #[test]
    fn test_id_bitmap_intersect_with() {
        let mut a = bitmap_from(&[1, 2, 3, 4]);
        let b = bitmap_from(&[2, 4, 6]);
        a.intersect_with(&b);

        assert!(!a.contains(1));
        assert!(a.contains(2));
        assert!(!a.contains(3));
        assert!(a.contains(4));
        assert!(!a.contains(6));
    }

    #[test]
    fn test_id_bitmap_intersect_with_shorter_other() {
        // Words beyond other's length should be zeroed
        let mut a = bitmap_from(&[1, 1000]);
        let b = bitmap_from(&[1]);
        a.intersect_with(&b);

        assert!(a.contains(1));
        assert!(!a.contains(1000));
    }

    #[test]
    fn test_id_bitmap_difference_with() {
        let mut a = bitmap_from(&[1, 2, 3, 4]);
        let b = bitmap_from(&[2, 3, 5]);
        a.difference_with(&b);

        assert!(a.contains(1));
        assert!(!a.contains(2));
        assert!(!a.contains(3));
        assert!(a.contains(4));
        assert!(!a.contains(5));
    }

    #[test]
    fn test_id_bitmap_difference_with_shorter_other() {
        // Words beyond other's length should be unchanged
        let mut a = bitmap_from(&[1, 1000]);
        let b = bitmap_from(&[1]);
        a.difference_with(&b);

        assert!(!a.contains(1));
        assert!(a.contains(1000));
    }

    // --- IdBitmapPool tests ---

    #[test]
    fn test_id_bitmap_pool_acquire_release() {
        let mut pool = IdBitmapPool::new();

        // first acquire should create a new bitmap
        let mut bm = pool.acquire();
        assert!(bm.is_empty());

        // populate it
        bm.insert(42);
        assert!(bm.contains(42));

        // release it back
        pool.release(bm);

        // acquire again should return the same allocation, but cleared
        let bm2 = pool.acquire();
        assert!(bm2.is_empty());
        assert!(!bm2.contains(42));
        // the page allocation should still be present from the previous use
        assert!(!bm2.pages.is_empty());
        assert!(bm2.pages[0].is_some());
    }

    #[test]
    fn test_id_bitmap_pool_multiple() {
        let mut pool = IdBitmapPool::new();

        let bm1 = pool.acquire();
        let bm2 = pool.acquire();
        pool.release(bm1);
        pool.release(bm2);

        // should get two back
        let _ = pool.acquire();
        let _ = pool.acquire();
        // third should create a new one
        let bm3 = pool.acquire();
        assert!(bm3.is_empty());
    }

    // --- Paged bitmap tests ---

    #[test]
    fn test_id_bitmap_sparse_pages() {
        // Adversarial case: IDs in distant pages. Should only allocate 2 pages (16 KiB),
        // not the full 512 MiB a flat bitmap would require.
        let mut bitmap = IdBitmap::new();
        bitmap.insert(0);
        bitmap.insert(1);
        bitmap.insert(u32::MAX - 10);

        assert!(bitmap.contains(0));
        assert!(bitmap.contains(1));
        assert!(bitmap.contains(u32::MAX - 10));
        assert!(!bitmap.contains(2));
        assert!(!bitmap.contains(u32::MAX));
        assert_eq!(bitmap.len(), 3);

        // Only 2 pages should be allocated (page 0 and page 65535)
        let allocated_pages = bitmap.pages.iter().filter(|p| p.is_some()).count();
        assert_eq!(allocated_pages, 2);
    }

    #[test]
    fn test_id_bitmap_paged_clear_preserves_pages() {
        let mut bitmap = IdBitmap::new();
        bitmap.insert(0);
        bitmap.insert(100_000); // page 1

        assert_eq!(bitmap.pages.iter().filter(|p| p.is_some()).count(), 2);

        bitmap.clear();
        assert!(bitmap.is_empty());
        // Pages should still be allocated
        assert_eq!(bitmap.pages.iter().filter(|p| p.is_some()).count(), 2);

        // Reuse should work
        bitmap.insert(0);
        assert!(bitmap.contains(0));
        assert!(!bitmap.contains(100_000));
    }

    #[test]
    fn test_id_bitmap_paged_set_operations_different_pages() {
        // union across different pages
        let mut a = bitmap_from(&[0, 1]);
        let b = bitmap_from(&[100_000, 100_001]); // page 1
        a.union_with(&b);

        assert!(a.contains(0));
        assert!(a.contains(1));
        assert!(a.contains(100_000));
        assert!(a.contains(100_001));

        // intersect: only page 0 has IDs in both
        let mut c = bitmap_from(&[0, 1, 100_000]);
        let d = bitmap_from(&[0, 1]);
        c.intersect_with(&d);

        assert!(c.contains(0));
        assert!(c.contains(1));
        assert!(!c.contains(100_000)); // zeroed because d has no page 1

        // difference across pages
        let mut e = bitmap_from(&[0, 1, 100_000]);
        let f = bitmap_from(&[1, 100_000]);
        e.difference_with(&f);

        assert!(e.contains(0));
        assert!(!e.contains(1));
        assert!(!e.contains(100_000));
    }

    #[test]
    fn test_id_bitmap_paged_equality() {
        // Two bitmaps with same logical content but different page structures should be equal
        let a = bitmap_from(&[1, 2, 3]);
        let b = bitmap_from(&[1, 2, 3]);
        assert_eq!(a, b);

        // Empty bitmap with allocated pages should equal a fresh empty bitmap
        let mut c = IdBitmap::new();
        c.insert(42);
        c.clear();
        let d = IdBitmap::new();
        assert_eq!(c, d);
    }

    #[test]
    fn test_id_bitmap_page_eviction_after_threshold() {
        let mut bitmap = IdBitmap::new();

        // Use page 0 and page 1
        bitmap.insert(0);
        bitmap.insert(100_000); // page 1
        assert_eq!(bitmap.pages.iter().filter(|p| p.is_some()).count(), 2);

        // Clear repeatedly, only using page 0 each time. Page 1 should survive for
        // PAGE_EVICTION_THRESHOLD cycles, then be evicted.
        for i in 0..ID_BITMAP_PAGE_EVICTION_THRESHOLD {
            bitmap.clear();
            bitmap.insert(0); // keep page 0 alive
            assert!(
                bitmap.pages.get(1).is_some_and(|p| p.is_some()),
                "page 1 should still be allocated at cycle {i}"
            );
        }

        // One more clear should evict page 1 (it's now threshold+1 cycles old)
        bitmap.clear();
        bitmap.insert(0);
        // Page 1 should be evicted (either None or the slot is gone)
        let page_1_exists = bitmap.pages.get(1).is_some_and(|p| p.is_some());
        assert!(
            !page_1_exists,
            "page 1 should have been evicted after {} idle cycles",
            ID_BITMAP_PAGE_EVICTION_THRESHOLD + 1
        );

        // Page 0 should still be present
        assert!(bitmap.contains(0));
    }

    #[test]
    fn test_id_bitmap_page_eviction_does_not_evict_recently_used() {
        let mut bitmap = IdBitmap::new();

        // Alternate between page 0 and page 1 every other cycle
        for _ in 0..50 {
            bitmap.clear();
            bitmap.insert(0); // page 0

            bitmap.clear();
            bitmap.insert(100_000); // page 1
        }

        // Both pages should still be allocated because neither goes more than
        // 1 cycle unused, well within the eviction threshold
        assert!(bitmap.pages[0].is_some());
        assert!(bitmap.pages[1].is_some());
    }

    #[test]
    fn test_id_bitmap_trailing_none_trimmed() {
        let mut bitmap = IdBitmap::new();

        // Allocate a page at a high index
        bitmap.insert(200_000); // page 3
        assert!(bitmap.pages.len() >= 4);

        // Clear enough times to evict the page
        for _ in 0..=ID_BITMAP_PAGE_EVICTION_THRESHOLD + 1 {
            bitmap.clear();
        }

        // The pages vec should have been trimmed — no trailing None slots
        assert!(
            bitmap.pages.is_empty() || bitmap.pages.last().unwrap().is_some(),
            "trailing None slots should be trimmed"
        );
    }
}
