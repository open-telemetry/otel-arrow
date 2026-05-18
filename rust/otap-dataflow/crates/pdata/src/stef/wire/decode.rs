// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Primitive STEF byte decoders and slice readers.

use super::{COMPRESSION_NONE, DecodeColumn, Error, HDR_FORMAT_VERSION, HDR_SIGNATURE};

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
pub(in crate::stef) struct DirectStringDict<'a> {
    values: Vec<&'a str>,
}

impl<'a> DirectStringDict<'a> {
    pub(in crate::stef) fn reset(&mut self) {
        self.values.clear();
    }

    pub(in crate::stef) fn decode(
        &mut self,
        value: i64,
        reader: &mut BytesReader<'a>,
    ) -> Result<&'a str, Error> {
        if value >= 0 {
            let len = usize::try_from(value).map_err(|_| Error::ValueOutOfRange("string"))?;
            let value = reader.read_shared_utf8_string(len)?;
            if len > 1 {
                self.values.push(value);
            }
            return Ok(value);
        }
        let ref_num = usize::try_from(-value - 1).map_err(|_| Error::InvalidRefNum)?;
        self.values
            .get(ref_num)
            .copied()
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
        dict: &mut DirectStringDict<'a>,
    ) -> Result<&'a str, Error> {
        let value = self.read_varint()?;
        dict.decode(value, self)
    }

    pub(in crate::stef) fn read_plain_bytes(&mut self) -> Result<&'a [u8], Error> {
        let len = usize::try_from(self.read_varint()?)
            .map_err(|_| Error::ValueOutOfRange("bytes length"))?;
        self.read_bytes(len)
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

#[derive(Default)]
pub(in crate::stef) struct U64Decoder<'a> {
    bytes: BytesReader<'a>,
    last_value: u64,
    last_delta: u64,
}

impl<'a> U64Decoder<'a> {
    pub(in crate::stef) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: BytesReader::new(bytes),
            last_value: 0,
            last_delta: 0,
        }
    }

    pub(in crate::stef) fn decode(&mut self) -> Result<u64, Error> {
        let delta_of_delta = self.bytes.read_varint()?;
        let delta = self.last_delta.wrapping_add(delta_of_delta as u64);
        self.last_delta = delta;
        self.last_value = self.last_value.wrapping_add(delta);
        Ok(self.last_value)
    }
}

#[derive(Default)]
pub(in crate::stef) struct I64Decoder<'a> {
    inner: U64Decoder<'a>,
}

impl<'a> I64Decoder<'a> {
    pub(in crate::stef) fn new(bytes: &'a [u8]) -> Self {
        Self {
            inner: U64Decoder::new(bytes),
        }
    }

    pub(in crate::stef) fn decode(&mut self) -> Result<i64, Error> {
        self.inner.decode().map(|value| value as i64)
    }
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
    last_value: f64,
    leading_bits: u64,
    trailing_bits: u64,
}

impl<'a> Float64Decoder<'a> {
    pub(in crate::stef) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bits: BitReader::new(bytes),
            last_value: 0.0,
            leading_bits: 0,
            trailing_bits: 0,
        }
    }

    pub(in crate::stef) fn decode(&mut self) -> Result<f64, Error> {
        if !self.bits.read_bit()? {
            return Ok(self.last_value);
        }
        let same_window = !self.bits.read_bit()?;
        let (leading, trailing, significant_bits) = if same_window {
            (
                self.leading_bits,
                self.trailing_bits,
                64 - self.leading_bits - self.trailing_bits,
            )
        } else {
            let leading = self.bits.read_bits(5)?;
            let significant_bits = self.bits.read_bits(6)? + 1;
            let trailing = 64 - leading - significant_bits;
            self.leading_bits = leading;
            self.trailing_bits = trailing;
            (leading, trailing, significant_bits)
        };
        let _ = leading;
        let xor_value = self.bits.read_bits(significant_bits as u32)? << trailing;
        self.last_value = f64::from_bits(xor_value ^ self.last_value.to_bits());
        Ok(self.last_value)
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

pub(in crate::stef) fn read_fixed_header(input: &mut SliceReader<'_>) -> Result<(), Error> {
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
    if header[1] & 0b11 != COMPRESSION_NONE {
        return Err(Error::InvalidHeader("compressed STEF is not implemented"));
    }
    Ok(())
}

pub(in crate::stef) fn read_frame<'a>(
    input: &mut SliceReader<'a>,
) -> Result<(u8, &'a [u8]), Error> {
    let flags = input.read_u8()?;
    let len =
        usize::try_from(input.read_uvarint()?).map_err(|_| Error::ValueOutOfRange("frame"))?;
    let content = input.read_bytes(len)?;
    Ok((flags, content))
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
