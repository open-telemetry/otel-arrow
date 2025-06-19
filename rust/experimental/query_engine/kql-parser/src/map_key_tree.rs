use std::collections::HashMap;

use data_engine_expressions::*;

pub(crate) struct MapKeyTree {
    children: HashMap<Box<str>, MapKeyTree>,
}

impl MapKeyTree {
    pub fn new() -> MapKeyTree {
        Self {
            children: HashMap::new(),
        }
    }

    pub fn push_selectors(
        &mut self,
        default_source_map_key: Option<&str>,
        selectors: &[ValueSelector],
    ) -> bool {
        if selectors.is_empty() {
            return false;
        }

        self.push_selectors_recursive(default_source_map_key, &mut selectors.iter())
    }

    pub fn build_clear_keys_expressions(
        &mut self,
        query_location: &QueryLocation,
        expressions: &mut Vec<TransformExpression>,
    ) {
        self.build_clear_keys_expressions_recursive(
            query_location,
            expressions,
            ValueAccessor::new(),
        );
    }

    fn push_selectors_recursive(
        &mut self,
        default_source_map_key: Option<&str>,
        selectors: &mut std::slice::Iter<ValueSelector>,
    ) -> bool {
        if let Some(selector) = selectors.next() {
            if let ValueSelector::MapKey(k) = selector {
                let key = k.get_value();
                if Some(key) == default_source_map_key {
                    // Note: If the root key is the default_source_map_key value
                    // it is stripped away. Given some accessor list like key1,
                    // source.attributes[key2], attributes[key3] when
                    // default_source_map_key=attributes we want the root key
                    // list to be: key1, key2, key3.
                    return self.push_selectors_recursive(None, selectors);
                }
                let child = self.children.entry(key.into()).or_insert(MapKeyTree::new());
                return child.push_selectors_recursive(None, selectors);
            } else {
                return false;
            }
        }

        true
    }

    fn build_clear_keys_expressions_recursive(
        &mut self,
        query_location: &QueryLocation,
        expressions: &mut Vec<TransformExpression>,
        value_accessor: ValueAccessor,
    ) {
        if self.children.is_empty() {
            return;
        }

        let keys_to_keep = self
            .children
            .keys()
            .map(|v| SourceKey::Identifier(StringScalarExpression::new(query_location.clone(), v)))
            .collect();

        expressions.push(TransformExpression::ClearKeys(
            ClearKeysTransformExpression::new(
                query_location.clone(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    query_location.clone(),
                    value_accessor.clone(),
                )),
                keys_to_keep,
            ),
        ));

        for (name, child) in &mut self.children {
            let mut selectors = value_accessor.get_selectors().clone();
            selectors.push(ValueSelector::MapKey(StringScalarExpression::new(
                query_location.clone(),
                name,
            )));
            child.build_clear_keys_expressions_recursive(
                query_location,
                expressions,
                ValueAccessor::new_with_selectors(selectors),
            );
        }
    }
}
