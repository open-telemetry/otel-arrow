// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::{ArrayBuilder as _, ArrayRef, BooleanBuilder};

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

    // whether any `true` value has been appended. When `skip_all_false` is true, the builder
    // skips producing an array if all values are `false` or null (the boolean default).
    has_true_value: bool,

    // whether this builder was created with optional = true
    optional: bool,

    // whether to skip producing an array when all values are false or null.
    // Use true for structural fields (e.g. is_monotonic) where false is the default.
    // Use false for data fields (e.g. bool attribute values) where false is meaningful.
    skip_all_false: bool,
}

pub struct BooleanBuilderOptions {
    // Whether the array that's being constructed is "optional". If optional = false, we'll produce
    // the boolean array regardless of whether all the values are null.
    pub optional: bool,

    // Whether to skip producing an array when all values are false or null.
    // Use true for structural fields (e.g. is_monotonic) where false is the default.
    // Use false for data fields (e.g. bool attribute values) where false is meaningful.
    pub skip_all_false: bool,
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
            has_true_value: false,
            optional: options.optional,
            skip_all_false: options.skip_all_false,
        }
    }

    pub fn append_value(&mut self, value: bool) {
        if value {
            self.has_true_value = true;
        }

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
    #[allow(dead_code)]
    pub fn append_null(&mut self) {
        match self.inner.as_mut() {
            Some(builder) => builder.append_null(),
            None => self.nulls_prefix += 1,
        };
    }

    /// Append `n` nulls to the builder
    pub(crate) fn append_nulls(&mut self, n: usize) {
        match self.inner.as_mut() {
            Some(builder) => builder.append_nulls(n),
            None => self.nulls_prefix += n,
        };
    }

    /// Returns the number of elements appended to the builder.
    #[must_use]
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        match &self.inner {
            Some(builder) => builder.len(),
            None => self.nulls_prefix,
        }
    }

    /// Returns `true` if no elements have been appended.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn finish(&mut self) -> Option<ArrayRef> {
        if self.optional && self.skip_all_false && !self.has_true_value {
            return None;
        }
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
        let mut builder = AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
            optional: true,
            skip_all_false: true,
        });
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
        let mut builder = AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
            optional: true,
            skip_all_false: true,
        });
        builder.append_null();
        builder.append_nulls(2);
        // expect we've returned None because there are no values
        assert!(builder.finish().is_none());
    }

    #[test]
    fn test_adaptive_boolean_builder_null_prefix() {
        let mut builder = AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
            optional: true,
            skip_all_false: true,
        });
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
        let mut builder = AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
            optional: true,
            skip_all_false: true,
        });
        // expect we've returned None because there are no values
        assert!(builder.finish().is_none());

        // expect if we then append values, we convert the 'None' (empty builder) into
        // a builder and finishing it returns a result
        builder.append_value(true);
        assert!(builder.finish().is_some());
    }

    #[test]
    fn test_adaptive_boolean_builder_all_false() {
        let mut builder = AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
            optional: true,
            skip_all_false: true,
        });
        builder.append_value(false);
        builder.append_value(false);
        builder.append_value(false);
        // all values are the boolean default (false), so skip producing an array
        assert!(builder.finish().is_none());
    }

    #[test]
    fn test_adaptive_boolean_builder_false_and_null() {
        let mut builder = AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
            optional: true,
            skip_all_false: true,
        });
        builder.append_null();
        builder.append_value(false);
        builder.append_nulls(2);
        builder.append_value(false);
        // mix of false and null — still all defaults, so skip producing an array
        assert!(builder.finish().is_none());
    }

    #[test]
    fn test_adaptive_boolean_builder_false_then_true() {
        let mut builder = AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
            optional: true,
            skip_all_false: true,
        });
        builder.append_value(false);
        builder.append_null();
        builder.append_value(true);
        let result = builder
            .finish()
            .expect("should produce array when true is present");
        let boolean_array = result
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("should downcast to BooleanArray");
        assert_eq!(boolean_array.len(), 3);
        assert!(!boolean_array.value(0));
        assert!(!boolean_array.is_valid(1));
        assert!(boolean_array.value(2));
    }

    #[test]
    fn test_adaptive_boolean_builder_all_false_non_optional() {
        let mut builder = AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
            optional: false,
            skip_all_false: true,
        });
        builder.append_value(false);
        builder.append_value(false);
        // non-optional builders always produce an array, even if all false
        let result = builder
            .finish()
            .expect("non-optional should always produce array");
        let boolean_array = result
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("should downcast to BooleanArray");
        assert_eq!(boolean_array.len(), 2);
        assert!(!boolean_array.value(0));
        assert!(!boolean_array.value(1));
    }
}
