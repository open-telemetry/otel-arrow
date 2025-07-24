use crate::ScalarExpression;

/// Contains the rules used to resolve data from a target
///
/// Notes:
///
/// * Given a target such as `source` and selectors `String('SubItem')`,
///   `Integer(0)` evaluation would be equivalent to: `source.SubItem[0]`.
/// * An empty set of selectors resolves the initial target.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueAccessor {
    selectors: Vec<ScalarExpression>,
}

impl ValueAccessor {
    pub fn new() -> ValueAccessor {
        Self {
            selectors: Vec::new(),
        }
    }

    pub fn new_with_selectors(selectors: Vec<ScalarExpression>) -> ValueAccessor {
        let mut accessor = ValueAccessor::new();

        for selector in selectors {
            accessor.push_selector(selector);
        }

        accessor
    }

    pub fn has_selectors(&self) -> bool {
        !self.selectors.is_empty()
    }

    pub fn get_selectors(&self) -> &Vec<ScalarExpression> {
        &self.selectors
    }

    pub fn insert_selector(&mut self, index: usize, selector: ScalarExpression) {
        self.selectors.insert(index, selector);
    }

    pub fn push_selector(&mut self, selector: ScalarExpression) {
        self.selectors.push(selector)
    }

    pub fn remove_selector(&mut self, index: usize) -> Option<ScalarExpression> {
        if index >= self.selectors.len() {
            return None;
        }

        Some(self.selectors.remove(index))
    }
}

impl Default for ValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}
