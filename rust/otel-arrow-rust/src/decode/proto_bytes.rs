// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Code and utilities for decoding OTAP directly into proto bytes

use std::sync::{Arc, LazyLock};

use crate::schema::consts;
use arrow::array::{
    Array, ArrayRef, RecordBatch, StructArray, UInt8Array, UInt16Array, UInt32Array,
};
use arrow::compute::sort_to_indices;
use arrow::datatypes::DataType;
use arrow::row::{Row, RowConverter, SortField};

// TOOD should this all be moved under crate::otlp ?

pub mod resource;

pub(crate) fn proto_encode_field_tag(field_number: u64, wire_type: u64, result_buf: &mut Vec<u8>) {
    let key = (field_number << 3) | wire_type;
    proto_encode_varint(key, result_buf);
}

// todo comment on what is happening
pub(crate) fn encode_len_placeholder(num_bytes: usize, result_buf: &mut Vec<u8>) {
    for _ in 0..num_bytes - 1 {
        result_buf.push(0x80)
    }
    result_buf.push(0x00);
}

pub(crate) fn patch_len_placeholder(
    num_bytes: usize,
    result_buf: &mut Vec<u8>,
    len: usize,
    len_start_pos: usize,
) {
    for i in 0..num_bytes {
        result_buf[len_start_pos + i] += ((len >> (i * 7)) & 0x7f) as u8;
    }
}

/// Helper for encoding with unknown length. usage:
/// ```norun
/// encode_len_delimited_mystery_size!(
///     1, // field tag
///     5, // number of bytes to use for len
///     encode_some_nested_field(&mut result_buf), // fills in the child body
///     result_buf
/// )
/// ```
#[macro_export]
macro_rules! proto_encode_len_delimited_mystery_size {
    ($field_tag: expr, $placeholder_size:expr, $encode_fn:expr, $buf:expr) => {{
        crate::decode::proto_bytes::proto_encode_field_tag(
            $field_tag,
            crate::proto::consts::wire_types::LEN,
            $buf,
        );
        let len_start_pos = $buf.len();
        crate::decode::proto_bytes::encode_len_placeholder($placeholder_size, $buf);
        $encode_fn;
        let len = $buf.len() - len_start_pos - $placeholder_size;
        crate::decode::proto_bytes::patch_len_placeholder(
            $placeholder_size,
            $buf,
            len,
            len_start_pos,
        );
    }};
}

/// Encodes a u64 as protobuf varint into `buf`.
// TODO should this be generic over T so we don't always have to pass a u64
pub(crate) fn proto_encode_varint(mut value: u64, buf: &mut Vec<u8>) {
    while value >= 0x80 {
        buf.push(((value as u8) & 0x7F) | 0x80);
        value >>= 7;
    }
    buf.push(value as u8);
}

pub(crate) fn encode_fixed64(value: u64, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn encode_float64(value: f64, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&value.to_le_bytes());
}

pub(crate) struct IdColumnSorter {
    row_converter: RowConverter,

    // TODO comment about reusing the heap allocation for the vec of rows
    rows: Vec<(usize, Row<'static>)>,

    u16_ids: Vec<(usize, u16)>,
}

impl IdColumnSorter {
    pub fn new() -> Self {
        // safety: these datatypes are sortable
        let row_converter = RowConverter::new(vec![
            SortField::new(DataType::UInt16),
            SortField::new(DataType::UInt16),
        ])
        .expect("can create row converter");

        Self {
            row_converter,
            rows: Vec::new(),
            u16_ids: Vec::new(),
        }
    }

    /// TODO comment
    // TODO revisit the return type here
    pub fn root_indices_sorted(&mut self, record_batch: &RecordBatch, result_buf: &mut Vec<usize>) {
        const PLACEHOLDER_ARRAY: LazyLock<ArrayRef> =
            LazyLock::new(|| Arc::new(UInt8Array::new_null(0)));

        let mut sort_columns = [PLACEHOLDER_ARRAY.clone(), PLACEHOLDER_ARRAY.clone()];

        let mut sort_columns_idx = 0;

        if let Some(resource_col) = record_batch.column_by_name(consts::RESOURCE) {
            // TODO nounwrap
            let resource_col = resource_col.as_any().downcast_ref::<StructArray>().unwrap();
            if let Some(resource_ids) = resource_col.column_by_name(consts::ID) {
                sort_columns[sort_columns_idx] = resource_ids.clone();
                sort_columns_idx += 1;
            }
        }

        if let Some(scope_col) = record_batch.column_by_name(consts::SCOPE) {
            // TODO nounwrap
            let scope_col = scope_col.as_any().downcast_ref::<StructArray>().unwrap();
            if let Some(scope_ids) = scope_col.column_by_name(consts::ID) {
                sort_columns[sort_columns_idx] = scope_ids.clone();
                sort_columns_idx += 1;
            }
        }

        // TODO see if there's a way to do this without allocating
        if sort_columns_idx == 2 {
            // TODO maybe handle error here?
            // safety: shouldn't happen? I guess if we got an invalid record batch
            let rows = self
                .row_converter
                .convert_columns(&sort_columns)
                .expect("error converting columns for sorting");

            let mut sort = reuse_rows_vec(std::mem::take(&mut self.rows));
            sort.extend(rows.iter().enumerate());
            sort.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
            result_buf.extend(sort.iter().map(|(i, _)| *i));
            self.rows = reuse_rows_vec(sort);
        } else if sort_columns_idx == 1 {
            let ids = sort_columns[0]
                .as_any()
                .downcast_ref::<UInt16Array>()
                .unwrap();
            self.u16_ids_sorted(ids, result_buf);
        } else {
            result_buf.extend(0..record_batch.num_rows());
        }
    }

    pub fn u16_ids_sorted(&mut self, ids: &UInt16Array, result_buf: &mut Vec<usize>) {
        self.u16_ids.clear();
        self.u16_ids
            .extend(ids.values().iter().copied().enumerate());

        if ids.null_count() == 0 {
            self.u16_ids.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
        } else {
            // sort while considering nulls
            todo!()
        }

        result_buf.extend(self.u16_ids.iter().map(|(i, _)| *i));
    }
}

// TODO comments
fn reuse_rows_vec<'a, 'b>(mut v: Vec<(usize, Row<'a>)>) -> Vec<(usize, Row<'b>)> {
    v.clear();
    v.into_iter().map(|_| unreachable!()).collect()
}
