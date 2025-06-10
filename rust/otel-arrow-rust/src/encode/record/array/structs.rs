// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! this module contains the implementation of the adaptive array builder for struct fields.
//! the `AdaptiveStructBuilder` is a container of child adaptive builders. Like adaptive array
//! builders for primitive types, the resulting array that this builder produces is wrapped in an
//! `Option` where the `None` variant means that all the fields for all columns in this struct
//! were either null.

use std::{any::Any, sync::Arc};

use arrow::{
    array::{Array, ArrayRef, StructArray},
    datatypes::{DataType, Field, FieldRef, Fields, UInt8Type, UInt16Type},
    error::ArrowError,
};
use paste::paste;

use crate::encode::record::array::{
    AdaptiveArrayBuilder, ArrayAppend, ArrayBuilder, ArrayOptions, CheckedArrayAppend,
    boolean::AdaptiveBooleanArrayBuilder, dictionary::DictionaryBuilder,
};

/// Data about some field contained in this struct
struct FieldData {
    /// name of the field
    name: String,
}

impl FieldData {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

trait StructArrayBuilderHelper {
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn nullable(&self) -> bool;

    fn finish(&mut self) -> Option<ArrayRef>;
}

impl<TArgs, TN, TD8, TD16> StructArrayBuilderHelper for AdaptiveArrayBuilder<TArgs, TN, TD8, TD16>
where
    TArgs: 'static,
    TN: ArrayBuilder + 'static,
    TD8: DictionaryBuilder<UInt8Type> + 'static,
    TD16: DictionaryBuilder<UInt16Type> + 'static,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn nullable(&self) -> bool {
        self.nullable
    }

    fn finish(&mut self) -> Option<ArrayRef> {
        self.finish()
    }
}

impl StructArrayBuilderHelper for AdaptiveBooleanArrayBuilder {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn nullable(&self) -> bool {
        self.nullable
    }

    fn finish(&mut self) -> Option<ArrayRef> {
        AdaptiveBooleanArrayBuilder::finish(self)
    }
}

/// Adaptive array builder for columns of type struct.
struct AdaptiveStructBuilder {
    fields: Vec<(FieldData, Box<dyn StructArrayBuilderHelper>)>,
}

impl AdaptiveStructBuilder {
    pub fn new(fields: Vec<(FieldData, Box<dyn StructArrayBuilderHelper>)>) -> Self {
        Self { fields }
    }

    /// Get the builder for some field index.
    ///
    /// If the field for at index doesn't exist or the generic type is wrong, this returns `None`
    pub fn field_builder<T>(&mut self, i: usize) -> Option<&mut T>
    where
        T: StructArrayBuilderHelper + ArrayAppend + 'static,
    {
        self.fields
            .get_mut(i)
            .and_then(|(_, builder)| builder.as_any_mut().downcast_mut())
    }

    /// Get the builder for some field index.
    ///
    /// If the field for at index doesn't exist or the generic type is wrong, this returns `None`
    ///
    // Note: this method  is somewhat similar to field_builder just with a different trait bound
    // on the generic. The hope with the trait bounds here is that is that it will avoid
    // accidentally  calling this method with the wrong generic type.
    fn checked_field_builder<T>(&mut self, i: usize) -> Option<&mut T>
    where
        T: StructArrayBuilderHelper + CheckedArrayAppend + 'static,
    {
        self.fields
            .get_mut(i)
            .and_then(|(_, builder)| builder.as_any_mut().downcast_mut())
    }

    fn finish(&mut self) -> Option<Result<ArrayRef, ArrowError>> {
        let mut fields: Vec<Field> = vec![];
        let mut arrays: Vec<ArrayRef> = vec![];
        for i in 0..self.fields.len() {
            let (field_data, builder) = &mut self.fields[i];
            let nullable = builder.nullable();
            if let Some(array) = builder.finish() {
                fields.push(Field::new(
                    &field_data.name,
                    array.data_type().clone(),
                    nullable,
                ));
                arrays.push(array)
            }
        }

        // if arrays is empty, this returns 'None' meaning that all the fields were all null/empty
        (!arrays.is_empty()).then(|| {
            // if arrays is not empty, try to build the struct array, then wrap in an Arc to produce ArrayRef
            StructArray::try_new(Fields::from(fields), arrays, None)
                .map(|s| Arc::new(s) as ArrayRef)
        })
    }
}

#[cfg(test)]
mod test {
    use arrow::{
        array::{
            BinaryArray, BooleanArray, DurationNanosecondArray, FixedSizeBinaryArray, Float32Array,
            Float64Array, Int8Array, Int16Array, Int32Array, Int64Array, StringArray,
            TimestampNanosecondArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
        },
        datatypes::{Fields, TimeUnit},
    };

    use crate::encode::record::array::{
        ArrayBuilderConstructor, BinaryArrayBuilder, DurationNanosecondArrayBuilder,
        FixedSizeBinaryArrayBuilder, Float32ArrayBuilder, Float64ArrayBuilder, Int8ArrayBuilder,
        Int16ArrayBuilder, Int32ArrayBuilder, Int64ArrayBuilder, NoArgs, PrimitiveArrayBuilder,
        StringArrayBuilder, TimestampNanosecondArrayBuilder, UInt8ArrayBuilder, UInt16ArrayBuilder,
        UInt32ArrayBuilder, UInt64ArrayBuilder, boolean::BooleanBuilderOptions,
    };

    use super::*;

    #[test]
    fn test_struct_builder_simple() {
        let mut struct_builder = AdaptiveStructBuilder::new(vec![
            (
                FieldData::new("name"),
                Box::new(StringArrayBuilder::new(ArrayOptions {
                    dictionary_options: None,
                    nullable: false,
                })),
            ),
            (
                FieldData::new("age"),
                Box::new(UInt8ArrayBuilder::new(ArrayOptions {
                    dictionary_options: None,
                    nullable: true,
                })),
            ),
        ]);

        let name_field_builder = struct_builder
            .field_builder::<StringArrayBuilder>(0)
            .unwrap();
        name_field_builder.append_value(&"joe".to_string());
        name_field_builder.append_value(&"mark".to_string());
        name_field_builder.append_value(&"terry".to_string());

        let age_field_builder = struct_builder
            .field_builder::<UInt8ArrayBuilder>(1)
            .unwrap();
        age_field_builder.append_value(&60);
        age_field_builder.append_value(&70);
        age_field_builder.append_value(&150);

        let result = struct_builder.finish().unwrap().unwrap();
        let result_struct_arr = result.as_any().downcast_ref::<StructArray>().unwrap();

        let expected = StructArray::new(
            Fields::from(vec![
                Field::new("name", DataType::Utf8, false),
                Field::new("age", DataType::UInt8, true),
            ]),
            vec![
                Arc::new(StringArray::from_iter_values(vec!["joe", "mark", "terry"])),
                Arc::new(UInt8Array::from_iter_values(vec![60, 70, 150])),
            ],
            None,
        );

        assert_eq!(result_struct_arr, &expected);
    }

    #[test]
    fn test_struct_builder_all_null_handling() {
        let mut struct_builder = AdaptiveStructBuilder::new(vec![
            (
                FieldData::new("name"),
                Box::new(StringArrayBuilder::new(ArrayOptions {
                    dictionary_options: None,
                    nullable: true,
                })),
            ),
            (
                FieldData::new("age"),
                Box::new(UInt8ArrayBuilder::new(ArrayOptions {
                    dictionary_options: None,
                    nullable: true,
                })),
            ),
        ]);

        // TODO - once we really support nulls here, we should append some nulls to the array
        // https://github.com/open-telemetry/otel-arrow/issues/534

        // here, expect that since all the fields have no real values in them, that we don't return any array at all
        assert!(struct_builder.finish().is_none());

        // check what happens if one of the arrays has no values, but is not nullable (e.g. it's empty)
        // there should be only one empty field.
        let mut struct_builder = AdaptiveStructBuilder::new(vec![
            (
                FieldData::new("name"),
                Box::new(StringArrayBuilder::new(ArrayOptions {
                    dictionary_options: None,
                    nullable: true,
                })),
            ),
            (
                FieldData::new("age"),
                Box::new(UInt8ArrayBuilder::new(ArrayOptions {
                    dictionary_options: None,
                    nullable: false,
                })),
            ),
        ]);
        let result = struct_builder.finish().unwrap().unwrap();
        let result_struct_arr = result.as_any().downcast_ref::<StructArray>().unwrap();
        let expected = StructArray::new(
            Fields::from(vec![Field::new("age", DataType::UInt8, false)]),
            vec![Arc::new(UInt8Array::from_iter_values(vec![]))],
            None,
        );

        assert_eq!(result_struct_arr, &expected);
    }

    #[test]
    fn test_struct_builder_with_invalid_lengths() {
        let mut struct_builder = AdaptiveStructBuilder::new(vec![
            (
                FieldData::new("name"),
                Box::new(StringArrayBuilder::new(ArrayOptions {
                    dictionary_options: None,
                    nullable: false,
                })),
            ),
            (
                FieldData::new("age"),
                Box::new(UInt8ArrayBuilder::new(ArrayOptions {
                    dictionary_options: None,
                    nullable: true,
                })),
            ),
        ]);

        let name_field_builder = struct_builder
            .field_builder::<StringArrayBuilder>(0)
            .unwrap();
        name_field_builder.append_value(&"mark".to_string());
        name_field_builder.append_value(&"terry".to_string());

        let age_field_builder = struct_builder
            .field_builder::<UInt8ArrayBuilder>(1)
            .unwrap();
        age_field_builder.append_value(&60);
        age_field_builder.append_value(&70);
        age_field_builder.append_value(&150);

        let result = struct_builder.finish().unwrap();
        println!("{:?}", result);
    }

    #[test]
    fn test_all_supported_types() {
        let mut fields: Vec<(FieldData, Box<dyn StructArrayBuilderHelper>)> = vec![];

        let mut builder = StringArrayBuilder::new(Default::default());
        builder.append_value(&"a".to_string());
        fields.push((FieldData::new("str"), Box::new(builder)));

        let mut builder = BinaryArrayBuilder::new(Default::default());
        builder.append_value(&b"b".to_vec());
        fields.push((FieldData::new("bin"), Box::new(builder)));
        let mut builder = UInt8ArrayBuilder::new(Default::default());
        builder.append_value(&1);
        fields.push((FieldData::new("u8"), Box::new(builder)));
        let mut builder = UInt16ArrayBuilder::new(Default::default());
        builder.append_value(&2);
        fields.push((FieldData::new("u16"), Box::new(builder)));
        let mut builder = UInt32ArrayBuilder::new(Default::default());
        builder.append_value(&3);
        fields.push((FieldData::new("u32"), Box::new(builder)));
        let mut builder = UInt64ArrayBuilder::new(Default::default());
        builder.append_value(&4);
        fields.push((FieldData::new("u64"), Box::new(builder)));
        let mut builder = Int8ArrayBuilder::new(Default::default());
        builder.append_value(&-1);
        fields.push((FieldData::new("i8"), Box::new(builder)));
        let mut builder = Int16ArrayBuilder::new(Default::default());
        builder.append_value(&-2);
        fields.push((FieldData::new("i16"), Box::new(builder)));
        let mut builder = Int32ArrayBuilder::new(Default::default());
        builder.append_value(&-3);
        fields.push((FieldData::new("i32"), Box::new(builder)));
        let mut builder = Int64ArrayBuilder::new(Default::default());
        builder.append_value(&-4);
        fields.push((FieldData::new("i64"), Box::new(builder)));
        let mut builder = Float32ArrayBuilder::new(Default::default());
        builder.append_value(&1.0);
        fields.push((FieldData::new("f32"), Box::new(builder)));
        let mut builder = Float64ArrayBuilder::new(Default::default());
        builder.append_value(&2.0);
        fields.push((FieldData::new("f64"), Box::new(builder)));
        let mut builder = TimestampNanosecondArrayBuilder::new(Default::default());
        builder.append_value(&1);
        fields.push((FieldData::new("ts_nano"), Box::new(builder)));
        let mut builder = DurationNanosecondArrayBuilder::new(Default::default());
        builder.append_value(&1);
        fields.push((FieldData::new("duration_nano"), Box::new(builder)));
        let mut builder = FixedSizeBinaryArrayBuilder::new_with_args(
            ArrayOptions {
                dictionary_options: None,
                nullable: false,
            },
            4,
        );
        builder.append_value(&b"1234".to_vec());
        fields.push((FieldData::new("fsb"), Box::new(builder)));

        let mut builder =
            AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions { nullable: false });
        builder.append_value(true);
        fields.push((FieldData::new("bool"), Box::new(builder)));

        let mut struct_builder = AdaptiveStructBuilder::new(fields);
        let result = struct_builder.finish().unwrap().unwrap();
        let result_struct_arr = result.as_any().downcast_ref::<StructArray>().unwrap();

        let expected = StructArray::new(
            Fields::from(vec![
                Field::new("str", DataType::Utf8, false),
                Field::new("bin", DataType::Binary, false),
                Field::new("u8", DataType::UInt8, false),
                Field::new("u16", DataType::UInt16, false),
                Field::new("u32", DataType::UInt32, false),
                Field::new("u64", DataType::UInt64, false),
                Field::new("i8", DataType::Int8, false),
                Field::new("i16", DataType::Int16, false),
                Field::new("i32", DataType::Int32, false),
                Field::new("i64", DataType::Int64, false),
                Field::new("f32", DataType::Float32, false),
                Field::new("f64", DataType::Float64, false),
                Field::new(
                    "ts_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    "duration_nano",
                    DataType::Duration(TimeUnit::Nanosecond),
                    false,
                ),
                Field::new("fsb", DataType::FixedSizeBinary(4), false),
                Field::new("bool", DataType::Boolean, false),
            ]),
            vec![
                Arc::new(StringArray::from(vec!["a"])),
                Arc::new(BinaryArray::from_iter_values(vec![b"b"])),
                Arc::new(UInt8Array::from_iter_values(vec![1])),
                Arc::new(UInt16Array::from_iter_values(vec![2])),
                Arc::new(UInt32Array::from_iter_values(vec![3])),
                Arc::new(UInt64Array::from_iter_values(vec![4])),
                Arc::new(Int8Array::from_iter_values(vec![-1])),
                Arc::new(Int16Array::from_iter_values(vec![-2])),
                Arc::new(Int32Array::from_iter_values(vec![-3])),
                Arc::new(Int64Array::from_iter_values(vec![-4])),
                Arc::new(Float32Array::from_iter_values(vec![1.0])),
                Arc::new(Float64Array::from_iter_values(vec![2.0])),
                Arc::new(TimestampNanosecondArray::from_iter_values(vec![1])),
                Arc::new(DurationNanosecondArray::from_iter_values(vec![1])),
                Arc::new(FixedSizeBinaryArray::try_from_iter([b"1234".to_vec()].iter()).unwrap()),
                Arc::new(BooleanArray::from(vec![true])),
            ],
            None,
        );
        assert_eq!(result_struct_arr, &expected);
    }

    #[test]
    fn test_get_field_builder() {
        // this "do_positive_test" macro just ensures that we can get the field builder from
        // the struct builder for the given type
        macro_rules! do_test {
            ($type:ident) => {
                paste! {
                    let mut struct_builder = AdaptiveStructBuilder::new(vec![(
                        FieldData {
                            name: "test".to_string(),
                        },
                        Box::new([<$type ArrayBuilder>]::new(ArrayOptions {
                            dictionary_options: None,
                            nullable: true,
                        })),
                    )]);
                    assert!(struct_builder.field_builder::<[<$type ArrayBuilder>]>(0).is_some(),
                        "Expected field_builder at index 0 to return Some for type {}ArrayBuilder, but got None",
                        stringify!($type)
                    );
                }
            };
        }

        do_test!(String);
        do_test!(Binary);
        do_test!(UInt8);
        do_test!(UInt16);
        do_test!(UInt32);
        do_test!(UInt64);
        do_test!(Int8);
        do_test!(Int16);
        do_test!(Int32);
        do_test!(Int64);
        do_test!(Float32);
        do_test!(Float64);
        do_test!(TimestampNanosecond);

        // check for boolean (special case b/c it is not a variation of AdaptiveArrayBuilder)
        let mut struct_builder = AdaptiveStructBuilder::new(vec![(
            FieldData {
                name: "test".to_string(),
            },
            Box::new(AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
                nullable: true,
            })),
        )]);
        assert!(
            struct_builder
                .field_builder::<AdaptiveBooleanArrayBuilder>(0)
                .is_some(),
            "Expected field_builder at index 0 to return Some for AdaptiveBooleanArrayBuilder, but got None"
        );

        // check for FSB (special case b/c it implements CheckedArrayAppend instead of ArrayAppend)
        let mut struct_builder = AdaptiveStructBuilder::new(vec![(
            FieldData {
                name: "test".to_string(),
            },
            Box::new(FixedSizeBinaryArrayBuilder::new_with_args(
                Default::default(),
                1,
            )),
        )]);
        assert!(
            struct_builder
                .checked_field_builder::<FixedSizeBinaryArrayBuilder>(0)
                .is_some(),
            "Expected field_builder at index 0 to return Some for AdaptiveBooleanArrayBuilder, but got None"
        );
    }

    #[test]
    fn test_get_field_builder_failures() {
        let mut struct_builder = AdaptiveStructBuilder::new(vec![
            (
                FieldData {
                    name: "test".to_string(),
                },
                Box::new(StringArrayBuilder::new(ArrayOptions {
                    dictionary_options: None,
                    nullable: true,
                })),
            ),
            (
                FieldData {
                    name: "test2".to_string(),
                },
                Box::new(FixedSizeBinaryArrayBuilder::new_with_args(
                    ArrayOptions {
                        dictionary_options: None,
                        nullable: true,
                    },
                    1,
                )),
            ),
        ]);

        // assert that we cannot get the field if the index is out of bounds
        assert!(
            struct_builder
                .field_builder::<StringArrayBuilder>(2)
                .is_none()
        );
        assert!(
            struct_builder
                .checked_field_builder::<FixedSizeBinaryArrayBuilder>(2)
                .is_none()
        );

        // assert we cannot get the field if it is the wrong type
        assert!(
            struct_builder
                .field_builder::<BinaryArrayBuilder>(0)
                .is_none()
        );
        assert!(
            struct_builder
                .checked_field_builder::<FixedSizeBinaryArrayBuilder>(0)
                .is_none()
        );
    }
}
