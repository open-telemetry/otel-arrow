// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

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

    fn get_name(&self) -> &'static str {
        match self {
            TransformExpression::Set(_) => "Transform(Set)",
            TransformExpression::Remove(_) => "Transform(Set)",
            TransformExpression::ReduceMap(r) => r.get_name(),
            TransformExpression::RemoveMapKeys(r) => r.get_name(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetTransformExpression {
    query_location: QueryLocation,
    source: ScalarExpression,
    destination: MutableValueExpression,
}

impl SetTransformExpression {
    pub fn new(
        query_location: QueryLocation,
        source: ScalarExpression,
        destination: MutableValueExpression,
    ) -> SetTransformExpression {
        Self {
            query_location,
            source,
            destination,
        }
    }

    pub fn get_source(&self) -> &ScalarExpression {
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

    fn get_name(&self) -> &'static str {
        "SetTransformExpression"
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

    fn get_name(&self) -> &'static str {
        "RemoveTransformExpression"
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

    fn get_name(&self) -> &'static str {
        match self {
            RemoveMapKeysTransformExpression::Remove(_) => "RemoveMapKeysTransform(Remove)",
            RemoveMapKeysTransformExpression::Retain(_) => "RemoveMapKeysTransform(Retain)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapKeyListExpression {
    query_location: QueryLocation,
    target: MutableValueExpression,
    keys: Vec<ScalarExpression>,
}

impl MapKeyListExpression {
    pub fn new(
        query_location: QueryLocation,
        target: MutableValueExpression,
        keys: Vec<ScalarExpression>,
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

    pub fn get_keys(&self) -> &[ScalarExpression] {
        &self.keys
    }
}

impl Expression for MapKeyListExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "MapKeyListExpression"
    }
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

    fn get_name(&self) -> &'static str {
        match self {
            ReduceMapTransformExpression::Remove(_) => "ReduceMapTransform(Remove)",
            ReduceMapTransformExpression::Retain(_) => "ReduceMapTransform(Retain)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapSelector {
    /// A top-level key or key pattern.
    KeyOrKeyPattern(ScalarExpression),

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

    pub fn get_selectors(&self) -> &[MapSelector] {
        &self.selectors
    }

    pub fn push_key_or_key_pattern(&mut self, key_or_key_pattern: ScalarExpression) -> bool {
        if let ScalarExpression::Static(s) = &key_or_key_pattern {
            let value_type = s.get_value_type();
            if value_type != ValueType::String && value_type != ValueType::Regex {
                return false;
            }
        }

        self.selectors
            .push(MapSelector::KeyOrKeyPattern(key_or_key_pattern));

        true
    }

    pub fn push_value_accessor(&mut self, value_accessor: ValueAccessor) -> bool {
        assert!(value_accessor.has_selectors());

        if let ScalarExpression::Static(s) = value_accessor.get_selectors().first().unwrap()
            && s.get_value_type() != ValueType::String
        {
            return false;
        }

        self.selectors
            .push(MapSelector::ValueAccessor(value_accessor));

        true
    }
}

impl Expression for MapSelectionExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "MapSelectionExpression"
    }
}
