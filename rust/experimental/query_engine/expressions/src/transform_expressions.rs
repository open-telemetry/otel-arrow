use crate::{Expression, ImmutableValueExpression, MutableValueExpression, QueryLocation};

#[derive(Debug, Clone, PartialEq)]
pub enum TransformExpression {
    Set(SetTransformExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetTransformExpression {
    query_location: QueryLocation,
    source: ImmutableValueExpression,
    destination: MutableValueExpression,
}

impl SetTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        source: ImmutableValueExpression,
        destination: MutableValueExpression,
    ) -> SetTransformExpression {
        Self {
            query_location,
            source,
            destination,
        }
    }

    pub fn get_source(&self) -> &ImmutableValueExpression {
        &self.source
    }

    pub fn get_destination(&self) -> &MutableValueExpression {
        &self.destination
    }
}

impl Expression for SetTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}
