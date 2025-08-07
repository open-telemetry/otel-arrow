use std::{cell::RefCell, collections::HashMap};

use data_engine_expressions::*;
use sha2::{Digest, Sha256};

use crate::{execution_context::ExecutionContext, *};

#[derive(Debug)]
pub(crate) struct Summaries {
    cardinality_limit: usize,
    pub values: RefCell<HashMap<String, Summary>>,
}

impl Summaries {
    pub fn new(cardinality_limit: usize) -> Summaries {
        Self {
            cardinality_limit,
            values: RefCell::new(HashMap::new()),
        }
    }

    pub fn create_or_update_summary<'a, T: Record>(
        &self,
        execution_context: &ExecutionContext<'a, '_, '_, T>,
        summary_data_expression: &'a SummaryDataExpression,
        mut group_by_values: Vec<(Box<str>, ResolvedValue)>,
        mut aggregation_values: HashMap<Box<str>, SummaryAggregationUpdate>,
    ) {
        let summary_id = Summary::generate_id(&group_by_values);

        let mut values = self.values.borrow_mut();

        let number_of_summaries = values.len();

        let summary_data = values.get_mut(&summary_id);

        if summary_data.is_some() {
            let summary = summary_data.expect("Summary could not be found");

            summary.update(
                execution_context,
                summary_data_expression,
                aggregation_values,
            );

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                summary_data_expression,
                || "Summary updated".into(),
            );

            return;
        }

        if number_of_summaries >= self.cardinality_limit {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Error,
                summary_data_expression,
                || "Summary cardinality limit reached".into(),
            );
            return;
        }

        let summary = Summary::new(
            Vec::from_iter(group_by_values.drain(..).map(|(k, v)| (k, v.into()))),
            HashMap::from_iter(aggregation_values.drain().map(|(k, v)| (k, v.into()))),
        );

        let old = values.insert(summary_id, summary);

        debug_assert!(old.is_none());

        execution_context.add_diagnostic_if_enabled(
            RecordSetEngineDiagnosticLevel::Verbose,
            summary_data_expression,
            || "Summary created".into(),
        );
    }
}

#[derive(Debug)]
pub(crate) struct Summary {
    pub group_by_values: Vec<(Box<str>, OwnedValue)>,
    pub aggregation_values: HashMap<Box<str>, SummaryAggregation>,
}

impl Summary {
    pub fn new(
        group_by_values: Vec<(Box<str>, OwnedValue)>,
        aggregation_values: HashMap<Box<str>, SummaryAggregation>,
    ) -> Summary {
        Self {
            group_by_values,
            aggregation_values,
        }
    }

    pub(crate) fn update<'a, T: Record>(
        &mut self,
        execution_context: &ExecutionContext<'a, '_, '_, T>,
        summary_data_expression: &'a SummaryDataExpression,
        aggregation_values: HashMap<Box<str>, SummaryAggregationUpdate>,
    ) {
        for (key, aggregation) in aggregation_values {
            let existing = self.aggregation_values.get_mut(&key);
            if existing.is_none() {
                self.aggregation_values.insert(key, aggregation.into());
                continue;
            }

            match existing.unwrap() {
                SummaryAggregation::Average { count, sum } => {
                    if let SummaryAggregationUpdate::Average(v) = aggregation {
                        *count += 1;
                        match sum {
                            SummaryValue::Double(d) => *d += v.to_double(),
                            SummaryValue::Integer(i) => *i += v.to_integer(),
                        }
                    } else {
                        panic!("Aggregation update didn't match")
                    }
                }
                SummaryAggregation::Count(count) => {
                    if let SummaryAggregationUpdate::Count = aggregation {
                        *count += 1;
                    } else {
                        panic!("Aggregation update didn't match")
                    }
                }
                SummaryAggregation::Maximum(max) => {
                    if let SummaryAggregationUpdate::Maximum(v) = aggregation {
                        let right_value = v.to_value();
                        match Value::compare_values(
                            summary_data_expression.get_query_location(),
                            &max.to_value(),
                            &right_value,
                        ) {
                            Ok(r) => {
                                if r < 0 {
                                    *max = right_value.into()
                                }
                            }
                            Err(_) => {
                                execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Warn,
                                    summary_data_expression,
                                    || format!("Max aggregation cannot compare values of '{:?}' and '{:?}' types", max.get_value_type(), right_value.get_value_type()));
                            }
                        }
                    } else {
                        panic!("Aggregation update didn't match")
                    }
                }
                SummaryAggregation::Minimum(min) => {
                    if let SummaryAggregationUpdate::Minimum(v) = aggregation {
                        let right_value = v.to_value();
                        match Value::compare_values(
                            summary_data_expression.get_query_location(),
                            &min.to_value(),
                            &right_value,
                        ) {
                            Ok(r) => {
                                if r > 0 {
                                    *min = right_value.into()
                                }
                            }
                            Err(_) => {
                                execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Warn,
                                    summary_data_expression,
                                    || format!("Min aggregation cannot compare values of '{:?}' and '{:?}' types", min.get_value_type(), right_value.get_value_type()));
                            }
                        }
                    } else {
                        panic!("Aggregation update didn't match")
                    }
                }
                SummaryAggregation::Sum(sum) => {
                    if let SummaryAggregationUpdate::Sum(v) = aggregation {
                        match sum {
                            SummaryValue::Double(d) => *d += v.to_double(),
                            SummaryValue::Integer(i) => *i += v.to_integer(),
                        }
                    } else {
                        panic!("Aggregation update didn't match")
                    }
                }
            }
        }
    }

    pub(crate) fn generate_id(group_by_values: &Vec<(Box<str>, ResolvedValue)>) -> String {
        let mut hasher = Sha256::new();

        for (key, value) in group_by_values {
            hasher.update(key.as_bytes());

            let v = value.to_value();
            match v {
                Value::String(s) => {
                    hasher.update([0]);
                    hasher.update(s.get_value().as_bytes());
                }
                Value::Integer(l) => {
                    hasher.update([1]);
                    hasher.update(l.get_value().to_le_bytes());
                }
                Value::Double(d) => {
                    hasher.update([2]);
                    hasher.update(d.get_value().to_le_bytes());
                }
                Value::Null => hasher.update([3]),
                _ => {
                    v.convert_to_string(&mut |s| {
                        hasher.update([4]);
                        hasher.update(s.as_bytes());
                    });
                }
            }
        }

        let hash = hasher.finalize();

        let bytes = &hash[..];

        hex::encode(bytes)
    }
}

pub(crate) enum SummaryAggregationUpdate<'a> {
    Average(SummaryValue),
    Count,
    Maximum(ResolvedValue<'a>),
    Minimum(ResolvedValue<'a>),
    Sum(SummaryValue),
}

#[derive(Debug)]
pub enum SummaryAggregation {
    Average { count: usize, sum: SummaryValue },
    Count(usize),
    Maximum(OwnedValue),
    Minimum(OwnedValue),
    Sum(SummaryValue),
}

impl From<SummaryAggregationUpdate<'_>> for SummaryAggregation {
    fn from(value: SummaryAggregationUpdate) -> Self {
        match value {
            SummaryAggregationUpdate::Average(v) => {
                SummaryAggregation::Average { count: 1, sum: v }
            }
            SummaryAggregationUpdate::Count => SummaryAggregation::Count(1),
            SummaryAggregationUpdate::Maximum(v) => SummaryAggregation::Maximum(v.into()),
            SummaryAggregationUpdate::Minimum(v) => SummaryAggregation::Minimum(v.into()),
            SummaryAggregationUpdate::Sum(v) => SummaryAggregation::Sum(v),
        }
    }
}

#[derive(Debug)]
pub enum SummaryValue {
    Double(f64),
    Integer(i64),
}

impl SummaryValue {
    pub fn to_double(&self) -> f64 {
        match self {
            SummaryValue::Double(d) => *d,
            SummaryValue::Integer(i) => *i as f64,
        }
    }

    pub fn to_integer(&self) -> i64 {
        match self {
            SummaryValue::Double(d) => *d as i64,
            SummaryValue::Integer(i) => *i,
        }
    }
}
