// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks request-scoped transport metadata from receiver to exporter.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otel_arrow_dfe_config::transport_headers_policy::{
    CaptureDefaults, CaptureRule, HeaderCapturePolicy, HeaderPropagationPolicy, PropagationDefault,
    PropagationSelector, PropagationSelectorType,
};
use tonic::metadata::{KeyAndValueRef, MetadataKey, MetadataMap, MetadataValue};

const HEADER_COUNTS: [usize; 4] = [1, 4, 16, 32];

fn main_benchmarks(c: &mut Criterion) {
    bench_capture(c);
    bench_receive(c);
    bench_carry(c);
    bench_propagate(c);
    bench_end_to_end(c);
}

fn bench_capture(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/capture");
    for header_count in HEADER_COUNTS {
        let capture = implementation::compile_capture(capture_policy(header_count));
        let pairs = inbound_pairs(header_count);
        let _ = group.bench_with_input(
            BenchmarkId::from_parameter(header_count),
            &header_count,
            |b, _| {
                b.iter(|| {
                    implementation::capture_pairs(black_box(&capture), black_box(pairs.as_slice()))
                });
            },
        );
    }
    group.finish();
}

fn bench_receive(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/receive_grpc");
    for header_count in HEADER_COUNTS {
        let capture = implementation::compile_capture(capture_policy(header_count));
        let metadata = inbound_metadata(header_count);
        let _ = group.bench_with_input(
            BenchmarkId::from_parameter(header_count),
            &header_count,
            |b, _| {
                b.iter(|| {
                    let pairs = decode_metadata(black_box(&metadata));
                    implementation::capture_decoded(black_box(&capture), black_box(&pairs))
                });
            },
        );
    }
    group.finish();
}

fn bench_carry(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/carry");
    for header_count in HEADER_COUNTS {
        let capture = implementation::compile_capture(capture_policy(header_count));
        let pairs = inbound_pairs(header_count);
        let context = implementation::capture_pairs(&capture, &pairs);
        let _ = group.bench_with_input(
            BenchmarkId::from_parameter(header_count),
            &header_count,
            |b, _| b.iter(|| black_box(context.clone())),
        );
    }
    group.finish();
}

fn bench_propagate(c: &mut Criterion) {
    let propagation = implementation::compile_propagation(propagation_policy());
    let mut group = c.benchmark_group("request_context/propagate_grpc");
    for header_count in HEADER_COUNTS {
        let capture = implementation::compile_capture(capture_policy(header_count));
        let pairs = inbound_pairs(header_count);
        let context = implementation::capture_pairs(&capture, &pairs);
        let _ = group.bench_with_input(
            BenchmarkId::from_parameter(header_count),
            &header_count,
            |b, _| {
                b.iter(|| {
                    implementation::propagate_metadata(black_box(&context), black_box(&propagation))
                });
            },
        );
    }
    group.finish();
}

fn bench_end_to_end(c: &mut Criterion) {
    let propagation = implementation::compile_propagation(propagation_policy());
    let mut group = c.benchmark_group("request_context/end_to_end_grpc");
    for header_count in HEADER_COUNTS {
        let capture = implementation::compile_capture(capture_policy(header_count));
        let metadata = inbound_metadata(header_count);
        let _ = group.bench_with_input(
            BenchmarkId::from_parameter(header_count),
            &header_count,
            |b, _| {
                b.iter(|| {
                    let pairs = decode_metadata(black_box(&metadata));
                    let context =
                        implementation::capture_decoded(black_box(&capture), black_box(&pairs));
                    let hop1 = context.clone();
                    let hop2 = hop1.clone();
                    implementation::propagate_metadata(black_box(&hop2), black_box(&propagation))
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

fn inbound_pairs(header_count: usize) -> Vec<(String, Vec<u8>)> {
    let mut pairs = Vec::with_capacity(header_count + 4);
    for index in 0..header_count {
        pairs.push((
            format!("x-context-{index}"),
            format!("value-{index:02}-0123456789abcdef").into_bytes(),
        ));
    }
    pairs.extend([
        ("content-type".to_string(), b"application/grpc".to_vec()),
        ("user-agent".to_string(), b"otel-collector/0.99.0".to_vec()),
        ("grpc-encoding".to_string(), b"gzip".to_vec()),
        ("grpc-timeout".to_string(), b"30S".to_vec()),
    ]);
    pairs
}

fn inbound_metadata(header_count: usize) -> MetadataMap {
    let mut metadata = MetadataMap::with_capacity(header_count + 4);
    for (name, value) in inbound_pairs(header_count) {
        let key = name
            .parse::<MetadataKey<tonic::metadata::Ascii>>()
            .expect("valid benchmark metadata key");
        let value = MetadataValue::try_from(value).expect("valid benchmark metadata value");
        let _ = metadata.append(key, value);
    }
    metadata
}

fn decode_metadata(metadata: &MetadataMap) -> Vec<(&str, Vec<u8>)> {
    metadata
        .iter()
        .filter_map(|entry| match entry {
            KeyAndValueRef::Ascii(key, value) => Some((key.as_str(), value.as_bytes().to_vec())),
            KeyAndValueRef::Binary(key, value) => value
                .to_bytes()
                .ok()
                .map(|decoded| (key.as_str(), decoded.to_vec())),
        })
        .collect()
}

fn append_text_metadata(metadata: &mut MetadataMap, name: &str, value: &[u8]) {
    let key = name
        .parse::<MetadataKey<tonic::metadata::Ascii>>()
        .expect("captured benchmark metadata key remains valid");
    let value =
        MetadataValue::try_from(value).expect("captured benchmark metadata value remains valid");
    let _ = metadata.append(key, value);
}

#[cfg(not(packed_pdata_context))]
mod implementation {
    use super::*;
    use otel_arrow_dfe_config::transport_headers::TransportHeaders;

    pub(super) type CapturePolicy = HeaderCapturePolicy;
    pub(super) type PropagationPolicy = HeaderPropagationPolicy;
    pub(super) type Context = TransportHeaders;

    pub(super) fn compile_capture(policy: HeaderCapturePolicy) -> CapturePolicy {
        policy
    }

    pub(super) fn compile_propagation(policy: HeaderPropagationPolicy) -> PropagationPolicy {
        policy
    }

    pub(super) fn capture_pairs(policy: &CapturePolicy, pairs: &[(String, Vec<u8>)]) -> Context {
        let mut context = TransportHeaders::new();
        let _ = policy.capture_from_pairs(
            pairs
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_slice())),
            &mut context,
        );
        context
    }

    pub(super) fn capture_decoded(policy: &CapturePolicy, pairs: &[(&str, Vec<u8>)]) -> Context {
        let mut context = TransportHeaders::new();
        let _ = policy.capture_from_pairs(
            pairs.iter().map(|(name, value)| (*name, value.as_slice())),
            &mut context,
        );
        context
    }

    pub(super) fn propagate_metadata(context: &Context, policy: &PropagationPolicy) -> MetadataMap {
        let mut metadata = MetadataMap::new();
        for header in policy.propagate(context) {
            append_text_metadata(&mut metadata, header.header_name, header.value);
        }
        metadata
    }
}

#[cfg(packed_pdata_context)]
mod implementation {
    use super::*;
    use otel_arrow_dfe_config::transport_headers_policy::{
        CompiledHeaderCapturePolicy, CompiledHeaderPropagationPolicy,
    };
    use otel_arrow_dfe_otap::context_bytes::PdataContextBytes;

    pub(super) type CapturePolicy = CompiledHeaderCapturePolicy;
    pub(super) type PropagationPolicy = CompiledHeaderPropagationPolicy;
    pub(super) type Context = PdataContextBytes;

    pub(super) fn compile_capture(policy: HeaderCapturePolicy) -> CapturePolicy {
        policy.compile().expect("valid benchmark capture policy")
    }

    pub(super) fn compile_propagation(policy: HeaderPropagationPolicy) -> PropagationPolicy {
        policy
            .compile()
            .expect("valid benchmark propagation policy")
    }

    pub(super) fn capture_pairs(policy: &CapturePolicy, pairs: &[(String, Vec<u8>)]) -> Context {
        PdataContextBytes::capture(
            policy,
            pairs
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_slice())),
        )
        .expect("benchmark context capture")
        .0
        .expect("benchmark captures matching headers")
    }

    pub(super) fn capture_decoded(policy: &CapturePolicy, pairs: &[(&str, Vec<u8>)]) -> Context {
        PdataContextBytes::capture(
            policy,
            pairs.iter().map(|(name, value)| (*name, value.as_slice())),
        )
        .expect("benchmark context capture")
        .0
        .expect("benchmark captures matching headers")
    }

    pub(super) fn propagate_metadata(context: &Context, policy: &PropagationPolicy) -> MetadataMap {
        let mut metadata = MetadataMap::new();
        for header in context.propagate(policy) {
            append_text_metadata(&mut metadata, header.header_name, header.value);
        }
        metadata
    }
}

criterion_group!(benches, main_benchmarks);
criterion_main!(benches);
