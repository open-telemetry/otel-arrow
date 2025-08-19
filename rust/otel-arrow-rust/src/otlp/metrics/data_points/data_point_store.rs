// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
