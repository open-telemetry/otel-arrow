// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration-style tests for command runners and CLI output contracts.
//!
//! These tests exercise parsed `dfctl` commands against mock admin endpoints or
//! local helpers. Each test documents the user scenario and the command contract
//! it protects so reviewers can understand which behavior is intentionally
//! stable for humans, scripts, and agents.

use crate::args::{ColorChoice, MetricsShape, MutationOutput, StreamOutput};
use crate::commands::output::{duration_to_admin_timeout_secs, validate_mutation_output_mode};
use crate::commands::watch::{watch_logs, watch_metrics};
use crate::crypto::ensure_crypto_provider;
use crate::pipeline_config_io::load_pipeline_config;
use crate::style::HumanStyle;
use crate::troubleshoot::{LogFilters, MetricsFilters};
use crate::{Cli, run, run_with_terminal, run_with_terminal_and_diagnostics};
use clap::Parser;
use otap_df_admin_api::config::pipeline::{PipelineConfig, PipelineConfigBuilder, PipelineType};
use otap_df_admin_api::telemetry::MetricsOptions;
use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};
use serde_json::json;
use std::fs;
use std::time::Duration;
use tempfile::tempdir;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn pipeline_config() -> PipelineConfig {
    PipelineConfigBuilder::new()
        .add_receiver("ingress", "receiver:otlp", None)
        .add_exporter("egress", "exporter:debug", None)
        .to("ingress", "egress")
        .build(PipelineType::Otap, "tenant-a", "ingest")
        .expect("pipeline config")
}

/// Scenario: the CLI runs `dfctl engine status --output json` against an admin
/// endpoint.
/// Guarantees: the command hits the committed engine status route and emits the
/// decoded response as JSON.
#[tokio::test]
async fn engine_status_json_command_hits_expected_route() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "generatedAt": "2026-01-01T00:00:00Z",
            "pipelines": {}
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "engine",
        "status",
        "--output",
        "json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output = String::from_utf8(stdout).expect("utf8");
    assert!(output.contains("\"generatedAt\""));
}

/// Scenario: the CLI runs a one-shot read command in agent JSON mode.
/// Guarantees: the output includes a stable dfctl envelope with provenance and
/// the SDK response nested under `data`.
#[tokio::test]
async fn engine_status_agent_json_wraps_data() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "generatedAt": "2026-01-01T00:00:00Z",
            "pipelines": {}
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "engine",
        "status",
        "--output",
        "agent-json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output: serde_json::Value =
        serde_json::from_slice(&stdout).expect("agent output should be JSON");
    assert_eq!(output["schemaVersion"], "dfctl/v1");
    assert_eq!(output["type"], "snapshot");
    assert_eq!(output["data"]["generatedAt"], "2026-01-01T00:00:00Z");
}

/// Scenario: the CLI runs `dfctl config view --output json` with a profile and
/// an explicit URL override.
/// Guarantees: the command resolves connection settings locally, performs no
/// admin API call, and redacts the client key path to a configured flag.
#[tokio::test]
async fn config_view_json_reports_resolved_connection() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("dfctl-profile.yaml");
    fs::write(
        &path,
        "url: https://profile.example.com\nclient_key_file: /secret/client.key\n",
    )
    .expect("write profile");

    let cli = Cli::try_parse_from([
        "dfctl",
        "--profile-file",
        path.to_str().expect("profile path"),
        "--url",
        "https://admin.example.com/engine-a",
        "config",
        "view",
        "--output",
        "json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output: serde_json::Value = serde_json::from_slice(&stdout).expect("json output");
    assert_eq!(output["url"], "https://admin.example.com:443/engine-a");
    assert_eq!(output["profileFile"], path.display().to_string());
    assert_eq!(output["tls"]["clientKeyFileConfigured"], true);
    assert_eq!(output["tls"]["enabled"], true);
}

/// Scenario: an automation client asks for a single output schema by name.
/// Guarantees: schema discovery is local-only and returns a versioned schema
/// document without contacting an admin endpoint.
#[tokio::test]
async fn schemas_named_schema_outputs_json_document() {
    let cli = Cli::try_parse_from(["dfctl", "schemas", "dfctl.error.v1", "--output", "json"])
        .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output: serde_json::Value = serde_json::from_slice(&stdout).expect("json");
    assert_eq!(output["schemaVersion"], "dfctl-schema-document/v1");
    assert_eq!(output["name"], "dfctl.error.v1");
    assert_eq!(output["schema"]["title"], "dfctl structured error");
}

/// Scenario: a human invokes a command with repeated verbose flags.
/// Guarantees: diagnostics are written to stderr and keep stdout reserved for
/// the command result.
#[tokio::test]
async fn verbose_config_view_writes_diagnostics_to_stderr() {
    let cli = Cli::try_parse_from(["dfctl", "-vv", "config", "view"]).expect("parse");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with_terminal_and_diagnostics(cli, &mut stdout, false, &mut stderr)
        .await
        .expect("run");

    let stdout = String::from_utf8(stdout).expect("stdout utf8");
    let stderr = String::from_utf8(stderr).expect("stderr utf8");
    assert!(stdout.contains("dfctl configuration"));
    assert!(stderr.contains("dfctl: target=http://127.0.0.1:8085"));
    assert!(stderr.contains("connect_timeout=3s"));
}

/// Scenario: the CLI requests compact metrics in non-human output mode.
/// Guarantees: the HTTP request uses the compact `format=json_compact` query
/// shape expected by the admin API.
#[tokio::test]
async fn metrics_get_uses_compact_route_shape() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/telemetry/metrics"))
        .and(query_param("format", "json_compact"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "metric_sets": []
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "telemetry",
        "metrics",
        "get",
        "--shape",
        "compact",
        "--output",
        "json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");
}

/// Scenario: the CLI requests full metrics in non-human output mode.
/// Guarantees: the HTTP request uses the full `format=json` query shape
/// expected by the admin API.
#[tokio::test]
async fn metrics_get_uses_full_route_shape() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/telemetry/metrics"))
        .and(query_param("format", "json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "metric_sets": []
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "telemetry",
        "metrics",
        "get",
        "--shape",
        "full",
        "--output",
        "json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");
}

/// Scenario: a human asks for retained logs with `--color always`.
/// Guarantees: human log rendering emits ANSI color sequences even outside a
/// detected terminal when the flag forces color on.
#[tokio::test]
async fn logs_get_human_color_always_emits_ansi() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/telemetry/logs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "oldest_seq": 1,
            "newest_seq": 1,
            "next_seq": 2,
            "truncated_before_seq": null,
            "dropped_on_ingest": 0,
            "dropped_on_retention": 0,
            "retained_bytes": 10,
            "logs": [
                {
                    "seq": 1,
                    "timestamp": "2026-01-01T00:00:00Z",
                    "level": "INFO",
                    "target": "test.target",
                    "event_name": "log.1",
                    "file": null,
                    "line": null,
                    "rendered": "hello world",
                    "contexts": []
                }
            ]
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "--color",
        "always",
        "telemetry",
        "logs",
        "get",
        "--limit",
        "1",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output = String::from_utf8(stdout).expect("utf8");
    assert!(output.contains("\u{1b}["));
    assert!(output.contains("hello world"));
}

/// Scenario: compact metrics are rendered for a non-terminal stdout without
/// forcing color.
/// Guarantees: auto color detection stays plain and does not inject ANSI
/// sequences into the human-readable metrics output.
#[tokio::test]
async fn metrics_get_human_auto_stays_plain_off_terminal() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/telemetry/metrics"))
        .and(query_param("format", "json_compact"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "metric_sets": [
                {
                    "name": "engine.runtime",
                    "attributes": {},
                    "metrics": {
                        "pipelines": 3
                    }
                }
            ]
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "telemetry",
        "metrics",
        "get",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output = String::from_utf8(stdout).expect("utf8");
    assert!(!output.contains("\u{1b}["));
    assert!(output.contains("metric_set: engine.runtime"));
}

/// Scenario: compact metrics are rendered in a terminal with `--color always`.
/// Guarantees: terminal-aware rendering keeps ANSI styling enabled when the user
/// explicitly forces color.
#[tokio::test]
async fn metrics_get_human_color_always_emits_ansi_on_terminal() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/telemetry/metrics"))
        .and(query_param("format", "json_compact"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "metric_sets": [
                {
                    "name": "engine.runtime",
                    "attributes": {},
                    "metrics": {
                        "pipelines": 3
                    }
                }
            ]
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "--color",
        "always",
        "telemetry",
        "metrics",
        "get",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run_with_terminal(cli, &mut stdout, true)
        .await
        .expect("run_with_terminal");

    let output = String::from_utf8(stdout).expect("utf8");
    assert!(output.contains("\u{1b}["));
    assert!(output.contains("engine.runtime"));
}

/// Scenario: the user asks for JSON metrics output while also forcing color.
/// Guarantees: machine-readable JSON output stays free of ANSI sequences.
#[tokio::test]
async fn metrics_json_output_ignores_color_setting() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/telemetry/metrics"))
        .and(query_param("format", "json_compact"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "metric_sets": []
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "--color",
        "always",
        "telemetry",
        "metrics",
        "get",
        "--output",
        "json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run_with_terminal(cli, &mut stdout, true)
        .await
        .expect("run_with_terminal");

    let output = String::from_utf8(stdout).expect("utf8");
    assert!(!output.contains("\u{1b}["));
    assert!(output.contains("\"timestamp\""));
}

/// Scenario: a waited pipeline reconfigure call is issued from a YAML file.
/// Guarantees: the request body, wait query parameters, and timeout rounding
/// match the current admin API contract.
#[tokio::test]
async fn reconfigure_sends_wait_query_and_request_body() {
    let server = MockServer::start().await;
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("pipeline.yaml");
    fs::write(
        &file_path,
        serde_yaml::to_string(&pipeline_config()).expect("yaml"),
    )
    .expect("write");

    Mock::given(method("PUT"))
        .and(path("/api/v1/groups/tenant-a/pipelines/ingest"))
        .and(query_param("wait", "true"))
        .and(query_param("timeout_secs", "30"))
        .and(body_json(json!({
            "pipeline": serde_json::to_value(pipeline_config()).expect("value"),
            "stepTimeoutSecs": 60,
            "drainTimeoutSecs": 60
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "rolloutId": "rollout-1",
            "pipelineGroupId": "tenant-a",
            "pipelineId": "ingest",
            "action": "replace",
            "state": "succeeded",
            "targetGeneration": 2,
            "previousGeneration": 1,
            "startedAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:00Z",
            "cores": []
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "pipelines",
        "reconfigure",
        "tenant-a",
        "ingest",
        "--file",
        file_path.to_str().expect("path"),
        "--wait",
        "--wait-timeout",
        "30s",
        "--output",
        "json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");
}

/// Scenario: a pipeline reconfigure is checked with client-side dry-run.
/// Guarantees: the CLI parses and validates the supplied config, emits a
/// structured preflight-only outcome, and does not require a live operation.
#[tokio::test]
async fn reconfigure_dry_run_emits_preflight_outcome() {
    let server = MockServer::start().await;
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("pipeline.yaml");
    fs::write(
        &file_path,
        serde_yaml::to_string(&pipeline_config()).expect("yaml"),
    )
    .expect("write");

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "pipelines",
        "reconfigure",
        "tenant-a",
        "ingest",
        "--file",
        file_path.to_str().expect("path"),
        "--dry-run",
        "--output",
        "json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output: serde_json::Value = serde_json::from_slice(&stdout).expect("json");
    assert_eq!(output["outcome"], "preflight_only");
    assert_eq!(output["data"]["mode"], "preflight-only");
    assert_eq!(output["data"]["operation"], "pipelines.reconfigure");
    assert_eq!(output["data"]["serverValidation"], false);
    assert_eq!(output["data"]["target"]["pipelineGroupId"], "tenant-a");
}

/// Scenario: a pipeline shutdown is checked with client-side dry-run.
/// Guarantees: the CLI emits a mutation-shaped preflight response and avoids
/// creating a shutdown operation.
#[tokio::test]
async fn shutdown_dry_run_emits_preflight_outcome() {
    let server = MockServer::start().await;
    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "pipelines",
        "shutdown",
        "tenant-a",
        "ingest",
        "--dry-run",
        "--output",
        "json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output: serde_json::Value = serde_json::from_slice(&stdout).expect("json");
    assert_eq!(output["outcome"], "preflight_only");
    assert_eq!(output["data"]["operation"], "pipelines.shutdown");
    assert_eq!(output["data"]["serverValidation"], false);
}

/// Scenario: the user watches a rollout resource in NDJSON mode and the first
/// poll already returns a terminal success state.
/// Guarantees: the watch loop emits one snapshot event and exits cleanly.
#[tokio::test]
async fn rollout_watch_emits_ndjson_and_stops_on_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/groups/tenant-a/pipelines/ingest/rollouts/rollout-1",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "rolloutId": "rollout-1",
            "pipelineGroupId": "tenant-a",
            "pipelineId": "ingest",
            "action": "replace",
            "state": "succeeded",
            "targetGeneration": 2,
            "previousGeneration": 1,
            "startedAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:01Z",
            "cores": []
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "pipelines",
        "rollouts",
        "watch",
        "tenant-a",
        "ingest",
        "rollout-1",
        "--output",
        "ndjson",
        "--interval",
        "100ms",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output = String::from_utf8(stdout).expect("utf8");
    assert!(output.contains("\"resource\":\"pipeline_rollout\""));
}

/// Scenario: the user watches a shutdown resource in NDJSON mode and the first
/// poll already returns a terminal success state.
/// Guarantees: the watch loop emits one snapshot event and exits cleanly.
#[tokio::test]
async fn shutdown_watch_emits_ndjson_and_stops_on_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/groups/tenant-a/pipelines/ingest/shutdowns/shutdown-1",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "shutdownId": "shutdown-1",
            "pipelineGroupId": "tenant-a",
            "pipelineId": "ingest",
            "state": "succeeded",
            "startedAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:01Z",
            "cores": []
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "pipelines",
        "shutdowns",
        "watch",
        "tenant-a",
        "ingest",
        "shutdown-1",
        "--output",
        "ndjson",
        "--interval",
        "100ms",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output = String::from_utf8(stdout).expect("utf8");
    assert!(output.contains("\"resource\":\"pipeline_shutdown\""));
}

/// Scenario: the logs watch command starts in `--tail` mode without an explicit
/// `--after` cursor.
/// Guarantees: the client uses `next_seq` from the tail bootstrap response as
/// the next polling cursor so retained logs are not replayed.
#[tokio::test]
async fn logs_watch_uses_next_seq_as_after_cursor() {
    ensure_crypto_provider().expect("crypto provider");
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/telemetry/logs"))
        .and(query_param("limit", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "oldest_seq": 49,
            "newest_seq": 50,
            "next_seq": 50,
            "truncated_before_seq": null,
            "dropped_on_ingest": 0,
            "dropped_on_retention": 0,
            "retained_bytes": 10,
            "logs": [
                {
                    "seq": 49,
                    "timestamp": "2026-01-01T00:00:00Z",
                    "level": "INFO",
                    "target": "test",
                    "event_name": "log.49",
                    "file": null,
                    "line": null,
                    "rendered": "log 49",
                    "contexts": []
                },
                {
                    "seq": 50,
                    "timestamp": "2026-01-01T00:00:01Z",
                    "level": "INFO",
                    "target": "test",
                    "event_name": "log.50",
                    "file": null,
                    "line": null,
                    "rendered": "log 50",
                    "contexts": []
                }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/telemetry/logs"))
        .and(query_param("after", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "oldest_seq": 49,
            "newest_seq": 50,
            "next_seq": 50,
            "truncated_before_seq": null,
            "dropped_on_ingest": 0,
            "dropped_on_retention": 0,
            "retained_bytes": 10,
            "logs": []
        })))
        .mount(&server)
        .await;

    let client = AdminClient::builder()
        .http(HttpAdminClientSettings::new(
            AdminEndpoint::from_url(&server.uri()).expect("endpoint"),
        ))
        .build()
        .expect("client");

    let mut stdout = Vec::new();
    let result = tokio::time::timeout(
        Duration::from_millis(200),
        watch_logs(
            &client,
            &mut stdout,
            HumanStyle::resolve(ColorChoice::Never, false),
            None,
            Some(2),
            None,
            LogFilters::default(),
            Duration::from_millis(1),
            StreamOutput::Ndjson,
        ),
    )
    .await;

    assert!(result.is_err(), "watch should still be polling");
    let output = String::from_utf8(stdout).expect("utf8");
    assert_eq!(
        output.lines().count(),
        2,
        "expected retained logs only once"
    );
    assert!(output.contains("\"seq\":49"));
    assert!(output.contains("\"seq\":50"));
}

/// Scenario: a human watches compact metrics with forced color in stream mode.
/// Guarantees: the stream header stays styled and the watch loop keeps using the
/// human stream framing instead of machine-readable NDJSON.
#[tokio::test]
async fn metrics_watch_human_color_always_styles_stream_header() {
    ensure_crypto_provider().expect("crypto provider");
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/telemetry/metrics"))
        .and(query_param("format", "json_compact"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "metric_sets": [
                {
                    "name": "engine.runtime",
                    "attributes": {},
                    "metrics": {
                        "pipelines": 3
                    }
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = AdminClient::builder()
        .http(HttpAdminClientSettings::new(
            AdminEndpoint::from_url(&server.uri()).expect("endpoint"),
        ))
        .build()
        .expect("client");

    let mut stdout = Vec::new();
    let result = tokio::time::timeout(
        Duration::from_millis(200),
        watch_metrics(
            &client,
            &mut stdout,
            HumanStyle::resolve(ColorChoice::Always, false),
            MetricsShape::Compact,
            MetricsOptions {
                reset: false,
                keep_all_zeroes: false,
            },
            MetricsFilters::default(),
            Duration::from_millis(1),
            StreamOutput::Human,
        ),
    )
    .await;

    assert!(result.is_err(), "watch should still be polling");
    let output = String::from_utf8(stdout).expect("utf8");
    assert!(output.contains("\u{1b}["));
    assert!(output.contains("[telemetry_metrics]"));
}

/// Scenario: the CLI runs `pipelines describe --output json`.
/// Guarantees: the client fetches details, status, livez, and readyz and
/// includes all of them in the describe report.
#[tokio::test]
async fn pipeline_describe_json_fetches_details_status_and_probes() {
    let server = MockServer::start().await;
    let details_value = serde_json::to_value(pipeline_config()).expect("pipeline value");

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/tenant-a/pipelines/ingest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "pipelineGroupId": "tenant-a",
            "pipelineId": "ingest",
            "activeGeneration": 7,
            "pipeline": details_value,
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/tenant-a/pipelines/ingest/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "True",
                    "reason": "Running"
                }
            ],
            "totalCores": 1,
            "runningCores": 1,
            "cores": {
                "0": {
                    "phase": "running",
                    "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                    "conditions": [],
                    "deletePending": false,
                    "recentEvents": [
                        {
                            "Engine": {
                                "key": {
                                    "pipeline_group_id": "tenant-a",
                                    "pipeline_id": "ingest",
                                    "core_id": 0
                                },
                                "time": "2026-01-01T00:00:00Z",
                                "type": {
                                    "Success": "Ready"
                                }
                            }
                        }
                    ]
                }
            },
            "activeGeneration": 7
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/tenant-a/pipelines/ingest/livez"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "ok"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/tenant-a/pipelines/ingest/readyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "ok"
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "pipelines",
        "describe",
        "tenant-a",
        "ingest",
        "--output",
        "json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output = String::from_utf8(stdout).expect("utf8");
    assert!(output.contains("\"details\""));
    assert!(output.contains("\"recentEvents\""));
    assert!(output.contains("\"livez\""));
    assert!(output.contains("\"readyz\""));
}

/// Scenario: the user filters retained logs by one pipeline scope.
/// Guarantees: client-side log filtering keeps only entries whose resolved
/// context matches the requested group and pipeline ids.
#[tokio::test]
async fn logs_get_filters_by_pipeline_scope_client_side() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/telemetry/logs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "oldest_seq": 1,
            "newest_seq": 2,
            "next_seq": 3,
            "truncated_before_seq": null,
            "dropped_on_ingest": 0,
            "dropped_on_retention": 0,
            "retained_bytes": 128,
            "logs": [
                {
                    "seq": 1,
                    "timestamp": "2026-01-01T00:00:00Z",
                    "level": "INFO",
                    "target": "controller",
                    "event_name": "matching",
                    "file": null,
                    "line": null,
                    "rendered": "matched",
                    "contexts": [
                        {
                            "entity_key": "EntityKey(1)",
                            "schema_name": "node.attrs",
                            "attributes": {
                                "pipeline.group.id": { "String": "tenant-a" },
                                "pipeline.id": { "String": "ingest" },
                                "node.id": { "String": "receiver" }
                            }
                        }
                    ]
                },
                {
                    "seq": 2,
                    "timestamp": "2026-01-01T00:00:01Z",
                    "level": "INFO",
                    "target": "controller",
                    "event_name": "other",
                    "file": null,
                    "line": null,
                    "rendered": "other",
                    "contexts": [
                        {
                            "entity_key": "EntityKey(2)",
                            "schema_name": "node.attrs",
                            "attributes": {
                                "pipeline.group.id": { "String": "tenant-b" },
                                "pipeline.id": { "String": "egress" },
                                "node.id": { "String": "receiver" }
                            }
                        }
                    ]
                }
            ]
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "telemetry",
        "logs",
        "get",
        "--group",
        "tenant-a",
        "--pipeline",
        "ingest",
        "--output",
        "json",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output = String::from_utf8(stdout).expect("utf8");
    assert!(output.contains("\"seq\": 1"));
    assert!(!output.contains("\"seq\": 2"));
}

/// Scenario: the CLI runs `groups shutdown --watch --output ndjson`.
/// Guarantees: the temporary client-side heuristic emits a group-shutdown
/// snapshot and recognizes the terminal state from groups status alone.
#[tokio::test]
async fn groups_shutdown_watch_ndjson_uses_status_heuristic() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/groups/shutdown"))
        .and(query_param("wait", "false"))
        .and(query_param("timeout_secs", "60"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "status": "accepted"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/groups/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "generatedAt": "2026-01-01T00:00:00Z",
            "pipelines": {
                "tenant-a:ingest": {
                    "conditions": [],
                    "totalCores": 1,
                    "runningCores": 0,
                    "cores": {
                        "0": {
                            "phase": "stopped",
                            "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                            "conditions": [],
                            "deletePending": false
                        }
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    let cli = Cli::try_parse_from([
        "dfctl",
        "--url",
        &server.uri(),
        "groups",
        "shutdown",
        "--watch",
        "--output",
        "ndjson",
    ])
    .expect("parse");

    let mut stdout = Vec::new();
    run(cli, &mut stdout).await.expect("run");

    let output = String::from_utf8(stdout).expect("utf8");
    assert!(output.contains("\"resource\":\"group_shutdown\""));
    assert!(output.contains("\"allTerminal\":true"));
}

/// Scenario: command parsing enables or disables watch-compatible mutation
/// outputs.
/// Guarantees: only the documented combinations of `--watch` and output format
/// are accepted.
#[test]
fn mutation_output_validation_enforces_watch_contract() {
    assert!(validate_mutation_output_mode(MutationOutput::Human, true).is_ok());
    assert!(validate_mutation_output_mode(MutationOutput::Ndjson, true).is_ok());
    assert!(validate_mutation_output_mode(MutationOutput::Json, true).is_err());
    assert!(validate_mutation_output_mode(MutationOutput::AgentJson, true).is_err());
    assert!(validate_mutation_output_mode(MutationOutput::AgentJson, false).is_ok());
    assert!(validate_mutation_output_mode(MutationOutput::Ndjson, false).is_err());
}

/// Scenario: CLI timeout arguments contain fractional seconds or millisecond
/// values.
/// Guarantees: timeout serialization rounds up to at least one whole second for
/// the admin API wire contract.
#[test]
fn duration_rounds_up_to_whole_seconds() {
    assert_eq!(duration_to_admin_timeout_secs(Duration::from_millis(1)), 1);
    assert_eq!(duration_to_admin_timeout_secs(Duration::from_secs(2)), 2);
    assert_eq!(
        duration_to_admin_timeout_secs(Duration::from_millis(2500)),
        3
    );
}

/// Scenario: the pipeline config helpers load a YAML file from disk for a
/// pipeline mutation.
/// Guarantees: the shared loader reads and parses the file successfully so CLI
/// command runners can reuse the same behavior as the TUI editor flow.
#[test]
fn load_pipeline_config_reads_yaml_file() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("pipeline.yaml");
    fs::write(
        &file_path,
        serde_yaml::to_string(&pipeline_config()).expect("yaml"),
    )
    .expect("write");

    let loaded = load_pipeline_config(&file_path, "tenant-a", "ingest")
        .expect("pipeline config should load from disk");
    assert!(loaded.eq_ignoring_policies(&pipeline_config()));
}
