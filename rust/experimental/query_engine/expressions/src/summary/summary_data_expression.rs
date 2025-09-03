// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SummaryDataExpression {
    query_location: QueryLocation,
    group_by_expressions: Vec<(Box<str>, ScalarExpression)>,
    aggregation_expressions: Vec<(Box<str>, AggregationExpression)>,
    post_expressions: Vec<DataExpression>,
}

impl SummaryDataExpression {
    pub fn new(
        query_location: QueryLocation,
        mut group_by_expressions: HashMap<Box<str>, ScalarExpression>,
        mut aggregation_expressions: HashMap<Box<str>, AggregationExpression>,
    ) -> SummaryDataExpression {
        let mut group_by_expressions = Vec::from_iter(group_by_expressions.drain());
        group_by_expressions.sort_by(|l, r| l.0.cmp(&r.0));

        let mut aggregation_expressions = Vec::from_iter(aggregation_expressions.drain());
        aggregation_expressions.sort_by(|l, r| l.0.cmp(&r.0));

        Self {
            query_location,
            group_by_expressions,
            aggregation_expressions,
            post_expressions: Vec::new(),
        }
    }

    pub fn with_post_expressions(mut self, expressions: Vec<DataExpression>) -> Self {
        self.post_expressions = expressions;
        self
    }

    pub fn get_group_by_expressions(&self) -> &[(Box<str>, ScalarExpression)] {
        &self.group_by_expressions
    }

    pub fn get_aggregation_expressions(&self) -> &[(Box<str>, AggregationExpression)] {
        &self.aggregation_expressions
    }

    pub fn get_post_expressions(&self) -> &[DataExpression] {
        &self.post_expressions
    }

    pub fn push_post_expression(&mut self, expression: DataExpression) {
        self.post_expressions.push(expression);
    }
}

impl Expression for SummaryDataExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "SummaryDataExpression"
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
