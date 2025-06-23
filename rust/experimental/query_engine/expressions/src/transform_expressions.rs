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

    /// Remove data from a target map.
    ReduceMap(ReduceMapTransformExpression),

    /// Remove top-level keys from a target map.
    ///
    /// Note: Remove map keys is a specialized form of the reduce map
    /// transformation which only operates on top-level keys.
    RemoveMapKeys(RemoveMapKeysTransformExpression),
}

impl Expression for TransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            TransformExpression::Set(s) => s.get_query_location(),
            TransformExpression::Remove(r) => r.get_query_location(),
            TransformExpression::ReduceMap(r) => r.get_query_location(),
            TransformExpression::RemoveMapKeys(r) => r.get_query_location(),
        }
    }
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
pub enum RemoveMapKeysTransformExpression {
    /// A map key transformation providing the top-level keys to remove. All other data is retained.
    Remove(MapKeyListExpression),

    /// A map key transformation providing the top-level keys to retain. All other data is removed.
    Retain(MapKeyListExpression),
}

impl Expression for RemoveMapKeysTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            RemoveMapKeysTransformExpression::Remove(m) => m.get_query_location(),
            RemoveMapKeysTransformExpression::Retain(m) => m.get_query_location(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapKeyListExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    keys: HashSet<MapKey>,
}

impl MapKeyListExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
        keys: HashSet<MapKey>,
    ) -> MapKeyListExpression {
        Self {
            query_location,
            target,
            keys,
        }
    }

    pub fn get_target(&self) -> &MutableValueExpression {
        &self.target
    }

    pub fn get_keys(&self) -> &HashSet<MapKey> {
        &self.keys
    }
}

impl Expression for MapKeyListExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum MapKey {
    /// A pattern used to resolve keys.
    ///
    /// Examples: `name*`, `*`, `*_value`
    Pattern(StringScalarExpression),

    /// A static key value.
    Value(StringScalarExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReduceMapTransformExpression {
    /// A map reduction providing the data to remove. All other data is retained.
    Remove(MapSelectionExpression),

    /// A map reduction providing the data to retain. All other data is removed.
    Retain(MapSelectionExpression),
}

impl Expression for ReduceMapTransformExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            ReduceMapTransformExpression::Remove(m) => m.get_query_location(),
            ReduceMapTransformExpression::Retain(m) => m.get_query_location(),
        }
    }
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
