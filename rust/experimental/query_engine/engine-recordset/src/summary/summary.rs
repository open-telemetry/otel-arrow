use std::{cell::RefCell, collections::HashMap};

use data_engine_expressions::*;
use sha2::{Digest, Sha256};

use crate::*;

pub(crate) struct Summaries {
    lookup: RefCell<HashMap<Box<str>, usize>>,
    summary_data: RefCell<Vec<Summary>>,
}

impl Summaries {
    pub fn new() -> Summaries {
        Self {
            lookup: RefCell::new(HashMap::new()),
            summary_data: RefCell::new(Vec::new()),
        }
    }

    pub fn create_or_update_summary(
        &self,
        mut group_by_values: HashMap<Box<str>, ResolvedValue>,
        aggregation_values: HashMap<Box<str>, SummaryAggregation>,
    ) -> bool {
        let summary_id = Summary::generate_id(&group_by_values);

        let mut lookup = self.lookup.borrow_mut();

        let summary_index = lookup.get_mut(&summary_id);

        if summary_index.is_some() {
            let index = summary_index.unwrap();

            let mut summary_data = self.summary_data.borrow_mut();

            let summary = summary_data
                .get_mut(*index)
                .expect("Summary could not be found");

            summary.update(&aggregation_values);

            return false;
        }

        let mut summary_data = self.summary_data.borrow_mut();

        let summary = Summary::new(
            HashMap::from_iter(group_by_values.drain().map(|(k, v)| (k, v.into()))),
            aggregation_values,
        );

        let index = summary_data.len();

        let old = lookup.insert(summary_id, index);

        debug_assert!(old.is_none());

        summary_data.push(summary);

        true
    }
}

#[derive(Debug)]
pub struct Summary {
    group_by_values: HashMap<Box<str>, OwnedValue>,
    aggregation_values: HashMap<Box<str>, SummaryAggregation>,
}

impl Summary {
    pub fn new(
        group_by_values: HashMap<Box<str>, OwnedValue>,
        aggregation_values: HashMap<Box<str>, SummaryAggregation>,
    ) -> Summary {
        Self {
            group_by_values,
            aggregation_values,
        }
    }

    pub(crate) fn update(&mut self, aggregation_values: &HashMap<Box<str>, SummaryAggregation>) {
        for (key, aggregation) in aggregation_values {}
    }

    pub(crate) fn generate_id(group_by_values: &HashMap<Box<str>, ResolvedValue>) -> Box<str> {
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

        hex::encode(bytes).into()
    }
}

#[derive(Debug)]
pub enum SummaryAggregation {
    Average { count: usize, sum: SummaryValue },
    Count { value: usize },
    Maximum(OwnedValue),
    Minimum(OwnedValue),
    Sum(SummaryValue),
}

#[derive(Debug)]
pub enum SummaryValue {
    Double(f64),
    Integer(i64),
}
