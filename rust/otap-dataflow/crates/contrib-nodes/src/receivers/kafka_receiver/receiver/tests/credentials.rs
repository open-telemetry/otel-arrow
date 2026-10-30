// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Initial SASL credential-provider consumption tests.

use super::*;
use futures::stream;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::capability::auth::SaslCredential;
use otel_arrow_dfe_engine::capability::auth::sasl_credential_provider::{
    SASL_CREDENTIAL_USABLE_MARGIN, SaslCredentialProvider as SaslCredentialProviderCapability,
    SaslCredentialStream,
};
use otel_arrow_dfe_engine::capability::registry::{Capabilities, CapabilityRegistry};
use otel_arrow_dfe_engine::capability::{CapabilityError, CapabilityErrorSource};
use otel_arrow_dfe_engine::capability::{ExtensionCapability, LocalInstanceFactory};
use otel_arrow_dfe_engine::control::{NodeControlMsg, RuntimeControlMsg, runtime_ctrl_msg_channel};
use otel_arrow_dfe_engine::extension_capabilities;
use otel_arrow_dfe_engine::local::capability::auth::sasl_credential_provider::SaslCredentialProvider;
use otel_arrow_dfe_engine::local::message::{LocalReceiver, LocalSender};
use otel_arrow_dfe_engine::local::receiver::{ControlChannel, EffectHandler, Receiver as _};
use otel_arrow_dfe_engine::message::{Receiver, Sender};
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::testing::capability::resolve_bindings_for_test;
use otel_arrow_dfe_engine::testing::receiver::TestRuntime;
use otel_arrow_dfe_engine::testing::{test_node, test_pipeline_runtime_services};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use std::any::Any;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

enum ProviderBehavior {
    Credential(SaslCredential),
    Error(&'static str),
    Pending(Rc<Cell<bool>>),
}

struct TestSaslCredentialProvider {
    behavior: ProviderBehavior,
    error_source: CapabilityErrorSource<SaslCredentialProviderCapability>,
}

impl TestSaslCredentialProvider {
    fn credential(credential: SaslCredential) -> Self {
        Self {
            behavior: ProviderBehavior::Credential(credential),
            error_source: CapabilityErrorSource::new("test-sasl-provider".into()),
        }
    }

    fn error(message: &'static str) -> Self {
        Self {
            behavior: ProviderBehavior::Error(message),
            error_source: CapabilityErrorSource::new("test-sasl-provider".into()),
        }
    }

    fn pending(started: Rc<Cell<bool>>) -> Self {
        Self {
            behavior: ProviderBehavior::Pending(started),
            error_source: CapabilityErrorSource::new("test-sasl-provider".into()),
        }
    }
}

#[async_trait(?Send)]
impl SaslCredentialProvider for TestSaslCredentialProvider {
    async fn get_credential(&self) -> Result<SaslCredential, CapabilityError> {
        match &self.behavior {
            ProviderBehavior::Credential(credential) => Ok(credential.clone()),
            ProviderBehavior::Error(message) => Err(self.error_source.error(*message)),
            ProviderBehavior::Pending(started) => {
                started.set(true);
                std::future::pending().await
            }
        }
    }

    fn credential_stream(&self) -> SaslCredentialStream {
        Box::pin(stream::empty())
    }
}

fn capability_receiver(mechanism: &str) -> KafkaReceiver {
    let config = serde_json::json!({
        "brokers": "kafka:9092",
        "group_id": "test-group",
        "client_id": "test-client",
        "logs": {"topics": ["logs"]},
        "auth": {
            "sasl": {
                "mechanism": mechanism,
                "credential_source": "capability"
            }
        }
    });

    KafkaReceiver::from_config(make_pipeline_ctx(0, 1, 0), &config)
        .expect("capability-backed receiver config should be valid")
}

fn receiver_id() -> NodeId {
    NodeId {
        index: 0,
        name: "test-receiver".into(),
    }
}

fn capability_node_config(extension: Option<&str>) -> NodeUserConfig {
    let mut node_config = NodeUserConfig::new_receiver_config(KAFKA_RECEIVER_URN);
    node_config.config = serde_json::json!({
        "brokers": "kafka:9092",
        "group_id": "test-group",
        "client_id": "test-client",
        "logs": {"topics": ["logs"]},
        "auth": {
            "sasl": {
                "mechanism": "PLAIN",
                "credential_source": "capability"
            }
        }
    });
    if let Some(extension) = extension {
        let _ = node_config.capabilities.insert(
            SaslCredentialProviderCapability::NAME.into(),
            extension.to_owned().into(),
        );
    }
    node_config
}

fn inline_node_config() -> NodeUserConfig {
    let mut node_config = NodeUserConfig::new_receiver_config(KAFKA_RECEIVER_URN);
    node_config.config = serde_json::json!({
        "brokers": "kafka:9092",
        "group_id": "test-group",
        "client_id": "test-client",
        "logs": {"topics": ["logs"]},
        "auth": {
            "sasl": {
                "mechanism": "PLAIN",
                "username": "inline-user",
                "password": "inline-password"
            }
        }
    });
    node_config
}

#[derive(Clone)]
struct FactorySaslCredentialProvider {
    acquisitions: Rc<Cell<u32>>,
}

#[async_trait(?Send)]
impl SaslCredentialProvider for FactorySaslCredentialProvider {
    async fn get_credential(&self) -> Result<SaslCredential, CapabilityError> {
        self.acquisitions.set(self.acquisitions.get() + 1);
        Ok(SaslCredential::new("provider-user", "provider-password")
            .expect("factory test credential should be valid"))
    }

    fn credential_stream(&self) -> SaslCredentialStream {
        Box::pin(stream::empty())
    }
}

fn resolved_sasl_capabilities(
    node_config: &NodeUserConfig,
    acquisitions: Rc<Cell<u32>>,
) -> Capabilities {
    let provider = FactorySaslCredentialProvider { acquisitions };
    let extension_capabilities = extension_capabilities!(
        local: FactorySaslCredentialProvider => [SaslCredentialProviderCapability]
    );
    let instance_factory =
        LocalInstanceFactory::new(move || Box::new(provider.clone()) as Box<dyn Any>);
    let mut registry = CapabilityRegistry::new();
    (extension_capabilities.register_local)(
        "test-sasl-extension".into(),
        instance_factory,
        &mut registry,
    )
    .expect("register SASL credential provider");
    let known_extensions =
        HashSet::<otel_arrow_dfe_config::ExtensionId>::from(["test-sasl-extension".into()]);
    resolve_bindings_for_test(&node_config.capabilities, &registry, &known_extensions)
        .expect("resolve SASL credential provider")
}

fn startup_effect_handler(
    receiver: NodeId,
) -> (
    EffectHandler<OtapPdata>,
    otel_arrow_dfe_engine::control::RuntimeCtrlMsgReceiver<OtapPdata>,
) {
    let (runtime_tx, runtime_rx) = runtime_ctrl_msg_channel(4);
    let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
    (
        EffectHandler::new(
            receiver,
            HashMap::new(),
            None,
            runtime_tx,
            metrics_reporter,
            test_pipeline_runtime_services(),
        ),
        runtime_rx,
    )
}

/// Scenario: A provider supplies an initial credential for PLAIN or SCRAM authentication.
/// Guarantees: The receiver applies both values before creating the Kafka consumer.
#[tokio::test]
async fn applies_initial_provider_credential_for_username_password_mechanisms() {
    for mechanism in ["PLAIN", "SCRAM-SHA-256", "SCRAM-SHA-512"] {
        let credential =
            SaslCredential::new("provider-user", "provider-password").expect("valid credential");
        let provider = TestSaslCredentialProvider::credential(credential);
        let receiver = capability_receiver(mechanism);
        let mut client_config = receiver.config.build_client_config();

        let credential = KafkaReceiver::acquire_initial_sasl_credential(&provider, &receiver_id())
            .await
            .expect("provider credential should be acquired");
        KafkaReceiver::apply_sasl_credential(&mut client_config, credential, &receiver_id())
            .expect("provider credential should be applied");

        assert_eq!(client_config.get("sasl.username"), Some("provider-user"));
        assert_eq!(
            client_config.get("sasl.password"),
            Some("provider-password")
        );
    }
}

/// Scenario: The bound provider cannot acquire its initial SASL credential.
/// Guarantees: Startup reports provider identity without exposing the provider's source message.
#[tokio::test]
async fn redacts_provider_acquisition_error_source() {
    let sensitive_source = "provider-password-must-not-leak";
    let provider = TestSaslCredentialProvider::error(sensitive_source);

    let error = KafkaReceiver::acquire_initial_sasl_credential(&provider, &receiver_id())
        .await
        .expect_err("provider failure must fail startup")
        .to_string();

    assert!(error.contains("test-sasl-provider"));
    assert!(!error.contains(sensitive_source));
}

/// Scenario: The provider returns a credential already inside the usability margin.
/// Guarantees: The receiver rejects the credential before any Kafka connection attempt.
#[tokio::test]
async fn rejects_initial_credential_inside_usability_margin() {
    let credential = SaslCredential::new("provider-user", "provider-password")
        .expect("valid credential")
        .with_expiry(Instant::now() + SASL_CREDENTIAL_USABLE_MARGIN);
    let receiver = capability_receiver("PLAIN");
    let mut client_config = receiver.config.build_client_config();

    let error =
        KafkaReceiver::apply_sasl_credential(&mut client_config, credential, &receiver_id())
            .expect_err("near-expiry credential must fail startup")
            .to_string();

    assert!(error.contains("expires within 30 seconds"));
    assert!(!error.contains("provider-user"));
    assert!(!error.contains("provider-password"));
}

/// Scenario: Initial capability-backed credential acquisition remains pending when ingress drain begins.
/// Guarantees: The receiver cancels startup, acknowledges ingress drain, and terminates before creating a Kafka consumer.
#[tokio::test]
async fn pending_initial_credential_acquisition_is_interrupted_by_drain() {
    let started = Rc::new(Cell::new(false));
    let receiver = capability_receiver("PLAIN").with_sasl_credential_provider(Box::new(
        TestSaslCredentialProvider::pending(Rc::clone(&started)),
    ));
    let receiver_id = receiver_id();
    let (effect_handler, mut runtime_rx) = startup_effect_handler(receiver_id);
    let (control_tx, control_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(4);
    let control_sender = Sender::Local(LocalSender::mpsc(control_tx));
    let control_channel = ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(control_rx)));
    let deadline = Instant::now() + Duration::from_secs(1);

    let receiver_task = Box::new(receiver).start(control_channel, effect_handler);
    let send_drain = async {
        while !started.get() {
            tokio::task::yield_now().await;
        }
        control_sender
            .send(NodeControlMsg::DrainIngress {
                deadline,
                reason: "test drain".to_owned(),
            })
            .await
            .expect("send drain");
    };

    let (terminal_state, ()) = tokio::time::timeout(Duration::from_secs(1), async {
        tokio::join!(receiver_task, send_drain)
    })
    .await
    .expect("pending credential acquisition should be interrupted promptly");
    let _ = terminal_state.expect("receiver should terminate cleanly");

    let runtime_message = tokio::time::timeout(Duration::from_secs(1), runtime_rx.recv())
        .await
        .expect("receiver-drained notification should be prompt")
        .expect("runtime control channel should remain open");
    assert!(matches!(
        runtime_message,
        RuntimeControlMsg::ReceiverDrained { .. }
    ));
}

/// Scenario: Initial capability-backed credential acquisition remains pending without a control message.
/// Guarantees: Receiver startup fails after the bounded credential lookup timeout.
#[tokio::test(start_paused = true)]
async fn pending_initial_credential_acquisition_times_out() {
    let started = Rc::new(Cell::new(false));
    let provider = TestSaslCredentialProvider::pending(Rc::clone(&started));
    let mut receiver = capability_receiver("PLAIN");
    let (_control_tx, control_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(4);
    let mut control_channel = ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(control_rx)));

    let result = KafkaReceiver::await_initial_sasl_credential(
        &provider,
        &receiver_id(),
        &mut control_channel,
        &mut receiver.metrics,
    )
    .await;
    let error = match result {
        Ok(_) => panic!("pending credential acquisition must time out"),
        Err(error) => error.to_string(),
    };

    assert!(started.get(), "provider acquisition should be polled");
    assert!(
        error.contains("timed out after 5 seconds"),
        "unexpected error: {error}"
    );
}

/// Scenario: Capability mode is constructed through the Kafka receiver factory with a resolved local provider.
/// Guarantees: Factory construction retains the provider and startup acquires exactly one initial credential.
#[test]
fn factory_retains_and_uses_bound_sasl_credential_provider() {
    let node_config = capability_node_config(Some("test-sasl-extension"));
    let acquisitions = Rc::new(Cell::new(0));
    let capabilities = resolved_sasl_capabilities(&node_config, Rc::clone(&acquisitions));
    let runtime = TestRuntime::<OtapPdata>::new();
    let receiver = (KAFKA_RECEIVER.create)(
        make_pipeline_ctx(0, 1, 0),
        test_node("kafka-receiver"),
        Arc::new(node_config),
        runtime.config(),
        &capabilities,
    )
    .expect("factory should create capability-backed receiver");
    let observed_acquisitions = Rc::clone(&acquisitions);

    runtime
        .set_receiver(receiver)
        .run_test(move |context| async move {
            tokio::time::timeout(Duration::from_secs(1), async {
                while observed_acquisitions.get() == 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .expect("receiver should acquire the initial credential");
            context
                .send_shutdown(Instant::now() + Duration::from_secs(1), "test complete")
                .await
                .expect("send shutdown");
        })
        .run_validation(|_| async {});

    assert_eq!(acquisitions.get(), 1);
}

/// Scenario: Capability mode is configured but the Kafka receiver factory receives no resolved binding.
/// Guarantees: Factory construction fails with the capability-registry error before receiver startup.
#[test]
fn factory_rejects_missing_sasl_credential_provider_binding() {
    let runtime = TestRuntime::<OtapPdata>::new();
    let result = (KAFKA_RECEIVER.create)(
        make_pipeline_ctx(0, 1, 0),
        test_node("kafka-receiver"),
        Arc::new(capability_node_config(None)),
        runtime.config(),
        &Capabilities::empty(),
    );
    let error = match result {
        Ok(_) => panic!("missing capability binding must fail factory construction"),
        Err(error) => error.to_string(),
    };

    assert!(
        error.contains("sasl_credential_provider"),
        "unexpected error: {error}"
    );
}

/// Scenario: Inline SASL credentials are configured and no provider capability is bound.
/// Guarantees: Kafka receiver factory construction succeeds without consulting the capability registry.
#[test]
fn factory_allows_inline_sasl_credentials_without_provider_binding() {
    let runtime = TestRuntime::<OtapPdata>::new();
    let result = (KAFKA_RECEIVER.create)(
        make_pipeline_ctx(0, 1, 0),
        test_node("kafka-receiver"),
        Arc::new(inline_node_config()),
        runtime.config(),
        &Capabilities::empty(),
    );

    assert!(result.is_ok(), "inline mode should not require a provider");
}
