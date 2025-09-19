// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the predicate struct used to filter metric, log, span signals

use serde::Deserialize;
use serde::Serialize;

use otel_arrow_rust::proto::opentelemetry::{
    common::v1::{AnyValue, KeyValue as KV},
    logs::v1::LogRecord,
    metrics::v1::{Metric, metric::Data},
    trace::v1::Span,
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Predicate {
    field: SignalField,
    value: MatchValue,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalField {
    Attribute,
}

// ToDo: Add bytes
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MatchValue {
    String(String),
    Int(i64),
    Double(f64),
    Boolean(bool),
    Array(Vec<MatchValue>),
    KeyValue(Vec<KeyValue>),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct KeyValue {
    key: String,
    value: MatchValue,
}

impl KeyValue {
    pub fn new(key: String, value: MatchValue) -> Self {
        Self { key, value }
    }
}

impl Predicate {
    fn map_anyvalue(&self, match_value: MatchValue) -> AnyValue {
        // get the anyvalue type
        match match_value {
            MatchValue::String(value) => AnyValue::new_string(value),
            MatchValue::Int(value) => AnyValue::new_int(value),
            MatchValue::Double(value) => AnyValue::new_double(value),
            MatchValue::Boolean(value) => AnyValue::new_double(value),
            MatchValue::KeyValue(value) => AnyValue::new_kvlist(self.map_keyvalue(value)),
            MatchValue::Array(value) => AnyValue::new_array(
                value
                    .iter()
                    .map(|match_value| self.map_anyvalue(match_value.clone()))
                    .collect::<Vec<AnyValue>>(),
            ),
        }
    }

    fn map_keyvalue(&self, key_value: Vec<KeyValue>) -> Vec<KV> {
        // map KeyValue to proto definition KV
        key_value
            .iter()
            .map(|kv| KV::new(kv.key.clone(), self.map_anyvalue(kv.value.clone())))
            .collect()
    }

    pub fn new(field: SignalField, value: MatchValue) -> Self {
        Self { field, value }
    }

    pub fn match_log(&self, log_record: &LogRecord) -> bool {
        // check if signal matches preicate defenition here
        match self.field {
            SignalField::Attribute => self.check_attributes(log_record.attributes.clone()),
        }
    }

    pub fn match_metric(&self, metric: &Metric) -> bool {
        let metric_data = &metric.data;
        match self.field {
            SignalField::Attribute => {
                if let Some(data) = metric_data {
                    match data {
                        // check attributes from each datapoint
                        // filter on datapoints instead of metric signal for metric pdata type?
                        Data::Gauge(gauge) => gauge.data_points.iter().any(|data_points| {
                            self.check_attributes(data_points.attributes.clone())
                        }),
                        Data::Sum(sum) => sum.data_points.iter().any(|data_points| {
                            self.check_attributes(data_points.attributes.clone())
                        }),
                        Data::Histogram(histogram) => {
                            histogram.data_points.iter().any(|data_points| {
                                self.check_attributes(data_points.attributes.clone())
                            })
                        }
                        Data::ExponentialHistogram(exp_histogram) => {
                            exp_histogram.data_points.iter().any(|data_points| {
                                self.check_attributes(data_points.attributes.clone())
                            })
                        }
                        Data::Summary(summary) => summary.data_points.iter().any(|data_points| {
                            self.check_attributes(data_points.attributes.clone())
                        }),
                    }
                } else {
                    // no data to check attributes for
                    false
                }
            }
        }
    }

    pub fn match_span(&self, span: &Span) -> bool {
        match self.field {
            SignalField::Attribute => self.check_attributes(span.attributes.clone()),
        }
    }

    fn check_attributes(&self, attributes: Vec<KV>) -> bool {
        match &self.value {
            MatchValue::KeyValue(value) => {
                let key_values = self.map_keyvalue(value.clone());
                key_values.iter().any(|kv| attributes.contains(kv))
            }
            _ => false, // we expect a key value to match against
        }
    }
}
