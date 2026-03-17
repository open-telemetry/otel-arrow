// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP schema definition types. These can be used to describe the schema of
//! any otap payload type. See [crate::schema::payloads::get]

use arrow::array::{Array, ArrayRef, AsArray, RecordBatch};

use crate::schema::error::Error;

/// Leaf Arrow data types used in OTAP schemas.
///
/// This is a closed enum of the primitive/variable-length types that actually
/// appear in the protocol. It converts to [`arrow::datatypes::DataType`] via
/// [`SimpleType::to_arrow`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpleType {
    Boolean,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int32,
    Int64,
    Float64,
    Utf8,
    Binary,
    FixedSizeBinary(i32),
    TimestampNanosecond,
    DurationNanosecond,
}

impl SimpleType {
    /// Convert to the corresponding [`arrow::datatypes::DataType`].
    #[must_use]
    pub fn to_arrow(&self) -> arrow::datatypes::DataType {
        use arrow::datatypes::{DataType as ArrowDT, TimeUnit};
        match self {
            Self::Boolean => ArrowDT::Boolean,
            Self::UInt8 => ArrowDT::UInt8,
            Self::UInt16 => ArrowDT::UInt16,
            Self::UInt32 => ArrowDT::UInt32,
            Self::UInt64 => ArrowDT::UInt64,
            Self::Int32 => ArrowDT::Int32,
            Self::Int64 => ArrowDT::Int64,
            Self::Float64 => ArrowDT::Float64,
            Self::Utf8 => ArrowDT::Utf8,
            Self::Binary => ArrowDT::Binary,
            Self::FixedSizeBinary(n) => ArrowDT::FixedSizeBinary(*n),
            Self::TimestampNanosecond => ArrowDT::Timestamp(TimeUnit::Nanosecond, None),
            Self::DurationNanosecond => ArrowDT::Duration(TimeUnit::Nanosecond),
        }
    }

    /// Returns true if `arrow_dt` is an exact match for this simple type.
    #[must_use]
    fn matches(&self, arrow_dt: &arrow::datatypes::DataType) -> bool {
        self.to_arrow() == *arrow_dt
    }
}

/// Minimum dictionary key size constraint from the OTAP spec.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DictKeySize {
    U8,
    U16,
}

/// OTAP-level data type.
#[derive(Debug, Clone, Copy)]
pub enum DataType {
    /// Simple
    Simple(SimpleType),
    /// A column which MAY be dictionary encoded along with the minimum size.
    /// Note that the max key size in OTAP is always U16.
    Dictionary {
        min_key_size: DictKeySize,
        value_type: SimpleType,
    },
    /// Struct
    Struct(&'static Schema),
    /// List whose items have the given OTAP data type.
    List(&'static DataType),
}

impl DataType {
    /// Validate that the given Arrow DataType conforms to this OTAP type
    /// definition, checking struct sub-fields and list item types.
    #[must_use]
    pub fn matches(&self, array: &ArrayRef) -> bool {
        use arrow::datatypes::DataType as ArrowDT;
        let arrow_dt = array.data_type();
        match self {
            DataType::Simple(s) => s.matches(arrow_dt),
            DataType::Dictionary {
                min_key_size,
                value_type,
            } => {
                if value_type.matches(arrow_dt) {
                    return true;
                }

                if let ArrowDT::Dictionary(key_type, val_type) = arrow_dt {
                    if !value_type.matches(val_type.as_ref()) {
                        return false;
                    }

                    return match key_type.as_ref() {
                        ArrowDT::UInt8 => *min_key_size == DictKeySize::U8,
                        ArrowDT::UInt16 => true,
                        _ => false,
                    };
                }

                false
            }
            DataType::Struct(sub_schema) => {
                let ArrowDT::Struct(_) = arrow_dt else {
                    return false;
                };
                // safety: we just checked that this is a struct
                let struct_array = array.as_struct();

                for (field, child_array) in struct_array.fields().iter().zip(struct_array.columns())
                {
                    let name = field.name().as_str();
                    let Some(field_def) = sub_schema.get(name) else {
                        return false;
                    };

                    if !field_def.data_type.matches(child_array) {
                        return false;
                    }

                    if field_def.required && child_array.null_count() > 0 {
                        return false;
                    }
                }

                true
            }
            DataType::List(inner_dt) => {
                let ArrowDT::List(_) = arrow_dt else {
                    return false;
                };
                // safety: We verified this is a list type.
                // note: i32 is not the type of the list, but the type of
                // offsets into the list.
                let list_array = array.as_list::<i32>();
                inner_dt.matches(list_array.values())
            }
        }
    }

    /// If this is a `Dictionary` variant, returns the minimum key size.
    /// Otherwise returns `None`.
    #[must_use]
    pub fn min_dict_key_size(&self) -> Option<DictKeySize> {
        match self {
            Self::Dictionary { min_key_size, .. } => Some(*min_key_size),
            _ => None,
        }
    }

    /// If this is a `Struct` variant, returns the nested schema.
    #[must_use]
    pub fn as_struct_schema(&self) -> Option<&'static Schema> {
        match self {
            Self::Struct(s) => Some(s),
            _ => None,
        }
    }

    /// Returns true for `Struct` or `List` variants.
    #[must_use]
    pub fn is_complex(&self) -> bool {
        matches!(self, Self::Struct(_) | Self::List(_))
    }
}

/// A single column/field in an OTAP schema definition.
#[derive(Clone, Copy)]
pub struct Field {
    /// Column name (matches the Arrow field name).
    pub name: &'static str,
    /// Expected OTAP data type.
    pub data_type: DataType,
    /// Whether this column must be present and non-null.
    pub required: bool,
}

impl std::fmt::Debug for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("data_type", &self.data_type)
            .field("required", &self.required)
            .finish()
    }
}

/// OTAP schema definition for a payload type (or a nested struct).
#[derive(Debug, Clone, Copy)]
pub struct Schema {
    pub(crate) fields: &'static [Field],
    pub(crate) idx: fn(&str) -> Option<usize>,
}

impl Schema {
    /// An empty schema with no fields.
    ///
    /// Any columns in a RecordBatch will be rejected as extraneous when
    /// validated against this schema.
    pub const EMPTY: Schema = Schema {
        fields: &[],
        idx: |_| None,
    };

    /// Returns all fields in this schema.
    #[must_use]
    pub fn fields(&self) -> &'static [Field] {
        self.fields
    }

    /// Look up a field by name. Returns `None` if the field is not present.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&'static Field> {
        let i = (self.idx)(name)?;
        Some(&self.fields[i])
    }

    /// Returns the names of all required fields.
    pub fn required_field_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.fields.iter().filter(|f| f.required).map(|f| f.name)
    }

    /// Validate that a [`RecordBatch`] conforms to this schema definition.
    ///
    /// Checks for:
    /// - Extraneous columns (present in batch but not in definition)
    /// - Missing required columns
    /// - Type mismatches (including dictionary key size constraints)
    /// - Null values in required columns
    /// - Struct/List sub-field validation (recursive)
    ///
    /// The hot path (all columns valid) performs zero heap allocations.
    pub fn check_match(&self, record_batch: &RecordBatch) -> crate::schema::error::Result<()> {
        let rb_schema = record_batch.schema();
        let mut found_required: usize = 0;
        let required_count = self.fields.iter().filter(|f| f.required).count();

        for (col_idx, arrow_field) in rb_schema.fields().iter().enumerate() {
            let name = arrow_field.name().as_str();
            let Some(field_def) = self.get(name) else {
                return Err(Error::ExtraneousField {
                    name: name.to_string(),
                });
            };

            let column = record_batch.column(col_idx);
            if !field_def.data_type.matches(column) {
                return Err(Error::FieldTypeMismatch {
                    name: name.to_string(),
                    expected: field_def.data_type,
                    actual: arrow_field.data_type().clone(),
                });
            }

            if field_def.required {
                found_required += 1;
                if column.null_count() > 0 {
                    return Err(Error::MissingRequiredFields {
                        names: vec![name.to_string()],
                    });
                }
            }
        }

        if found_required >= required_count {
            return Ok(());
        }

        // Slow path: find which required columns are missing.
        let fields = rb_schema.fields();
        let missing: Vec<String> = self
            .required_field_names()
            .filter(|&req| !fields.iter().any(|f| f.name().as_str() == req))
            .map(|n| n.to_string())
            .collect();

        Err(Error::MissingRequiredFields { names: missing })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{
        Float64Array, Int32Array, ListArray, RecordBatch, StringArray, StructArray, UInt64Array,
        new_empty_array,
    };
    use arrow::datatypes::{
        DataType as ArrowDT, Field as ArrowField, Fields, Schema as ArrowSchema, TimeUnit,
    };
    use std::sync::Arc;

    /// Creates a zero-length array of the given Arrow data type.
    fn empty_array(dt: &ArrowDT) -> ArrayRef {
        new_empty_array(dt)
    }

    #[test]
    fn simple_type_to_arrow_roundtrip() {
        let cases: &[(SimpleType, ArrowDT)] = &[
            (SimpleType::Boolean, ArrowDT::Boolean),
            (SimpleType::UInt8, ArrowDT::UInt8),
            (SimpleType::UInt16, ArrowDT::UInt16),
            (SimpleType::UInt32, ArrowDT::UInt32),
            (SimpleType::UInt64, ArrowDT::UInt64),
            (SimpleType::Int32, ArrowDT::Int32),
            (SimpleType::Int64, ArrowDT::Int64),
            (SimpleType::Float64, ArrowDT::Float64),
            (SimpleType::Utf8, ArrowDT::Utf8),
            (SimpleType::Binary, ArrowDT::Binary),
            (
                SimpleType::FixedSizeBinary(16),
                ArrowDT::FixedSizeBinary(16),
            ),
            (
                SimpleType::TimestampNanosecond,
                ArrowDT::Timestamp(TimeUnit::Nanosecond, None),
            ),
            (
                SimpleType::DurationNanosecond,
                ArrowDT::Duration(TimeUnit::Nanosecond),
            ),
        ];
        for (st, expected) in cases {
            assert_eq!(st.to_arrow(), *expected, "SimpleType::{st:?}");
            assert!(st.matches(expected), "SimpleType::{st:?} should match");
        }
    }

    #[test]
    fn data_type_simple_matches() {
        let dt = DataType::Simple(SimpleType::Utf8);
        assert!(dt.matches(&empty_array(&ArrowDT::Utf8)));
        assert!(!dt.matches(&empty_array(&ArrowDT::Binary)));
    }

    #[test]
    fn data_type_dict_accepts_native() {
        let dt = DataType::Dictionary {
            min_key_size: DictKeySize::U8,
            value_type: SimpleType::Utf8,
        };
        assert!(dt.matches(&empty_array(&ArrowDT::Utf8)));
    }

    #[test]
    fn u8_dict_accepts_valid_keys() {
        let dt = DataType::Dictionary {
            min_key_size: DictKeySize::U8,
            value_type: SimpleType::Utf8,
        };
        let u8_dict = ArrowDT::Dictionary(Box::new(ArrowDT::UInt8), Box::new(ArrowDT::Utf8));
        let u16_dict = ArrowDT::Dictionary(Box::new(ArrowDT::UInt16), Box::new(ArrowDT::Utf8));
        let u32_dict = ArrowDT::Dictionary(Box::new(ArrowDT::UInt32), Box::new(ArrowDT::Utf8));
        // U8 allowed
        assert!(dt.matches(&empty_array(&u8_dict)));
        // U16 allowed
        assert!(dt.matches(&empty_array(&u16_dict)));
        // U32 not allowed
        assert!(!dt.matches(&empty_array(&u32_dict)));
    }

    #[test]
    fn u16_dict_accepts_valid_keys() {
        let dt = DataType::Dictionary {
            min_key_size: DictKeySize::U16,
            value_type: SimpleType::Utf8,
        };
        let u8_dict = ArrowDT::Dictionary(Box::new(ArrowDT::UInt8), Box::new(ArrowDT::Utf8));
        let u16_dict = ArrowDT::Dictionary(Box::new(ArrowDT::UInt16), Box::new(ArrowDT::Utf8));
        let u32_dict = ArrowDT::Dictionary(Box::new(ArrowDT::UInt32), Box::new(ArrowDT::Utf8));
        // U8 not allowed
        assert!(!dt.matches(&empty_array(&u8_dict)));
        // U16 allowed
        assert!(dt.matches(&empty_array(&u16_dict)));
        // U32 not allowed
        assert!(!dt.matches(&empty_array(&u32_dict)));
    }

    #[test]
    fn data_type_struct_accepts_matching() {
        static SUB: Schema = Schema {
            fields: &[
                Field {
                    name: "x",
                    data_type: DataType::Simple(SimpleType::Int32),
                    required: false,
                },
                Field {
                    name: "y",
                    data_type: DataType::Simple(SimpleType::Utf8),
                    required: false,
                },
            ],
            idx: |n| match n {
                "x" => Some(0),
                "y" => Some(1),
                _ => None,
            },
        };
        let dt = DataType::Struct(&SUB);
        let struct_array: ArrayRef = Arc::new(StructArray::new(
            Fields::from(vec![
                ArrowField::new("x", ArrowDT::Int32, false),
                ArrowField::new("y", ArrowDT::Utf8, true),
            ]),
            vec![
                Arc::new(Int32Array::from(Vec::<i32>::new())),
                Arc::new(StringArray::from(Vec::<&str>::new())),
            ],
            None,
        ));
        assert!(dt.matches(&struct_array));
    }

    #[test]
    fn data_type_struct_accepts_subset() {
        static SUB: Schema = Schema {
            fields: &[
                Field {
                    name: "x",
                    data_type: DataType::Simple(SimpleType::Int32),
                    required: false,
                },
                Field {
                    name: "y",
                    data_type: DataType::Simple(SimpleType::Utf8),
                    required: false,
                },
            ],
            idx: |n| match n {
                "x" => Some(0),
                "y" => Some(1),
                _ => None,
            },
        };
        let dt = DataType::Struct(&SUB);
        let struct_array: ArrayRef = Arc::new(StructArray::new(
            Fields::from(vec![ArrowField::new("x", ArrowDT::Int32, false)]),
            vec![Arc::new(Int32Array::from(Vec::<i32>::new()))],
            None,
        ));
        assert!(dt.matches(&struct_array));
    }

    #[test]
    fn data_type_struct_rejects_extraneous() {
        static SUB: Schema = Schema {
            fields: &[],
            idx: |_| None,
        };
        let struct_array: ArrayRef = Arc::new(
            StructArray::try_new(
                Fields::from(vec![ArrowField::new("z", ArrowDT::Int32, false)]),
                vec![Arc::new(Int32Array::from(Vec::<i32>::new()))],
                None,
            )
            .unwrap(),
        );
        assert!(!DataType::Struct(&SUB).matches(&struct_array));
    }

    #[test]
    fn data_type_struct_rejects_wrong_type() {
        static SUB: Schema = Schema {
            fields: &[Field {
                name: "x",
                data_type: DataType::Simple(SimpleType::Int32),
                required: false,
            }],
            idx: |n| match n {
                "x" => Some(0),
                _ => None,
            },
        };
        let struct_array: ArrayRef = Arc::new(
            StructArray::try_new(
                Fields::from(vec![ArrowField::new("x", ArrowDT::Utf8, false)]),
                vec![Arc::new(StringArray::from(Vec::<&str>::new()))],
                None,
            )
            .unwrap(),
        );
        assert!(!DataType::Struct(&SUB).matches(&struct_array));
    }

    #[test]
    fn data_type_struct_rejects_non_struct() {
        static SUB: Schema = Schema {
            fields: &[],
            idx: |_| None,
        };
        assert!(!DataType::Struct(&SUB).matches(&empty_array(&ArrowDT::Utf8)));
    }

    #[test]
    fn data_type_struct_rejects_null_in_required_field() {
        static SUB: Schema = Schema {
            fields: &[
                Field {
                    name: "x",
                    data_type: DataType::Simple(SimpleType::Int32),
                    required: true,
                },
                Field {
                    name: "y",
                    data_type: DataType::Simple(SimpleType::Utf8),
                    required: false,
                },
            ],
            idx: |n| match n {
                "x" => Some(0),
                "y" => Some(1),
                _ => None,
            },
        };
        let dt = DataType::Struct(&SUB);

        // "x" is required but contains a null — should be rejected.
        let struct_array: ArrayRef = Arc::new(StructArray::new(
            Fields::from(vec![
                ArrowField::new("x", ArrowDT::Int32, true),
                ArrowField::new("y", ArrowDT::Utf8, true),
            ]),
            vec![
                Arc::new(Int32Array::from(vec![Some(1), None])),
                Arc::new(StringArray::from(vec![Some("a"), Some("b")])),
            ],
            None,
        ));
        assert!(!dt.matches(&struct_array));
    }

    #[test]
    fn data_type_struct_accepts_null_in_optional_field() {
        static SUB: Schema = Schema {
            fields: &[
                Field {
                    name: "x",
                    data_type: DataType::Simple(SimpleType::Int32),
                    required: false,
                },
                Field {
                    name: "y",
                    data_type: DataType::Simple(SimpleType::Utf8),
                    required: false,
                },
            ],
            idx: |n| match n {
                "x" => Some(0),
                "y" => Some(1),
                _ => None,
            },
        };
        let dt = DataType::Struct(&SUB);

        // "x" is optional and contains a null — should be accepted.
        let struct_array: ArrayRef = Arc::new(StructArray::new(
            Fields::from(vec![
                ArrowField::new("x", ArrowDT::Int32, true),
                ArrowField::new("y", ArrowDT::Utf8, true),
            ]),
            vec![
                Arc::new(Int32Array::from(vec![Some(1), None])),
                Arc::new(StringArray::from(vec![Some("a"), Some("b")])),
            ],
            None,
        ));
        assert!(dt.matches(&struct_array));
    }

    #[test]
    fn data_type_list_struct() {
        static ITEM_SCHEMA: Schema = Schema {
            fields: &[Field {
                name: "q",
                data_type: DataType::Simple(SimpleType::Float64),
                required: false,
            }],
            idx: |n| match n {
                "q" => Some(0),
                _ => None,
            },
        };
        static INNER: DataType = DataType::Struct(&ITEM_SCHEMA);
        let dt = DataType::List(&INNER);

        // Matching sub-fields
        let ok_fields = Fields::from(vec![ArrowField::new("q", ArrowDT::Float64, true)]);
        let ok_struct_values: ArrayRef = Arc::new(StructArray::new(
            ok_fields.clone(),
            vec![Arc::new(Float64Array::from(Vec::<f64>::new()))],
            None,
        ));
        let ok_list: ArrayRef = Arc::new(ListArray::new(
            Arc::new(ArrowField::new("item", ArrowDT::Struct(ok_fields), true)),
            arrow::buffer::OffsetBuffer::new(vec![0i32].into()),
            ok_struct_values,
            None,
        ));
        assert!(dt.matches(&ok_list));

        // Empty struct items - ok (sub-fields optional)
        let empty_struct_values: ArrayRef = Arc::new(StructArray::new_empty_fields(0, None));
        let empty_list: ArrayRef = Arc::new(ListArray::new(
            Arc::new(ArrowField::new(
                "item",
                ArrowDT::Struct(Fields::empty()),
                true,
            )),
            arrow::buffer::OffsetBuffer::new(vec![0i32].into()),
            empty_struct_values,
            None,
        ));
        assert!(dt.matches(&empty_list));

        // Extraneous sub-field
        let extra_fields = Fields::from(vec![ArrowField::new("bogus", ArrowDT::Int32, true)]);
        let extra_struct_values: ArrayRef = Arc::new(StructArray::new(
            extra_fields.clone(),
            vec![Arc::new(Int32Array::from(Vec::<i32>::new()))],
            None,
        ));
        let bad_list: ArrayRef = Arc::new(ListArray::new(
            Arc::new(ArrowField::new("item", ArrowDT::Struct(extra_fields), true)),
            arrow::buffer::OffsetBuffer::new(vec![0i32].into()),
            extra_struct_values,
            None,
        ));
        assert!(!dt.matches(&bad_list));

        // Non-struct item
        let wrong_values: ArrayRef = Arc::new(UInt64Array::from(Vec::<u64>::new()));
        let wrong_list: ArrayRef = Arc::new(ListArray::new(
            Arc::new(ArrowField::new("item", ArrowDT::UInt64, true)),
            arrow::buffer::OffsetBuffer::new(vec![0i32].into()),
            wrong_values,
            None,
        ));
        assert!(!dt.matches(&wrong_list));
    }

    static TEST_SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: "a",
                data_type: DataType::Simple(SimpleType::Int32),
                required: true,
            },
            Field {
                name: "b",
                data_type: DataType::Simple(SimpleType::Utf8),
                required: false,
            },
        ],
        idx: |n| match n {
            "a" => Some(0),
            "b" => Some(1),
            _ => None,
        },
    };

    #[test]
    fn schema_get_lookup() {
        assert_eq!(TEST_SCHEMA.get("a").unwrap().name, "a");
        assert!(TEST_SCHEMA.get("a").unwrap().required);
        assert_eq!(TEST_SCHEMA.get("b").unwrap().name, "b");
        assert!(!TEST_SCHEMA.get("b").unwrap().required);
        assert!(TEST_SCHEMA.get("c").is_none());
    }

    #[test]
    fn schema_check_match_missing_required() {
        // Only "b" (optional), missing "a" (required).
        let schema = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "b",
            ArrowDT::Utf8,
            true,
        )]));
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(StringArray::from(vec!["x"]))]).unwrap();
        match TEST_SCHEMA.check_match(&batch) {
            Err(Error::MissingRequiredFields { names, .. }) => {
                assert!(names.contains(&"a".to_string()));
            }
            other => panic!("expected MissingRequiredFields, got {other:?}"),
        }
    }

    #[test]
    fn schema_check_match_type_mismatch() {
        let schema = Arc::new(ArrowSchema::new(vec![ArrowField::new(
            "a",
            ArrowDT::Utf8,
            false,
        )]));
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(StringArray::from(vec!["x"]))]).unwrap();
        match TEST_SCHEMA.check_match(&batch) {
            Err(Error::FieldTypeMismatch { .. }) => {}
            other => panic!("expected FieldTypeMismatch, got {other:?}"),
        }
    }
}
