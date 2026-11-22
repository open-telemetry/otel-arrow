// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

mod drain;
pub use drain::finite_id_drain;

use super::fixtures::{Fixture, Probe, Service};
use super::graph::{self, ProbeContext};
use anyhow::{Context, Result, bail, ensure};
use otel_arrow_dfe_admin_api::{
    Error,
    operations::OperationOptions,
    pipelines::{PipelineRolloutState, ReconfigureOutcome},
};
use otel_arrow_dfe_contract_model::OperationOutcome as Outcome;
use otel_arrow_dfe_validation::deployment::{
    Engine, EngineSession,
    syslog::{self, delivered, send},
    wait_for as until,
};
use rdkafka::mocking::MockCluster;
use rdkafka::producer::DefaultProducerContext;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::{fs, net::TcpListener, path::Path, time::Duration};
use tokio::{
    net::TcpStream,
    time::{Instant, timeout_at},
};
use wiremock::MockServer;

fn syslog(port: u16) -> Value {
    json!({"type":"receiver:syslog_cef", "config":{"batch":{"max_size":1},
        "protocol":{"tcp":{"listening_addr":format!("127.0.0.1:{port}")}}}})
}

fn file_sink(dir: &Path, prefix: &str) -> Value {
    json!({"type":"exporter:file","config":{"path":dir.join(format!("{prefix}-{{signal}}-{{core_id}}-{{generation}}.jsonl"))}})
}

fn pipeline(port: u16, dir: &Path, name: &str) -> Value {
    json!({"nodes":{"input":syslog(port),"out":file_sink(dir,name)},
        "connections":[{"from":"input","to":"out"}]})
}

struct FixtureSession {
    engine: EngineSession,
    directory: tempfile::TempDir,
    input: u16,
    observer: u16,
    generation: u64,
    original: Value,
    ports: [u16; 2],
    reserved: [Option<TcpListener>; 2],
    tls: Option<super::tls::TlsCapture>,
}

impl FixtureSession {
    async fn probe(
        &self,
        port: u16,
        prefix: &str,
        marker: &str,
        attribute: Option<(&str, Option<&str>)>,
    ) -> Result<()> {
        syslog::assert_delivery(port, self.directory.path(), prefix, marker, attribute).await
    }

    async fn observer(&self, marker: &str) -> Result<()> {
        self.probe(self.observer, "observer", marker, None).await?;
        ensure!(
            self.engine
                .details("default", "observer")
                .await?
                .active_generation
                == Some(self.generation),
            "observer changed generation at {marker}"
        );
        Ok(())
    }

    async fn rollout(&self, pipeline: &Value, step: &str) -> Result<()> {
        let request = serde_json::from_value(json!({"pipeline":pipeline}))?;
        self.engine
            .rollout("default", "main", &request, async || {
                self.observer(&format!("during-{step}")).await
            })
            .await
            .with_context(|| format!("{step} rollout"))?;
        self.observer(&format!("after-{step}")).await
    }

    async fn noop(&self, pipeline: &Value) -> Result<()> {
        let request = serde_json::from_value(json!({"pipeline":pipeline}))?;
        self.engine.assert_noop("default", "main", &request).await
    }

    async fn rejection(&self, committed_prefix: &str) -> Result<()> {
        let before = self.engine.details("default", "main").await?;
        let mut invalid = serde_json::to_value(&before.pipeline)?;
        invalid["connections"]
            .as_array_mut()
            .context("connections")?
            .push(json!({"from":"missing","to":"out"}));
        let request = serde_json::from_value(json!({"pipeline": invalid}))?;
        let result = self
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
        let config_url = format!("{}/api/v1/config", self.engine.admin_url());
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
        observability["engine"]["observability"]["pipeline"] = json!({
            "nodes":{
                "telemetry":{"type":"receiver:internal_telemetry", "config":{"signals":["metrics"],"metrics":{"interval":"1s"}}},
                "out":file_sink(self.directory.path(), "observability")
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
        let after = self.engine.details("default", "main").await?;
        ensure!(
            before.active_generation == after.active_generation
                && before.pipeline == after.pipeline,
            "rejection changed the active pipeline"
        );
        self.probe(self.input, committed_prefix, "after-rejection", None)
            .await?;
        self.observer("observer-after-rejection").await
    }
}

async fn closed(port: u16) -> Result<()> {
    match TcpStream::connect(("127.0.0.1", port)).await {
        Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => Ok(()),
        _ => bail!("retired listener {port} still accepts connections or could not be verified"),
    }
}

async fn start(
    engine: &Engine,
    fixture: Option<&Fixture>,
    services: &MockServer,
    configure: impl FnOnce(&mut Value, &Path),
) -> Result<FixtureSession> {
    let temp = tempfile::tempdir()?;
    let capture = fixture
        .filter(|f| matches!(f.probe, Probe::OtlpHttpHeader { .. }))
        .map(|_| services);
    let mut env = Vec::new();
    let tls = if fixture.is_some_and(|f| f.services.contains(&Service::Geneva)) {
        let tls = super::tls::TlsCapture::start(temp.path(), *services.address()).await?;
        graph::geneva_services(services, &tls.url).await;
        env.push((
            "SSL_CERT_FILE",
            temp.path().join("fixture-ca.pem").display().to_string(),
        ));
        Some(tls)
    } else {
        None
    };
    if fixture.is_some_and(|f| f.services.contains(&Service::AzureIdentity)) {
        env.push(("IDENTITY_ENDPOINT", format!("{}/identity", services.uri())));
        env.push(("IDENTITY_HEADER", "fixture".into()));
    }
    if fixture.is_some_and(|f| matches!(f.probe, Probe::OtlpAuthorization)) {
        let path = temp.path().join("kubeconfig");
        fs::write(
            &path,
            serde_json::to_vec(&json!({
                "apiVersion":"v1","kind":"Config","current-context":"fixture",
                "contexts":[{"name":"fixture","context":{"cluster":"fixture","user":"fixture"}}],
                "clusters":[{"name":"fixture","cluster":{"server":services.uri()}}],
                "users":[{"name":"fixture","user":{}}]
            }))?,
        )?;
        env.push(("KUBECONFIG", path.display().to_string()));
    }
    // Reserve the complete set together to ensure all five loopback ports are distinct.
    let mut sockets = (0..5)
        .map(|_| TcpListener::bind(("127.0.0.1", 0)))
        .collect::<std::io::Result<Vec<_>>>()?;
    let ports = sockets
        .iter()
        .map(|s| Ok(s.local_addr()?.port()))
        .collect::<std::io::Result<Vec<_>>>()?;
    let [admin, input, observer, extra, changed] = ports[..] else {
        unreachable!()
    };
    let mut main = pipeline(input, temp.path(), "main");
    if let Some(server) = capture {
        main["nodes"]["out"] = json!({"type":"exporter:otlp_http","config":{"endpoint":server.uri(),
            "client_pool_size":1,"http":{}}});
    }
    let mut config = json!({"version":"otel_dataflow/v1","engine":{"http_admin":{"bind_address":format!("127.0.0.1:{admin}")}},
        "policies":{"resources":{"core_allocation":{"type":"core_count","count":1}}},
        "groups":{"default":{"pipelines":{"main":main,"observer":pipeline(observer,temp.path(),"observer")}}}});
    if fixture.is_some_and(|f| f.services.contains(&Service::Topics)) {
        config["groups"]["default"]["topics"] = json!({"add":{"impl_selection":"force_mixed"},"change":{"impl_selection":"force_mixed"}});
    }
    configure(&mut config, temp.path());
    let path = temp.path().join("engine.json");
    fs::write(&path, serde_json::to_vec(&config)?)?;
    let new_address = sockets.pop();
    let first_address = sockets.pop();
    drop(sockets);
    let engine = EngineSession::start(
        engine,
        &path,
        &env,
        format!("http://127.0.0.1:{admin}"),
        &[("default", "main", 1), ("default", "observer", 1)],
    )
    .await?;
    let generation = engine
        .details("default", "observer")
        .await?
        .active_generation
        .context("missing observer generation")?;
    let original = serde_json::to_value(engine.details("default", "main").await?.pipeline)?;
    let session = FixtureSession {
        engine,
        directory: temp,
        input,
        observer,
        generation,
        original,
        ports: [extra, changed],
        reserved: [first_address, new_address],
        tls,
    };
    if let Some(server) = capture {
        http_probe(server, input, "initial-main", None).await?;
    } else {
        session.probe(input, "main", "initial-main", None).await?;
    }
    session.observer("initial-observer").await?;
    Ok(session)
}

async fn http_probe(
    server: &MockServer,
    input: u16,
    marker: &str,
    header: Option<(&str, Option<&str>)>,
) -> Result<()> {
    send(input, marker).await?;
    until(marker, async || {
        for req in server
            .received_requests()
            .await
            .context("capture disabled")?
        {
            if req.method == "POST"
                && req.url.path() == "/v1/logs"
                && req
                    .body
                    .windows(marker.len())
                    .any(|bytes| bytes == marker.as_bytes())
            {
                if let Some((key, expected)) = header {
                    ensure!(
                        req.headers.get(key).map(|v| v.as_bytes()) == expected.map(str::as_bytes),
                        "unexpected {key} on {marker}"
                    );
                }
                return Ok(true);
            }
        }
        Ok(false)
    })
    .await
}

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Change,
    Remove,
}

impl Operation {
    const ALL: [Self; 3] = [Self::Add, Self::Change, Self::Remove];

    fn name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Change => "change",
            Self::Remove => "remove",
        }
    }
}

pub type Outcomes = BTreeMap<String, Outcome>;

impl FixtureSession {
    async fn step(
        &mut self,
        fixture: &Fixture,
        operation: Operation,
        capture: &MockServer,
        brokers: &str,
        noop: &mut Option<bool>,
    ) -> Result<()> {
        let phase = operation.name();
        let present = !matches!(operation, Operation::Remove);
        let port = self.ports[usize::from(matches!(operation, Operation::Change))];
        let marker = format!("fixture-{phase}-record");
        let mut candidate = self.original.clone();
        if present {
            let key_file = self.directory.path().join(format!("{phase}.key"));
            if matches!(fixture.probe, Probe::OtlpHttpHeader { .. }) {
                fs::write(&key_file, phase)?;
            }
            let output_file = self.directory.path().join(format!(
                "{phase}-{{signal}}-{{core_id}}-{{generation}}.jsonl"
            ));
            let vars = minijinja::context!(
                phase => phase, address => format!("127.0.0.1:{port}"),
                output_file => output_file, key_file => key_file, dir => self.directory.path(),
                http_url => capture.uri(), brokers => brokers,
                https_url => self.tls.as_ref().map(|t| t.url.as_str()),
            );
            let subject = json!({"type": fixture.urn, "config": fixture.configuration(matches!(operation, Operation::Change), &vars)?});
            match &fixture.probe {
                Probe::Pipeline(graph) => {
                    candidate["nodes"]["out"] = file_sink(self.directory.path(), phase);
                    graph.prepare(&self.directory.path().join(phase))?;
                    candidate = graph.configure(
                        matches!(operation, Operation::Change),
                        &candidate,
                        subject,
                        &vars,
                    )?;
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
        if matches!(operation, Operation::Change) {
            let before =
                serde_json::to_value(self.engine.details("default", "main").await?.pipeline)?;
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
        if present {
            // Reserve future generation listeners until the candidate can bind them.
            drop(self.reserved[usize::from(matches!(operation, Operation::Change))].take());
        }
        self.rollout(&candidate, phase).await?;
        if matches!(operation, Operation::Change) {
            *noop = Some(false);
            self.noop(&candidate).await?;
            *noop = Some(true);
        }
        match &fixture.probe {
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
                    delivered(self.directory.path(), "main", &marker, None)
                })
                .await
            }
            Probe::Pipeline(graph) => {
                if !present {
                    return self.probe(self.input, "main", &marker, None).await;
                }
                graph
                    .check(&ProbeContext {
                        dir: self.directory.path(),
                        phase,
                        marker: &marker,
                        input: self.input,
                        port,
                        server: capture,
                    })
                    .await
            }
            Probe::SyslogTcp => {
                match operation {
                    Operation::Add => (),
                    Operation::Change => closed(self.ports[0]).await?,
                    Operation::Remove => closed(self.ports[1]).await?,
                }
                self.probe(
                    if present { port } else { self.input },
                    "main",
                    &marker,
                    None,
                )
                .await
            }
            Probe::LogAttribute { key } => {
                self.probe(
                    self.input,
                    "main",
                    &marker,
                    Some((key, present.then_some(phase))),
                )
                .await
            }
            Probe::JsonlFile => {
                let active = if present { phase } else { "main" };
                self.probe(self.input, active, &marker, None).await?;
                for old in ["main", "add", "change"] {
                    if old != active {
                        ensure!(
                            !delivered(self.directory.path(), old, &marker, None)?,
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
                    self.input,
                    &marker,
                    Some((header, present.then_some(value))),
                )
                .await
            }
        }
    }
}

pub async fn exercise(
    engine: &Engine,
    fixture: &Fixture,
    results: &mut Outcomes,
    noop: &mut Option<bool>,
) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(60);
    let (capture, mut session) = timeout_at(deadline, async {
        let capture = graph::services().await;
        let session = start(engine, Some(fixture), &capture, |_, _| {}).await?;
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
            session.step(fixture, operation, &capture, &brokers, noop),
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

pub fn pending() -> Outcomes {
    Operation::ALL
        .into_iter()
        .map(|op| (op.name().to_owned(), Outcome::Unexercised))
        .collect()
}

// Keep the rejected-update regression independently observable if another fixture fails.
pub async fn rejected_update(engine: &Engine) -> Result<()> {
    let server = graph::services().await;
    let session = start(engine, None, &server, |_, _| {}).await?;
    let mut committed = session.original.clone();
    committed["nodes"]["out"] = file_sink(session.directory.path(), "committed");
    session.rollout(&committed, "before-rejection").await?;
    session
        .probe(session.input, "committed", "before-rejection", None)
        .await?;
    session.rejection("committed").await
}

// A valid candidate fails at runtime because its new listener is already owned.
pub async fn reconfiguration_consistency(engine: &Engine) -> Result<()> {
    let server = graph::services().await;
    let session = start(engine, None, &server, |_, _| {}).await?;
    session.noop(&session.original).await?;
    session
        .probe(session.input, "main", "after-noop", None)
        .await?;
    let before = session.engine.details("default", "main").await?;
    let mut candidate = session.original.clone();
    // Session owns this bound listener until after the rollout reaches a terminal state.
    candidate["nodes"]["input"] = syslog(session.ports[0]);
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

pub async fn embedded_ui(
    engine: &Engine,
    rows: &mut Vec<otel_arrow_dfe_contract_model::UiSignal>,
) -> Result<()> {
    let server = graph::services().await;
    let session = start(engine, None, &server, |config, _| {
        super::dashboard::configure(config)
    })
    .await?;
    super::dashboard::check(
        session.engine.admin_url(),
        session.ports[0],
        session.probe(session.input, "main", "dashboard-traffic", None),
        rows,
    )
    .await?;
    drop(session);
    let without_tap = start(engine, None, &server, |config, _| {
        config["engine"]["telemetry"] = json!({"logs":{"tap":{"enabled":false}}});
    })
    .await?;
    super::dashboard::check_disabled_tap(without_tap.engine.admin_url()).await
}
