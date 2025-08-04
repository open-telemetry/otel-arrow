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

    fn get(&self, index: usize) -> Option<&(dyn AsStaticValue + 'static)> {
        self.value.get(index).map(|v| v as &dyn AsStaticValue)
    }

    fn get_items(&self, item_callback: &mut dyn IndexValueCallback) -> bool {
        for (index, value) in self.value.iter().enumerate() {
            if !item_callback.next(index, value.to_value()) {
                return false;
            }
        }

        true
    }
}
