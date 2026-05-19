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
pub(super) const COMPRESSION_ZSTD: u8 = 1;
pub(super) const FRAME_FLAG_NONE: u8 = 0;
pub(super) const FRAME_FLAG_RESTART_COMPRESSION: u8 = 1 << 1;
pub(super) const FRAME_FLAGS_MASK: u8 = 0b111;
pub(super) const FRAME_SIZE_LIMIT: usize = 1 << 26;

pub(super) const METRIC_FIELD_MASK: u64 = 0b11111111;
pub(super) const RESOURCE_FIELD_MASK: u64 = 0b111;
pub(super) const SCOPE_FIELD_MASK: u64 = 0b11111;
pub(super) const ROOT_METRIC_FIELD: u64 = 1 << 1;
pub(super) const ROOT_RESOURCE_FIELD: u64 = 1 << 2;
pub(super) const ROOT_SCOPE_FIELD: u64 = 1 << 3;
pub(super) const ROOT_ATTRIBUTES_FIELD: u64 = 1 << 4;
pub(super) const ROOT_POINT_FIELD: u64 = 1 << 5;

use zstd::stream::raw::{Encoder as ZstdEncoder, InBuffer, Operation, OutBuffer};

use super::{Error, METRICS_WIRE_SCHEMA, StefCompression};

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

pub(super) fn write_fixed_header(dst: &mut Vec<u8>, compression: StefCompression) {
    dst.extend_from_slice(HDR_SIGNATURE);
    append_uvarint(2, dst);
    dst.push(HDR_FORMAT_VERSION);
    dst.push(match compression {
        StefCompression::None => COMPRESSION_NONE,
        StefCompression::Zstd => COMPRESSION_ZSTD,
    });
}

pub(super) fn write_var_header_frame(
    frame_encoder: &mut FrameEncoder,
    dst: &mut Vec<u8>,
) -> Result<(), Error> {
    let mut var_header = Vec::with_capacity(METRICS_WIRE_SCHEMA.len() + 4);
    append_uvarint(METRICS_WIRE_SCHEMA.len() as u64, &mut var_header);
    var_header.extend_from_slice(METRICS_WIRE_SCHEMA);
    append_uvarint(0, &mut var_header);
    frame_encoder.write_frame(dst, FRAME_FLAG_NONE, &var_header)
}

pub(super) struct FrameEncoder {
    compression: StefCompression,
    zstd: Option<ZstdEncoder<'static>>,
    compressed: Vec<u8>,
}

impl FrameEncoder {
    pub(super) fn new(compression: StefCompression) -> Result<Self, Error> {
        let zstd = match compression {
            StefCompression::None => None,
            StefCompression::Zstd => Some(
                ZstdEncoder::new(zstd::DEFAULT_COMPRESSION_LEVEL)
                    .map_err(|_| Error::InvalidFrame("zstd encoder init failed"))?,
            ),
        };
        Ok(Self {
            compression,
            zstd,
            compressed: Vec::new(),
        })
    }

    pub(super) fn write_frame(
        &mut self,
        dst: &mut Vec<u8>,
        flags: u8,
        content: &[u8],
    ) -> Result<(), Error> {
        if flags & !FRAME_FLAGS_MASK != 0 {
            return Err(Error::InvalidFrame("unknown frame flags"));
        }
        if content.len() > FRAME_SIZE_LIMIT {
            return Err(Error::ValueOutOfRange("frame"));
        }

        match self.compression {
            StefCompression::None => {
                write_uncompressed_frame(dst, flags, content);
                Ok(())
            }
            StefCompression::Zstd => {
                let compressed = self.compress_zstd_frame(flags, content)?;
                dst.push(flags);
                append_uvarint(content.len() as u64, dst);
                append_uvarint(compressed.len() as u64, dst);
                dst.extend_from_slice(compressed);
                Ok(())
            }
        }
    }

    fn compress_zstd_frame(&mut self, flags: u8, content: &[u8]) -> Result<&[u8], Error> {
        if flags & FRAME_FLAG_RESTART_COMPRESSION != 0 {
            if let Some(encoder) = self.zstd.as_mut() {
                encoder
                    .reinit()
                    .map_err(|_| Error::InvalidFrame("zstd encoder reset failed"))?;
            }
        }

        self.compressed.clear();
        self.compressed.reserve(
            zstd::zstd_safe::compress_bound(content.len())
                .saturating_add(1024)
                .max(1024),
        );

        let encoder = self
            .zstd
            .as_mut()
            .ok_or(Error::InvalidFrame("zstd encoder unavailable"))?;
        let mut input = InBuffer::around(content);
        while input.pos < content.len() {
            let before_in = input.pos;
            let before_out = self.compressed.len();
            {
                let mut output = OutBuffer::around_pos(&mut self.compressed, before_out);
                let _remaining = encoder
                    .run(&mut input, &mut output)
                    .map_err(|_| Error::InvalidFrame("zstd frame encode failed"))?;
            }
            if input.pos == before_in && self.compressed.len() == before_out {
                self.compressed.reserve(64 * 1024);
            }
        }

        loop {
            let before_out = self.compressed.len();
            let remaining = {
                let mut output = OutBuffer::around_pos(&mut self.compressed, before_out);
                encoder
                    .flush(&mut output)
                    .map_err(|_| Error::InvalidFrame("zstd frame flush failed"))?
            };
            if remaining == 0 {
                break;
            }
            if self.compressed.len() == before_out {
                self.compressed.reserve(64 * 1024);
            }
        }

        if self.compressed.len() > FRAME_SIZE_LIMIT {
            return Err(Error::ValueOutOfRange("compressed frame"));
        }

        Ok(&self.compressed)
    }
}

fn write_uncompressed_frame(dst: &mut Vec<u8>, flags: u8, content: &[u8]) {
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
