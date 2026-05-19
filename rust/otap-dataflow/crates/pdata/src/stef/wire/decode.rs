// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Primitive STEF byte decoders and slice readers.

use std::borrow::Cow;
use std::sync::Arc;

use zstd::stream::raw::{Decoder as ZstdDecoder, InBuffer, Operation, OutBuffer};

use crate::stef::StefCompression;

use super::{
    COMPRESSION_NONE, COMPRESSION_ZSTD, DecodeColumn, Error, FRAME_FLAG_RESTART_COMPRESSION,
    FRAME_FLAGS_MASK, FRAME_SIZE_LIMIT, HDR_FORMAT_VERSION, HDR_SIGNATURE,
};

#[derive(Default)]
pub(in crate::stef) struct ArrayDecoder<'a> {
    bits: BitReader<'a>,
}

impl<'a> ArrayDecoder<'a> {
    pub(in crate::stef) fn new(column: &DecodeColumn<'a>) -> Self {
        Self {
            bits: BitReader::new(column.data),
        }
    }

    pub(in crate::stef) fn decode_empty(&mut self) -> Result<(), Error> {
        let len = self.bits.read_uvarint_compact()?;
        if len == 0 {
            Ok(())
        } else {
            Err(Error::UnsupportedStefValue("non-empty array"))
        }
    }
}

#[derive(Default)]
pub(in crate::stef) struct DirectStringDict {
    values: Vec<Arc<str>>,
}

impl DirectStringDict {
    pub(in crate::stef) fn reset(&mut self) {
        self.values.clear();
    }

    pub(in crate::stef) fn decode<'a>(
        &mut self,
        value: i64,
        reader: &mut BytesReader<'a>,
    ) -> Result<Arc<str>, Error> {
        if value >= 0 {
            let len = usize::try_from(value).map_err(|_| Error::ValueOutOfRange("string"))?;
            let value = Arc::<str>::from(reader.read_shared_utf8_string(len)?);
            if len > 1 {
                self.values.push(value.clone());
            }
            return Ok(value);
        }
        let ref_num = usize::try_from(-value - 1).map_err(|_| Error::InvalidRefNum)?;
        self.values
            .get(ref_num)
            .cloned()
            .ok_or(Error::InvalidRefNum)
    }
}

#[derive(Default)]
pub(in crate::stef) struct BytesReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BytesReader<'a> {
    pub(in crate::stef) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub(in crate::stef) fn read_uvarint(&mut self) -> Result<u64, Error> {
        read_uvarint_from_slice(self.bytes, &mut self.position)
    }

    pub(in crate::stef) fn read_varint(&mut self) -> Result<i64, Error> {
        let value = self.read_uvarint()?;
        Ok(((value >> 1) as i64) ^ (-((value & 1) as i64)))
    }

    pub(in crate::stef) fn read_direct_dict_string(
        &mut self,
        dict: &mut DirectStringDict,
    ) -> Result<Arc<str>, Error> {
        let value = self.read_varint()?;
        dict.decode(value, self)
    }

    pub(in crate::stef) fn read_plain_bytes(&mut self) -> Result<Arc<[u8]>, Error> {
        let len = usize::try_from(self.read_varint()?)
            .map_err(|_| Error::ValueOutOfRange("bytes length"))?;
        self.read_bytes(len).map(Arc::<[u8]>::from)
    }

    pub(in crate::stef) fn read_shared_utf8_string(
        &mut self,
        len: usize,
    ) -> Result<&'a str, Error> {
        let bytes = self.read_bytes(len)?;
        std::str::from_utf8(bytes).map_err(|_| Error::InvalidFrame("invalid utf8 string"))
    }

    pub(in crate::stef) fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(Error::ValueOutOfRange("reader position"))?;
        if end > self.bytes.len() {
            return Err(Error::UnexpectedEof);
        }
        let bytes = &self.bytes[self.position..end];
        self.position = end;
        Ok(bytes)
    }
}

#[derive(Clone, Copy, Default)]
pub(in crate::stef) struct U64DecoderState {
    last_value: u64,
    last_delta: u64,
}

#[derive(Default)]
pub(in crate::stef) struct U64Decoder<'a> {
    bytes: BytesReader<'a>,
    state: U64DecoderState,
}

impl<'a> U64Decoder<'a> {
    pub(in crate::stef) fn with_state(bytes: &'a [u8], state: U64DecoderState) -> Self {
        Self {
            bytes: BytesReader::new(bytes),
            state,
        }
    }

    pub(in crate::stef) fn decode(&mut self) -> Result<u64, Error> {
        let delta_of_delta = self.bytes.read_varint()?;
        let delta = self.state.last_delta.wrapping_add(delta_of_delta as u64);
        self.state.last_delta = delta;
        self.state.last_value = self.state.last_value.wrapping_add(delta);
        Ok(self.state.last_value)
    }

    pub(in crate::stef) fn state(&self) -> U64DecoderState {
        self.state
    }
}

#[derive(Default)]
pub(in crate::stef) struct I64Decoder<'a> {
    inner: U64Decoder<'a>,
}

impl<'a> I64Decoder<'a> {
    pub(in crate::stef) fn with_state(bytes: &'a [u8], state: U64DecoderState) -> Self {
        Self {
            inner: U64Decoder::with_state(bytes, state),
        }
    }

    pub(in crate::stef) fn decode(&mut self) -> Result<i64, Error> {
        self.inner.decode().map(|value| value as i64)
    }

    pub(in crate::stef) fn state(&self) -> U64DecoderState {
        self.inner.state()
    }
}

#[derive(Clone, Copy, Default)]
pub(in crate::stef) struct Float64DecoderState {
    last_value: f64,
    leading_bits: u64,
    trailing_bits: u64,
}

#[derive(Default)]
pub(in crate::stef) struct BoolDecoder<'a> {
    bits: BitReader<'a>,
}

impl<'a> BoolDecoder<'a> {
    pub(in crate::stef) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bits: BitReader::new(bytes),
        }
    }

    pub(in crate::stef) fn decode(&mut self) -> Result<bool, Error> {
        self.bits.read_bit()
    }
}

#[derive(Default)]
pub(in crate::stef) struct Float64Decoder<'a> {
    bits: BitReader<'a>,
    state: Float64DecoderState,
}

impl<'a> Float64Decoder<'a> {
    pub(in crate::stef) fn with_state(bytes: &'a [u8], state: Float64DecoderState) -> Self {
        Self {
            bits: BitReader::new(bytes),
            state,
        }
    }

    pub(in crate::stef) fn decode(&mut self) -> Result<f64, Error> {
        if !self.bits.read_bit()? {
            return Ok(self.state.last_value);
        }
        let same_window = !self.bits.read_bit()?;
        let (leading, trailing, significant_bits) = if same_window {
            (
                self.state.leading_bits,
                self.state.trailing_bits,
                64 - self.state.leading_bits - self.state.trailing_bits,
            )
        } else {
            let leading = self.bits.read_bits(5)?;
            let significant_bits = self.bits.read_bits(6)? + 1;
            let trailing = 64 - leading - significant_bits;
            self.state.leading_bits = leading;
            self.state.trailing_bits = trailing;
            (leading, trailing, significant_bits)
        };
        let _ = leading;
        let xor_value = self.bits.read_bits(significant_bits as u32)? << trailing;
        self.state.last_value = f64::from_bits(xor_value ^ self.state.last_value.to_bits());
        Ok(self.state.last_value)
    }

    pub(in crate::stef) fn state(&self) -> Float64DecoderState {
        self.state
    }
}

#[derive(Default)]
pub(in crate::stef) struct BitReader<'a> {
    bytes: &'a [u8],
    bit_position: usize,
}

impl<'a> BitReader<'a> {
    pub(in crate::stef) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_position: 0,
        }
    }

    pub(in crate::stef) fn read_bit(&mut self) -> Result<bool, Error> {
        let byte_index = self.bit_position / 8;
        if byte_index >= self.bytes.len() {
            return Err(Error::UnexpectedEof);
        }
        let bit_index = 7 - (self.bit_position % 8);
        self.bit_position += 1;
        Ok(((self.bytes[byte_index] >> bit_index) & 1) == 1)
    }

    pub(in crate::stef) fn read_bits(&mut self, bit_count: u32) -> Result<u64, Error> {
        let mut value = 0;
        for _ in 0..bit_count {
            value = (value << 1) | u64::from(self.read_bit()?);
        }
        Ok(value)
    }

    pub(in crate::stef) fn read_uvarint_compact(&mut self) -> Result<u64, Error> {
        let mut zeros = 0;
        while !self.read_bit()? {
            zeros += 1;
            if zeros > 7 {
                return Err(Error::InvalidFrame("invalid compact integer"));
            }
        }
        let bit_count = match zeros {
            0 => return Ok(0),
            1 => 2,
            2 => 5,
            3 => 12,
            4 => 19,
            5 => 26,
            6 => 33,
            7 => 48,
            _ => return Err(Error::InvalidFrame("invalid compact integer")),
        };
        self.read_bits(bit_count)
    }
}

pub(in crate::stef) struct SliceReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> SliceReader<'a> {
    pub(in crate::stef) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub(in crate::stef) fn is_empty(&self) -> bool {
        self.position >= self.bytes.len()
    }

    pub(in crate::stef) fn remaining_len(&self) -> usize {
        self.bytes.len().saturating_sub(self.position)
    }

    pub(in crate::stef) fn position(&self) -> usize {
        self.position
    }

    pub(in crate::stef) fn read_u8(&mut self) -> Result<u8, Error> {
        Ok(*self.read_bytes(1)?.first().expect("one byte"))
    }

    pub(in crate::stef) fn read_uvarint(&mut self) -> Result<u64, Error> {
        read_uvarint_from_slice(self.bytes, &mut self.position)
    }

    pub(in crate::stef) fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(Error::ValueOutOfRange("reader position"))?;
        if end > self.bytes.len() {
            return Err(Error::UnexpectedEof);
        }
        let bytes = &self.bytes[self.position..end];
        self.position = end;
        Ok(bytes)
    }
}

pub(in crate::stef) fn read_uvarint_from_slice(
    bytes: &[u8],
    position: &mut usize,
) -> Result<u64, Error> {
    let mut value = 0_u64;
    let mut shift = 0;
    loop {
        if *position >= bytes.len() {
            return Err(Error::UnexpectedEof);
        }
        let byte = bytes[*position];
        *position += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift >= 64 {
            return Err(Error::ValueOutOfRange("uvarint"));
        }
    }
}

pub(in crate::stef) fn read_fixed_header(
    input: &mut SliceReader<'_>,
) -> Result<StefCompression, Error> {
    if input.read_bytes(4)? != HDR_SIGNATURE {
        return Err(Error::InvalidHeader("bad signature"));
    }
    let header_size = usize::try_from(input.read_uvarint()?)
        .map_err(|_| Error::ValueOutOfRange("fixed header"))?;
    if header_size < 2 {
        return Err(Error::InvalidHeader("too short"));
    }
    let header = input.read_bytes(header_size)?;
    if header[0] & 0x0f != HDR_FORMAT_VERSION {
        return Err(Error::InvalidHeader("unsupported version"));
    }
    match header[1] & 0b11 {
        COMPRESSION_NONE => Ok(StefCompression::None),
        COMPRESSION_ZSTD => Ok(StefCompression::Zstd),
        _ => Err(Error::InvalidHeader("unsupported compression")),
    }
}

pub(in crate::stef) struct FrameDecoder {
    compression: StefCompression,
    zstd: Option<ZstdDecoder<'static>>,
    seen_compressed_frame: bool,
}

impl FrameDecoder {
    pub(in crate::stef) fn new(compression: StefCompression) -> Result<Self, Error> {
        let zstd = match compression {
            StefCompression::None => None,
            StefCompression::Zstd => Some(
                ZstdDecoder::new().map_err(|_| Error::InvalidFrame("zstd decoder init failed"))?,
            ),
        };
        Ok(Self {
            compression,
            zstd,
            seen_compressed_frame: false,
        })
    }

    pub(in crate::stef) fn read_frame<'a>(
        &mut self,
        input: &mut SliceReader<'a>,
    ) -> Result<(u8, Cow<'a, [u8]>), Error> {
        let flags = input.read_u8()?;
        if flags & !FRAME_FLAGS_MASK != 0 {
            return Err(Error::InvalidFrame("unknown frame flags"));
        }

        let uncompressed_len =
            usize::try_from(input.read_uvarint()?).map_err(|_| Error::ValueOutOfRange("frame"))?;
        if uncompressed_len > FRAME_SIZE_LIMIT {
            return Err(Error::ValueOutOfRange("frame"));
        }

        match self.compression {
            StefCompression::None => {
                let content = input.read_bytes(uncompressed_len)?;
                Ok((flags, Cow::Borrowed(content)))
            }
            StefCompression::Zstd => {
                let compressed_len = usize::try_from(input.read_uvarint()?)
                    .map_err(|_| Error::ValueOutOfRange("compressed frame"))?;
                if compressed_len > FRAME_SIZE_LIMIT {
                    return Err(Error::ValueOutOfRange("compressed frame"));
                }
                let compressed = input.read_bytes(compressed_len)?;
                let content = self.decode_zstd_frame(flags, compressed, uncompressed_len)?;
                Ok((flags, Cow::Owned(content)))
            }
        }
    }

    fn decode_zstd_frame(
        &mut self,
        flags: u8,
        compressed: &[u8],
        uncompressed_len: usize,
    ) -> Result<Vec<u8>, Error> {
        let decoder = self
            .zstd
            .as_mut()
            .ok_or(Error::InvalidFrame("zstd decoder unavailable"))?;

        if !self.seen_compressed_frame || flags & FRAME_FLAG_RESTART_COMPRESSION != 0 {
            decoder
                .reinit()
                .map_err(|_| Error::InvalidFrame("zstd decoder reset failed"))?;
            self.seen_compressed_frame = true;
        }

        let mut output = vec![0; uncompressed_len];
        let mut input = InBuffer::around(compressed);
        let mut out = OutBuffer::around(output.as_mut_slice());

        while out.pos() < uncompressed_len {
            let before_in = input.pos;
            let before_out = out.pos();
            let _remaining = decoder
                .run(&mut input, &mut out)
                .map_err(|_| Error::InvalidFrame("zstd frame decode failed"))?;
            if input.pos == before_in && out.pos() == before_out {
                return Err(Error::InvalidFrame("zstd decoder made no progress"));
            }
        }

        if input.pos != compressed.len() {
            return Err(Error::InvalidFrame("zstd frame size mismatch"));
        }

        Ok(output)
    }
}

pub(in crate::stef) fn read_var_header(bytes: &[u8]) -> Result<(), Error> {
    let mut reader = SliceReader::new(bytes);
    let schema_len =
        usize::try_from(reader.read_uvarint()?).map_err(|_| Error::ValueOutOfRange("schema"))?;
    let _schema = reader.read_bytes(schema_len)?;
    let user_data_count = reader.read_uvarint()?;
    for _ in 0..user_data_count {
        let key_len = usize::try_from(reader.read_uvarint()?)
            .map_err(|_| Error::ValueOutOfRange("user data key"))?;
        let _ = reader.read_bytes(key_len)?;
        let value_len = usize::try_from(reader.read_uvarint()?)
            .map_err(|_| Error::ValueOutOfRange("user data value"))?;
        let _ = reader.read_bytes(value_len)?;
    }
    Ok(())
}

pub(in crate::stef) fn read_decode_column_sizes(
    column: &mut DecodeColumn<'_>,
    sizes: &mut BitReader<'_>,
    mut read_limit: u64,
) -> Result<(), Error> {
    read_decode_column_sizes_inner(column, sizes, &mut read_limit)
}

pub(in crate::stef) fn read_decode_column_sizes_inner(
    column: &mut DecodeColumn<'_>,
    sizes: &mut BitReader<'_>,
    read_limit: &mut u64,
) -> Result<(), Error> {
    let size = sizes.read_uvarint_compact()?;
    if size > *read_limit {
        return Err(Error::InvalidFrame("column exceeds frame"));
    }
    *read_limit -= size;
    column.byte_len = usize::try_from(size).map_err(|_| Error::ValueOutOfRange("column"))?;
    column.data = &[];
    if size == 0 {
        reset_decode_column_data(column);
        return Ok(());
    }
    for child in &mut column.children {
        read_decode_column_sizes_inner(child, sizes, read_limit)?;
    }
    Ok(())
}

pub(in crate::stef) fn reset_decode_column_data(column: &mut DecodeColumn<'_>) {
    column.data = &[];
    column.byte_len = 0;
    for child in &mut column.children {
        reset_decode_column_data(child);
    }
}

pub(in crate::stef) fn read_decode_column_data<'a>(
    column: &mut DecodeColumn<'a>,
    reader: &mut SliceReader<'a>,
) -> Result<(), Error> {
    if column.byte_len == 0 {
        return Ok(());
    }
    column.data = reader.read_bytes(column.byte_len)?;
    for child in &mut column.children {
        read_decode_column_data(child, reader)?;
    }
    Ok(())
}
