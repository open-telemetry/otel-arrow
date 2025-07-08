use std::fmt::Debug;

use crate::{ExpressionError, QueryLocation, Value};

pub trait MapValue: Debug {
    fn is_empty(&self) -> bool;

    fn len(&self) -> usize;

    fn contains_key(&self, key: &str) -> bool;

    fn get(&self, key: &str) -> Option<Value>;

    fn get_items(&self, item_callback: &mut dyn KeyValueCallback) -> bool;
}

pub trait KeyValueCallback {
    fn next(&mut self, key: &str, value: Value) -> bool;
}

pub struct KeyValueClosureCallback<F>
where
    F: FnMut(&str, Value) -> bool,
{
    callback: F,
}

impl<F> KeyValueClosureCallback<F>
where
    F: FnMut(&str, Value) -> bool,
{
    pub fn new(callback: F) -> KeyValueClosureCallback<F> {
        Self { callback }
    }
}

impl<F> KeyValueCallback for KeyValueClosureCallback<F>
where
    F: FnMut(&str, Value) -> bool,
{
    fn next(&mut self, key: &str, value: Value) -> bool {
        (self.callback)(key, value)
    }
}

pub(crate) fn equal_to(
    query_location: &QueryLocation,
    left: &dyn MapValue,
    right: &dyn MapValue,
    case_insensitive: bool,
) -> Result<bool, ExpressionError> {
    if left.len() != right.len() {
        return Ok(false);
    }

    let mut e = None;

    let completed = left.get_items(&mut KeyValueClosureCallback::new(
        |k, left_value| match right.get(k) {
            Some(right_value) => {
                let r = Value::are_values_equal(
                    query_location,
                    &left_value,
                    &right_value,
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
        },
    ));

    if let Some(exp_e) = e {
        Err(exp_e)
    } else {
        Ok(completed)
    }
}
