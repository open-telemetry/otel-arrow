// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Offline SDK-backed Basic Auth lifecycle and validation coverage.

use std::sync::Arc;
use std::time::Duration;

use azure_core::http::RetryOptions;
use futures::{FutureExt, StreamExt};
use otel_arrow_dfe_engine::shared::capability::auth::basic_auth_provider::BasicAuthProvider as SharedBasicAuthProvider;
use otel_arrow_dfe_engine::shared::extension::{ControlChannel, Extension};
use otel_arrow_dfe_engine::shared::message::SharedReceiver;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;

use super::*;
use crate::common::azure_key_vault::testing::{
    FakeCredential, FakeTransport, PAYLOAD, Reply, TOKEN, config, config_value, failure, secret,
    source,
};

fn extension(
    transport: Arc<FakeTransport>,
    config: Config,
    credential: FakeCredential,
) -> AzureKeyVaultUserPassAuthExtension {
    let source = source(
        config,
        transport,
        Arc::new(credential),
        RetryOptions::none(),
    );
    let registry = TelemetryRegistryHandle::new();
    let metrics =
        registry.register_metric_set::<AzureKeyVaultUserPassAuthMetrics>(EmptyAttributes());
    let (tx, _rx) = watch::channel(None);
    AzureKeyVaultUserPassAuthExtension::new(
        "vault-test",
        Auth::from_source(source),
        BackgroundProviderRefreshPolicy::once(),
        tx,
        BackgroundProviderMetricsTracker::new(metrics),
    )
}

fn control() -> (
    tokio::sync::mpsc::Sender<otel_arrow_dfe_engine::control::ExtensionControlMsg>,
    ControlChannel,
) {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    (
        tx,
        ControlChannel::new(SharedReceiver::mpsc(rx), shutdown_rx),
    )
}

fn assert_redacted(error: impl std::fmt::Debug + std::fmt::Display) {
    let text = format!("{error:?} {error}");
    for sensitive in [TOKEN, PAYLOAD, "user-marker", "password-marker"] {
        assert!(
            !text.contains(sensitive),
            "diagnostic exposed a fixture value"
        );
    }
}

/// Scenario: The opt-in factory constructs its real shared wrapper without fetching credentials.
/// Guarantees: Only BasicAuthProvider is advertised and startup readiness uses the configured bound.
#[test]
fn factory_registers_basic_auth_and_readiness() {
    let factory = &AZURE_KEY_VAULT_USER_PASS_AUTH_EXTENSION;
    assert_eq!(factory.name, AZURE_KEY_VAULT_USER_PASS_AUTH_URN);
    assert_eq!(
        factory.capabilities.as_ref().unwrap().shared,
        vec!["basic_auth_provider"]
    );
    assert_eq!(
        OTAP_EXTENSION_FACTORIES
            .iter()
            .filter(|item| item.name == factory.name)
            .count(),
        1
    );
    let (ctx, _registry) = otel_arrow_dfe_engine::testing::test_extension_ctx();
    let name: otel_arrow_dfe_config::ExtensionId = "vault-test".into();
    let mut value = config_value();
    value["startup_timeout"] = "45s".into();
    let user = Arc::new(ExtensionUserConfig::new(factory.name.into(), value));
    let runtime = ExtensionConfig::new(name.clone());
    let mut bundle = create(&ctx, name, user, &runtime).unwrap();
    assert!(bundle.local().is_none());
    let mut wrapper = bundle.take_shared().unwrap();
    assert!(!wrapper.is_passive());
    let probe =
        otel_arrow_dfe_engine::testing::take_test_extension_readiness_probe(&mut wrapper).unwrap();
    assert_eq!(probe.timeout(), Duration::from_secs(45));
}

/// Scenario: Inputs request inline/file fallback, rotation, malformed references, or unsafe endpoints.
/// Guarantees: Factory validation fails closed and never echoes arbitrary config or token values.
#[test]
fn config_rejects_fallback_and_redacts_invalid_inputs() {
    for field in [
        "username",
        "password",
        "username_file",
        "password_secret_file",
        "refresh_interval",
    ] {
        let mut value = config_value();
        value[field] = PAYLOAD.into();
        assert_redacted(validate_config(&value).unwrap_err());
    }
    for (field, invalid) in [
        (
            "vault_url",
            serde_json::json!(format!("https://user:{TOKEN}@test-vault.vault.azure.net")),
        ),
        (
            "username_secret",
            serde_json::json!({"name": format!("../{PAYLOAD}")}),
        ),
        (
            "password_secret",
            serde_json::json!({"name": "password", "version": TOKEN}),
        ),
        ("identity", serde_json::json!({"method": TOKEN})),
        ("startup_timeout", serde_json::json!(PAYLOAD)),
    ] {
        let mut value = config_value();
        value[field] = invalid;
        assert_redacted(validate_config(&value).unwrap_err());
    }
}

/// Scenario: Two callers contend while the password read is deliberately held in flight.
/// Guarantees: Neither a partial pair nor readiness escapes, and concurrent startup coalesces to two secret reads.
#[tokio::test]
async fn concurrent_acquisition_publishes_complete_pair_before_readiness() {
    let (release, gated) = tokio::sync::oneshot::channel();
    let transport = FakeTransport::new(vec![secret(" user-marker "), Reply::Gated(gated)]);
    let extension = extension(Arc::clone(&transport), config(), FakeCredential::default());
    let (tx, control) = control();
    let (effects, probe) =
        otel_arrow_dfe_engine::testing::test_extension_effect_handler_with_readiness(
            "vault-test".into(),
            Duration::from_secs(30),
        );
    // Shared extension tasks are Send by contract, and are joined before returning.
    let task = tokio::spawn(Box::new(extension.clone()).start(control, effects));
    transport.wait_for_request_count(3).await;
    let mut stream = extension.credential_stream();
    let first = extension.get_credential();
    let second = extension.get_credential();
    let ready = probe.wait_ready();
    tokio::pin!(first, second, ready);
    assert!(first.as_mut().now_or_never().is_none());
    assert!(second.as_mut().now_or_never().is_none());
    assert!(ready.as_mut().now_or_never().is_none());
    assert!(stream.next().now_or_never().is_none());
    assert!(extension.subscribe().borrow().is_none());
    release
        .send(serde_json::json!({"value": " password-marker:with-colon "}))
        .unwrap();
    let (first, second, ready) = tokio::join!(first, second, ready);
    ready.unwrap();
    for value in [
        first.unwrap(),
        second.unwrap(),
        stream.next().await.unwrap(),
    ] {
        assert_eq!(value.expose_username(), " user-marker ");
        assert_eq!(value.expose_password(), " password-marker:with-colon ");
        assert!(value.expires_on().is_none());
        assert!(!format!("{value:?}").contains("marker"));
    }
    assert!(extension.subscribe().borrow().is_some());
    assert_eq!(transport.request_count(), 3, "one challenge and two reads");
    drop(tx);
    let _ = task.await.unwrap().unwrap();
}

/// Scenario: A cached snapshot precedes startup and is reused by reconnect-like calls and late subscribers.
/// Guarantees: Startup still signals readiness, and no elapsed time or consumer error triggers another vault request.
#[tokio::test(start_paused = true)]
async fn cached_snapshot_survives_late_subscribers_and_reconnects() {
    let transport = FakeTransport::new(vec![secret("user-marker"), secret("password-marker")]);
    let extension = extension(Arc::clone(&transport), config(), FakeCredential::default());
    let _ = extension.get_credential().await.unwrap();
    let (tx, control) = control();
    let (effects, probe) =
        otel_arrow_dfe_engine::testing::test_extension_effect_handler_with_readiness(
            "vault-test".into(),
            Duration::from_secs(30),
        );
    let task = tokio::spawn(Box::new(extension.clone()).start(control, effects));
    probe.wait_ready().await.unwrap();
    for _ in 0..3 {
        tokio::time::advance(Duration::from_secs(365 * 24 * 60 * 60)).await;
        let mut reconnect = extension.credential_stream();
        assert_eq!(
            reconnect.next().await.unwrap().expose_password(),
            "password-marker"
        );
        assert_eq!(
            extension.get_credential().await.unwrap().expose_username(),
            "user-marker"
        );
        assert!(reconnect.next().now_or_never().is_none());
    }
    drop(tx);
    let _ = task.await.unwrap().unwrap();
    assert_eq!(transport.request_count(), 3);
}

/// Scenario: Either secret read or identity acquisition fails with sensitive SDK response text.
/// Guarantees: No partial/fallback credential is published, diagnostics are redacted, and immediate retries are throttled.
#[tokio::test]
async fn failures_are_redacted_and_never_publish() {
    for (replies, identity_failure) in [
        (vec![failure(404)], false),
        (vec![secret("user-marker"), failure(403)], false),
        (vec![failure(401)], false),
        (vec![], true),
    ] {
        let transport = FakeTransport::new(replies);
        let credential = FakeCredential {
            fail: identity_failure,
            ..Default::default()
        };
        let extension = extension(Arc::clone(&transport), config(), credential);
        assert_redacted(extension.get_credential().await.unwrap_err());
        let count = transport.request_count();
        assert_redacted(extension.get_credential().await.unwrap_err());
        assert_eq!(transport.request_count(), count);
        assert!(extension.subscribe().borrow().is_none());
        assert!(
            extension
                .credential_stream()
                .next()
                .now_or_never()
                .is_none()
        );
    }
}

/// Scenario: AKV returns malformed, missing, empty, inactive, or protocol-incompatible values in either slot.
/// Guarantees: Basic Auth restrictions are enforced without trimming, normalization, or partial publication.
#[tokio::test]
async fn invalid_secret_values_and_basic_auth_restrictions() {
    let invalid_models = [
        serde_json::json!({}),
        serde_json::json!({"value": 123}),
        serde_json::json!({"value": ""}),
        serde_json::json!({"value": PAYLOAD, "attributes": {"enabled": false}}),
        serde_json::json!({"value": PAYLOAD, "attributes": {"exp": 1}}),
        serde_json::json!({"value": PAYLOAD, "attributes": {"nbf": 253402300799_i64}}),
    ];
    for password in [false, true] {
        for model in &invalid_models {
            let mut replies = vec![];
            if password {
                replies.push(secret("user-marker"));
            }
            replies.push(Reply::Json(200, model.clone()));
            let extension = extension(
                FakeTransport::new(replies),
                config(),
                FakeCredential::default(),
            );
            assert_redacted(extension.get_credential().await.unwrap_err());
            assert!(extension.subscribe().borrow().is_none());
        }
        let mut invalid_values = vec!["\n", "\r", "\t", "\0", "\u{7f}", "\u{85}"];
        if !password {
            invalid_values.push(":");
        }
        for invalid in invalid_values {
            let user = if password {
                "user-marker".to_owned()
            } else {
                format!("user-marker{invalid}")
            };
            let pass = if password {
                format!("password-marker{invalid}")
            } else {
                "password-marker".to_owned()
            };
            let transport = FakeTransport::new(vec![secret(&user), secret(&pass)]);
            let extension = extension(transport, config(), FakeCredential::default());
            assert_redacted(extension.get_credential().await.unwrap_err());
            assert!(
                extension
                    .credential_stream()
                    .next()
                    .now_or_never()
                    .is_none()
            );
        }
    }
}

/// Scenario: Startup loses its first password read and subsequently acquires a new complete pair.
/// Guarantees: Recovery cannot combine the previous username with a later password and readiness follows publication.
#[tokio::test(start_paused = true)]
async fn transient_failure_reacquires_both_secrets_before_readiness() {
    let transport = FakeTransport::new(vec![
        secret("discarded-user"),
        failure(503),
        secret("user-marker"),
        secret("password-marker"),
    ]);
    let extension = extension(Arc::clone(&transport), config(), FakeCredential::default());
    let (tx, control) = control();
    let (effects, probe) =
        otel_arrow_dfe_engine::testing::test_extension_effect_handler_with_readiness(
            "vault-test".into(),
            Duration::from_secs(30),
        );
    let task = tokio::spawn(Box::new(extension.clone()).start(control, effects));
    transport.wait_for_request_count(3).await;
    assert!(extension.subscribe().borrow().is_none());
    probe.wait_ready().await.unwrap();
    assert_eq!(
        extension.get_credential().await.unwrap().expose_username(),
        "user-marker"
    );
    assert_eq!(transport.request_count(), 5);
    drop(tx);
    let _ = task.await.unwrap().unwrap();
}

/// Scenario: A username or password SDK read hangs at startup or is cancelled during shutdown.
/// Guarantees: Deadline/shutdown releases in-flight requests without publishing or signaling readiness.
#[tokio::test(start_paused = true)]
async fn timeout_and_shutdown_cancel_either_read() {
    for password in [false, true] {
        for shutdown in [false, true] {
            let replies = if password {
                vec![secret("user-marker"), Reply::Pending]
            } else {
                vec![Reply::Pending]
            };
            let transport = FakeTransport::new(replies);
            let mut config = config();
            config.startup_timeout = Duration::from_secs(2);
            let extension = extension(Arc::clone(&transport), config, FakeCredential::default());
            let (tx, control) = control();
            let (effects, probe) =
                otel_arrow_dfe_engine::testing::test_extension_effect_handler_with_readiness(
                    "vault-test".into(),
                    Duration::from_secs(2),
                );
            let task = tokio::spawn(Box::new(extension.clone()).start(control, effects));
            transport
                .wait_for_request_count(if password { 3 } else { 2 })
                .await;
            assert_eq!(
                transport
                    .in_flight
                    .load(std::sync::atomic::Ordering::SeqCst),
                1
            );
            if shutdown {
                tx.send(
                    otel_arrow_dfe_engine::control::ExtensionControlMsg::Shutdown {
                        deadline: std::time::Instant::now(),
                        reason: "test cancellation".into(),
                    },
                )
                .await
                .unwrap();
                let _ = task.await.unwrap().unwrap();
                assert!(probe.wait_ready().await.is_err());
            } else {
                assert!(
                    tokio::time::timeout(Duration::from_secs(2), probe.wait_ready())
                        .await
                        .is_err()
                );
                drop(tx);
                let _ = task.await.unwrap().unwrap();
            }

            assert_eq!(
                transport
                    .in_flight
                    .load(std::sync::atomic::Ordering::SeqCst),
                0
            );
            assert!(extension.subscribe().borrow().is_none());
        }
    }
}

/// Scenario: The engine's real lifecycle gate sees either a complete pair or a pending SDK request.
/// Guarantees: Only successful acquisition passes readiness; timeout stops and joins acquisition with no later reads.
#[tokio::test(start_paused = true)]
async fn engine_readiness_gate_bounds_startup_and_drains_acquisition() {
    for succeeds in [false, true] {
        let replies = if succeeds {
            vec![secret("user-marker"), secret("password-marker")]
        } else {
            vec![Reply::Pending]
        };
        let transport = FakeTransport::new(replies);
        let provider = extension(Arc::clone(&transport), config(), FakeCredential::default());
        let user = Arc::new(ExtensionUserConfig::new(
            AZURE_KEY_VAULT_USER_PASS_AUTH_URN.into(),
            config_value(),
        ));
        let runtime = ExtensionConfig::new("vault-test");
        let mut bundle = ExtensionWrapper::builder("vault-test".into(), user, &runtime)
            .active()
            .with_readiness_probe_timeout_override(Duration::from_secs(2))
            .shared::<AzureKeyVaultUserPassAuthExtension>(provider.clone())
            .build()
            .unwrap();
        let (context, _registry) = otel_arrow_dfe_engine::testing::test_extension_ctx();
        let tasks = tokio::task::LocalSet::new();
        let outcome = tasks
            .run_until(otel_arrow_dfe_engine::testing::test_extension_readiness(
                bundle.take_shared().unwrap(),
                &tasks,
                &context,
            ))
            .await;
        assert_eq!(outcome.is_ok(), succeeds);
        assert_eq!(provider.subscribe().borrow().is_some(), succeeds);
        let requests = transport.request_count();
        assert_eq!(requests, if succeeds { 3 } else { 2 });
        assert_eq!(
            transport
                .in_flight
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
        tokio::time::advance(Duration::from_secs(3600)).await;
        assert_eq!(transport.request_count(), requests);
    }
}
