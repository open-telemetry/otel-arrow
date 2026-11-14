// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Basic usage example: evaluate policies against an OTAP log batch.

use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogRecord;
use otel_arrow_dfe_pdata::testing::round_trip::to_otap_logs;
use otel_arrow_dfe_policy_engine::engine::PolicyEngine;
use policy_rs::proto::tero::policy::v1::{
    AttributePath, LogField, LogMatcher, LogRename, LogTarget, LogTransform, NumericValue,
    Policy as ProtoPolicy, log_matcher, log_rename, policy,
};
use policy_rs::{Policy, PolicyRegistry};

fn main() {
    println!("=== Policy Engine Basic Usage ===\n");

    // ---------------------------------------------------------------
    // 1. Create policies
    // ---------------------------------------------------------------

    let registry = PolicyRegistry::new();
    let handle = registry.register_provider();

    // Policy A: drop all DEBUG logs (string exact match on severity_text).
    let drop_debug = create_log_policy(
        "drop-debug",
        vec![severity_text_matcher("DEBUG")],
        "none",
        None,
    );

    // Policy B: drop all logs that have a "debug_only" attribute.
    // This uses an existence check on a log attribute.
    let drop_debug_tagged = create_log_policy(
        "drop-debug-tagged",
        vec![attr_exists_matcher("debug_only")],
        "none",
        None,
    );

    // Policy C: rename attribute "old_key" -> "new_key" on ERROR logs.
    // This uses a string exact match on severity_text + a transform.
    let rename_on_error = create_log_policy(
        "rename-on-error",
        vec![severity_text_matcher("ERROR")],
        "all",
        Some(LogTransform {
            rename: vec![LogRename {
                from: Some(log_rename::From::FromLogAttribute(attr_path("old_key"))),
                to: "new_key".to_string(),
                upsert: false,
            }],
            ..Default::default()
        }),
    );

    handle.update(vec![drop_debug, drop_debug_tagged, rename_on_error]);
    let snapshot = registry.snapshot();
    println!("Loaded {} policies\n", snapshot.len());

    // ---------------------------------------------------------------
    // 2. Create an OTAP log batch
    // ---------------------------------------------------------------

    let otap_batch = to_otap_logs(vec![
        // Log 0: has "debug_only" attr -- should be dropped by policy A
        LogRecord::build()
            .severity_text("DEBUG")
            .body(AnyValue::new_string("Debug trace info"))
            .attributes(vec![
                KeyValue::new("debug_only", AnyValue::new_bool(true)),
                KeyValue::new("component", AnyValue::new_string("cache")),
            ])
            .finish(),
        // Log 1: has status_code=503 and old_key -- should survive,
        //         "old_key" should be renamed to "new_key" by policy B
        LogRecord::build()
            .severity_text("ERROR")
            .body(AnyValue::new_string("Connection timeout"))
            .attributes(vec![
                KeyValue::new("status_code", AnyValue::new_int(503)),
                KeyValue::new("old_key", AnyValue::new_string("important_value")),
                KeyValue::new("service", AnyValue::new_string("api")),
            ])
            .finish(),
        // Log 2: has status_code=200 -- should survive unmodified
        //         (status_code < 500, so policy B doesn't match)
        LogRecord::build()
            .severity_text("INFO")
            .body(AnyValue::new_string("Request completed"))
            .attributes(vec![KeyValue::new("status_code", AnyValue::new_int(200))])
            .finish(),
        // Log 3: has status_code=500 but no "old_key" -- should survive,
        //         rename is a no-op (attribute to rename doesn't exist)
        LogRecord::build()
            .severity_text("ERROR")
            .body(AnyValue::new_string("Internal error"))
            .attributes(vec![
                KeyValue::new("status_code", AnyValue::new_int(500)),
                KeyValue::new("host", AnyValue::new_string("server-1")),
            ])
            .finish(),
        // Log 4: no special attributes -- should survive unmodified
        LogRecord::build()
            .severity_text("INFO")
            .body(AnyValue::new_string("Startup complete"))
            .attributes(vec![KeyValue::new(
                "component",
                AnyValue::new_string("main"),
            )])
            .finish(),
    ]);

    let input_rows = otap_batch
        .root_record_batch()
        .map(|rb| rb.num_rows())
        .unwrap_or(0);
    println!("Input batch: {} log records", input_rows);

    // ---------------------------------------------------------------
    // 3. Evaluate policies
    // ---------------------------------------------------------------

    let engine = PolicyEngine::new();
    let result = engine
        .evaluate_and_transform(&snapshot, otap_batch)
        .expect("evaluation should succeed");

    // ---------------------------------------------------------------
    // 4. Inspect results
    // ---------------------------------------------------------------

    let output_rows = result
        .root_record_batch()
        .map(|rb| rb.num_rows())
        .unwrap_or(0);
    println!("Output batch: {} log records", output_rows);
    println!(
        "Dropped: {} records\n",
        input_rows.saturating_sub(output_rows)
    );

    // Print some columns from the output
    if let Some(root_rb) = result.root_record_batch() {
        if let Some(col) = root_rb.column_by_name("severity_text") {
            println!("severity_text: {:?}\n", col);
        }
    }

    // Print attribute keys from the output to verify rename
    if let Some(attrs_batch) = result
        .get(otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType::LogAttrs)
    {
        println!("Attribute batch ({} rows):", attrs_batch.num_rows());
        if let Some(key_col) = attrs_batch.column_by_name("key") {
            println!("  keys: {:?}", key_col);
        }
    }

    // Print policy stats
    println!("\n--- Policy Stats ---");
    for entry in snapshot.iter() {
        let hits = entry.stats.hits();
        let misses = entry.stats.misses();
        println!("{}: {} hits, {} misses", entry.policy.id(), hits, misses);
    }
}

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn attr_path(s: &str) -> AttributePath {
    AttributePath {
        path: vec![s.to_string()],
    }
}

/// Matcher: severity_text == value (exact string match via Vectorscan).
fn severity_text_matcher(value: &str) -> LogMatcher {
    LogMatcher {
        field: Some(log_matcher::Field::LogField(LogField::SeverityText.into())),
        r#match: Some(log_matcher::Match::Exact(value.to_string())),
        negate: false,
        case_insensitive: false,
    }
}

/// Matcher: log_attribute["key"] exists.
fn attr_exists_matcher(key: &str) -> LogMatcher {
    LogMatcher {
        field: Some(log_matcher::Field::LogAttribute(attr_path(key))),
        r#match: Some(log_matcher::Match::Exists(true)),
        negate: false,
        case_insensitive: false,
    }
}

fn create_log_policy(
    id: &str,
    matchers: Vec<LogMatcher>,
    keep: &str,
    transform: Option<LogTransform>,
) -> Policy {
    Policy::new(ProtoPolicy {
        id: id.to_string(),
        name: id.to_string(),
        enabled: true,
        target: Some(policy::Target::Log(LogTarget {
            r#match: matchers,
            keep: keep.to_string(),
            transform,
            sample_key: None,
        })),
        ..Default::default()
    })
}
