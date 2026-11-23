// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Apply each component fixture to the target pipeline and observe its data-plane effect.
use super::{
    fixture_session::{FixtureSession, file_sink, http_probe},
    fixtures::{Fixture, Probe, Service},
    graph::{self, ProbeContext},
};
use anyhow::{Context, Result, bail, ensure};
use otel_arrow_dfe_contract_model::{Operation, OperationOutcome as Outcome, Rollout};
use otel_arrow_dfe_validation::deployment::{Engine, syslog::delivered, wait_for as until};
use rdkafka::{mocking::MockCluster, producer::DefaultProducerContext};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, time::Duration};
use tokio::{
    net::TcpStream,
    time::{Instant, timeout_at},
};
use wiremock::MockServer;

/// Keep per-stage evidence even if a later assertion or setup fails.
pub async fn exercise(engine: &Engine, fixture: &Fixture, role: &str) -> (Rollout, Option<bool>) {
    let mut rollout = Rollout::pending();
    let mut noop = None;
    rollout.error = run_stages(engine, fixture, role, &mut rollout.operations, &mut noop)
        .await
        .err()
        .map(|error| format!("{error:#}"));
    (rollout, noop)
}

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
        let capture = graph::services().await;
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
        let key_file = session.directory.path().join(format!("{phase}.key"));
        if matches!(fixture.probe, Probe::OtlpHttpHeader { .. }) {
            fs::write(&key_file, phase)?;
        }
        let output_file = session.directory.path().join(format!(
            "{phase}-{{signal}}-{{core_id}}-{{generation}}.jsonl"
        ));
        let vars = minijinja::context!(
            phase => phase, address => format!("127.0.0.1:{port}"),
            output_file => output_file, key_file => key_file, dir => session.directory.path(),
            http_url => capture.uri(), brokers => brokers,
            https_url => session.tls.as_ref().map(|t| t.url.as_str()),
        );
        let subject = json!({"type": fixture.urn, "config": fixture.configuration(operation == Operation::Change, &vars)?});
        match &fixture.probe {
            Probe::Pipeline(graph) => {
                candidate["nodes"]["out"] = file_sink(session.directory.path(), phase);
                graph.prepare(&session.directory.path().join(phase))?;
                candidate =
                    graph.configure(operation == Operation::Change, &candidate, subject, &vars)?;
            }
            Probe::OtlpAuthorization => {
                candidate["extensions"] = json!({"subject":subject});
            }
            Probe::SyslogTcp => {
                candidate["nodes"]["subject"] = subject;
                candidate["connections"]
                    .as_array_mut()
                    .context("connections")?
                    .push(json!({"from":"subject","to":"out"}));
            }
            Probe::LogAttribute { .. } => {
                candidate["nodes"]["subject"] = subject;
                candidate["connections"] =
                    json!([{"from":"input","to":"subject"},{"from":"subject","to":"out"}]);
            }
            Probe::JsonlFile => {
                let _ = candidate["nodes"]
                    .as_object_mut()
                    .context("nodes")?
                    .remove("out");
                candidate["nodes"]["subject"] = subject;
                candidate["connections"] = json!([{"from":"input","to":"subject"}]);
            }
            Probe::OtlpHttpHeader { capability, .. } => {
                candidate["extensions"] = json!({"subject":subject});
                candidate["nodes"]["out"]["capabilities"] = json!({(capability): "subject"});
            }
        }
    }
    if matches!(fixture.probe, Probe::OtlpAuthorization) {
        candidate["nodes"]["feed"] = graph::otlp_feed(&format!("127.0.0.1:{port}"));
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
                    graph::post_otlp(port, "denied", phase, false, Some("wrong-token")).await?;
                ensure!(
                    matches!(refused.status().as_u16(), 401 | 403),
                    "unauthorized token received {}",
                    refused.status()
                );
            }
            let token = format!("token-{phase}");
            let _ = graph::post_otlp(
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
        Probe::Pipeline(graph) => {
            if !present {
                return session.probe(session.input, "main", &marker, None).await;
            }
            graph
                .check(&ProbeContext {
                    dir: session.directory.path(),
                    phase,
                    marker: &marker,
                    input: session.input,
                    port,
                    server: capture,
                })
                .await
        }
        Probe::SyslogTcp => {
            match operation {
                Operation::Add => (),
                Operation::Change => closed(session.add_listener.port).await?,
                Operation::Remove => closed(session.change_listener.port).await?,
            }
            session
                .probe(
                    if present { port } else { session.input },
                    "main",
                    &marker,
                    None,
                )
                .await
        }
        Probe::LogAttribute { key } => {
            session
                .probe(
                    session.input,
                    "main",
                    &marker,
                    Some((key, present.then_some(phase))),
                )
                .await
        }
        Probe::JsonlFile => {
            let active = if present { phase } else { "main" };
            session.probe(session.input, active, &marker, None).await?;
            for old in ["main", "add", "change"] {
                if old != active {
                    ensure!(
                        !delivered(session.directory.path(), old, &marker, None)?,
                        "{old} still received {marker}"
                    );
                }
            }
            Ok(())
        }
        Probe::OtlpHttpHeader {
            header, expected, ..
        } => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let vars = minijinja::context!(phase => phase, basic => STANDARD.encode(format!("ops:{phase}")));
            let value = super::fixtures::rendered(
                &json!(expected.as_deref().unwrap_or("{{ phase }}")),
                vars,
            )?;
            let value = value.as_str().context("header value must be text")?;
            http_probe(
                capture,
                session.input,
                &marker,
                Some((header, present.then_some(value))),
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
