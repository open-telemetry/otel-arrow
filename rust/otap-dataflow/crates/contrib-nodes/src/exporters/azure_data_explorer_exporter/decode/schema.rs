// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Private allocation layouts, using existing field numbers and actual Rust field types.

use super::{DecodeSchema, Field};
use otel_arrow_dfe_pdata::proto::consts::field_num::{
    common as c, logs as l, metrics as m, resource as r, traces as t,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
    AnyValue, ArrayValue, EntityRef, InstrumentationScope, KeyValue, KeyValueList, any_value,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
    LogRecord, LogsData, ResourceLogs, ScopeLogs,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
    HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    Summary, SummaryDataPoint, exemplar, exponential_histogram_data_point::Buckets, metric,
    number_data_point, summary_data_point::ValueAtQuantile,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status, TracesData,
    span::{Event, Link},
};
use prost::encoding::WireType;

// Entity-reference tags are not yet represented in the shared field constants.
const RESOURCE_ENTITY_REFS: u64 = 3;
const ENTITY_REF_SCHEMA_URL: u64 = 1;
const ENTITY_REF_TYPE: u64 = 2;
const ENTITY_REF_ID_KEYS: u64 = 3;
const ENTITY_REF_DESCRIPTION_KEYS: u64 = 4;

trait Storage {
    type Element;
    const REPEATED: bool;
}

impl<T> Storage for Vec<T> {
    type Element = T;
    const REPEATED: bool = true;
}

impl<T> Storage for Option<T> {
    type Element = T;
    const REPEATED: bool = false;
}

trait ScalarValue {}
trait ByteValue {}
impl ByteValue for String {}
impl ByteValue for Vec<u8> {}

macro_rules! scalars {
    ($($ty:ty => $value:expr),* $(,)?) => {$(
        impl Storage for $ty {
            type Element = Self;
            const REPEATED: bool = false;
        }
        impl ScalarValue for $ty {}
        #[cfg(test)]
        impl Probe for $ty {
            fn probe() -> Self { $value }
        }
    )*};
}

scalars!(bool => true, i32 => 1, i64 => 1, u32 => 1, u64 => 1, f32 => 1.0, f64 => 1.0);

fn message<T: DecodeSchema>() -> Field {
    Field::Message {
        schema: T::field,
        size: size_of::<T>(),
    }
}

fn message_field<M, F: Storage>(_: fn(&M) -> &F) -> Field
where
    F::Element: DecodeSchema,
{
    message::<F::Element>()
}

fn scalar_field<M, F: Storage>(_: fn(&M) -> &F, wire: WireType) -> Field
where
    F::Element: ScalarValue,
{
    Field::Scalar {
        wire,
        repeated: F::REPEATED,
        size: size_of::<F::Element>(),
    }
}

fn bytes_field<M, F: ByteValue>(_: fn(&M) -> &F) -> Field {
    Field::Bytes {
        repeated: false,
        size: size_of::<F>(),
    }
}

fn buffers_field<M, F: Storage>(_: fn(&M) -> &F) -> Field
where
    F::Element: ByteValue,
{
    Field::Bytes {
        repeated: F::REPEATED,
        size: size_of::<F::Element>(),
    }
}

fn variant_message<T: DecodeSchema, V>(_: fn(T) -> V) -> Field {
    message::<T>()
}

fn variant_bytes<T: ByteValue, V>(_: fn(T) -> V) -> Field {
    Field::Bytes {
        repeated: false,
        size: size_of::<T>(),
    }
}

fn variant_scalar<T: ScalarValue, V>(_: fn(T) -> V, wire: WireType) -> Field {
    Field::Scalar {
        wire,
        repeated: false,
        size: size_of::<T>(),
    }
}

macro_rules! field_layout {
    ($ty:ty, $field:ident, message($number:path)) => {
        message_field::<$ty, _>(|message| &message.$field)
    };
    ($ty:ty, $field:ident, scalar($number:path, $wire:ident)) => {
        scalar_field::<$ty, _>(|message| &message.$field, WireType::$wire)
    };
    ($ty:ty, $field:ident, bytes($number:path)) => {
        bytes_field::<$ty, _>(|message| &message.$field)
    };
    ($ty:ty, $field:ident, buffers($number:path)) => {
        buffers_field::<$ty, _>(|message| &message.$field)
    };
}

macro_rules! variant_layout {
    (message($variant:path)) => {
        variant_message($variant)
    };
    (bytes($variant:path)) => {
        variant_bytes($variant)
    };
    (scalar($variant:path, $wire:ident)) => {
        variant_scalar($variant, WireType::$wire)
    };
}

macro_rules! lookup {
    ($tag:expr, $ty:ty, $field:ident, oneof(
        $($number:path => $kind:ident($variant:path $(, $wire:ident)?)),* $(,)?
    )) => {{
        let _ = |value| match value { $($variant(_) => (),)* };
        match u64::from($tag) {
            $($number => Some(variant_layout!($kind($variant $(, $wire)?))),)*
            _ => None,
        }
    }};
    ($tag:expr, $ty:ty, $field:ident, $kind:ident($number:path $(, $wire:ident)?)) => {
        if u64::from($tag) == $number {
            Some(field_layout!($ty, $field, $kind($number $(, $wire)?)))
        } else {
            None
        }
    };
}

#[cfg(test)]
trait Probe {
    fn probe() -> Self;
}

#[cfg(test)]
impl Probe for String {
    fn probe() -> Self {
        "x".to_string()
    }
}

#[cfg(test)]
impl<T: Default> Probe for Vec<T> {
    fn probe() -> Self {
        vec![T::default()]
    }
}

#[cfg(test)]
impl<T: Default> Probe for Option<T> {
    fn probe() -> Self {
        Some(T::default())
    }
}

#[cfg(test)]
fn variant_probe<T: Default, V>(variant: fn(T) -> V) -> V {
    variant(T::default())
}

#[cfg(test)]
fn check_field<T: DecodeSchema + prost::Message + Default + PartialEq>(
    number: u64,
    set: impl FnOnce(&mut T),
) {
    let mut message = T::default();
    set(&mut message);
    let bytes = message.encode_to_vec();
    let (tag, wire) = prost::encoding::decode_key(&mut bytes.as_slice()).expect("Prost field");
    assert_eq!(u64::from(tag), number, "{}", std::any::type_name::<T>());
    let field = T::field(tag).expect("known field");
    let packed =
        matches!(field, Field::Scalar { repeated: true, .. }) && wire == WireType::LengthDelimited;
    let expected = match field {
        Field::Message { .. } | Field::Bytes { .. } => WireType::LengthDelimited,
        Field::Scalar { repeated: true, .. } if wire == WireType::LengthDelimited => wire,
        Field::Scalar { wire, .. } => wire,
    };
    assert_eq!(wire, expected, "{} field {tag}", std::any::type_name::<T>());
    let mut admission =
        super::DecodeAdmission::new::<T>(&bytes, usize::MAX, usize::MAX).expect("admission");
    while !admission.step(8, 64).expect("valid Prost field") {}
    if packed {
        assert_eq!(
            admission.work,
            2,
            "one packed element in {} field {tag}",
            std::any::type_name::<T>()
        );
    }
    assert!(
        T::decode(bytes.as_slice()).expect("Prost round trip") == message,
        "Prost round trip for {} field {tag}",
        std::any::type_name::<T>(),
    );
}

#[cfg(test)]
macro_rules! probe {
    ($ty:ident, $field:ident, oneof(
        $($number:path => $kind:ident($variant:path $(, $wire:ident)?)),* $(,)?
    )) => {$(
        check_field::<$ty>($number, |message| {
            message.$field = Some(variant_probe($variant));
        });
    )*};
    ($ty:ident, $field:ident, $kind:ident($number:path $(, $wire:ident)?)) => {{
        check_field::<$ty>($number, |message| {
            message.$field = Probe::probe();
        });
    }};
}

// Exhaustive struct and oneof patterns require layouts to be updated when the model grows.
macro_rules! schemas {
    ($($ty:ident { $($field:ident: $kind:ident($($args:tt)*)),* $(,)? })*) => {
        $(impl DecodeSchema for $ty {
            fn field(tag: u32) -> Option<Field> {
                let _ = |message: &Self| { let Self { $($field: _),* } = message; };
                $(if let Some(field) = lookup!(tag, Self, $field, $kind($($args)*)) {
                    return Some(field);
                })*
                None
            }
            #[cfg(test)]
            fn check_fields() {
                $(probe!(Self, $field, $kind($($args)*));)*
            }
        })*
        #[cfg(test)]
        pub(super) fn check_all_fields() {
            $(<$ty as DecodeSchema>::check_fields();)*
        }
    };
}

schemas! {
    AnyValue {
        value: oneof(
            c::ANY_VALUE_STRING_VALUE => bytes(any_value::Value::StringValue),
            c::ANY_VALUE_BOOL_VALUE => scalar(any_value::Value::BoolValue, Varint),
            c::ANY_VALUE_INT_VALUE => scalar(any_value::Value::IntValue, Varint),
            c::ANY_VALUE_DOUBLE_VALUE => scalar(any_value::Value::DoubleValue, SixtyFourBit),
            c::ANY_VALUE_ARRAY_VALUE => message(any_value::Value::ArrayValue),
            c::ANY_VALUE_KVLIST_VALUE => message(any_value::Value::KvlistValue),
            c::ANY_VALUE_BYTES_VALUE => bytes(any_value::Value::BytesValue),
        ),
    }
    ArrayValue { values: message(c::ARRAY_VALUE_VALUES) }
    KeyValueList { values: message(c::KEY_VALUE_LIST_VALUES) }
    KeyValue {
        key: bytes(c::KEY_VALUE_KEY),
        value: message(c::KEY_VALUE_VALUE),
    }
    InstrumentationScope {
        name: bytes(c::INSTRUMENTATION_SCOPE_NAME),
        version: bytes(c::INSTRUMENTATION_SCOPE_VERSION),
        attributes: message(c::INSTRUMENTATION_SCOPE_ATTRIBUTES),
        dropped_attributes_count: scalar(c::INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT, Varint),
    }
    EntityRef {
        schema_url: bytes(ENTITY_REF_SCHEMA_URL),
        r#type: bytes(ENTITY_REF_TYPE),
        id_keys: buffers(ENTITY_REF_ID_KEYS),
        description_keys: buffers(ENTITY_REF_DESCRIPTION_KEYS),
    }
    Resource {
        attributes: message(r::RESOURCE_ATTRIBUTES),
        dropped_attributes_count: scalar(r::RESOURCE_DROPPED_ATTRIBUTES_COUNT, Varint),
        entity_refs: message(RESOURCE_ENTITY_REFS),
    }
    LogsData { resource_logs: message(l::LOGS_DATA_RESOURCE) }
    ResourceLogs {
        resource: message(l::RESOURCE_LOGS_RESOURCE),
        scope_logs: message(l::RESOURCE_LOGS_SCOPE_LOGS),
        schema_url: bytes(l::RESOURCE_LOGS_SCHEMA_URL),
    }
    ScopeLogs {
        scope: message(l::SCOPE_LOG_SCOPE),
        log_records: message(l::SCOPE_LOGS_LOG_RECORDS),
        schema_url: bytes(l::SCOPE_LOGS_SCHEMA_URL),
    }
    LogRecord {
        time_unix_nano: scalar(l::LOG_RECORD_TIME_UNIX_NANO, SixtyFourBit),
        observed_time_unix_nano: scalar(l::LOG_RECORD_OBSERVED_TIME_UNIX_NANO, SixtyFourBit),
        severity_number: scalar(l::LOG_RECORD_SEVERITY_NUMBER, Varint),
        severity_text: bytes(l::LOG_RECORD_SEVERITY_TEXT),
        body: message(l::LOG_RECORD_BODY),
        attributes: message(l::LOG_RECORD_ATTRIBUTES),
        dropped_attributes_count: scalar(l::LOG_RECORD_DROPPED_ATTRIBUTES_COUNT, Varint),
        flags: scalar(l::LOG_RECORD_FLAGS, ThirtyTwoBit),
        trace_id: bytes(l::LOG_RECORD_TRACE_ID),
        span_id: bytes(l::LOG_RECORD_SPAN_ID),
        event_name: bytes(l::LOG_RECORD_EVENT_NAME),
    }
    MetricsData { resource_metrics: message(m::METRICS_DATA_RESOURCE_METRICS) }
    ResourceMetrics {
        resource: message(m::RESOURCE_METRICS_RESOURCE),
        scope_metrics: message(m::RESOURCE_METRICS_SCOPE_METRICS),
        schema_url: bytes(m::RESOURCE_METRICS_SCHEMA_URL),
    }
    ScopeMetrics {
        scope: message(m::SCOPE_METRICS_SCOPE),
        metrics: message(m::SCOPE_METRICS_METRICS),
        schema_url: bytes(m::SCOPE_METRICS_SCHEMA_URL),
    }
    Metric {
        name: bytes(m::METRIC_NAME),
        description: bytes(m::METRIC_DESCRIPTION),
        unit: bytes(m::METRIC_UNIT),
        metadata: message(m::METRIC_METADATA),
        data: oneof(
            m::METRIC_GAUGE => message(metric::Data::Gauge),
            m::METRIC_SUM => message(metric::Data::Sum),
            m::METRIC_HISTOGRAM => message(metric::Data::Histogram),
            m::METRIC_EXPONENTIAL_HISTOGRAM => message(metric::Data::ExponentialHistogram),
            m::METRIC_SUMMARY => message(metric::Data::Summary),
        ),
    }
    Gauge { data_points: message(m::GAUGE_DATA_POINTS) }
    Sum {
        data_points: message(m::SUM_DATA_POINTS),
        aggregation_temporality: scalar(m::SUM_AGGREGATION_TEMPORALITY, Varint),
        is_monotonic: scalar(m::SUM_IS_MONOTONIC, Varint),
    }
    Histogram {
        data_points: message(m::HISTOGRAM_DATA_POINTS),
        aggregation_temporality: scalar(m::HISTOGRAM_AGGREGATION_TEMPORALITY, Varint),
    }
    ExponentialHistogram {
        data_points: message(m::EXPONENTIAL_HISTOGRAM_DATA_POINTS),
        aggregation_temporality: scalar(m::EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY, Varint),
    }
    Summary { data_points: message(m::SUMMARY_DATA_POINTS) }
    NumberDataPoint {
        attributes: message(m::NUMBER_DP_ATTRIBUTES),
        start_time_unix_nano: scalar(m::NUMBER_DP_START_TIME_UNIX_NANO, SixtyFourBit),
        time_unix_nano: scalar(m::NUMBER_DP_TIME_UNIX_NANO, SixtyFourBit),
        exemplars: message(m::NUMBER_DP_EXEMPLARS),
        flags: scalar(m::NUMBER_DP_FLAGS, Varint),
        value: oneof(
            m::NUMBER_DP_AS_DOUBLE => scalar(number_data_point::Value::AsDouble, SixtyFourBit),
            m::NUMBER_DP_AS_INT => scalar(number_data_point::Value::AsInt, SixtyFourBit),
        ),
    }
    HistogramDataPoint {
        attributes: message(m::HISTOGRAM_DP_ATTRIBUTES),
        start_time_unix_nano: scalar(m::HISTOGRAM_DP_START_TIME_UNIX_NANO, SixtyFourBit),
        time_unix_nano: scalar(m::HISTOGRAM_DP_TIME_UNIX_NANO, SixtyFourBit),
        count: scalar(m::HISTOGRAM_DP_COUNT, SixtyFourBit),
        sum: scalar(m::HISTOGRAM_DP_SUM, SixtyFourBit),
        bucket_counts: scalar(m::HISTOGRAM_DP_BUCKET_COUNTS, SixtyFourBit),
        explicit_bounds: scalar(m::HISTOGRAM_DP_EXPLICIT_BOUNDS, SixtyFourBit),
        exemplars: message(m::HISTOGRAM_DP_EXEMPLARS),
        flags: scalar(m::HISTOGRAM_DP_FLAGS, Varint),
        min: scalar(m::HISTOGRAM_DP_MIN, SixtyFourBit),
        max: scalar(m::HISTOGRAM_DP_MAX, SixtyFourBit),
    }
    ExponentialHistogramDataPoint {
        attributes: message(m::EXP_HISTOGRAM_DP_ATTRIBUTES),
        start_time_unix_nano: scalar(m::EXP_HISTOGRAM_DP_START_TIME_UNIX_NANO, SixtyFourBit),
        time_unix_nano: scalar(m::EXP_HISTOGRAM_DP_TIME_UNIX_NANO, SixtyFourBit),
        count: scalar(m::EXP_HISTOGRAM_DP_COUNT, SixtyFourBit),
        sum: scalar(m::EXP_HISTOGRAM_DP_SUM, SixtyFourBit),
        scale: scalar(m::EXP_HISTOGRAM_DP_SCALE, Varint),
        zero_count: scalar(m::EXP_HISTOGRAM_DP_ZERO_COUNT, SixtyFourBit),
        positive: message(m::EXP_HISTOGRAM_DP_POSITIVE),
        negative: message(m::EXP_HISTOGRAM_DP_NEGATIVE),
        flags: scalar(m::EXP_HISTOGRAM_DP_FLAGS, Varint),
        exemplars: message(m::EXP_HISTOGRAM_DP_EXEMPLARS),
        min: scalar(m::EXP_HISTOGRAM_DP_MIN, SixtyFourBit),
        max: scalar(m::EXP_HISTOGRAM_DP_MAX, SixtyFourBit),
        zero_threshold: scalar(m::EXP_HISTOGRAM_DP_ZERO_THRESHOLD, SixtyFourBit),
    }
    Buckets {
        offset: scalar(m::EXP_HISTOGRAM_BUCKET_OFFSET, Varint),
        bucket_counts: scalar(m::EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, Varint),
    }
    SummaryDataPoint {
        attributes: message(m::SUMMARY_DP_ATTRIBUTES),
        start_time_unix_nano: scalar(m::SUMMARY_DP_START_TIME_UNIX_NANO, SixtyFourBit),
        time_unix_nano: scalar(m::SUMMARY_DP_TIME_UNIX_NANO, SixtyFourBit),
        count: scalar(m::SUMMARY_DP_COUNT, SixtyFourBit),
        sum: scalar(m::SUMMARY_DP_SUM, SixtyFourBit),
        quantile_values: message(m::SUMMARY_DP_QUANTILE_VALUES),
        flags: scalar(m::SUMMARY_DP_FLAGS, Varint),
    }
    ValueAtQuantile {
        quantile: scalar(m::VALUE_AT_QUANTILE_QUANTILE, SixtyFourBit),
        value: scalar(m::VALUE_AT_QUANTILE_VALUE, SixtyFourBit),
    }
    Exemplar {
        filtered_attributes: message(m::EXEMPLAR_FILTERED_ATTRIBUTES),
        time_unix_nano: scalar(m::EXEMPLAR_TIME_UNIX_NANO, SixtyFourBit),
        span_id: bytes(m::EXEMPLAR_SPAN_ID),
        trace_id: bytes(m::EXEMPLAR_TRACE_ID),
        value: oneof(
            m::EXEMPLAR_AS_DOUBLE => scalar(exemplar::Value::AsDouble, SixtyFourBit),
            m::EXEMPLAR_AS_INT => scalar(exemplar::Value::AsInt, SixtyFourBit),
        ),
    }
    TracesData { resource_spans: message(t::TRACES_DATA_RESOURCE_SPANS) }
    ResourceSpans {
        resource: message(t::RESOURCE_SPANS_RESOURCE),
        scope_spans: message(t::RESOURCE_SPANS_SCOPE_SPANS),
        schema_url: bytes(t::RESOURCE_SPANS_SCHEMA_URL),
    }
    ScopeSpans {
        scope: message(t::SCOPE_SPANS_SCOPE),
        spans: message(t::SCOPE_SPANS_SPANS),
        schema_url: bytes(t::SCOPE_SPANS_SCHEMA_URL),
    }
    Span {
        trace_id: bytes(t::SPAN_TRACE_ID),
        span_id: bytes(t::SPAN_SPAN_ID),
        trace_state: bytes(t::SPAN_TRACE_STATE),
        parent_span_id: bytes(t::SPAN_PARENT_SPAN_ID),
        flags: scalar(t::SPAN_FLAGS, ThirtyTwoBit),
        name: bytes(t::SPAN_NAME),
        kind: scalar(t::SPAN_KIND, Varint),
        start_time_unix_nano: scalar(t::SPAN_START_TIME_UNIX_NANO, SixtyFourBit),
        end_time_unix_nano: scalar(t::SPAN_END_TIME_UNIX_NANO, SixtyFourBit),
        attributes: message(t::SPAN_ATTRIBUTES),
        dropped_attributes_count: scalar(t::SPAN_DROPPED_ATTRIBUTES_COUNT, Varint),
        events: message(t::SPAN_EVENTS),
        dropped_events_count: scalar(t::SPAN_DROPPED_EVENTS_COUNT, Varint),
        links: message(t::SPAN_LINKS),
        dropped_links_count: scalar(t::SPAN_DROPPED_LINKS_COUNT, Varint),
        status: message(t::SPAN_STATUS),
    }
    Event {
        time_unix_nano: scalar(t::SPAN_EVENT_TIME_UNIX_NANO, SixtyFourBit),
        name: bytes(t::SPAN_EVENT_NAME),
        attributes: message(t::SPAN_EVENT_ATTRIBUTES),
        dropped_attributes_count: scalar(t::SPAN_EVENT_DROPPED_ATTRIBUTES_COUNTS, Varint),
    }
    Link {
        trace_id: bytes(t::SPAN_LINK_TRACE_ID),
        span_id: bytes(t::SPAN_LINK_SPAN_ID),
        trace_state: bytes(t::SPAN_LINK_TRACE_STATE),
        attributes: message(t::SPAN_LINK_ATTRIBUTES),
        dropped_attributes_count: scalar(t::SPAN_LINK_DROPPED_ATTRIBUTES_COUNT, Varint),
        flags: scalar(t::SPAN_LINK_FLAGS, ThirtyTwoBit),
    }
    Status {
        message: bytes(t::SPAN_STATUS_MESSAGE),
        code: scalar(t::SPAN_STATUS_CODE, Varint),
    }
}
