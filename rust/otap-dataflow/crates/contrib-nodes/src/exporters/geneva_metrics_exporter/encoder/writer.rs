// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Binary writing helpers for the Geneva Metrics ingestion protocol.

#[derive(Default)]
pub(super) struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub(super) fn len(&self) -> usize {
        self.bytes.len()
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(super) fn finish(self) -> Vec<u8> {
        self.bytes
    }

    pub(super) fn reserve(&mut self, count: usize) -> usize {
        let position = self.len();
        self.bytes.resize(position + count, 0);
        position
    }

    pub(super) fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub(super) fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub(super) fn write_u16(&mut self, value: u16) {
        self.write_bytes(&value.to_le_bytes());
    }

    pub(super) fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_le_bytes());
    }

    pub(super) fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    pub(super) fn write_f64(&mut self, value: f64) {
        self.write_bytes(&value.to_le_bytes());
    }

    pub(super) fn write_u8_at(&mut self, position: usize, value: u8) {
        self.bytes[position] = value;
    }

    pub(super) fn write_u16_at(&mut self, position: usize, value: u16) {
        self.bytes[position..position + size_of::<u16>()].copy_from_slice(&value.to_le_bytes());
    }

    pub(super) fn write_u32_at(&mut self, position: usize, value: u32) {
        self.bytes[position..position + size_of::<u32>()].copy_from_slice(&value.to_le_bytes());
    }

    pub(super) fn write_u64_at(&mut self, position: usize, value: u64) {
        self.bytes[position..position + size_of::<u64>()].copy_from_slice(&value.to_le_bytes());
    }

    pub(super) fn write_unsigned_base128(&mut self, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte);
            if value == 0 {
                break;
            }
        }
    }

    pub(super) fn write_signed_base128(&mut self, value: i64) {
        let negative = value < 0;
        let mut remaining = value.unsigned_abs();
        let mut first = true;
        loop {
            let mut byte = if first {
                let mut byte = (remaining & 0x3f) as u8;
                remaining >>= 6;
                if negative {
                    byte |= 0x40;
                }
                first = false;
                byte
            } else {
                let byte = (remaining & 0x7f) as u8;
                remaining >>= 7;
                byte
            };
            if remaining != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte);
            if remaining == 0 {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_signed_base128(bytes: &[u8]) -> i64 {
        let negative = bytes[0] & 0x40 != 0;
        let mut value = u64::from(bytes[0] & 0x3f);
        let mut shift = 6;
        for byte in &bytes[1..] {
            value |= u64::from(byte & 0x7f) << shift;
            shift += 7;
        }
        if negative {
            -(value as i128) as i64
        } else {
            value as i64
        }
    }

    /// Scenario: Unsigned values cross each base-128 byte boundary.
    /// Guarantees: The writer emits the expected compact little-endian base-128 representation.
    #[test]
    fn writes_unsigned_base128_boundaries() {
        for (value, expected) in [
            (0, vec![0x00]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (16_383, vec![0xff, 0x7f]),
            (16_384, vec![0x80, 0x80, 0x01]),
            (u64::MAX, vec![0xff; 9].into_iter().chain([0x01]).collect()),
        ] {
            let mut writer = Writer::default();
            writer.write_unsigned_base128(value);
            assert_eq!(writer.finish(), expected, "value {value}");
        }
    }

    /// Scenario: Signed values include positive, negative, and integer-limit boundaries.
    /// Guarantees: Sign-magnitude base-128 output round-trips without losing the i64 extremes.
    #[test]
    fn writes_signed_base128_boundaries() {
        for value in [
            i64::MIN,
            -16_384,
            -64,
            -63,
            -1,
            0,
            1,
            63,
            64,
            16_384,
            i64::MAX,
        ] {
            let mut writer = Writer::default();
            writer.write_signed_base128(value);
            let bytes = writer.finish();
            assert_eq!(read_signed_base128(&bytes), value, "bytes {bytes:02x?}");
        }
    }

    /// Scenario: Fixed-width protocol fields are written in native packet byte order.
    /// Guarantees: Integers and doubles are serialized as little-endian bytes.
    #[test]
    fn writes_fixed_width_values_little_endian() {
        let mut writer = Writer::default();
        writer.write_u16(0x1234);
        writer.write_u32(0x1234_5678);
        writer.write_u64(0x0123_4567_89ab_cdef);
        writer.write_f64(1.5);

        let mut expected = Vec::new();
        expected.extend_from_slice(&0x1234_u16.to_le_bytes());
        expected.extend_from_slice(&0x1234_5678_u32.to_le_bytes());
        expected.extend_from_slice(&0x0123_4567_89ab_cdef_u64.to_le_bytes());
        expected.extend_from_slice(&1.5_f64.to_le_bytes());
        assert_eq!(writer.finish(), expected);
    }

    /// Scenario: Packet offsets and lengths are unknown until their bodies have been written.
    /// Guarantees: Reserved bytes can be backfilled without changing surrounding content.
    #[test]
    fn reserves_and_backfills_fields() {
        let mut writer = Writer::default();
        writer.write_u8(0xaa);
        let u8_position = writer.reserve(size_of::<u8>());
        let u16_position = writer.reserve(size_of::<u16>());
        let u32_position = writer.reserve(size_of::<u32>());
        let u64_position = writer.reserve(size_of::<u64>());
        writer.write_u8(0xbb);

        writer.write_u8_at(u8_position, 0x12);
        writer.write_u16_at(u16_position, 0x3456);
        writer.write_u32_at(u32_position, 0x789a_bcde);
        writer.write_u64_at(u64_position, 0x0123_4567_89ab_cdef);

        let bytes = writer.finish();
        assert_eq!(bytes[0], 0xaa);
        assert_eq!(bytes[u8_position], 0x12);
        assert_eq!(
            &bytes[u16_position..u16_position + size_of::<u16>()],
            &0x3456_u16.to_le_bytes()
        );
        assert_eq!(
            &bytes[u32_position..u32_position + size_of::<u32>()],
            &0x789a_bcde_u32.to_le_bytes()
        );
        assert_eq!(
            &bytes[u64_position..u64_position + size_of::<u64>()],
            &0x0123_4567_89ab_cdef_u64.to_le_bytes()
        );
        assert_eq!(bytes.last(), Some(&0xbb));
    }
}
