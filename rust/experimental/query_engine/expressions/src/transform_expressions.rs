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
    keys_to_remove: HashSet<MapKey>,
}

impl RemoveKeysTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
        keys_to_remove: HashSet<MapKey>,
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

    pub fn get_keys_to_remove(&self) -> &HashSet<MapKey> {
        &self.keys_to_remove
    }
}

impl Expression for RemoveKeysTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum MapKey {
    Pattern(StringScalarExpression),
    Value(StringScalarExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReduceMapTransformExpression {
    /// A map reduction providing the data to keep. All other data is removed.
    Keep(MapSelectionExpression),

    /// A map reduction providing the data to remove. All other data is retained.
    Remove(MapSelectionExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapSelector {
    /// Top-level key pattern.
    KeyPattern(StringScalarExpression),

    /// Static top-level key.
    Key(StringScalarExpression),

    /// A [`ValueAccessor`] representing a path to data on the map.
    ///
    /// Note: The [`ValueAccessor`] could refer to top-level keys or nested
    /// elements.
    ValueAccessor(ValueAccessor),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapSelectionExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    selectors: Vec<MapSelector>,
}

impl MapSelectionExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
    ) -> MapSelectionExpression {
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
    ) -> MapSelectionExpression {
        Self {
            query_location,
            target,
            selectors,
        }
    }

    pub fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }

    pub fn get_selectors(&self) -> &Vec<MapSelector> {
        &self.selectors
    }

    pub fn push_selector(&mut self, selector: MapSelector) {
        self.selectors.push(selector)
    }

    pub fn push_value_accessor(&mut self, value_accessor: &ValueAccessor) -> bool {
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

impl Expression for MapSelectionExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}
