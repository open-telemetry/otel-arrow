// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Columnar OTAP (`OtapArrowRecords`) inspection, CBOR path navigation, and masked 4-stage
//! `LogTransformPolicy` execution (`1. Rename -> 2. Add -> 3. Remove -> 4. Redact`).

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use ahash::AHashMap;
use arrow::array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray,
    Float64Array, Int32Array, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array,
    UInt16Array, UInt32Array,
};
use arrow::buffer::{BooleanBuffer, NullBuffer};
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmapPool, filter_otap_batch};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;
use roaring::RoaringBitmap;

use crate::model::{
    AttributePath, LogAdd, LogFieldSelector, LogRecordField, LogRemove, LogRename,
    LogTargetSelector, LogTransformPolicy, PolicyError, ScalarValue, ScopeField,
    apply_regex_redaction, eq_i64_f64,
};

/// Decoded scalar or array value extracted from an OTAP column or CBOR `ATTRIBUTE_SER` payload.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractedValue<'a> {
    /// UTF-8 string value.
    Str(Cow<'a, str>),
    /// 64-bit signed integer value.
    Int(i64),
    /// 64-bit float value.
    Double(f64),
    /// Boolean value.
    Bool(bool),
    /// Raw byte slice.
    Bytes(Cow<'a, [u8]>),
    /// Array of scalar values (from CBOR `AttributeValueType::Slice`).
    Array(Vec<ExtractedValue<'static>>),
    /// Map of key-value pairs (from CBOR `AttributeValueType::Map`).
    Map(Vec<(String, ExtractedValue<'static>)>),
}

impl<'a> ExtractedValue<'a> {
    /// Converts borrowed data into `'static`.
    #[must_use]
    pub fn into_owned(self) -> ExtractedValue<'static> {
        match self {
            Self::Str(s) => ExtractedValue::Str(Cow::Owned(s.into_owned())),
            Self::Int(i) => ExtractedValue::Int(i),
            Self::Double(d) => ExtractedValue::Double(d),
            Self::Bool(b) => ExtractedValue::Bool(b),
            Self::Bytes(b) => ExtractedValue::Bytes(Cow::Owned(b.into_owned())),
            Self::Array(v) => ExtractedValue::Array(v),
            Self::Map(m) => ExtractedValue::Map(m),
        }
    }

    /// Checks exact equality against a policy `ScalarValue` (`policy.md:253-264`).
    #[must_use]
    pub fn equals_scalar(&self, target: &ScalarValue) -> bool {
        match (self, target) {
            (Self::Str(a), ScalarValue::String(b)) => a.as_ref() == b.as_str(),
            (Self::Int(a), ScalarValue::Int(b)) => *a == *b,
            (Self::Double(a), ScalarValue::Double(b)) => !a.is_nan() && !b.is_nan() && *a == *b,
            (Self::Int(a), ScalarValue::Double(b)) => eq_i64_f64(*a, *b),
            (Self::Double(a), ScalarValue::Int(b)) => eq_i64_f64(*b, *a),
            (Self::Bool(a), ScalarValue::Bool(b)) => *a == *b,
            (Self::Bytes(a), ScalarValue::Bytes(b)) => a.as_ref() == b.as_slice(),
            _ => false,
        }
    }

    /// Checks `contains` against a policy `ScalarValue` (`policy.md:279-284`).
    #[must_use]
    pub fn contains_scalar(&self, target: &ScalarValue) -> bool {
        match (self, target) {
            (Self::Str(a), ScalarValue::String(b)) => a.contains(b.as_str()),
            (Self::Bytes(a), ScalarValue::Bytes(b)) => {
                if b.is_empty() {
                    true
                } else {
                    memchr::memmem::find(a.as_ref(), b.as_slice()).is_some()
                }
            }
            (Self::Array(elems), scalar) => elems.iter().any(|e| e.equals_scalar(scalar)),
            _ => false,
        }
    }
}

/// Formats a 16-byte `trace_id` or 8-byte `span_id` into lowercase hex, returning `None` if all zeros.
#[inline]
#[must_use]
pub fn format_non_zero_hex(bytes: &[u8]) -> Option<String> {
    if bytes.iter().all(|&b| b == 0) {
        return None;
    }
    Some(hex::encode(bytes))
}

/// Decodes a CBOR byte slice (`ATTRIBUTE_SER`) into an `ExtractedValue<'static>`.
pub fn decode_cbor_value(bytes: &[u8]) -> Option<ExtractedValue<'static>> {
    let val: ciborium::Value = ciborium::from_reader(bytes).ok()?;
    cbor_to_extracted(val)
}

fn cbor_to_extracted(val: ciborium::Value) -> Option<ExtractedValue<'static>> {
    match val {
        ciborium::Value::Text(s) => Some(ExtractedValue::Str(Cow::Owned(s))),
        ciborium::Value::Integer(i) => i128::from(i).try_into().ok().map(ExtractedValue::Int),
        ciborium::Value::Float(f) => Some(ExtractedValue::Double(f)),
        ciborium::Value::Bool(b) => Some(ExtractedValue::Bool(b)),
        ciborium::Value::Bytes(b) => Some(ExtractedValue::Bytes(Cow::Owned(b))),
        ciborium::Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for item in arr {
                if let Some(ev) = cbor_to_extracted(item) {
                    out.push(ev);
                }
            }
            Some(ExtractedValue::Array(out))
        }
        ciborium::Value::Map(map) => {
            let mut out = Vec::with_capacity(map.len());
            for (k, v) in map {
                if let ciborium::Value::Text(key_str) = k
                    && let Some(ev) = cbor_to_extracted(v)
                {
                    out.push((key_str, ev));
                }
            }
            Some(ExtractedValue::Map(out))
        }
        _ => None,
    }
}

/// Navigates a nested `AttributePath` (`segments`) inside a decoded `ExtractedValue::Map`.
#[must_use]
pub fn navigate_extracted_path<'a>(
    mut current: ExtractedValue<'a>,
    segments: &[String],
) -> Option<ExtractedValue<'a>> {
    for seg in segments {
        match current {
            ExtractedValue::Map(entries) => {
                let found = entries.into_iter().find(|(k, _)| k == seg)?.1;
                current = found;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// Pre-indexed parent-to-row mapping for a `Logs` `OtapArrowRecords` batch.
pub struct LogsBatchView<'a> {
    /// Borrowed `OtapArrowRecords`.
    pub records: &'a OtapArrowRecords,
    /// Root `Logs` `RecordBatch`.
    pub root: &'a RecordBatch,
    /// Number of log rows in `root`.
    pub num_rows: usize,
    /// Maps `log.id` (`u16`) -> row index (`0..num_rows`), or `u32::MAX` if unmapped.
    pub log_id_to_row: Vec<u32>,
    /// Maps `resource.id` (`u16`) -> bitmap of log row indices (`0..num_rows`).
    pub resource_id_to_rows: AHashMap<u16, RoaringBitmap>,
    /// Maps `scope.id` (`u16`) -> bitmap of log row indices (`0..num_rows`).
    pub scope_id_to_rows: AHashMap<u16, RoaringBitmap>,
}

impl<'a> LogsBatchView<'a> {
    /// Constructs a `LogsBatchView` from an `OtapArrowRecords` batch.
    pub fn try_new(records: &'a OtapArrowRecords) -> Result<Option<Self>, PolicyError> {
        let Some(root) = records.get(ArrowPayloadType::Logs) else {
            return Ok(None);
        };
        let num_rows = root.num_rows();
        if num_rows == 0 {
            return Ok(None);
        }

        // Map log.id -> row_idx
        let mut log_id_to_row = Vec::new();
        if let Some(id_col) = root.column_by_name(consts::ID)
            && let Some(id_arr) = id_col.as_any().downcast_ref::<UInt16Array>()
        {
            let mut max_id = 0usize;
            for row in 0..num_rows {
                if id_arr.is_valid(row) {
                    max_id = max_id.max(id_arr.value(row) as usize);
                }
            }
            log_id_to_row.resize(max_id + 1, u32::MAX);
            for row in 0..num_rows {
                if id_arr.is_valid(row) {
                    log_id_to_row[id_arr.value(row) as usize] = row as u32;
                }
            }
        } else {
            // Identity mapping when ID column is absent
            log_id_to_row = (0..num_rows as u32).collect();
        }

        // Map resource.id -> RoaringBitmap of log rows
        let mut resource_id_to_rows: AHashMap<u16, RoaringBitmap> = AHashMap::new();
        if let Some(res_col) = root.column_by_name(consts::RESOURCE)
            && let Some(res_struct) = res_col.as_any().downcast_ref::<StructArray>()
            && let Some(res_id_col) = res_struct.column_by_name(consts::ID)
            && let Some(res_id_arr) = res_id_col.as_any().downcast_ref::<UInt16Array>()
        {
            for row in 0..num_rows {
                if res_id_arr.is_valid(row) {
                    resource_id_to_rows
                        .entry(res_id_arr.value(row))
                        .or_default()
                        .insert(row as u32);
                }
            }
        }

        // Map scope.id -> RoaringBitmap of log rows
        let mut scope_id_to_rows: AHashMap<u16, RoaringBitmap> = AHashMap::new();
        if let Some(scope_col) = root.column_by_name(consts::SCOPE)
            && let Some(scope_struct) = scope_col.as_any().downcast_ref::<StructArray>()
            && let Some(scope_id_col) = scope_struct.column_by_name(consts::ID)
            && let Some(scope_id_arr) = scope_id_col.as_any().downcast_ref::<UInt16Array>()
        {
            for row in 0..num_rows {
                if scope_id_arr.is_valid(row) {
                    scope_id_to_rows
                        .entry(scope_id_arr.value(row))
                        .or_default()
                        .insert(row as u32);
                }
            }
        }

        Ok(Some(Self {
            records,
            root,
            num_rows,
            log_id_to_row,
            resource_id_to_rows,
            scope_id_to_rows,
        }))
    }

    /// Extracts the value of `field` at a specific log row `row` (`0..num_rows`), if present.
    #[must_use]
    pub fn extract_field_at_row(&self, field: &LogFieldSelector, row: usize) -> Option<ExtractedValue<'a>> {
        match field {
            LogFieldSelector::RecordField(rf) => self.extract_record_field_at_row(*rf, row),
            LogFieldSelector::ScopeField(sf) => self.extract_scope_field_at_row(*sf, row),
            LogFieldSelector::BodyField(path) => {
                if path.path.is_empty() {
                    return None;
                }
                let val = self.extract_record_field_at_row(LogRecordField::Body, row)?;
                navigate_extracted_path(val, &path.path)
            }
            LogFieldSelector::LogAttribute(path) => {
                let log_id = self.row_to_log_id(row)?;
                let attrs = self.records.get(ArrowPayloadType::LogAttrs)?;
                extract_attr_from_batch(attrs, log_id, &path.path)
            }
            LogFieldSelector::ResourceAttribute(path) => {
                let res_id = self.row_to_resource_id(row)?;
                let attrs = self.records.get(ArrowPayloadType::ResourceAttrs)?;
                extract_attr_from_batch(attrs, res_id, &path.path)
            }
            LogFieldSelector::ScopeAttribute(path) => {
                let scope_id = self.row_to_scope_id(row)?;
                let attrs = self.records.get(ArrowPayloadType::ScopeAttrs)?;
                extract_attr_from_batch(attrs, scope_id, &path.path)
            }
        }
    }

    /// Returns `log.id` for `row`.
    #[inline]
    #[must_use]
    pub fn row_to_log_id(&self, row: usize) -> Option<u16> {
        if let Some(id_col) = self.root.column_by_name(consts::ID)
            && let Some(id_arr) = id_col.as_any().downcast_ref::<UInt16Array>()
        {
            if id_arr.is_valid(row) {
                Some(id_arr.value(row))
            } else {
                None
            }
        } else {
            u16::try_from(row).ok()
        }
    }

    /// Returns `resource.id` for `row`.
    #[inline]
    #[must_use]
    pub fn row_to_resource_id(&self, row: usize) -> Option<u16> {
        let res_col = self.root.column_by_name(consts::RESOURCE)?;
        let res_struct = res_col.as_any().downcast_ref::<StructArray>()?;
        let id_col = res_struct.column_by_name(consts::ID)?;
        let id_arr = id_col.as_any().downcast_ref::<UInt16Array>()?;
        id_arr.is_valid(row).then(|| id_arr.value(row))
    }

    /// Returns `scope.id` for `row`.
    #[inline]
    #[must_use]
    pub fn row_to_scope_id(&self, row: usize) -> Option<u16> {
        let scope_col = self.root.column_by_name(consts::SCOPE)?;
        let scope_struct = scope_col.as_any().downcast_ref::<StructArray>()?;
        let id_col = scope_struct.column_by_name(consts::ID)?;
        let id_arr = id_col.as_any().downcast_ref::<UInt16Array>()?;
        id_arr.is_valid(row).then(|| id_arr.value(row))
    }

    fn extract_record_field_at_row(
        &self,
        rf: LogRecordField,
        row: usize,
    ) -> Option<ExtractedValue<'a>> {
        match rf {
            LogRecordField::Unspecified => None,
            LogRecordField::Body => {
                let body_col = self.root.column_by_name(consts::BODY)?;
                if !body_col.is_valid(row) {
                    return None;
                }
                if let Some(body_struct) = body_col.as_any().downcast_ref::<StructArray>() {
                    extract_anyval_from_struct(body_struct, row)
                } else {
                    extract_str_from_array(body_col, row).map(|s| ExtractedValue::Str(Cow::Borrowed(s)))
                }
            }
            LogRecordField::SeverityText => {
                let col = self.root.column_by_name(consts::SEVERITY_TEXT)?;
                let s = extract_str_from_array(col, row)?;
                if s.is_empty() {
                    None
                } else {
                    Some(ExtractedValue::Str(Cow::Borrowed(s)))
                }
            }
            LogRecordField::EventName => {
                let col = self.root.column_by_name(consts::EVENT_NAME)?;
                let s = extract_str_from_array(col, row)?;
                if s.is_empty() {
                    None
                } else {
                    Some(ExtractedValue::Str(Cow::Borrowed(s)))
                }
            }
            LogRecordField::SeverityNumber => {
                let col = self.root.column_by_name(consts::SEVERITY_NUMBER)?;
                let v = extract_i64_from_array(col, row)?;
                if v == 0 {
                    None
                } else {
                    Some(ExtractedValue::Int(v))
                }
            }
            LogRecordField::TraceId => {
                let col = self.root.column_by_name(consts::TRACE_ID)?;
                let bytes = extract_fixed_bytes_from_array(col, row)?;
                let hex_str = format_non_zero_hex(bytes)?;
                Some(ExtractedValue::Str(Cow::Owned(hex_str)))
            }
            LogRecordField::SpanId => {
                let col = self.root.column_by_name(consts::SPAN_ID)?;
                let bytes = extract_fixed_bytes_from_array(col, row)?;
                let hex_str = format_non_zero_hex(bytes)?;
                Some(ExtractedValue::Str(Cow::Owned(hex_str)))
            }
        }
    }

    fn extract_scope_field_at_row(&self, sf: ScopeField, row: usize) -> Option<ExtractedValue<'a>> {
        match sf {
            ScopeField::Unspecified => None,
            ScopeField::SchemaUrl => {
                let col = self.root.column_by_name(consts::SCHEMA_URL)?;
                let s = extract_str_from_array(col, row)?;
                (!s.is_empty()).then(|| ExtractedValue::Str(Cow::Borrowed(s)))
            }
            ScopeField::Name => {
                let scope_col = self.root.column_by_name(consts::SCOPE)?;
                let scope_struct = scope_col.as_any().downcast_ref::<StructArray>()?;
                let col = scope_struct.column_by_name(consts::NAME)?;
                let s = extract_str_from_array(col, row)?;
                (!s.is_empty()).then(|| ExtractedValue::Str(Cow::Borrowed(s)))
            }
            ScopeField::Version => {
                let scope_col = self.root.column_by_name(consts::SCOPE)?;
                let scope_struct = scope_col.as_any().downcast_ref::<StructArray>()?;
                let col = scope_struct.column_by_name(consts::VERSION)?;
                let s = extract_str_from_array(col, row)?;
                (!s.is_empty()).then(|| ExtractedValue::Str(Cow::Borrowed(s)))
            }
        }
    }
}

/// Extracts a `&str` at `row` from a `StringArray` or `DictionaryArray<UInt8 | UInt16, StringArray>`.
#[inline]
#[must_use]
pub fn extract_str_from_array(col: &ArrayRef, row: usize) -> Option<&str> {
    if !col.is_valid(row) {
        return None;
    }
    if let Some(sa) = col.as_any().downcast_ref::<StringArray>() {
        return Some(sa.value(row));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da.values().as_any().downcast_ref::<StringArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da.values().as_any().downcast_ref::<StringArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    None
}

/// Extracts an `i64` at `row` from `Int32Array`, `Int64Array`, or `DictionaryArray<UInt8 | UInt16, Int32 | Int64>`.
#[inline]
#[must_use]
pub fn extract_i64_from_array(col: &ArrayRef, row: usize) -> Option<i64> {
    if !col.is_valid(row) {
        return None;
    }
    if let Some(a) = col.as_any().downcast_ref::<Int64Array>() {
        return Some(a.value(row));
    }
    if let Some(a) = col.as_any().downcast_ref::<Int32Array>() {
        return Some(i64::from(a.value(row)));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        let k = da.keys().value(row) as usize;
        if let Some(vals) = da.values().as_any().downcast_ref::<Int64Array>() {
            return vals.is_valid(k).then(|| vals.value(k));
        }
        if let Some(vals) = da.values().as_any().downcast_ref::<Int32Array>() {
            return vals.is_valid(k).then(|| i64::from(vals.value(k)));
        }
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
        let k = da.keys().value(row) as usize;
        if let Some(vals) = da.values().as_any().downcast_ref::<Int64Array>() {
            return vals.is_valid(k).then(|| vals.value(k));
        }
        if let Some(vals) = da.values().as_any().downcast_ref::<Int32Array>() {
            return vals.is_valid(k).then(|| i64::from(vals.value(k)));
        }
    }
    None
}

/// Extracts a `&[u8]` at `row` from a `BinaryArray` or `DictionaryArray<UInt8 | UInt16, BinaryArray>`.
#[inline]
#[must_use]
pub fn extract_bytes_from_array(col: &ArrayRef, row: usize) -> Option<&[u8]> {
    if !col.is_valid(row) {
        return None;
    }
    if let Some(ba) = col.as_any().downcast_ref::<BinaryArray>() {
        return Some(ba.value(row));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da.values().as_any().downcast_ref::<BinaryArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da.values().as_any().downcast_ref::<BinaryArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    None
}

/// Extracts a fixed-size byte slice at `row` from `FixedSizeBinaryArray` or `DictionaryArray`.
#[inline]
#[must_use]
pub fn extract_fixed_bytes_from_array(col: &ArrayRef, row: usize) -> Option<&[u8]> {
    if !col.is_valid(row) {
        return None;
    }
    if let Some(fsb) = col.as_any().downcast_ref::<FixedSizeBinaryArray>() {
        return Some(fsb.value(row));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da.values().as_any().downcast_ref::<FixedSizeBinaryArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da.values().as_any().downcast_ref::<FixedSizeBinaryArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    None
}

/// Extracts an `ExtractedValue` at `row` from a `logs.body` `StructArray`.
#[must_use]
pub fn extract_anyval_from_struct(struct_arr: &StructArray, row: usize) -> Option<ExtractedValue<'_>> {
    if !struct_arr.is_valid(row) {
        return None;
    }
    let type_col = struct_arr.column_by_name(consts::ATTRIBUTE_TYPE)?;
    let type_arr = type_col.as_any().downcast_ref::<UInt8Array>()?;
    if !type_arr.is_valid(row) {
        return None;
    }
    let vtype = AttributeValueType::try_from(type_arr.value(row)).ok()?;
    match vtype {
        AttributeValueType::Empty => None,
        AttributeValueType::Str => {
            let col = struct_arr.column_by_name(consts::ATTRIBUTE_STR)?;
            extract_str_from_array(col, row).map(|s| ExtractedValue::Str(Cow::Borrowed(s)))
        }
        AttributeValueType::Int => {
            let col = struct_arr.column_by_name(consts::ATTRIBUTE_INT)?;
            extract_i64_from_array(col, row).map(ExtractedValue::Int)
        }
        AttributeValueType::Double => {
            let col = struct_arr.column_by_name(consts::ATTRIBUTE_DOUBLE)?;
            let da = col.as_any().downcast_ref::<Float64Array>()?;
            da.is_valid(row).then(|| ExtractedValue::Double(da.value(row)))
        }
        AttributeValueType::Bool => {
            let col = struct_arr.column_by_name(consts::ATTRIBUTE_BOOL)?;
            let ba = col.as_any().downcast_ref::<BooleanArray>()?;
            ba.is_valid(row).then(|| ExtractedValue::Bool(ba.value(row)))
        }
        AttributeValueType::Bytes => {
            let col = struct_arr.column_by_name(consts::ATTRIBUTE_BYTES)?;
            extract_bytes_from_array(col, row).map(|b| ExtractedValue::Bytes(Cow::Borrowed(b)))
        }
        AttributeValueType::Map | AttributeValueType::Slice => {
            let col = struct_arr.column_by_name(consts::ATTRIBUTE_SER)?;
            let bytes = extract_bytes_from_array(col, row)?;
            decode_cbor_value(bytes)
        }
    }
}

/// Extracts an `ExtractedValue` at `row` from an attribute `RecordBatch` (`LogAttrs`, `ResourceAttrs`, `ScopeAttrs`).
#[must_use]
pub fn extract_anyval_from_attr_batch(rb: &RecordBatch, row: usize) -> Option<ExtractedValue<'_>> {
    let type_col = rb.column_by_name(consts::ATTRIBUTE_TYPE)?;
    let type_arr = type_col.as_any().downcast_ref::<UInt8Array>()?;
    if !type_arr.is_valid(row) {
        return None;
    }
    let vtype = AttributeValueType::try_from(type_arr.value(row)).ok()?;
    match vtype {
        AttributeValueType::Empty => None,
        AttributeValueType::Str => {
            let col = rb.column_by_name(consts::ATTRIBUTE_STR)?;
            extract_str_from_array(col, row).map(|s| ExtractedValue::Str(Cow::Borrowed(s)))
        }
        AttributeValueType::Int => {
            let col = rb.column_by_name(consts::ATTRIBUTE_INT)?;
            extract_i64_from_array(col, row).map(ExtractedValue::Int)
        }
        AttributeValueType::Double => {
            let col = rb.column_by_name(consts::ATTRIBUTE_DOUBLE)?;
            let da = col.as_any().downcast_ref::<Float64Array>()?;
            da.is_valid(row).then(|| ExtractedValue::Double(da.value(row)))
        }
        AttributeValueType::Bool => {
            let col = rb.column_by_name(consts::ATTRIBUTE_BOOL)?;
            let ba = col.as_any().downcast_ref::<BooleanArray>()?;
            ba.is_valid(row).then(|| ExtractedValue::Bool(ba.value(row)))
        }
        AttributeValueType::Bytes => {
            let col = rb.column_by_name(consts::ATTRIBUTE_BYTES)?;
            extract_bytes_from_array(col, row).map(|b| ExtractedValue::Bytes(Cow::Borrowed(b)))
        }
        AttributeValueType::Map | AttributeValueType::Slice => {
            let col = rb.column_by_name(consts::ATTRIBUTE_SER)?;
            let bytes = extract_bytes_from_array(col, row)?;
            decode_cbor_value(bytes)
        }
    }
}

/// Extracts `parent_id` (`u16`) at `attr_row` from an attribute `RecordBatch`.
#[inline]
#[must_use]
pub fn extract_parent_id_u16(rb: &RecordBatch, attr_row: usize) -> Option<u16> {
    let col = rb.column_by_name(consts::PARENT_ID)?;
    if !col.is_valid(attr_row) {
        return None;
    }
    if let Some(a) = col.as_any().downcast_ref::<UInt16Array>() {
        return Some(a.value(attr_row));
    }
    if let Some(a) = col.as_any().downcast_ref::<UInt32Array>() {
        return u16::try_from(a.value(attr_row)).ok();
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        let k = da.keys().value(attr_row) as usize;
        if let Some(vals) = da.values().as_any().downcast_ref::<UInt16Array>() {
            return vals.is_valid(k).then(|| vals.value(k));
        }
        if let Some(vals) = da.values().as_any().downcast_ref::<UInt32Array>() {
            return vals
                .is_valid(k)
                .then(|| u16::try_from(vals.value(k)).ok())
                .flatten();
        }
    }
    None
}

fn extract_attr_from_batch<'a>(
    rb: &'a RecordBatch,
    target_parent_id: u16,
    path: &[String],
) -> Option<ExtractedValue<'a>> {
    let first_key = path.first()?;
    let key_col = rb.column_by_name(consts::ATTRIBUTE_KEY)?;
    let num_rows = rb.num_rows();
    for r in 0..num_rows {
        if extract_parent_id_u16(rb, r) == Some(target_parent_id)
            && extract_str_from_array(key_col, r) == Some(first_key.as_str())
        {
            let val = extract_anyval_from_attr_batch(rb, r)?;
            if path.len() == 1 {
                return Some(val);
            }
            return navigate_extracted_path(val, &path[1..]);
        }
    }
    None
}

/// Applies a boolean row-keep mask (`RoaringBitmap` over `0..num_rows`) to an `OtapArrowRecords` batch.
pub fn filter_otap_logs_by_bitmap(
    batch: &OtapArrowRecords,
    keep_rows: &RoaringBitmap,
    num_rows: usize,
    pool: &mut IdBitmapPool,
) -> Result<OtapArrowRecords, PolicyError> {
    if keep_rows.len() as usize == num_rows {
        return Ok(batch.clone());
    }
    let bool_buf = BooleanBuffer::collect_bool(num_rows, |i| keep_rows.contains(i as u32));
    let selection = BooleanArray::new(bool_buf, None);
    filter_otap_batch(&selection, batch, pool).map_err(|e| PolicyError::Evaluation(e.to_string()))
}

/// Pre-compiled transform policy stage operations ready for global 4-stage execution.
#[derive(Debug, Clone)]
pub struct CompiledTransformPolicy {
    /// Policy ID (sorted ascending lexicographically before compilation).
    pub policy_id: String,
    /// Stage 1: Rename operations.
    pub rename: Vec<LogRename>,
    /// Stage 2: Add operations.
    pub add: Vec<LogAdd>,
    /// Stage 3: Remove operations.
    pub remove: Vec<LogRemove>,
    /// Stage 4: Redact operations with pre-compiled optional `regex::Regex`.
    pub redact: Vec<CompiledLogRedact>,
}

/// Pre-compiled `LogRedact` operation.
#[derive(Debug, Clone)]
pub struct CompiledLogRedact {
    /// Target selector.
    pub target: LogTargetSelector,
    /// Effective replacement string (`"[REDACTED]"` if unset).
    pub replacement: String,
    /// Compiled RE2-compatible regex if `pattern.regex` was set.
    pub regex: Option<regex::Regex>,
}

impl CompiledTransformPolicy {
    /// Compiles a `LogTransformPolicy` with its `policy_id`.
    pub fn compile(policy_id: String, tx: &LogTransformPolicy) -> Result<Self, PolicyError> {
        let mut redact = Vec::with_capacity(tx.redact.len());
        for r in &tx.redact {
            let compiled_re = match &r.regex {
                Some(pat) => Some(
                    regex::RegexBuilder::new(pat)
                        .multi_line(false)
                        .build()
                        .map_err(|e| PolicyError::Compile(format!("invalid redact regex '{pat}': {e}")))?,
                ),
                None => None,
            };
            redact.push(CompiledLogRedact {
                target: r.target.clone(),
                replacement: r.effective_replacement().to_owned(),
                regex: compiled_re,
            });
        }
        Ok(Self {
            policy_id,
            rename: tx.rename.clone(),
            add: tx.add.clone(),
            remove: tx.remove.clone(),
            redact,
        })
    }
}

/// Executes all matching `CompiledTransformPolicy` stages in official `opentelemetry-proto#865` order
/// (`1. Rename -> 2. Add -> 3. Remove -> 4. Redact`) directly on `OtapArrowRecords` without
/// splitting and re-concatenating batches via `concatenate::concatenate`.
///
/// `policy_matches` must be aligned with `transforms` (which are pre-sorted by ascending `policy_id`).
pub fn apply_spec_transforms_masked(
    batch: &mut OtapArrowRecords,
    transforms: &[CompiledTransformPolicy],
    policy_matches: &[RoaringBitmap],
) -> Result<u64, PolicyError> {
    if transforms.is_empty() || policy_matches.iter().all(RoaringBitmap::is_empty) {
        return Ok(0);
    }
    let Some(root) = batch.get(ArrowPayloadType::Logs).cloned() else {
        return Ok(0);
    };
    let num_rows = root.num_rows();
    if num_rows == 0 {
        return Ok(0);
    }

    // Extract mutable per-row state for top-level fields (`body`, `severity_text`, `severity_number`, `event_name`)
    // and per-parent attribute maps (`LogAttrs`, `ResourceAttrs`, `ScopeAttrs`) ONLY for columns/tables touched by transforms.
    let mut touches_root_body = false;
    let mut touches_sev_text = false;
    let mut touches_sev_num = false;
    let mut touches_event_name = false;
    let mut touches_log_attrs = false;
    let mut touches_res_attrs = false;
    let mut touches_scope_attrs = false;

    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() {
            continue;
        }
        let mut mark_target = |t: &LogTargetSelector| match t {
            LogTargetSelector::RecordField(LogRecordField::Body) => touches_root_body = true,
            LogTargetSelector::RecordField(LogRecordField::SeverityText) => touches_sev_text = true,
            LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => touches_sev_num = true,
            LogTargetSelector::RecordField(LogRecordField::EventName) => touches_event_name = true,
            LogTargetSelector::RecordField(_) => {}
            LogTargetSelector::LogAttribute(_) => touches_log_attrs = true,
            LogTargetSelector::ResourceAttribute(_) => touches_res_attrs = true,
            LogTargetSelector::ScopeAttribute(_) => touches_scope_attrs = true,
        };
        for op in &tx.rename {
            mark_target(&op.from);
            mark_target(&op.to);
        }
        for op in &tx.add {
            mark_target(&op.target);
        }
        for op in &tx.remove {
            mark_target(&op.target);
        }
        for op in &tx.redact {
            mark_target(&op.target);
        }
    }

    let mut body_vals: Vec<Option<ScalarValue>> = if touches_root_body {
        extract_root_body_vec(&root, num_rows)
    } else {
        Vec::new()
    };
    let mut sev_text_vals: Vec<Option<String>> = if touches_sev_text {
        extract_root_str_vec(&root, consts::SEVERITY_TEXT, num_rows)
    } else {
        Vec::new()
    };
    let mut sev_num_vals: Vec<Option<i32>> = if touches_sev_num {
        extract_root_i32_vec(&root, consts::SEVERITY_NUMBER, num_rows)
    } else {
        Vec::new()
    };
    let mut event_name_vals: Vec<Option<String>> = if touches_event_name {
        extract_root_str_vec(&root, consts::EVENT_NAME, num_rows)
    } else {
        Vec::new()
    };

    // Row-to-ID mappings
    let row_to_log_id: Vec<u16> = (0..num_rows)
        .map(|r| {
            root.column_by_name(consts::ID)
                .and_then(|c| c.as_any().downcast_ref::<UInt16Array>())
                .and_then(|a| a.is_valid(r).then(|| a.value(r)))
                .unwrap_or(r as u16)
        })
        .collect();
    let row_to_res_id: Vec<Option<u16>> = (0..num_rows)
        .map(|r| {
            root.column_by_name(consts::RESOURCE)
                .and_then(|c| c.as_any().downcast_ref::<StructArray>())
                .and_then(|s| s.column_by_name(consts::ID))
                .and_then(|c| c.as_any().downcast_ref::<UInt16Array>())
                .and_then(|a| a.is_valid(r).then(|| a.value(r)))
        })
        .collect();
    let row_to_scope_id: Vec<Option<u16>> = (0..num_rows)
        .map(|r| {
            root.column_by_name(consts::SCOPE)
                .and_then(|c| c.as_any().downcast_ref::<StructArray>())
                .and_then(|s| s.column_by_name(consts::ID))
                .and_then(|c| c.as_any().downcast_ref::<UInt16Array>())
                .and_then(|a| a.is_valid(r).then(|| a.value(r)))
        })
        .collect();

    let mut log_attrs_map = if touches_log_attrs {
        decode_attrs_table(batch.get(ArrowPayloadType::LogAttrs))
    } else {
        BTreeMap::new()
    };
    let mut res_attrs_map = if touches_res_attrs {
        decode_attrs_table(batch.get(ArrowPayloadType::ResourceAttrs))
    } else {
        BTreeMap::new()
    };
    let mut scope_attrs_map = if touches_scope_attrs {
        decode_attrs_table(batch.get(ArrowPayloadType::ScopeAttrs))
    } else {
        BTreeMap::new()
    };

    let mut total_transform_hits = 0u64;

    // Context closure for reading/writing/removing a target at `row`.
    macro_rules! get_target {
        ($row:expr, $target:expr) => {{
            let r: usize = $row;
            match $target {
                LogTargetSelector::RecordField(LogRecordField::Body) => body_vals[r].clone(),
                LogTargetSelector::RecordField(LogRecordField::SeverityText) => {
                    sev_text_vals[r].clone().map(ScalarValue::String)
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                    sev_num_vals[r].map(|v| ScalarValue::Int(i64::from(v)))
                }
                LogTargetSelector::RecordField(LogRecordField::EventName) => {
                    event_name_vals[r].clone().map(ScalarValue::String)
                }
                LogTargetSelector::RecordField(_) => None,
                LogTargetSelector::LogAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        let pid = row_to_log_id[r];
                        log_attrs_map
                            .get(&pid)
                            .and_then(|m| m.get(&path[0]).cloned())
                    } else {
                        None
                    }
                }
                LogTargetSelector::ResourceAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        row_to_res_id[r].and_then(|pid| {
                            res_attrs_map
                                .get(&pid)
                                .and_then(|m| m.get(&path[0]).cloned())
                        })
                    } else {
                        None
                    }
                }
                LogTargetSelector::ScopeAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        row_to_scope_id[r].and_then(|pid| {
                            scope_attrs_map
                                .get(&pid)
                                .and_then(|m| m.get(&path[0]).cloned())
                        })
                    } else {
                        None
                    }
                }
            }
        }};
    }

    macro_rules! remove_target {
        ($row:expr, $target:expr) => {{
            let r: usize = $row;
            match $target {
                LogTargetSelector::RecordField(LogRecordField::Body) => body_vals[r].take(),
                LogTargetSelector::RecordField(LogRecordField::SeverityText) => {
                    sev_text_vals[r].take().map(ScalarValue::String)
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                    sev_num_vals[r].take().map(|v| ScalarValue::Int(i64::from(v)))
                }
                LogTargetSelector::RecordField(LogRecordField::EventName) => {
                    event_name_vals[r].take().map(ScalarValue::String)
                }
                LogTargetSelector::RecordField(_) => None,
                LogTargetSelector::LogAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        let pid = row_to_log_id[r];
                        log_attrs_map.get_mut(&pid).and_then(|m| m.remove(&path[0]))
                    } else {
                        None
                    }
                }
                LogTargetSelector::ResourceAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        row_to_res_id[r].and_then(|pid| {
                            res_attrs_map.get_mut(&pid).and_then(|m| m.remove(&path[0]))
                        })
                    } else {
                        None
                    }
                }
                LogTargetSelector::ScopeAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        row_to_scope_id[r].and_then(|pid| {
                            scope_attrs_map
                                .get_mut(&pid)
                                .and_then(|m| m.remove(&path[0]))
                        })
                    } else {
                        None
                    }
                }
            }
        }};
    }

    macro_rules! set_target {
        ($row:expr, $target:expr, $val:expr) => {{
            let r: usize = $row;
            let v: ScalarValue = $val;
            match $target {
                LogTargetSelector::RecordField(LogRecordField::Body) => {
                    body_vals[r] = Some(v);
                    true
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityText) => {
                    if let ScalarValue::String(s) = v {
                        sev_text_vals[r] = Some(s);
                        true
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                    if let ScalarValue::Int(i) = v
                        && let Ok(i32_val) = i32::try_from(i)
                    {
                        sev_num_vals[r] = Some(i32_val);
                        true
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(LogRecordField::EventName) => {
                    if let ScalarValue::String(s) = v {
                        event_name_vals[r] = Some(s);
                        true
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(_) => false,
                LogTargetSelector::LogAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        let pid = row_to_log_id[r];
                        log_attrs_map.entry(pid).or_default().insert(path[0].clone(), v);
                        true
                    } else {
                        false
                    }
                }
                LogTargetSelector::ResourceAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        if let Some(pid) = row_to_res_id[r] {
                            res_attrs_map.entry(pid).or_default().insert(path[0].clone(), v);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                LogTargetSelector::ScopeAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        if let Some(pid) = row_to_scope_id[r] {
                            scope_attrs_map.entry(pid).or_default().insert(path[0].clone(), v);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            }
        }};
    }

    // STAGE 1: RENAME (across all matching policies in ascending Policy.id order)
    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() || tx.rename.is_empty() {
            continue;
        }
        for op in &tx.rename {
            if op.from == op.to {
                continue;
            }
            for row_u32 in mask.iter() {
                let row = row_u32 as usize;
                let Some(src_val) = get_target!(row, &op.from) else {
                    continue;
                };
                if !op.upsert && get_target!(row, &op.to).is_some() {
                    continue;
                }
                let _ = remove_target!(row, &op.from);
                if set_target!(row, &op.to, src_val) {
                    total_transform_hits += 1;
                }
            }
        }
    }

    // STAGE 2: ADD (across all matching policies in ascending Policy.id order)
    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() || tx.add.is_empty() {
            continue;
        }
        for op in &tx.add {
            for row_u32 in mask.iter() {
                let row = row_u32 as usize;
                if !op.upsert && get_target!(row, &op.target).is_some() {
                    continue;
                }
                if set_target!(row, &op.target, op.value.clone()) {
                    total_transform_hits += 1;
                }
            }
        }
    }

    // STAGE 3: REMOVE (across all matching policies in ascending Policy.id order)
    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() || tx.remove.is_empty() {
            continue;
        }
        for op in &tx.remove {
            for row_u32 in mask.iter() {
                let row = row_u32 as usize;
                if remove_target!(row, &op.target).is_some() {
                    total_transform_hits += 1;
                }
            }
        }
    }

    // STAGE 4: REDACT (across all matching policies in ascending Policy.id order)
    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() || tx.redact.is_empty() {
            continue;
        }
        for op in &tx.redact {
            for row_u32 in mask.iter() {
                let row = row_u32 as usize;
                let Some(cur_val) = get_target!(row, &op.target) else {
                    continue;
                };
                match &op.regex {
                    None => {
                        // Whole-value mask (converts value to replacement string per policy.md:513-523)
                        if set_target!(row, &op.target, ScalarValue::String(op.replacement.clone())) {
                            total_transform_hits += 1;
                        }
                    }
                    Some(re) => {
                        if let ScalarValue::String(s) = cur_val
                            && let Some(redacted) = apply_regex_redaction(re, &s, &op.replacement)
                        {
                            if set_target!(row, &op.target, ScalarValue::String(redacted)) {
                                total_transform_hits += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // Rebuild modified root columns if any root field was touched
    if touches_root_body || touches_sev_text || touches_sev_num || touches_event_name {
        let schema = root.schema();
        let mut new_fields: Vec<Arc<Field>> = Vec::with_capacity(schema.fields().len());
        let mut new_cols: Vec<ArrayRef> = Vec::with_capacity(schema.fields().len());
        for (idx, field) in schema.fields().iter().enumerate() {
            match field.name().as_str() {
                consts::BODY if touches_root_body => {
                    let arr = build_body_struct_array(&body_vals);
                    new_fields.push(Arc::new(Field::new(
                        consts::BODY,
                        arr.data_type().clone(),
                        true,
                    )));
                    new_cols.push(arr);
                }
                consts::SEVERITY_TEXT if touches_sev_text => {
                    let arr: ArrayRef = Arc::new(StringArray::from(sev_text_vals.clone()));
                    new_fields.push(Arc::new(Field::new(
                        consts::SEVERITY_TEXT,
                        DataType::Utf8,
                        true,
                    )));
                    new_cols.push(arr);
                }
                consts::SEVERITY_NUMBER if touches_sev_num => {
                    let arr: ArrayRef = Arc::new(Int32Array::from(sev_num_vals.clone()));
                    new_fields.push(Arc::new(Field::new(
                        consts::SEVERITY_NUMBER,
                        DataType::Int32,
                        true,
                    )));
                    new_cols.push(arr);
                }
                consts::EVENT_NAME if touches_event_name => {
                    let arr: ArrayRef = Arc::new(StringArray::from(event_name_vals.clone()));
                    new_fields.push(Arc::new(Field::new(consts::EVENT_NAME, DataType::Utf8, true)));
                    new_cols.push(arr);
                }
                _ => {
                    new_fields.push(field.clone());
                    new_cols.push(root.column(idx).clone());
                }
            }
        }
        let new_root = RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_cols)
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        batch
            .set(ArrowPayloadType::Logs, new_root)
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
    }

    if touches_log_attrs {
        let rb = encode_attrs_table(&log_attrs_map)?;
        batch
            .set(ArrowPayloadType::LogAttrs, rb)
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
    }
    if touches_res_attrs {
        let rb = encode_attrs_table(&res_attrs_map)?;
        batch
            .set(ArrowPayloadType::ResourceAttrs, rb)
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
    }
    if touches_scope_attrs {
        let rb = encode_attrs_table(&scope_attrs_map)?;
        batch
            .set(ArrowPayloadType::ScopeAttrs, rb)
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
    }

    Ok(total_transform_hits)
}

fn extract_root_body_vec(root: &RecordBatch, num_rows: usize) -> Vec<Option<ScalarValue>> {
    let mut out = vec![None; num_rows];
    let Some(col) = root.column_by_name(consts::BODY) else {
        return out;
    };
    if let Some(sa) = col.as_any().downcast_ref::<StructArray>() {
        for (r, slot) in out.iter_mut().enumerate().take(num_rows) {
            *slot = match extract_anyval_from_struct(sa, r) {
                Some(ExtractedValue::Str(s)) => Some(ScalarValue::String(s.into_owned())),
                Some(ExtractedValue::Int(i)) => Some(ScalarValue::Int(i)),
                Some(ExtractedValue::Double(d)) => Some(ScalarValue::Double(d)),
                Some(ExtractedValue::Bool(b)) => Some(ScalarValue::Bool(b)),
                Some(ExtractedValue::Bytes(b)) => Some(ScalarValue::Bytes(b.into_owned())),
                _ => None,
            };
        }
    } else {
        for (r, slot) in out.iter_mut().enumerate().take(num_rows) {
            if let Some(s) = extract_str_from_array(col, r) {
                *slot = Some(ScalarValue::String(s.to_owned()));
            }
        }
    }
    out
}

fn extract_root_str_vec(root: &RecordBatch, col_name: &str, num_rows: usize) -> Vec<Option<String>> {
    let mut out = vec![None; num_rows];
    let Some(col) = root.column_by_name(col_name) else {
        return out;
    };
    for (r, slot) in out.iter_mut().enumerate().take(num_rows) {
        if let Some(s) = extract_str_from_array(col, r)
            && !s.is_empty()
        {
            *slot = Some(s.to_owned());
        }
    }
    out
}

fn extract_root_i32_vec(root: &RecordBatch, col_name: &str, num_rows: usize) -> Vec<Option<i32>> {
    let mut out = vec![None; num_rows];
    let Some(col) = root.column_by_name(col_name) else {
        return out;
    };
    for (r, slot) in out.iter_mut().enumerate().take(num_rows) {
        if let Some(v) = extract_i64_from_array(col, r)
            && v != 0
            && let Ok(v32) = i32::try_from(v)
        {
            *slot = Some(v32);
        }
    }
    out
}

fn build_body_struct_array(vals: &[Option<ScalarValue>]) -> ArrayRef {
    let len = vals.len();
    let mut types = Vec::with_capacity(len);
    let mut strs: Vec<Option<&str>> = Vec::with_capacity(len);
    let mut ints: Vec<Option<i64>> = Vec::with_capacity(len);
    let mut doubles: Vec<Option<f64>> = Vec::with_capacity(len);
    let mut bools: Vec<Option<bool>> = Vec::with_capacity(len);
    let mut bytes_vec: Vec<Option<&[u8]>> = Vec::with_capacity(len);
    let mut validity = Vec::with_capacity(len);

    for v in vals {
        match v {
            None => {
                validity.push(false);
                types.push(AttributeValueType::Empty as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
            }
            Some(ScalarValue::String(s)) => {
                validity.push(true);
                types.push(AttributeValueType::Str as u8);
                strs.push(Some(s.as_str()));
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
            }
            Some(ScalarValue::Int(i)) => {
                validity.push(true);
                types.push(AttributeValueType::Int as u8);
                strs.push(None);
                ints.push(Some(*i));
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
            }
            Some(ScalarValue::Double(d)) => {
                validity.push(true);
                types.push(AttributeValueType::Double as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(Some(*d));
                bools.push(None);
                bytes_vec.push(None);
            }
            Some(ScalarValue::Bool(b)) => {
                validity.push(true);
                types.push(AttributeValueType::Bool as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(Some(*b));
                bytes_vec.push(None);
            }
            Some(ScalarValue::Bytes(b)) => {
                validity.push(true);
                types.push(AttributeValueType::Bytes as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(Some(b.as_slice()));
            }
        }
    }

    let fields = vec![
        Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
        Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
        Arc::new(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true)),
        Arc::new(Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true)),
        Arc::new(Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true)),
        Arc::new(Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true)),
    ];
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt8Array::from(types)),
        Arc::new(StringArray::from(strs)),
        Arc::new(Int64Array::from(ints)),
        Arc::new(Float64Array::from(doubles)),
        Arc::new(BooleanArray::from(bools)),
        Arc::new(BinaryArray::from_opt_vec(bytes_vec)),
    ];
    let nulls = NullBuffer::from(validity);
    Arc::new(StructArray::new(fields.into(), arrays, Some(nulls)))
}

fn decode_attrs_table(rb_opt: Option<&RecordBatch>) -> BTreeMap<u16, BTreeMap<String, ScalarValue>> {
    let mut map: BTreeMap<u16, BTreeMap<String, ScalarValue>> = BTreeMap::new();
    let Some(rb) = rb_opt else {
        return map;
    };
    let Some(key_col) = rb.column_by_name(consts::ATTRIBUTE_KEY) else {
        return map;
    };
    for r in 0..rb.num_rows() {
        let Some(pid) = extract_parent_id_u16(rb, r) else {
            continue;
        };
        let Some(k) = extract_str_from_array(key_col, r) else {
            continue;
        };
        let scalar = match extract_anyval_from_attr_batch(rb, r) {
            Some(ExtractedValue::Str(s)) => ScalarValue::String(s.into_owned()),
            Some(ExtractedValue::Int(i)) => ScalarValue::Int(i),
            Some(ExtractedValue::Double(d)) => ScalarValue::Double(d),
            Some(ExtractedValue::Bool(b)) => ScalarValue::Bool(b),
            Some(ExtractedValue::Bytes(b)) => ScalarValue::Bytes(b.into_owned()),
            _ => continue,
        };
        map.entry(pid).or_default().insert(k.to_owned(), scalar);
    }
    map
}

fn encode_attrs_table(
    map: &BTreeMap<u16, BTreeMap<String, ScalarValue>>,
) -> Result<RecordBatch, PolicyError> {
    let total_rows: usize = map.values().map(BTreeMap::len).sum();
    let mut parent_ids: Vec<u16> = Vec::with_capacity(total_rows);
    let mut keys: Vec<&str> = Vec::with_capacity(total_rows);
    let mut types: Vec<u8> = Vec::with_capacity(total_rows);
    let mut strs: Vec<Option<&str>> = Vec::with_capacity(total_rows);
    let mut ints: Vec<Option<i64>> = Vec::with_capacity(total_rows);
    let mut doubles: Vec<Option<f64>> = Vec::with_capacity(total_rows);
    let mut bools: Vec<Option<bool>> = Vec::with_capacity(total_rows);
    let mut bytes_vec: Vec<Option<&[u8]>> = Vec::with_capacity(total_rows);

    for (&pid, kvs) in map {
        for (k, v) in kvs {
            parent_ids.push(pid);
            keys.push(k.as_str());
            match v {
                ScalarValue::String(s) => {
                    types.push(AttributeValueType::Str as u8);
                    strs.push(Some(s.as_str()));
                    ints.push(None);
                    doubles.push(None);
                    bools.push(None);
                    bytes_vec.push(None);
                }
                ScalarValue::Int(i) => {
                    types.push(AttributeValueType::Int as u8);
                    strs.push(None);
                    ints.push(Some(*i));
                    doubles.push(None);
                    bools.push(None);
                    bytes_vec.push(None);
                }
                ScalarValue::Double(d) => {
                    types.push(AttributeValueType::Double as u8);
                    strs.push(None);
                    ints.push(None);
                    doubles.push(Some(*d));
                    bools.push(None);
                    bytes_vec.push(None);
                }
                ScalarValue::Bool(b) => {
                    types.push(AttributeValueType::Bool as u8);
                    strs.push(None);
                    ints.push(None);
                    doubles.push(None);
                    bools.push(Some(*b));
                    bytes_vec.push(None);
                }
                ScalarValue::Bytes(b) => {
                    types.push(AttributeValueType::Bytes as u8);
                    strs.push(None);
                    ints.push(None);
                    doubles.push(None);
                    bools.push(None);
                    bytes_vec.push(Some(b.as_slice()));
                }
            }
        }
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
        Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt16Array::from(parent_ids)),
            Arc::new(StringArray::from(keys)),
            Arc::new(UInt8Array::from(types)),
            Arc::new(StringArray::from(strs)),
            Arc::new(Int64Array::from(ints)),
            Arc::new(Float64Array::from(doubles)),
            Arc::new(BooleanArray::from(bools)),
            Arc::new(BinaryArray::from_opt_vec(bytes_vec)),
        ],
    )
    .map_err(|e| PolicyError::Evaluation(e.to_string()))
}
