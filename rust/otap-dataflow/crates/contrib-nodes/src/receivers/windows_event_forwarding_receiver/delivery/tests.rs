// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::config::AuthConfig;
use super::*;
use futures::{FutureExt, poll};
use otel_arrow_dfe_channel::mpsc::{Channel, Receiver};
use otel_arrow_dfe_engine::{
    control::runtime_ctrl_msg_channel,
    local::message::LocalSender,
    message::Sender,
    testing::{test_node, test_pipeline_runtime_services},
};
use otel_arrow_dfe_otap::testing::{next_ack, next_nack};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use std::time::Duration;

struct Harness {
    delivery: Delivery,
    handler: EffectHandler<OtapPdata>,
    output: Receiver<OtapPdata>,
    path: String,
}

fn harness(max_sources: usize, max_in_flight: usize) -> Harness {
    let mut config: Config = serde_json::from_value(serde_json::json!({
        "endpoint": "127.0.0.1:5986",
        "public_endpoint": "https://collector",
        "tls": {"cert_file": "cert.pem", "key_file": "key.pem", "client_ca_files": ["ca.pem"]},
        "auth": {"allowed_sources": ["host.example.com"]},
        "subscriptions": [{"name": "audit", "query": "<QueryList/>"}]
    }))
    .unwrap();
    config.limits.max_sources = max_sources;
    config.limits.max_in_flight_batches = max_in_flight;
    let delivery = Delivery::from_config(&config, "wef").unwrap();
    let path = format!(
        "/wsman/subscriptions/{}/old-version",
        delivery.subscription_id
    );
    let (sender, output) = Channel::new(1);
    let (runtime, _) = runtime_ctrl_msg_channel(8);
    let (_, reporter) = MetricsReporter::create_new_and_receiver(1);
    let handler = EffectHandler::new(
        test_node("wef"),
        [("out".into(), Sender::Local(LocalSender::mpsc(sender)))].into(),
        Some("out".into()),
        runtime,
        reporter,
        test_pipeline_runtime_services(),
    );
    Harness {
        delivery,
        handler,
        output,
        path,
    }
}

fn source(name: &str) -> SourceIdentity {
    let cert = rcgen::generate_simple_self_signed(vec![name.into()]).unwrap();
    SourceIdentity::from_verified_certificate(
        cert.cert.der(),
        &AuthConfig {
            allowed_sources: vec![name.into()],
        },
    )
    .unwrap()
}

fn context<'a>(source: &'a SourceIdentity, path: &'a str) -> DeliveryContext<'a> {
    DeliveryContext {
        source,
        path,
        peer: "127.0.0.1:1234",
        observed_time_unix_nano: 123,
    }
}

fn event(marker: bool) -> String {
    let provider = if marker {
        "Microsoft-Windows-EventForwarder"
    } else {
        "Example"
    };
    let content = if marker {
        "<SubscriptionBookmarkEvent><SubscriptionId/></SubscriptionBookmarkEvent>"
    } else {
        "<EventData><Data Name=\"key\">value</Data></EventData>"
    };
    format!(
        r#"<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event"><System><Provider Name="{provider}"/><EventID>111</EventID><TimeCreated SystemTime="2026-09-22T19:28:11Z"/><Computer>untrusted</Computer></System>{content}</Event>"#
    )
}

fn soap(h: &Harness, events: &[String], bookmark: Option<&str>, ack: bool) -> String {
    let bookmark = bookmark
        .map(|b| format!("<w:Bookmark>{b}</w:Bookmark>"))
        .unwrap_or_default();
    let events: String = events.iter().map(|event| format!(
        r#"<w:Event Action="http://schemas.dmtf.org/wbem/wsman/1/wsman/Event"><![CDATA[{event}]]></w:Event>"#
    )).collect();
    let ack = if ack { "<w:AckRequested/>" } else { "" };
    format!(
        r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing" xmlns:w="http://schemas.dmtf.org/wbem/wsman/1/wsman.xsd" xmlns:e="http://schemas.xmlsoap.org/ws/2004/08/eventing"><s:Header><a:Action>{}</a:Action><a:MessageID>request</a:MessageID><a:To>https://collector{}</a:To><e:Identifier>{}</e:Identifier>{ack}{bookmark}</s:Header><s:Body><w:Events>{events}</w:Events></s:Body></s:Envelope>"#,
        Action::Events.uri(),
        h.path,
        h.delivery.subscription_id,
    )
}

fn envelope(xml: &str) -> Envelope<'_> {
    Envelope::parse(xml, xml.len(), 256).unwrap()
}

fn output(h: &mut Receiver<OtapPdata>) -> OtapPdata {
    h.recv()
        .now_or_never()
        .expect("delivery should already be enqueued")
        .unwrap()
}

fn ack(pdata: OtapPdata) -> AckMsg<OtapPdata> {
    next_ack(AckMsg::new(pdata))
        .expect("receiver Ack subscription")
        .1
}

fn saved(delivery: &Delivery, source: &SourceIdentity) -> Option<String> {
    delivery.bookmark(source).map(|b| b.as_xml().to_owned())
}

/// Scenario: a multi-event request is enqueued, then receives whole-batch Ack.
/// Guarantees: enqueue and feedback routing alone do not commit; success bytes follow commit and admission remains held until then.
#[tokio::test(start_paused = true)]
async fn commits_before_returning_soap_ack() {
    let mut h = harness(2, 2);
    let source = source("host.example.com");
    let context = context(&source, &h.path);
    let xml = soap(&h, &[event(false), event(false)], Some("first"), true);
    let marker = soap(&h, &[event(true)], Some("must-not-overtake"), true);
    let marker_envelope = envelope(&marker);
    let envelope = envelope(&xml);
    let mut delivery = Box::pin(h.delivery.deliver(&envelope, &context, &h.handler));
    assert!(poll!(&mut delivery).is_pending());
    let mut pdata = output(&mut h.output);
    assert_eq!(pdata.num_items(), 2);
    assert!(saved(&h.delivery, &source).is_none());
    assert!(matches!(
        h.delivery.deliver(&envelope, &context, &h.handler).await,
        Err(DeliveryError::Admission(_))
    ));
    assert!(matches!(
        h.delivery
            .deliver(&marker_envelope, &context, &h.handler)
            .await,
        Err(DeliveryError::Admission(_))
    ));
    let ack = ack(pdata);
    assert!(h.delivery.handle_ack(ack.clone()).unwrap());
    assert!(!h.delivery.handle_ack(ack).unwrap());
    assert!(saved(&h.delivery, &source).is_none());
    assert_eq!(h.delivery.state.borrow().pending.len(), 1);
    let response = delivery.await.unwrap();
    assert_eq!(&response[..2], &[0xff, 0xfe]);
    let text = String::from_utf16(
        &response[2..]
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    assert!(text.contains("<a:RelatesTo>request</a:RelatesTo>"));
    assert!(
        saved(&h.delivery, &source)
            .unwrap()
            .contains(">first</w:Bookmark>")
    );
    assert!(h.delivery.state.borrow().pending.is_empty());
}

/// Scenario: validated markers, ordinary events without a new bookmark, and permanent/temporary Nacks follow a committed cursor.
/// Guarantees: marker-only batches commit locally, ordinary batches still await feedback, and any Nack preserves progress.
#[tokio::test(start_paused = true)]
async fn preserves_progress_on_nack_and_missing_bookmark() {
    let mut h = harness(1, 1);
    let source = source("host.example.com");
    let context = context(&source, &h.path);
    let initial = soap(&h, &[event(true)], Some("initial"), true);
    assert!(
        h.delivery
            .deliver(&envelope(&initial), &context, &h.handler)
            .await
            .is_ok()
    );
    assert!(h.output.recv().now_or_never().is_none());
    let committed = saved(&h.delivery, &source).unwrap();
    for permanent in [false, true] {
        let xml = soap(&h, &[event(false)], Some("rejected"), true);
        let envelope = envelope(&xml);
        let mut delivery = Box::pin(h.delivery.deliver(&envelope, &context, &h.handler));
        assert!(poll!(&mut delivery).is_pending());
        let pdata = output(&mut h.output);
        let nack = if permanent {
            NackMsg::new_permanent("refused", pdata)
        } else {
            NackMsg::new("retry", pdata)
        };
        assert!(h.delivery.handle_nack(next_nack(nack).unwrap().1).unwrap());
        assert!(matches!(delivery.await, Err(DeliveryError::Downstream(_))));
        assert_eq!(
            saved(&h.delivery, &source).as_deref(),
            Some(committed.as_str())
        );
    }
    let xml = soap(&h, &[event(false)], None, true);
    let envelope = envelope(&xml);
    let mut delivery = Box::pin(h.delivery.deliver(&envelope, &context, &h.handler));
    assert!(poll!(&mut delivery).is_pending());
    assert!(h.delivery.handle_ack(ack(output(&mut h.output))).unwrap());
    assert!(delivery.await.is_ok());
    assert_eq!(saved(&h.delivery, &source).unwrap(), committed);
}

/// Scenario: conversion fails after admission or SOAP identity, destination, bookmark, and response validation fails.
/// Guarantees: no partial output or progress is published and failed conversion releases its in-flight slot.
#[tokio::test(start_paused = true)]
async fn rejects_invalid_deliveries_atomically() {
    let h = harness(1, 1);
    let source = source("host.example.com");
    let context = context(&source, &h.path);
    let valid = soap(&h, &[event(true)], Some("valid"), true);
    let bad_event = soap(&h, &[event(false), "<Event/>".into()], Some("bad"), true);
    assert!(matches!(
        h.delivery
            .deliver(&envelope(&bad_event), &context, &h.handler)
            .await,
        Err(DeliveryError::Conversion(_))
    ));
    for xml in [
        valid.replace("<w:AckRequested/>", ""),
        valid.replace(
            &format!("<e:Identifier>{}", h.delivery.subscription_id),
            "<e:Identifier>00000000-0000-0000-0000-000000000000",
        ),
        valid.replace("https://collector", "https://other"),
        valid.replace(
            "<w:Bookmark>valid</w:Bookmark>",
            "<w:Bookmark><A/><B/></w:Bookmark>",
        ),
        valid.replace(
            "<w:AckRequested/>",
            "<w:AckRequested/><w:MaxEnvelopeSize>1</w:MaxEnvelopeSize>",
        ),
    ] {
        assert!(matches!(
            h.delivery
                .deliver(&envelope(&xml), &context, &h.handler)
                .await,
            Err(DeliveryError::Request(_))
        ));
    }
    assert!(h.output.recv().now_or_never().is_none());
    assert!(saved(&h.delivery, &source).is_none());
    assert!(h.delivery.state.borrow().pending.is_empty());
    assert!(
        h.delivery
            .deliver(&envelope(&valid), &context, &h.handler)
            .await
            .is_ok()
    );
}

/// Scenario: an enqueued delivery times out or is cancelled, then its source retries.
/// Guarantees: capacity is released, stale and foreign feedback cannot commit the retry, and only its own Ack advances progress.
#[tokio::test(start_paused = true)]
async fn rejects_late_and_foreign_feedback() {
    let mut h = harness(1, 1);
    let source = source("host.example.com");
    let context = context(&source, &h.path);
    let xml = soap(&h, &[event(false)], Some("cursor"), true);
    let envelope = envelope(&xml);
    let mut expired = Box::pin(h.delivery.deliver(&envelope, &context, &h.handler));
    assert!(poll!(&mut expired).is_pending());
    let late = ack(output(&mut h.output));
    tokio::time::advance(h.delivery.limits.feedback_timeout).await;
    assert!(matches!(expired.await, Err(DeliveryError::Timeout)));
    let mut cancelled = Box::pin(h.delivery.deliver(&envelope, &context, &h.handler));
    assert!(poll!(&mut cancelled).is_pending());
    let stale = ack(output(&mut h.output));
    drop(cancelled);
    assert!(h.delivery.state.borrow().pending.is_empty());
    let mut cancelled_ack = Box::pin(h.delivery.deliver(&envelope, &context, &h.handler));
    assert!(poll!(&mut cancelled_ack).is_pending());
    assert!(h.delivery.handle_ack(ack(output(&mut h.output))).unwrap());
    drop(cancelled_ack);
    assert!(saved(&h.delivery, &source).is_none());
    let mut retry = Box::pin(h.delivery.deliver(&envelope, &context, &h.handler));
    assert!(poll!(&mut retry).is_pending());
    let current = ack(output(&mut h.output));
    assert!(!h.delivery.handle_ack(late).unwrap());
    assert!(!h.delivery.handle_ack(stale).unwrap());
    let other = harness(1, 1);
    assert!(matches!(
        other.delivery.handle_ack(current.clone()),
        Err(DeliveryError::InvalidFeedback)
    ));
    let mut invalid = current.clone();
    invalid.unwind.route.calldata.clear();
    assert!(matches!(
        h.delivery.handle_ack(invalid),
        Err(DeliveryError::InvalidFeedback)
    ));
    assert!(poll!(&mut retry).is_pending());
    assert!(saved(&h.delivery, &source).is_none());
    assert!(h.delivery.handle_ack(current).unwrap());
    assert!(retry.await.is_ok());
    assert!(saved(&h.delivery, &source).is_some());
}

/// Scenario: one batch occupies the output queue while another waits to enqueue.
/// Guarantees: global admission is bounded; the same deadline bounds enqueue and feedback, with no hidden post-timeout send.
#[tokio::test(start_paused = true)]
async fn bounds_backpressured_enqueue() {
    let mut h = harness(3, 2);
    let a = source("a.example.com");
    let b = source("b.example.com");
    let c = source("c.example.com");
    let ca = context(&a, &h.path);
    let cb = context(&b, &h.path);
    let cc = context(&c, &h.path);
    let xml = soap(&h, &[event(false)], Some("cursor"), true);
    let envelope = envelope(&xml);
    let mut first = Box::pin(h.delivery.deliver(&envelope, &ca, &h.handler));
    assert!(poll!(&mut first).is_pending());
    let mut blocked = Box::pin(h.delivery.deliver(&envelope, &cb, &h.handler));
    assert!(poll!(&mut blocked).is_pending());
    assert!(matches!(
        h.delivery.deliver(&envelope, &cc, &h.handler).await,
        Err(DeliveryError::Admission(_))
    ));
    tokio::time::advance(Duration::from_secs(30)).await;
    // Make enqueue ready at the expired deadline, before polling the blocked future.
    let _ = output(&mut h.output);
    assert!(matches!(blocked.await, Err(DeliveryError::Timeout)));
    assert!(matches!(first.await, Err(DeliveryError::Timeout)));
    assert!(h.output.recv().now_or_never().is_none());
    assert!(h.delivery.state.borrow().pending.is_empty());
    assert!(saved(&h.delivery, &a).is_none());
    assert!(saved(&h.delivery, &b).is_none());
}

/// Scenario: one source finishes while another spends part of its delivery budget waiting to enqueue.
/// Guarantees: sources complete independently and successful enqueue does not restart the feedback deadline.
#[tokio::test(start_paused = true)]
async fn shares_one_deadline_across_enqueue_and_feedback() {
    let mut h = harness(2, 2);
    let a = source("a.example.com");
    let b = source("b.example.com");
    let ca = context(&a, &h.path);
    let cb = context(&b, &h.path);
    let xml = soap(&h, &[event(false)], Some("cursor"), true);
    let envelope = envelope(&xml);
    let mut first = Box::pin(h.delivery.deliver(&envelope, &ca, &h.handler));
    assert!(poll!(&mut first).is_pending());
    let mut second = Box::pin(h.delivery.deliver(&envelope, &cb, &h.handler));
    assert!(poll!(&mut second).is_pending());
    tokio::time::advance(Duration::from_secs(20)).await;
    assert!(h.delivery.handle_ack(ack(output(&mut h.output))).unwrap());
    assert!(first.await.is_ok());
    assert!(poll!(&mut second).is_pending());
    let late = ack(output(&mut h.output));
    assert!(saved(&h.delivery, &a).is_some());
    assert!(saved(&h.delivery, &b).is_none());
    tokio::time::advance(Duration::from_secs(10)).await;
    // Route feedback before the expired future is polled, exercising the boundary.
    assert!(h.delivery.handle_ack(late).unwrap());
    assert!(matches!(second.await, Err(DeliveryError::Timeout)));
    assert!(saved(&h.delivery, &b).is_none());
}

/// Scenario: Ack is queued before the deadline but the request resumes only at its deadline.
/// Guarantees: queued success cannot commit after expiry, and the admission is released.
#[tokio::test(start_paused = true)]
async fn queued_ack_cannot_commit_after_deadline() {
    let mut h = harness(1, 1);
    let source = source("host.example.com");
    let context = context(&source, &h.path);
    let xml = soap(&h, &[event(false)], Some("cursor"), true);
    let envelope = envelope(&xml);
    let mut delivery = Box::pin(h.delivery.deliver(&envelope, &context, &h.handler));
    assert!(poll!(&mut delivery).is_pending());
    assert!(h.delivery.handle_ack(ack(output(&mut h.output))).unwrap());
    tokio::time::advance(h.delivery.limits.feedback_timeout).await;
    assert!(matches!(delivery.await, Err(DeliveryError::Timeout)));
    assert!(saved(&h.delivery, &source).is_none());
    assert!(h.delivery.state.borrow().pending.is_empty());
}

/// Scenario: shutdown arrives with a feedback waiter, a blocked enqueue, or an Ack queued but not consumed.
/// Guarantees: shutdown promptly releases every admission, prevents commits, and permanently closes ingress.
#[tokio::test(start_paused = true)]
async fn shutdown_rejects_pending_work() {
    for queued_ack in [false, true] {
        let mut h = harness(2, 2);
        let a = source("a.example.com");
        let b = source("b.example.com");
        let ca = context(&a, &h.path);
        let cb = context(&b, &h.path);
        let xml = soap(&h, &[event(false)], Some("cursor"), true);
        let envelope = envelope(&xml);
        let mut first = Box::pin(h.delivery.deliver(&envelope, &ca, &h.handler));
        assert!(poll!(&mut first).is_pending());
        let mut second = Box::pin(h.delivery.deliver(&envelope, &cb, &h.handler));
        assert!(poll!(&mut second).is_pending());
        if queued_ack {
            assert!(h.delivery.handle_ack(ack(output(&mut h.output))).unwrap());
        }
        h.delivery.shutdown().unwrap();
        h.delivery.shutdown().unwrap();
        assert!(h.delivery.state.borrow().pending.is_empty());
        assert!(matches!(
            first.now_or_never(),
            Some(Err(DeliveryError::Shutdown))
        ));
        assert!(matches!(
            second.now_or_never(),
            Some(Err(DeliveryError::Shutdown))
        ));
        assert!(matches!(
            h.delivery.deliver(&envelope, &ca, &h.handler).await,
            Err(DeliveryError::Shutdown)
        ));
        assert!(saved(&h.delivery, &a).is_none());
        assert!(saved(&h.delivery, &b).is_none());
    }
}

/// Scenario: the output channel closes before delivery and the retained-source bound is reached by that failed admission.
/// Guarantees: enqueue fails without progress, a different source is rejected, and the original source may retry.
#[tokio::test(start_paused = true)]
async fn closed_output_preserves_source_admission() {
    let h = harness(1, 1);
    let a = source("a.example.com");
    let b = source("b.example.com");
    let xml = soap(&h, &[event(false)], Some("cursor"), true);
    let envelope = envelope(&xml);
    drop(h.output);
    for source in [&a, &b, &a] {
        let result = h
            .delivery
            .deliver(&envelope, &context(source, &h.path), &h.handler)
            .await;
        if source == &b {
            assert!(matches!(result, Err(DeliveryError::Admission(_))));
        } else {
            assert!(matches!(result, Err(DeliveryError::Downstream(_))));
        }
    }
    assert!(h.delivery.state.borrow().pending.is_empty());
    assert!(saved(&h.delivery, &a).is_none());
}
