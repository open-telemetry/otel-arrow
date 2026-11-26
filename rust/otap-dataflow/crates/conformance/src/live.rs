// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Apply each component fixture to the target pipeline and observe its data-plane effect.
//! [`exercise`] drives the lifecycle. For each stage, [`candidate_pipeline`] builds
//! the desired graph, [`FixtureSession::rollout`] commits it, and [`observe`] checks the effect.
use super::{
    fixture_session::{FixtureSession, http_probe, output_path},
    fixtures::{Fixture, Listener, Probe, Service},
    pipeline_node::{self, ProbeContext},
};
use anyhow::{Context, Result, bail, ensure};
use otel_arrow_dfe_conformance_model::{Operation, OperationOutcome as Outcome, Rollout};
use otel_arrow_dfe_validation::deployment::{Engine, syslog::delivered, wait_for as until};
use rdkafka::{mocking::MockCluster, producer::DefaultProducerContext};
use serde_json::{Value, json};
use std::{collections::BTreeMap, time::Duration};
use tokio::{
    net::TcpStream,
    time::{Instant, timeout_at},
};
use wiremock::MockServer;

/// Run add, change and remove on one deployment. Completed stages keep their
/// outcomes if a later stage fails. The separate no-op result stays absent until
/// the changed configuration can be reapplied.
pub async fn exercise(engine: &Engine, fixture: &Fixture, role: &str) -> (Rollout, Option<bool>) {
    let mut rollout = Rollout::pending();
    let mut noop = None;
    rollout.error = run_stages(engine, fixture, role, &mut rollout.operations, &mut noop)
        .await
        .err()
        .map(|error| format!("{error:#}"));
    (rollout, noop)
}

/// Share one session and deadline across the ordered stages. A failure leaves later stages unexercised.
async fn run_stages(
    engine: &Engine,
    fixture: &Fixture,
    role: &str,
    results: &mut BTreeMap<String, Outcome>,
    noop: &mut Option<bool>,
) -> Result<()> {
    ensure!(
        role == fixture.probe.role(),
        "fixture probe incompatible with {role} registry"
    );
    let deadline = Instant::now() + Duration::from_secs(60);
    let (capture, mut session) = timeout_at(deadline, async {
        let capture = pipeline_node::services().await;
        let session = FixtureSession::start(engine, Some(fixture), &capture, |_, _| {}).await?;
        Ok::<_, anyhow::Error>((capture, session))
    })
    .await
    .context("fixture setup timed out")??;
    let cluster = if fixture.services.contains(&Service::Kafka) {
        let cluster = MockCluster::<DefaultProducerContext>::new(1)?;
        for topic in ["add", "change"] {
            cluster.create_topic(topic, 1, 1)?;
        }
        Some(cluster)
    } else {
        None
    };
    let brokers = cluster
        .as_ref()
        .map(MockCluster::bootstrap_servers)
        .unwrap_or_default();
    for operation in Operation::ALL {
        let name = operation.name();
        let result = timeout_at(
            deadline,
            step(&mut session, fixture, operation, &capture, &brokers, noop),
        )
        .await
        .with_context(|| format!("timed out at {name}"))
        .and_then(|r| r);
        if let Err(error) = result {
            let _ = results.insert(
                name.to_owned(),
                Outcome::Failed {
                    reason: format!("{error:#}"),
                },
            );
            return Err(error.context(name));
        }
        let _ = results.insert(name.to_owned(), Outcome::Passed);
    }
    Ok(())
}

/// Apply a candidate and check its effect on traffic. The change stage also requires
/// a different subject configuration or policy and a successful no-op on reapplication.
async fn step(
    session: &mut FixtureSession,
    fixture: &Fixture,
    operation: Operation,
    capture: &MockServer,
    brokers: &str,
    noop: &mut Option<bool>,
) -> Result<()> {
    let port = match operation {
        Operation::Add | Operation::Remove => session.add_listener.port,
        Operation::Change => session.change_listener.port,
    };
    let candidate = candidate_pipeline(session, fixture, operation, capture, brokers, port)?;
    if operation == Operation::Change {
        let before =
            serde_json::to_value(session.engine.details("default", "main").await?.pipeline)?;
        let section = if fixture.probe.role() == "extension" {
            "extensions"
        } else {
            "nodes"
        };
        let old = &before[section]["subject"];
        let new = &candidate[section]["subject"];
        ensure!(
            old["config"] != new["config"] || old["policies"] != new["policies"],
            "fixture must change the subject configuration or its node policy"
        );
    }
    match operation {
        Operation::Add => session.add_listener.release(),
        Operation::Change => session.change_listener.release(),
        Operation::Remove => (),
    }
    session.rollout(&candidate, operation.name()).await?;
    if operation == Operation::Change {
        *noop = Some(false);
        session.noop(&candidate).await?;
        *noop = Some(true);
    }

    observe(session, &fixture.probe, operation, capture, port).await
}

/// Build each candidate from the committed baseline so the previous stage cannot
/// leak nodes or bindings into the next. Removal keeps any peer needed to probe the unbound path.
fn candidate_pipeline(
    session: &FixtureSession,
    fixture: &Fixture,
    operation: Operation,
    capture: &MockServer,
    brokers: &str,
    port: u16,
) -> Result<Value> {
    let phase = operation.name();
    let present = operation != Operation::Remove;
    let mut candidate = session.original.clone();
    if present {
        let output_file = output_path(session.directory.path(), phase);
        let vars = minijinja::context!(
            phase => phase, address => format!("127.0.0.1:{port}"),
            output_file => output_file, dir => session.directory.path(),
            http_url => capture.uri(), brokers => brokers,
            https_url => session.tls.as_ref().map(|t| t.url.as_str()),
        );
        fixture
            .prepare
            .install(&session.directory.path().join(phase), &vars)?;
        let subject = json!({"type": fixture.urn, "config": fixture.configuration(operation == Operation::Change, &vars)?});
        match &fixture.probe {
            Probe::PipelineNode(node_probe) => {
                candidate["nodes"]["out"]["config"]["path"] = json!(output_file);
                candidate = node_probe.configure(
                    operation == Operation::Change,
                    &candidate,
                    subject,
                    &vars,
                )?;
            }
            Probe::OtlpAuthorization => {
                candidate["extensions"] = json!({"subject":subject});
            }
            Probe::SyslogTcp { connections, .. } | Probe::JsonlFile { connections, .. } => {
                if matches!(fixture.probe, Probe::JsonlFile { .. }) {
                    let _ = candidate["nodes"]
                        .as_object_mut()
                        .context("nodes")?
                        .remove("out");
                }
                candidate["nodes"]["subject"] = subject;
                candidate["connections"] = json!(connections);
            }
            Probe::OtlpHttpHeader(probe) => {
                candidate["extensions"] = json!({"subject":subject});
                candidate["nodes"]["out"] = probe.consumer(&capture.uri(), true)?;
            }
        }
    }
    if matches!(fixture.probe, Probe::OtlpAuthorization) {
        candidate["nodes"]["feed"] = pipeline_node::otlp_feed(&format!("127.0.0.1:{port}"));
        if present {
            candidate["nodes"]["feed"]["capabilities"] =
                json!({"bearer_token_authorizer":"subject"});
        }
        candidate["connections"]
            .as_array_mut()
            .context("connections")?
            .push(json!({"from":"feed","to":"out"}));
    }

    Ok(candidate)
}

/// Check the effect specified by the probe after rollout, using a fresh marker for
/// each stage so an earlier delivery cannot satisfy a marker check.
async fn observe(
    session: &FixtureSession,
    probe: &Probe,
    operation: Operation,
    capture: &MockServer,
    port: u16,
) -> Result<()> {
    let phase = operation.name();
    let present = operation != Operation::Remove;
    let marker = format!("fixture-{phase}-record");
    match probe {
        Probe::OtlpAuthorization => {
            if present {
                let refused =
                    pipeline_node::post_otlp(port, "denied", phase, false, Some("wrong-token"))
                        .await?;
                ensure!(
                    matches!(refused.status().as_u16(), 401 | 403),
                    "unauthorized token received {}",
                    refused.status()
                );
            }
            let token = format!("token-{phase}");
            let _ = pipeline_node::post_otlp(
                port,
                &marker,
                phase,
                false,
                present.then_some(token.as_str()),
            )
            .await?
            .error_for_status()?;
            until(&marker, async || {
                delivered(session.directory.path(), "main", &marker, None)
            })
            .await
        }
        Probe::PipelineNode(node_probe) => {
            if !present {
                return session
                    .probe(
                        session.input,
                        "main",
                        &marker,
                        node_probe.removed_attribute(),
                    )
                    .await;
            }
            node_probe
                .check(&ProbeContext {
                    dir: session.directory.path(),
                    operation,
                    marker: &marker,
                    input: session.input,
                    port,
                    server: capture,
                })
                .await
        }
        Probe::SyslogTcp { expect, .. } => {
            let port_for = |listener: Listener| match listener {
                Listener::Baseline => session.input,
                Listener::Add => session.add_listener.port,
                Listener::Change => session.change_listener.port,
            };
            let check = expect.at(operation);
            if let Some(retired) = check.closed {
                closed(port_for(retired)).await?;
            }
            session
                .probe(port_for(check.delivers_on), "main", &marker, None)
                .await
        }
        Probe::JsonlFile { expect, .. } => {
            let check = expect.at(operation);
            session
                .probe(session.input, &check.delivers_to, &marker, None)
                .await?;
            for old in &check.absent_from {
                ensure!(
                    !delivered(session.directory.path(), old, &marker, None)?,
                    "{old} still received {marker}"
                );
            }
            Ok(())
        }
        Probe::OtlpHttpHeader(probe) => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let vars = minijinja::context!(phase => phase, basic => STANDARD.encode(format!("ops:{phase}")));
            let value = probe
                .expect
                .at(operation)
                .value()
                .map(|v| super::fixtures::render_text(v, &vars))
                .transpose()?;
            http_probe(
                capture,
                session.input,
                &marker,
                Some((&probe.header, value.as_deref())),
            )
            .await
        }
    }
}

async fn closed(port: u16) -> Result<()> {
    match TcpStream::connect(("127.0.0.1", port)).await {
        Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => Ok(()),
        _ => bail!("retired listener {port} still accepts connections or could not be verified"),
    }
}
