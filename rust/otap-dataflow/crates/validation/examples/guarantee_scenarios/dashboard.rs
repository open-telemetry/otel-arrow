// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exercise the same-origin wire contract of the shipped admin dashboard.
use anyhow::{Context, Result, bail, ensure};
use chrono::DateTime;
use futures_util::{SinkExt, StreamExt};
use otel_arrow_dfe_admin_api::pipelines::{ErrorEvent, EventType, RequestEvent, SuccessEvent};
use otel_arrow_dfe_admin_api::telemetry::MetricsResponse;
use otel_arrow_dfe_contract_model::UiSignal;
use otel_arrow_dfe_contract_model::ui::{Events, Inputs, METRICS, SHIPPED_CATALOG, event_pipeline};
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

pub fn configure(config: &mut Value) {
    config["engine"]["telemetry"] = json!({"reporting_interval":"200ms","logs":{"level":"info","providers":{"admin":"its"},"tap":{"enabled":true}}});
    config["engine"]["observability"]["pipeline"] = json!({
        "nodes": {
            "telemetry":{"type":"receiver:internal_telemetry","config":{"signals":["logs"]}},
            "sink":{"type":"exporter:noop","config":{}}
        },
        "connections":[{"from":"telemetry","to":"sink"}]
    });
    config["policies"]["telemetry"] = json!({"runtime_metrics":"detailed"});
    let group = &mut config["groups"]["default"];
    group["topics"] = json!({"ui":{"impl_selection":"force_mixed"}});
    let pipelines = &mut group["pipelines"];
    let file = pipelines["main"]["nodes"]["out"].take();
    pipelines["main"]["nodes"]["out"] = json!({"type":"exporter:topic","config":{"topic":"ui"}});
    pipelines["reader"] = json!({"nodes":{"topic":{"type":"receiver:topic","config":{"topic":"ui"}},"out":file},
        "connections":[{"from":"topic","to":"out"}]});
}

pub async fn check(
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
    let charts = assets(&http, &base)
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
        let baseline = samples(&http, &base, &charts, None, rows).await?;
        workload.await.context("marked traffic delivery")?;
        let _ = samples(&http, &base, &charts, Some(&baseline), rows).await?;
        Ok(())
    }
    .await;
    let status = async {
        let status = get_json(&http, base.join(STATUS)?).await?;
        status_events(&status, &charts.events).context(STATUS)
    }
    .await;
    let status = observation(
        rows,
        "Status overlay",
        "Ready and Accepted conditions, Success.Ready",
        status,
    );
    let logs = tokio::time::timeout(
        Duration::from_secs(7),
        logs(&http, &base, &charts.events, rows),
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
    let lifecycle = lifecycle(&http, &base, occupied_port, &charts.events, rows).await;
    let failures: Vec<_> = [metrics, status, logs, lifecycle]
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

type Baselines = BTreeMap<String, BTreeMap<String, f64>>;

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

// Follow the entry point's local assets and ES module imports. A new API path used by
// one of those modules needs a wire probe before it can be reported as covered.
async fn assets(http: &Client, base: &Url) -> Result<Inputs> {
    let references = Regex::new(r#"(?:src|href)=["']([^"']+)["']"#)?;
    let imports = Regex::new(r#"(?:\bfrom\s+|\bimport\s*(?:\(\s*)?)["']([^"']+)["']"#)?;
    let routes = Regex::new(r"/api/v[0-9]+/[a-zA-Z0-9_/-]+")?;
    let mut pending = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut api = BTreeSet::new();
    let mut charts = None;
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
        if url.path() == "/static/js/chart-inputs.js" {
            let served = std::str::from_utf8(&bytes)?;
            ensure!(
                served == SHIPPED_CATALOG,
                "dashboard catalog differs from the candidate revision"
            );
            charts = Some(Inputs::parse(served)?);
        }
        if js && !url.path().starts_with("/static/vendor/") {
            let source = std::str::from_utf8(&bytes)?
                .lines()
                .filter(|s| !s.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            super::ui_readers::catalog_reads(&source)
                .with_context(|| format!("dashboard module {url}"))?;
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
    charts.context("dashboard modules did not load the chart input contract")
}

fn timestamp(v: &Value) -> bool {
    v.as_str()
        .is_some_and(|s| DateTime::parse_from_rfc3339(s).is_ok())
}

fn status_events(status: &Value, events: &Events) -> Result<()> {
    let pipeline = &status["pipelines"]["default:main"];
    ensure!(
        pipeline["runningCores"] == 1 && pipeline["totalCores"] == 1,
        "missing running main pipeline"
    );
    let conditions = pipeline["conditions"]
        .as_array()
        .context("missing pipeline conditions")?;
    for kind in events.conditions.values() {
        ensure!(
            conditions
                .iter()
                .any(|c| c["type"] == *kind && c["status"] == "True"),
            "missing {kind} condition"
        );
    }
    let cores = pipeline["cores"].as_object().context("missing cores")?;
    let events = cores
        .values()
        .filter_map(|c| c["recentEvents"].as_array())
        .flatten();
    ensure!(
        events.map(|e| &e["Engine"]).any(|e| timestamp(&e["time"])
            && e["type"]["Success"] == "Ready"
            && e["message"].is_string()),
        "missing renderable Ready event"
    );
    Ok(())
}

// Exercise the three status categories the overlay renders. The failed receiver
// binds a port held by the fixture for the lifetime of this deployment.
async fn lifecycle(
    http: &Client,
    base: &Url,
    occupied: u16,
    events: &Events,
    rows: &mut Vec<UiSignal>,
) -> Result<()> {
    let mut config = get_json(http, base.join("/api/v1/config")?).await?;
    let mut broken = config["groups"]["default"]["pipelines"]["observer"].clone();
    broken["nodes"]["input"]["config"]["protocol"]["tcp"]["listening_addr"] =
        json!(format!("127.0.0.1:{occupied}"));
    config["groups"]["default"]["pipelines"]["broken"] = broken;
    let reconcile = http
        .post(base.join("/api/v1/config/reconcile")?)
        .json(&json!({"config": config}))
        .send()
        .await?;
    // A fast bind failure can finish before the reconcile response is sent.
    if reconcile.status() == reqwest::StatusCode::CONFLICT {
        let body: Value = reconcile.json().await?;
        ensure!(
            body["changes"].as_array().is_some_and(|changes| changes
                .iter()
                .any(|c| c["pipelineId"] == "broken" && c["state"] == "failed")),
            "unexpected reconcile conflict: {body}"
        );
    } else {
        let _ = reconcile.error_for_status()?;
    }
    let _ = http
        .post(base.join("/api/v1/groups/default/pipelines/observer/shutdown")?)
        .json(&json!({}))
        .send()
        .await?
        .error_for_status()?;
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut pending = events.kinds.clone();
    while !pending.is_empty() {
        let status = get_json(http, base.join(STATUS)?).await?;
        pending.retain(|key, kind| {
            let Some(pipeline) = event_pipeline(key) else {
                return true;
            };
            let observed = observed_kind(&status["pipelines"][pipeline], kind);
            if observed {
                rows.push(UiSignal {
                    consumer: "Status overlay".into(),
                    signal: format!("{pipeline} {kind}"),
                    result: "declared and observed".into(),
                });
            }
            !observed
        });
        if Instant::now() >= deadline {
            break;
        }
        if !pending.is_empty() {
            sleep(Duration::from_millis(50)).await;
        }
    }
    for (key, kind) in &pending {
        rows.push(UiSignal {
            consumer: "Status overlay".into(),
            signal: format!(
                "{} {kind}",
                event_pipeline(key).unwrap_or("missing trigger")
            ),
            result: "expected event not observed".into(),
        });
    }
    ensure!(
        pending.is_empty(),
        "expected event not observed: {pending:?}"
    );
    Ok(())
}

fn observed_kind(pipeline: &Value, kind: &str) -> bool {
    pipeline["cores"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(_, c)| c["recentEvents"].as_array())
        .flatten()
        .any(|e| {
            let e = &e["Engine"];
            timestamp(&e["time"])
                && e["type"].get(kind).is_some()
                && matches!(
                    EventType::deserialize(&e["type"]),
                    Ok(EventType::Request(RequestEvent::ShutdownRequested)
                        | EventType::Success(SuccessEvent::Ready)
                        | EventType::Error(ErrorEvent::RuntimeError(_)))
                )
        })
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

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

fn log_entry(entry: &Value, fields: &BTreeMap<String, String>) -> bool {
    timestamp(&entry["timestamp"])
        && entry["seq"].is_u64()
        && fields
            .values()
            .all(|k| entry[k.as_str()].as_str().is_some_and(|s| !s.is_empty()))
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

pub async fn check_disabled_tap(root: &str) -> Result<()> {
    let mut ws = socket(&Url::parse(root)?).await?;
    let frame = receive(&mut ws, "error").await?;
    ensure!(
        frame["message"].as_str().is_some_and(|s| !s.is_empty()),
        "missing log panel diagnostic when the tap is disabled"
    );
    Ok(())
}

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
    ensure!(
        log_entry(&event, &events.log_fields)
            && event["event_name"]
                .as_str()
                .is_some_and(|s| s.contains(marker))
            && event["seq"].as_u64() > Some(cursor),
        "unusable live log: {event}"
    );
    // Today the live log view displays arbitrary event names. If a view starts
    // depending on a named event, require a declaration and actual emission.
    for name in &events.named_logs {
        let mut declared = false;
        for file in std::fs::read_dir("semconv/groups")? {
            let path = file?.path();
            if path.extension().is_some_and(|e| e == "yaml") {
                let doc: Value = serde_yaml::from_slice(&std::fs::read(path)?)?;
                declared |= doc["groups"].as_array().is_some_and(|groups| {
                    groups
                        .iter()
                        .any(|g| g["type"] == "event" && g["name"] == *name)
                });
            }
        }
        let observed = snapshot["logs"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(std::iter::once(&event))
            .any(|e| e["event_name"] == *name && log_entry(e, &events.log_fields));
        let result = if !declared {
            "missing event declaration"
        } else if !observed {
            "expected event not observed"
        } else {
            "declared and observed"
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
            .all(|r| r.result == "declared and observed"),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: an otherwise ready pipeline loses the recent-events field rendered by the overlay.
    /// Guarantees: passing health checks cannot supply missing lifecycle event evidence.
    #[test]
    fn healthy_status_requires_a_renderable_lifecycle_event() {
        let mut status = json!({"pipelines":{"default:main":{"runningCores":1,"totalCores":1,
            "conditions":[{"type":"Ready","status":"True"},{"type":"Accepted","status":"True"}],
            "cores":{"0":{"recentEvents":[{"Engine":{"time":"2026-01-01T00:00:00Z","type":{"Success":"Ready"},"message":"running"}}]}}}}});
        let inputs = Inputs::parse(include_str!("../../../admin/ui/js/chart-inputs.js")).unwrap();
        status_events(&status, &inputs.events).unwrap();
        status["pipelines"]["default:main"]["cores"]["0"] = json!({"events":[]});
        assert!(status_events(&status, &inputs.events).is_err());
        let wrong = json!({"cores":{"0":{"recentEvents":[{"Engine":{"time":"2026-01-01T00:00:00Z","type":{"Request":"StartRequested"}}}]}}});
        assert!(!observed_kind(&wrong, "Request"));
        assert!(!observed_kind(
            &status["pipelines"]["default:main"],
            "Success"
        ));
    }
}
