// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Expression, LogicalExpression, QueryLocation, SummaryDataExpression, TransformExpression,
};

#[derive(Debug, Clone, PartialEq)]
pub enum DataExpression {
    /// Discard data expression.
    Discard(DiscardDataExpression),

    /// Summary data expression.
    Summary(SummaryDataExpression),

    /// Transform data expression.
    Transform(TransformExpression),
}

impl Expression for DataExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            DataExpression::Discard(d) => d.get_query_location(),
            DataExpression::Summary(s) => s.get_query_location(),
            DataExpression::Transform(t) => t.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            DataExpression::Discard(_) => "DataExpression(Discard)",
            DataExpression::Summary(_) => "DataExpression(Summary)",
            DataExpression::Transform(_) => "DataExpression(Transform)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscardDataExpression {
    query_location: QueryLocation,
    predicate: Option<LogicalExpression>,
}

impl DiscardDataExpression {
    pub fn new(query_location: QueryLocation) -> DiscardDataExpression {
        Self {
            query_location,
            predicate: None,
        }
    }

    pub fn with_predicate(mut self, predicate: LogicalExpression) -> DiscardDataExpression {
        self.predicate = Some(predicate);

        self
    }

    pub fn get_predicate(&self) -> Option<&LogicalExpression> {
        self.predicate.as_ref()
    }
}

impl Expression for DiscardDataExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "DiscardDataExpression"
    }
}
