// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Translator for AMCS (Azure Monitor Configuration Service) third-party configuration.
//!
//! AMCS delivers the Azure Monitor Agent every customer-authored Data Collection Rule (DCR) that
//! applies to a host, as a single JSON document. This module turns that document into an
//! [`OtelDataflowSpec`] the OTAP dataflow engine can run.
//!
//! This is a port of `AMCSParser.ExtractConfiguration` from the .NET `AMCSConfiguration`
//! project. The input is byte-for-byte the same payload the .NET agent consumes; only the output
//! differs -- a pipeline specification rather than a list of in-memory endpoint bindings.
//!
//! It additionally supports the **Agent Settings DCR** (`content.kind: "AgentSettings"`), which
//! the .NET parser does not read -- `Content.kind` and `Content.settings` are commented out in
//! `Configurations.cs`. That behaviour follows
//! `Telemetry-Collection-Spec/AMACoreAgent/otel-port-configuration.md` (owner: Ragu Marimuthu):
//! listener ports resolve from the environment first, then the Agent Settings rule, then the
//! built-in defaults. See [`listener`] for the full precedence chain.
//!
//! # Stages
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`schema`] | Serde model of the AMCS JSON payload |
//! | [`listener`] | OTLP listener discovery from the environment and Agent Settings |
//! | [`extract`] | Payload plus listeners to routable endpoint bindings |
//! | [`emit`] | Endpoint bindings to an engine pipeline specification |
//!
//! # Example
//!
//! ```no_run
//! use otel_arrow_dfe_contrib_config_translators::ConfigTranslator;
//! use otel_arrow_dfe_contrib_config_translators::amcs::AmcsTranslator;
//!
//! let payload = std::fs::read_to_string("amcs-config.json")?;
//! let yaml = AmcsTranslator::new().translate_to_yaml(&payload)?;
//! println!("{yaml}");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod emit;
pub mod extract;
pub mod listener;
pub mod schema;

use crate::amcs::emit::HostContext;
use crate::amcs::listener::{EnvironmentProvider, ProcessEnvironment};
use crate::{ConfigTranslator, Error};
use otel_arrow_dfe_config::engine::OtelDataflowSpec;

/// The dialect name reported by [`AmcsTranslator`].
pub const AMCS_DIALECT: &str = "amcs";

/// Translates AMCS configuration payloads into engine pipeline specifications.
///
/// Listener settings are read through an [`EnvironmentProvider`], which defaults to the process
/// environment. Tests and callers that already hold the values can substitute
/// [`StaticEnvironment`](listener::StaticEnvironment) via [`AmcsTranslator::with_environment`].
///
/// Values the agent knows about itself -- its Azure resource id, region and version -- do not
/// appear in a Data Collection Rule, so the embedding host supplies them through a
/// [`HostContext`] set with [`AmcsTranslator::with_host_context`]. Without one, the generated
/// exporters carry no request metadata.
#[derive(Debug, Clone, Default)]
pub struct AmcsTranslator<E = ProcessEnvironment> {
    environment: E,
    host: HostContext,
}

impl AmcsTranslator<ProcessEnvironment> {
    /// Create a translator that reads listener settings from the process environment.
    #[must_use]
    pub fn new() -> Self {
        Self {
            environment: ProcessEnvironment,
            host: HostContext::default(),
        }
    }
}

impl<E: EnvironmentProvider> AmcsTranslator<E> {
    /// Create a translator backed by a specific [`EnvironmentProvider`].
    #[must_use]
    pub fn with_environment(environment: E) -> Self {
        Self {
            environment,
            host: HostContext::default(),
        }
    }

    /// Attach the host-supplied values carried on every generated exporter.
    #[must_use]
    pub fn with_host_context(mut self, host: HostContext) -> Self {
        self.host = host;
        self
    }

    /// Access the environment provider backing this translator.
    #[must_use]
    pub const fn environment(&self) -> &E {
        &self.environment
    }

    /// Access the host context backing this translator.
    #[must_use]
    pub const fn host_context(&self) -> &HostContext {
        &self.host
    }
}

impl<E: EnvironmentProvider> ConfigTranslator for AmcsTranslator<E> {
    fn dialect(&self) -> &str {
        AMCS_DIALECT
    }

    fn translate(&self, raw: &str) -> Result<OtelDataflowSpec, Error> {
        let configurations = schema::Configurations::from_json(raw)?;
        let infos = extract::extract_configuration(&self.environment, &configurations);
        emit::build_pipeline(&infos, &self.host)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amcs::listener::{ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, StaticEnvironment};

    /// A payload mirroring the shape of `PipelineAgentTests/TestData/ConfigPackage/AMCSConfig`.
    const CANONICAL: &str = r#"{
      "configurations": [{
        "configurationId": "dcr-00000000000000000000000000000002",
        "eTag": "dcr-00000000000000000000000000000002/%22e1e1e1e1%22",
        "op": "added",
        "content": {
          "dataSources": [
            {
              "configuration": { "scheduledTransferPeriod": "PT1M" },
              "id": "myPerfCounterDataSource1",
              "kind": "perfCounter",
              "streams": [{ "stream": "GENERIC_PERF_BLOB", "solution": "LogManagement" }],
              "sendToChannels": ["ods-aaaaaaaa"]
            },
            {
              "configuration": {
                "resourceAttributeRouting": { "attributeName": "service.name", "attributeValue": "amcs" }
              },
              "id": "myOtelLogs1DataSource",
              "kind": "otelLogs",
              "streams": [{ "stream": "OPENTELEMETRY_LOGS_AGENT" }],
              "sendToChannels": ["gigl-dce-00000002"]
            },
            {
              "configuration": {
                "resourceAttributeRouting": { "attributeName": "service.name", "attributeValue": "amcs" }
              },
              "id": "myOtelTraces1DataSource",
              "kind": "otelTraces",
              "streams": [{ "stream": "OPENTELEMETRY_TRACES_AGENT" }],
              "sendToChannels": ["gigl-dce-00000002"]
            }
          ],
          "channels": [
            {
              "endpoint": "https://aaaaaaaa.ods.opinsights.azure.com",
              "id": "ods-aaaaaaaa",
              "protocol": "ods"
            },
            {
              "endpointUriTemplate": "https://dce.example.com/dataCollectionRules/dcr-1/streams/<STREAM>?api-version=2021-11-01-preview",
              "otelLogsEndpointUriTemplate": "https://dce.example.com/dataCollectionRules/dcr-1/streams/<STREAM>/otlp/v1/logs?api-version=2021-11-01-preview",
              "otelTracesEndpointUriTemplate": "https://dce.example.com/dataCollectionRules/dcr-1/streams/<STREAM>/otlp/v1/traces?api-version=2021-11-01-preview",
              "id": "gigl-dce-00000002",
              "protocol": "gig"
            }
          ]
        }
      }]
    }"#;

    fn translator() -> AmcsTranslator<StaticEnvironment> {
        AmcsTranslator::with_environment(
            StaticEnvironment::new().with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1"),
        )
    }

    #[test]
    fn reports_its_dialect() {
        assert_eq!(translator().dialect(), AMCS_DIALECT);
    }

    #[test]
    fn translates_the_canonical_payload() {
        let yaml = translator()
            .translate_to_yaml(CANONICAL)
            .expect("translation should succeed");

        assert!(yaml.contains("urn:otel:receiver:otlp"));
        assert!(yaml.contains("urn:otel:exporter:otlp_http"));
        assert!(yaml.contains("OPENTELEMETRY_LOGS_AGENT"));
        assert!(yaml.contains("OPENTELEMETRY_TRACES_AGENT"));
        assert!(yaml.contains("service.name"));
        // The `perfCounter` data source and the `ods` channel must not appear.
        assert!(!yaml.contains("GENERIC_PERF_BLOB"));
        assert!(!yaml.contains("opinsights"));
    }

    #[test]
    fn generated_yaml_is_accepted_by_the_engine_loader() {
        let yaml = translator()
            .translate_to_yaml(CANONICAL)
            .expect("translation should succeed");

        let spec = OtelDataflowSpec::from_yaml(&yaml);
        assert!(
            spec.is_ok(),
            "engine rejected generated config: {spec:?}\n{yaml}"
        );
    }

    #[test]
    fn translation_is_deterministic() {
        // The engine stores nodes in a `HashMap`, so the *textual* order of the serialized YAML
        // is not stable across runs. What must be stable is the specification itself, so compare
        // the parsed values rather than the strings.
        let first = translator().translate(CANONICAL).expect("first");
        let second = translator().translate(CANONICAL).expect("second");
        assert_eq!(first, second);
    }

    #[test]
    fn malformed_payload_is_a_deserialization_error() {
        let err = translator()
            .translate_to_yaml("{ not json")
            .expect_err("should fail");
        assert!(matches!(err, Error::Deserialization { .. }));
    }

    #[test]
    fn payload_without_otlp_data_sources_yields_an_empty_pipeline_error() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "id": "perf", "kind": "perfCounter",
                "streams": [{ "stream": "GENERIC_PERF_BLOB" }],
                "sendToChannels": ["ods-1"]
              }],
              "channels": [{ "id": "ods-1", "protocol": "ods" }]
            }
          }]
        }"#;

        let err = translator()
            .translate_to_yaml(json)
            .expect_err("should fail");
        assert!(matches!(err, Error::EmptyPipeline { .. }));
    }
}
