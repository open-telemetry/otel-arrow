// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Buf;

use crate::{serializer::*, *};

pub struct ProtobufReader<'a> {
    data: &'a [u8],
}

impl<'a> ProtobufReader<'a> {
    pub fn new(protobuf_data: &'a [u8]) -> ProtobufReader<'a> {
        Self {
            data: protobuf_data,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline(always)]
    pub fn read_tag(&mut self) -> Result<ProtobufTag, SerializerError> {
        let v = self.read_varint64()? as u32;
        let tag = ProtobufTag::new(v)?;
        if matches!(
            tag.wire_type,
            ProtobufWireType::StartGroup | ProtobufWireType::EndGroup
        ) {
            Err(SerializerError::Deprecated("group wire type"))
        } else {
            Ok(tag)
        }
    }

    #[inline(always)]
    pub fn read_field(&mut self, tag: ProtobufTag) -> Result<ProtobufField, SerializerError> {
        Ok(match tag.wire_type {
            ProtobufWireType::Varint => ProtobufField::Varint {
                field_number: tag.field_number,
                value: self.read_varint64()?,
            },
            ProtobufWireType::Fixed64 => ProtobufField::Fixed64 {
                field_number: tag.field_number,
                value: self.read_fixed64()?,
            },
            ProtobufWireType::LengthDelimited => ProtobufField::LengthDelimited {
                field_number: tag.field_number,
                value: self.read_bytes()?.to_vec(),
            },
            ProtobufWireType::Fixed32 => ProtobufField::Fixed32 {
                field_number: tag.field_number,
                value: self.read_fixed32()?,
            },
            _ => panic!("WireType not supported"),
        })
    }

    #[inline(always)]
    pub fn read_message<F>(&mut self, read_message: F) -> Result<bool, SerializerError>
    where
        F: FnOnce(ProtobufReader) -> Result<(), SerializerError>,
    {
        let message = self.read_bytes()?;
        if message.is_empty() {
            Ok(false)
        } else {
            read_message(ProtobufReader::new(message))?;

            Ok(true)
        }
    }

    #[inline(always)]
    pub fn read_bytes(&mut self) -> Result<&[u8], SerializerError> {
        let length = self.read_varint64()? as usize;
        if length == 0 {
            Ok(&[])
        } else {
            if self.data.len() < length {
                return Err(SerializerError::UnexpectedEndOfBuffer);
            }

            unsafe {
                let result = self.data.get_unchecked(0..length);

                self.data = &self.data[length..];

                Ok(result)
            }
        }
    }

    #[inline(always)]
    pub fn read_string(&mut self) -> Result<String, SerializerError> {
        String::from_utf8(self.read_bytes()?.to_vec())
            .map_err(|e| SerializerError::Utf8(e.utf8_error()))
    }

    #[inline(always)]
    fn read_u8(&mut self) -> Result<u8, SerializerError> {
        self.data
            .try_get_u8()
            .map_err(|_| SerializerError::UnexpectedEndOfBuffer)
    }

    #[inline(always)]
    pub fn read_bool(&mut self) -> Result<bool, SerializerError> {
        self.read_u8().map(|r| r != 0)
    }

    #[inline(always)]
    pub fn read_double(&mut self) -> Result<f64, SerializerError> {
        let mut buffer = [0x00; 8];
        self.data
            .try_copy_to_slice(&mut buffer)
            .map_err(|_| SerializerError::UnexpectedEndOfBuffer)?;
        Ok(f64::from_le_bytes(buffer))
    }

    #[inline(always)]
    pub fn read_int64(&mut self) -> Result<i64, SerializerError> {
        Ok(self.read_varint64()? as i64)
    }

    #[inline(always)]
    pub fn read_fixed32(&mut self) -> Result<u32, SerializerError> {
        let mut buffer = [0x00; 4];
        self.data
            .try_copy_to_slice(&mut buffer)
            .map_err(|_| SerializerError::UnexpectedEndOfBuffer)?;
        Ok(u32::from_le_bytes(buffer))
    }

    #[inline(always)]
    pub fn read_fixed64(&mut self) -> Result<u64, SerializerError> {
        let mut buffer = [0x00; 8];
        self.data
            .try_copy_to_slice(&mut buffer)
            .map_err(|_| SerializerError::UnexpectedEndOfBuffer)?;
        Ok(u64::from_le_bytes(buffer))
    }

    pub fn read_varint64(&mut self) -> Result<u64, SerializerError> {
        let mut buffer = [0x00; 10];
        let size = std::cmp::min(self.data.len(), 10);
        if size == 0 {
            return Err(SerializerError::UnexpectedEndOfBuffer);
        }

        unsafe {
            buffer
                .get_unchecked_mut(..size)
                .copy_from_slice(self.data.get_unchecked(..size));

            let mut ptr = buffer.as_ptr();

            let b = *ptr;
            if b & 0x80 == 0 {
                self.data = &self.data[1..];
                return Ok(b as u64);
            }

            let mut value = (b as u64) & 0x7f;
            let mut count = 1;
            let mut shift = 7;
            while count < size {
                ptr = ptr.add(1);

                let b = *ptr;
                value |= ((b & 0x7f) as u64) << shift;
                if b & 0x80 == 0 {
                    self.data = &self.data[count + 1..];
                    return Ok(value);
                }

                count += 1;
                shift += 7;
            }
        }

        Err(SerializerError::UnexpectedEndOfBuffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_varint32() {
        let run_test = |value: &[u8], expected: u64| {
            let mut reader = ProtobufReader::new(value);

            assert_eq!(expected, reader.read_varint64().unwrap());
        };

        run_test(&[0], 0);
        run_test(&[0x80, 0x80, 0x80, 0], 0);
        run_test(&[1], 1);
        run_test(&[0x7F], 127);
        run_test(&[0xFF, 0xFF, 0xFF, 0x7F], 268435455);
        run_test(
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
            u64::MAX,
        );
    }
}
