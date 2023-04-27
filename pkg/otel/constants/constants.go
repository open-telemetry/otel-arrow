// Copyright The OpenTelemetry Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package constants

// All constants used a column names.

const ResourceMetrics = "resource_metrics"
const ResourceSpans = "resource_spans"
const ResourceLogs = "resource_logs"
const TimeUnixNano string = "time_unix_nano"
const StartTimeUnixNano string = "start_time_unix_nano"
const DurationTimeUnixNano string = "duration_time_unix_nano"
const ObservedTimeUnixNano string = "observed_time_unix_nano"
const SeverityNumber string = "severity_number"
const SeverityText string = "severity_text"
const DroppedAttributesCount string = "dropped_attributes_count"
const DroppedEventsCount string = "dropped_events_count"
const DroppedLinksCount string = "dropped_links_count"
const Flags string = "flags"
const TraceId string = "trace_id"
const TraceState string = "trace_state"
const SpanId string = "span_id"
const ParentSpanId string = "parent_span_id"
const Attributes string = "attributes"
const AttributesID string = "attrs_id"
const Resource string = "resource"
const ScopeMetrics string = "scope_metrics"
const ScopeLogs string = "scope_logs"
const ScopeSpans string = "scope_spans"
const Scope string = "scope"
const Spans string = "spans"
const Logs string = "logs"
const Name string = "name"
const KIND string = "kind"
const Version string = "version"
const Body string = "body"
const Status string = "status"
const Description string = "description"
const Unit string = "unit"
const Data string = "data"
const StatusMessage string = "status_message"
const StatusCode string = "code"
const GaugeMetrics string = "gauge"
const SumMetrics string = "sum"
const SummaryMetrics string = "summary"
const SpanEvents string = "events"
const SpanLinks string = "links"
const SummaryCount string = "count"
const SummarySum string = "sum"
const SummaryQuantileValues string = "quantile"
const SummaryQuantile string = "quantile"
const SummaryValue string = "value"
const MetricValue string = "value"
const HistogramCount string = "count"
const HistogramSum string = "sum"
const HistogramMin string = "min"
const HistogramMax string = "max"
const HistogramBucketCounts string = "bucket_counts"
const HistogramExplicitBounds string = "explicit_bounds"
const ExpHistogramScale string = "scale"
const ExpHistogramZeroCount string = "zero_count"
const ExpHistogramPositive string = "positive"
const ExpHistogramNegative string = "negative"
const ExpHistogramOffset string = "offset"
const ExpHistogramBucketCounts string = "bucket_counts"
const SchemaUrl string = "schema_url"
const UnivariateMetrics string = "univariate_metrics"
const I64MetricValue string = "i64"
const F64MetricValue string = "f64"
const Exemplars string = "exemplars"
const DataPoints string = "data_points"
const IsMonotonic string = "is_monotonic"
const AggregationTemporality string = "aggregation_temporality"
const HistogramMetrics string = "histogram"
const ExpHistogramMetrics string = "exp_histogram"

const SharedAttributes string = "shared_attributes"
const SharedEventAttributes string = "shared_event_attributes"
const SharedLinkAttributes string = "shared_link_attributes"
const SharedStartTimeUnixNano string = "shared_start_time_unix_nano"
const SharedTimeUnixNano string = "shared_time_unix_nano"

const ID string = "id"

const AttrsRecordKey string = "key"
const AttrsRecordValue string = "value"
