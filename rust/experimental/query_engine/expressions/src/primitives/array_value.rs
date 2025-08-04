use std::fmt::Debug;

use crate::*;

pub trait ArrayValue: Debug {
    fn is_empty(&self) -> bool;

    fn len(&self) -> usize;

    fn get(&self, index: usize) -> Option<&(dyn AsStaticValue + 'static)>;

    fn get_items(&self, item_callback: &mut dyn IndexValueCallback) -> bool;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        let mut values = Vec::new();

        self.get_items(&mut IndexValueClosureCallback::new(|_, value| {
            values.push(value.to_json_value());
            true
        }));

        (action)(serde_json::Value::Array(values).to_string().as_str())
    }
}

pub trait IndexValueCallback {
    fn next(&mut self, index: usize, value: Value) -> bool;
}

pub struct IndexValueClosureCallback<F>
where
    F: FnMut(usize, Value) -> bool,
{
    callback: F,
}

impl<F> IndexValueClosureCallback<F>
where
    F: FnMut(usize, Value) -> bool,
{
    pub fn new(callback: F) -> IndexValueClosureCallback<F> {
        Self { callback }
    }
}

impl<F> IndexValueCallback for IndexValueClosureCallback<F>
where
    F: FnMut(usize, Value) -> bool,
{
    fn next(&mut self, index: usize, value: Value) -> bool {
        (self.callback)(index, value)
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

    let completed =
        left.get_items(&mut IndexValueClosureCallback::new(
            |index, left_value| match right.get(index) {
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
            },
        ));

    if let Some(exp_e) = e {
        Err(exp_e)
    } else {
        Ok(completed)
    }
}
