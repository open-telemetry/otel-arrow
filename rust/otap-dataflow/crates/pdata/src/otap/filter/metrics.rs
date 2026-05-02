// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metric filtering support for OTAP batches.

use crate::arrays::get_required_array;
use crate::otap::OtapArrowRecords;
use crate::otap::error::{Error, Result};
use crate::otap::filter::{
    IdBitmapPool, MatchType, default_match_type, filter_otap_batch, nulls_to_false,
    regex_match_column,
};
use crate::schema::consts;
use arrow::array::{BooleanArray, StringArray};
use arrow::buffer::BooleanBuffer;
use serde::Deserialize;

/// Overall requirements to use when filtering metrics.
#[derive(Debug, Clone, Deserialize)]
pub struct MetricFilter {
    /// Include match properties describe metrics that should be included in the pipeline.
    /// If both include and exclude are specified, include filtering occurs first.
    include: Option<MetricMatchProperties>,
    /// Exclude match properties describe metrics that should be excluded from the pipeline.
    /// If both include and exclude are specified, include filtering occurs first.
    exclude: Option<MetricMatchProperties>,
}

/// Set of metric properties to match against.
#[derive(Debug, Clone, Deserialize)]
pub struct MetricMatchProperties {
    /// MatchType specifies the type of matching desired.
    #[serde(default = "default_match_type")]
    match_type: MatchType,

    /// MetricNames is a list of metric names that the metric's name field must match against.
    #[serde(default)]
    metric_names: Vec<String>,
}

impl MetricFilter {
    /// Create a new metric filter.
    #[must_use]
    pub const fn new(
        include: Option<MetricMatchProperties>,
        exclude: Option<MetricMatchProperties>,
    ) -> Self {
        Self { include, exclude }
    }

    /// Take a metrics payload and return the filtered result.
    ///
    /// Returns tuple of (filtered batch, metrics_consumed, metrics_filtered).
    pub fn filter(
        &self,
        metrics_payload: OtapArrowRecords,
    ) -> Result<(OtapArrowRecords, u64, u64)> {
        let payload_type = metrics_payload.root_payload_type();
        let metrics = metrics_payload
            .root_record_batch()
            .ok_or_else(|| Error::RecordBatchNotFound { payload_type })?;
        let num_rows = metrics.num_rows() as u64;

        let metric_filter = if let Some(include_config) = &self.include
            && let Some(exclude_config) = &self.exclude
        {
            let include_filter = include_config.create_filter(&metrics_payload, false)?;
            let exclude_filter = exclude_config.create_filter(&metrics_payload, true)?;
            arrow::compute::and_kleene(&include_filter, &exclude_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?
        } else if self.include.is_none()
            && let Some(exclude_config) = &self.exclude
        {
            exclude_config.create_filter(&metrics_payload, true)?
        } else if let Some(include_config) = &self.include
            && self.exclude.is_none()
        {
            include_config.create_filter(&metrics_payload, false)?
        } else {
            return Ok((metrics_payload, num_rows, 0));
        };

        let filtered_count = num_rows - metric_filter.true_count() as u64;
        let mut pool = IdBitmapPool::new();
        let filtered = filter_otap_batch(&metric_filter, &metrics_payload, &mut pool)?;
        Ok((filtered, num_rows, filtered_count))
    }
}

impl MetricMatchProperties {
    /// Create a new metric match properties value.
    #[must_use]
    pub const fn new(match_type: MatchType, metric_names: Vec<String>) -> Self {
        Self {
            match_type,
            metric_names,
        }
    }

    /// Create a filter for the metrics root record batch.
    pub fn create_filter(
        &self,
        metrics_payload: &OtapArrowRecords,
        invert: bool,
    ) -> Result<BooleanArray> {
        let metrics =
            metrics_payload
                .root_record_batch()
                .ok_or_else(|| Error::RecordBatchNotFound {
                    payload_type: metrics_payload.root_payload_type(),
                })?;
        let num_rows = metrics.num_rows();

        if self.metric_names.is_empty() {
            return Ok(BooleanArray::from(BooleanBuffer::new_set(num_rows)));
        }

        let names_column = get_required_array(metrics, consts::NAME)?;
        let mut filter = BooleanArray::from(BooleanBuffer::new_unset(num_rows));
        for name in &self.metric_names {
            let name_filter = match self.match_type {
                MatchType::Regexp => regex_match_column(names_column, name)?,
                MatchType::Strict => {
                    let value_scalar = StringArray::new_scalar(name);
                    arrow::compute::kernels::cmp::eq(&names_column, &value_scalar)
                        .expect("can compare string name column to string scalar")
                }
            };
            filter = arrow::compute::or_kleene(&name_filter, &filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        }

        let filter = nulls_to_false(&filter);
        if invert {
            Ok(arrow::compute::not(&filter).expect("not doesn't fail"))
        } else {
            Ok(filter)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::otap::filter::MatchType;
    use crate::proto::OtlpProtoMessage;
    use crate::proto::opentelemetry::common::v1::InstrumentationScope;
    use crate::proto::opentelemetry::metrics::v1::{
        AggregationTemporality, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
        ScopeMetrics, Sum,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::testing::equiv::assert_equivalent;
    use crate::testing::round_trip::{otap_to_otlp, otlp_to_otap};

    fn build_metrics(names: &[&str]) -> MetricsData {
        build_metrics_with_indices(&names.iter().copied().enumerate().collect::<Vec<_>>())
    }

    fn build_metrics_with_indices(names: &[(usize, &str)]) -> MetricsData {
        MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build()
                    .name("scope".to_string())
                    .finish(),
                names
                    .iter()
                    .map(|&(index, name)| {
                        Metric::build()
                            .name(name)
                            .data_sum(Sum::new(
                                AggregationTemporality::Cumulative,
                                true,
                                vec![
                                    NumberDataPoint::build()
                                        .time_unix_nano(1000u64 + index as u64)
                                        .value_int(index as i64)
                                        .finish(),
                                ],
                            ))
                            .finish()
                    })
                    .collect::<Vec<_>>(),
            )],
        )])
    }

    #[test]
    fn test_filter_include_metric_names() {
        let include = MetricMatchProperties::new(
            MatchType::Strict,
            vec!["test.counter1".into(), "test.counter3".into()],
        );
        let filter = MetricFilter::new(Some(include), None);

        let input = otlp_to_otap(&OtlpProtoMessage::Metrics(build_metrics(&[
            "test.counter1",
            "test.counter2",
            "test.counter3",
        ])));

        let (result, metrics_consumed, metrics_filtered) = filter.filter(input).unwrap();
        assert_eq!(metrics_consumed, 3);
        assert_eq!(metrics_filtered, 1);

        let expected = otlp_to_otap(&OtlpProtoMessage::Metrics(build_metrics_with_indices(&[
            (0, "test.counter1"),
            (2, "test.counter3"),
        ])));

        assert_equivalent(&[otap_to_otlp(&result)], &[otap_to_otlp(&expected)]);
    }

    #[test]
    fn test_filter_exclude_metric_names() {
        let exclude = MetricMatchProperties::new(MatchType::Strict, vec!["test.counter2".into()]);
        let filter = MetricFilter::new(None, Some(exclude));

        let input = otlp_to_otap(&OtlpProtoMessage::Metrics(build_metrics(&[
            "test.counter1",
            "test.counter2",
            "test.counter3",
        ])));

        let (result, metrics_consumed, metrics_filtered) = filter.filter(input).unwrap();
        assert_eq!(metrics_consumed, 3);
        assert_eq!(metrics_filtered, 1);

        let expected = otlp_to_otap(&OtlpProtoMessage::Metrics(build_metrics_with_indices(&[
            (0, "test.counter1"),
            (2, "test.counter3"),
        ])));

        assert_equivalent(&[otap_to_otlp(&result)], &[otap_to_otlp(&expected)]);
    }

    #[test]
    fn test_filter_metric_names_regex() {
        let include = MetricMatchProperties::new(MatchType::Regexp, vec![r"^aio_.*_count$".into()]);
        let filter = MetricFilter::new(Some(include), None);

        let input = otlp_to_otap(&OtlpProtoMessage::Metrics(build_metrics(&[
            "aio_akri_count",
            "aio_akri_latency",
            "other_count",
        ])));

        let (result, metrics_consumed, metrics_filtered) = filter.filter(input).unwrap();
        assert_eq!(metrics_consumed, 3);
        assert_eq!(metrics_filtered, 2);

        let expected = otlp_to_otap(&OtlpProtoMessage::Metrics(build_metrics(&[
            "aio_akri_count",
        ])));

        assert_equivalent(&[otap_to_otlp(&result)], &[otap_to_otlp(&expected)]);
    }
}
