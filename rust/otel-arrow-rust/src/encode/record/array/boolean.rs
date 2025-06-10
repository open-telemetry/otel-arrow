// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::{Array, ArrayRef, ArrowPrimitiveType, BooleanBuilder};
use arrow::compute::kernels::boolean;
use arrow::datatypes::{ArrowDictionaryKeyType, DataType};
use arrow::error::ArrowError;

use super::dictionary::{self, DictionaryArrayAppend};
use super::{ArrayBuilder, ArrayBuilderConstructor};

/// `AdaptiveBooleanArray` builder an adaptive array builder that can be either all null, in which case
/// the finish function won't construct an array (will return None), otherwise it will create the array.
///
/// This is implemented a bit differently than for other types because `Boolean` is the one datatype
/// where it would never really make sense to have it in a dictionary.
pub struct AdaptiveBooleanArrayBuilder {
    pub nullable: bool,
    inner: Option<BooleanBuilder>,
}

pub struct BooleanBuilderOptions {
    pub nullable: bool,
}

impl AdaptiveBooleanArrayBuilder {
    pub fn new(options: BooleanBuilderOptions) -> Self {
        let inner = if options.nullable {
            None
        } else {
            Some(BooleanBuilder::new())
        };

        Self {
            inner,
            nullable: options.nullable,
        }
    }

    pub fn append_value(&mut self, value: bool) {
        if self.inner.is_none() {
            // TODO -- when we handle nulls here we need to keep track of how many
            // nulls have been appended before the first value, and prefix this
            // newly initialized array with that number of nulls
            // https://github.com/open-telemetry/otel-arrow/issues/534

            self.inner = Some(BooleanBuilder::new());
        }

        let inner = self
            .inner
            .as_mut()
            .expect("inner should now be initialized");
        inner.append_value(value);
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

    use arrow::array::{BooleanArray, BooleanBuilder};
    use arrow::datatypes::DataType;

    #[test]
    fn test_adaptive_boolean_builder() {
        let mut builder =
            AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions { nullable: false });
        builder.append_value(true);
        builder.append_value(false);
        let result = builder.finish().expect("should finish successfully");

        assert_eq!(result.data_type(), &DataType::Boolean);
        let boolean_array = result
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("should downcast to BooleanArray");
        assert_eq!(boolean_array.len(), 2);
        assert!(boolean_array.value(0));
        assert!(!boolean_array.value(1));
    }

    #[test]
    fn test_adaptive_boolean_builder_empty() {
        let mut builder =
            AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions { nullable: true });
        // expect we've returned None because there are no values
        assert!(builder.finish().is_none());

        // expect if we then append values, we convert the 'None' (empty builder) into
        // a builder and finishing it returns a result
        builder.append_value(true);
        assert!(builder.finish().is_some());
    }
}
