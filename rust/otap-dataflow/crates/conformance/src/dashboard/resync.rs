// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reproduce a slow log viewer, then use the recovery cursor sent by the engine.
use super::{Events, STREAM, Socket, command, get_json, log_entry, receive};
use anyhow::{Context, Result, ensure};
use futures_util::{StreamExt, TryStreamExt, stream};
use reqwest::{Client, StatusCode, Url};
use serde_json::{Value, json};
use tokio::net::TcpSocket;
use tokio_tungstenite::{MaybeTlsStream, client_async, tungstenite::client::IntoClientRequest};

const LOGS: &str = "/api/v1/telemetry/logs";
const GROUP: &str = "conformance-log-resync";

// Repeating an ordinary create request produces one named audit event per request.
// Only the first create changes state. Limit both requests in flight and retained data.
async fn emit(http: &Client, base: &Url, count: usize) -> Result<()> {
    let url = base.join(&format!("/api/v1/groups/{GROUP}"))?;
    stream::iter(0..count)
        .map(|_| async {
            let response = http.post(url.clone()).json(&json!({})).send().await?;
            ensure!(
                matches!(
                    response.status(),
                    StatusCode::CREATED | StatusCode::CONFLICT
                ),
                "fixture group request: {}",
                response.status()
            );
            let _ = response.bytes().await?;
            Ok::<_, anyhow::Error>(())
        })
        .buffer_unordered(8)
        .try_collect::<()>()
        .await?;
    Ok(())
}

async fn slow_socket(base: &Url) -> Result<Socket> {
    let addr = base
        .socket_addrs(|| None)?
        .into_iter()
        .next()
        .context("missing admin address")?;
    let tcp = if addr.is_ipv4() {
        TcpSocket::new_v4()?
    } else {
        TcpSocket::new_v6()?
    };
    tcp.set_recv_buffer_size(32 * 1024)?;
    let mut url = base.join(STREAM)?;
    url.set_scheme("ws")
        .map_err(|()| anyhow::anyhow!("invalid socket URL"))?;
    let mut request = url.as_str().into_client_request()?;
    let _ = request
        .headers_mut()
        .insert("origin", base.origin().ascii_serialization().parse()?);
    Ok(
        client_async(request, MaybeTlsStream::Plain(tcp.connect(addr).await?))
            .await?
            .0,
    )
}

async fn frame(ws: &mut Socket) -> Result<Value> {
    let msg = ws
        .next()
        .await
        .context("log stream ended during resync")??;
    Ok(serde_json::from_str(msg.to_text()?)?)
}

/// Make a slow subscriber miss retained events. Use the reported lag cursor to
/// retrieve the skipped entries through the same paged backfill the viewer uses.
pub(super) async fn check(http: &Client, base: &Url, events: &Events) -> Result<()> {
    // Seed a full 100-entry backfill page before applying socket backpressure.
    emit(http, base, 128).await?;
    let mut ws = slow_socket(base).await?;
    command(&mut ws, json!({"type":"subscribe","limit":100})).await?;
    let snapshot = receive(&mut ws, "snapshot").await?;
    let last = snapshot["next_seq"]
        .as_u64()
        .context("missing initial cursor")?;
    let _ = receive(&mut ws, "state").await?;

    // Backpressure the outbound WebSocket with historical pages, without advancing
    // the live cursor. Pause would drain the broadcast receiver and hide real lag.
    for _ in 0..160 {
        command(&mut ws, json!({"type":"backfill","after":0,"limit":100})).await?;
    }
    command(&mut ws, json!({"type":"ping"})).await?;
    // Exceed the live broadcast buffer (512) while staying inside the configured
    // retention window (2048), so the lost events are still retrievable.
    emit(http, base, 768).await?;
    let (mut cursor, first_after_gap) = observe_gap(&mut ws, last, events).await?;
    let mut recovered_fixture_event = false;
    // The dashboard requests at most 100 records on each resync/backfill.
    while cursor < first_after_gap - 1 {
        command(
            &mut ws,
            json!({"type":"backfill","after":cursor,"limit":100}),
        )
        .await?;
        let page = receive(&mut ws, "snapshot").await?;
        let retained = get_json(
            http,
            base.join(&format!("{LOGS}?after={cursor}&limit=100"))?,
        )
        .await?;
        let (next, marked) = recovery_page(&page, &retained, cursor, first_after_gap, events)?;
        cursor = next;
        recovered_fixture_event |= marked;
    }
    ensure!(
        recovered_fixture_event,
        "no skipped fixture event recovered"
    );
    Ok(())
}

/// Return the last sequence before a reported gap and the first delivered after it. An unannounced skip fails.
async fn observe_gap(ws: &mut Socket, mut last: u64, events: &Events) -> Result<(u64, u64)> {
    let mut barrier = false;
    let mut lag_reported = false;
    let mut gap = None;
    while !barrier || gap.is_none() {
        let f = frame(ws).await?;
        match f["type"].as_str() {
            Some("snapshot") => ensure!(
                f["next_seq"].as_u64().is_some_and(|s| s <= last),
                "historical backfill advanced past live cursor"
            ),
            Some("pong") => barrier = true,
            Some("error") => {
                let cursor = f[&events.lag_cursor]
                    .as_u64()
                    .with_context(|| format!("missing {}", events.lag_cursor))?;
                ensure!(
                    cursor == last && f["message"].as_str().is_some_and(|s| !s.is_empty()),
                    "unusable lag cursor {cursor}, last delivered {last}"
                );
                lag_reported = true;
            }
            Some("log") => {
                let seq = f["seq"].as_u64().context("missing live seq")?;
                let skipped = seq
                    .checked_sub(last)
                    .context("live cursor went backwards")?;
                if lag_reported {
                    ensure!(skipped > 1, "lag error without a gap");
                    let _ = gap.get_or_insert((last, seq));
                    lag_reported = false;
                } else {
                    ensure!(
                        skipped == 1,
                        "live stream skipped a sequence without reporting lag"
                    );
                }
                last = seq;
            }
            other => anyhow::bail!("unexpected log frame {other:?}"),
        }
    }
    gap.context("no broadcast lag observed")
}

/// Check a WebSocket backfill page against HTTP retention and require consecutive
/// sequence numbers. Return the next cursor and whether the page recovered a missed fixture event.
fn recovery_page(
    page: &Value,
    retained: &Value,
    mut cursor: u64,
    first_after_gap: u64,
    events: &Events,
) -> Result<(u64, bool)> {
    let mut recovered_fixture_event = false;
    let logs = page["logs"].as_array().context("missing recovery page")?;
    let end = logs
        .last()
        .and_then(|e| e["seq"].as_u64())
        .context("empty or unsequenced recovery page")?;
    ensure!(
        logs.len() <= 100
            && page["truncated_before_seq"].is_null()
            && page["logs"] == retained["logs"]
            && page["next_seq"] == retained["next_seq"]
            && page["next_seq"] == end,
        "backfill differs from the retained log page"
    );
    let name = events
        .log_fields
        .get("name")
        .context("no log event name in catalog")?;
    let rendered = events
        .log_fields
        .get("rendered")
        .context("no rendered log field in catalog")?;
    for entry in logs {
        cursor = cursor.checked_add(1).context("invalid log cursor")?;
        ensure!(
            log_entry(entry, &events.log_fields) && entry["seq"] == cursor,
            "unusable or discontinuous recovery page"
        );
        recovered_fixture_event |= cursor < first_after_gap
            && entry[name] == "group.create.requested"
            && entry[rendered].as_str().is_some_and(|s| s.contains(GROUP));
    }
    Ok((cursor, recovered_fixture_event))
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_conformance_model::ui::{Inputs, SIGNAL_CATALOG};

    /// Scenario: a backfill page agrees with retention but omits one of the skipped sequence numbers.
    /// Guarantees: both continuity from the lag cursor and a renderable missed fixture event are observed.
    #[test]
    fn recovery_checks_each_missing_sequence() {
        let events = Inputs::parse(SIGNAL_CATALOG).unwrap().events;
        let entry = |seq| {
            json!({"seq":seq,"event_name":"group.create.requested","target":"admin",
            "level":"INFO","rendered":GROUP,"timestamp":"2026-01-01T00:00:00Z"})
        };
        let page = json!({"next_seq":12,"logs":[entry(11), entry(12)]});
        assert_eq!(
            recovery_page(&page, &page, 10, 13, &events).unwrap(),
            (12, true)
        );
        let hole = json!({"next_seq":12,"logs":[entry(12)]});
        assert!(recovery_page(&hole, &hole, 10, 13, &events).is_err());
        assert!(recovery_page(&page, &hole, 10, 13, &events).is_err());
        assert_eq!(
            recovery_page(&page, &page, 10, 11, &events).unwrap(),
            (12, false)
        );
    }
}
