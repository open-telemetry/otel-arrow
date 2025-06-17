use std::collections::HashSet;

use crate::{Expression, ImmutableValueExpression, MutableValueExpression, QueryLocation};

#[derive(Debug, Clone, PartialEq)]
pub enum TransformExpression {
    Set(SetTransformExpression),
    Clear(ClearTransformExpression),
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

#[derive(Debug, Clone, PartialEq)]
pub struct ClearTransformExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    keys_to_keep: HashSet<SourceKey>,
}

impl ClearTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
        keys_to_keep: HashSet<SourceKey>,
    ) -> ClearTransformExpression {
        Self {
            query_location,
            target,
            keys_to_keep,
        }
    }

    pub fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }

    pub fn get_keys_to_keep(&self) -> &HashSet<SourceKey> {
        &self.keys_to_keep
    }
}

impl Expression for ClearTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum SourceKey {
    Identifier(Box<str>),
    Pattern(Box<str>)
}
