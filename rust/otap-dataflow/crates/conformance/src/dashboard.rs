// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exercise the HTTP and WebSocket interfaces consumed by the shipped dashboard.
//! [`Inputs`] defines expectations from `telemetry-signals.yaml`. This module
//! supplies live responses and records the result for each UI consumer.
mod resync;
mod status;

use super::{fixture_session::FixtureSession, graph};
use anyhow::{Context, Result, bail, ensure};
use chrono::DateTime;
use futures_util::{SinkExt, StreamExt};
use otel_arrow_dfe_admin_api::telemetry::MetricsResponse;
use otel_arrow_dfe_conformance_model::UiSignal;
use otel_arrow_dfe_conformance_model::ui::{Events, Inputs, METRICS, SIGNAL_CATALOG};
use otel_arrow_dfe_validation::deployment::Engine;
use regex::Regex;
use reqwest::{Client, Url};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::{Instant, sleep};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};

const LIVE: &str = "/api/v1/livez";
const READY: &str = "/api/v1/readyz";
const STATUS: &str = "/api/v1/status";
const STREAM: &str = "/api/v1/telemetry/logs/stream";
const QUERY: &str = "format=json&reset=false&keep_all_zeroes=true";

/// Start with log tapping disabled to check its diagnostic and discover available cores.
/// Then run the dashboard workload with detailed metrics and log tapping enabled.
pub async fn run(engine: &Engine, rows: &mut Vec<UiSignal>) -> Result<()> {
    let server = graph::services().await;
    // Discover actual core IDs from the candidate, including non-contiguous
    // cpusets in containers. Reuse these two cores for all dashboard pipelines.
    let cores = {
        let without_tap = FixtureSession::start(engine, None, &server, |config, _| {
            config["engine"]["telemetry"] = json!({"logs":{"tap":{"enabled":false}}});
        })
        .await?;
        check_disabled_tap(without_tap.engine.admin_url()).await?;
        let status = without_tap.engine.client().engine().status().await?;
        let core = |name: &str| -> Result<usize> {
            status
                .pipelines
                .get(name)
                .and_then(|p| p.cores.keys().next().copied())
                .with_context(|| format!("missing assigned core for {name}"))
        };
        let cores = [core("default:main")?, core("default:observer")?];
        ensure!(
            cores[0] != cores[1],
            "dashboard deployment requires two visible engine cores"
        );
        cores
    };
    let session =
        FixtureSession::start(engine, None, &server, |config, _| configure(config, cores)).await?;
    check(
        session.engine.admin_url(),
        session.add_listener.port,
        session.probe(session.input, "main", "dashboard-traffic", None),
        rows,
    )
    .await
}

/// Publish from `main` to a topic consumed by a two-core reader. Its file output
/// confirms delivery and its two instances supply samples for the all-core selector.
fn configure(config: &mut Value, cores: [usize; 2]) {
    config["engine"]["telemetry"] = json!({"reporting_interval":"200ms","logs":{"level":"info","providers":{"admin":"its"},"tap":{"enabled":true,"max_entries":2048,"max_bytes":8388608}}});
    config["engine"]["observability"]["pipeline"] = json!({
        "nodes": {
            "telemetry":{"type":"receiver:internal_telemetry","config":{"signals":["logs"]}},
            "sink":{"type":"exporter:noop","config":{}}
        },
        "connections":[{"from":"telemetry","to":"sink"}]
    });
    config["policies"]["telemetry"] = json!({"runtime_metrics":"detailed"});
    let core_set = |ids: &[usize]| {
        let ranges: Vec<_> = ids.iter().map(|id| json!({"start":id,"end":id})).collect();
        json!({"type":"core_set","set":ranges})
    };
    config["policies"]["resources"]["core_allocation"] = core_set(&cores[..1]);
    let group = &mut config["groups"]["default"];
    group["topics"] = json!({"ui":{"impl_selection":"force_mixed"}});
    let pipelines = &mut group["pipelines"];
    pipelines["observer"]["policies"]["resources"]["core_allocation"] = core_set(&cores[1..]);
    let file = pipelines["main"]["nodes"]["out"].take();
    pipelines["main"]["nodes"]["out"] = json!({"type":"exporter:topic","config":{"topic":"ui"}});
    pipelines["reader"] = json!({"policies":{"resources":{"core_allocation":core_set(&cores)},"channel_capacity":{"pdata":32}},"nodes":{"topic":{"type":"receiver:topic","config":{"topic":"ui"}},"out":file},
        "connections":[{"from":"topic","to":"out"}]});
}

/// Check assets and health endpoints first. Once reachable, exercise metrics, status
/// and logs, collecting independent failures so each affected view can be reported.
async fn check(
    root: &str,
    occupied_port: u16,
    workload: impl Future<Output = Result<()>>,
    rows: &mut Vec<UiSignal>,
) -> Result<()> {
    let http = Client::builder()
        .timeout(Duration::from_secs(4))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let base = Url::parse(root)?;
    let inputs = Inputs::parse(SIGNAL_CATALOG)?;
    assets(&http, &base)
        .await
        .context("dashboard assets and API references")?;
    for path in [LIVE, READY] {
        let response = http.get(base.join(path)?).send().await?;
        ensure!(
            response.status().is_success(),
            "{path}: {}",
            response.status()
        );
    }
    let metrics = async {
        let baseline = samples(&http, &base, &inputs, None, rows).await?;
        workload.await.context("marked traffic delivery")?;
        let _ = samples(&http, &base, &inputs, Some(&baseline), rows).await?;
        Ok(())
    }
    .await;
    let status = async {
        let status = get_json(&http, base.join(STATUS)?).await?;
        status::healthy(&status, &inputs.events, rows).context(STATUS)
    }
    .await;
    let status = observation(
        rows,
        "Status overlay",
        "running pipeline and renderable Ready event",
        status,
    );
    let logs = tokio::time::timeout(
        Duration::from_secs(7),
        logs(&http, &base, &inputs.events, rows),
    )
    .await
    .context("dashboard log stream timed out")
    .and_then(|r| r);
    let logs = observation(
        rows,
        "Live logs",
        "log stream and retained event fields",
        logs,
    );
    let resync = tokio::time::timeout(
        Duration::from_secs(12),
        resync::check(&http, &base, &inputs.events),
    )
    .await
    .context("timed out waiting for lag and recovery")
    .and_then(|r| r);
    let resync = observation(rows, "Live logs", &inputs.events.resync_signal(), resync);
    let lifecycle = status::lifecycle(&http, &base, occupied_port, &inputs.events, rows).await;
    let failures: Vec<_> = [metrics, status, logs, resync, lifecycle]
        .into_iter()
        .filter_map(Result::err)
        .map(|e| format!("{e:#}"))
        .collect();
    ensure!(failures.is_empty(), "{}", failures.join("; "));
    Ok(())
}

fn observation(
    rows: &mut Vec<UiSignal>,
    consumer: &str,
    signal: &str,
    result: Result<()>,
) -> Result<()> {
    rows.push(UiSignal {
        consumer: consumer.into(),
        signal: signal.into(),
        result: match &result {
            Ok(()) => "observed".into(),
            Err(e) => format!("{e:#}"),
        },
    });
    result
}

/// Pre-workload counter totals keyed by endpoint, then by series. Each endpoint is compared to its own baseline.
type Baselines = BTreeMap<String, BTreeMap<String, f64>>;

/// Poll the schema and both metrics endpoints across reporting intervals. The first
/// pass captures counter totals. The second records per-signal results, including
/// whether the required traffic counters increased after confirmed delivery.
async fn samples(
    http: &Client,
    base: &Url,
    inputs: &Inputs,
    before: Option<&Baselines>,
    rows: &mut Vec<UiSignal>,
) -> Result<Baselines> {
    let mut baselines = BTreeMap::new();
    let mut failures = Vec::new();
    for path in METRICS {
        let mut url = base.join(path)?;
        url.set_query(Some(QUERY));
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let schema = get_json(http, base.join("/api/v1/telemetry/live-schema")?)
                .await
                .context("live telemetry declaration")?;
            let data = get_json(http, url.clone()).await?;
            ensure!(timestamp(&data["timestamp"]), "invalid metrics timestamp");
            let _ = MetricsResponse::deserialize(&data)
                .context("unusable dashboard metrics snapshot")?;
            let result = inputs.measure(&schema, &data, path, before.and_then(|b| b.get(path)))?;
            let passed = result.passed();
            if passed || Instant::now() >= deadline {
                if before.is_some() {
                    if !passed {
                        failures.push(path);
                    }
                    rows.extend(result.rows);
                }
                let _ = baselines.insert(path.to_owned(), result.traffic);
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
    }
    ensure!(
        failures.is_empty(),
        "dashboard telemetry mismatch on {} (see per-signal results)",
        failures.join(", ")
    );
    Ok(baselines)
}

async fn get_json(http: &Client, url: Url) -> Result<Value> {
    Ok(http
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

/// Follow the dashboard's local assets and module imports. Compare the API routes
/// referenced by its JavaScript with the routes exercised by the wire probes.
async fn assets(http: &Client, base: &Url) -> Result<()> {
    let references = Regex::new(r#"(?:src|href)=["']([^"']+)["']"#)?;
    let imports = Regex::new(r#"(?:\bfrom\s+|\bimport\s*(?:\(\s*)?)["']([^"']+)["']"#)?;
    let routes = Regex::new(r"/api/v[0-9]+/[a-zA-Z0-9_/-]+")?;
    let mut pending = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut api = BTreeSet::new();
    for entry in ["/", "/dashboard"] {
        let response = http
            .get(base.join(entry)?)
            .send()
            .await?
            .error_for_status()?;
        let headers = response.headers();
        ensure!(
            headers
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .is_some_and(|v| v.starts_with("text/html")),
            "{entry}: expected HTML"
        );
        if let Some(header) = headers.get("content-security-policy") {
            let directives: BTreeMap<_, _> = header
                .to_str()?
                .split(';')
                .filter_map(|d| d.trim().split_once(' '))
                .collect();
            for kind in ["script-src", "connect-src", "style-src", "img-src"] {
                if let Some(sources) = directives.get(kind).or(directives.get("default-src")) {
                    ensure!(
                        sources
                            .split_whitespace()
                            .any(|s| s == "'self'" || s == "*"),
                        "{entry}: {kind} blocks dashboard resources on its own origin"
                    );
                }
            }
        }
        let html = response.text().await?;
        ensure!(
            html.contains("data-live-logs-ws=\"true\"") && html.contains("type=\"module\""),
            "{entry}: expected dashboard with live logs"
        );
        for capture in references.captures_iter(&html) {
            let _ = pending.insert(base.join(&capture[1])?);
        }
    }
    while let Some(url) = pending.pop_first() {
        if !seen.insert(url.clone()) {
            continue;
        }
        ensure!(
            url.origin() == base.origin() && url.path().starts_with("/static/"),
            "unexpected dashboard asset {url}"
        );
        let response = http.get(url.clone()).send().await?.error_for_status()?;
        let mime = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let js = url.path().ends_with(".js");
        let expected = if js {
            "javascript"
        } else if url.path().ends_with(".css") {
            "text/css"
        } else {
            "image/"
        };
        ensure!(
            mime.contains(expected),
            "{url}: unexpected MIME type {mime}"
        );
        let bytes = response.bytes().await?;
        ensure!(!bytes.is_empty(), "{url}: empty asset");
        if js && !url.path().starts_with("/static/vendor/") {
            let source = std::str::from_utf8(&bytes)?
                .lines()
                .filter(|s| !s.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            api.extend(routes.find_iter(&source).map(|m| m.as_str().to_owned()));
            for capture in imports.captures_iter(&source) {
                let _ = pending.insert(url.join(&capture[1])?);
            }
        }
    }
    let exercised = [LIVE, READY, STATUS, STREAM, METRICS[0], METRICS[1]]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    ensure!(
        api == exercised,
        "dashboard references {api:?}, wire probes cover {exercised:?}"
    );
    Ok(())
}

fn timestamp(v: &Value) -> bool {
    v.as_str()
        .is_some_and(|s| DateTime::parse_from_rfc3339(s).is_ok())
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Wait for a frame of the requested type, allowing live logs to arrive between command responses.
async fn receive(ws: &mut Socket, kind: &str) -> Result<Value> {
    loop {
        let message = ws.next().await.context("log socket closed")??;
        if let Message::Text(s) = message {
            let event: Value = serde_json::from_str(&s)?;
            if event["type"] == kind {
                return Ok(event);
            }
            ensure!(
                event["type"] != "error",
                "log stream error: {}",
                event["message"]
            );
            ensure!(
                event["type"] == "log",
                "expected {kind}, received {}",
                event["type"]
            );
        } else if message.is_close() {
            bail!("log socket closed waiting for {kind}");
        }
    }
}

async fn command(ws: &mut Socket, json: Value) -> Result<()> {
    Ok(ws.send(Message::text(json.to_string())).await?)
}

fn usable_log_field(value: &Value, role: &str) -> bool {
    if role == "timestamp" {
        timestamp(value)
    } else {
        value.as_str().is_some_and(|s| !s.is_empty())
    }
}

fn log_entry(entry: &Value, fields: &BTreeMap<String, String>) -> bool {
    entry["seq"].is_u64()
        && fields
            .iter()
            .all(|(role, key)| usable_log_field(&entry[key], role))
}

async fn socket(base: &Url) -> Result<Socket> {
    let mut url = base.join(STREAM)?;
    url.set_scheme("ws")
        .map_err(|()| anyhow::anyhow!("invalid WebSocket URL"))?;
    let mut request = url.as_str().into_client_request()?;
    let _ = request
        .headers_mut()
        .insert("origin", base.origin().ascii_serialization().parse()?);
    let (ws, _) = connect_async(request).await?;
    Ok(ws)
}

/// Require a log-stream diagnostic that the dashboard can display when tapping is disabled.
async fn check_disabled_tap(root: &str) -> Result<()> {
    let mut ws = socket(&Url::parse(root)?).await?;
    let frame = receive(&mut ws, "error").await?;
    ensure!(
        frame["message"].as_str().is_some_and(|s| !s.is_empty()),
        "missing log panel diagnostic when the tap is disabled"
    );
    Ok(())
}

/// Exercise the viewer's stream controls and trigger a config reconciliation.
/// Require a renderable live event and recover that same sequence number through backfill.
async fn logs(http: &Client, base: &Url, events: &Events, rows: &mut Vec<UiSignal>) -> Result<()> {
    let mut ws = socket(base).await?;
    command(&mut ws,json!({"type":"subscribe","after":null,"limit":100,"searchQuery":null,"minimumTimestamp":null,"minimumLevel":null,"paused":false})).await?;
    let snapshot = receive(&mut ws, "snapshot").await?;
    let cursor = snapshot["next_seq"]
        .as_u64()
        .context("missing snapshot cursor")?;
    ensure!(
        snapshot["logs"]
            .as_array()
            .is_some_and(|entries| entries.iter().all(|e| log_entry(e, &events.log_fields))),
        "unusable retained logs"
    );
    let initial = receive(&mut ws, "state").await?;
    ensure!(
        initial["paused"] == false && initial["next_seq"] == cursor,
        "invalid initial log state"
    );
    command(&mut ws, json!({"type":"pause"})).await?;
    ensure!(
        receive(&mut ws, "state").await?["paused"] == true,
        "pause failed"
    );
    command(&mut ws, json!({"type":"resume"})).await?;
    ensure!(
        receive(&mut ws, "state").await?["paused"] == false,
        "resume failed"
    );
    let marker = "config.reconcile.requested";
    command(&mut ws,json!({"type":"setFilter","searchQuery":marker,"minimumTimestamp":null,"minimumLevel":"INFO"})).await?;
    command(&mut ws, json!({"type":"ping"})).await?;
    let _ = receive(&mut ws, "pong").await?;
    let url = base.join("/api/v1/config")?;
    let config = get_json(http, url).await?;
    let _ = http
        .post(base.join("/api/v1/config/reconcile")?)
        .json(&json!({"config":config}))
        .send()
        .await?
        .error_for_status()?;
    let event = receive(&mut ws, "log").await?;
    for (role, field) in &events.log_fields {
        let usable = usable_log_field(&event[field], role);
        rows.push(UiSignal {
            consumer: "Live logs".into(),
            signal: format!("field {field}"),
            result: if usable {
                "observed"
            } else {
                "missing or unusable log field"
            }
            .into(),
        });
    }
    let name_field = events.log_fields.get("name");
    ensure!(
        log_entry(&event, &events.log_fields)
            && name_field.is_none_or(|field| event[field]
                .as_str()
                .is_some_and(|name| name.contains(marker)))
            && event["seq"].as_u64() > Some(cursor),
        "unusable live log: {event}"
    );
    // Today the live log view displays arbitrary event names. If a view starts
    // depending on a named event, require a matching record from this deployment.
    for name in &events.named_logs {
        let observed = snapshot["logs"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(std::iter::once(&event))
            .any(|e| {
                name_field.is_some_and(|field| e[field] == *name)
                    && log_entry(e, &events.log_fields)
            });
        let result = if observed {
            "observed"
        } else {
            "expected event not observed"
        };
        rows.push(UiSignal {
            consumer: "Live logs".into(),
            signal: name.clone(),
            result: result.into(),
        });
    }
    ensure!(
        rows.iter()
            .filter(|r| r.consumer == "Live logs")
            .all(|r| r.result == "observed"),
        "missing named dashboard log event (see per-signal results)"
    );
    command(
        &mut ws,
        json!({"type":"backfill","after":cursor,"limit":100}),
    )
    .await?;
    let backfill = receive(&mut ws, "snapshot").await?;
    ensure!(
        backfill["logs"]
            .as_array()
            .is_some_and(|logs| logs
                .iter()
                .any(|l| log_entry(l, &events.log_fields) && l["seq"] == event["seq"])),
        "backfill lost the live event"
    );
    Ok(())
}
