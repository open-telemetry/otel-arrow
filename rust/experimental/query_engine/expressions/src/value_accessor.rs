use crate::{IntegerScalarExpression, ScalarExpression, StringScalarExpression};

/// Contains the rules used to resolve data from a target
///
/// Notes:
///
/// * Given a target such as `source` and selectors `MapKey('SubItem')`,
///   `ArrayIndex(0)` evaluation would be equivalent to: `source.SubItem[0]`.
/// * An empty set of selectors resolves the initial target.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueAccessor {
    selectors: Vec<ValueSelector>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueSelector {
    /// Resolve a value from an array using a static 32bit integer provided
    /// directly in a query.
    ///
    /// Note: Negative values indicate access from the end of the array. For
    /// example `-1` will select the last item.
    ArrayIndex(IntegerScalarExpression),

    /// Resolve a value from a map using a static string provided directly in a
    /// query as the key.
    ///
    /// Note: Keys are case-sensitive.
    MapKey(StringScalarExpression),

    /// Resolve a value using a [`ScalarExpression`].
    ///
    /// Notes:
    /// * If the [`ScalarExpression`] returns an integer an array index
    ///   operation will be attempted. If the integer value is negative then the
    ///   array index will be performed from the end of the array.
    /// * If the [`ScalarExpression`] returns a string a map key operation will
    ///   be attempted.
    /// * If any other value type is returned by the [`ScalarExpression`] then
    ///   no data will be resolved.
    ScalarExpression(ScalarExpression),
}

impl ValueAccessor {
    pub fn new() -> ValueAccessor {
        Self {
            selectors: Vec::new(),
        }
    }

    pub fn get_selectors(&self) -> &Vec<ValueSelector> {
        &self.selectors
    }

    pub fn insert_selector(&mut self, index: usize, selector: ValueSelector) {
        self.selectors.insert(index, selector);
    }

    pub fn push_selector(&mut self, selector: ValueSelector) {
        self.selectors.push(selector)
    }
}

impl Default for ValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}
