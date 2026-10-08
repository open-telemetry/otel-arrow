// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Initial SASL credential-provider consumption tests.

use super::*;
use futures::stream;
use otel_arrow_dfe_engine::capability::auth::SaslCredential;
use otel_arrow_dfe_engine::capability::auth::sasl_credential_provider::{
    SASL_CREDENTIAL_USABLE_MARGIN, SaslCredentialProvider as SaslCredentialProviderCapability,
    SaslCredentialStream,
};
use otel_arrow_dfe_engine::capability::{CapabilityError, CapabilityErrorSource};
use otel_arrow_dfe_engine::local::capability::auth::sasl_credential_provider::SaslCredentialProvider;
use otel_arrow_dfe_engine::node::NodeId;
use std::time::Instant;

enum ProviderBehavior {
    Credential(SaslCredential),
    Error(&'static str),
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
}

#[async_trait(?Send)]
impl SaslCredentialProvider for TestSaslCredentialProvider {
    async fn get_credential(&self) -> Result<SaslCredential, CapabilityError> {
        match &self.behavior {
            ProviderBehavior::Credential(credential) => Ok(credential.clone()),
            ProviderBehavior::Error(message) => Err(self.error_source.error(*message)),
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

/// Scenario: A provider supplies an initial credential for PLAIN or SCRAM authentication.
/// Guarantees: The receiver applies both values before creating the Kafka consumer.
#[tokio::test]
async fn applies_initial_provider_credential_for_username_password_mechanisms() {
    for mechanism in ["PLAIN", "SCRAM-SHA-256", "SCRAM-SHA-512"] {
        let credential =
            SaslCredential::new("provider-user", "provider-password").expect("valid credential");
        let receiver = capability_receiver(mechanism).with_sasl_credential_provider(Box::new(
            TestSaslCredentialProvider::credential(credential),
        ));
        let mut client_config = receiver.config.build_client_config();

        receiver
            .apply_initial_sasl_credential(&mut client_config, &receiver_id())
            .await
            .expect("provider credential should be applied");

        assert_eq!(client_config.get("sasl.username"), Some("provider-user"));
        assert_eq!(
            client_config.get("sasl.password"),
            Some("provider-password")
        );
    }
}

/// Scenario: Capability mode is selected but no provider was bound during factory creation.
/// Guarantees: Startup fails with an actionable configuration error rather than using no auth.
#[tokio::test]
async fn rejects_missing_sasl_credential_provider() {
    let receiver = capability_receiver("PLAIN");
    let mut client_config = receiver.config.build_client_config();

    let error = receiver
        .apply_initial_sasl_credential(&mut client_config, &receiver_id())
        .await
        .expect_err("missing provider must fail")
        .to_string();

    assert!(
        error.contains("sasl_credential_provider"),
        "unexpected error: {error}"
    );
}

/// Scenario: The bound provider cannot acquire its initial SASL credential.
/// Guarantees: Startup reports provider identity without exposing the provider's source message.
#[tokio::test]
async fn redacts_provider_acquisition_error_source() {
    let sensitive_source = "provider-password-must-not-leak";
    let receiver = capability_receiver("PLAIN").with_sasl_credential_provider(Box::new(
        TestSaslCredentialProvider::error(sensitive_source),
    ));
    let mut client_config = receiver.config.build_client_config();

    let error = receiver
        .apply_initial_sasl_credential(&mut client_config, &receiver_id())
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
    let receiver = capability_receiver("PLAIN").with_sasl_credential_provider(Box::new(
        TestSaslCredentialProvider::credential(credential),
    ));
    let mut client_config = receiver.config.build_client_config();

    let error = receiver
        .apply_initial_sasl_credential(&mut client_config, &receiver_id())
        .await
        .expect_err("near-expiry credential must fail startup")
        .to_string();

    assert!(error.contains("expires within 30 seconds"));
    assert!(!error.contains("provider-user"));
    assert!(!error.contains("provider-password"));
}
