// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks request-scoped transport metadata from receiver to exporter.

use std::borrow::Cow;
use std::hint::black_box;
use std::mem::size_of;
use std::sync::Arc;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::transport_headers::{TransportHeader, TransportHeaders, ValueKind};
use otel_arrow_dfe_config::transport_headers_policy::{
    CaptureDefaults, CaptureRule, CompiledHeaderCapturePolicy, HeaderCapturePolicy,
    HeaderPropagationPolicy, NameStrategy, PropagationDefault, PropagationSelector,
    PropagationSelectorType,
};
use rdkafka::message::{Header, Headers, OwnedHeaders};
use tonic::metadata::{KeyAndValueRef, MetadataKey, MetadataMap, MetadataValue};

const HEADER_COUNTS: [usize; 4] = [1, 4, 16, 32];
const PRODUCER_CASES: [ProducerCase; 2] = [ProducerCase::Unrenamed, ProducerCase::Renamed];
// Stored-name and absent consumers compile the same receive policy.
const RECEIVE_CONSUMER_CASES: [ConsumerCase; 2] = [ConsumerCase::None, ConsumerCase::Original];
const CONSUMER_CASES: [ConsumerCase; 3] = [
    ConsumerCase::None,
    ConsumerCase::Normalized,
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
    Normalized,
    Original,
}

impl ConsumerCase {
    const fn name(self) -> &'static str {
        match self {
            Self::None => "no_consumer",
            Self::Normalized => "normalized_consumer",
            Self::Original => "original_consumer",
        }
    }

    fn propagation_policy(self) -> Option<HeaderPropagationPolicy> {
        let name = match self {
            Self::None => return None,
            Self::Normalized => NameStrategy::StoredName,
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
    bench_end_to_end(c);
    bench_receive_kafka_original(c);
    bench_end_to_end_kafka_original(c);
}

#[derive(Clone)]
struct LegacyHeaders {
    headers: Arc<Vec<LegacyTransportHeader>>,
}

#[derive(Clone)]
struct CurrentHeaders {
    headers: Arc<Vec<TransportHeader>>,
}

enum LegacyHeaderName {
    Prepared(ContextEntryName),
    Preserved {
        normal: ContextEntryName,
        original: Cow<'static, str>,
    },
}

impl LegacyHeaderName {
    fn normalized(&self) -> &ContextEntryName {
        match self {
            Self::Prepared(normal) | Self::Preserved { normal, .. } => normal,
        }
    }

    fn original(&self) -> &str {
        match self {
            Self::Prepared(normal) => normal,
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
            |normal, wire_name, value_kind, value| {
                let name = if normal.as_str() == wire_name {
                    LegacyHeaderName::Prepared(normal)
                } else {
                    LegacyHeaderName::Preserved {
                        normal,
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
) -> CurrentHeaders {
    CurrentHeaders {
        headers: capture_kafka_headers(
            match_names,
            headers,
            |normal, wire_name, value_kind, value| {
                TransportHeader::captured(normal, wire_name, true, value_kind, value)
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
        let Some(normal) = match_names
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
        captured.push(capture(normal.clone(), header.key, value_kind, value));
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

fn propagate_kafka_current(context: &CurrentHeaders) -> OwnedHeaders {
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
            normal: context_name("preserved"),
            original: "Preserved".to_owned().into(),
        },
    ];
    for variant in &variants {
        let _ = black_box(variant.normalized());
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
