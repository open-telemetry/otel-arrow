//! Microbenchmarks for OTLP batch splitting logic.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use otap_df_otlp::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};
use otap_df_otlp::{ExportTraceServiceRequest, HierarchicalBatchSplit};
use std::hint::black_box;

fn make_large_trace_fixture(
    num_resources: usize,
    scopes_per_resource: usize,
    spans_per_scope: usize,
) -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: (0..num_resources)
            .map(|r| ResourceSpans {
                resource: None,
                schema_url: String::new(),
                scope_spans: (0..scopes_per_resource)
                    .map(|s| ScopeSpans {
                        scope: None,
                        spans: (0..spans_per_scope)
                            .map(|i| Span {
                                name: format!("span_{}_{}_{}", r, s, i),
                                ..Default::default()
                            })
                            .collect(),
                        schema_url: String::new(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn bench_trace_batch_split(c: &mut Criterion) {
    let req = make_large_trace_fixture(10, 100, 100);
    let _ = c.bench_function("trace_split_into_batches", |b| {
        b.iter(|| {
            let req = black_box(req.clone());
            let _ = req.split_into_batches(black_box(128));
        });
    });
}

criterion_group!(benches, bench_trace_batch_split);
criterion_main!(benches);
