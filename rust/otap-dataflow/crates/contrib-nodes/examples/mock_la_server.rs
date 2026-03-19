// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Workspace lints are strict for library code; relax for this CLI dev tool.
#![allow(clippy::print_stdout, clippy::print_stderr)]
#![allow(missing_docs, unused_results)]

//! Mock Azure Monitor Logs Ingestion API server for local testing.
//!
//! Simulates the Azure Monitor Logs Ingestion API endpoint for development
//! and performance testing without incurring Log Analytics costs.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example mock_la_server -p otap-df-contrib-nodes --features azure-monitor-exporter -- --port 9999
//! ```
//!
//! Then point your Azure Monitor Exporter config at `http://localhost:9999`
//! with `auth.method: "dev"`.
//!
//! # Error simulation
//!
//! ```bash
//! # 10% of requests return 500
//! cargo run --example mock_la_server -p otap-df-contrib-nodes --features azure-monitor-exporter -- --fail-rate 0.1
//!
//! # 10% of requests return 429 with Retry-After: 5
//! cargo run --example mock_la_server -p otap-df-contrib-nodes --features azure-monitor-exporter -- --fail-rate 0.1 --retry-after 5
//!
//! # Artificial 200ms latency
//! cargo run --example mock_la_server -p otap-df-contrib-nodes --features azure-monitor-exporter -- --latency 200ms
//!
//! # Return 503 after 1000 successful requests
//! cargo run --example mock_la_server -p otap-df-contrib-nodes --features azure-monitor-exporter -- --fail-after 1000
//! ```

use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Router;
use clap::Parser;
use flate2::read::GzDecoder;
use rand::RngExt;

/// CLI arguments for the mock server.
#[derive(Parser, Debug)]
#[command(name = "mock-la-server")]
#[command(about = "Mock Azure Monitor Logs Ingestion API server for local testing")]
struct Cli {
    /// Port to listen on.
    #[arg(short, long, default_value_t = 9999)]
    port: u16,

    /// Fraction of requests that receive a simulated server error (0.0–1.0).
    /// Returns 500 by default, or 429 if --retry-after is also set.
    #[arg(long, default_value_t = 0.0)]
    fail_rate: f64,

    /// When failing, return 429 with this Retry-After value (seconds).
    /// Without this flag, failures return 500.
    #[arg(long)]
    retry_after: Option<u64>,

    /// Artificial response latency (e.g., "200ms", "1s").
    #[arg(long, value_parser = parse_duration)]
    latency: Option<Duration>,

    /// Fraction of requests that receive 401 Unauthorized (0.0–1.0).
    #[arg(long, default_value_t = 0.0)]
    unauthorized_rate: f64,

    /// Return 413 if the compressed body exceeds this many bytes.
    #[arg(long)]
    payload_too_large: Option<usize>,

    /// Return 503 after this many successful requests.
    #[arg(long)]
    fail_after: Option<u64>,

    /// Stats reporting interval (e.g., "10s", "30s").
    #[arg(long, default_value = "10s", value_parser = parse_duration)]
    stats_interval: Duration,
}

fn parse_duration(s: &str) -> Result<Duration, humantime::DurationError> {
    humantime::parse_duration(s)
}

/// Atomic counters for request statistics.
struct Stats {
    total_requests: AtomicU64,
    total_entries: AtomicU64,
    total_compressed_bytes: AtomicU64,
    total_decompressed_bytes: AtomicU64,
    success_count: AtomicU64,
    // Per-interval counters, reset after each stats report.
    interval_requests: AtomicU64,
    interval_entries: AtomicU64,
    interval_compressed_bytes: AtomicU64,
    interval_decompressed_bytes: AtomicU64,
}

#[derive(Clone)]
struct AppState {
    cli: Arc<Cli>,
    stats: Arc<Stats>,
}

/// Handler for POST /dataCollectionRules/{dcr}/streams/{stream}
async fn ingest_handler(
    State(state): State<AppState>,
    Path((dcr, stream)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let compressed_size = body.len() as u64;

    // Update request counters.
    state
        .stats
        .total_requests
        .fetch_add(1, Ordering::Relaxed);
    state
        .stats
        .interval_requests
        .fetch_add(1, Ordering::Relaxed);
    state
        .stats
        .total_compressed_bytes
        .fetch_add(compressed_size, Ordering::Relaxed);
    state
        .stats
        .interval_compressed_bytes
        .fetch_add(compressed_size, Ordering::Relaxed);

    // Simulate latency.
    if let Some(latency) = state.cli.latency {
        tokio::time::sleep(latency).await;
    }

    // Check payload size limit.
    if let Some(max_size) = state.cli.payload_too_large {
        if body.len() > max_size {
            println!(
                "[mock-la] POST dcr={dcr} stream={stream} \
                 — 413 Payload Too Large ({} > {max_size})",
                body.len()
            );
            return (StatusCode::PAYLOAD_TOO_LARGE, "Payload Too Large").into_response();
        }
    }

    // Check fail-after threshold.
    let success_so_far = state.stats.success_count.load(Ordering::Relaxed);
    if let Some(fail_after) = state.cli.fail_after {
        if success_so_far >= fail_after {
            println!(
                "[mock-la] POST dcr={dcr} stream={stream} \
                 — 503 (fail-after {fail_after} reached)"
            );
            return (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable").into_response();
        }
    }

    // Random failure simulation.
    let mut rng = rand::rng();

    if state.cli.unauthorized_rate > 0.0 && rng.random::<f64>() < state.cli.unauthorized_rate {
        println!("[mock-la] POST dcr={dcr} stream={stream} — 401 (simulated)");
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    if state.cli.fail_rate > 0.0 && rng.random::<f64>() < state.cli.fail_rate {
        if let Some(retry_secs) = state.cli.retry_after {
            println!(
                "[mock-la] POST dcr={dcr} stream={stream} \
                 — 429 (simulated, Retry-After: {retry_secs}s)"
            );
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, retry_secs.to_string())],
                "Rate Limited",
            )
                .into_response();
        }
        println!("[mock-la] POST dcr={dcr} stream={stream} — 500 (simulated)");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response();
    }

    // Decompress and count entries.
    let result = match decompress_and_count(&body, &headers) {
        Ok(r) => r,
        Err(e) => {
            println!("[mock-la] POST dcr={dcr} stream={stream} — 400 Bad Request: {e}");
            return (StatusCode::BAD_REQUEST, format!("Bad Request: {e}")).into_response();
        }
    };
    let entry_count = result.entry_count;
    state
        .stats
        .total_decompressed_bytes
        .fetch_add(result.decompressed_size, Ordering::Relaxed);
    state
        .stats
        .interval_decompressed_bytes
        .fetch_add(result.decompressed_size, Ordering::Relaxed);

    // Update success stats.
    state.stats.success_count.fetch_add(1, Ordering::Relaxed);
    state
        .stats
        .total_entries
        .fetch_add(entry_count, Ordering::Relaxed);
    state
        .stats
        .interval_entries
        .fetch_add(entry_count, Ordering::Relaxed);

    StatusCode::NO_CONTENT.into_response()
}

/// Result of decompressing and counting entries.
struct DecompressResult {
    entry_count: u64,
    decompressed_size: u64,
}

/// Decompress (if gzipped) and count JSON array entries in the body.
fn decompress_and_count(body: &[u8], headers: &HeaderMap) -> Result<DecompressResult, String> {
    let is_gzip = headers
        .get(header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("gzip"));

    let json_bytes = if is_gzip {
        let mut decoder = GzDecoder::new(body);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| format!("gzip decompress error: {e}"))?;
        decompressed
    } else {
        body.to_vec()
    };

    let decompressed_size = json_bytes.len() as u64;

    let value: serde_json::Value =
        serde_json::from_slice(&json_bytes).map_err(|e| format!("JSON parse error: {e}"))?;

    let entry_count = match value.as_array() {
        Some(arr) => arr.len() as u64,
        None => 1, // Single object counts as 1 entry.
    };

    Ok(DecompressResult {
        entry_count,
        decompressed_size,
    })
}

/// Format a byte count as a human-readable string.
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

/// Periodically print aggregate stats to stdout.
async fn stats_reporter(state: AppState) {
    let interval = state.cli.stats_interval;
    let start = std::time::Instant::now();
    loop {
        tokio::time::sleep(interval).await;

        let int_req = state.stats.interval_requests.swap(0, Ordering::Relaxed);
        let int_entries = state.stats.interval_entries.swap(0, Ordering::Relaxed);
        let int_comp = state
            .stats
            .interval_compressed_bytes
            .swap(0, Ordering::Relaxed);
        let int_decomp = state
            .stats
            .interval_decompressed_bytes
            .swap(0, Ordering::Relaxed);

        let total_req = state.stats.total_requests.load(Ordering::Relaxed);
        let total_entries = state.stats.total_entries.load(Ordering::Relaxed);
        let total_comp = state.stats.total_compressed_bytes.load(Ordering::Relaxed);
        let total_decomp = state.stats.total_decompressed_bytes.load(Ordering::Relaxed);
        let elapsed = start.elapsed().as_secs_f64();

        let secs = interval.as_secs_f64();
        let int_entries_per_sec = if secs > 0.0 { int_entries as f64 / secs } else { 0.0 };
        let int_comp_per_sec = if secs > 0.0 { int_comp as f64 / secs } else { 0.0 };

        let cum_entries_per_sec = if elapsed > 0.0 { total_entries as f64 / elapsed } else { 0.0 };
        let cum_comp_per_sec = if elapsed > 0.0 { total_comp as f64 / elapsed } else { 0.0 };

        println!("[mock-la] --- Stats ---");
        println!(
            "[mock-la]   Last {interval:?}: {int_req} reqs | {int_entries} entries | \
             {int_entries_per_sec:.0} entries/s | {}/s compressed | {} decompressed",
            format_bytes(int_comp_per_sec as u64),
            format_bytes(int_decomp),
        );
        println!(
            "[mock-la]   Cumulative ({elapsed:.0}s): {total_req} reqs | {total_entries} entries | \
             {cum_entries_per_sec:.0} entries/s | {}/s compressed | {} total compressed | {} total decompressed",
            format_bytes(cum_comp_per_sec as u64),
            format_bytes(total_comp),
            format_bytes(total_decomp),
        );
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let port = cli.port;

    println!("[mock-la] Mock Azure Monitor Logs Ingestion API Server");
    println!("[mock-la] Listening on 0.0.0.0:{port}");

    if cli.fail_rate > 0.0 {
        if let Some(retry_secs) = cli.retry_after {
            println!(
                "[mock-la] Failure simulation: {:.0}% → 429 with Retry-After: {retry_secs}s",
                cli.fail_rate * 100.0
            );
        } else {
            println!(
                "[mock-la] Failure simulation: {:.0}% → 500",
                cli.fail_rate * 100.0
            );
        }
    }
    if cli.unauthorized_rate > 0.0 {
        println!(
            "[mock-la] Unauthorized simulation: {:.0}% → 401",
            cli.unauthorized_rate * 100.0
        );
    }
    if let Some(latency) = cli.latency {
        println!("[mock-la] Artificial latency: {latency:?}");
    }
    if let Some(max) = cli.payload_too_large {
        println!("[mock-la] Payload size limit: {max} bytes");
    }
    if let Some(n) = cli.fail_after {
        println!("[mock-la] Will return 503 after {n} successful requests");
    }
    println!("[mock-la] Stats interval: {:?}", cli.stats_interval);
    println!();

    let state = AppState {
        cli: Arc::new(cli),
        stats: Arc::new(Stats {
            total_requests: AtomicU64::new(0),
            total_entries: AtomicU64::new(0),
            total_compressed_bytes: AtomicU64::new(0),
            total_decompressed_bytes: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            interval_requests: AtomicU64::new(0),
            interval_entries: AtomicU64::new(0),
            interval_compressed_bytes: AtomicU64::new(0),
            interval_decompressed_bytes: AtomicU64::new(0),
        }),
    };

    // Spawn periodic stats reporter.
    let _stats_handle = tokio::spawn(stats_reporter(state.clone()));

    let app = Router::new()
        .route(
            "/dataCollectionRules/{dcr}/streams/{stream}",
            post(ingest_handler),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("failed to bind to port");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
