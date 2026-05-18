// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Low-level STEF wire framing and column-tree helpers.
//!
//! Higher-level encode/decode modules work with logical metric entities. This module owns
//! the byte-oriented pieces: STEF stream headers, frames, modified-field bit streams,
//! column size tables, and the static metrics column layout.

mod decode;
mod encode;

pub(super) use self::decode::*;
pub(super) use self::encode::*;

pub(super) const HDR_SIGNATURE: &[u8; 4] = b"STEF";
pub(super) const HDR_FORMAT_VERSION: u8 = 0;
pub(super) const COMPRESSION_NONE: u8 = 0;
pub(super) const FRAME_FLAG_NONE: u8 = 0;

pub(super) const METRIC_FIELD_MASK: u64 = 0b11111111;
pub(super) const RESOURCE_FIELD_MASK: u64 = 0b111;
pub(super) const SCOPE_FIELD_MASK: u64 = 0b11111;
pub(super) const ROOT_METRIC_FIELD: u64 = 1 << 1;
pub(super) const ROOT_RESOURCE_FIELD: u64 = 1 << 2;
pub(super) const ROOT_SCOPE_FIELD: u64 = 1 << 3;
pub(super) const ROOT_ATTRIBUTES_FIELD: u64 = 1 << 4;
pub(super) const ROOT_POINT_FIELD: u64 = 1 << 5;

use super::{Error, METRICS_WIRE_SCHEMA};

#[derive(Default)]
pub(super) struct Column {
    pub(in crate::stef) data: Vec<u8>,
    pub(in crate::stef) children: Vec<Column>,
}

impl Column {
    pub(super) fn with_children(children: Vec<Column>) -> Self {
        Self {
            data: Vec::new(),
            children,
        }
    }
}

#[derive(Default)]
pub(super) struct DecodeColumn<'a> {
    pub(in crate::stef) data: &'a [u8],
    byte_len: usize,
    pub(in crate::stef) children: Vec<DecodeColumn<'a>>,
}

impl<'a> DecodeColumn<'a> {
    pub(super) fn with_children(children: Vec<DecodeColumn<'a>>) -> Self {
        Self {
            data: &[],
            byte_len: 0,
            children,
        }
    }
}

pub(super) fn metrics_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        metric_column_tree(),
        resource_column_tree(),
        scope_column_tree(),
        attributes_column_tree(),
        point_column_tree(),
    ])
}

pub(super) fn metric_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
        attributes_column_tree(),
        array_column_tree(Column::default()),
        Column::default(),
        Column::default(),
    ])
}

pub(super) fn resource_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        attributes_column_tree(),
        Column::default(),
    ])
}

pub(super) fn scope_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        Column::default(),
        Column::default(),
        attributes_column_tree(),
        Column::default(),
    ])
}

pub(super) fn attributes_column_tree() -> Column {
    Column::with_children(vec![Column::default(), any_value_column_tree()])
}

pub(super) fn any_value_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
    ])
}

pub(super) fn point_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        Column::default(),
        point_value_column_tree(),
        array_column_tree(Column::default()),
    ])
}

pub(super) fn point_value_column_tree() -> Column {
    Column::with_children(vec![
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
        Column::default(),
    ])
}

pub(super) fn array_column_tree(element: Column) -> Column {
    Column::with_children(vec![element])
}

pub(super) fn decode_metrics_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        decode_metric_column_tree(),
        decode_resource_column_tree(),
        decode_scope_column_tree(),
        decode_attributes_column_tree(),
        decode_point_column_tree(),
    ])
}

pub(super) fn decode_metric_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        decode_attributes_column_tree(),
        decode_array_column_tree(DecodeColumn::default()),
        DecodeColumn::default(),
        DecodeColumn::default(),
    ])
}

pub(super) fn decode_resource_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        decode_attributes_column_tree(),
        DecodeColumn::default(),
    ])
}

pub(super) fn decode_scope_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        decode_attributes_column_tree(),
        DecodeColumn::default(),
    ])
}

pub(super) fn decode_attributes_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        decode_any_value_column_tree(),
    ])
}

pub(super) fn decode_any_value_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
    ])
}

pub(super) fn decode_point_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        DecodeColumn::default(),
        decode_point_value_column_tree(),
        decode_array_column_tree(DecodeColumn::default()),
    ])
}

pub(super) fn decode_point_value_column_tree<'a>() -> DecodeColumn<'a> {
    DecodeColumn::with_children(vec![
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
        DecodeColumn::default(),
    ])
}

pub(super) fn decode_array_column_tree(element: DecodeColumn<'_>) -> DecodeColumn<'_> {
    DecodeColumn::with_children(vec![element])
}

pub(super) fn write_columns(columns: &Column, dst: &mut Vec<u8>) -> Result<(), Error> {
    let mut sizes = BitWriter::default();
    write_column_sizes(columns, &mut sizes)?;
    let size_bytes = sizes.take_bytes();
    append_uvarint(size_bytes.len() as u64, dst);
    dst.extend_from_slice(&size_bytes);
    write_column_data(columns, dst);
    Ok(())
}

pub(super) fn write_column_sizes(column: &Column, sizes: &mut BitWriter) -> Result<(), Error> {
    let byte_len =
        u64::try_from(column.data.len()).map_err(|_| Error::ValueOutOfRange("column"))?;
    if byte_len >= (1_u64 << 48) {
        return Err(Error::ValueOutOfRange("column"));
    }
    sizes.write_uvarint_compact(byte_len);
    if column.data.is_empty() {
        return Ok(());
    }
    for child in &column.children {
        write_column_sizes(child, sizes)?;
    }
    Ok(())
}

pub(super) fn write_column_data(column: &Column, dst: &mut Vec<u8>) {
    dst.extend_from_slice(&column.data);
    if column.data.is_empty() {
        return;
    }
    for child in &column.children {
        write_column_data(child, dst);
    }
}

pub(super) fn write_fixed_header(dst: &mut Vec<u8>) {
    dst.extend_from_slice(HDR_SIGNATURE);
    append_uvarint(2, dst);
    dst.push(HDR_FORMAT_VERSION);
    dst.push(COMPRESSION_NONE);
}

pub(super) fn write_var_header_frame(dst: &mut Vec<u8>) {
    let mut var_header = Vec::with_capacity(METRICS_WIRE_SCHEMA.len() + 4);
    append_uvarint(METRICS_WIRE_SCHEMA.len() as u64, &mut var_header);
    var_header.extend_from_slice(METRICS_WIRE_SCHEMA);
    append_uvarint(0, &mut var_header);
    write_frame(dst, FRAME_FLAG_NONE, &var_header);
}

pub(super) fn write_frame(dst: &mut Vec<u8>, flags: u8, content: &[u8]) {
    dst.push(flags);
    append_uvarint(content.len() as u64, dst);
    dst.extend_from_slice(content);
}

pub(super) fn append_uvarint(mut value: u64, dst: &mut Vec<u8>) {
    while value >= 0x80 {
        dst.push((value as u8) | 0x80);
        value >>= 7;
    }
    dst.push(value as u8);
}
