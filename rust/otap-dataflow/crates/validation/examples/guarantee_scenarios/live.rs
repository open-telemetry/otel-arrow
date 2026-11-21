// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::engine::{Engine, Running};
use anyhow::{Context, Result, bail, ensure};
use otel_arrow_dfe_admin_api::{
    AdminClient, AdminEndpoint, Error, HttpAdminClientSettings,
    operations::OperationOptions,
    pipelines::{PipelineDetails, PipelineRolloutState, ReconfigureOutcome},
};
use serde_json::{Value, json};
use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{sleep, timeout},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// Coverage is attached to these specific operations after all data-plane assertions pass.
pub const NODE_OPERATIONS: &[&str] = &[
    "urn:otel:receiver:syslog_cef",
    "urn:otel:processor:attribute",
    "urn:otel:exporter:file",
];
pub const EXTENSION_OPERATIONS: &[&str] = &["urn:otel:extension:flat_file_api_key_auth"];

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

fn delivered(dir: &Path, prefix: &str, marker: &str, phase: Option<&str>) -> Result<bool> {
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
                        if attr("syslog.message") == Some(marker)
                            && attr("guarantee.phase") == phase
                        {
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
        phase: Option<&str>,
    ) -> Result<()> {
        send(port, marker).await?;
        until(marker, async || {
            delivered(&self.directory, prefix, marker, phase)
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
        self.observer(&format!("after-{step}")).await
    }

    async fn topology(&self, original: &Value, extra: u16, changed: u16) -> Result<()> {
        let mut candidate = original.clone();
        candidate["connections"]
            .as_array_mut()
            .context("connections")?
            .push(json!({"from":"syslog","to":"out"}));
        for (step, port) in [("add", extra), ("change", changed)] {
            candidate["nodes"]["syslog"] = syslog(port);
            self.rollout(&candidate, step).await?;
            self.probe(port, "main", step, None).await?;
        }
        closed(extra).await?;
        self.rollout(original, "remove").await?;
        closed(changed).await?;
        for phase in ["introduced", "modified"] {
            let mut candidate = original.clone();
            let _ = candidate["nodes"]
                .as_object_mut()
                .context("nodes")?
                .remove("out");
            candidate["nodes"]["destination"] = file_sink(&self.directory, phase);
            candidate["nodes"]["annotate"] = json!({"type":"processor:attribute","config":{
                "actions":[{"action":"upsert","key":"guarantee.phase","value":phase}]}});
            candidate["connections"] =
                json!([{"from":"input","to":"annotate"},{"from":"annotate","to":"destination"}]);
            self.rollout(&candidate, phase).await?;
            self.probe(self.input, phase, phase, Some(phase)).await?;
        }
        self.rollout(original, "restore").await?;
        self.probe(self.input, "main", "restored", None).await?;
        self.rejection().await
    }

    async fn rejection(&self) -> Result<()> {
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
        self.probe(self.input, "main", "after-rejection", None)
            .await?;
        self.observer("observer-after-rejection").await
    }

    async fn extension(&self, original: &Value) -> Result<()> {
        let capture = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/logs"))
            .respond_with(
                ResponseTemplate::new(200).insert_header("content-type", "application/x-protobuf"),
            )
            .mount(&capture)
            .await;
        for (step, secret) in [
            ("add-key", Some("first")),
            ("change-key", Some("second")),
            ("remove-key", None),
        ] {
            let mut candidate = original.clone();
            let mut exporter = json!({"type":"exporter:otlp_http","config":{"endpoint":capture.uri(),
                "client_pool_size":1,"http":{}}});
            if let Some(secret) = secret {
                let file = self.directory.join(format!("{step}.key"));
                fs::write(&file, secret)?;
                candidate["extensions"] = json!({"key":{"type":"urn:otel:extension:flat_file_api_key_auth",
                    "config":{"key_secret_file":file,"attributes":{"http.header_name":"x-test-key"}}}});
                exporter["capabilities"] = json!({"api_key_provider":"key"});
            }
            candidate["nodes"]["out"] = exporter;
            self.rollout(&candidate, step).await?;
            send(self.input, step).await?;
            until(step, async || {
                for req in capture
                    .received_requests()
                    .await
                    .context("capture disabled")?
                {
                    if req
                        .body
                        .windows(step.len())
                        .any(|bytes| bytes == step.as_bytes())
                    {
                        ensure!(
                            req.headers.get("x-test-key").map(|v| v.as_bytes())
                                == secret.map(str::as_bytes),
                            "unexpected API key header at {step}"
                        );
                        return Ok(true);
                    }
                }
                Ok(false)
            })
            .await?;
        }
        Ok(())
    }
}

async fn closed(port: u16) -> Result<()> {
    match TcpStream::connect(("127.0.0.1", port)).await {
        Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => Ok(()),
        _ => bail!("retired listener {port} still accepts connections or could not be verified"),
    }
}

pub async fn run(engine: &Engine, extension: bool) -> Result<()> {
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
    let config = json!({"version":"otel_dataflow/v1","engine":{"http_admin":{"bind_address":format!("127.0.0.1:{admin}")}},
        "policies":{"resources":{"core_allocation":{"type":"core_count","count":1}}},
        "groups":{"default":{"pipelines":{"main":pipeline(input,temp.path(),"main"),"observer":pipeline(observer,temp.path(),"observer")}}}});
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
    harness.probe(input, "main", "initial-main", None).await?;
    harness.observer("initial-observer").await?;
    let original = serde_json::to_value(harness.details("main").await?.pipeline)?;
    if extension {
        harness.extension(&original).await
    } else {
        harness.topology(&original, extra, changed).await
    }
}
