use bytes::{BufMut, Bytes, BytesMut};

use crate::{serializer::*, *};

const RESERVATION_SIZE_IN_BYTES: usize = 4;

pub struct ProtobufWriter {
    data: BytesMut,
}

impl ProtobufWriter {
    pub fn new(initial_capacity: usize) -> ProtobufWriter {
        ProtobufWriter {
            data: BytesMut::with_capacity(initial_capacity),
        }
    }

    #[inline(always)]
    pub fn write_field(&mut self, field: &ProtobufField) {
        match field {
            ProtobufField::Varint {
                field_number,
                value,
            } => {
                self.write_tag(*field_number, ProtobufWireType::Varint);
                self.write_varint64(*value);
            }
            ProtobufField::Fixed32 {
                field_number,
                value,
            } => {
                self.write_fixed32_field(*field_number, *value);
            }
            ProtobufField::Fixed64 {
                field_number,
                value,
            } => {
                self.write_fixed64_field(*field_number, *value);
            }
            ProtobufField::LengthDelimited {
                field_number,
                value,
            } => {
                self.write_bytes_field(*field_number, value);
            }
        }
    }

    #[inline(always)]
    pub fn write_fixed64_field(&mut self, field_number: u32, v: u64) {
        self.write_tag(field_number, ProtobufWireType::Fixed64);
        self.data.put_u64_le(v);
    }

    #[inline(always)]
    pub fn write_fixed32_field(&mut self, field_number: u32, v: u32) {
        self.write_tag(field_number, ProtobufWireType::Fixed32);
        self.data.put_u32_le(v);
    }

    #[inline(always)]
    pub fn write_bytes_field(&mut self, field_number: u32, bytes: &[u8]) {
        self.write_tag(field_number, ProtobufWireType::LengthDelimited);
        self.write_varint32(bytes.len() as u32);
        self.data.extend_from_slice(bytes);
    }

    #[inline(always)]
    pub fn write_int64_field(&mut self, field_number: u32, v: i64) {
        self.write_tag(field_number, ProtobufWireType::Varint);
        self.write_varint64(v as u64);
    }

    #[inline(always)]
    pub fn write_int32_field(&mut self, field_number: u32, v: i32) {
        self.write_tag(field_number, ProtobufWireType::Varint);
        self.write_varint32(v as u32);
    }

    #[inline(always)]
    pub fn write_double_field(&mut self, field_number: u32, v: f64) {
        self.write_tag(field_number, ProtobufWireType::Fixed64);
        self.data.put_f64_le(v);
    }

    #[inline(always)]
    pub fn write_bool_field(&mut self, field_number: u32, v: bool) {
        self.write_tag(field_number, ProtobufWireType::Varint);
        self.data.put_u8(if v { 1 } else { 0 });
    }

    #[inline(always)]
    pub fn write_string_field(&mut self, field_number: u32, s: &str) {
        self.write_bytes_field(field_number, s.as_bytes());
    }

    pub fn write_message_field<F>(
        &mut self,
        field_number: u32,
        write_message: F,
    ) -> Result<(), SerializerError>
    where
        F: FnOnce(&mut Self) -> Result<(), SerializerError>,
    {
        self.write_tag(field_number, ProtobufWireType::LengthDelimited);

        let reservation_position = self.reserve_varint32();

        write_message(self)?;

        self.write_reserved_varint32(
            reservation_position,
            (self.get_position() - reservation_position - RESERVATION_SIZE_IN_BYTES) as u32,
        )
    }

    pub fn freeze(self) -> Bytes {
        self.data.freeze()
    }

    #[inline(always)]
    fn get_position(&self) -> usize {
        self.data.len()
    }

    #[inline(always)]
    fn write_varint64(&mut self, mut v: u64) {
        let mut buffer = [0x00; 10];
        let mut count = 0;
        unsafe {
            let mut ptr = buffer.as_mut_ptr();
            while v > 0x7F {
                *ptr = ((v as u8) & 0x7F) | 0x80;
                ptr = ptr.add(1);
                count += 1;
                v >>= 7;
            }
            *ptr = v as u8;
        }
        self.data.extend_from_slice(&buffer[0..(count + 1)]);
    }

    #[inline(always)]
    fn write_varint32(&mut self, mut v: u32) {
        let mut buffer = [0x00; 5];
        let mut count = 0;
        unsafe {
            let mut ptr = buffer.as_mut_ptr();
            while v > 0x7F {
                *ptr = ((v as u8) & 0x7F) | 0x80;
                ptr = ptr.add(1);
                count += 1;
                v >>= 7;
            }
            *ptr = v as u8;
        }
        self.data.extend_from_slice(&buffer[0..(count + 1)]);
    }

    #[inline(always)]
    fn write_tag(&mut self, field_number: u32, wire_type: ProtobufWireType) {
        self.write_varint32((field_number << TAG_TYPE_BITS) | (wire_type as u32));
    }

    #[inline(always)]
    fn reserve_varint32(&mut self) -> usize {
        let position = self.data.len();
        self.data.extend_from_slice(&[0, 0, 0, 0]);
        position
    }

    #[inline(always)]
    fn write_reserved_varint32(
        &mut self,
        position: usize,
        mut v: u32,
    ) -> Result<(), SerializerError> {
        if v > 268435455 {
            return Err(SerializerError::Message(
                "Cannot write u32 greater than 268435455 using write_reserved_varint32",
            ));
        }
        let mut buffer: [u8; RESERVATION_SIZE_IN_BYTES] = [0x80, 0x80, 0x80, 0x00];
        unsafe {
            let mut ptr = buffer.as_mut_ptr();
            while v > 0x7F {
                *ptr |= (v as u8) & 0x7F;
                ptr = ptr.add(1);
                v >>= 7;
            }
            *ptr |= v as u8;
        }
        self.data[position..position + RESERVATION_SIZE_IN_BYTES].copy_from_slice(&buffer);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::serializer::protobuf_reader::ProtobufReader;

    use super::*;

    #[test]
    fn test_reserve_varint32() {
        let run_test = |value: u32, expected: &[u8]| {
            let mut writer = ProtobufWriter::new(0);

            let position = writer.reserve_varint32();

            assert_eq!(RESERVATION_SIZE_IN_BYTES, writer.get_position());

            writer.write_reserved_varint32(position, value).unwrap();

            let bytes = writer.freeze();

            assert_eq!(expected, &bytes);

            let mut reader = ProtobufReader::new(&bytes);

            assert_eq!(value, reader.read_varint64().unwrap() as u32);
        };

        run_test(0, &[0x80, 0x80, 0x80, 0x00]);

        run_test(1, &[0x81, 0x80, 0x80, 0x00]);

        run_test(300, &[0xAC, 0x82, 0x80, 0x00]);

        run_test(127, &[0xFF, 0x80, 0x80, 0x00]);

        run_test(128, &[0x80, 0x81, 0x80, 0x00]);

        run_test(16383, &[0xFF, 0xFF, 0x80, 0x00]);

        run_test(16384, &[0x80, 0x80, 0x81, 0x00]);

        run_test(2097151, &[0xFF, 0xFF, 0xFF, 0x00]);

        run_test(2097152, &[0x80, 0x80, 0x80, 0x01]);

        run_test(268435455, &[0xFF, 0xFF, 0xFF, 0x7F]);
    }
}
