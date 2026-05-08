// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Primitive STEF byte encoders.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{Column, Error, append_uvarint, array_column_tree};

#[derive(Default)]
pub(in crate::stef) struct Float64ArrayEncoder {
    bits: BitWriter,
}

impl Float64ArrayEncoder {
    pub(in crate::stef) fn encode_empty(&mut self) {
        self.bits.write_uvarint_compact(0);
    }

    pub(in crate::stef) fn take_column(&mut self) -> Column {
        let mut column = array_column_tree(Column::default());
        column.data = self.bits.take_bytes();
        column
    }
}

#[derive(Default)]
pub(in crate::stef) struct ExemplarArrayEncoder {
    bits: BitWriter,
}

impl ExemplarArrayEncoder {
    pub(in crate::stef) fn take_column(&mut self) -> Column {
        let mut column = array_column_tree(Column::default());
        column.data = self.bits.take_bytes();
        column
    }
}
#[derive(Clone, Default)]
pub(in crate::stef) struct SharedStringDict(Rc<RefCell<StefStringDict>>);

pub(in crate::stef) type StefStringDict = HashMap<String, usize, ahash::RandomState>;

#[derive(Default)]
pub(in crate::stef) struct StringEncoder {
    bytes: BytesWriter,
    dict: SharedStringDict,
}

impl StringEncoder {
    pub(in crate::stef) fn with_dict(dict: SharedStringDict) -> Self {
        Self {
            bytes: BytesWriter::default(),
            dict,
        }
    }

    pub(in crate::stef) fn encode(&mut self, value: &str) {
        if value.len() <= 1 {
            self.encode_literal(value);
            return;
        }

        let mut dict = self.dict.0.borrow_mut();
        if let Some(ref_num) = dict.get(value) {
            self.bytes.write_varint(-(*ref_num as i64) - 1);
            return;
        }
        let ref_num = dict.len();
        let _ = dict.insert(value.to_owned(), ref_num);
        drop(dict);
        self.encode_literal(value);
    }

    pub(in crate::stef) fn encode_literal(&mut self, value: &str) {
        self.bytes.write_varint(value.len() as i64);
        self.bytes.write_bytes(value.as_bytes());
    }

    pub(in crate::stef) fn encode_bytes(
        &mut self,
        value: &[u8],
        field: &'static str,
    ) -> Result<(), Error> {
        let value = std::str::from_utf8(value).map_err(|_| Error::InvalidUtf8(field))?;
        self.encode(value);
        Ok(())
    }

    pub(in crate::stef) fn take_bytes(&mut self) -> Vec<u8> {
        self.bytes.take_bytes()
    }
}

#[derive(Default)]
pub(in crate::stef) struct BytesEncoder {
    bytes: BytesWriter,
}

impl BytesEncoder {
    pub(in crate::stef) fn encode(&mut self, value: &[u8]) {
        self.bytes.write_varint(value.len() as i64);
        self.bytes.write_bytes(value);
    }

    pub(in crate::stef) fn take_bytes(&mut self) -> Vec<u8> {
        self.bytes.take_bytes()
    }
}

#[derive(Default)]
pub(in crate::stef) struct BoolEncoder {
    bits: BitWriter,
}

impl BoolEncoder {
    pub(in crate::stef) fn encode(&mut self, value: bool) {
        self.bits.write_bit(value);
    }

    pub(in crate::stef) fn take_bytes(&mut self) -> Vec<u8> {
        self.bits.take_bytes()
    }
}

#[derive(Default)]
pub(in crate::stef) struct U64Encoder {
    bytes: BytesWriter,
    last_value: u64,
    last_delta: u64,
}

impl U64Encoder {
    pub(in crate::stef) fn encode(&mut self, value: u64) {
        let delta = value.wrapping_sub(self.last_value);
        self.last_value = value;
        let delta_of_delta = delta.wrapping_sub(self.last_delta);
        self.last_delta = delta;
        self.bytes.write_varint(delta_of_delta as i64);
    }

    pub(in crate::stef) fn take_bytes(&mut self) -> Vec<u8> {
        self.bytes.take_bytes()
    }
}

#[derive(Default)]
pub(in crate::stef) struct I64Encoder {
    inner: U64Encoder,
}

impl I64Encoder {
    pub(in crate::stef) fn encode(&mut self, value: i64) {
        self.inner.encode(value as u64);
    }

    pub(in crate::stef) fn take_bytes(&mut self) -> Vec<u8> {
        self.inner.take_bytes()
    }
}

#[derive(Default)]
pub(in crate::stef) struct Float64Encoder {
    bits: BitWriter,
    last_value: f64,
    leading_bits: u32,
    trailing_bits: u32,
}

impl Float64Encoder {
    pub(in crate::stef) fn encode(&mut self, value: f64) {
        let xor_value = value.to_bits() ^ self.last_value.to_bits();
        self.last_value = value;
        if xor_value == 0 {
            self.bits.write_bit(false);
            return;
        }

        let mut leading = xor_value.leading_zeros();
        if leading >= 32 {
            leading = 31;
        }
        let trailing = xor_value.trailing_zeros();
        let significant_bits = 64 - leading - trailing;

        if leading >= self.leading_bits && trailing >= self.trailing_bits {
            let current_bit_count = 64 - self.leading_bits - self.trailing_bits;
            if 53 - self.leading_bits - self.trailing_bits <= significant_bits {
                self.bits.write_bits(0b10, 2);
                self.bits
                    .write_bits(xor_value >> self.trailing_bits, current_bit_count);
                return;
            }
        }

        self.leading_bits = leading;
        self.trailing_bits = trailing;
        let mut header = 0b11_u64;
        header = (header << 5) | u64::from(leading);
        header = (header << 6) | u64::from(significant_bits - 1);
        self.bits.write_bits(header, 13);
        self.bits
            .write_bits(xor_value >> trailing, significant_bits);
    }

    pub(in crate::stef) fn take_bytes(&mut self) -> Vec<u8> {
        self.bits.take_bytes()
    }
}

#[derive(Default)]
pub(in crate::stef) struct BytesWriter {
    bytes: Vec<u8>,
}

impl BytesWriter {
    pub(in crate::stef) fn write_uvarint(&mut self, value: u64) {
        append_uvarint(value, &mut self.bytes);
    }

    pub(in crate::stef) fn write_varint(&mut self, value: i64) {
        let encoded = ((value >> 63) ^ (value << 1)) as u64;
        self.write_uvarint(encoded);
    }

    pub(in crate::stef) fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub(in crate::stef) fn take_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }
}

#[derive(Default)]
pub(in crate::stef) struct BitWriter {
    bytes: Vec<u8>,
    current: u8,
    used: u8,
}

impl BitWriter {
    pub(in crate::stef) fn write_bit(&mut self, value: bool) {
        if value {
            self.current |= 1 << (7 - self.used);
        }
        self.used += 1;
        if self.used == 8 {
            self.bytes.push(self.current);
            self.current = 0;
            self.used = 0;
        }
    }

    pub(in crate::stef) fn write_bits(&mut self, value: u64, bit_count: u32) {
        debug_assert!(bit_count <= 64);

        let mut remaining = bit_count;
        while remaining > 0 {
            let available = u32::from(8 - self.used);
            let take = remaining.min(available);
            let shift = remaining - take;
            let mask = (1_u64 << take) - 1;
            let bits = ((value >> shift) & mask) as u8;

            self.current |= bits << (available - take);
            self.used += take as u8;
            remaining -= take;

            if self.used == 8 {
                self.bytes.push(self.current);
                self.current = 0;
                self.used = 0;
            }
        }
    }

    pub(in crate::stef) fn write_uvarint_compact(&mut self, value: u64) {
        if value == 0 {
            self.write_bit(true);
        } else if value < (1 << 2) {
            self.write_bits(0b01, 2);
            self.write_bits(value, 2);
        } else if value < (1 << 5) {
            self.write_bits(0b001, 3);
            self.write_bits(value, 5);
        } else if value < (1 << 12) {
            self.write_bits(0b0001, 4);
            self.write_bits(value, 12);
        } else if value < (1 << 19) {
            self.write_bits(0b00001, 5);
            self.write_bits(value, 19);
        } else if value < (1 << 26) {
            self.write_bits(0b000001, 6);
            self.write_bits(value, 26);
        } else if value < (1_u64 << 33) {
            self.write_bits(0b0000001, 7);
            self.write_bits(value, 33);
        } else {
            self.write_bits(0b00000001, 8);
            self.write_bits(value, 48);
        }
    }

    pub(in crate::stef) fn take_bytes(&mut self) -> Vec<u8> {
        if self.used != 0 {
            self.bytes.push(self.current);
            self.current = 0;
            self.used = 0;
        }
        std::mem::take(&mut self.bytes)
    }
}
