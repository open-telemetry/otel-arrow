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

// All constants used as metadata keys.

const METADATA_METRIC_TYPE = "mt"
const METADATA_METRIC_MULTIVARIATE_ATTR = "mma"
const METADATA_METRIC_UNIT = "mu"
const METADATA_METRIC_DESCRIPTION = "md"

// All constants used a column names.

const RESOURCE_METRICS = "resource_metrics"
const RESOURCE_SPANS = "resource_spans"
const RESOURCE_LOGS = "resource_logs"
const TIME_UNIX_NANO string = "time_unix_nano"
const START_TIME_UNIX_NANO string = "start_time_unix_nano"
const END_TIME_UNIX_NANO string = "end_time_unix_nano"
const OBSERVED_TIME_UNIX_NANO string = "observed_time_unix_nano"
const SEVERITY_NUMBER string = "severity_number"
const SEVERITY_TEXT string = "severity_text"
const DROPPED_ATTRIBUTES_COUNT string = "dropped_attributes_count"
const DROPPED_EVENTS_COUNT string = "dropped_events_count"
const DROPPED_LINKS_COUNT string = "dropped_links_count"
const FLAGS string = "flags"
const TRACE_ID string = "trace_id"
const TRACE_STATE string = "trace_state"
const SPAN_ID string = "span_id"
const PARENT_SPAN_ID string = "parent_span_id"
const ATTRIBUTES string = "attributes"
const RESOURCE string = "resource"
const SCOPE_METRICS string = "scope_metrics"
const SCOPE_LOGS string = "scope_logs"
const SCOPE_SPANS string = "scope_spans"
const SCOPE string = "scope"
const SPANS string = "spans"
const LOGS string = "logs"
const NAME string = "name"
const KIND string = "kind"
const VERSION string = "version"
const BODY string = "body"
const STATUS string = "status"
const DESCRIPTION string = "description"
const UNIT string = "unit"
const DATA string = "data"
const STATUS_MESSAGE string = "status_message"
const STATUS_CODE string = "code"
const GAUGE_METRICS string = "gauge"
const SUM_METRICS string = "sum"
const SUMMARY_METRICS string = "summary"
const SPAN_EVENTS string = "events"
const SPAN_LINKS string = "links"
const SUMMARY_COUNT string = "count"
const SUMMARY_SUM string = "sum"
const SUMMARY_QUANTILE_VALUES string = "quantile"
const SUMMARY_QUANTILE string = "quantile"
const SUMMARY_VALUE string = "value"
const METRIC_VALUE string = "value"
const HISTOGRAM string = "histogram"
const HISTOGRAM_COUNT string = "count"
const HISTOGRAM_SUM string = "sum"
const HISTOGRAM_MIN string = "min"
const HISTOGRAM_MAX string = "max"
const HISTOGRAM_BUCKET_COUNTS string = "bucket_counts"
const HISTOGRAM_EXPLICIT_BOUNDS string = "explicit_bounds"
const EXP_HISTOGRAM string = "exp_histogram"
const EXP_HISTOGRAM_SCALE string = "scale"
const EXP_HISTOGRAM_ZERO_COUNT string = "zero_count"
const EXP_HISTOGRAM_POSITIVE string = "positive"
const EXP_HISTOGRAM_NEGATIVE string = "negative"
const EXP_HISTOGRAM_OFFSET string = "offset"
const SCHEMA_URL string = "schema_url"
const METRICS string = "metrics"
const I64_METRIC_VALUE string = "i64"
const F64_METRIC_VALUE string = "f64"
const EXEMPLARS string = "exemplars"
const DATA_POINTS string = "data_points"
const IS_MONOTONIC string = "is_monotonic"
const AGGREGATION_TEMPORALITY string = "aggregation_temporality"
const UNIVARIATE_METRICS string = "univariate_metrics"
const MULTIVARIATE_METRICS string = "multivariate_metrics"
