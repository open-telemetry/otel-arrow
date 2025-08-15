use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayScalarExpression {
    query_location: QueryLocation,
    value: Vec<StaticScalarExpression>,
}

impl ArrayScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        value: Vec<StaticScalarExpression>,
    ) -> ArrayScalarExpression {
        Self {
            query_location,
            value,
        }
    }
}

impl Expression for ArrayScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ArrayScalarExpression"
    }
}

impl ArrayValue for ArrayScalarExpression {
    fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    fn len(&self) -> usize {
        self.value.len()
    }

    fn get(&self, index: usize) -> Option<&(dyn AsValue)> {
        self.value.get(index).map(|v| v as &dyn AsValue)
    }

    fn get_static(&self, index: usize) -> Result<Option<&(dyn AsStaticValue + 'static)>, String> {
        Ok(self.value.get(index).map(|v| v as &dyn AsStaticValue))
    }

    fn get_item_range(
        &self,
        range: ArrayRange,
        item_callback: &mut dyn IndexValueCallback,
    ) -> bool {
        for (index, value) in range.get_slice(&self.value).iter().enumerate() {
            if !item_callback.next(index, value.to_value()) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_item_range() {
        let test = |range: ArrayRange, expected: &[i64]| {
            let array = ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        0,
                    )),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        2,
                    )),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        3,
                    )),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        4,
                    )),
                ],
            );

            let mut indices = Vec::new();

            array.get_item_range(
                range,
                &mut IndexValueClosureCallback::new(|_, v| {
                    if let Value::Integer(i) = v {
                        indices.push(i.get_value());
                    }
                    true
                }),
            );

            assert_eq!(expected, indices);
        };

        test((..).into(), &[0, 1, 2, 3, 4]);
        test((1..).into(), &[1, 2, 3, 4]);
        test((1..1).into(), &[]);
        test((..1).into(), &[0]);
        test((..=1).into(), &[0, 1]);
        test((1..=1).into(), &[1]);
    }
}
