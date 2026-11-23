// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Account for a finite, admitted partial batch after graceful pipeline shutdown.

use super::{fixture_session::FixtureSession, graph};
use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_admin_api::{
    operations::OperationOptions,
    pipelines::{ProbeStatus, ShutdownOutcome},
    telemetry::{AttributeValue, MetricValue, MetricsOptions},
};
use otel_arrow_dfe_pdata::proto::opentelemetry::{
    collector::logs::v1::{ExportLogsServiceRequest, ExportLogsServiceResponse},
    common::v1::any_value,
};
use otel_arrow_dfe_validation::deployment::{Engine, wait_for as until};
use prost::Message;
use serde_json::json;
use std::{net::TcpListener, time::Duration};
use wiremock::MockServer;

async fn captured_ids(server: &MockServer) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for request in server
        .received_requests()
        .await
        .context("capture disabled")?
    {
        if request.url.path() != "/v1/logs" {
            continue;
        }
        let batch = ExportLogsServiceRequest::decode(&request.body[..])?;
        for record in batch
            .resource_logs
            .into_iter()
            .flat_map(|r| r.scope_logs)
            .flat_map(|s| s.log_records)
        {
            let Some(any_value::Value::StringValue(id)) = record.body.and_then(|b| b.value) else {
                anyhow::bail!("drained log has no string ID");
            };
            ids.push(id);
        }
    }
    Ok(ids)
}

/// Check graceful shutdown with a batch too small to flush on its own. Require
/// every admitted ID to reach the OTLP sink exactly once and the observer to keep serving.
pub async fn finite_id_drain(engine: &Engine) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(45), run(engine))
        .await
        .context("finite-ID drain timed out")?
}

async fn run(engine: &Engine) -> Result<()> {
    let sink = graph::services().await;
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let address = listener.local_addr()?;
    let session = FixtureSession::start(engine, None, &sink, |config, dir| {
        drop(listener);
        config["engine"]["telemetry"] = json!({"reporting_interval":"100ms"});
        config["policies"]["telemetry"] = json!({"runtime_metrics":"detailed"});
        config["groups"]["default"]["pipelines"]["drain"] = json!({
            "nodes":{
                "input":{"type":"receiver:otlp","config":{"protocols":{"http":{
                    "listening_addr":address.to_string(),"wait_for_result":false}}}},
                "batch":{"type":"processor:batch","config":{"format":"otap","max_batch_duration":"1h",
                    "otap":{"sizer":"items","min_size":64,"max_size":64}}},
                "buffer":{"type":"processor:durable_buffer","config":{
                    "path":dir.join("durable"),"retention_size_cap":"256 MiB"}},
                "out":{"type":"exporter:otlp_http","config":{"endpoint":sink.uri(),"client_pool_size":1,"http":{}}}
            },
            "connections":[{"from":"input","to":"batch"},{"from":"batch","to":"buffer"},{"from":"buffer","to":"out"}]
        });
    }).await?;
    until("drain pipeline ready", async || {
        Ok(session
            .engine
            .client()
            .pipelines()
            .readyz("default", "drain")
            .await?
            .status
            == ProbeStatus::Ok)
    })
    .await?;
    let mut admitted = Vec::new();
    for i in 0..7 {
        let id = format!("drain-{i}");
        let response = graph::post_otlp(address.port(), &id, "drain", false, None)
            .await?
            .error_for_status()?;
        let reply = ExportLogsServiceResponse::decode(response.bytes().await?)?;
        ensure!(
            reply
                .partial_success
                .is_none_or(|p| p.rejected_log_records == 0),
            "receiver rejected {id}"
        );
        admitted.push(id);
    }

    // A successful HTTP response acknowledges ingress. Also wait for the batch
    // processor to consume every request so the test actually drains buffered work.
    until(
        "batch processor to receive all seven admitted requests",
        async || {
            let metrics = session
                .engine
                .client()
                .telemetry()
                .metrics(&MetricsOptions::default())
                .await?;
            Ok(metrics.metric_sets.iter().any(|set| {
                let attr = |key: &str| match set.attributes.get(key) {
                    Some(AttributeValue::String(s)) => Some(s.as_str()),
                    _ => None,
                };
                set.name == "channel.receiver"
                    && attr("pipeline.id") == Some("drain")
                    && attr("node.id") == Some("batch")
                    && attr("channel.kind") == Some("pdata")
                    && set.metrics.iter().any(|m| {
                        m.metadata.name == "messages"
                            && m.value == MetricValue::U64(admitted.len() as u64)
                    })
            }))
        },
    )
    .await?;
    ensure!(
        captured_ids(&sink).await?.is_empty(),
        "partial batch escaped before shutdown"
    );
    let shutdown = session
        .engine
        .client()
        .pipelines()
        .shutdown(
            "default",
            "drain",
            &OperationOptions {
                wait: true,
                timeout_secs: 15,
            },
        )
        .await?;
    ensure!(
        matches!(shutdown, ShutdownOutcome::Completed(_)),
        "drain did not complete: {shutdown:?}"
    );
    verify_delivery(admitted, captured_ids(&sink).await?)?;
    session.observer("after-finite-id-drain").await
}

/// Compare IDs with multiplicity. Reordering is allowed, while a duplicate cannot stand in for a lost record.
fn verify_delivery(mut admitted: Vec<String>, mut delivered: Vec<String>) -> Result<()> {
    delivered.sort_unstable();
    admitted.sort_unstable();
    ensure!(
        delivered == admitted,
        "admitted {admitted:?}, delivered {delivered:?}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: the sink exports the same number of records, but duplicates one ID in place of the last.
    /// Guarantees: drain evidence detects the loss and allows healthy out-of-order delivery.
    #[test]
    fn equal_counts_do_not_hide_a_missing_id() {
        let ids = |pair: [&str; 2]| pair.map(str::to_owned).to_vec();
        let expected = ids(["first", "last"]);
        assert!(verify_delivery(expected.clone(), ids(["first", "first"])).is_err());
        verify_delivery(expected, ids(["last", "first"])).unwrap();
    }
}
