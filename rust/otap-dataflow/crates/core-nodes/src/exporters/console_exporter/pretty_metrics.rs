// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Human-readable metrics formatting for the console exporter.

use super::{HierarchicalFormatter, pretty_writer::PrettyWriter};
use otap_df_pdata_views::views::common::{AttributeView, InstrumentationScopeView};
use otap_df_pdata_views::views::metrics::{
    AggregationTemporality, BucketsView, DataType, DataView, ExemplarView,
    ExponentialHistogramDataPointView, ExponentialHistogramView, GaugeView, HistogramDataPointView,
    HistogramView, MetricView, MetricsView, NumberDataPointView, ResourceMetricsView,
    ScopeMetricsView, SumView, SummaryDataPointView, SummaryView, Value, ValueAtQuantileView,
};
use otap_df_pdata_views::views::resource::ResourceView;
use otap_df_telemetry::self_tracing::AnsiCode;
use std::io::{self, Write};

type MetricsWriter<'a> = PrettyWriter<'a, dyn Write + 'a>;

impl HierarchicalFormatter {
    /// Format metrics from a generic metrics view.
    pub(super) fn format_metrics_data_to<M: MetricsView, W: Write>(
        &self,
        metrics_data: &M,
        output: &mut W,
    ) -> io::Result<()> {
        for resource_metrics in metrics_data.resources() {
            self.format_resource_metrics(&resource_metrics, output)?;
        }
        Ok(())
    }

    fn format_resource_metrics<R: ResourceMetricsView>(
        &self,
        resource_metrics: &R,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        let schema_url = resource_metrics.schema_url();
        self.format_attribute_line(
            0,
            "RESOURCE",
            resource_metrics
                .resource()
                .iter()
                .flat_map(|resource| resource.attributes()),
            output,
            |w| write_optional_bytes(w, "schema_url", schema_url),
        )?;

        for scope_metrics in resource_metrics.scopes() {
            self.format_scope_metrics(&scope_metrics, output)?;
        }
        Ok(())
    }

    fn format_scope_metrics<S: ScopeMetricsView>(
        &self,
        scope_metrics: &S,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        let scope = scope_metrics.scope();
        let name = scope.as_ref().and_then(|scope| scope.name());
        let version = scope.as_ref().and_then(|scope| scope.version());
        let schema_url = scope_metrics.schema_url();
        self.format_attribute_line(
            1,
            "SCOPE",
            scope.iter().flat_map(|scope| scope.attributes()),
            output,
            |w| {
                write_optional_bytes(w, "name", name)?;
                write_optional_bytes(w, "version", version)?;
                write_optional_bytes(w, "schema_url", Some(schema_url))
            },
        )?;

        for metric in scope_metrics.metrics() {
            self.format_metric(&metric, output)?;
        }
        Ok(())
    }

    fn format_metric<M: MetricView>(&self, metric: &M, output: &mut dyn Write) -> io::Result<()> {
        self.format_attribute_line(2, "METRIC", metric.metadata(), output, |w| {
            write_bytes_field(w, "name", metric.name())?;
            write_optional_bytes(w, "description", non_empty(metric.description()))?;
            write_optional_bytes(w, "unit", non_empty(metric.unit()))
        })?;

        let Some(data) = metric.data() else {
            self.format_plain_line(3, "EMPTY", output, |_| Ok(()))?;
            return Ok(());
        };

        match data.value_type() {
            DataType::Gauge => {
                if let Some(gauge) = data.as_gauge() {
                    self.format_gauge(&gauge, output)?;
                }
            }
            DataType::Sum => {
                if let Some(sum) = data.as_sum() {
                    self.format_sum(&sum, output)?;
                }
            }
            DataType::Histogram => {
                if let Some(histogram) = data.as_histogram() {
                    self.format_histogram(&histogram, output)?;
                }
            }
            DataType::ExponentialHistogram => {
                if let Some(histogram) = data.as_exponential_histogram() {
                    self.format_exponential_histogram(&histogram, output)?;
                }
            }
            DataType::Summary => {
                if let Some(summary) = data.as_summary() {
                    self.format_summary(&summary, output)?;
                }
            }
        }
        Ok(())
    }

    fn format_gauge<G: GaugeView>(&self, gauge: &G, output: &mut dyn Write) -> io::Result<()> {
        self.format_plain_line(3, "GAUGE", output, |_| Ok(()))?;
        for point in gauge.data_points() {
            self.format_number_data_point(&point, output)?;
        }
        Ok(())
    }

    fn format_sum<S: SumView>(&self, sum: &S, output: &mut dyn Write) -> io::Result<()> {
        self.format_plain_line(3, "SUM", output, |w| {
            write_temporality(w, sum.aggregation_temporality())?;
            write!(w, " monotonic={}", sum.is_monotonic())
        })?;
        for point in sum.data_points() {
            self.format_number_data_point(&point, output)?;
        }
        Ok(())
    }

    fn format_number_data_point<P: NumberDataPointView>(
        &self,
        point: &P,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        self.format_attribute_line(4, "DATA_POINT", point.attributes(), output, |w| {
            write_times(w, point.start_time_unix_nano(), point.time_unix_nano())?;
            write_value(w, point.value())?;
            write_flags(w, point.flags().into_inner())
        })?;
        for exemplar in point.exemplars() {
            self.format_exemplar(&exemplar, output)?;
        }
        Ok(())
    }

    fn format_histogram<H: HistogramView>(
        &self,
        histogram: &H,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        self.format_plain_line(3, "HISTOGRAM", output, |w| {
            write_temporality(w, histogram.aggregation_temporality())
        })?;
        for point in histogram.data_points() {
            self.format_histogram_data_point(&point, output)?;
        }
        Ok(())
    }

    fn format_histogram_data_point<P: HistogramDataPointView>(
        &self,
        point: &P,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        self.format_attribute_line(4, "DATA_POINT", point.attributes(), output, |w| {
            write_times(w, point.start_time_unix_nano(), point.time_unix_nano())?;
            write!(w, " count={}", point.count())?;
            write_optional_f64(w, "sum", point.sum())?;
            write_optional_f64(w, "min", point.min())?;
            write_optional_f64(w, "max", point.max())?;
            write_flags(w, point.flags().into_inner())
        })?;
        for (index, bound) in point.explicit_bounds().enumerate() {
            self.format_plain_line(5, "EXPLICIT_BOUND", output, |w| {
                write!(w, " index={index} value={bound}")
            })?;
        }
        for (index, count) in point.bucket_counts().enumerate() {
            self.format_plain_line(5, "BUCKET_COUNT", output, |w| {
                write!(w, " index={index} count={count}")
            })?;
        }
        for exemplar in point.exemplars() {
            self.format_exemplar(&exemplar, output)?;
        }
        Ok(())
    }

    fn format_exponential_histogram<H: ExponentialHistogramView>(
        &self,
        histogram: &H,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        self.format_plain_line(3, "EXPONENTIAL_HISTOGRAM", output, |w| {
            write_temporality(w, histogram.aggregation_temporality())
        })?;
        for point in histogram.data_points() {
            self.format_exponential_histogram_data_point(&point, output)?;
        }
        Ok(())
    }

    fn format_exponential_histogram_data_point<P: ExponentialHistogramDataPointView>(
        &self,
        point: &P,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        self.format_attribute_line(4, "DATA_POINT", point.attributes(), output, |w| {
            write_times(w, point.start_time_unix_nano(), point.time_unix_nano())?;
            write!(
                w,
                " count={} scale={} zero_count={} zero_threshold={}",
                point.count(),
                point.scale(),
                point.zero_count(),
                point.zero_threshold()
            )?;
            write_optional_f64(w, "sum", point.sum())?;
            write_optional_f64(w, "min", point.min())?;
            write_optional_f64(w, "max", point.max())?;
            write_flags(w, point.flags().into_inner())
        })?;
        if let Some(positive) = point.positive() {
            self.format_buckets("POSITIVE_BUCKET", &positive, output)?;
        }
        if let Some(negative) = point.negative() {
            self.format_buckets("NEGATIVE_BUCKET", &negative, output)?;
        }
        for exemplar in point.exemplars() {
            self.format_exemplar(&exemplar, output)?;
        }
        Ok(())
    }

    fn format_buckets<B: BucketsView>(
        &self,
        label: &str,
        buckets: &B,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        let offset = buckets.offset();
        for (index, count) in buckets.bucket_counts().enumerate() {
            self.format_plain_line(5, label, output, |w| {
                let bucket_index = i64::from(offset) + index as i64;
                write!(
                    w,
                    " offset={offset} bucket_index={bucket_index} count={count}"
                )
            })?;
        }
        Ok(())
    }

    fn format_summary<S: SummaryView>(
        &self,
        summary: &S,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        self.format_plain_line(3, "SUMMARY", output, |_| Ok(()))?;
        for point in summary.data_points() {
            self.format_summary_data_point(&point, output)?;
        }
        Ok(())
    }

    fn format_summary_data_point<P: SummaryDataPointView>(
        &self,
        point: &P,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        self.format_attribute_line(4, "DATA_POINT", point.attributes(), output, |w| {
            write_times(w, point.start_time_unix_nano(), point.time_unix_nano())?;
            write!(w, " count={} sum={}", point.count(), point.sum())?;
            write_flags(w, point.flags().into_inner())
        })?;
        for quantile in point.quantile_values() {
            self.format_quantile(&quantile, output)?;
        }
        Ok(())
    }

    fn format_quantile<Q: ValueAtQuantileView>(
        &self,
        quantile: &Q,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        self.format_plain_line(5, "QUANTILE", output, |w| {
            write!(
                w,
                " quantile={} value={}",
                quantile.quantile(),
                quantile.value()
            )
        })
    }

    fn format_exemplar<E: ExemplarView>(
        &self,
        exemplar: &E,
        output: &mut dyn Write,
    ) -> io::Result<()> {
        self.format_attribute_line(5, "EXEMPLAR", exemplar.filtered_attributes(), output, |w| {
            write!(w, " time_unix_nano={}", exemplar.time_unix_nano())?;
            write_value(w, exemplar.value())?;
            if let Some(span_id) = exemplar.span_id() {
                write!(w, " span_id={}", hex::encode(span_id))?;
            }
            if let Some(trace_id) = exemplar.trace_id() {
                write!(w, " trace_id={}", hex::encode(trace_id))?;
            }
            Ok(())
        })
    }

    fn format_plain_line(
        &self,
        depth: usize,
        label: &str,
        output: &mut dyn Write,
        fields: impl FnOnce(&mut MetricsWriter<'_>) -> io::Result<()>,
    ) -> io::Result<()> {
        self.format_metrics_line(output, |w| {
            self.write_prefix(w, depth)?;
            w.write_styled(AnsiCode::Green, |w| w.write_all(label.as_bytes()))?;
            fields(w)?;
            w.finish_line()
        })
    }

    fn format_attribute_line<A: AttributeView>(
        &self,
        depth: usize,
        label: &str,
        attrs: impl Iterator<Item = A>,
        output: &mut dyn Write,
        fields: impl FnOnce(&mut MetricsWriter<'_>) -> io::Result<()>,
    ) -> io::Result<()> {
        self.format_metrics_line(output, |w| {
            self.write_prefix(w, depth)?;
            w.write_styled(AnsiCode::Green, |w| w.write_all(label.as_bytes()))?;
            fields(w)?;
            w.write_attrs(attrs)?;
            w.finish_line()
        })
    }

    fn format_metrics_line(
        &self,
        output: &mut dyn Write,
        format: impl FnOnce(&mut MetricsWriter<'_>) -> io::Result<()>,
    ) -> io::Result<()> {
        format(&mut PrettyWriter::new(output, self.color))
    }

    fn write_prefix(&self, w: &mut MetricsWriter<'_>, depth: usize) -> io::Result<()> {
        for _ in 0..depth {
            w.write_all(self.tree.vertical.as_bytes())?;
            w.write_all(b" ")?;
        }
        if depth > 0 {
            w.write_all(self.tree.tee.as_bytes())?;
            w.write_all(b" ")?;
        }
        Ok(())
    }
}

fn non_empty(value: &[u8]) -> Option<&[u8]> {
    (!value.is_empty()).then_some(value)
}

fn write_optional_bytes(
    w: &mut MetricsWriter<'_>,
    name: &str,
    value: Option<&[u8]>,
) -> io::Result<()> {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        write_bytes_field(w, name, value)?;
    }
    Ok(())
}

fn write_bytes_field(w: &mut MetricsWriter<'_>, name: &str, value: &[u8]) -> io::Result<()> {
    write!(w, " {name}=")?;
    w.write_all(value)
}

fn write_times(w: &mut MetricsWriter<'_>, start_time: u64, time: u64) -> io::Result<()> {
    write!(
        w,
        " start_time_unix_nano={start_time} time_unix_nano={time}"
    )
}

fn write_value(w: &mut MetricsWriter<'_>, value: Option<Value>) -> io::Result<()> {
    match value {
        Some(Value::Double(value)) => write!(w, " value_double={value}"),
        Some(Value::Integer(value)) => write!(w, " value_int={value}"),
        None => w.write_all(b" value=none"),
    }
}

fn write_flags(w: &mut MetricsWriter<'_>, flags: u32) -> io::Result<()> {
    if flags != 0 {
        write!(w, " flags={flags}")?;
    }
    Ok(())
}

fn write_optional_f64(w: &mut MetricsWriter<'_>, name: &str, value: Option<f64>) -> io::Result<()> {
    if let Some(value) = value {
        write!(w, " {name}={value}")?;
    }
    Ok(())
}

fn write_temporality(
    w: &mut MetricsWriter<'_>,
    temporality: AggregationTemporality,
) -> io::Result<()> {
    let value = match temporality {
        AggregationTemporality::Unspecified => "unspecified",
        AggregationTemporality::Delta => "delta",
        AggregationTemporality::Cumulative => "cumulative",
    };
    write!(w, " temporality={value}")
}
