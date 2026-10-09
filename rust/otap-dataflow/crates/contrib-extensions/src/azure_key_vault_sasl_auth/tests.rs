// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Offline SDK-backed SASL provider lifecycle tests.

use std::sync::Arc;
use std::time::Duration;

use azure_core::http::RetryOptions;
use futures::{FutureExt, StreamExt};
use otel_arrow_dfe_engine::shared::capability::auth::sasl_credential_provider::SaslCredentialProvider as SharedSaslCredentialProvider;
use otel_arrow_dfe_engine::shared::extension::{ControlChannel, Extension};
use otel_arrow_dfe_engine::shared::message::SharedReceiver;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;

use super::*;
use crate::common::azure_key_vault::testing::{
    FakeCredential, FakeTransport, PAYLOAD, Reply, TOKEN, config, failure, secret, source,
};

fn extension(transport: Arc<FakeTransport>, config: Config) -> AzureKeyVaultSaslAuthExtension {
    let source = source(
        config,
        transport,
        Arc::new(FakeCredential::default()),
        RetryOptions::none(),
    );
    let registry = TelemetryRegistryHandle::new();
    let metrics = registry.register_metric_set::<AzureKeyVaultSaslAuthMetrics>(EmptyAttributes());
    let (tx, _rx) = watch::channel(None);
    AzureKeyVaultSaslAuthExtension::new(
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

/// Scenario: The real factory wires managed identity without any network acquisition.
/// Guarantees: It advertises shared SASL, builds active/shared only, and carries the configured readiness timeout.
#[test]
fn factory_wires_capability_and_startup_readiness() {
    let factory = &AZURE_KEY_VAULT_SASL_AUTH_EXTENSION;
    assert_eq!(factory.name, AZURE_KEY_VAULT_SASL_AUTH_URN);
    assert_eq!(
        factory.capabilities.as_ref().unwrap().shared,
        vec!["sasl_credential_provider"]
    );
    let (ctx, _registry) = otel_arrow_dfe_engine::testing::test_extension_ctx();
    let name: otel_arrow_dfe_config::ExtensionId = "vault-test".into();
    let mut value = crate::common::azure_key_vault::testing::config_value();
    value["startup_timeout"] = "45s".into();
    let user = Arc::new(ExtensionUserConfig::new(factory.name.into(), value));
    let runtime = ExtensionConfig::new(name.clone());
    let mut bundle = create(&ctx, name, user, &runtime).expect("factory wires offline");
    assert!(bundle.local().is_none());
    let mut wrapper = bundle.take_shared().unwrap();
    assert_eq!(wrapper.variant(), ExtensionVariant::Shared);
    assert!(!wrapper.is_passive());
    let probe = otel_arrow_dfe_engine::testing::take_test_extension_readiness_probe(&mut wrapper)
        .expect("real factory readiness gate");
    assert_eq!(probe.timeout(), Duration::from_secs(45));
}

/// Scenario: Config errors contain secret-shaped arbitrary values or endpoint credentials.
/// Guarantees: Factory diagnostics reject the config without echoing those values.
#[test]
fn config_errors_do_not_echo_input() {
    for value in [
        serde_json::json!({ "password": PAYLOAD }),
        serde_json::json!({
            "vault_url": format!("https://user:{TOKEN}@test-vault.vault.azure.net"),
            "username_secret": {"name":"username"},
            "password_secret": {"name":"password"}
        }),
    ] {
        let error = validate_config(&value).unwrap_err();
        assert!(!format!("{error:?} {error}").contains(TOKEN));
        assert!(!format!("{error:?} {error}").contains(PAYLOAD));
    }
}

/// Scenario: Both secret reads succeed and include a colon username and exact whitespace.
/// Guarantees: Direct SASL construction preserves values, redacts Debug, and has no expiry.
#[tokio::test]
async fn sasl_values_are_preserved_and_cache_coalesces() {
    let transport = FakeTransport::new(vec![
        secret(" user:service-marker \n"),
        secret(" password-value-marker \n"),
    ]);
    let extension = extension(Arc::clone(&transport), config());
    let (first, second) = tokio::join!(extension.get_credential(), extension.get_credential());
    for value in [first.unwrap(), second.unwrap()] {
        assert_eq!(value.expose_username(), " user:service-marker \n");
        assert_eq!(value.expose_password(), " password-value-marker \n");
        assert!(value.expires_on().is_none());
        let debug = format!("{value:?}");
        assert!(!debug.contains("service-marker"));
        assert!(!debug.contains("password-value-marker"));
    }
    assert_eq!(
        transport.request_count(),
        3,
        "challenge plus two reads exactly once"
    );
}

/// Scenario: Capability acquisition succeeds before the active task starts.
/// Guarantees: Readiness still signals, streams yield immediately and stay open, and no future request occurs.
#[tokio::test(start_paused = true)]
async fn once_cache_before_start_signals_readiness_and_never_refetches() {
    let transport = FakeTransport::new(vec![secret("user"), secret("password")]);
    let extension = extension(Arc::clone(&transport), config());
    let _ = extension.get_credential().await.unwrap();
    let mut early = extension.credential_stream();
    assert_eq!(early.next().await.unwrap().expose_username(), "user");
    let (tx, control) = control();
    let (effects, probe) =
        otel_arrow_dfe_engine::testing::test_extension_effect_handler_with_readiness(
            "vault-test".into(),
            Duration::from_secs(30),
        );
    let task = tokio::spawn(Box::new(extension.clone()).start(control, effects));
    probe
        .wait_ready()
        .await
        .expect("cached startup signals readiness");
    tokio::time::advance(Duration::from_secs(10 * 365 * 24 * 60 * 60)).await;
    let mut late = extension.credential_stream();
    assert_eq!(late.next().await.unwrap().expose_password(), "password");
    assert!(early.next().now_or_never().is_none());
    assert!(late.next().now_or_never().is_none());
    assert_eq!(
        extension.get_credential().await.unwrap().expose_username(),
        "user"
    );
    assert_eq!(transport.request_count(), 3);
    drop(tx);
    let _ = task.await.unwrap().unwrap();
}

/// Scenario: Startup's first acquisition fails transiently and the existing loop retries.
/// Guarantees: No partial value/readiness is published; recovery publishes once within the startup deadline.
#[tokio::test(start_paused = true)]
async fn transient_startup_failure_recovers_without_partial_publication() {
    let transport = FakeTransport::new(vec![failure(503), secret("user"), secret("password")]);
    let extension = extension(Arc::clone(&transport), config());
    let mut stream = extension.credential_stream();
    let (tx, control) = control();
    let (effects, probe) =
        otel_arrow_dfe_engine::testing::test_extension_effect_handler_with_readiness(
            "vault-test".into(),
            Duration::from_secs(30),
        );
    let task = tokio::spawn(Box::new(extension.clone()).start(control, effects));
    tokio::task::yield_now().await;
    assert!(stream.next().now_or_never().is_none());
    assert!(extension.subscribe().borrow().is_none());
    tokio::time::timeout(Duration::from_secs(30), probe.wait_ready())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stream.next().await.unwrap().expose_username(), "user");
    let requests = transport.request_count();
    tokio::time::advance(Duration::from_secs(365 * 24 * 60 * 60)).await;
    tokio::task::yield_now().await;
    assert_eq!(transport.request_count(), requests);
    drop(tx);
    let _ = task.await.unwrap().unwrap();
}

/// Scenario: Either username or password read fails after an SDK-authenticated request.
/// Guarantees: There is no usable partial credential, the stream stays empty, and errors never leak body/token data.
#[tokio::test]
async fn either_secret_failure_never_publishes_partial_credentials() {
    for replies in [vec![failure(404)], vec![secret("user"), failure(403)]] {
        let transport = FakeTransport::new(replies);
        let extension = extension(transport, config());
        let error = extension.get_credential().await.unwrap_err();
        let message = format!("{error:?} {error}");
        assert!(!message.contains(PAYLOAD));
        assert!(!message.contains(TOKEN));
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

/// Scenario: An authenticated secret request hangs beyond startup and another is cancelled by shutdown.
/// Guarantees: Timeout never signals readiness; dropping the active task's fetch releases SDK request state promptly.
#[tokio::test(start_paused = true)]
async fn startup_timeout_and_shutdown_cancel_in_flight_sdk_reads() {
    for shutdown in [false, true] {
        let transport = FakeTransport::new(vec![Reply::Pending]);
        let mut config = config();
        config.startup_timeout = Duration::from_secs(2);
        let extension = extension(Arc::clone(&transport), config);
        let (tx, control) = control();
        let (effects, probe) =
            otel_arrow_dfe_engine::testing::test_extension_effect_handler_with_readiness(
                "vault-test".into(),
                Duration::from_secs(2),
            );
        let task = tokio::spawn(Box::new(extension.clone()).start(control, effects));
        tokio::task::yield_now().await;
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
            assert!(extension.subscribe().borrow().is_none());
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
