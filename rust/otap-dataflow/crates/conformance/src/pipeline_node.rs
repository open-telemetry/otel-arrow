// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Offline peer pipelines and operator-visible observations used by catalog fixtures.
use super::fixtures::{ByOperation, ExpectedText, rendered};
use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_conformance_model::Operation;
use otel_arrow_dfe_pdata::proto::opentelemetry::{
    collector::{
        logs::v1::{ExportLogsServiceRequest, ExportLogsServiceResponse},
        metrics::v1::{ExportMetricsServiceRequest, ExportMetricsServiceResponse},
    },
    common::v1::{AnyValue, KeyValue, any_value},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
    metrics::v1::{
        Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, metric, number_data_point,
    },
    resource::v1::Resource,
};
use otel_arrow_dfe_validation::deployment::{
    syslog::{delivered, send},
    wait_for as until,
};
use prost::Message;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{fs, path::Path};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

/// Recipe for a YAML `pipeline-node` probe. The fixture supplies the node named `subject`.
/// This graph connects its peers and pairs an input with the observation that proves the stage passed.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineNodeProbe {
    pub role: String,
    #[serde(default)]
    peers: Value,
    #[serde(default)]
    connections: Option<Value>,
    #[serde(default)]
    outputs: Option<Value>,
    #[serde(default)]
    bindings: Option<Value>,
    #[serde(default)]
    extensions: Option<Value>,
    #[serde(default)]
    pub input: Input,
    #[serde(default)]
    expect: Expect,
    #[serde(default)]
    change_policies: Option<Value>,
}

/// How to drive the deployed graph once its rollout succeeds.
#[derive(Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Input {
    #[default]
    Syslog,
    OtlpLogs,
    OtlpMetrics,
    /// The component produces its own traffic, for example by scraping a metrics endpoint.
    Autonomous,
}

/// Evidence required from the graph. The default looks for the stage marker in an
/// exported log. Other variants inspect a protocol response, signal payload or sink.
#[derive(Deserialize, Default)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Expect {
    #[default]
    MarkedLog,
    Signal {
        signal: String,
        #[serde(default)]
        contains: Option<String>,
    },
    /// Require both metric presence and a stage-specific integer sum point.
    Metrics {
        present: Vec<String>,
        sum: ExpectedSum,
    },
    LogAttribute {
        key: String,
        values: ByOperation<ExpectedText>,
    },
    Ack,
    Nack {
        reason: String,
    },
    LogBody {
        value: String,
    },
    GenevaPacket,
    GenevaUpload,
    ScrapedMetric,
    Clickhouse,
    Console,
    Parquet,
    Http {
        path: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedSum {
    name: String,
    add: i64,
    change: i64,
}

impl ExpectedSum {
    // Check the metric sum only while the subject is installed.
    fn value(&self, operation: Operation) -> Result<i64> {
        match operation {
            Operation::Add => Ok(self.add),
            Operation::Change => Ok(self.change),
            Operation::Remove => {
                anyhow::bail!("removing the subject uses the baseline delivery probe")
            }
        }
    }

    fn matches(&self, metric: &Value, expected: i64) -> bool {
        metric["name"] == self.name
            && metric
                .pointer("/sum/dataPoints")
                .and_then(Value::as_array)
                .is_some_and(|points| {
                    points.iter().any(|point| {
                        let value = &point["asInt"];
                        value.as_i64().or_else(|| value.as_str()?.parse().ok()) == Some(expected)
                    })
                })
    }
}

/// Iterate OTLP records across all resource and instrumentation scope groups.
fn records_in_payload<'a>(
    payload: &'a Value,
    resources: &str,
    scopes: &'a str,
    records: &'a str,
) -> impl Iterator<Item = &'a Value> + Clone {
    payload[resources]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(move |r| r[scopes].as_array().into_iter().flatten())
        .flat_map(move |s| s[records].as_array().into_iter().flatten())
}

/// Build an OTLP HTTP receiver that waits for downstream acknowledgement so the client can observe refusal.
pub fn otlp_feed(address: &str) -> Value {
    json!({
        "type": "receiver:otlp",
        "config": {"protocols": {"http": {"listening_addr": address, "wait_for_result": true}}}
    })
}

impl PipelineNodeProbe {
    /// Wire the subject and its peers into the baseline graph. The default route is
    /// `input -> subject -> out`. A fixture using a separate feed keeps a baseline syslog route.
    pub fn configure(
        &self,
        change: bool,
        original: &Value,
        mut subject: Value,
        vars: &minijinja::Value,
    ) -> Result<Value> {
        let mut pipeline = original.clone();
        let nodes = pipeline["nodes"]
            .as_object_mut()
            .context("pipeline nodes")?;
        if let Some(peers) = rendered(&self.peers, vars)?.as_object() {
            nodes.extend(peers.clone());
        }
        if let Some(outputs) = &self.outputs {
            subject["outputs"] = rendered(outputs, vars)?;
        }
        if let Some(bindings) = &self.bindings {
            subject["capabilities"] = rendered(bindings, vars)?;
        }
        if change && let Some(policies) = &self.change_policies {
            subject["policies"] = policies.clone();
        }
        let _ = nodes.insert("subject".into(), subject);
        let edges = match &self.connections {
            Some(edges) => rendered(edges, vars)?,
            None => json!([{"from":"input","to":"subject"},{"from":"subject","to":"out"}]),
        };
        let mut edges = edges
            .as_array()
            .context("connections must be a sequence")?
            .clone();
        if matches!(self.input, Input::OtlpLogs | Input::OtlpMetrics) {
            let _ = nodes.insert("feed".into(), rendered(&otlp_feed("{{ address }}"), vars)?);
            for edge in &mut edges {
                if edge["from"] == "input" {
                    edge["from"] = json!("feed");
                }
            }
        }
        if !edges.iter().any(|e| e["from"] == "input") {
            let _ = nodes.insert("baseline".into(), original["nodes"]["out"].clone());
            edges.push(json!({"from":"input","to":"baseline"}));
        }
        nodes.retain(|name, _| {
            edges.iter().any(|edge| {
                ["from", "to"].iter().any(|key| {
                    edge[key]
                        .as_str()
                        .is_some_and(|end| end.split('[').next() == Some(name.as_str()))
                })
            })
        });
        ensure!(
            nodes.contains_key("subject"),
            "fixture must wire the subject into its pipeline"
        );
        pipeline["connections"] = json!(edges);
        if let Some(extensions) = &self.extensions {
            pipeline["extensions"] = rendered(extensions, vars)?;
        }
        Ok(pipeline)
    }

    /// Probe the restored baseline with the same attribute rule used while installed.
    pub fn removed_attribute(&self) -> Option<(&str, Option<&str>)> {
        match &self.expect {
            Expect::LogAttribute { key, values } => {
                Some((key, values.at(Operation::Remove).value()))
            }
            _ => None,
        }
    }

    /// Drive the configured input once, validate any OTLP reply, then poll for the
    /// expected effect. Polling an asynchronous sink never resends the input.
    pub async fn check(&self, ctx: &ProbeContext<'_>) -> Result<()> {
        let response = match self.input {
            Input::Syslog => {
                send(ctx.input, ctx.marker).await?;
                None
            }
            Input::OtlpLogs | Input::OtlpMetrics => Some(
                post_otlp(
                    ctx.port,
                    ctx.marker,
                    ctx.operation.name(),
                    self.input == Input::OtlpMetrics,
                    None,
                )
                .await?,
            ),
            Input::Autonomous => None,
        };
        if let Some(response) = response {
            let status = response.status();
            let body = response.bytes().await?;
            if let Expect::Nack { reason } = &self.expect {
                let reason = reason.replace("{{ phase }}", ctx.operation.name());
                ensure!(
                    status == reqwest::StatusCode::SERVICE_UNAVAILABLE,
                    "expected downstream refusal, received {status}"
                );
                let rpc = otel_arrow_dfe_otap::http::otlp::RpcStatus::decode(body)?;
                ensure!(
                    rpc.code == 14 && rpc.message.contains(&reason),
                    "missing downstream refusal reason"
                );
            } else {
                ensure!(
                    status.is_success(),
                    "OTLP request failed: {status} {body:?}"
                );
                let rejected = if self.input == Input::OtlpMetrics {
                    ExportMetricsServiceResponse::decode(body)?
                        .partial_success
                        .map_or(0, |p| p.rejected_data_points)
                } else {
                    ExportLogsServiceResponse::decode(body)?
                        .partial_success
                        .map_or(0, |p| p.rejected_log_records)
                };
                ensure!(rejected == 0, "downstream rejected {rejected} items");
            }
        }
        let label = match &self.expect {
            Expect::Metrics { present, sum } => format!(
                "exported metrics {present:?} and {} = {}",
                sum.name,
                sum.value(ctx.operation)?
            ),
            _ => ctx.marker.to_owned(),
        };
        until(&label, async || self.observed(ctx).await).await
    }

    /// Poll the sink or capture for this stage. For an acknowledgement or refusal,
    /// the OTLP reply already supplied the evidence in [`Self::check`].
    async fn observed(&self, ctx: &ProbeContext<'_>) -> Result<bool> {
        let phase = ctx.operation.name();
        match &self.expect {
            Expect::MarkedLog => delivered(ctx.dir, phase, ctx.marker, None),
            Expect::Ack | Expect::Nack { .. } => {
                ensure!(
                    matches!(self.input, Input::OtlpLogs | Input::OtlpMetrics),
                    "response probe requires OTLP input"
                );
                Ok(true)
            }
            Expect::LogBody { value } => output_matches(ctx.dir, phase, "logs", |payload| {
                let expected = value.replace("{{ phase }}", phase);
                payload["resourceLogs"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .flat_map(|r| r["scopeLogs"].as_array().into_iter().flatten())
                    .flat_map(|s| s["logRecords"].as_array().into_iter().flatten())
                    .any(|record| {
                        record.pointer("/body/stringValue").and_then(Value::as_str)
                            == Some(expected.as_str())
                    })
            }),
            Expect::ScrapedMetric => {
                let metric = format!("guarantee_{phase}");
                let requests = ctx
                    .server
                    .received_requests()
                    .await
                    .context("HTTP capture unavailable")?;
                for req in requests.iter().filter(|r| r.url.path() == "/v1/metrics") {
                    let metrics = ExportMetricsServiceRequest::decode(req.body.as_slice())?;
                    let samples = metrics
                        .resource_metrics
                        .iter()
                        .flat_map(|resource| &resource.scope_metrics)
                        .flat_map(|scope| &scope.metrics);
                    for sample in samples {
                        if sample.name == metric
                            && let Some(metric::Data::Gauge(gauge)) = &sample.data
                            && !gauge.data_points.is_empty()
                        {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            }
            Expect::GenevaUpload => {
                let token = format!("Bearer ingestion-{phase}");
                let requests = ctx
                    .server
                    .received_requests()
                    .await
                    .context("HTTP capture unavailable")?;
                Ok(requests.iter().any(|r| {
                    r.method == "POST"
                        && r.url.path() == "/api/v1/ingestion/ingest"
                        && r.headers
                            .get("authorization")
                            .is_some_and(|v| v == token.as_str())
                        && !r.body.is_empty()
                }))
            }
            Expect::GenevaPacket => {
                let expected = format!("/packets/{phase}");
                let bearer = format!("Bearer {phase}");
                let requests = ctx
                    .server
                    .received_requests()
                    .await
                    .context("HTTP capture unavailable")?;
                Ok(requests.iter().any(|req| {
                    req.method == "POST"
                        && req.url.path() == expected
                        && req
                            .headers
                            .get("authorization")
                            .is_some_and(|v| v == bearer.as_str())
                        && req
                            .headers
                            .get("content-type")
                            .is_some_and(|v| v == "application/x-mepacket")
                        && !req.body.is_empty()
                }))
            }
            Expect::Clickhouse => {
                let table = format!("logs_{phase}");
                let requests = ctx
                    .server
                    .received_requests()
                    .await
                    .context("HTTP capture unavailable")?;
                Ok(requests.iter().any(|req| {
                    req.method == "POST"
                        && !req.body.is_empty()
                        && req.url.query_pairs().any(|(key, value)| {
                            key == "query"
                                && value.starts_with("INSERT INTO")
                                && value.contains(&table)
                                && value.contains("ArrowStream")
                        })
                }))
            }
            Expect::Console => {
                Ok(fs::read_to_string(ctx.dir.join("engine.log"))?.contains(ctx.marker))
            }
            Expect::LogAttribute { key, values } => delivered(
                ctx.dir,
                phase,
                ctx.marker,
                Some((key, values.at(ctx.operation).value())),
            ),
            Expect::Metrics { present, sum } => {
                let expected = sum.value(ctx.operation)?;
                output_matches(ctx.dir, phase, "metrics", |payload| {
                    let metrics =
                        records_in_payload(payload, "resourceMetrics", "scopeMetrics", "metrics");
                    present
                        .iter()
                        .all(|name| metrics.clone().any(|metric| metric["name"] == *name))
                        && metrics.clone().any(|metric| sum.matches(metric, expected))
                })
            }
            Expect::Signal {
                signal,
                contains: needle,
            } => {
                let value = needle.as_deref().map(|s| {
                    s.replace("{{ phase }}", phase)
                        .replace("{{ marker }}", ctx.marker)
                });
                let (resources, scopes, records) = match signal.as_str() {
                    "logs" => ("resourceLogs", "scopeLogs", "logRecords"),
                    "metrics" => ("resourceMetrics", "scopeMetrics", "metrics"),
                    other => anyhow::bail!("unsupported signal {other}"),
                };
                output_matches(ctx.dir, phase, signal, |v| {
                    records_in_payload(v, resources, scopes, records)
                        .any(|record| value.as_deref().is_none_or(|n| contains(record, n)))
                })
            }
            Expect::Parquet => parquet_contains(&ctx.dir.join(phase), ctx.marker),
            Expect::Http { path } => {
                let expected = path.replace("{{ phase }}", phase);
                let requests = ctx
                    .server
                    .received_requests()
                    .await
                    .context("HTTP capture unavailable")?;
                for req in requests {
                    if req.method != "POST" || !req.url.path().contains(&expected) {
                        continue;
                    }
                    let mut data = Vec::new();
                    if req.body.starts_with(&[0x1f, 0x8b]) {
                        use std::io::Read;
                        let _ =
                            flate2::read::GzDecoder::new(&req.body[..]).read_to_end(&mut data)?;
                    } else {
                        data = req.body;
                    }
                    if data
                        .windows(ctx.marker.len())
                        .any(|v| v == ctx.marker.as_bytes())
                    {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
}

/// Addresses, files and marker for observing one add or change stage.
pub struct ProbeContext<'a> {
    pub dir: &'a Path,
    pub operation: Operation,
    pub marker: &'a str,
    /// Port of the baseline syslog receiver.
    pub input: u16,
    /// Port reserved for this stage's listener, such as the OTLP feed.
    pub port: u16,
    pub server: &'a MockServer,
}

fn string(s: &str) -> AnyValue {
    AnyValue {
        value: Some(any_value::Value::StringValue(s.into())),
    }
}
fn attribute(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.into(),
        value: Some(string(value)),
    }
}

/// Send one OTLP/HTTP protobuf request with the marker in the log body or metric name.
/// Return the raw response so the caller can check acceptance, partial success or refusal.
pub async fn post_otlp(
    port: u16,
    marker: &str,
    phase: &str,
    metrics: bool,
    token: Option<&str>,
) -> Result<reqwest::Response> {
    let resource = Some(Resource {
        attributes: vec![attribute("service.name", phase)],
        ..Default::default()
    });
    let (signal, bytes) = if metrics {
        let point = NumberDataPoint {
            start_time_unix_nano: 1,
            time_unix_nano: 2,
            value: Some(number_data_point::Value::AsInt(7)),
            ..Default::default()
        };
        let metric = Metric {
            name: marker.into(),
            data: Some(metric::Data::Sum(Sum {
                aggregation_temporality: 1,
                is_monotonic: true,
                data_points: vec![point],
            })),
            ..Default::default()
        };
        (
            "metrics",
            ExportMetricsServiceRequest {
                resource_metrics: vec![ResourceMetrics {
                    resource,
                    scope_metrics: vec![ScopeMetrics {
                        metrics: vec![metric],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }
            .encode_to_vec(),
        )
    } else {
        let log = LogRecord {
            body: Some(string(marker)),
            severity_text: "ERROR".into(),
            severity_number: 17,
            attributes: vec![
                attribute("syslog.message", marker),
                attribute("guarantee.input", phase),
            ],
            ..Default::default()
        };
        (
            "logs",
            ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    resource,
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![log],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }
            .encode_to_vec(),
        )
    };
    let mut request = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{port}/v1/{signal}"))
        .header("content-type", "application/x-protobuf")
        .body(bytes);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    Ok(request.send().await?)
}

fn contains(v: &Value, needle: &str) -> bool {
    match v {
        Value::Object(m) => m.values().any(|v| contains(v, needle)),
        Value::Array(a) => a.iter().any(|v| contains(v, needle)),
        Value::String(s) => s == needle,
        _ => false,
    }
}

/// Search completed JSONL records across cores and generations, ignoring a trailing write in progress.
fn output_matches(
    dir: &Path,
    prefix: &str,
    signal: &str,
    matches: impl Fn(&Value) -> bool,
) -> Result<bool> {
    for item in fs::read_dir(dir)? {
        let item = item?;
        if !item
            .file_name()
            .to_string_lossy()
            .starts_with(&format!("{prefix}-{signal}-"))
        {
            continue;
        }
        for line in fs::read(item.path())?
            .split_inclusive(|b| *b == b'\n')
            .filter(|l| l.ends_with(b"\n"))
        {
            let v: Value = serde_json::from_slice(line)?;
            if matches(&v) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn parquet_contains(dir: &Path, marker: &str) -> Result<bool> {
    use parquet::file::reader::{FileReader, SerializedFileReader};
    if !dir.exists() {
        return Ok(false);
    }
    for item in fs::read_dir(dir)? {
        let path = item?.path();
        if path.is_dir() && parquet_contains(&path, marker)? {
            return Ok(true);
        }
        if path.extension().is_some_and(|s| s == "parquet") {
            let reader = SerializedFileReader::new(fs::File::open(path)?)?;
            for row in reader.get_row_iter(None)? {
                if row?.to_string().contains(marker) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

/// Start a loopback HTTP peer that serves fixture inputs and records outbound requests for inspection.
pub async fn services() -> MockServer {
    let server = MockServer::start().await;
    for phase in ["add", "change"] {
        let token = json!({"access_token":phase,"token_type":"Bearer","expires_in":3600});
        for (verb, route) in [
            ("POST", format!("/token/{phase}")),
            (
                "GET",
                format!("/computeMetadata/v1/instance/service-accounts/{phase}/token"),
            ),
        ] {
            Mock::given(method(verb))
                .and(path(route))
                .respond_with(
                    ResponseTemplate::new(200)
                        .insert_header("metadata-flavor", "Google")
                        .set_body_json(&token),
                )
                .mount(&server)
                .await;
        }
        Mock::given(method("GET"))
            .and(path(format!("/metrics/{phase}")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                format!("# HELP guarantee_{phase} Fixture sample.\n# TYPE guarantee_{phase} gauge\nguarantee_{phase}{{source=\"fixture\"}} 7\n"),
                "text/plain; version=0.0.4",
            ))
            .mount(&server)
            .await;
    }
    Mock::given(method("POST")).and(path("/apis/authentication.k8s.io/v1/tokenreviews")).respond_with(|req:&wiremock::Request| {
        let body:Value=serde_json::from_slice(&req.body).unwrap_or_default();
        let phase=body["spec"]["audiences"][0].as_str().unwrap_or_default();
        let accepted=!phase.is_empty() && body["spec"]["token"] == format!("token-{phase}");
        ResponseTemplate::new(201).set_body_json(json!({
            "apiVersion": "authentication.k8s.io/v1", "kind": "TokenReview", "metadata": {},
            "status": {"authenticated": accepted, "audiences": [phase], "user": {"username":"system:serviceaccount:fixture:test"}}
        }))
    }).mount(&server).await;
    for phase in ["add", "change"] {
        let resource = format!("https://{phase}.example");
        let token = json!({"access_token":phase,"token_type":"Bearer","resource":resource,"expires_on":"4102444800"});
        Mock::given(method("GET"))
            .and(path("/identity"))
            .and(wiremock::matchers::query_param("resource", &resource))
            .respond_with(ResponseTemplate::new(200).set_body_json(token))
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).insert_header("content-type", "application/x-protobuf"),
        )
        .with_priority(10)
        .mount(&server)
        .await;
    server
}

/// Serve Geneva discovery and ingestion locally, issuing a distinct ingestion token for each stage.
pub async fn geneva_services(server: &MockServer, endpoint: &str) {
    for phase in ["add", "change"] {
        Mock::given(method("GET")).and(path(format!("/userapi/agent/v3/fixture/{phase}/MonitoringStorageKeys/")))
            .and(wiremock::matchers::header("authorization",format!("Bearer {phase}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "IngestionGatewayInfo":{"Endpoint":endpoint,"AuthToken":format!("ingestion-{phase}"),"AuthTokenExpiryTime":"2099-01-01T00:00:00Z"},
                "StorageAccountKeys":[{"AccountMonikerName":"fixture-moniker","AccountGroupName":"fixture","IsPrimaryMoniker":true}],
                "TagId":"fixture"
            }))).mount(server).await;
    }
    Mock::given(method("POST"))
        .and(path("/api/v1/ingestion/ingest"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({"ticket":"fixture"})))
        .mount(server)
        .await;
}
