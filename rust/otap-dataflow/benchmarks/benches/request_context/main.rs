// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Baseline for the per-request cost of carrying request metadata.
//!
//! Every request that enters the engine drags a small amount of metadata with
//! it -- today a `TransportHeaders` vector of owned name/value pairs, reachable
//! from `Context`. This benchmark measures that path so a later change to the
//! representation has a before and after to be judged against.
//!
//! Three phases are timed separately because they have different shapes:
//!
//! - `capture` runs once per request at the receiver and allocates.
//! - `carry` runs at every node the request passes through and clones.
//! - `propagate` runs once per request at the exporter and reads.
//!
//! Wall time alone is a poor summary here, since the interesting cost is
//! allocator traffic that shows up as fragmentation and cache pressure rather
//! than as time in the benchmarked region. `alloc_report` therefore counts
//! allocations per request directly.
//!
//! Counting requires implementing `GlobalAlloc`, which the workspace denies.
//! The allow is scoped to this benchmark: the three methods below forward
//! unmodified arguments to the system allocator and add a relaxed counter, and
//! no benchmark code ships.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_config::tenant::compiled::{
    TenantTokenRegistry, TenantTokenRegistryBuilder, TenantView, TokenInputs, TokenScratch,
    attribute_field,
};
use otap_df_config::tenant::{Extractor, TenantTokenSpec, TenantTokens};
use otap_df_config::transport_headers::TransportHeaders;
use otap_df_config::transport_headers_policy::{
    CaptureDefaults, CaptureRule, HeaderCapturePolicy, HeaderPropagationPolicy, PropagationDefault,
    PropagationSelector, PropagationSelectorType,
};

/// Global allocator that counts allocations so the benchmark can report
/// allocations per request rather than only wall time.
struct Counting;

static ALLOCS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOCS.fetch_add(1, Ordering::Relaxed);
        let _ = BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        // SAFETY: forwarding an unmodified layout to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarding a pointer this allocator handed out.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let _ = ALLOCS.fetch_add(1, Ordering::Relaxed);
        let _ = BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        // SAFETY: forwarding a pointer and layout this allocator handed out.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// Run `f` `iters` times and return allocations and bytes per iteration.
fn measure<T>(iters: u64, mut f: impl FnMut() -> T) -> (f64, f64) {
    // Warm up so lazily initialized statics are not attributed to the region.
    let _ = black_box(f());
    let a0 = ALLOCS.load(Ordering::Relaxed);
    let b0 = BYTES.load(Ordering::Relaxed);
    for _ in 0..iters {
        let _ = black_box(f());
    }
    let allocs = ALLOCS.load(Ordering::Relaxed) - a0;
    let bytes = BYTES.load(Ordering::Relaxed) - b0;
    #[allow(clippy::cast_precision_loss)]
    (allocs as f64 / iters as f64, bytes as f64 / iters as f64)
}

/// A request's inbound metadata. Names and sizes are chosen to look like a
/// gateway hop: routing identity, tracing, and a couple of headers the policy
/// will not match, since a real request carries more than the pipeline wants.
fn inbound() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("x-tenant-id", b"tenant-a".as_slice()),
        ("x-project-id", b"proj-40f1c2".as_slice()),
        ("x-request-id", b"01J8Z9QK7YB3F4N6P8R2T5V7W9".as_slice()),
        (
            "traceparent",
            b"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".as_slice(),
        ),
        ("user-agent", b"otel-collector/0.99.0".as_slice()),
        ("content-type", b"application/x-protobuf".as_slice()),
        ("accept-encoding", b"gzip".as_slice()),
        ("grpc-timeout", b"30S".as_slice()),
    ]
}

fn capture_policy(n_rules: usize) -> HeaderCapturePolicy {
    let names = [
        ("x-tenant-id", "tenant_id"),
        ("x-project-id", "project_id"),
        ("x-request-id", "request_id"),
        ("traceparent", "traceparent"),
    ];
    let rules = names
        .iter()
        .take(n_rules)
        .map(|(wire, stored)| CaptureRule {
            match_names: vec![(*wire).to_owned()],
            store_as: Some((*stored).to_owned()),
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
            ..Default::default()
        },
        Vec::new(),
    )
}

/// The tenant-token equivalent of `capture_policy`: one token whose `n`
/// extractors read the same wire headers and retain the same logical names.
///
/// `bag` decides whether key names are encoded alongside the values, which is
/// what makes the run appendable to telemetry as OTLP attributes.
fn registry(n_keys: usize, bag: bool) -> TenantTokenRegistry {
    let names = [
        ("x-tenant-id", "tenant_id"),
        ("x-project-id", "project_id"),
        ("x-request-id", "request_id"),
        ("traceparent", "traceparent"),
    ];
    let extractors = names
        .iter()
        .take(n_keys)
        .map(|(wire, key)| Extractor::TransportHeader {
            key: (*key).to_owned(),
            transport_header: (*wire).to_owned(),
            retain: true,
            bag,
        })
        .collect();
    let mut tokens = TenantTokens::default();
    let _ = tokens.insert("gateway".to_owned(), TenantTokenSpec { extractors });
    let mut builder = TenantTokenRegistryBuilder::new().with_bag_field(attribute_field::SCOPE);
    builder.add_tokens(&tokens).expect("registry builds");
    builder.build(1)
}

/// The exporter side: the logical keys an exporter re-emits, paired with the
/// wire names it chooses. Resolved once at build time, exactly as a compiled
/// exporter would.
fn egress_map(reg: &TenantTokenRegistry, n_keys: usize) -> Vec<(u16, &'static str)> {
    let names = [
        ("tenant_id", "x-acme-tenant"),
        ("project_id", "x-acme-project"),
        ("request_id", "x-acme-request"),
        ("traceparent", "traceparent"),
    ];
    names
        .iter()
        .take(n_keys)
        .filter_map(|(key, wire)| {
            let id = reg.key_id(key)?;
            Some((reg.value_slot(id)?, *wire))
        })
        .collect()
}

/// Counts of matched headers to sweep. One is the degenerate case a router
/// needs; four is a realistic gateway that also forwards tracing context.
const MATCHED: [usize; 3] = [1, 2, 4];

fn bench_capture(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/capture");
    for n in MATCHED {
        let policy = capture_policy(n);
        let pairs = inbound();
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut out = TransportHeaders::default();
                let _ = policy
                    .capture_from_pairs(pairs.iter().map(|(k, v)| (*k, *v)), black_box(&mut out));
                out
            });
        });
    }
    group.finish();
}

fn bench_carry(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/carry");
    for n in MATCHED {
        let policy = capture_policy(n);
        let pairs = inbound();
        let mut headers = TransportHeaders::default();
        let _ = policy.capture_from_pairs(pairs.iter().map(|(k, v)| (*k, *v)), &mut headers);
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(headers.clone()));
        });
    }
    group.finish();
}

fn bench_propagate(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/propagate");
    let egress = propagation_policy();
    for n in MATCHED {
        let policy = capture_policy(n);
        let pairs = inbound();
        let mut headers = TransportHeaders::default();
        let _ = policy.capture_from_pairs(pairs.iter().map(|(k, v)| (*k, *v)), &mut headers);
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut emitted = 0usize;
                for h in egress.propagate(&headers) {
                    emitted += black_box(h.header_name.len()) + black_box(h.value.len());
                }
                emitted
            });
        });
    }
    group.finish();
}

/// One receiver capture, two intermediate node hops, one exporter read.
fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/end_to_end");
    let egress = propagation_policy();
    for n in MATCHED {
        let policy = capture_policy(n);
        let pairs = inbound();
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut headers = TransportHeaders::default();
                let _ =
                    policy.capture_from_pairs(pairs.iter().map(|(k, v)| (*k, *v)), &mut headers);
                let hop1 = headers.clone();
                let hop2 = hop1.clone();
                let mut emitted = 0usize;
                for h in egress.propagate(&hop2) {
                    emitted += h.value.len();
                }
                black_box(emitted)
            });
        });
    }
    group.finish();
}

fn bench_tenant(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_context/tenant_resolve");
    for n in MATCHED {
        let reg = registry(n, false);
        let pairs = inbound();
        let mut scratch = TokenScratch::new();
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(reg.resolve(
                    &mut scratch,
                    TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
                ))
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("request_context/tenant_carry");
    for n in MATCHED {
        let reg = registry(n, false);
        let pairs = inbound();
        let mut scratch = TokenScratch::new();
        let words = reg
            .resolve(
                &mut scratch,
                TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
            )
            .expect("resolves");
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(words.clone()));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("request_context/tenant_propagate");
    for n in MATCHED {
        let reg = registry(n, false);
        let egress = egress_map(&reg, n);
        let pairs = inbound();
        let mut scratch = TokenScratch::new();
        let words = reg
            .resolve(
                &mut scratch,
                TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
            )
            .expect("resolves");
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let view = TenantView::new(&words);
                let mut emitted = 0usize;
                for (slot, wire) in &egress {
                    if let Some(value) = view.slot_value(*slot) {
                        emitted += black_box(wire.len()) + black_box(value.len());
                    }
                }
                emitted
            });
        });
    }
    group.finish();

    // One receiver resolve, two node hops, one exporter read: the same shape
    // as `bench_end_to_end`, so the totals are directly comparable.
    let mut group = c.benchmark_group("request_context/tenant_end_to_end");
    for n in MATCHED {
        let reg = registry(n, false);
        let egress = egress_map(&reg, n);
        let pairs = inbound();
        let mut scratch = TokenScratch::new();
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let words = reg
                    .resolve(
                        &mut scratch,
                        TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
                    )
                    .expect("resolves");
                let hop1 = words.clone();
                let hop2 = hop1.clone();
                let view = TenantView::new(&hop2);
                let mut emitted = 0usize;
                for (slot, _) in &egress {
                    if let Some(value) = view.slot_value(*slot) {
                        emitted += value.len();
                    }
                }
                black_box(emitted)
            });
        });
    }
    group.finish();

    // Instrumenting a request with its tenant context. The baseline has no
    // equivalent: it would have to encode a KeyValue per header.
    let mut group = c.benchmark_group("request_context/tenant_attributes");
    for n in MATCHED {
        let reg = registry(n, true);
        let pairs = inbound();
        let mut scratch = TokenScratch::new();
        let words = reg
            .resolve(
                &mut scratch,
                TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
            )
            .expect("resolves");
        let mut dst = Vec::with_capacity(1024);
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                dst.clear();
                dst.extend_from_slice(TenantView::new(&words).attributes());
                black_box(dst.len())
            });
        });
    }
    group.finish();
}

/// Report allocations per request for each phase.
///
/// This is the number the representation change is meant to move, and it is
/// printed rather than timed so a before and after can be compared directly
/// without reading Criterion's statistics.
#[allow(clippy::print_stdout)]
fn alloc_report(_c: &mut Criterion) {
    const ITERS: u64 = 20_000;
    let egress = propagation_policy();
    let pairs = inbound();

    println!("\nrequest_context allocations per request (baseline)");
    println!(
        "{:<12} {:>8} {:>10} {:>10}",
        "phase", "matched", "allocs", "bytes"
    );

    for n in MATCHED {
        let policy = capture_policy(n);

        let (a, b) = measure(ITERS, || {
            let mut out = TransportHeaders::default();
            let _ = policy.capture_from_pairs(pairs.iter().map(|(k, v)| (*k, *v)), &mut out);
            out
        });
        println!("{:<12} {n:>8} {a:>10.2} {b:>10.1}", "capture");

        let mut headers = TransportHeaders::default();
        let _ = policy.capture_from_pairs(pairs.iter().map(|(k, v)| (*k, *v)), &mut headers);
        let (a, b) = measure(ITERS, || headers.clone());
        println!("{:<12} {n:>8} {a:>10.2} {b:>10.1}", "carry");

        let (a, b) = measure(ITERS, || {
            let mut emitted = 0usize;
            for h in egress.propagate(&headers) {
                emitted += h.value.len();
            }
            emitted
        });
        println!("{:<12} {n:>8} {a:>10.2} {b:>10.1}", "propagate");

        let (a, b) = measure(ITERS, || {
            let mut h = TransportHeaders::default();
            let _ = policy.capture_from_pairs(pairs.iter().map(|(k, v)| (*k, *v)), &mut h);
            let hop1 = h.clone();
            let hop2 = hop1.clone();
            let mut emitted = 0usize;
            for p in egress.propagate(&hop2) {
                emitted += p.value.len();
            }
            emitted
        });
        println!("{:<12} {n:>8} {a:>10.2} {b:>10.1}", "end_to_end");
    }

    println!("\nrequest_context allocations per request (tenant token)");
    println!(
        "{:<12} {:>8} {:>10} {:>10}",
        "phase", "matched", "allocs", "bytes"
    );

    for n in MATCHED {
        let reg = registry(n, false);
        let egress = egress_map(&reg, n);
        let mut scratch = TokenScratch::new();

        let (a, b) = measure(ITERS, || {
            reg.resolve(
                &mut scratch,
                TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
            )
        });
        println!("{:<12} {n:>8} {a:>10.2} {b:>10.1}", "capture");

        let words = reg
            .resolve(
                &mut scratch,
                TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
            )
            .expect("resolves");
        let (a, b) = measure(ITERS, || words.clone());
        println!("{:<12} {n:>8} {a:>10.2} {b:>10.1}", "carry");

        let (a, b) = measure(ITERS, || {
            let view = TenantView::new(&words);
            let mut emitted = 0usize;
            for (slot, wire) in &egress {
                if let Some(value) = view.slot_value(*slot) {
                    emitted += wire.len() + value.len();
                }
            }
            emitted
        });
        println!("{:<12} {n:>8} {a:>10.2} {b:>10.1}", "propagate");

        let (a, b) = measure(ITERS, || {
            let w = reg
                .resolve(
                    &mut scratch,
                    TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
                )
                .expect("resolves");
            let hop1 = w.clone();
            let hop2 = hop1.clone();
            let view = TenantView::new(&hop2);
            let mut emitted = 0usize;
            for (slot, _) in &egress {
                if let Some(value) = view.slot_value(*slot) {
                    emitted += value.len();
                }
            }
            emitted
        });
        println!("{:<12} {n:>8} {a:>10.2} {b:>10.1}", "end_to_end");

        let bag = registry(n, true);
        let mut s2 = TokenScratch::new();
        let bw = bag
            .resolve(
                &mut s2,
                TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
            )
            .expect("resolves");
        let mut dst = Vec::with_capacity(1024);
        let (a, b) = measure(ITERS, || {
            dst.clear();
            dst.extend_from_slice(TenantView::new(&bw).attributes());
            dst.len()
        });
        println!("{:<12} {n:>8} {a:>10.2} {b:>10.1}", "attributes");
    }
    println!();
}

criterion_group!(
    benches,
    bench_capture,
    bench_carry,
    bench_propagate,
    bench_end_to_end,
    bench_tenant,
    alloc_report,
);
criterion_main!(benches);
