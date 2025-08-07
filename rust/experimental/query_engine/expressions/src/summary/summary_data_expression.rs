use std::collections::HashMap;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SummaryDataExpression {
    /// A summary which emits as a record with all individual summarized records dropped.
    Flatten(SummaryExpression),
}

impl Expression for SummaryDataExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            SummaryDataExpression::Flatten(f) => f.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            SummaryDataExpression::Flatten(_) => "SummaryDataExpression(Flatten)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SummaryExpression {
    query_location: QueryLocation,
    aggregation_expressions: HashMap<Box<str>, AggregationExpression>,
    group_by_expressions: HashMap<Box<str>, ScalarExpression>,
}

impl SummaryExpression {
    pub fn new(
        query_location: QueryLocation,
        aggregation_expressions: HashMap<Box<str>, AggregationExpression>,
        group_by_expressions: HashMap<Box<str>, ScalarExpression>,
    ) -> SummaryExpression {
        Self {
            query_location,
            aggregation_expressions,
            group_by_expressions,
        }
    }

    pub fn get_aggregation_expressions(&self) -> &HashMap<Box<str>, AggregationExpression> {
        &self.aggregation_expressions
    }

    pub fn get_group_by_expressions(&self) -> &HashMap<Box<str>, ScalarExpression> {
        &self.group_by_expressions
    }
}

impl Expression for SummaryExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "SummaryExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AggregationExpression {
    query_location: QueryLocation,
    aggregation_function: AggregationFunction,
    value_expression: Option<ScalarExpression>,
}

impl AggregationExpression {
    pub fn new(
        query_location: QueryLocation,
        aggregation_function: AggregationFunction,
        value_expression: Option<ScalarExpression>,
    ) -> AggregationExpression {
        Self {
            query_location,
            aggregation_function,
            value_expression,
        }
    }

    pub fn get_aggregation_function(&self) -> AggregationFunction {
        self.aggregation_function.clone()
    }

    pub fn get_value_expression(&self) -> &Option<ScalarExpression> {
        &self.value_expression
    }
}

impl Expression for AggregationExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "AggregationExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregationFunction {
    Average,
    Count,
    Maximum,
    Minimum,
    Sum,
}
