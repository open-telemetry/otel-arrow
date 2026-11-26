// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine-wide checks for rejected updates, no-ops and failed replacements.
use super::{
    fixture_session::{FixtureSession, output_path},
    pipeline_node,
};
use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_admin_api::{
    Error,
    operations::OperationOptions,
    pipelines::{PipelineRolloutState, ReconfigureOutcome},
};
use otel_arrow_dfe_validation::deployment::Engine;
use serde_json::{Value, json};

/// Commit a new output, submit invalid updates, and verify that traffic still reaches
/// the committed output with the same serving generation.
pub async fn rejected_update(engine: &Engine) -> Result<()> {
    let server = pipeline_node::services().await;
    let session = FixtureSession::start(engine, None, &server, |_, _| {}).await?;
    let mut committed = session.original.clone();
    committed["nodes"]["out"]["config"]["path"] =
        json!(output_path(session.directory.path(), "committed"));
    session.rollout(&committed, "before-rejection").await?;
    session
        .probe(session.input, "committed", "before-rejection", None)
        .await?;
    assert_rejection(&session, "committed").await
}

/// Reapply the active graph as a no-op, then force a replacement to fail at startup
/// on an occupied port. Check the retained graph, generation and delivery on both pipelines.
pub async fn reconfiguration_consistency(engine: &Engine) -> Result<()> {
    let server = pipeline_node::services().await;
    let session = FixtureSession::start(engine, None, &server, |_, _| {}).await?;
    session.noop(&session.original).await?;
    session
        .probe(session.input, "main", "after-noop", None)
        .await?;
    let before = session.engine.details("default", "main").await?;
    let mut candidate = session.original.clone();
    // Session owns this bound listener until after the rollout reaches a terminal state.
    candidate["nodes"]["input"]["config"]["protocol"]["tcp"]["listening_addr"] =
        json!(format!("127.0.0.1:{}", session.add_listener.port));
    let request = serde_json::from_value(json!({
        "pipeline": candidate, "stepTimeoutSecs": 3, "drainTimeoutSecs": 3
    }))?;
    let result = session
        .engine
        .client()
        .pipelines()
        .reconfigure(
            "default",
            "main",
            &request,
            &OperationOptions {
                wait: true,
                timeout_secs: 15,
            },
        )
        .await?;
    ensure!(
        matches!(&result, ReconfigureOutcome::Failed(status)
        if status.state == PipelineRolloutState::Failed && status.action == "replace"),
        "expected the accepted replacement to fail and roll back: {result:?}"
    );
    let after = session.engine.details("default", "main").await?;
    ensure!(
        after.pipeline == before.pipeline && after.active_generation == before.active_generation,
        "candidate startup failure changed the committed pipeline or generation"
    );
    session
        .probe(session.input, "main", "after-candidate-failure", None)
        .await?;
    session.observer("observer-after-candidate-failure").await
}

/// Reject an invalid edge via the pipeline API and startup-only changes via full-config reconciliation.
async fn assert_rejection(session: &FixtureSession, committed_prefix: &str) -> Result<()> {
    let before = session.engine.details("default", "main").await?;
    let mut invalid = serde_json::to_value(&before.pipeline)?;
    invalid["connections"]
        .as_array_mut()
        .context("connections")?
        .push(json!({"from":"missing","to":"out"}));
    let request = serde_json::from_value(json!({"pipeline": invalid}))?;
    let result = session
        .engine
        .client()
        .pipelines()
        .reconfigure("default", "main", &request, &OperationOptions::default())
        .await;
    ensure!(
        matches!(result, Err(Error::AdminOperation { status: 422, .. })),
        "unexpected invalid topology result {result:?}"
    );
    // The SDK currently exposes pipeline rollouts, but no full-config reconciliation method.
    let http = reqwest::Client::new();
    let config_url = format!("{}/api/v1/config", session.engine.admin_url());
    let committed: Value = http
        .get(&config_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let mut interval = committed.clone();
    interval["engine"]["telemetry"]["reporting_interval"] = json!("47s");
    let mut observability = committed.clone();
    let mut out = session.original["nodes"]["out"].clone();
    out["config"]["path"] = json!(output_path(session.directory.path(), "observability"));
    observability["engine"]["observability"]["pipeline"] = json!({
        "nodes":{
            "telemetry":{"type":"receiver:internal_telemetry", "config":{"signals":["metrics"],"metrics":{"interval":"1s"}}},
            "out":out
        },
        "connections":[{"from":"telemetry","to":"out"}]
    });
    for (setting, desired) in [
        ("reporting interval", interval),
        ("observability pipeline", observability),
    ] {
        let response = http
            .post(format!("{config_url}/reconcile"))
            .json(&json!({"config":desired}))
            .send()
            .await?;
        let status = response.status();
        let diagnostic = response.json::<Value>().await?;
        ensure!(
            status == 422 && diagnostic["kind"] == "unsupported_mutation",
            "expected a startup-owned mutation rejection for {setting}, received {status}: {diagnostic}"
        );
    }
    let retained: Value = http
        .get(&config_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    ensure!(
        retained == committed,
        "rejection changed committed configuration"
    );
    let after = session.engine.details("default", "main").await?;
    ensure!(
        before.active_generation == after.active_generation && before.pipeline == after.pipeline,
        "rejection changed the active pipeline"
    );
    session
        .probe(session.input, committed_prefix, "after-rejection", None)
        .await?;
    session.observer("observer-after-rejection").await
}
