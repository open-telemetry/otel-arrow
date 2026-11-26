// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The target and observer pipelines shared by the deployment scenarios.
use super::{
    fixtures::{Fixture, Probe, Service},
    pipeline_node,
};
use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_validation::deployment::{
    Engine, EngineSession,
    syslog::{self, send},
    wait_for as until,
};
use serde_json::{Value, json};
use std::{fs, net::TcpListener, path::Path};
use wiremock::MockServer;

/// File exporter pattern shared by the baseline and per-operation observations.
pub(super) fn output_path(dir: &Path, prefix: &str) -> std::path::PathBuf {
    dir.join(format!(
        "{prefix}-{{signal}}-{{core_id}}-{{generation}}.jsonl"
    ))
}

fn pipeline(port: u16, dir: &Path, name: &str) -> Result<Value> {
    let template = serde_yaml::from_str(include_str!("../fixtures/baseline-pipeline.yaml"))?;
    super::fixtures::rendered(
        &template,
        minijinja::context!(
            input_address => format!("127.0.0.1:{port}"),
            output_file => output_path(dir, name),
        ),
    )
}

/// All candidate ports are reserved together. A listener is released immediately
/// before the engine needs to bind it. Probes retain its address.
pub(super) struct ReservedPort {
    pub(super) port: u16,
    listener: Option<TcpListener>,
}

impl ReservedPort {
    fn bind() -> std::io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        Ok(Self {
            port: listener.local_addr()?.port(),
            listener: Some(listener),
        })
    }

    pub(super) fn release(&mut self) {
        drop(self.listener.take());
    }
}

/// A running engine with a target pipeline (`main`) and an independent `observer`.
/// The validation crate owns engine startup and shutdown. This session owns the
/// fixture's files, ports and baseline configuration for the lifetime of the probes.
pub(super) struct FixtureSession {
    pub(super) engine: EngineSession,
    pub(super) directory: tempfile::TempDir,
    pub(super) input: u16,
    observer: u16,
    generation: u64,
    pub(super) original: Value,
    pub(super) add_listener: ReservedPort,
    pub(super) change_listener: ReservedPort,
    pub(super) tls: Option<super::tls::TlsCapture>,
}

impl FixtureSession {
    /// Configure local peers, start both pipelines, and verify initial delivery on each.
    /// Capture the committed target graph and the observer's serving generation before any rollout.
    pub(super) async fn start(
        engine: &Engine,
        fixture: Option<&Fixture>,
        services: &MockServer,
        configure: impl FnOnce(&mut Value, &Path),
    ) -> Result<Self> {
        let temp = tempfile::tempdir()?;
        let header_probe = fixture.and_then(|fixture| match &fixture.probe {
            Probe::OtlpHttpHeader(probe) => Some(probe),
            _ => None,
        });
        let mut env = Vec::new();
        let tls = if fixture.is_some_and(|f| f.services.contains(&Service::Geneva)) {
            let tls = super::tls::TlsCapture::start(temp.path(), *services.address()).await?;
            pipeline_node::geneva_services(services, &tls.url).await;
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
        let admin_listener = ReservedPort::bind()?;
        let input_listener = ReservedPort::bind()?;
        let observer_listener = ReservedPort::bind()?;
        let add_listener = ReservedPort::bind()?;
        let change_listener = ReservedPort::bind()?;
        let (admin, input, observer) = (
            admin_listener.port,
            input_listener.port,
            observer_listener.port,
        );
        let mut main = pipeline(input, temp.path(), "main")?;
        if let Some(probe) = header_probe {
            main["nodes"]["out"] = probe.consumer(&services.uri(), false)?;
        }
        let observer_pipeline = pipeline(observer, temp.path(), "observer")?;
        let mut config = json!({"version":"otel_dataflow/v1","engine":{"http_admin":{"bind_address":format!("127.0.0.1:{admin}")}},
        "policies":{"resources":{"core_allocation":{"type":"core_count","count":1}}},
        "groups":{"default":{"pipelines":{"main":main,"observer":observer_pipeline}}}});
        if fixture.is_some_and(|f| f.services.contains(&Service::Topics)) {
            config["groups"]["default"]["topics"] = json!({"add":{"impl_selection":"force_mixed"},"change":{"impl_selection":"force_mixed"}});
        }
        configure(&mut config, temp.path());
        let path = temp.path().join("engine.json");
        fs::write(&path, serde_json::to_vec(&config)?)?;
        drop((admin_listener, input_listener, observer_listener));
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
        let session = Self {
            engine,
            directory: temp,
            input,
            observer,
            generation,
            original,
            add_listener,
            change_listener,
            tls,
        };
        if header_probe.is_some() {
            http_probe(services, input, "initial-main", None).await?;
        } else {
            session.probe(input, "main", "initial-main", None).await?;
        }
        session.observer("initial-observer").await?;
        Ok(session)
    }

    /// Send a syslog marker and wait for the expected file output and optional attribute check.
    pub(super) async fn probe(
        &self,
        port: u16,
        prefix: &str,
        marker: &str,
        attribute: Option<(&str, Option<&str>)>,
    ) -> Result<()> {
        syslog::assert_delivery(port, self.directory.path(), prefix, marker, attribute).await
    }

    /// Require a fresh delivery from the observer and the generation recorded at startup.
    pub(super) async fn observer(&self, marker: &str) -> Result<()> {
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

    /// Submit a replacement for `main` and probe the observer after submission and again after commit.
    pub(super) async fn rollout(&self, pipeline: &Value, step: &str) -> Result<()> {
        let request = serde_json::from_value(json!({"pipeline":pipeline}))?;
        self.engine
            .rollout("default", "main", &request, async || {
                self.observer(&format!("during-{step}")).await
            })
            .await
            .with_context(|| format!("{step} rollout"))?;
        self.observer(&format!("after-{step}")).await
    }

    /// Reapply `main` and require a successful no-op with the same configuration and serving generation.
    pub(super) async fn noop(&self, pipeline: &Value) -> Result<()> {
        let request = serde_json::from_value(json!({"pipeline":pipeline}))?;
        self.engine.assert_noop("default", "main", &request).await
    }
}

/// Find the marked export at the capture server. When a header check is supplied,
/// an absent expected value means the header must have disappeared from the request.
pub(super) async fn http_probe(
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
