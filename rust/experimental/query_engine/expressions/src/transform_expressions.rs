use std::collections::HashSet;

use crate::{
    Expression, ImmutableValueExpression, MutableValueExpression, QueryLocation,
    StringScalarExpression, ValueAccessor, ValueSelector,
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

    /// Remove data from a target map.
    ReduceMap(ReduceMapTransformExpression),
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

#[derive(Debug, Clone, PartialEq)]
pub enum ReduceMapTransformExpression {
    Keep(KeepMapReductionExpression),
    Remove(RemoveMapReductionExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapSelector {
    KeyPattern(StringScalarExpression),
    Key(StringScalarExpression),
    ValueAccessor(ValueAccessor),
}

pub trait MapReductionExpression: Expression {
    fn get_target(&self) -> &MutableValueExpression;

    fn get_selectors(&self) -> &Vec<MapSelector>;

    fn push_selector(&mut self, selector: MapSelector);

    fn push_value_accessor(&mut self, value_accessor: &ValueAccessor) -> bool {
        let selectors = value_accessor.get_selectors();
        if selectors.is_empty() {
            return false;
        }

        if selectors.len() == 1 {
            match selectors.first().unwrap() {
                ValueSelector::ArrayIndex(_) => return false,
                ValueSelector::MapKey(s) => self.push_selector(MapSelector::Key(s.clone())),
                ValueSelector::ScalarExpression(_) => {
                    self.push_selector(MapSelector::ValueAccessor(value_accessor.clone()))
                }
            }
        } else {
            self.push_selector(MapSelector::ValueAccessor(value_accessor.clone()));
        }

        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeepMapReductionExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    selectors: Vec<MapSelector>,
}

impl KeepMapReductionExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
    ) -> KeepMapReductionExpression {
        Self {
            query_location,
            target,
            selectors: Vec::new(),
        }
    }

    pub fn new_with_selectors(
        query_location: QueryLocation,
        target: MutableValueExpression,
        selectors: Vec<MapSelector>,
    ) -> KeepMapReductionExpression {
        Self {
            query_location,
            target,
            selectors,
        }
    }
}

impl Expression for KeepMapReductionExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

impl MapReductionExpression for KeepMapReductionExpression {
    fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }

    fn get_selectors(&self) -> &Vec<MapSelector> {
        &self.selectors
    }

    fn push_selector(&mut self, selector: MapSelector) {
        self.selectors.push(selector)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveMapReductionExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    selectors: Vec<MapSelector>,
}

impl RemoveMapReductionExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
    ) -> RemoveMapReductionExpression {
        Self {
            query_location,
            target,
            selectors: Vec::new(),
        }
    }
}

impl Expression for RemoveMapReductionExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

impl MapReductionExpression for RemoveMapReductionExpression {
    fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }

    fn get_selectors(&self) -> &Vec<MapSelector> {
        &self.selectors
    }

    fn push_selector(&mut self, selector: MapSelector) {
        self.selectors.push(selector)
    }
}
