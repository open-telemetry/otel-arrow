// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::engine::{Engine, Running};
use super::fixtures::{Fixture, Probe};
use anyhow::{Context, Result, bail, ensure};
use otel_arrow_dfe_admin_api::{
    AdminClient, AdminEndpoint, Error, HttpAdminClientSettings,
    operations::OperationOptions,
    pipelines::{PipelineDetails, PipelineRolloutState, ReconfigureOutcome},
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{Instant, sleep, timeout, timeout_at},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

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

async fn send(port: u16, marker: &str) -> Result<()> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
    stream
        .write_all(format!("<14>1 2025-01-01T00:00:00Z host app - - - {marker}\n").as_bytes())
        .await?;
    stream.shutdown().await?;
    Ok(())
}

fn delivered(
    dir: &Path,
    prefix: &str,
    marker: &str,
    attribute: Option<(&str, Option<&str>)>,
) -> Result<bool> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry
            .file_name()
            .to_string_lossy()
            .starts_with(&format!("{prefix}-logs-"))
        {
            continue;
        }
        let data = fs::read(entry.path())?;
        for line in data
            .split_inclusive(|&b| b == b'\n')
            .filter(|line| line.ends_with(b"\n"))
        {
            let payload: Value = serde_json::from_slice(line)?;
            for resource in payload["resourceLogs"]
                .as_array()
                .context("missing resourceLogs")?
            {
                for scope in resource["scopeLogs"]
                    .as_array()
                    .context("missing scopeLogs")?
                {
                    for record in scope["logRecords"]
                        .as_array()
                        .context("missing logRecords")?
                    {
                        let attrs = record["attributes"]
                            .as_array()
                            .context("missing attributes")?;
                        let attr = |key: &str| {
                            attrs
                                .iter()
                                .find(|a| a["key"] == key)
                                .and_then(|a| a["value"]["stringValue"].as_str())
                        };
                        if attr("syslog.message") == Some(marker) {
                            if let Some((key, expected)) = attribute {
                                let actual = attrs.iter().find(|a| a["key"] == key);
                                ensure!(
                                    actual.map(|a| &a["value"])
                                        == expected.map(|s| json!({"stringValue":s})).as_ref(),
                                    "{marker}: unexpected {key}: {actual:?}"
                                );
                            }
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

async fn until(label: &str, mut probe: impl AsyncFnMut() -> Result<bool>) -> Result<()> {
    timeout(Duration::from_secs(12), async {
        loop {
            if probe().await? {
                return Ok(());
            }
            sleep(Duration::from_millis(30)).await;
        }
    })
    .await
    .with_context(|| format!("timed out waiting for {label}"))?
}

struct Harness {
    client: AdminClient,
    root: String,
    directory: PathBuf,
    input: u16,
    observer: u16,
    generation: u64,
}

impl Harness {
    async fn details(&self, id: &str) -> Result<PipelineDetails> {
        self.client
            .pipelines()
            .details("default", id)
            .await?
            .context("pipeline missing")
    }

    async fn probe(
        &self,
        port: u16,
        prefix: &str,
        marker: &str,
        attribute: Option<(&str, Option<&str>)>,
    ) -> Result<()> {
        send(port, marker).await?;
        until(marker, async || {
            delivered(&self.directory, prefix, marker, attribute)
        })
        .await
    }

    async fn observer(&self, marker: &str) -> Result<()> {
        self.probe(self.observer, "observer", marker, None).await?;
        ensure!(
            self.details("observer").await?.active_generation == Some(self.generation),
            "observer changed generation at {marker}"
        );
        Ok(())
    }

    async fn rollout(&self, pipeline: &Value, step: &str) -> Result<()> {
        let request = serde_json::from_value(json!({"pipeline":pipeline}))?;
        let outcome = self
            .client
            .pipelines()
            .reconfigure("default", "main", &request, &OperationOptions::default())
            .await
            .with_context(|| format!("{step} rollout"))?;
        self.observer(&format!("during-{step}")).await?;
        let status = match outcome {
            ReconfigureOutcome::Accepted(status) | ReconfigureOutcome::Completed(status) => status,
            other => bail!("{step} rollout: {other:?}"),
        };
        until(step, async || {
            let current = self
                .client
                .pipelines()
                .rollout_status("default", "main", &status.rollout_id)
                .await?
                .context("missing rollout")?;
            match current.state {
                PipelineRolloutState::Succeeded => Ok(true),
                PipelineRolloutState::Failed | PipelineRolloutState::RollbackFailed => {
                    bail!("{step} rollout failed: {:?}", current.failure_reason)
                }
                _ => Ok(false),
            }
        })
        .await?;
        ensure!(
            self.details("main").await?.pipeline == request.pipeline,
            "{step}: committed pipeline differs from requested topology"
        );
        self.observer(&format!("after-{step}")).await
    }

    async fn rejection(&self, committed_prefix: &str) -> Result<()> {
        let before = self.details("main").await?;
        let mut invalid = serde_json::to_value(&before.pipeline)?;
        invalid["connections"]
            .as_array_mut()
            .context("connections")?
            .push(json!({"from":"missing","to":"out"}));
        let request = serde_json::from_value(json!({"pipeline": invalid}))?;
        let result = self
            .client
            .pipelines()
            .reconfigure("default", "main", &request, &OperationOptions::default())
            .await;
        ensure!(
            matches!(result, Err(Error::AdminOperation { status: 422, .. })),
            "unexpected invalid topology result {result:?}"
        );
        // The SDK currently exposes pipeline rollouts, but no full-config reconciliation method.
        let http = reqwest::Client::new();
        let config_url = format!("{}/api/v1/config", self.root);
        let committed: Value = http
            .get(&config_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let mut desired = committed.clone();
        desired["engine"]["telemetry"]["reporting_interval"] = json!("47s");
        let rejection = http
            .post(format!("{config_url}/reconcile"))
            .json(&json!({"config":desired}))
            .send()
            .await?;
        ensure!(
            rejection.status() == 422
                && rejection.json::<Value>().await?["kind"] == "unsupported_mutation",
            "unexpected startup-owned mutation result"
        );
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
        let after = self.details("main").await?;
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

struct Session {
    harness: Harness,
    original: Value,
    ports: [u16; 2],
    _process: Running,
    _directory: tempfile::TempDir,
}

async fn start(engine: &Engine, capture: Option<&MockServer>) -> Result<Session> {
    let temp = tempfile::tempdir()?;
    // Reserve the complete set together to ensure all five loopback ports are distinct.
    let sockets = (0..5)
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
    let config = json!({"version":"otel_dataflow/v1","engine":{"http_admin":{"bind_address":format!("127.0.0.1:{admin}")}},
        "policies":{"resources":{"core_allocation":{"type":"core_count","count":1}}},
        "groups":{"default":{"pipelines":{"main":main,"observer":pipeline(observer,temp.path(),"observer")}}}});
    let path = temp.path().join("engine.json");
    fs::write(&path, serde_json::to_vec(&config)?)?;
    drop(sockets);
    let mut child: Running = engine.start(&path)?;
    let root = format!("http://127.0.0.1:{admin}");
    let client = AdminClient::builder()
        .http(HttpAdminClientSettings::new(AdminEndpoint::from_url(
            &root,
        )?))
        .build()?;
    until("two running pipelines", async || {
        child.check()?;
        if !matches!(client.engine().readyz().await, Ok(probe) if probe.status == otel_arrow_dfe_admin_api::engine::ProbeStatus::Ok) {
            return Ok(false);
        }
        for id in ["main","observer"] {
            if !matches!(client.pipelines().status("default",id).await,Ok(Some(s)) if s.running_cores == 1) {
                return Ok(false);
            }
        }
        Ok(true)
    }).await?;
    let generation = client
        .pipelines()
        .details("default", "observer")
        .await?
        .and_then(|d| d.active_generation)
        .context("missing observer generation")?;
    let harness = Harness {
        client,
        root,
        directory: temp.path().to_owned(),
        input,
        observer,
        generation,
    };
    if let Some(server) = capture {
        http_probe(server, input, "initial-main", None).await?;
    } else {
        harness.probe(input, "main", "initial-main", None).await?;
    }
    harness.observer("initial-observer").await?;
    let original = serde_json::to_value(harness.details("main").await?.pipeline)?;
    Ok(Session {
        harness,
        original,
        ports: [extra, changed],
        _process: child,
        _directory: temp,
    })
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

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Outcome {
    Passed,
    Failed { reason: String },
    Unexercised,
}

pub type Outcomes = BTreeMap<&'static str, Outcome>;

impl Session {
    async fn step(
        &self,
        fixture: &Fixture,
        operation: Operation,
        capture: Option<&MockServer>,
    ) -> Result<()> {
        let h = &self.harness;
        let phase = operation.name();
        let present = !matches!(operation, Operation::Remove);
        let port = self.ports[usize::from(matches!(operation, Operation::Change))];
        let marker = format!("fixture-{phase}-record");
        let mut candidate = self.original.clone();
        if present {
            let key_file = h.directory.join(format!("{phase}.key"));
            if matches!(fixture.probe, Probe::OtlpHttpHeader { .. }) {
                fs::write(&key_file, phase)?;
            }
            let output_file = h.directory.join(format!(
                "{phase}-{{signal}}-{{core_id}}-{{generation}}.jsonl"
            ));
            let vars = minijinja::context!(phase => phase, address => format!("127.0.0.1:{port}"), output_file => output_file, key_file => key_file);
            let subject = json!({"type": fixture.urn, "config": fixture.configuration(matches!(operation, Operation::Change), vars)?});
            match &fixture.probe {
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
        h.rollout(&candidate, phase).await?;
        match &fixture.probe {
            Probe::SyslogTcp => {
                match operation {
                    Operation::Add => (),
                    Operation::Change => closed(self.ports[0]).await?,
                    Operation::Remove => closed(self.ports[1]).await?,
                }
                h.probe(if present { port } else { h.input }, "main", &marker, None)
                    .await
            }
            Probe::LogAttribute { key } => {
                h.probe(
                    h.input,
                    "main",
                    &marker,
                    Some((key, present.then_some(phase))),
                )
                .await
            }
            Probe::JsonlFile => {
                let active = if present { phase } else { "main" };
                h.probe(h.input, active, &marker, None).await?;
                for old in ["main", "add", "change"] {
                    if old != active {
                        ensure!(
                            !delivered(&h.directory, old, &marker, None)?,
                            "{old} still received {marker}"
                        );
                    }
                }
                Ok(())
            }
            Probe::OtlpHttpHeader { header, .. } => {
                http_probe(
                    capture.context("missing HTTP capture")?,
                    h.input,
                    &marker,
                    Some((header, present.then_some(phase))),
                )
                .await
            }
        }
    }
}

pub async fn exercise(engine: &Engine, fixture: &Fixture, results: &mut Outcomes) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(60);
    let (capture, session) = timeout_at(deadline, async {
        let capture = if matches!(fixture.probe, Probe::OtlpHttpHeader { .. }) {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/logs"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .insert_header("content-type", "application/x-protobuf"),
                )
                .mount(&server)
                .await;
            Some(server)
        } else {
            None
        };
        let session = start(engine, capture.as_ref()).await?;
        Ok::<_, anyhow::Error>((capture, session))
    })
    .await
    .context("fixture setup timed out")??;
    for operation in Operation::ALL {
        let name = operation.name();
        let result = timeout_at(deadline, session.step(fixture, operation, capture.as_ref()))
            .await
            .with_context(|| format!("timed out at {name}"))
            .and_then(|r| r);
        if let Err(error) = result {
            let _ = results.insert(
                name,
                Outcome::Failed {
                    reason: format!("{error:#}"),
                },
            );
            return Err(error.context(name));
        }
        let _ = results.insert(name, Outcome::Passed);
    }
    Ok(())
}

pub fn pending() -> Outcomes {
    Operation::ALL
        .into_iter()
        .map(|op| (op.name(), Outcome::Unexercised))
        .collect()
}

// Keep the rejected-update regression independently observable if another fixture fails.
pub async fn rejected_update(engine: &Engine) -> Result<()> {
    let session = start(engine, None).await?;
    let mut committed = session.original;
    let h = &session.harness;
    committed["nodes"]["out"] = file_sink(&h.directory, "committed");
    h.rollout(&committed, "before-rejection").await?;
    h.probe(h.input, "committed", "before-rejection", None)
        .await?;
    h.rejection("committed").await
}
