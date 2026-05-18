// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compression-ratio sanity tests covering the cartesian product of:
//! - **Data source**: `static` (hard-coded varied payloads) vs `semantic_conventions`
//!   (registry-driven payloads).
//! - **Generation strategy**: `fresh` (new batch every iteration, simulating the
//!   loadgen `fresh` strategy) vs `pre_generated` (one batch reused for every
//!   iteration, simulating the loadgen `pre_generated` strategy).
//!
//! These tests are primarily **informational** — run with `--nocapture` to
//! eyeball the bytes/record and compression ratios for each combination and
//! sanity-check whether the loadgen output looks realistic. The asserts use
//! deliberately wide bands so they don't flake on platform variance; their job
//! is to catch order-of-magnitude regressions (e.g. `fresh` accidentally
//! collapsing into `pre_generated`-style 300:1 compression), not exact numbers.
//!
//! ## Reference numbers (10 batches × 512 records, zstd level 3)
//!
//! Captured on macOS at the time of writing — your numbers should be within
//! a few percent. Run with `--nocapture` to see live values.
//!
//! | Source     | Strategy        |       Raw | B/rec | Compressed | Ratio   |
//! |------------|-----------------|-----------|-------|------------|---------|
//! | Static     | `fresh`         | 1,908,600 | 372.8 |     43,930 |  43.4:1 |
//! | Static     | `pre_generated` | 1,908,600 | 372.8 |     17,714 | 107.7:1 |
//! | Static+TC  | `fresh`         | 2,051,960 | 400.8 |    188,111 |  10.9:1 |
//! | Static+TC  | `pre_generated` | 2,051,960 | 400.8 |     32,900 |  62.4:1 |
//! | Semconv    | `fresh`         | 1,843,603 | 360.1 |     71,127 |  25.9:1 |
//! | Semconv    | `pre_generated` | 1,843,570 | 360.1 |     10,161 | 181.4:1 |
//!
//! Takeaways:
//! - `pre_generated` collapses everything via cross-batch byte-identical replay.
//! - `semconv + fresh` is ~2× more compressible than `static + fresh` at
//!   comparable per-record size, because the semconv registry's event vocabulary
//!   is narrow (~30 event groups, ~4 distinct shapes) and enum/single-example
//!   attributes emit constant values.
//! - Adding `use_trace_context: true` to `static + fresh` drops the ratio ~4×
//!   (40:1 → 11:1) because the 24 random bytes of trace+span ID per record are
//!   essentially incompressible — this most closely matches the wire footprint
//!   of OTel-instrumented application logs, where the SDK automatically attaches
//!   the active span context to every emitted log. Logs ingested through
//!   collector receivers (syslog, journald, filelog) typically lack trace
//!   context and look more like the Static row.
//!
//! Run with:
//! ```sh
//! cargo test -p otap-df-core-nodes --features dev-tools \
//!     -- receivers::traffic_generator::compression_ratio_tests --nocapture
//! ```

#![cfg(test)]

use super::semconv_signal::semconv_otlp_logs;
use super::static_signal::static_otlp_logs_with_config;
use prost::Message;
use weaver_common::{result::WResult, vdir::VirtualDirectoryPath};
use weaver_forge::registry::ResolvedRegistry;
use weaver_resolver::SchemaResolver;
use weaver_semconv::registry_repo::RegistryRepo;

/// OTel SDK default batch size for log/span/metric exporters.
const BATCH_SIZE: usize = 512;

/// Number of batches concatenated into a single zstd stream, approximating
/// what a long-lived OTAP gRPC stream sees over time.
const NUM_BATCHES: usize = 10;

/// Generous window for `fresh` generation across both data sources.
/// Observed values today: static ~40:1, semconv ~25:1, static+trace_context
/// ~11:1 (random 16-byte trace_id + 8-byte span_id per record are
/// essentially incompressible and drag the ratio way down). Bounds are wide
/// on purpose — they exist to catch order-of-magnitude regressions only.
const FRESH_RATIO_RANGE: std::ops::RangeInclusive<f64> = 3.0..=60.0;

/// Floor for `pre_generated` replay across both data sources. Observed
/// values today: static ~105:1, semconv ~180:1, static+trace_context ~60:1
/// (per-batch random IDs become incompressible bulk inside the replayed
/// batch). Anything noticeably below this suggests the generator has
/// gained enough per-batch entropy that replay is no longer trivially
/// compressible (which would be a notable behavior change worth
/// re-examining this guard for).
const PRE_GENERATED_RATIO_FLOOR: f64 = 50.0;

/// Load the OpenTelemetry semantic-conventions registry for tests.
///
/// The registry source can be overridden via the `OTAP_SEMCONV_REGISTRY`
/// env var (any `VirtualDirectoryPath`-parseable value, e.g. a local path).
/// When unset, the upstream Git repo is used.
fn load_semconv_registry() -> ResolvedRegistry {
    let registry_path = std::env::var("OTAP_SEMCONV_REGISTRY")
        .map(|path| {
            path.parse::<VirtualDirectoryPath>()
                .expect("valid OTAP_SEMCONV_REGISTRY")
        })
        .unwrap_or_else(|_| VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: None,
        });

    let mut semconv_errors = Vec::new();
    let registry_repo = RegistryRepo::try_new(None, &registry_path, &mut semconv_errors)
        .expect("semantic convention registry");
    let registry = match SchemaResolver::load_semconv_repository(registry_repo, false) {
        WResult::Ok(registry) | WResult::OkWithNFEs(registry, _) => registry,
        WResult::FatalErr(err) => panic!("failed to load semantic convention registry: {err}"),
    };
    let resolved_schema = match SchemaResolver::resolve(registry, true) {
        WResult::Ok(schema) | WResult::OkWithNFEs(schema, _) => schema,
        WResult::FatalErr(err) => {
            panic!("failed to resolve semantic convention registry: {err}");
        }
    };

    ResolvedRegistry::try_from_resolved_registry(
        &resolved_schema.registry,
        resolved_schema.catalog(),
    )
    .expect("resolved semantic convention registry")
}

/// Generate a `static` log batch with a per-record shape chosen so its
/// byte/record footprint roughly matches semconv-generated logs (~360 B/rec):
/// default body template pool (~150-char bodies), 4 attributes, no trace
/// context. Keeps the static vs semconv compression-ratio numbers
/// apples-to-apples.
fn static_batch_bytes() -> Vec<u8> {
    static_otlp_logs_with_config(BATCH_SIZE, None, Some(4), false, None).encode_to_vec()
}

/// Same as [`static_batch_bytes`] but with `use_trace_context: true`, which
/// adds a fresh random 16-byte trace_id and 8-byte span_id per record. These
/// 24 bytes/record of pure entropy are essentially incompressible and let us
/// see how much compression drops once realistic correlation IDs are in the
/// payload.
fn static_with_trace_ctx_batch_bytes() -> Vec<u8> {
    static_otlp_logs_with_config(BATCH_SIZE, None, Some(4), true, None).encode_to_vec()
}

/// Generate a `semconv` log batch.
fn semconv_batch_bytes(registry: &ResolvedRegistry) -> Vec<u8> {
    semconv_otlp_logs(BATCH_SIZE, registry).encode_to_vec()
}

/// Compress `bytes` with zstd level 3 and return (raw_size, compressed_size, ratio).
fn measure(bytes: &[u8]) -> (usize, usize, f64) {
    let raw_size = bytes.len();
    let compressed = zstd::bulk::compress(bytes, 3).expect("zstd compression failed");
    let compressed_size = compressed.len();
    let ratio = raw_size as f64 / compressed_size as f64;
    (raw_size, compressed_size, ratio)
}

/// Concatenate `NUM_BATCHES` independently-generated `static` batches.
fn static_fresh_stream() -> Vec<u8> {
    let mut concatenated = Vec::new();
    for _ in 0..NUM_BATCHES {
        concatenated.extend_from_slice(&static_batch_bytes());
    }
    concatenated
}

/// Concatenate `NUM_BATCHES` independently-generated `static` batches with
/// trace context enabled.
fn static_with_trace_ctx_fresh_stream() -> Vec<u8> {
    let mut concatenated = Vec::new();
    for _ in 0..NUM_BATCHES {
        concatenated.extend_from_slice(&static_with_trace_ctx_batch_bytes());
    }
    concatenated
}

/// Concatenate `NUM_BATCHES` independently-generated `semconv` batches.
fn semconv_fresh_stream(registry: &ResolvedRegistry) -> Vec<u8> {
    let mut concatenated = Vec::new();
    for _ in 0..NUM_BATCHES {
        concatenated.extend_from_slice(&semconv_batch_bytes(registry));
    }
    concatenated
}

/// Concatenate the **same** `static` batch `NUM_BATCHES` times.
fn static_pregenerated_stream() -> Vec<u8> {
    let batch = static_batch_bytes();
    let mut concatenated = Vec::with_capacity(batch.len() * NUM_BATCHES);
    for _ in 0..NUM_BATCHES {
        concatenated.extend_from_slice(&batch);
    }
    concatenated
}

/// Concatenate the **same** `static`-with-trace-context batch `NUM_BATCHES`
/// times. The trace/span IDs inside the single generated batch are random
/// but identical across replays, so zstd can still back-reference them.
fn static_with_trace_ctx_pregenerated_stream() -> Vec<u8> {
    let batch = static_with_trace_ctx_batch_bytes();
    let mut concatenated = Vec::with_capacity(batch.len() * NUM_BATCHES);
    for _ in 0..NUM_BATCHES {
        concatenated.extend_from_slice(&batch);
    }
    concatenated
}

/// Concatenate the **same** `semconv` batch `NUM_BATCHES` times.
fn semconv_pregenerated_stream(registry: &ResolvedRegistry) -> Vec<u8> {
    let batch = semconv_batch_bytes(registry);
    let mut concatenated = Vec::with_capacity(batch.len() * NUM_BATCHES);
    for _ in 0..NUM_BATCHES {
        concatenated.extend_from_slice(&batch);
    }
    concatenated
}

fn print_ratio(label: &str, raw: usize, compressed: usize, ratio: f64) {
    let total_records = BATCH_SIZE * NUM_BATCHES;
    println!(
        "{label}: raw={raw} bytes ({:.1} B/rec), compressed={compressed} bytes, ratio={ratio:.1}:1",
        raw as f64 / total_records as f64
    );
}

/// Print compression ratios for all four (source × strategy) combinations.
/// Numbers are informational — run with `--nocapture` to inspect them. The
/// asserts enforce only loose, order-of-magnitude bounds:
///
/// - `fresh` ratios land within [`FRESH_RATIO_RANGE`].
/// - `pre_generated` ratios stay above [`PRE_GENERATED_RATIO_FLOOR`] (i.e.
///   replay remains trivially compressible, as expected when batches are
///   byte-identical).
/// - For each source, `pre_generated >= fresh` (replay can never compress
///   worse than fresh batches).
#[test]
fn test_compression_ratios_across_source_and_strategy() {
    let registry = load_semconv_registry();

    let (sf_raw, sf_comp, sf_ratio) = measure(&static_fresh_stream());
    print_ratio(
        &format!("Static  fresh         ({NUM_BATCHES}x{BATCH_SIZE})"),
        sf_raw,
        sf_comp,
        sf_ratio,
    );

    let (sp_raw, sp_comp, sp_ratio) = measure(&static_pregenerated_stream());
    print_ratio(
        &format!("Static  pre_generated ({NUM_BATCHES}x{BATCH_SIZE})"),
        sp_raw,
        sp_comp,
        sp_ratio,
    );

    let (tf_raw, tf_comp, tf_ratio) = measure(&static_with_trace_ctx_fresh_stream());
    print_ratio(
        &format!("Static+TC fresh       ({NUM_BATCHES}x{BATCH_SIZE})"),
        tf_raw,
        tf_comp,
        tf_ratio,
    );

    let (tp_raw, tp_comp, tp_ratio) = measure(&static_with_trace_ctx_pregenerated_stream());
    print_ratio(
        &format!("Static+TC pre_generated ({NUM_BATCHES}x{BATCH_SIZE})"),
        tp_raw,
        tp_comp,
        tp_ratio,
    );

    let (cf_raw, cf_comp, cf_ratio) = measure(&semconv_fresh_stream(&registry));
    print_ratio(
        &format!("Semconv fresh         ({NUM_BATCHES}x{BATCH_SIZE})"),
        cf_raw,
        cf_comp,
        cf_ratio,
    );

    let (cp_raw, cp_comp, cp_ratio) = measure(&semconv_pregenerated_stream(&registry));
    print_ratio(
        &format!("Semconv pre_generated ({NUM_BATCHES}x{BATCH_SIZE})"),
        cp_raw,
        cp_comp,
        cp_ratio,
    );

    for (label, ratio) in [
        ("static/fresh", sf_ratio),
        ("static+tc/fresh", tf_ratio),
        ("semconv/fresh", cf_ratio),
    ] {
        assert!(
            FRESH_RATIO_RANGE.contains(&ratio),
            "{label} compression ratio {ratio:.1}:1 is outside expected range {:?}:1",
            FRESH_RATIO_RANGE,
        );
    }
    for (label, ratio) in [
        ("static/pre_generated", sp_ratio),
        ("static+tc/pre_generated", tp_ratio),
        ("semconv/pre_generated", cp_ratio),
    ] {
        assert!(
            ratio > PRE_GENERATED_RATIO_FLOOR,
            "{label} compression ratio {ratio:.1}:1 is below expected floor \
             {PRE_GENERATED_RATIO_FLOOR}:1 (replayed identical batches should compress trivially)",
        );
    }
    assert!(
        sp_ratio >= sf_ratio,
        "static: pre_generated ratio {sp_ratio:.1}:1 should be >= fresh ratio {sf_ratio:.1}:1",
    );
    assert!(
        tp_ratio >= tf_ratio,
        "static+tc: pre_generated ratio {tp_ratio:.1}:1 should be >= fresh ratio {tf_ratio:.1}:1",
    );
    assert!(
        cp_ratio >= cf_ratio,
        "semconv: pre_generated ratio {cp_ratio:.1}:1 should be >= fresh ratio {cf_ratio:.1}:1",
    );
}
