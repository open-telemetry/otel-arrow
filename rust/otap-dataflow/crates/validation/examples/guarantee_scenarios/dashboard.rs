// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exercise the same-origin wire contract of the shipped admin dashboard.
use anyhow::{Context, Result, bail, ensure};
use chrono::DateTime;
use futures_util::{SinkExt, StreamExt};
use otel_arrow_dfe_admin_api::telemetry::MetricsResponse;
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
const METRICS: [&str; 2] = ["/api/v1/telemetry/metrics", "/api/v1/metrics"];
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

pub async fn check(root: &str) -> Result<()> {
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
    let status = get_json(&http, base.join(STATUS)?).await?;
    status_events(&status).context(STATUS)?;
    for path in METRICS {
        let mut url = base.join(path)?;
        url.set_query(Some(QUERY));
        let deadline = Instant::now() + Duration::from_secs(7);
        loop {
            let data = get_json(&http, url.clone()).await?;
            match metrics(&data, &charts) {
                Ok(()) => break,
                Err(e) if Instant::now() >= deadline => return Err(e.context(url.to_string())),
                Err(_) => sleep(Duration::from_millis(100)).await,
            }
        }
    }
    tokio::time::timeout(Duration::from_secs(7), logs(&http, &base))
        .await
        .context("dashboard timed out waiting for a log-stream response")?
        .context(STREAM)
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
async fn assets(http: &Client, base: &Url) -> Result<ChartInputs> {
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
            charts = Some(chart_inputs(std::str::from_utf8(&bytes)?)?);
        }
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
    charts.context("dashboard modules did not load the chart input contract")
}

fn timestamp(v: &Value) -> bool {
    v.as_str()
        .is_some_and(|s| DateTime::parse_from_rfc3339(s).is_ok())
}

fn status_events(status: &Value) -> Result<()> {
    let pipeline = &status["pipelines"]["default:main"];
    ensure!(
        pipeline["runningCores"] == 1 && pipeline["totalCores"] == 1,
        "missing running main pipeline"
    );
    let conditions = pipeline["conditions"]
        .as_array()
        .context("missing pipeline conditions")?;
    for kind in ["Ready", "Accepted"] {
        ensure!(
            conditions
                .iter()
                .any(|c| c["type"] == kind && c["status"] == "True"),
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

// The UI module exports a JSON object so the wire probe and the chart readers
// use the same names without a browser or JavaScript runtime dependency.
type ChartInputs = BTreeMap<String, BTreeMap<String, String>>;

fn chart_inputs(source: &str) -> Result<ChartInputs> {
    let (_, object) = source
        .split_once("export const chartInputs =")
        .context("missing chart input declaration")?;
    let charts: ChartInputs = serde_json::from_str(object.trim().trim_end_matches(';'))?;
    for scope in [
        "engine",
        "pipeline",
        "tokio.runtime",
        "pipeline.runtime_control",
        "pipeline.completion",
    ] {
        ensure!(
            charts.get(scope).is_some_and(|inputs| !inputs.is_empty()),
            "missing chart inputs for {scope}"
        );
    }
    Ok(charts)
}

fn attr<'a>(attrs: &'a Value, key: &str) -> Option<&'a str> {
    attrs[key]
        .as_str()
        .or_else(|| attrs[key]["String"].as_str())
}

fn metrics(data: &Value, charts: &ChartInputs) -> Result<()> {
    ensure!(timestamp(&data["timestamp"]), "invalid metrics timestamp");
    let sets = data["metric_sets"]
        .as_array()
        .context("missing metric_sets")?;
    let mut names = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut sender = BTreeSet::new();
    let mut receiver = BTreeSet::new();
    let mut publishers = BTreeSet::new();
    let mut subscribers = BTreeSet::new();
    let mut failures = BTreeSet::new();
    let mut queue = BTreeSet::new();
    let mut scoped_core = false;
    for set in sets {
        let scope = set["name"].as_str().context("unnamed metric set")?;
        let attrs = &set["attributes"];
        let pipeline = attr(attrs, "pipeline.id");
        if scope != "engine" && attr(attrs, "pipeline.group.id") != Some("default") {
            continue;
        }
        if let Some(topic) = attr(attrs, "topic").filter(|t| !t.is_empty()) {
            let key = (topic, attr(attrs, "process.instance.id"));
            match (scope, pipeline, attr(attrs, "node.id")) {
                ("exporter.topic", Some("main"), Some("out")) => {
                    let _ = publishers.insert(key);
                }
                ("receiver.topic", Some("reader"), Some("topic")) => {
                    let _ = subscribers.insert(key);
                }
                _ => {}
            }
        }
        if scope != "engine" && pipeline != Some("main") {
            continue;
        }
        if scope == "pipeline" && attrs["core.id"]["UInt"].is_u64() {
            scoped_core = true;
        }
        let channel = attr(attrs, "channel.id");
        for m in set["metrics"].as_array().context("missing metrics")? {
            let Some(value) = m["value"].as_f64() else {
                continue;
            };
            let name = m["name"].as_str().context("unnamed metric")?;
            let _ = names.entry(scope).or_default().insert(name);
            if attr(attrs, "channel.kind") != Some("pdata") {
                continue;
            }
            let Some(channel) = channel else { continue };
            match (scope, attr(attrs, "node.id"), name) {
                ("channel.sender", Some("input"), "messages")
                    if value > 0.0 && attr(&m["attributes"], "outcome") == Some("success") =>
                {
                    let _ = sender.insert(channel);
                }
                ("channel.sender", Some("input"), "failures") => {
                    let _ = failures.insert(attr(&m["attributes"], "error.type"));
                }
                ("channel.receiver", Some("out"), "messages") if value > 0.0 => {
                    let _ = receiver.insert(channel);
                }
                ("channel.receiver", Some("out"), "capacity" | "queue.depth") => {
                    let _ = queue.insert(name);
                }
                _ => {}
            }
        }
    }
    for (scope, instruments) in charts {
        for (key, name) in instruments {
            let available = match (scope.as_str(), key.as_str()) {
                ("engine", "memoryCommit") => cfg!(windows),
                ("pipeline", "memoryUsage" | "memoryAllocatedDelta" | "memoryFreedDelta") => {
                    !cfg!(windows)
                }
                (
                    "pipeline",
                    "contextSwitchesVoluntary"
                    | "contextSwitchesInvoluntary"
                    | "pageFaultsMinor"
                    | "pageFaultsMajor",
                ) => cfg!(any(
                    target_os = "linux",
                    target_os = "freebsd",
                    target_os = "openbsd"
                )),
                (
                    "tokio.runtime",
                    "workerBusyTime" | "workerParkCount" | "workerParkUnparkCount",
                ) => cfg!(target_has_atomic = "64"),
                _ => true,
            };
            if !available {
                continue;
            }
            ensure!(
                names
                    .get(scope.as_str())
                    .is_some_and(|n| n.contains(name.as_str())),
                "missing numeric {scope}.{name}"
            );
        }
    }
    ensure!(
        scoped_core
            && !sender.is_disjoint(&receiver)
            && failures.contains(&Some("closed"))
            && failures.contains(&Some("full"))
            && queue.len() == 2,
        "missing DAG traffic, failures or queue metrics for main/input -> out"
    );
    ensure!(
        !publishers.is_disjoint(&subscribers),
        "missing topic endpoints for main -> reader"
    );
    // Dynamic node tables use the snapshot metadata, including distribution summaries.
    let _ = MetricsResponse::deserialize(data).context("unusable dashboard metrics snapshot")?;
    Ok(())
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

fn log_entry(entry: &Value) -> bool {
    timestamp(&entry["timestamp"])
        && entry["seq"].is_u64()
        && ["level", "target", "event_name", "rendered"]
            .iter()
            .all(|k| entry[k].as_str().is_some_and(|s| !s.is_empty()))
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

async fn logs(http: &Client, base: &Url) -> Result<()> {
    let mut ws = socket(base).await?;
    command(&mut ws,json!({"type":"subscribe","after":null,"limit":100,"searchQuery":null,"minimumTimestamp":null,"minimumLevel":null,"paused":false})).await?;
    let snapshot = receive(&mut ws, "snapshot").await?;
    let cursor = snapshot["next_seq"]
        .as_u64()
        .context("missing snapshot cursor")?;
    ensure!(
        snapshot["logs"]
            .as_array()
            .is_some_and(|entries| entries.iter().all(log_entry)),
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
        log_entry(&event)
            && event["event_name"]
                .as_str()
                .is_some_and(|s| s.contains(marker))
            && event["seq"].as_u64() > Some(cursor),
        "unusable live log: {event}"
    );
    command(
        &mut ws,
        json!({"type":"backfill","after":cursor,"limit":100}),
    )
    .await?;
    let backfill = receive(&mut ws, "snapshot").await?;
    ensure!(
        backfill["logs"].as_array().is_some_and(|logs| logs
            .iter()
            .any(|l| log_entry(l) && l["seq"] == event["seq"])),
        "backfill lost the live event"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: the server returns HTTP success and numeric metrics under an obsolete name.
    /// Guarantees: a chart requesting the new name fails with the exact missing instrument.
    #[test]
    fn a_missing_chart_input_fails_the_wire_check() {
        let charts = BTreeMap::from([(
            "engine".into(),
            BTreeMap::from([("cpu".into(), "new.cpu".into())]),
        )]);
        let snapshot = json!({"timestamp":"2026-01-01T00:00:00Z","metric_sets":[
            {"name":"engine","metrics":[{"name":"old.cpu","value":0.4}]}
        ]});
        assert!(
            metrics(&snapshot, &charts)
                .unwrap_err()
                .to_string()
                .contains("engine.new.cpu")
        );
    }

    /// Scenario: an otherwise ready pipeline loses the recent-events field rendered by the overlay.
    /// Guarantees: passing health checks cannot supply missing lifecycle event evidence.
    #[test]
    fn healthy_status_requires_a_renderable_lifecycle_event() {
        let mut status = json!({"pipelines":{"default:main":{"runningCores":1,"totalCores":1,
            "conditions":[{"type":"Ready","status":"True"},{"type":"Accepted","status":"True"}],
            "cores":{"0":{"recentEvents":[{"Engine":{"time":"2026-01-01T00:00:00Z","type":{"Success":"Ready"},"message":"running"}}]}}}}});
        status_events(&status).unwrap();
        status["pipelines"]["default:main"]["cores"]["0"] = json!({"events":[]});
        assert!(status_events(&status).is_err());
    }
}
