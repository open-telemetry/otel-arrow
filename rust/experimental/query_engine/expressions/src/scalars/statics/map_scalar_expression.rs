// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct MapScalarExpression {
    query_location: QueryLocation,
    values: HashMap<Box<str>, StaticScalarExpression>,
}

impl MapScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        value: HashMap<Box<str>, StaticScalarExpression>,
    ) -> MapScalarExpression {
        Self {
            query_location,
            values: value,
        }
    }

    pub fn get_values(&self) -> &HashMap<Box<str>, StaticScalarExpression> {
        &self.values
    }
}

impl Expression for MapScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "MapScalarExpression"
    }
}

impl MapValue for MapScalarExpression {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    fn get(&self, key: &str) -> Option<&(dyn AsStaticValue + 'static)> {
        self.values.get(key).map(|v| v as &dyn AsStaticValue)
    }

    fn get_items(&self, item_callback: &mut dyn KeyValueCallback) -> bool {
        for (key, value) in &self.values {
            if !item_callback.next(key, value.to_value()) {
                return false;
            }
        }

        true
    }
}
