// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the ArrayPrefixBuilder which is used for keeping track of the prefix
//! of an uninitialized AdaptiveArrayBuilder.

use arrow::{array::NullBufferBuilder, error::ArrowError};

use crate::encode::record::array::{
    ArrayAppend, ArrayAppendNulls, CheckedArrayAppend, dictionary::UncheckedArrayBuilderAdapter,
};

/// `ArrayPrefixBuilder` keeps track of what would go at the start of an uninitialized builder
/// when we decide we need to initialize it.
///
/// For example, if we're appending many nulls to the start of an optional array and suddenly
/// we discover that we need to initialize the builder because we received a non-null or non-
/// default value, we'll append the prefix and then append the value.
///
pub struct ArrayPrefixBuilder {
    null_buffer_builder: NullBufferBuilder,
}

impl ArrayPrefixBuilder {
    pub fn new() -> Self {
        Self {
            null_buffer_builder: NullBufferBuilder::new(0),
        }
    }

    pub fn append_value(&mut self) {
        self.null_buffer_builder.append_non_null();
    }

    pub fn append_null(&mut self) {
        self.null_buffer_builder.append_null();
    }

    pub fn append_nulls(&mut self, n: usize) {
        self.null_buffer_builder.append_n_nulls(n);
    }

    pub fn init_builder<B, T>(&mut self, builder: &mut B, default_value: Option<T>)
    where
        T: Clone,
        B: ArrayAppend<Native = T> + ArrayAppendNulls,
    {
        // safety: we're OK to call expect() here because UncheckedArrayAdapter will never return Error
        // which means populate_native_builder will also not return an error
        self.init_builder_checked(
            &mut UncheckedArrayBuilderAdapter { inner: builder },
            default_value,
        )
        .expect("can safely append values");
    }

    pub fn init_builder_checked<B, T>(
        &mut self,
        builder: &mut B,
        default_value: Option<T>,
    ) -> Result<(), ArrowError>
    where
        T: Clone,
        B: CheckedArrayAppend<Native = T> + ArrayAppendNulls,
    {
        let len = self.null_buffer_builder.len();

        if default_value.is_none() {
            self.init_builder_all_nulls(builder, len);
            return Ok(());
        }
        // safety: we've just checked this above
        let default_value = default_value.clone().expect("default value is some");

        let nulls = self.null_buffer_builder.finish();
        if let Some(nulls) = nulls {
            if nulls.null_count() == len {
                self.init_builder_all_nulls(builder, len);
                return Ok(());
            }

            // it's a mix of nulls and values, append all null or value depending on if the
            // value at the index is valid
            for is_valid in nulls.iter() {
                if is_valid {
                    builder.append_value(&default_value.clone())?;
                } else {
                    builder.append_null();
                }
            }
            return Ok(());
        }

        // there are no nulls, must append all the values:
        for _ in 0..len {
            builder.append_value(&default_value.clone())?
        }

        Ok(())
    }

    pub fn init_builder_all_nulls<B>(&mut self, builder: &mut B, len: usize)
    where
        B: ArrayAppendNulls,
    {
        builder.append_nulls(len);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{Array, Int32Builder};

    struct TestArrayBuilder {
        builder: Int32Builder,
        nulls: usize,
    }

    impl ArrayAppend for TestArrayBuilder {
        type Native = i32;
        fn append_value(&mut self, value: &Self::Native) {
            self.builder.append_value(*value);
        }
    }

    impl ArrayAppendNulls for TestArrayBuilder {
        fn append_null(&mut self) {
            self.builder.append_null();
            self.nulls += 1;
        }
        fn append_nulls(&mut self, n: usize) {
            for _ in 0..n {
                self.append_null();
            }
        }
    }

    impl CheckedArrayAppend for TestArrayBuilder {
        type Native = i32;
        fn append_value(&mut self, value: &Self::Native) -> Result<(), ArrowError> {
            self.builder.append_value(*value);
            Ok(())
        }
    }

    #[test]
    fn test_append_nulls_and_values() {
        let mut prefix = ArrayPrefixBuilder::new();
        prefix.append_null();
        prefix.append_value();
        prefix.append_null();
        prefix.append_value();

        let mut builder = TestArrayBuilder {
            builder: Int32Builder::new(),
            nulls: 0,
        };
        prefix.init_builder(&mut builder, Some(42));
        let array = builder.builder.finish();

        assert_eq!(array.len(), 4);
        assert_eq!(array.null_count(), 2);
        assert_eq!(array.value(1), 42);
        assert_eq!(array.value(3), 42);
        assert!(!array.is_valid(0));
        assert!(!array.is_valid(2));
    }

    #[test]
    fn test_all_nulls() {
        let mut prefix = ArrayPrefixBuilder::new();
        prefix.append_nulls(5);

        let mut builder = TestArrayBuilder {
            builder: Int32Builder::new(),
            nulls: 0,
        };
        prefix.init_builder(&mut builder, None);
        let array = builder.builder.finish();

        assert_eq!(array.len(), 5);
        assert_eq!(array.null_count(), 5);
        for i in 0..5 {
            assert!(!array.is_valid(i));
        }
    }

    #[test]
    fn test_all_values() {
        let mut prefix = ArrayPrefixBuilder::new();
        for _ in 0..3 {
            prefix.append_value();
        }

        let mut builder = TestArrayBuilder {
            builder: Int32Builder::new(),
            nulls: 0,
        };
        prefix.init_builder(&mut builder, Some(7));
        let array = builder.builder.finish();

        assert_eq!(array.len(), 3);
        assert_eq!(array.null_count(), 0);
        for i in 0..3 {
            assert_eq!(array.value(i), 7);
        }
    }

    #[test]
    fn test_init_builder_checked_error_free() {
        let mut prefix = ArrayPrefixBuilder::new();
        prefix.append_value();
        prefix.append_null();
        prefix.append_value();

        let mut builder = TestArrayBuilder {
            builder: Int32Builder::new(),
            nulls: 0,
        };
        let result = prefix.init_builder_checked(&mut builder, Some(99));
        assert!(result.is_ok());
        let array = builder.builder.finish();
        assert_eq!(array.len(), 3);
        assert_eq!(array.value(0), 99);
        assert!(!array.is_valid(1));
        assert_eq!(array.value(2), 99);
    }

    #[test]
    fn test_init_builder_all_nulls_trait() {
        let mut prefix = ArrayPrefixBuilder::new();
        let mut builder = TestArrayBuilder {
            builder: Int32Builder::new(),
            nulls: 0,
        };
        prefix.init_builder_all_nulls(&mut builder, 4);
        let array = builder.builder.finish();
        assert_eq!(array.len(), 4);
        assert_eq!(array.null_count(), 4);
    }
}
