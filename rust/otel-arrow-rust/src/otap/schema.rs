// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains utilities for working with OTAP schemas

use arrow::datatypes::{DataType, Field, Schema};

/// Builds unique schema id for a schema.
/// Fields are sorted by name before creating the id (done at each nested level).
//
// The internal representation tries to avoid reallocating the string and the
// internal sorted array of column IDs
pub struct SchemaIdBuilder {
    sort_buf: Vec<usize>,
    sort_buf_idx: usize,
    out: String,
}

impl SchemaIdBuilder {
    /// create a new instance of `SchemaIdBuilder`
    pub fn new() -> Self {
        Self {
            sort_buf: Vec::with_capacity(32), // re-used buffer
            sort_buf_idx: 0,
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
        self.sort_buf.clear();

        schema
            .fields
            .iter()
            .enumerate()
            .for_each(|(i, _)| self.sort_buf.push(i));
        let field_ids = &mut self.sort_buf[0..schema.fields.len()];
        field_ids.sort_by(|a, b| schema.field(*a).name().cmp(schema.field(*b).name()));

        for i in 0..schema.fields.len() {
            let field_idx = self.sort_buf[i];
            let field = schema.field(field_idx);
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
                write!(&mut self.out, "FSB<{}>", n).unwrap();
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

            // Struct(fields) => {
            //     self.out.push('{');
            //     self.sort_buf.clear();
            //     self.sort_buf.extend(fields.iter().map(|f| unsafe { &*(f as *const _) }));
            //     self.sort_buf.sort_by(|a, b| a.name().cmp(b.name()));

            //     for (i, f) in self.sort_buf.iter().enumerate() {
            //         if i > 0 {
            //             self.out.push(',');
            //         }
            //         self.write_field(f);
            //     }
            //     self.out.push('}');
            // }
            Map(field, _) => {
                self.out.push_str("Map<");
                self.write_data_type(field.data_type());
                self.out.push(',');
                self.write_data_type(field.data_type());
                self.out.push('>');
            }

            // Union(fields, _, mode) => {
            //     let tag = match mode {
            //         arrow_schema::UnionMode::Dense => "DU",
            //         arrow_schema::UnionMode::Sparse => "SU",
            //     };
            //     self.out.push_str(tag);
            //     self.out.push('{');
            //     self.sort_buf.clear();
            //     self.sort_buf.extend(fields.iter().map(|f| unsafe { &*(f as *const _) }));
            //     self.sort_buf.sort_by(|a, b| a.name().cmp(b.name()));
            //     for (i, f) in self.sort_buf.iter().enumerate() {
            //         if i > 0 {
            //             self.out.push(',');
            //         }
            //         self.write_field(f);
            //     }
            //     self.out.push('}');
            // }
            _ => panic!("Unsupported datatype: {dt:?}"),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn test_all_field_types() {
        let schema = Schema::new(vec![]);
    }
}
