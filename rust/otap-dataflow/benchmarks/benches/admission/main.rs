// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Admission-specific hot-path characterization for OTLP and Syslog receivers.
//!
//! These cases isolate the work introduced by admission from network, decoding,
//! and parsing costs. Compare results across revisions when evaluating the
//! disabled branch, the shared OTLP gate, and the local Syslog gate.

#![allow(missing_docs)]

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_config::policy::{
    RateLimitAggregation, RateLimitEnforcement, RateLimitPressure, RateLimitUnit,
    RateLimiterPolicy, TokenBucketPolicy,
};
use otel_arrow_dfe_engine::admission::{
    AdmissionBinder, AdmissionContext, AdmissionDimension, LocalAdmissionGate, SharedAdmissionGate,
};
use otel_arrow_dfe_engine::memory_limiter::{
    LocalReceiverAdmissionState, MemoryPressureState, SharedReceiverAdmissionState,
};
use std::hint::black_box;
use std::time::Duration;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn policy(unit: RateLimitUnit) -> RateLimiterPolicy {
    RateLimiterPolicy {
        enforcement: RateLimitEnforcement::Enforce,
        aggregation: RateLimitAggregation::ReceiverInstance,
        unit,
        pressure: RateLimitPressure::Soft,
        token_bucket: TokenBucketPolicy {
            allow: 1_000_000_000,
            interval: Duration::from_secs(1),
            burst: Some(1_000_000_000),
        },
    }
}

#[inline]
fn otlp_admit(gate: &Option<SharedAdmissionGate>, units: u64) {
    if gate
        .as_ref()
        .is_some_and(SharedAdmissionGate::is_instance_saturated)
    {
        return;
    }
    if gate
        .as_ref()
        .is_some_and(SharedAdmissionGate::is_instance_saturated)
    {
        return;
    }
    if let Some(gate) = gate {
        let _ = black_box(gate.admit(units, AdmissionContext::EMPTY));
    }
}

#[inline]
fn syslog_admit(gate: &Option<LocalAdmissionGate>) {
    if let Some(gate) = gate {
        let _ = black_box(gate.admit(1, AdmissionContext::EMPTY));
    }
}

fn bench_admission(c: &mut Criterion) {
    let process_state = MemoryPressureState::default();

    let shared_binder = AdmissionBinder::configured("otlp", policy(RateLimitUnit::RequestBytes));
    let shared_gate = shared_binder
        .bind_shared(
            AdmissionDimension::Bytes,
            SharedReceiverAdmissionState::from_process_state(&process_state),
        )
        .expect("shared admission binding")
        .expect("configured shared gate");

    let local_binder = AdmissionBinder::configured("syslog", policy(RateLimitUnit::Messages));
    let local_gate = local_binder
        .bind_local(
            AdmissionDimension::Messages,
            LocalReceiverAdmissionState::from_process_state(&process_state),
        )
        .expect("local admission binding")
        .expect("configured local gate");

    let mut group = c.benchmark_group("admission_hot_path");
    _ = group.throughput(Throughput::Elements(1));

    let disabled: Option<SharedAdmissionGate> = None;
    let _ = group.bench_function("otlp_disabled_small_request", |b| {
        b.iter(|| otlp_admit(black_box(&disabled), black_box(128)))
    });

    let enabled = Some(shared_gate);
    let _ = group.bench_function("otlp_enabled_normal_pressure", |b| {
        b.iter(|| otlp_admit(black_box(&enabled), black_box(128)))
    });

    let syslog = Some(local_gate);
    let _ = group.bench_function("syslog_local_normal_pressure", |b| {
        b.iter(|| syslog_admit(black_box(&syslog)))
    });

    group.finish();
}

criterion_group!(benches, bench_admission);
criterion_main!(benches);
