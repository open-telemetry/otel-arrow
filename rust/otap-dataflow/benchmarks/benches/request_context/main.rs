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
    println!();
}

criterion_group!(
    benches,
    bench_capture,
    bench_carry,
    bench_propagate,
    bench_end_to_end,
    alloc_report,
);
criterion_main!(benches);
