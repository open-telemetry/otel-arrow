// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks transport-header propagation selector costs.
//!
//! Run with:
//!
//! ```text
//! cargo bench -p otel-arrow-dfe-config --bench header_propagation
//! ```

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_config::context::{ContextEntryName, ContextEntryRef};
use otel_arrow_dfe_config::context_policy::{
    ContextEntryDeclaration, ContextEntryDefinition, ContextEntryPart, ContextScope,
};
use otel_arrow_dfe_config::transport_headers::{TransportHeader, TransportHeaders};
use otel_arrow_dfe_config::transport_headers_policy::HeaderPropagationPolicy;
use std::hint::black_box;

const HEADER_COUNTS: [usize; 4] = [1, 4, 16, 32];
const CONDITION_COUNTS: [usize; 4] = [1, 2, 3, 4];
const CONDITION_NAME_VARIANTS: [&str; 4] = ["header", "HEADER", "Header", "hEaDeR"];

fn bench_header_propagation(c: &mut Criterion) {
    let mut group = c.benchmark_group("header_propagation");

    for header_count in HEADER_COUNTS {
        let headers = headers(header_count);
        let _ = group.throughput(Throughput::Elements(header_count as u64));

        let unqualified = unqualified_policy(header_count);
        assert_eq!(unqualified.propagate(&headers).count(), 1);
        let _ = group.bench_with_input(
            BenchmarkId::new("unqualified", format!("{header_count}_headers")),
            &headers,
            |b, headers| {
                b.iter(|| black_box(unqualified.propagate(black_box(headers)).count()));
            },
        );

        for condition_count in CONDITION_COUNTS {
            for matches in [true, false] {
                let conditional = conditional_policy(header_count, condition_count, matches);
                assert_eq!(
                    conditional.propagate(&headers).count(),
                    usize::from(matches)
                );
                let case = if matches { "match" } else { "miss" };
                let _ = group.bench_with_input(
                    BenchmarkId::new(
                        format!("conditional_{case}_{condition_count}_conditions"),
                        format!("{header_count}_headers"),
                    ),
                    &headers,
                    |b, headers| {
                        b.iter(|| black_box(conditional.propagate(black_box(headers)).count()));
                    },
                );
            }
        }
    }

    group.finish();
}

fn headers(header_count: usize) -> TransportHeaders {
    let mut headers = TransportHeaders::with_capacity(header_count);
    for index in 0..header_count {
        headers.push(TransportHeader::text(
            context_name(&format!("header_{index}")),
            format!("value_{index}"),
        ));
    }
    headers
}

fn unqualified_policy(header_count: usize) -> HeaderPropagationPolicy {
    serde_yaml::from_str(&format!(
        r#"
default:
  selector:
    type: named
    named: [header_{}]
"#,
        header_count - 1
    ))
    .expect("valid unqualified propagation policy")
}

fn conditional_policy(
    header_count: usize,
    condition_count: usize,
    matches: bool,
) -> HeaderPropagationPolicy {
    let policy: HeaderPropagationPolicy = serde_yaml::from_str(
        r#"
default:
  selector:
    type: named
    named: [composite:selected]
"#,
    )
    .expect("valid conditional propagation policy");
    policy
        .compile_context(&[conditional_declaration(
            header_count,
            condition_count,
            matches,
        )])
        .expect("conditional propagation policy compiles")
}

fn conditional_declaration(
    header_count: usize,
    condition_count: usize,
    matches: bool,
) -> ContextEntryDeclaration {
    let selected_index = header_count - 1;
    let mut parts = Vec::with_capacity(condition_count + 1);
    parts.push(ContextEntryPart::TransportHeader {
        name: context_ref(&format!("header_{selected_index}")),
        store_as: Some(context_name("selected")),
    });

    for condition_index in 0..condition_count {
        let source_index = selected_index - (condition_index % header_count);
        // Case variants let the one-header case exercise multiple condition
        // scans while preserving the requested total captured-header count.
        let name_variant = CONDITION_NAME_VARIANTS[condition_index / header_count];
        let expected_value = if matches || condition_index + 1 < condition_count {
            format!("value_{source_index}")
        } else {
            "missing".to_owned()
        };
        parts.push(ContextEntryPart::TransportHeaderMatch {
            name: context_ref(&format!("{name_variant}_{source_index}")),
            value: expected_value,
        });
    }

    let declaration = ContextEntryDeclaration {
        scope: ContextScope::Engine,
        name: context_name("composite"),
        definition: ContextEntryDefinition(parts),
    };
    assert!(
        declaration
            .definition
            .validation_errors("benchmark.composite")
            .is_empty()
    );
    declaration
}

fn context_name(raw: &str) -> ContextEntryName {
    ContextEntryName::try_from(raw).expect("valid benchmark context entry name")
}

fn context_ref(raw: &str) -> ContextEntryRef {
    ContextEntryRef::try_from(raw).expect("valid benchmark context entry reference")
}

criterion_group!(benches, bench_header_propagation);
criterion_main!(benches);
