// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

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

    pub fn get_selectors(&self) -> &[ScalarExpression] {
        &self.selectors
    }

    pub fn get_selectors_mut(&mut self) -> &mut [ScalarExpression] {
        &mut self.selectors
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

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        for selector in &mut self.selectors {
            selector.try_resolve_static(scope)?;
        }
        Ok(())
    }

    pub(crate) fn try_resolve_static<'a>(
        &'a mut self,
        root: &'a StaticScalarExpression,
        scope: &PipelineResolutionScope<'a>,
    ) -> Result<Option<&'a StaticScalarExpression>, ExpressionError> {
        let mut s: Vec<ScalarStaticResolutionResult> = self
            .selectors
            .iter_mut()
            .map(|s| s.try_resolve_static(scope))
            .collect();

        Self::select_from_value(root, &mut s.drain(..))
    }

    pub(crate) fn fmt_with_indent(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
    ) -> std::fmt::Result {
        if self.selectors.is_empty() {
            writeln!(f, "None")?;
        } else {
            writeln!(f)?;
            let last_idx = self.selectors.len() - 1;
            for (i, e) in self.selectors.iter().enumerate() {
                if i == last_idx {
                    write!(f, "{indent}└── ")?;
                    e.fmt_with_indent(f, format!("{indent}    ").as_str())?;
                } else {
                    write!(f, "{indent}├── ")?;
                    e.fmt_with_indent(f, format!("{indent}│   ").as_str())?;
                }
            }
        }
        Ok(())
    }

    fn select_from_value<'a>(
        root: &'a StaticScalarExpression,
        selectors: &mut std::vec::Drain<ScalarStaticResolutionResult<'a>>,
    ) -> Result<Option<&'a StaticScalarExpression>, ExpressionError> {
        match selectors.next() {
            Some(s) => {
                match s? {
                    None => Ok(None),
                    Some(value) => {
                        let scalar = value.as_ref();

                        let next = match scalar {
                            StaticScalarExpression::String(map_key) => {
                                if let StaticScalarExpression::Map(m) = root {
                                    m.get_values().get(map_key.get_value())
                                } else {
                                    return Err(ExpressionError::ValidationFailure(
                                        scalar.get_query_location().clone(),
                                        format!(
                                            "Could not search for map key '{}' specified in accessor expression because current node is a '{:?}' value",
                                            map_key.get_value(),
                                            root.get_value_type()
                                        ),
                                    ));
                                }
                            }
                            StaticScalarExpression::Integer(array_index) => {
                                if let StaticScalarExpression::Array(a) = root {
                                    let mut index = array_index.get_value();
                                    if index < 0 {
                                        index += a.len() as i64;
                                    }
                                    if index < 0 {
                                        return Err(ExpressionError::ValidationFailure(
                                            scalar.get_query_location().clone(),
                                            format!(
                                                "Array index '{index}' specified in accessor expression is invalid"
                                            ),
                                        ));
                                    } else {
                                        a.get_values().get(index as usize)
                                    }
                                } else {
                                    return Err(ExpressionError::ValidationFailure(
                                        scalar.get_query_location().clone(),
                                        format!(
                                            "Could not search for array index '{}' specified in accessor expression because current node is a '{:?}' value",
                                            array_index.get_value(),
                                            root.get_value_type()
                                        ),
                                    ));
                                }
                            }
                            _ => {
                                return Err(ExpressionError::ValidationFailure(scalar.get_query_location().clone(), "Unexpected scalar expression encountered in accessor expression".into()));
                            }
                        };

                        if let Some(v) = next {
                            Self::select_from_value(v, selectors)
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            None => Ok(Some(root)),
        }
    }
}

impl Default for ValueAccessor {
    fn default() -> Self {
        Self::new()
    }
}
