// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks request-scoped transport metadata from receiver to exporter.

use std::borrow::Cow;
use std::hint::black_box;
use std::mem::size_of;
use std::sync::Arc;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use http::{HeaderMap, HeaderName, HeaderValue};
use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::transport_headers::{TransportHeader, TransportHeaders, ValueKind};
use otel_arrow_dfe_config::transport_headers_policy::{
    CaptureDefaults, CaptureRule, CompiledHeaderCapturePolicy, HeaderCapturePolicy,
    HeaderPropagationPolicy, NameStrategy, PropagationDefault, PropagationSelector,
    PropagationSelectorType,
};
use rdkafka::message::{Header, Headers, OwnedHeaders};
use tonic::metadata::{KeyAndValueRef, MetadataKey, MetadataMap, MetadataValue};

const HEADER_COUNTS: [usize; 7] = [1, 2, 4, 5, 6, 16, 32];
const PRODUCER_CASES: [ProducerCase; 2] = [ProducerCase::Unrenamed, ProducerCase::Renamed];
// Stored-name and absent consumers compile the same receive policy.
const RECEIVE_CONSUMER_CASES: [ConsumerCase; 2] = [ConsumerCase::None, ConsumerCase::Original];
const CONSUMER_CASES: [ConsumerCase; 3] = [
    ConsumerCase::None,
    ConsumerCase::Stored,
    ConsumerCase::Original,
];

#[derive(Clone, Copy)]
enum ProducerCase {
    Unrenamed,
    Renamed,
}

impl ProducerCase {
    const fn name(self) -> &'static str {
        match self {
            Self::Unrenamed => "unrenamed",
            Self::Renamed => "renamed",
        }
    }
}

#[derive(Clone, Copy)]
enum ConsumerCase {
    None,
    Stored,
    Original,
}

impl ConsumerCase {
    const fn name(self) -> &'static str {
        match self {
            Self::None => "no_consumer",
            Self::Stored => "stored_name_consumer",
            Self::Original => "original_consumer",
        }
    }

    fn propagation_policy(self) -> Option<HeaderPropagationPolicy> {
        let name = match self {
            Self::None => return None,
            Self::Stored => NameStrategy::StoredName,
            Self::Original => NameStrategy::Preserve,
        };
        Some(HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                name,
                ..PropagationDefault::default()
            },
            vec![],
        ))
    }

    const fn preserves_original_names(self) -> bool {
        matches!(self, Self::Original)
    }
}

fn case_name(producer: ProducerCase, consumer: ConsumerCase) -> String {
    format!("{}/{}", producer.name(), consumer.name())
}

fn context_name(raw: impl AsRef<str>) -> ContextEntryName {
    ContextEntryName::try_from(raw.as_ref()).expect("valid benchmark context entry name")
}

fn main_benchmarks(c: &mut Criterion) {
    bench_receive(c);
    bench_receive_http(c);
    bench_end_to_end(c);
    bench_lookup_clone_and_append(c);
    bench_receive_kafka_original(c);
    bench_end_to_end_kafka_original(c);
}

fn bench_receive_http(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/receive_http");
    for header_count in HEADER_COUNTS {
        for producer in PRODUCER_CASES {
            for consumer in RECEIVE_CONSUMER_CASES {
                let preserve_original_names = consumer.preserves_original_names();
                let capture =
                    capture_policy(header_count, producer).compile(|_| preserve_original_names);
                let headers = inbound_http_headers(header_count);
                let _ = group.bench_with_input(
                    BenchmarkId::new(
                        format!("generic_string_adapter/{}", case_name(producer, consumer)),
                        header_count,
                    ),
                    &header_count,
                    |b, _| {
                        b.iter(|| {
                            let mut context = TransportHeaders::new();
                            let pairs = black_box(&headers)
                                .iter()
                                .map(|(name, value)| (name.as_str(), value.as_bytes()));
                            let _ = black_box(&capture).capture_from_pairs(pairs, &mut context);
                            black_box(context)
                        });
                    },
                );
                let _ = group.bench_with_input(
                    BenchmarkId::new(
                        format!("native_header_name/{}", case_name(producer, consumer)),
                        header_count,
                    ),
                    &header_count,
                    |b, _| {
                        b.iter(|| {
                            let mut context = TransportHeaders::new();
                            let _ = black_box(&capture)
                                .capture_from_http_headers(black_box(&headers), &mut context);
                            black_box(context)
                        });
                    },
                );
            }
        }
    }
    group.finish();
}

fn bench_lookup_clone_and_append(c: &mut Criterion) {
    assert_eq!(size_of::<TransportHeaders>(), size_of::<usize>());

    {
        let mut group = c.benchmark_group("request_context/lookup");
        for header_count in HEADER_COUNTS {
            let (packed, previous, lookup_name) = comparison_headers(header_count);
            let _ = group.bench_with_input(
                BenchmarkId::new("previous_arc_vec", header_count),
                &header_count,
                |b, _| {
                    b.iter(|| {
                        black_box(&previous)
                            .find_by_name(black_box(&lookup_name))
                            .map(|header| header.value.bytes.len())
                    });
                },
            );
            let _ = group.bench_with_input(
                BenchmarkId::new("packed", header_count),
                &header_count,
                |b, _| {
                    b.iter(|| {
                        black_box(&packed)
                            .find_by_name(black_box(&lookup_name))
                            .next()
                            .map(|header| header.value.bytes.len())
                    });
                },
            );
        }
        group.finish();
    }

    {
        let mut group = c.benchmark_group("request_context/clone");
        for header_count in HEADER_COUNTS {
            let (packed, previous, _) = comparison_headers(header_count);
            let _ = group.bench_with_input(
                BenchmarkId::new("previous_arc_vec", header_count),
                &header_count,
                |b, _| b.iter(|| black_box(previous.clone())),
            );
            let _ = group.bench_with_input(
                BenchmarkId::new("packed", header_count),
                &header_count,
                |b, _| b.iter(|| black_box(packed.clone())),
            );
        }
        group.finish();
    }

    {
        let mut group = c.benchmark_group("request_context/append_after_capture");
        for header_count in HEADER_COUNTS {
            let (packed, previous, _) = comparison_headers(header_count);
            let _ = group.bench_with_input(
                BenchmarkId::new("previous_arc_vec", header_count),
                &header_count,
                |b, _| {
                    b.iter_batched(
                        || (previous.clone(), partition_header()),
                        |(mut headers, header)| {
                            headers.push(header);
                            black_box(headers)
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
            let _ = group.bench_with_input(
                BenchmarkId::new("packed", header_count),
                &header_count,
                |b, _| {
                    b.iter_batched(
                        || (packed.clone(), partition_header()),
                        |(mut headers, header)| {
                            headers.push(header);
                            black_box(headers)
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
        group.finish();
    }
}

#[derive(Clone)]
struct LegacyHeaders {
    headers: Arc<Vec<LegacyTransportHeader>>,
}

#[derive(Clone)]
struct ArcVecHeaders {
    headers: Arc<Vec<TransportHeader>>,
}

impl ArcVecHeaders {
    fn find_by_name(&self, name: &str) -> Option<&TransportHeader> {
        self.headers
            .iter()
            .find(|header| header.name.as_str() == name)
    }

    fn push(&mut self, header: TransportHeader) {
        Arc::make_mut(&mut self.headers).push(header);
    }
}

enum LegacyHeaderName {
    Prepared(ContextEntryName),
    Preserved {
        stored: ContextEntryName,
        original: Cow<'static, str>,
    },
}

impl LegacyHeaderName {
    fn stored(&self) -> &ContextEntryName {
        match self {
            Self::Prepared(stored) | Self::Preserved { stored, .. } => stored,
        }
    }

    fn original(&self) -> &str {
        match self {
            Self::Prepared(stored) => stored,
            Self::Preserved { original, .. } => original,
        }
    }
}

struct LegacyTransportHeader {
    name: LegacyHeaderName,
    value_kind: ValueKind,
    value: Box<[u8]>,
}

fn bench_receive(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/receive_grpc");
    for header_count in HEADER_COUNTS {
        for producer in PRODUCER_CASES {
            for consumer in RECEIVE_CONSUMER_CASES {
                let preserve_original_names = consumer.preserves_original_names();
                let capture =
                    capture_policy(header_count, producer).compile(|_| preserve_original_names);
                let metadata = inbound_metadata(header_count);
                let _ = group.bench_with_input(
                    BenchmarkId::new(case_name(producer, consumer), header_count),
                    &header_count,
                    |b, _| {
                        b.iter(|| {
                            black_box(receive_metadata(black_box(&capture), black_box(&metadata)))
                        });
                    },
                );
            }
        }
    }
    group.finish();
}

fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/end_to_end_grpc");
    for header_count in HEADER_COUNTS {
        for producer in PRODUCER_CASES {
            for consumer in CONSUMER_CASES {
                let propagation = consumer.propagation_policy();
                let preserve_original_names = consumer.preserves_original_names();
                let capture =
                    capture_policy(header_count, producer).compile(|_| preserve_original_names);
                let metadata = inbound_metadata(header_count);
                let _ = group.bench_with_input(
                    BenchmarkId::new(case_name(producer, consumer), header_count),
                    &header_count,
                    |b, _| {
                        b.iter(|| {
                            let context =
                                receive_metadata(black_box(&capture), black_box(&metadata));
                            let hop1 = context.clone();
                            let hop2 = hop1.clone();
                            if let Some(propagation) = &propagation {
                                let _ = black_box(propagate_metadata(
                                    black_box(&hop2),
                                    black_box(propagation),
                                ));
                            } else {
                                let _ = black_box(hop2);
                            }
                        });
                    },
                );
            }
        }
    }
    group.finish();
}

fn bench_receive_kafka_original(c: &mut Criterion) {
    assert_header_layout_improved();
    let mut group = c.benchmark_group("request_context/receive_kafka_original");
    for header_count in HEADER_COUNTS {
        let match_names = capture_match_names(header_count);
        let headers = inbound_kafka_headers(header_count);

        let _ = group.bench_with_input(
            BenchmarkId::new("legacy_header_name_enum", header_count),
            &header_count,
            |b, _| {
                b.iter(|| {
                    black_box(receive_kafka_legacy(
                        black_box(&match_names),
                        black_box(&headers),
                    ))
                });
            },
        );
        let _ = group.bench_with_input(
            BenchmarkId::new("current_header_value", header_count),
            &header_count,
            |b, _| {
                b.iter(|| {
                    black_box(receive_kafka_current(
                        black_box(&match_names),
                        black_box(&headers),
                    ))
                });
            },
        );
    }
    group.finish();
}

fn bench_end_to_end_kafka_original(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/end_to_end_kafka_original");
    for header_count in HEADER_COUNTS {
        let match_names = capture_match_names(header_count);
        let headers = inbound_kafka_headers(header_count);

        let _ = group.bench_with_input(
            BenchmarkId::new("legacy_header_name_enum", header_count),
            &header_count,
            |b, _| {
                b.iter(|| {
                    let context =
                        receive_kafka_legacy(black_box(&match_names), black_box(&headers));
                    let hop1 = context.clone();
                    let hop2 = hop1.clone();
                    black_box(propagate_kafka_legacy(black_box(&hop2)))
                });
            },
        );
        let _ = group.bench_with_input(
            BenchmarkId::new("current_header_value", header_count),
            &header_count,
            |b, _| {
                b.iter(|| {
                    let context =
                        receive_kafka_current(black_box(&match_names), black_box(&headers));
                    let hop1 = context.clone();
                    let hop2 = hop1.clone();
                    black_box(propagate_kafka_current(black_box(&hop2)))
                });
            },
        );
    }
    group.finish();
}

fn capture_policy(header_count: usize, producer: ProducerCase) -> HeaderCapturePolicy {
    let rules = (0..header_count)
        .map(|index| {
            let wire_name = format!("x-context-{index}");
            let store_as = match producer {
                // Omitted and same-name store_as values compile identically.
                ProducerCase::Unrenamed => None,
                ProducerCase::Renamed => Some(context_name(format!("context_{index}"))),
            };
            CaptureRule {
                match_names: vec![context_name(wire_name)],
                store_as,
                sensitive: false,
                value_kind: None,
            }
        })
        .collect();
    HeaderCapturePolicy::new(CaptureDefaults::default(), rules)
}

fn inbound_metadata(header_count: usize) -> MetadataMap {
    let mut metadata = MetadataMap::with_capacity(header_count + 4);
    for index in 0..header_count {
        append_text_metadata(
            &mut metadata,
            &format!("x-context-{index}"),
            format!("value-{index:02}-0123456789abcdef").as_bytes(),
        );
    }
    for (name, value) in [
        ("content-type", b"application/grpc".as_slice()),
        ("user-agent", b"otel-collector/0.99.0".as_slice()),
        ("grpc-encoding", b"gzip".as_slice()),
        ("grpc-timeout", b"30S".as_slice()),
    ] {
        append_text_metadata(&mut metadata, name, value);
    }
    metadata
}

fn inbound_http_headers(header_count: usize) -> HeaderMap {
    let mut headers = HeaderMap::with_capacity(header_count + 4);
    for index in 0..header_count {
        let name = HeaderName::try_from(format!("x-context-{index}"))
            .expect("valid benchmark HTTP header name");
        let value = HeaderValue::try_from(format!("value-{index:02}-0123456789abcdef"))
            .expect("valid benchmark HTTP header value");
        let _ = headers.append(name, value);
    }
    for (name, value) in [
        ("content-type", "application/json"),
        ("user-agent", "otel-collector/0.99.0"),
        ("content-encoding", "gzip"),
        ("x-timeout", "30S"),
    ] {
        let _ = headers.append(
            HeaderName::from_static(name),
            HeaderValue::from_static(value),
        );
    }
    headers
}

fn previous_headers(header_count: usize) -> ArcVecHeaders {
    ArcVecHeaders {
        headers: Arc::new(
            (0..header_count)
                .map(|index| {
                    TransportHeader::captured(
                        context_name(format!("context_{index}")),
                        &format!("x-context-{index}"),
                        true,
                        ValueKind::Text,
                        format!("value-{index:02}-0123456789abcdef").as_bytes(),
                    )
                })
                .collect(),
        ),
    }
}

fn comparison_headers(header_count: usize) -> (TransportHeaders, ArcVecHeaders, String) {
    let capture = capture_policy(header_count, ProducerCase::Renamed).compile(|_| true);
    let metadata = inbound_metadata(header_count);
    (
        receive_metadata(&capture, &metadata),
        previous_headers(header_count),
        format!("context_{}", header_count - 1),
    )
}

fn partition_header() -> TransportHeader {
    TransportHeader::text(context_name("partition"), b"partition-0")
}

fn capture_match_names(header_count: usize) -> Vec<ContextEntryName> {
    (0..header_count)
        .map(|index| context_name(format!("x-context-{index}")))
        .collect()
}

fn inbound_kafka_headers(header_count: usize) -> OwnedHeaders {
    let mut headers = OwnedHeaders::new_with_capacity(header_count + 1);
    headers = headers.insert(Header {
        key: "encoding",
        value: Some(b"otlp".as_slice()),
    });
    for index in 0..header_count {
        let key = format!("X-Context-{index}");
        let value = format!("value-{index:02}-0123456789abcdef");
        headers = headers.insert(Header {
            key: &key,
            value: Some(value.as_bytes()),
        });
    }
    headers
}

fn receive_metadata(
    policy: &CompiledHeaderCapturePolicy,
    metadata: &MetadataMap,
) -> TransportHeaders {
    let pairs = metadata.iter().filter_map(|entry| match entry {
        KeyAndValueRef::Ascii(key, value) => Some((key.as_str(), Cow::Borrowed(value.as_bytes()))),
        KeyAndValueRef::Binary(key, value) => value
            .to_bytes()
            .ok()
            .map(|decoded| (key.as_str(), Cow::Owned(decoded.to_vec()))),
    });
    let mut context = TransportHeaders::new();
    let _ = policy.capture_from_pairs(pairs, &mut context);
    context
}

fn receive_kafka_legacy(match_names: &[ContextEntryName], headers: &OwnedHeaders) -> LegacyHeaders {
    LegacyHeaders {
        headers: capture_kafka_headers(
            match_names,
            headers,
            |stored, wire_name, value_kind, value| {
                let name = if stored.as_str() == wire_name {
                    LegacyHeaderName::Prepared(stored)
                } else {
                    LegacyHeaderName::Preserved {
                        stored,
                        original: wire_name.to_owned().into(),
                    }
                };
                LegacyTransportHeader {
                    name,
                    value_kind,
                    value: value.into(),
                }
            },
        ),
    }
}

fn receive_kafka_current(
    match_names: &[ContextEntryName],
    headers: &OwnedHeaders,
) -> ArcVecHeaders {
    ArcVecHeaders {
        headers: capture_kafka_headers(
            match_names,
            headers,
            |stored, wire_name, value_kind, value| {
                TransportHeader::captured(stored, wire_name, true, value_kind, value)
            },
        ),
    }
}

fn capture_kafka_headers<T>(
    match_names: &[ContextEntryName],
    headers: &OwnedHeaders,
    mut capture: impl FnMut(ContextEntryName, &str, ValueKind, &[u8]) -> T,
) -> Arc<Vec<T>> {
    let mut captured = Vec::with_capacity(headers.count());
    for header in headers.iter() {
        let Some(value) = header.value else {
            continue;
        };
        let Some(stored) = match_names
            .iter()
            .find(|name| header.key.eq_ignore_ascii_case(name))
        else {
            continue;
        };
        let value_kind = if header.key.ends_with("-bin") {
            ValueKind::Binary
        } else {
            ValueKind::Text
        };
        captured.push(capture(stored.clone(), header.key, value_kind, value));
    }
    Arc::new(captured)
}

fn propagate_metadata(context: &TransportHeaders, policy: &HeaderPropagationPolicy) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    for header in policy.propagate(context) {
        append_text_metadata(&mut metadata, header.header_name, header.value);
    }
    metadata
}

fn propagate_kafka_current(context: &ArcVecHeaders) -> OwnedHeaders {
    let mut headers = OwnedHeaders::new().insert(Header {
        key: "encoding",
        value: Some(b"otlp".as_slice()),
    });
    for header in context.headers.iter() {
        let _ = black_box(&header.value.value_kind);
        headers = headers.insert(Header {
            key: header.wire_name(),
            value: Some(header.value.bytes.as_ref()),
        });
    }
    headers
}

fn propagate_kafka_legacy(context: &LegacyHeaders) -> OwnedHeaders {
    let mut headers = OwnedHeaders::new().insert(Header {
        key: "encoding",
        value: Some(b"otlp".as_slice()),
    });
    for header in context.headers.iter() {
        let _ = black_box(&header.value_kind);
        headers = headers.insert(Header {
            key: header.name.original(),
            value: Some(header.value.as_ref()),
        });
    }
    headers
}

fn assert_header_layout_improved() {
    let variants = [
        LegacyHeaderName::Prepared(context_name("prepared")),
        LegacyHeaderName::Preserved {
            stored: context_name("preserved"),
            original: "Preserved".to_owned().into(),
        },
    ];
    for variant in &variants {
        let _ = black_box(variant.stored());
        let _ = black_box(variant.original());
    }
    assert!(size_of::<LegacyHeaderName>() > size_of::<ContextEntryName>());
    assert!(size_of::<LegacyTransportHeader>() > size_of::<TransportHeader>());
}

fn append_text_metadata(metadata: &mut MetadataMap, name: &str, value: &[u8]) {
    let key = name
        .parse::<MetadataKey<tonic::metadata::Ascii>>()
        .expect("valid benchmark metadata key");
    let value = MetadataValue::try_from(value).expect("valid benchmark metadata value");
    let _ = metadata.append(key, value);
}

criterion_group!(benches, main_benchmarks);
criterion_main!(benches);
