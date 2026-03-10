// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains utilities for working with OTAP schemas

use arrow::datatypes::{DataType, Field, Schema, UnionMode};

/// Builds unique schema id for a schema.
/// Fields are sorted by name before creating the id (done at each nested level).
//
// The internal representation tries to avoid reallocating the string and the
// internal sorted array of column IDs
pub struct SchemaIdBuilder {
    // This will be used as a stack, where each stack frame has the sorted index of
    // fields at some nested level of the schema
    out: String,
}

impl SchemaIdBuilder {
    /// create a new instance of `SchemaIdBuilder`
    #[must_use]
    pub fn new() -> Self {
        Self {
            out: String::with_capacity(128),
        }
    }

    /// build the unique ID for the passed schema
    pub fn build_id(&mut self, schema: &Schema) -> &str {
        self.out.clear();
        self.write_schema(schema);
        &self.out
    }

    fn write_schema(&mut self, schema: &Schema) {
        for (i, field) in schema.fields.iter().enumerate() {
            if i != 0 {
                self.out.push(',')
            }
            self.write_field(field);
        }
    }

    fn write_field(&mut self, field: &Field) {
        self.out.push_str(field.name());
        self.out.push(':');
        self.write_data_type(field.data_type());
    }

    fn write_data_type(&mut self, dt: &DataType) {
        use DataType::*;
        match dt {
            Boolean => self.out.push_str("Bol"),
            UInt8 => self.out.push_str("U8"),
            UInt16 => self.out.push_str("U16"),
            UInt32 => self.out.push_str("U32"),
            UInt64 => self.out.push_str("U64"),
            Int8 => self.out.push_str("I8"),
            Int16 => self.out.push_str("I16"),
            Int32 => self.out.push_str("I32"),
            Int64 => self.out.push_str("I64"),
            Float32 => self.out.push_str("F32"),
            Float64 => self.out.push_str("F64"),
            Utf8 => self.out.push_str("Str"),
            Binary => self.out.push_str("Bin"),
            FixedSizeBinary(n) => {
                use std::fmt::Write;
                write!(&mut self.out, "FSB<{n}>").expect("writing to String should never fail");
            }
            Timestamp(_, _) => self.out.push_str("Tns"),
            Duration(_) => self.out.push_str("Dur"),

            List(field) => {
                self.out.push('[');
                self.write_data_type(field.data_type());
                self.out.push(']');
            }

            Dictionary(index, value) => {
                self.out.push_str("Dic<");
                self.write_data_type(index);
                self.out.push(',');
                self.write_data_type(value);
                self.out.push('>');
            }

            Struct(fields) => {
                self.out.push('{');
                for (i, field) in fields.iter().enumerate() {
                    if i != 0 {
                        self.out.push(',');
                    }
                    self.write_field(field);
                }
                self.out.push('}');
            }

            Map(field, _) => {
                self.out.push_str("Map<");
                if let Struct(fields) = field.data_type() {
                    if fields.len() == 2 {
                        // Assume the first field is key type and the second field is value type for simplicity
                        // arrow-go has similar logic
                        // See https://github.com/apache/arrow-go/blob/3ae84281674622d33b4617c878e099d13d4a1113/arrow/datatype_nested.go#L593-L596
                        self.write_data_type(fields[0].data_type());
                        self.out.push(',');
                        self.write_data_type(fields[1].data_type());
                    }
                }
                self.out.push('>');
            }

            Union(union_fields, mode) => {
                let tag = match mode {
                    UnionMode::Dense => "DU",
                    UnionMode::Sparse => "SU",
                };
                self.out.push_str(tag);
                self.out.push('{');
                for (i, field) in union_fields.iter().enumerate() {
                    if i > 0 {
                        self.out.push(',');
                    }
                    self.write_field(field.1);
                }
                self.out.push('}');
            }
            _ => panic!("Unsupported datatype: {dt:?}"),
        }
    }
}

impl Default for SchemaIdBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use arrow::datatypes::{DataType, Field, Schema, TimeUnit, UnionFields, UnionMode};
    use std::sync::Arc;

    use crate::otap::schema::SchemaIdBuilder;

    #[test]
    pub fn test_all_field_types() {
        let schema = Schema::new(vec![
            Field::new("boolean", DataType::Boolean, true),
            Field::new("uint8", DataType::UInt8, true),
            Field::new("uint16", DataType::UInt16, true),
            Field::new("uint32", DataType::UInt32, true),
            Field::new("uint64", DataType::UInt64, true),
            Field::new("int8", DataType::Int8, true),
            Field::new("int16", DataType::Int16, true),
            Field::new("int32", DataType::Int32, true),
            Field::new("int64", DataType::Int64, true),
            Field::new("float32", DataType::Float32, true),
            Field::new("float64", DataType::Float64, true),
            Field::new("string", DataType::Utf8, true),
            Field::new("binary", DataType::Binary, true),
            Field::new("fsb4", DataType::FixedSizeBinary(4), true),
            Field::new("ts", DataType::Timestamp(TimeUnit::Nanosecond, None), true),
            Field::new("duration", DataType::Duration(TimeUnit::Nanosecond), true),
            Field::new(
                "list",
                DataType::List(Arc::new(Field::new("item", DataType::UInt8, true))),
                true,
            ),
            Field::new(
                "dict",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(
                "map",
                DataType::Map(
                    Arc::new(Field::new(
                        "entries",
                        DataType::Struct(
                            vec![
                                Field::new("key", DataType::Utf8, false),
                                Field::new("value", DataType::Utf8, true),
                            ]
                            .into(),
                        ),
                        true,
                    )),
                    true,
                ),
                true,
            ),
            Field::new(
                "map_invalid",
                DataType::Map(
                    Arc::new(Field::new(
                        "entries",
                        DataType::Struct(
                            vec![
                                Field::new("key", DataType::Utf8, false),
                                Field::new("value", DataType::Boolean, true),
                                Field::new("uint8", DataType::UInt8, true),
                            ]
                            .into(),
                        ),
                        true,
                    )),
                    true,
                ),
                true,
            ),
            Field::new(
                "map_nested",
                DataType::Map(
                    Arc::new(Field::new(
                        "entries",
                        DataType::Struct(
                            vec![
                                Field::new("key", DataType::Utf8, false),
                                Field::new(
                                    "value",
                                    DataType::List(Arc::new(Field::new(
                                        "item",
                                        DataType::UInt16,
                                        true,
                                    ))),
                                    true,
                                ),
                            ]
                            .into(),
                        ),
                        true,
                    )),
                    true,
                ),
                true,
            ),
            Field::new(
                "struct",
                DataType::Struct(
                    vec![
                        Field::new("s.b", DataType::UInt16, true),
                        Field::new("s.a", DataType::UInt8, true),
                        Field::new("s.c", DataType::UInt32, true),
                    ]
                    .into(),
                ),
                true,
            ),
            Field::new(
                "dense_union",
                DataType::Union(
                    UnionFields::from_iter([
                        (1i8, Arc::new(Field::new("du.b", DataType::Int8, true))),
                        (2i8, Arc::new(Field::new("du.b", DataType::Int8, true))),
                        (3i8, Arc::new(Field::new("du.a", DataType::Int8, true))),
                    ]),
                    UnionMode::Dense,
                ),
                true,
            ),
            Field::new(
                "sparse_union",
                DataType::Union(
                    UnionFields::from_iter([
                        (1i8, Arc::new(Field::new("su.b", DataType::Int8, true))),
                        (3i8, Arc::new(Field::new("su.a", DataType::Int8, true))),
                        (2i8, Arc::new(Field::new("su.b", DataType::Int8, true))),
                    ]),
                    UnionMode::Sparse,
                ),
                true,
            ),
        ]);

        let mut builder = SchemaIdBuilder::new();
        let result = builder.build_id(&schema);

        let expected = vec![
            "boolean:Bol",
            "uint8:U8",
            "uint16:U16",
            "uint32:U32",
            "uint64:U64",
            "int8:I8",
            "int16:I16",
            "int32:I32",
            "int64:I64",
            "float32:F32",
            "float64:F64",
            "string:Str",
            "binary:Bin",
            "fsb4:FSB<4>",
            "ts:Tns",
            "duration:Dur",
            "list:[U8]",
            "dict:Dic<U8,Str>",
            "map:Map<Str,Str>",
            "map_invalid:Map<>",
            "map_nested:Map<Str,[U16]>",
            "struct:{s.b:U16,s.a:U8,s.c:U32}",
            "dense_union:DU{du.b:I8,du.b:I8,du.a:I8}",
            "sparse_union:SU{su.b:I8,su.a:I8,su.b:I8}",
        ]
        .join(",");

        assert_eq!(expected.as_str(), result)
    }
}
