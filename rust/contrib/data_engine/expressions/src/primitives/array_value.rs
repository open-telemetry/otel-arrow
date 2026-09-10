// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Debug, ops::*};

use crate::*;

pub trait ArrayValue: Debug {
    fn is_empty(&self) -> bool;

    fn len(&self) -> usize;

    fn get(&self, index: usize) -> Option<&dyn AsValue>;

    // Note: Used to update the RefCell borrow when accessing sub-elements of
    // the source or variables which use interior mutability. In arrays that
    // have dynamic elements a string message will be returned indicating lack
    // of support for this method
    fn get_static(&self, index: usize) -> Result<Option<&(dyn AsStaticValue + 'static)>, String>;

    fn get_items<'a>(&'a self, item_callback: &mut ArrayValueIteratorCallback<'a, '_>) -> bool {
        self.get_item_range((..).into(), item_callback)
    }

    fn get_item_range<'a>(
        &'a self,
        range: ArrayRange,
        item_callback: &mut ArrayValueIteratorCallback<'a, '_>,
    ) -> bool;

    fn to_string(&self) -> ValueString<'_> {
        let mut values = Vec::new();

        self.get_items(&mut |_, value| {
            values.push(value.to_json_value());
            true
        });

        ValueString::Owned(serde_json::Value::Array(values).to_string())
    }
}

pub type ArrayValueIteratorCallback<'a, 'b> = dyn FnMut(usize, Value<'a>) -> bool + 'b;

#[derive(Debug)]
pub struct ArrayRange {
    start_range_inclusize: Option<usize>,
    end_range_exclusive: Option<usize>,
}

impl ArrayRange {
    pub fn get_start_range_inclusize(&self) -> Option<usize> {
        self.start_range_inclusize
    }

    pub fn get_end_range_exclusive(&self) -> Option<usize> {
        self.end_range_exclusive
    }

    pub fn get_slice<'a, T>(&self, value: &'a [T]) -> &'a [T] {
        let start = self.start_range_inclusize.unwrap_or(0);
        let end = self.end_range_exclusive.unwrap_or(value.len());
        &value[start..end]
    }
}

impl From<usize> for ArrayRange {
    fn from(index: usize) -> Self {
        Self {
            start_range_inclusize: Some(index),
            end_range_exclusive: Some(index + 1),
        }
    }
}

impl From<RangeFull> for ArrayRange {
    fn from(_: RangeFull) -> Self {
        Self {
            start_range_inclusize: None,
            end_range_exclusive: None,
        }
    }
}

impl From<Range<usize>> for ArrayRange {
    fn from(value: Range<usize>) -> Self {
        Self {
            start_range_inclusize: Some(value.start),
            end_range_exclusive: Some(value.end),
        }
    }
}

impl From<RangeFrom<usize>> for ArrayRange {
    fn from(value: RangeFrom<usize>) -> Self {
        Self {
            start_range_inclusize: Some(value.start),
            end_range_exclusive: None,
        }
    }
}

impl From<RangeTo<usize>> for ArrayRange {
    fn from(value: RangeTo<usize>) -> Self {
        Self {
            start_range_inclusize: None,
            end_range_exclusive: Some(value.end),
        }
    }
}

impl From<RangeToInclusive<usize>> for ArrayRange {
    fn from(value: RangeToInclusive<usize>) -> Self {
        Self {
            start_range_inclusize: None,
            end_range_exclusive: Some(value.end + 1),
        }
    }
}

impl From<RangeInclusive<usize>> for ArrayRange {
    fn from(value: RangeInclusive<usize>) -> Self {
        Self {
            start_range_inclusize: Some(*value.start()),
            end_range_exclusive: Some(value.end() + 1),
        }
    }
}

pub(crate) fn equal_to(
    query_location: &QueryLocation,
    left: &dyn ArrayValue,
    right: &dyn ArrayValue,
    case_insensitive: bool,
) -> Result<bool, ExpressionError> {
    if left.len() != right.len() {
        return Ok(false);
    }

    let mut e = None;

    let completed = left.get_items(&mut |index, left_value| match right.get(index) {
        Some(right_value) => {
            let r = Value::are_values_equal(
                query_location,
                &left_value,
                &right_value.to_value(),
                case_insensitive,
            );
            if let Err(exp_e) = r {
                e = Some(exp_e);
                false
            } else {
                r.unwrap()
            }
        }
        None => false,
    });

    if let Some(exp_e) = e {
        Err(exp_e)
    } else {
        Ok(completed)
    }
}
