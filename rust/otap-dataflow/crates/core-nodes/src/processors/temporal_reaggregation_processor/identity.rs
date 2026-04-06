// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Semantic identity types for metrics.
//!
//! This module provides identity types which determine whether two
//! resources, scopes, metrics, or data-point streams refer to the same
//! logical entity for the purpose of identifying metrics streams.
//!
//! In general these type use Cow to allow creating a 0 copy id from a view which
//! can be used to index into a [hashbrown::HashMap] with a restrictive 'static
//! lifetime thanks to the [Equivalent] trait.

use std::borrow::Cow;

use hashbrown::Equivalent;
use otap_df_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, ValueType,
};
use otap_df_pdata_views::views::metrics::{
    AggregationTemporality, DataType, DataView, ExponentialHistogramView, HistogramView,
    MetricView, SumView,
};
use otap_df_pdata_views::views::resource::ResourceView;

/// Compute a [`ResourceId`] from an optional resource view. When the view is
/// `None`, the identity uses `AttributeHash::EMPTY`.
pub fn resource_id_of<R: ResourceView>(
    hash_buf: &mut HashBuffer,
    resource: Option<&R>,
) -> ResourceId {
    let attrs = resource
        .map(|r| AttributeHash::compute(hash_buf, r.attributes()))
        .unwrap_or(AttributeHash::EMPTY);
    ResourceId { attrs }
}

/// Compute a [`ScopeId`] from an optional scope view. When the view is `None`,
/// the identity uses empty name/version and `AttributeHash::EMPTY`.
pub fn scope_id_of<'a, S: InstrumentationScopeView>(
    hash_buf: &mut HashBuffer,
    resource_id: ResourceId,
    scope: Option<&'a S>,
) -> ScopeId<'a> {
    let attrs = scope
        .map(|s| AttributeHash::compute(hash_buf, s.attributes()))
        .unwrap_or(AttributeHash::EMPTY);
    let name = scope.and_then(|s| s.name()).unwrap_or(b"");
    let version = scope.and_then(|s| s.version()).unwrap_or(b"");
    ScopeId {
        resource: resource_id,
        name: Cow::Borrowed(name),
        version: Cow::Borrowed(version),
        attrs,
    }
}

/// Extract (data_type, aggregation_temporality, is_monotonic) from a
/// [`DataView`] without constructing a full [`MetricId`].
pub fn metric_type_info_of<'a>(data: &impl DataView<'a>) -> (u8, u8, bool) {
    let dt = data.value_type();
    let (is_monotonic, temporality) = match dt {
        DataType::Sum => {
            let sum = data.as_sum().expect("DataType::Sum should have sum data");
            (sum.is_monotonic(), sum.aggregation_temporality())
        }
        DataType::Histogram => {
            let hist = data
                .as_histogram()
                .expect("DataType::Histogram should have histogram data");
            (true, hist.aggregation_temporality())
        }
        DataType::ExponentialHistogram => {
            let exp = data
                .as_exponential_histogram()
                .expect("DataType::ExponentialHistogram should have exp histogram data");
            (true, exp.aggregation_temporality())
        }
        DataType::Gauge | DataType::Summary => (false, AggregationTemporality::Unspecified),
    };
    (dt as u8, temporality as u8, is_monotonic)
}

// Helper for computing metric id from a view.
pub fn metric_id_of<'a, M: MetricView>(
    scope_id: ScopeId<'a>,
    metric: &'a M,
) -> Option<MetricId<'a>> {
    let data = metric.data()?;
    let dt = data.value_type();

    let (is_monotonic, temporality) = match dt {
        DataType::Sum => {
            let sum = data.as_sum().expect("DataType::Sum should have sum data");
            (sum.is_monotonic(), sum.aggregation_temporality())
        }
        DataType::Histogram => {
            let hist = data
                .as_histogram()
                .expect("DataType::Histogram should have histogram data");
            (true, hist.aggregation_temporality())
        }
        DataType::ExponentialHistogram => {
            let exp = data
                .as_exponential_histogram()
                .expect("DataType::ExponentialHistogram should have exp histogram data");
            (true, exp.aggregation_temporality())
        }
        DataType::Gauge | DataType::Summary => (false, AggregationTemporality::Unspecified),
    };

    Some(MetricId {
        scope: scope_id,
        name: Cow::Borrowed(metric.name()),
        unit: Cow::Borrowed(metric.unit()),
        data_type: dt as u8,
        is_monotonic,
        aggregation_temporality: temporality as u8,
    })
}

/// Compute a [`StreamId`] from a [`MetricId`] and a data point's attributes.
pub fn stream_id_of<'a, A: AttributeView>(
    hash_buf: &mut HashBuffer,
    metric_id: MetricId<'a>,
    attrs: impl Iterator<Item = A>,
) -> StreamId<'a> {
    StreamId {
        metric: metric_id,
        attrs: AttributeHash::compute(hash_buf, attrs),
    }
}

/// Identity of a data-point stream within a metric.
///
/// A stream is a unique time series identified by the parent [`MetricId`]
/// plus the data point's own attributes. This is the finest-grained
/// identity level and is the key used for temporal reaggregation.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StreamId<'a> {
    pub(super) metric: MetricId<'a>,
    pub(super) attrs: AttributeHash,
}

impl<'a> StreamId<'a> {
    /// Convert all borrowed data into owned data, producing a
    /// `StreamId<'static>`.
    #[must_use]
    pub fn into_owned(self) -> StreamId<'static> {
        StreamId {
            metric: self.metric.into_owned(),
            attrs: self.attrs,
        }
    }
}

/// Identity of a metric within a scope.
///
/// Incorporates the parent [`ScopeId`], the metric name, unit, data type,
/// monotonicity flag, and aggregation temporality. The enum-typed fields
/// are stored as `u8` to allow deriving `Hash`.
///
/// TODO: Consider if we want to ignore/drop/reject metrics which have the same
/// name, but differ in any other field. Spec seems to think that consumers can
/// do this:
/// https://opentelemetry.io/docs/specs/otel/metrics/data-model/#opentelemetry-protocol-data-model-consumer-recommendations
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MetricId<'a> {
    pub(super) scope: ScopeId<'a>,
    pub(super) name: Cow<'a, [u8]>,
    pub(super) unit: Cow<'a, [u8]>,
    pub(super) data_type: u8,
    pub(super) is_monotonic: bool,
    pub(super) aggregation_temporality: u8,
}

impl<'a> MetricId<'a> {
    /// Convert all borrowed data into owned data, producing a
    /// `MetricId<'static>`.
    #[must_use]
    pub fn into_owned(self) -> MetricId<'static> {
        MetricId {
            scope: self.scope.into_owned(),
            name: Cow::Owned(self.name.into_owned()),
            unit: Cow::Owned(self.unit.into_owned()),
            data_type: self.data_type,
            is_monotonic: self.is_monotonic,
            aggregation_temporality: self.aggregation_temporality,
        }
    }
}

/// Identity of an instrumentation scope within a resource.
///
/// Incorporates the parent [`ResourceId`], the scope name and version, and
/// the scope's own attributes.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ScopeId<'a> {
    pub(super) resource: ResourceId,
    pub(super) name: Cow<'a, [u8]>,
    pub(super) version: Cow<'a, [u8]>,
    pub(super) attrs: AttributeHash,
}

impl<'a> ScopeId<'a> {
    /// Convert all borrowed data into owned data, producing a
    /// `ScopeId<'static>`.
    #[must_use]
    pub fn into_owned(self) -> ScopeId<'static> {
        ScopeId {
            resource: self.resource,
            name: Cow::Owned(self.name.into_owned()),
            version: Cow::Owned(self.version.into_owned()),
            attrs: self.attrs,
        }
    }
}

/// Identity of a Resource.
///
/// Two resources are considered identical if they have the same attributes.
/// This is the root of the identity hierarchy.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResourceId {
    pub(super) attrs: AttributeHash,
}

// hashbrown::Equivalent wrappers
//
// We can't impl Equivalent<XxxId<'static>> for XxxId<'a> directly because it
// conflicts with the blanket impl when 'a = 'static. Instead we use newtype
// wrappers that hashbrown::HashMap::get accepts as lookup keys.

/// Wrapper for looking up a borrowed [`ScopeId`] in a
/// `HashMap<ScopeId<'static>, _>`.
pub struct ScopeIdRef<'a>(pub &'a ScopeId<'a>);

impl<'a> Equivalent<ScopeId<'static>> for ScopeIdRef<'a> {
    fn equivalent(&self, key: &ScopeId<'static>) -> bool {
        self.0 == key
    }
}

impl<'a> std::hash::Hash for ScopeIdRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Wrapper for looking up a borrowed [`MetricId`] in a
/// `HashMap<MetricId<'static>, _>`.
pub struct MetricIdRef<'a>(pub &'a MetricId<'a>);

impl<'a> Equivalent<MetricId<'static>> for MetricIdRef<'a> {
    fn equivalent(&self, key: &MetricId<'static>) -> bool {
        self.0 == key
    }
}

impl<'a> std::hash::Hash for MetricIdRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Wrapper for looking up a borrowed [`StreamId`] in a
/// `HashMap<StreamId<'static>, _>`.
pub struct StreamIdRef<'a>(pub &'a StreamId<'a>);

impl<'a> Equivalent<StreamId<'static>> for StreamIdRef<'a> {
    fn equivalent(&self, key: &StreamId<'static>) -> bool {
        self.0 == key
    }
}

impl<'a> std::hash::Hash for StreamIdRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// A 128-bit hash of a set of key-value attributes.
///
/// Computed by sorting attributes by key, then hashing each key-value pair
/// with type-discriminator prefixes using xxh3_128.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AttributeHash([u8; 16]);

/// Reusable byte buffer for encoding prior to computing the hash
pub struct HashBuffer {
    buf: Vec<u8>,
}

impl HashBuffer {
    /// Create a new empty buffer.
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }
}

/// Type-discriminator bytes used during attribute hashing.
///
/// Each variant has a unique `u8` value so that different value types
/// (and structural markers) can never collide in the hash input buffer.
#[repr(u8)]
enum HashTag {
    /// Prefixes each attribute key.
    Key = 0xf4,
    /// Represents an empty / null value.
    Empty = 0xf5,
    /// Prefixes a byte-array value.
    Bytes = 0xf6,
    /// Prefixes a string value.
    Str = 0xf7,
    /// Represents `true`.
    BoolTrue = 0xf8,
    /// Represents `false`.
    BoolFalse = 0xf9,
    /// Prefixes an i64 value
    Int = 0xfa,
    /// Prefixes an f64 value
    Double = 0xfb,
    /// Marks the start of a map (key-value list) value.
    MapPrefix = 0xfc,
    /// Marks the end of a map (key-value list) value.
    MapSuffix = 0xfd,
    /// Marks the start of an array (slice) value.
    ArrayPrefix = 0xfe,
    /// Marks the end of an array (slice) value.
    ArraySuffix = 0xff,
}

impl AttributeHash {
    /// The hash of an empty attribute set.
    pub const EMPTY: Self = Self([0u8; 16]);

    /// Compute a hash from an iterator of [`AttributeView`] items.
    pub fn compute<A: AttributeView>(buf: &mut HashBuffer, attrs: impl Iterator<Item = A>) -> Self {
        buf.buf.clear();

        let mut entries: Vec<A> = attrs.collect();

        if entries.is_empty() {
            return Self::EMPTY;
        }

        entries.sort_by(|a, b| a.key().cmp(b.key()));

        for attr in &entries {
            buf.buf.push(HashTag::Key as u8);
            write_len(&mut buf.buf, attr.key().len());
            buf.buf.extend_from_slice(attr.key());
            write_attr_value(&mut buf.buf, attr);
        }

        Self(xxhash_rust::xxh3::xxh3_128(&buf.buf).to_le_bytes())
    }
}

/// Write an attribute's value into the hash buffer with type-discriminator
/// prefixes. Delegates to [`write_value`] for the actual value encoding.
fn write_attr_value<A: AttributeView>(buf: &mut Vec<u8>, attr: &A) {
    let Some(val) = attr.value() else {
        buf.push(HashTag::Empty as u8);
        return;
    };
    write_value(buf, &val);
}

/// Recursively encode an [`AnyValueView`] into the hash buffer.
///
/// Scalar types are encoded with a type-discriminator prefix followed by
/// the value bytes. Composite types use prefix/suffix delimiters:
///
/// - Map (KeyValueList): `MapPrefix`, then key-value pairs sorted by key
/// - Array: `ArrayPrefix`, then each element recursively encoded in order,
///   then `ArraySuffix`.
fn write_value<'a>(buf: &mut Vec<u8>, val: &impl AnyValueView<'a>) {
    match val.value_type() {
        ValueType::String => {
            buf.push(HashTag::Str as u8);
            if let Some(s) = val.as_string() {
                write_len(buf, s.len());
                buf.extend_from_slice(s);
            }
        }
        ValueType::Bool => {
            if val.as_bool().unwrap_or(false) {
                buf.push(HashTag::BoolTrue as u8);
            } else {
                buf.push(HashTag::BoolFalse as u8);
            }
        }
        ValueType::Int64 => {
            buf.push(HashTag::Int as u8);
            buf.extend_from_slice(&val.as_int64().unwrap_or(0).to_le_bytes());
        }
        ValueType::Double => {
            buf.push(HashTag::Double as u8);
            buf.extend_from_slice(&val.as_double().unwrap_or(0.0).to_bits().to_le_bytes());
        }
        ValueType::Bytes => {
            buf.push(HashTag::Bytes as u8);
            if let Some(b) = val.as_bytes() {
                write_len(buf, b.len());
                buf.extend_from_slice(b);
            }
        }
        ValueType::Empty => {
            buf.push(HashTag::Empty as u8);
        }
        ValueType::KeyValueList => {
            buf.push(HashTag::MapPrefix as u8);
            if let Some(entries) = val.as_kvlist() {
                let mut sorted: Vec<_> = entries.collect();
                sorted.sort_by(|a, b| a.key().cmp(b.key()));
                for entry in &sorted {
                    buf.push(HashTag::Key as u8);
                    write_len(buf, entry.key().len());
                    buf.extend_from_slice(entry.key());
                    write_attr_value(buf, entry);
                }
            }
            buf.push(HashTag::MapSuffix as u8);
        }
        ValueType::Array => {
            buf.push(HashTag::ArrayPrefix as u8);
            if let Some(elements) = val.as_array() {
                for element in elements {
                    write_value(buf, &element);
                }
            }
            buf.push(HashTag::ArraySuffix as u8);
        }
    }
}

fn write_len(vec: &mut Vec<u8>, len: usize) {
    vec.extend_from_slice(&(len as u32).to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_attributes() {
        let mut buf = HashBuffer::new();
        let hash = AttributeHash::compute(&mut buf, std::iter::empty::<TestAttr>());
        assert_eq!(hash, AttributeHash::EMPTY);
    }

    #[test]
    fn test_determinism() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(
            &mut buf,
            vec![str_attr("host", "a"), int_attr("port", 8080)].into_iter(),
        );
        let h2 = AttributeHash::compute(
            &mut buf,
            vec![str_attr("host", "a"), int_attr("port", 8080)].into_iter(),
        );
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_order_independence() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(
            &mut buf,
            vec![str_attr("a", "1"), str_attr("b", "2")].into_iter(),
        );
        let h2 = AttributeHash::compute(
            &mut buf,
            vec![str_attr("b", "2"), str_attr("a", "1")].into_iter(),
        );
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_distinctness() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(&mut buf, vec![str_attr("a", "1")].into_iter());
        let h2 = AttributeHash::compute(&mut buf, vec![str_attr("a", "2")].into_iter());
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_different_types_distinct() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(&mut buf, vec![str_attr("k", "42")].into_iter());
        let h2 = AttributeHash::compute(&mut buf, vec![int_attr("k", 42)].into_iter());
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_non_empty_hash() {
        let mut buf = HashBuffer::new();
        let hash = AttributeHash::compute(&mut buf, vec![str_attr("k", "v")].into_iter());
        assert_ne!(hash, AttributeHash::EMPTY);
    }

    #[test]
    fn test_scalar_value_types_produce_distinct_hashes() {
        let mut buf = HashBuffer::new();
        let hashes: Vec<_> = vec![
            AttributeHash::compute(&mut buf, vec![str_attr("k", "v")].into_iter()),
            AttributeHash::compute(&mut buf, vec![int_attr("k", 1)].into_iter()),
            AttributeHash::compute(&mut buf, vec![double_attr("k", 1.0)].into_iter()),
            AttributeHash::compute(&mut buf, vec![bool_attr("k", true)].into_iter()),
            AttributeHash::compute(&mut buf, vec![bytes_attr("k", b"v")].into_iter()),
            AttributeHash::compute(&mut buf, vec![empty_attr("k")].into_iter()),
        ];
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "hash[{i}] == hash[{j}]");
            }
        }
    }

    #[test]
    fn test_null_and_empty_are_equivalent() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(&mut buf, vec![empty_attr("k")].into_iter());
        let h2 = AttributeHash::compute(&mut buf, vec![null_attr("k")].into_iter());
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_bool_true_false_distinct() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(&mut buf, vec![bool_attr("k", true)].into_iter());
        let h2 = AttributeHash::compute(&mut buf, vec![bool_attr("k", false)].into_iter());
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_resource_id_equality() {
        let mut buf = HashBuffer::new();
        let r1 = ResourceId {
            attrs: AttributeHash::compute(&mut buf, vec![str_attr("host", "a")].into_iter()),
        };
        let r2 = ResourceId {
            attrs: AttributeHash::compute(&mut buf, vec![str_attr("host", "a")].into_iter()),
        };
        let r3 = ResourceId {
            attrs: AttributeHash::compute(&mut buf, vec![str_attr("host", "b")].into_iter()),
        };
        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }

    #[test]
    fn test_scope_id_equality() {
        let res = ResourceId {
            attrs: AttributeHash::EMPTY,
        };
        let s1 = make_scope(res, b"otel", b"1.0");
        let s2 = make_scope(res, b"otel", b"1.0");
        let s3 = make_scope(res, b"otel", b"2.0");
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_scope_id_different_resource() {
        let mut buf = HashBuffer::new();
        let r1 = ResourceId {
            attrs: AttributeHash::compute(&mut buf, vec![str_attr("a", "1")].into_iter()),
        };
        let r2 = ResourceId {
            attrs: AttributeHash::compute(&mut buf, vec![str_attr("a", "2")].into_iter()),
        };
        let s1 = make_scope(r1, b"otel", b"1.0");
        let s2 = make_scope(r2, b"otel", b"1.0");
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_metric_id_different_type() {
        let scope = make_scope(
            ResourceId {
                attrs: AttributeHash::EMPTY,
            },
            b"",
            b"",
        );
        let m1 = make_metric(
            scope.clone(),
            b"cpu",
            b"s",
            DataType::Sum,
            true,
            AggregationTemporality::Cumulative,
        );
        let m2 = make_metric(
            scope,
            b"cpu",
            b"s",
            DataType::Gauge,
            false,
            AggregationTemporality::Unspecified,
        );
        assert_ne!(m1, m2);
    }

    #[test]
    fn test_stream_id_different_dp_attrs() {
        let mut buf = HashBuffer::new();
        let scope = make_scope(
            ResourceId {
                attrs: AttributeHash::EMPTY,
            },
            b"",
            b"",
        );
        let metric = make_metric(
            scope,
            b"cpu",
            b"s",
            DataType::Sum,
            true,
            AggregationTemporality::Cumulative,
        );
        let s1 = StreamId {
            metric: metric.clone(),
            attrs: AttributeHash::compute(&mut buf, vec![str_attr("host", "a")].into_iter()),
        };
        let s2 = StreamId {
            metric,
            attrs: AttributeHash::compute(&mut buf, vec![str_attr("host", "b")].into_iter()),
        };
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_into_owned_preserves_equality() {
        let mut buf = HashBuffer::new();
        let scope = make_scope(
            ResourceId {
                attrs: AttributeHash::compute(&mut buf, vec![str_attr("h", "v")].into_iter()),
            },
            b"sc",
            b"1.0",
        );
        let metric = make_metric(
            scope,
            b"cpu",
            b"s",
            DataType::Sum,
            true,
            AggregationTemporality::Cumulative,
        );
        let stream = StreamId {
            metric,
            attrs: AttributeHash::compute(&mut buf, vec![str_attr("env", "prod")].into_iter()),
        };
        let owned = stream.clone().into_owned();
        assert_eq!(stream, owned);
    }

    #[test]
    fn test_buffer_reuse_produces_consistent_results() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(&mut buf, vec![str_attr("k", "v")].into_iter());
        // Hash something different to exercise the buffer
        let _ = AttributeHash::compute(&mut buf, vec![int_attr("x", 99)].into_iter());
        // Hash the original again -- should match
        let h2 = AttributeHash::compute(&mut buf, vec![str_attr("k", "v")].into_iter());
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_byte_array_collision() {
        // If we don't have a length in the encoding for byte arrays and keys
        // then it's easy to manufacture collisions like in the below case
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(
            &mut buf,
            vec![bytes_attr("a", &[0xF5, 0x62, 0xF6])].into_iter(),
        );
        let h2 = AttributeHash::compute(
            &mut buf,
            vec![bytes_attr("a", &[]), empty_attr("b")].into_iter(),
        );
        assert_ne!(h1, h2);
    }

    fn make_scope<'a>(resource: ResourceId, name: &'a [u8], version: &'a [u8]) -> ScopeId<'a> {
        ScopeId {
            resource,
            name: Cow::Borrowed(name),
            version: Cow::Borrowed(version),
            attrs: AttributeHash::EMPTY,
        }
    }

    fn make_metric<'a>(
        scope: ScopeId<'a>,
        name: &'a [u8],
        unit: &'a [u8],
        data_type: DataType,
        is_monotonic: bool,
        temporality: AggregationTemporality,
    ) -> MetricId<'a> {
        MetricId {
            scope,
            name: Cow::Borrowed(name),
            unit: Cow::Borrowed(unit),
            data_type: data_type as u8,
            is_monotonic,
            aggregation_temporality: temporality as u8,
        }
    }

    struct TestAttr {
        key: Vec<u8>,
        val: Option<TestValue>,
    }

    #[derive(Clone)]
    enum TestValue {
        Str(Vec<u8>),
        Int(i64),
        Double(f64),
        Bool(bool),
        Bytes(Vec<u8>),
        Empty,
        Array(Vec<TestValue>),
        Map(Vec<(Vec<u8>, TestValue)>),
    }

    struct TestAnyValue<'a>(&'a TestValue);

    impl<'a> AnyValueView<'a> for TestAnyValue<'a> {
        type KeyValue = TestAttr;
        type ArrayIter<'arr>
            = std::vec::IntoIter<TestAnyValue<'a>>
        where
            Self: 'arr;
        type KeyValueIter<'kv>
            = std::vec::IntoIter<TestAttr>
        where
            Self: 'kv;

        fn value_type(&self) -> ValueType {
            match self.0 {
                TestValue::Str(_) => ValueType::String,
                TestValue::Int(_) => ValueType::Int64,
                TestValue::Double(_) => ValueType::Double,
                TestValue::Bool(_) => ValueType::Bool,
                TestValue::Bytes(_) => ValueType::Bytes,
                TestValue::Empty => ValueType::Empty,
                TestValue::Array(_) => ValueType::Array,
                TestValue::Map(_) => ValueType::KeyValueList,
            }
        }
        fn as_string(&self) -> Option<&[u8]> {
            if let TestValue::Str(s) = self.0 {
                Some(s)
            } else {
                None
            }
        }
        fn as_bool(&self) -> Option<bool> {
            if let TestValue::Bool(b) = self.0 {
                Some(*b)
            } else {
                None
            }
        }
        fn as_int64(&self) -> Option<i64> {
            if let TestValue::Int(i) = self.0 {
                Some(*i)
            } else {
                None
            }
        }
        fn as_double(&self) -> Option<f64> {
            if let TestValue::Double(d) = self.0 {
                Some(*d)
            } else {
                None
            }
        }
        fn as_bytes(&self) -> Option<&[u8]> {
            if let TestValue::Bytes(b) = self.0 {
                Some(b)
            } else {
                None
            }
        }
        fn as_array(&self) -> Option<Self::ArrayIter<'_>> {
            if let TestValue::Array(items) = self.0 {
                let views: Vec<TestAnyValue<'_>> = items.iter().map(TestAnyValue).collect();
                Some(views.into_iter())
            } else {
                None
            }
        }
        fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
            if let TestValue::Map(entries) = self.0 {
                let attrs: Vec<TestAttr> = entries
                    .iter()
                    .map(|(k, v)| TestAttr {
                        key: k.clone(),
                        val: Some(v.clone()),
                    })
                    .collect();
                Some(attrs.into_iter())
            } else {
                None
            }
        }
    }

    impl AttributeView for TestAttr {
        type Val<'val>
            = TestAnyValue<'val>
        where
            Self: 'val;
        fn key(&self) -> &[u8] {
            &self.key
        }
        fn value(&self) -> Option<Self::Val<'_>> {
            self.val.as_ref().map(TestAnyValue)
        }
    }

    fn str_attr(key: &str, val: &str) -> TestAttr {
        TestAttr {
            key: key.as_bytes().to_vec(),
            val: Some(TestValue::Str(val.as_bytes().to_vec())),
        }
    }

    fn int_attr(key: &str, val: i64) -> TestAttr {
        TestAttr {
            key: key.as_bytes().to_vec(),
            val: Some(TestValue::Int(val)),
        }
    }

    fn double_attr(key: &str, val: f64) -> TestAttr {
        TestAttr {
            key: key.as_bytes().to_vec(),
            val: Some(TestValue::Double(val)),
        }
    }

    fn bool_attr(key: &str, val: bool) -> TestAttr {
        TestAttr {
            key: key.as_bytes().to_vec(),
            val: Some(TestValue::Bool(val)),
        }
    }

    fn bytes_attr(key: &str, val: &[u8]) -> TestAttr {
        TestAttr {
            key: key.as_bytes().to_vec(),
            val: Some(TestValue::Bytes(val.to_vec())),
        }
    }

    fn empty_attr(key: &str) -> TestAttr {
        TestAttr {
            key: key.as_bytes().to_vec(),
            val: Some(TestValue::Empty),
        }
    }

    fn null_attr(key: &str) -> TestAttr {
        TestAttr {
            key: key.as_bytes().to_vec(),
            val: None,
        }
    }

    fn list_attr(key: &str, values: Vec<TestValue>) -> TestAttr {
        TestAttr {
            key: key.as_bytes().to_vec(),
            val: Some(TestValue::Array(values)),
        }
    }

    fn map_attr(key: &str, entries: Vec<(&str, TestValue)>) -> TestAttr {
        TestAttr {
            key: key.as_bytes().to_vec(),
            val: Some(TestValue::Map(
                entries
                    .into_iter()
                    .map(|(k, v)| (k.as_bytes().to_vec(), v))
                    .collect(),
            )),
        }
    }

    #[test]
    fn test_array_value_produces_distinct_hash() {
        let mut buf = HashBuffer::new();
        let h_array = AttributeHash::compute(
            &mut buf,
            vec![list_attr("k", vec![TestValue::Int(1), TestValue::Int(2)])].into_iter(),
        );
        let h_empty = AttributeHash::compute(&mut buf, vec![empty_attr("k")].into_iter());
        assert_ne!(h_array, h_empty);
    }

    #[test]
    fn test_map_value_produces_distinct_hash() {
        let mut buf = HashBuffer::new();
        let h_map = AttributeHash::compute(
            &mut buf,
            vec![map_attr(
                "k",
                vec![("a", TestValue::Int(1)), ("b", TestValue::Int(2))],
            )]
            .into_iter(),
        );
        let h_empty = AttributeHash::compute(&mut buf, vec![empty_attr("k")].into_iter());
        assert_ne!(h_map, h_empty);
    }

    #[test]
    fn test_array_order_matters() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(
            &mut buf,
            vec![list_attr("k", vec![TestValue::Int(1), TestValue::Int(2)])].into_iter(),
        );
        let h2 = AttributeHash::compute(
            &mut buf,
            vec![list_attr("k", vec![TestValue::Int(2), TestValue::Int(1)])].into_iter(),
        );
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_map_order_independence() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(
            &mut buf,
            vec![map_attr(
                "k",
                vec![("a", TestValue::Int(1)), ("b", TestValue::Int(2))],
            )]
            .into_iter(),
        );
        let h2 = AttributeHash::compute(
            &mut buf,
            vec![map_attr(
                "k",
                vec![("b", TestValue::Int(2)), ("a", TestValue::Int(1))],
            )]
            .into_iter(),
        );
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_nested_map_in_array() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(
            &mut buf,
            vec![list_attr(
                "k",
                vec![TestValue::Map(vec![
                    (b"x".to_vec(), TestValue::Int(10)),
                    (b"y".to_vec(), TestValue::Int(20)),
                ])],
            )]
            .into_iter(),
        );
        let h2 = AttributeHash::compute(
            &mut buf,
            vec![list_attr(
                "k",
                vec![TestValue::Map(vec![
                    (b"x".to_vec(), TestValue::Int(10)),
                    (b"y".to_vec(), TestValue::Int(99)),
                ])],
            )]
            .into_iter(),
        );
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_nested_array_in_map() {
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(
            &mut buf,
            vec![map_attr(
                "k",
                vec![(
                    "items",
                    TestValue::Array(vec![TestValue::Int(1), TestValue::Int(2)]),
                )],
            )]
            .into_iter(),
        );
        let h2 = AttributeHash::compute(
            &mut buf,
            vec![map_attr(
                "k",
                vec![(
                    "items",
                    TestValue::Array(vec![TestValue::Int(3), TestValue::Int(4)]),
                )],
            )]
            .into_iter(),
        );
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_empty_array_vs_empty_map_distinct() {
        let mut buf = HashBuffer::new();
        let h_array = AttributeHash::compute(&mut buf, vec![list_attr("k", vec![])].into_iter());
        let h_map = AttributeHash::compute(&mut buf, vec![map_attr("k", vec![])].into_iter());
        assert_ne!(h_array, h_map);
    }

    #[test]
    fn test_array_vs_scalar_distinct() {
        let mut buf = HashBuffer::new();
        // [42] as an array with a single int element
        let h_array = AttributeHash::compute(
            &mut buf,
            vec![list_attr("k", vec![TestValue::Int(42)])].into_iter(),
        );
        // 42 as a bare int
        let h_scalar = AttributeHash::compute(&mut buf, vec![int_attr("k", 42)].into_iter());
        assert_ne!(h_array, h_scalar);
    }

    #[test]
    fn test_map_determinism() {
        let mut buf = HashBuffer::new();
        let make_attrs = || {
            vec![map_attr(
                "k",
                vec![
                    ("host", TestValue::Str(b"server1".to_vec())),
                    ("port", TestValue::Int(8080)),
                ],
            )]
        };
        let h1 = AttributeHash::compute(&mut buf, make_attrs().into_iter());
        let h2 = AttributeHash::compute(&mut buf, make_attrs().into_iter());
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_deeply_nested_structure() {
        // map containing an array containing another map
        let mut buf = HashBuffer::new();
        let h1 = AttributeHash::compute(
            &mut buf,
            vec![map_attr(
                "k",
                vec![(
                    "nested",
                    TestValue::Array(vec![TestValue::Map(vec![(
                        b"inner_key".to_vec(),
                        TestValue::Str(b"inner_val".to_vec()),
                    )])]),
                )],
            )]
            .into_iter(),
        );
        let h2 = AttributeHash::compute(
            &mut buf,
            vec![map_attr(
                "k",
                vec![(
                    "nested",
                    TestValue::Array(vec![TestValue::Map(vec![(
                        b"inner_key".to_vec(),
                        TestValue::Str(b"changed".to_vec()),
                    )])]),
                )],
            )]
            .into_iter(),
        );
        assert_ne!(h1, h2);
        // And it should be non-empty
        assert_ne!(h1, AttributeHash::EMPTY);
    }
}
