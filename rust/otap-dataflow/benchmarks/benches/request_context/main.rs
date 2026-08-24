// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks request-scoped transport metadata from receiver to exporter.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otel_arrow_dfe_config::transport_headers::TransportHeaders;
use otel_arrow_dfe_config::transport_headers_policy::{
    CaptureDefaults, CaptureRule, HeaderCapturePolicy, HeaderPropagationPolicy, PropagationDefault,
    PropagationSelector, PropagationSelectorType,
};
use tonic::metadata::{KeyAndValueRef, MetadataKey, MetadataMap, MetadataValue};

const HEADER_COUNTS: [usize; 4] = [1, 4, 16, 32];

fn main_benchmarks(c: &mut Criterion) {
    bench_receive(c);
    bench_end_to_end(c);
}

fn bench_receive(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/receive_grpc");
    for header_count in HEADER_COUNTS {
        let capture = capture_policy(header_count);
        let metadata = inbound_metadata(header_count);
        let _ = group.bench_with_input(
            BenchmarkId::from_parameter(header_count),
            &header_count,
            |b, _| {
                b.iter(|| black_box(receive_metadata(black_box(&capture), black_box(&metadata))));
            },
        );
    }
    group.finish();
}

fn bench_end_to_end(c: &mut Criterion) {
    let propagation = propagation_policy();
    let mut group = c.benchmark_group("request_context/end_to_end_grpc");
    for header_count in HEADER_COUNTS {
        let capture = capture_policy(header_count);
        let metadata = inbound_metadata(header_count);
        let _ = group.bench_with_input(
            BenchmarkId::from_parameter(header_count),
            &header_count,
            |b, _| {
                b.iter(|| {
                    let context = receive_metadata(black_box(&capture), black_box(&metadata));
                    let hop1 = context.clone();
                    let hop2 = hop1.clone();
                    black_box(propagate_metadata(
                        black_box(&hop2),
                        black_box(&propagation),
                    ))
                });
            },
        );
    }
    group.finish();
}

fn capture_policy(header_count: usize) -> HeaderCapturePolicy {
    let rules = (0..header_count)
        .map(|index| CaptureRule {
            match_names: vec![format!("x-context-{index}")],
            store_as: Some(format!("context_{index}")),
            sensitive: false,
            value_kind: None,
        })
        .collect();
    HeaderCapturePolicy::new(CaptureDefaults::default(), rules)
}

fn propagation_policy() -> HeaderPropagationPolicy {
    HeaderPropagationPolicy::new(
        PropagationDefault {
            selector: PropagationSelector {
                selector_type: PropagationSelectorType::AllCaptured,
                named: None,
            },
            ..PropagationDefault::default()
        },
        vec![],
    )
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

fn receive_metadata(policy: &HeaderCapturePolicy, metadata: &MetadataMap) -> TransportHeaders {
    let pairs: Vec<(&str, Vec<u8>)> = metadata
        .iter()
        .filter_map(|entry| match entry {
            KeyAndValueRef::Ascii(key, value) => Some((key.as_str(), value.as_bytes().to_vec())),
            KeyAndValueRef::Binary(key, value) => value
                .to_bytes()
                .ok()
                .map(|decoded| (key.as_str(), decoded.to_vec())),
        })
        .collect();
    let mut context = TransportHeaders::new();
    let _ = policy.capture_from_pairs(
        pairs.iter().map(|(name, value)| (*name, value.as_slice())),
        &mut context,
    );
    context
}

fn propagate_metadata(context: &TransportHeaders, policy: &HeaderPropagationPolicy) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    for header in policy.propagate(context) {
        append_text_metadata(&mut metadata, header.header_name, header.value);
    }
    metadata
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
