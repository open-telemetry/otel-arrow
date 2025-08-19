// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::opentelemetry::metrics::v1::{
    ExponentialHistogramDataPoint, HistogramDataPoint, NumberDataPoint, SummaryDataPoint,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct DataPointStore<T> {
    //todo: looks like this field is also unused in otel-arrow: https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/metrics/otlp/number_data_point.go#L40
    #[allow(dead_code)]
    next_id: u16,
    data_point_by_id: HashMap<u16, Vec<T>>,
}

impl<T> DataPointStore<T>
where
    T: Default,
{
    pub fn get_or_default(&mut self, key: u16) -> &mut Vec<T> {
        self.data_point_by_id.entry(key).or_default()
    }
}

pub type NumberDataPointsStore = DataPointStore<NumberDataPoint>;
pub type SummaryDataPointsStore = DataPointStore<SummaryDataPoint>;
pub type HistogramDataPointsStore = DataPointStore<HistogramDataPoint>;
pub type EHistogramDataPointsStore = DataPointStore<ExponentialHistogramDataPoint>;
