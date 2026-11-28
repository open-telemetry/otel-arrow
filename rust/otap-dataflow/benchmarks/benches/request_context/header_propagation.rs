// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks transport-header propagation through compiled context bindings.
//!
//! Every case compiles an `arrival -> sink` pipeline, forms request context
//! through the arrival binding, and propagates it through the sink binding.
//! `header_propagation_sink_small` propagates 0-5 captured headers, with at
//! most one condition on an existing member (no extra condition headers).
//! `header_propagation_sink` compares unqualified propagation with
//! conditional propagation of header-only composites, alone and together with
//! forming the context. `arrival_form` measures forming context with
//! composites at arrival.
//!
//! Run with:
//!
//! ```text
//! cargo bench -p benchmarks --bench request_context -- header_propagation
//! ```

use criterion::{BenchmarkId, Criterion, Throughput};
use otel_arrow_dfe_config::context::policy::{
    ContextEntryDeclaration, ContextEntryDefinition, ContextEntryPart, ContextPolicy,
    ContextRandomnessKind,
};
use otel_arrow_dfe_config::context::transport_policy::{
    CaptureDefaults, CaptureRule, HeaderCapturePolicy,
    HeaderPropagationPolicy as HeaderPropagationConfig, PropagationDefault, PropagationSelector,
    PropagationSelectorType,
};
use otel_arrow_dfe_config::context::{ContextEntryName, ContextEntryRef};
use otel_arrow_dfe_engine::pdata_context::{
    Arrival, ArrivalBinding, ArrivalHeaders, Metadata, SinkBinding,
};
use otel_arrow_dfe_engine::testing::context::{ContextFixture, ContextFixtureBuilder};
use std::hint::black_box;

const HEADER_COUNTS: [usize; 4] = [1, 4, 16, 32];
const CONDITION_COUNTS: [usize; 4] = [1, 2, 3, 4];
const CONDITION_NAME_VARIANTS: [&str; 4] = ["header", "HEADER", "Header", "hEaDeR"];
const DUPLICATE_SOURCE_COUNTS: [usize; 4] = [1, 4, 16, 28];
const DUPLICATE_TOTAL_HEADERS: usize = 32;
const SHARED_CONDITION_BINDING_COUNTS: [usize; 3] = [4, 5, 32];
const SINK: &str = "out";

/// Inbound header names and values, in arrival order.
type Inbound = Vec<(String, String)>;

/// A compiled `arrival -> sink` pipeline.
struct Bound {
    arrival: ArrivalBinding,
    sink: SinkBinding,
}

impl Bound {
    /// Compiles a pipeline that captures exactly `headers` and propagates
    /// with `propagation`, resolving composites from `context`.
    fn compile(
        headers: &Inbound,
        propagation: &HeaderPropagationConfig,
        context: Option<ContextPolicy>,
    ) -> Self {
        let mut builder = capture(ContextFixture::builder(), headers).sink(
            SINK,
            serde_json::to_value(propagation).expect("serializable propagation policy"),
        );
        if let Some(context) = context {
            builder = builder.policy("context", context);
        }
        let fixture = builder.compile_standalone();
        Self {
            arrival: fixture.arrival(),
            sink: fixture
                .node_bindings(SINK)
                .sink()
                .cloned()
                .expect("sink binding"),
        }
    }

    /// Forms context from `headers` at arrival.
    fn form(&self, headers: &[(&str, &[u8])]) -> Metadata {
        form(&self.arrival, headers)
    }

    /// Returns the number of values propagated from `metadata`.
    fn propagate(&self, metadata: &Metadata) -> usize {
        let mut propagated = 0;
        self.sink
            .propagate(metadata, |value| {
                let _ = black_box(value);
                propagated += 1;
            })
            .expect("context matches the sink");
        propagated
    }
}

pub(super) fn benchmarks(c: &mut Criterion) {
    arrival_form_benchmarks(c);
    small_composite_benchmarks(c);
    let mut group = c.benchmark_group("header_propagation_sink");

    for header_count in HEADER_COUNTS {
        let headers = headers(header_count);
        let pairs = pairs(&headers);
        let _ = group.throughput(Throughput::Elements(header_count as u64));

        let unqualified = Bound::compile(&headers, &unqualified_policy(header_count), None);
        let formed = unqualified.form(&pairs);
        assert_eq!(unqualified.propagate(&formed), 1);
        let _ = group.bench_with_input(
            BenchmarkId::new("unqualified", format!("{header_count}_headers")),
            &formed,
            |b, formed| {
                b.iter(|| black_box(unqualified.propagate(black_box(formed))));
            },
        );

        for condition_count in CONDITION_COUNTS {
            for matches in [true, false] {
                let conditional = Bound::compile(
                    &headers,
                    &composite_selected_policy(),
                    Some(ContextPolicy::from_iter([conditional_declaration(
                        header_count,
                        condition_count,
                        matches,
                    )])),
                );
                let formed = conditional.form(&pairs);
                assert_eq!(conditional.propagate(&formed), usize::from(matches));
                let case = if matches { "match" } else { "miss" };
                let _ = group.bench_with_input(
                    BenchmarkId::new(
                        format!("conditional_{case}_{condition_count}_conditions"),
                        format!("{header_count}_headers"),
                    ),
                    &formed,
                    |b, formed| {
                        b.iter(|| black_box(conditional.propagate(black_box(formed))));
                    },
                );
                let _ = group.bench_with_input(
                    BenchmarkId::new(
                        format!("form_and_propagate_{case}_{condition_count}_conditions"),
                        format!("{header_count}_headers"),
                    ),
                    &pairs,
                    |b, pairs| {
                        b.iter(|| {
                            let formed = conditional.form(black_box(pairs));
                            black_box(conditional.propagate(&formed))
                        });
                    },
                );
            }
        }
    }

    group.finish();

    let mut duplicate_group = c.benchmark_group("header_propagation_sink_duplicates");
    let _ = duplicate_group.throughput(Throughput::Elements(DUPLICATE_TOTAL_HEADERS as u64));
    for source_count in DUPLICATE_SOURCE_COUNTS {
        let headers = duplicate_source_headers(source_count);
        let pairs = pairs(&headers);
        for matches in [true, false] {
            let conditional = Bound::compile(
                &headers,
                &composite_selected_policy(),
                Some(duplicate_source_context(matches)),
            );
            let formed = conditional.form(&pairs);
            assert_eq!(
                conditional.propagate(&formed),
                if matches { source_count } else { 0 }
            );
            let case = if matches { "match" } else { "miss" };
            let _ = duplicate_group.bench_with_input(
                BenchmarkId::new(
                    format!("conditional_duplicate_{case}_4_conditions"),
                    format!("{source_count}_sources_32_headers"),
                ),
                &formed,
                |b, formed| {
                    b.iter(|| black_box(conditional.propagate(black_box(formed))));
                },
            );
        }
    }

    duplicate_group.finish();

    let mut binding_group = c.benchmark_group("header_propagation_sink_bindings");
    for binding_count in SHARED_CONDITION_BINDING_COUNTS {
        let headers = shared_condition_headers(binding_count);
        let pairs = pairs(&headers);
        let _ = binding_group.throughput(Throughput::Elements(binding_count as u64));
        for matches in [true, false] {
            let (propagation, context) = shared_condition_policy(binding_count, matches);
            let conditional = Bound::compile(&headers, &propagation, Some(context));
            let formed = conditional.form(&pairs);
            assert_eq!(
                conditional.propagate(&formed),
                if matches { binding_count } else { 0 }
            );
            let case = if matches { "match" } else { "miss" };
            let _ = binding_group.bench_with_input(
                BenchmarkId::new(
                    format!("conditional_shared_{case}_4_conditions"),
                    binding_count,
                ),
                &formed,
                |b, formed| {
                    b.iter(|| black_box(conditional.propagate(black_box(formed))));
                },
            );
        }
    }

    binding_group.finish();
}

fn arrival_form_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("arrival_form");
    for composite_count in [0usize, 1, 3] {
        let headers = (0..composite_count)
            .flat_map(|index| {
                [
                    (format!("workspace_{index}"), "acme".to_owned()),
                    (format!("environment_{index}"), "production".to_owned()),
                ]
            })
            .collect::<Inbound>();
        let declarations = (0..composite_count)
            .map(|index| ContextEntryDeclaration {
                name: context_name(&format!("composite_{index}")),
                definition: ContextEntryDefinition(vec![
                    ContextEntryPart::TransportHeader {
                        name: context_name(&format!("workspace_{index}")),
                        store_as: Some(context_name("workspace")),
                    },
                    ContextEntryPart::TransportHeaderMatch {
                        name: context_name(&format!("environment_{index}")),
                        value: "production".to_owned(),
                    },
                    ContextEntryPart::Constant {
                        name: context_name("route"),
                        value: "otlp-grpc".to_owned(),
                    },
                    ContextEntryPart::Randomness {
                        name: context_name("correlation"),
                        value: ContextRandomnessKind::Uuid7,
                    },
                ]),
            })
            .collect::<Vec<_>>();
        let mut builder = capture(ContextFixture::builder(), &headers);
        if composite_count > 0 {
            // A sink selecting each composite makes arrival materialize it.
            let propagation =
                named_policy((0..composite_count).map(|index| format!("composite_{index}:route")));
            builder = builder
                .policy("context", ContextPolicy::from_iter(declarations))
                .sink(
                    SINK,
                    serde_json::to_value(&propagation).expect("serializable propagation policy"),
                );
        }
        let arrival = builder.compile_standalone().arrival();
        let pairs = pairs(&headers);
        let _ = group.bench_with_input(
            BenchmarkId::new("composites", composite_count),
            &pairs,
            |b, pairs| {
                b.iter(|| black_box(form(&arrival, black_box(pairs))));
            },
        );
    }
    group.finish();
}

fn small_composite_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("header_propagation_sink_small");
    for member_count in 0..=5 {
        let headers = small_composite_headers(member_count);
        let pairs = pairs(&headers);
        let unqualified = HeaderPropagationConfig::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![],
        );
        let bound = Bound::compile(&headers, &unqualified, None);
        let formed = bound.form(&pairs);
        assert_eq!(bound.propagate(&formed), member_count);
        let _ = group.bench_with_input(
            BenchmarkId::new("unqualified", format!("{member_count}_members")),
            &formed,
            |b, formed| {
                b.iter(|| black_box(bound.propagate(black_box(formed))));
            },
        );
        if member_count == 0 {
            continue;
        }
        for (case, condition) in [
            ("plain", None),
            ("match", Some(true)),
            ("miss", Some(false)),
        ] {
            let (config, context) = small_composite_config(member_count, condition);
            let bound = Bound::compile(&headers, &config, Some(context));
            let formed = bound.form(&pairs);
            assert_eq!(
                bound.propagate(&formed),
                if condition == Some(false) {
                    0
                } else {
                    member_count
                }
            );
            let _ = group.bench_with_input(
                BenchmarkId::new(case, format!("{member_count}_members")),
                &formed,
                |b, formed| {
                    b.iter(|| black_box(bound.propagate(black_box(formed))));
                },
            );
        }
    }
    group.finish();
}

/// Forms context from `headers` through `arrival`.
fn form(arrival: &ArrivalBinding, headers: &[(&str, &[u8])]) -> Metadata {
    arrival
        .form(Arrival {
            headers: ArrivalHeaders::Pairs(headers),
            identity: None,
            peer: None,
        })
        .expect("arrival forms context")
        .0
}

/// Adds an arrival capture policy that captures every name in `headers`.
fn capture(builder: ContextFixtureBuilder, headers: &Inbound) -> ContextFixtureBuilder {
    if headers.is_empty() {
        return builder;
    }
    builder.header_capture(&HeaderCapturePolicy::new(
        CaptureDefaults {
            max_entries: headers.len(),
            ..CaptureDefaults::default()
        },
        vec![CaptureRule {
            match_names: headers.iter().map(|(name, _)| context_name(name)).collect(),
            store_as: None,
            sensitive: false,
            value_kind: None,
        }],
    ))
}

/// Borrows `headers` as arrival pairs.
fn pairs(headers: &Inbound) -> Vec<(&str, &[u8])> {
    headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_bytes()))
        .collect()
}

fn small_composite_headers(member_count: usize) -> Inbound {
    (0..member_count)
        .map(|index| (format!("field_{index}"), "value".into()))
        .collect()
}

fn small_composite_config(
    member_count: usize,
    condition: Option<bool>,
) -> (HeaderPropagationConfig, ContextPolicy) {
    let policy = named_policy((0..member_count).map(|index| format!("composite:field_{index}")));
    let mut parts = (0..member_count)
        .map(|index| ContextEntryPart::TransportHeader {
            name: context_name(&format!("field_{index}")),
            store_as: None,
        })
        .collect::<Vec<_>>();
    if let Some(matches) = condition {
        parts.push(ContextEntryPart::TransportHeaderMatch {
            name: context_name("field_0"),
            value: if matches { "value" } else { "missing" }.into(),
        });
    }
    (
        policy,
        ContextPolicy::from_iter([ContextEntryDeclaration {
            name: context_name("composite"),
            definition: ContextEntryDefinition(parts),
        }]),
    )
}

fn headers(header_count: usize) -> Inbound {
    (0..header_count)
        .map(|index| (format!("header_{index}"), format!("value_{index}")))
        .collect()
}

fn unqualified_policy(header_count: usize) -> HeaderPropagationConfig {
    named_policy([format!("header_{}", header_count - 1)])
}

/// Propagates the `selected` member of the `composite` entry.
fn composite_selected_policy() -> HeaderPropagationConfig {
    named_policy(["composite:selected".to_owned()])
}

/// Propagates exactly the named entries or composite members.
fn named_policy(named: impl IntoIterator<Item = String>) -> HeaderPropagationConfig {
    HeaderPropagationConfig::new(
        PropagationDefault {
            selector: PropagationSelector {
                selector_type: PropagationSelectorType::Named,
                named: Some(named.into_iter().map(|name| context_ref(&name)).collect()),
            },
            ..PropagationDefault::default()
        },
        vec![],
    )
}

fn conditional_declaration(
    header_count: usize,
    condition_count: usize,
    matches: bool,
) -> ContextEntryDeclaration {
    let selected_index = header_count - 1;
    let mut parts = Vec::with_capacity(condition_count + 1);
    parts.push(ContextEntryPart::TransportHeader {
        name: context_name(&format!("header_{selected_index}")),
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
            name: context_name(&format!("{name_variant}_{source_index}")),
            value: expected_value,
        });
    }

    let declaration = ContextEntryDeclaration {
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

fn duplicate_source_headers(source_count: usize) -> Inbound {
    let mut headers = Vec::with_capacity(DUPLICATE_TOTAL_HEADERS);
    for _ in 0..source_count {
        headers.push(("selected_source".to_owned(), "selected".to_owned()));
    }
    for index in source_count..DUPLICATE_TOTAL_HEADERS - CONDITION_COUNTS.len() {
        headers.push((format!("filler_{index}"), "filler".to_owned()));
    }
    for index in 0..CONDITION_COUNTS.len() {
        headers.push((format!("condition_{index}"), format!("value_{index}")));
    }
    headers
}

fn duplicate_source_context(matches: bool) -> ContextPolicy {
    let mut parts = vec![ContextEntryPart::TransportHeader {
        name: context_name("selected_source"),
        store_as: Some(context_name("selected")),
    }];
    parts.extend(condition_parts(matches));
    ContextPolicy::from_iter([ContextEntryDeclaration {
        name: context_name("composite"),
        definition: ContextEntryDefinition(parts),
    }])
}

fn shared_condition_headers(binding_count: usize) -> Inbound {
    let mut headers = Vec::with_capacity(binding_count + CONDITION_COUNTS.len());
    for index in 0..binding_count {
        headers.push((format!("selected_source_{index}"), "selected".to_owned()));
    }
    for index in 0..CONDITION_COUNTS.len() {
        headers.push((format!("condition_{index}"), format!("value_{index}")));
    }
    headers
}

fn shared_condition_policy(
    binding_count: usize,
    matches: bool,
) -> (HeaderPropagationConfig, ContextPolicy) {
    let policy =
        named_policy((0..binding_count).map(|index| format!("composite:selected_{index}")));
    let mut parts = Vec::with_capacity(binding_count + CONDITION_COUNTS.len());
    for index in 0..binding_count {
        parts.push(ContextEntryPart::TransportHeader {
            name: context_name(&format!("selected_source_{index}")),
            store_as: Some(context_name(&format!("selected_{index}"))),
        });
    }
    parts.extend(condition_parts(matches));
    (
        policy,
        ContextPolicy::from_iter([ContextEntryDeclaration {
            name: context_name("composite"),
            definition: ContextEntryDefinition(parts),
        }]),
    )
}

/// Matches `condition_<i>` headers; the last one misses unless `matches`.
fn condition_parts(matches: bool) -> impl Iterator<Item = ContextEntryPart> {
    (0..CONDITION_COUNTS.len()).map(move |index| ContextEntryPart::TransportHeaderMatch {
        name: context_name(&format!("condition_{index}")),
        value: if matches || index + 1 < CONDITION_COUNTS.len() {
            format!("value_{index}")
        } else {
            "missing".to_owned()
        },
    })
}

fn context_name(raw: &str) -> ContextEntryName {
    ContextEntryName::try_from(raw).expect("valid benchmark context entry name")
}

fn context_ref(raw: &str) -> ContextEntryRef {
    ContextEntryRef::try_from(raw).expect("valid benchmark context entry reference")
}
