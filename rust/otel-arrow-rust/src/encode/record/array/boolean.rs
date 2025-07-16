// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanBuilder};

/// `AdaptiveBooleanArray` builder an adaptive array builder that can be either all null, in which case
/// the finish function won't construct an array (will return None), otherwise it will create the array.
///
/// This is implemented a bit differently than for other types because `Boolean` is the one datatype
/// where it would never really make sense to have it in a dictionary.
pub struct AdaptiveBooleanArrayBuilder {
    inner: Option<BooleanBuilder>,

    // the number of nulls that have been appended to the builder before the first value. This is
    // used as a counter until the underlying builder possibly gets initialized, then we prepend
    // this many nulls
    nulls_prefix: usize,
}

pub struct BooleanBuilderOptions {
    // Whether the array that's being constructed is "optional". If optional = false, we'll produce
    // the boolean array regardless of whether all the values are null.
    pub optional: bool,
}

impl AdaptiveBooleanArrayBuilder {
    pub fn new(options: BooleanBuilderOptions) -> Self {
        let inner = if options.optional {
            None
        } else {
            Some(BooleanBuilder::new())
        };

        Self {
            inner,
            nulls_prefix: 0,
        }
    }

    pub fn append_value(&mut self, value: bool) {
        if self.inner.is_none() {
            self.inner = Some(BooleanBuilder::new());
            if self.nulls_prefix > 0 {
                self.append_nulls(self.nulls_prefix);
            }
        }

        let inner = self
            .inner
            .as_mut()
            .expect("inner should now be initialized");
        inner.append_value(value);
    }

    /// Append a null value to the builder
    pub fn append_null(&mut self) {
        match self.inner.as_mut() {
            Some(builder) => builder.append_null(),
            None => self.nulls_prefix += 1,
        };
    }

    /// Append `n` nulls to the builder
    fn append_nulls(&mut self, n: usize) {
        match self.inner.as_mut() {
            Some(builder) => builder.append_nulls(n),
            None => self.nulls_prefix += n,
        };
    }

    pub fn finish(&mut self) -> Option<ArrayRef> {
        self.inner
            .as_mut()
            .map(|inner| Arc::new(inner.finish()) as ArrayRef)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{Array, BooleanArray};
    use arrow::datatypes::DataType;

    #[test]
    fn test_adaptive_boolean_builder() {
        let mut builder =
            AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions { optional: true });
        builder.append_value(true);
        builder.append_value(false);
        builder.append_null();
        builder.append_value(true);
        builder.append_nulls(2);
        builder.append_value(false);
        let result = builder.finish().expect("should finish successfully");

        assert_eq!(result.data_type(), &DataType::Boolean);
        let boolean_array = result
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("should downcast to BooleanArray");
        assert_eq!(boolean_array.len(), 7);
        assert!(boolean_array.value(0));
        assert!(!boolean_array.value(1));
        assert!(!boolean_array.is_valid(2));
        assert!(boolean_array.value(3));
        assert!(!boolean_array.is_valid(4));
        assert!(!boolean_array.is_valid(5));
        assert!(!boolean_array.value(6));
    }

    #[test]
    fn test_adaptive_boolean_builder_all_null() {
        let mut builder =
            AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions { optional: true });
        builder.append_null();
        builder.append_nulls(2);
        // expect we've returned None because there are no values
        assert!(builder.finish().is_none());
    }

    #[test]
    fn test_adaptive_boolean_builder_null_prefix() {
        let mut builder =
            AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions { optional: true });
        builder.append_null();
        builder.append_nulls(2);
        builder.append_value(true);
        let result = builder.finish().unwrap();
        let boolean_array = result
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("should downcast to BooleanArray");
        assert!(!boolean_array.is_valid(0));
        assert!(!boolean_array.is_valid(1));
        assert!(!boolean_array.is_valid(2));
        assert!(boolean_array.value(3));
    }

    #[test]
    fn test_adaptive_boolean_builder_empty() {
        let mut builder =
            AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions { optional: true });
        // expect we've returned None because there are no values
        assert!(builder.finish().is_none());

        // expect if we then append values, we convert the 'None' (empty builder) into
        // a builder and finishing it returns a result
        builder.append_value(true);
        assert!(builder.finish().is_some());
    }
}
