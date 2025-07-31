pub(crate) mod otlp_reader;
pub(crate) mod otlp_writer;
pub(crate) mod protobuf_reader;
pub(crate) mod protobuf_writer;
pub(crate) mod serializer_error;

const TAG_TYPE_BITS: u32 = 3;
const TAG_TYPE_MASK: u32 = (1u32 << TAG_TYPE_BITS as usize) - 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ProtobufTag {
    field_number: u32,
    wire_type: ProtobufWireType,
}

impl ProtobufTag {
    /// Extract wire type and field number from integer tag
    pub fn new(value: u32) -> Result<ProtobufTag, serializer_error::SerializerError> {
        let wire_type = ProtobufWireType::new(value & TAG_TYPE_MASK);
        if wire_type.is_none() {
            return Err(serializer_error::SerializerError::UnknownWireType(
                value & TAG_TYPE_MASK,
            ));
        }
        let field_number = value >> TAG_TYPE_BITS;
        if field_number == 0 {
            return Err(serializer_error::SerializerError::InvalidFieldNumber(
                field_number,
            ));
        }
        Ok(ProtobufTag {
            field_number,
            wire_type: wire_type.unwrap(),
        })
    }
}

/// All supported "wire types" are listed in this enum.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ProtobufWireType {
    /// Variable-length integer
    Varint = 0,
    /// 64-bit field (e. g. `fixed64` or `double`)
    Fixed64 = 1,
    /// Length-delimited field
    LengthDelimited = 2,
    /// Groups are not supported in rust-protobuf
    StartGroup = 3,
    /// Groups are not supported in rust-protobuf
    EndGroup = 4,
    /// 32-bit field (e. g. `fixed32` or `float`)
    Fixed32 = 5,
}

impl ProtobufWireType {
    fn new(n: u32) -> Option<ProtobufWireType> {
        match n {
            0 => Some(ProtobufWireType::Varint),
            1 => Some(ProtobufWireType::Fixed64),
            2 => Some(ProtobufWireType::LengthDelimited),
            3 => Some(ProtobufWireType::StartGroup),
            4 => Some(ProtobufWireType::EndGroup),
            5 => Some(ProtobufWireType::Fixed32),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProtobufField {
    Varint { field_number: u32, value: u64 },
    Fixed32 { field_number: u32, value: u32 },
    Fixed64 { field_number: u32, value: u64 },
    LengthDelimited { field_number: u32, value: Vec<u8> },
}
