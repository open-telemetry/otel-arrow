// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Observe pipeline condition values and their triggering events on the status API.
use super::{STATUS, get_json, observation, timestamp};
use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_admin_api::pipelines::{ErrorEvent, EventType, RequestEvent, SuccessEvent};
use otel_arrow_dfe_conformance_model::{
    UiSignal,
    ui::{Events, PENDING, UNHEALTHY, event_pipeline},
};
use reqwest::{Client, Url};
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::{Instant, sleep};

/// Require running core counts, positive catalog conditions and a renderable Ready event for `main`.
pub(super) fn healthy(status: &Value, events: &Events, rows: &mut Vec<UiSignal>) -> Result<()> {
    let pipeline = &status["pipelines"]["default:main"];
    ensure!(
        pipeline["runningCores"] == 1 && pipeline["totalCores"] == 1,
        "missing running main pipeline"
    );
    let conditions = pipeline["conditions"]
        .as_array()
        .context("missing pipeline conditions")?;
    let condition_rows = rows.len();
    for kind in events.conditions.values() {
        let found = conditions
            .iter()
            .any(|c| c["type"] == *kind && c["status"] == "True");
        rows.push(UiSignal {
            consumer: "Status overlay".into(),
            signal: format!("condition {kind}"),
            result: if found {
                "observed"
            } else {
                "expected condition not observed"
            }
            .into(),
        });
    }
    ensure!(
        rows[condition_rows..]
            .iter()
            .all(|r| r.result == "observed"),
        "missing dashboard status condition"
    );
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

/// Observe a new pipeline while pending and after its receiver fails to bind.
/// Shutting down the observer supplies the request event alongside the ready and error events.
pub(super) async fn lifecycle(
    http: &Client,
    base: &Url,
    occupied: u16,
    events: &Events,
    rows: &mut Vec<UiSignal>,
) -> Result<()> {
    let config = get_json(http, base.join("/api/v1/config")?).await?;
    let mut broken = config["groups"]["default"]["pipelines"]["observer"].clone();
    broken["nodes"]["input"]["config"]["protocol"]["tcp"]["listening_addr"] =
        json!(format!("127.0.0.1:{occupied}"));
    let (armed, wait_for_observer) = tokio::sync::oneshot::channel();
    let create = async {
        wait_for_observer
            .await
            .context("status observer failed to arm")?;
        let _ = http
            .put(base.join("/api/v1/groups/default/pipelines/broken?wait=false")?)
            .json(&json!({"pipeline":broken}))
            .send()
            .await?
            .error_for_status()?;
        Ok::<(), anyhow::Error>(())
    };
    // Observe absence before allowing the create, then keep a status request in
    // flight during admission. A rollout acknowledgement alone is insufficient.
    let observe_pending = async {
        let before = get_json(http, base.join(STATUS)?).await?;
        ensure!(
            before["pipelines"]["default:broken"].is_null(),
            "failed receiver fixture already exists"
        );
        armed
            .send(())
            .map_err(|()| anyhow::anyhow!("create request cancelled"))?;
        let deadline = Instant::now() + Duration::from_secs(4);
        loop {
            let status = get_json(http, base.join(STATUS)?).await?;
            let p = &status["pipelines"]["default:broken"];
            if !p.is_null() {
                ensure!(
                    events
                        .pipeline_states
                        .pending
                        .matches(p, &events.conditions),
                    "pending status not observed before admission: cores={}/{}, conditions={}",
                    p["runningCores"],
                    p["totalCores"],
                    p["conditions"]
                );
                return Ok(());
            }
            ensure!(Instant::now() < deadline, "new pipeline absent from status");
        }
    };
    let (created, pending_state) = tokio::join!(create, observe_pending);
    created?;
    let pending_state = observation(rows, "Status overlay", PENDING, pending_state);
    let _ = http
        .post(base.join("/api/v1/groups/default/pipelines/observer/shutdown")?)
        .json(&json!({}))
        .send()
        .await?
        .error_for_status()?;
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut pending = events.kinds.clone();
    let mut unhealthy = false;
    while !pending.is_empty() || !unhealthy {
        let status = get_json(http, base.join(STATUS)?).await?;
        let broken = &status["pipelines"]["default:broken"];
        unhealthy |= events
            .pipeline_states
            .unhealthy
            .matches(broken, &events.conditions)
            && observed_kind(broken, "Error");
        pending.retain(|key, kind| {
            let Some(pipeline) = event_pipeline(key) else {
                return true;
            };
            let observed = observed_kind(&status["pipelines"][pipeline], kind);
            if observed {
                rows.push(UiSignal {
                    consumer: "Status overlay".into(),
                    signal: format!("{pipeline} {kind}"),
                    result: "observed".into(),
                });
            }
            !observed
        });
        if Instant::now() >= deadline {
            break;
        }
        if !pending.is_empty() || !unhealthy {
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
    observation(
        rows,
        "Status overlay",
        UNHEALTHY,
        unhealthy.then_some(()).context(
            "expected unhealthy conditions and receiver runtime error not observed together",
        ),
    )?;
    pending_state?;
    ensure!(
        pending.is_empty(),
        "expected event not observed: {pending:?}"
    );
    Ok(())
}

/// Match the specific lifecycle transition triggered by the fixture in a core's timestamped event history.
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

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_conformance_model::ui::{Inputs, SIGNAL_CATALOG};

    /// Scenario: an otherwise ready pipeline loses the recent-events field rendered by the overlay.
    /// Guarantees: passing health checks cannot supply missing lifecycle event evidence.
    #[test]
    fn healthy_status_requires_a_renderable_lifecycle_event() {
        let mut status = json!({"pipelines":{"default:main":{"runningCores":1,"totalCores":1,
            "conditions":[{"type":"Ready","status":"True"},{"type":"Accepted","status":"True"}],
            "cores":{"0":{"recentEvents":[{"Engine":{"time":"2026-01-01T00:00:00Z","type":{"Success":"Ready"},"message":"running"}}]}}}}});
        let inputs = Inputs::parse(SIGNAL_CATALOG).unwrap();
        healthy(&status, &inputs.events, &mut Vec::new()).unwrap();
        status["pipelines"]["default:main"]["cores"]["0"] = json!({"events":[]});
        assert!(healthy(&status, &inputs.events, &mut Vec::new()).is_err());
        let wrong = json!({"cores":{"0":{"recentEvents":[{"Engine":{"time":"2026-01-01T00:00:00Z","type":{"Request":"StartRequested"}}}]}}});
        assert!(!observed_kind(&wrong, "Request"));
        assert!(!observed_kind(
            &status["pipelines"]["default:main"],
            "Success"
        ));
    }
}
