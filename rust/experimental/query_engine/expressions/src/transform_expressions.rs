use std::collections::HashSet;

use crate::{
    Expression, ImmutableValueExpression, MutableValueExpression, QueryLocation,
    StringScalarExpression,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TransformExpression {
    /// Set data transformation.
    Set(SetTransformExpression),

    /// Remove data transformation.
    Remove(RemoveTransformExpression),

    /// Remove keys transformation.
    ///
    /// Note: Remove keys is a specialized form of the remove transformation
    /// which takes a target map and a list of keys to be removed.
    RemoveKeys(RemoveKeysTransformExpression),

    /// Clear keys transformation.
    ///
    /// Note: Clear keys is used to remove all keys from a target map with an
    /// optional list of keys to retain.
    ClearKeys(ClearKeysTransformExpression),
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
pub struct RemoveTransformExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
}

impl RemoveTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
    ) -> RemoveTransformExpression {
        Self {
            query_location,
            target,
        }
    }

    pub fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }
}

impl Expression for RemoveTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveKeysTransformExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    keys_to_remove: HashSet<SourceKey>,
}

impl RemoveKeysTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
        keys_to_remove: HashSet<SourceKey>,
    ) -> RemoveKeysTransformExpression {
        Self {
            query_location,
            target,
            keys_to_remove,
        }
    }

    pub fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }

    pub fn get_keys_to_remove(&self) -> &HashSet<SourceKey> {
        &self.keys_to_remove
    }
}

impl Expression for RemoveKeysTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClearKeysTransformExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    keys_to_keep: HashSet<SourceKey>,
}

impl ClearKeysTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
        keys_to_keep: HashSet<SourceKey>,
    ) -> ClearKeysTransformExpression {
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

impl Expression for ClearKeysTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum SourceKey {
    Identifier(StringScalarExpression),
    Pattern(StringScalarExpression),
}
