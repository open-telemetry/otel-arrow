// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Code and utilities for decoding OTAP directly into proto bytes

use std::sync::{Arc, LazyLock};

use crate::schema::consts;
use arrow::array::{ArrayRef, RecordBatch, StructArray, UInt8Array, UInt32Array};
use arrow::compute::sort_to_indices;
use arrow::datatypes::DataType;
use arrow::row::{Row, RowConverter, SortField};

pub mod resource;

// TODO don't duplicate this
/// Protobuf wire types
pub mod wire_types {
    /// Varint (int32, int64, uint32, uint64, sint32, sint64, bool)
    pub const VARINT: u64 = 0;
    /// 64-bit (fixed64, sfixed64, double)
    pub const FIXED64: u64 = 1;
    /// Length-delimited (string, bytes, embedded messages)
    pub const LEN: u64 = 2;
    /// 32-bit (fixed32, sfixed32, float)
    pub const FIXED32: u64 = 5;
}

pub(crate) fn encode_field_tag(field_number: u64, wire_type: u64, result_buf: &mut Vec<u8>) {
    let key = (field_number << 3) | wire_type;
    encode_varint(key, result_buf);
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
        result_buf[len_start_pos + i] += ((len >> (i * 8)) & 0x7f) as u8;
    }
}

#[macro_export]
macro_rules! encode_len_delimited_mystery_size {
    ($field_tag: expr, $placeholder_size:expr, $encode_fn:expr, $buf:expr) => {{
        encode_field_tag($field_tag, wire_types::LEN, $buf);
        let len_start_pos = $buf.len();
        encode_len_placeholder($placeholder_size, $buf);
        $encode_fn;
        let len = $buf.len() - len_start_pos - $placeholder_size;
        patch_len_placeholder($placeholder_size, $buf, len, len_start_pos);
    }};
}

/// Encodes a u64 as protobuf varint into `buf`.
pub(crate) fn encode_varint(mut value: u64, buf: &mut Vec<u8>) {
    while value >= 0x80 {
        buf.push(((value as u8) & 0x7F) | 0x80);
        value >>= 7;
    }
    buf.push(value as u8);
}

pub(crate) fn encode_fixed64(value: u64, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&value.to_le_bytes());
}

pub(crate) struct RootColumnSorter {
    row_converter: RowConverter,

    // TODO comment about reusing the heap allocation for the vec of rows
    rows: Vec<(usize, Row<'static>)>,
}

impl RootColumnSorter {
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
        }
    }

    /// TODO comment
    // TODO revisit the return type here
    pub fn set_root_indices_to_visit_in_order(
        &mut self,
        record_batch: &RecordBatch,
        result_buf: &mut Vec<usize>,
    ) {
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
            // TODO
            todo!()
        } else {
            result_buf.extend(0..record_batch.num_rows())
        };
    }
}

// TODO comments
fn reuse_rows_vec<'a, 'b>(mut v: Vec<(usize, Row<'a>)>) -> Vec<(usize, Row<'b>)> {
    v.clear();
    v.into_iter().map(|_| unreachable!()).collect()
}
