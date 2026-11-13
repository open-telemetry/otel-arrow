// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration coverage for cooperative codec conversions through engine effect handlers.

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::local::exporter::EffectHandler;
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata, PdataEffectHandlerExtension};
use otel_arrow_dfe_pdata_codec::builtins::syslog::{SyslogBatchBuilder, resolve_syslog};
use otel_arrow_dfe_pdata_codec::{
    AcceptedEncodings, BatchSettings, CodecService, EncodePolicy, EncodedPdataBatch, EncodingPlan,
    OtapPayload, PdataEncoding, PdataView,
};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use std::future::Future;
use std::task::{Context as TaskContext, Poll, Waker};

const SYSLOG_CEF_ENCODING: PdataEncoding = PdataEncoding::new("syslog-batch");

fn handler() -> EffectHandler<OtapPdata> {
    let (_, metrics) = MetricsReporter::create_new_and_receiver(1);
    EffectHandler::new(
        otel_arrow_dfe_engine::testing::test_node("cooperative-codec"),
        metrics,
        otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
    )
}

fn batch(messages: &[&[u8]], observed_time: i64) -> Bytes {
    let mut builder = SyslogBatchBuilder::with_max_items(messages.len());
    for message in messages {
        builder.append_with_year(message, 2024).unwrap();
    }
    builder.finish(observed_time).unwrap().into_parts().0
}

fn framed_batch(messages: &[&[u8]]) -> Bytes {
    batch(messages, 42)
}

/// Scenario: Corruption appears after several completed parse steps while another node uses the same service.
/// Guarantees: Mutable service access is released before suspension; failure returns exact bytes, count and delivery context, then valid work still succeeds.
#[tokio::test(flavor = "current_thread")]
async fn cooperative_failure_recovers_after_partial_progress() {
    let handler = handler();
    let codec = resolve_syslog().unwrap();
    let complete = framed_batch(&vec![b"SECRET_TELEMETRY".as_slice(); 512]);
    let bytes = complete.slice(..complete.len() - 1);
    let pointer = bytes.as_ptr();
    let payload = codec.admit(SignalType::Logs, bytes.clone()).unwrap().into();
    let payload = otel_arrow_dfe_pdata_codec::testing::with_cached_item_count(payload, 512);
    let peer = "127.0.0.1:1234".parse().unwrap();
    let pdata = OtapPdata::new(Context::default(), payload).with_peer_addr(peer);
    let mut conversion = Box::pin(handler.materialize_otap(pdata));
    assert!(matches!(
        conversion
            .as_mut()
            .poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Pending
    ));
    let other = OtapPdata::new_default(
        codec
            .admit(SignalType::Logs, batch(&[b"valid"], 42))
            .unwrap()
            .into(),
    );
    assert!(handler.materialize_otap(other).await.is_ok());
    let error = conversion.await.unwrap_err();
    assert!(!format!("{error:?}").contains("SECRET_TELEMETRY"));
    let (_, recovered) = error.into_parts();
    assert_eq!(recovered.peer_addr(), Some(peer));
    assert_eq!(recovered.signal_type(), SignalType::Logs);
    assert_eq!(
        recovered.payload_ref().encoding(),
        Some(&SYSLOG_CEF_ENCODING)
    );
    assert_eq!(recovered.payload_ref().known_item_count(), Some(512));
    let recovered_bytes = recovered.payload_ref().encoded_bytes().unwrap();
    assert_eq!(recovered_bytes.as_ptr(), pointer);
    assert_eq!(recovered_bytes, &bytes);
    let next = OtapPdata::new_default(
        codec
            .admit(SignalType::Logs, batch(&[b"next"], 43))
            .unwrap()
            .into(),
    );
    assert!(handler.materialize_otap(next).await.is_ok());
}

/// Scenario: A conversion future is dropped after yielding with a partially built batch.
/// Guarantees: Input sharing and partial state are released, and cancellation neither leases nor poisons the reusable codec instance.
#[tokio::test(flavor = "current_thread")]
async fn cooperative_cancellation_releases_owned_progress() {
    let handler = handler();
    let codec = resolve_syslog().unwrap();
    let bytes = framed_batch(&vec![b"message".as_slice(); 512]);
    let pdata =
        OtapPdata::new_default(codec.admit(SignalType::Logs, bytes.clone()).unwrap().into());
    let mut conversion = Box::pin(handler.materialize_otap(pdata));
    assert!(matches!(
        conversion
            .as_mut()
            .poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Pending
    ));
    assert!(!bytes.is_unique());
    drop(conversion);
    assert!(bytes.is_unique());
    let next = OtapPdata::new_default(codec.admit(SignalType::Logs, bytes.clone()).unwrap().into());
    assert!(handler.materialize_otap(next).await.is_ok());
}

/// Scenario: Batch preparation and finishing convert large syslog input to native OTAP or OTLP.
/// Guarantees: Both paths yield; cancellation and late decode failure preserve original bytes, identity and cached counts, and successful conversion remains reusable.
#[tokio::test(flavor = "current_thread")]
async fn batch_plan_conversions_preserve_input_across_yields() {
    use otel_arrow_dfe_pdata_codec::{BatchPlan, PdataFormat};

    let handler = handler();
    let codec = resolve_syslog().unwrap();
    let complete = framed_batch(&vec![b"message".as_slice(); 512]);
    let truncated = complete.slice(..complete.len() - 1);
    let otlp = CodecService::new()
        .unwrap()
        .registry()
        .resolve(&PdataEncoding::OTLP)
        .unwrap();
    let encoded_otlp = PdataFormat::encoded(otlp);
    for (output, settings, working) in [
        (PdataFormat::OTAP, BatchSettings::otap(), PdataFormat::OTAP),
        (encoded_otlp, BatchSettings::otlp(), encoded_otlp),
        (encoded_otlp, BatchSettings::otap(), PdataFormat::OTAP),
    ] {
        let plan = BatchPlan::new(output, settings, false).unwrap();
        for prepare in [true, false] {
            for bytes in [&complete, &truncated] {
                let payload = codec.admit(SignalType::Logs, bytes.clone()).unwrap().into();
                let mut payload =
                    otel_arrow_dfe_pdata_codec::testing::with_cached_item_count(payload, 512);
                let convert = async |payload: &mut OtapPayload| {
                    if prepare {
                        handler.prepare_batch(&plan, payload).await
                    } else {
                        handler.finish_batch(&plan, payload).await
                    }
                };
                let mut conversion = Box::pin(convert(&mut payload));
                assert!(matches!(
                    conversion
                        .as_mut()
                        .poll(&mut TaskContext::from_waker(Waker::noop())),
                    Poll::Pending
                ));
                drop(conversion);
                assert_eq!(payload.encoded_bytes().unwrap().as_ptr(), bytes.as_ptr());
                assert_eq!(payload.encoded_bytes().unwrap(), bytes);
                assert_eq!(payload.encoding(), Some(&SYSLOG_CEF_ENCODING));
                assert_eq!(payload.known_item_count(), Some(512));

                let result = convert(&mut payload).await;
                if bytes.len() == complete.len() {
                    result.unwrap();
                    assert_eq!(payload.format(), if prepare { working } else { output });
                    assert_eq!(
                        handler
                            .materialize_otap_payload(payload)
                            .await
                            .unwrap()
                            .num_items(),
                        512
                    );
                } else {
                    assert!(result.is_err());
                    assert_eq!(payload.encoded_bytes().unwrap().as_ptr(), bytes.as_ptr());
                    assert_eq!(payload.encoded_bytes().unwrap(), bytes);
                    assert_eq!(payload.encoding(), Some(&SYSLOG_CEF_ENCODING));
                    assert_eq!(payload.known_item_count(), Some(512));
                }
            }
        }
    }
}

/// Scenario: Read-only views accept encoded syslog or require native records, then export to OTLP.
/// Guarantees: Direct borrowing creates no codec or buffer copy; fallback and cross-codec output use the cooperative boundary without changing the input.
#[tokio::test(flavor = "current_thread")]
async fn cooperative_views_and_cross_encoding_preserve_input() {
    let handler = handler();
    let codec = resolve_syslog().unwrap();
    let bytes = framed_batch(&vec![b"message".as_slice(); 512]);
    let pointer = bytes.as_ptr();
    let mut payload: OtapPayload = codec.admit(SignalType::Logs, bytes).unwrap().into();
    let accepted = AcceptedEncodings::accept_encoded([codec]);
    let view = handler.view(&payload, &accepted).await.unwrap();
    let PdataView::Encoded(view) = view else {
        panic!("accepted input must remain borrowed");
    };
    assert_eq!(view.bytes().as_ptr(), pointer);
    use otel_arrow_dfe_engine::runtime_services::CodecEffectHandler;
    assert_eq!(handler.codec_service().test_instance_count().unwrap(), 0);
    let view = handler
        .view(&payload, &AcceptedEncodings::native())
        .await
        .unwrap();
    assert!(matches!(view, PdataView::Native(_)));
    let plan = EncodingPlan::resolve(
        CodecService::new().unwrap().registry(),
        &PdataEncoding::OTLP,
        EncodePolicy::default(),
    )
    .unwrap();
    let encoded = handler.encode_owned(&mut payload, &plan).await.unwrap();
    assert!(!encoded.is_empty());
    assert_eq!(payload.encoded_bytes().unwrap().as_ptr(), pointer);
    assert_eq!(payload.encoding(), Some(&SYSLOG_CEF_ENCODING));
}
