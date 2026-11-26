// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Fixtures describe the component configuration and its observable effect.
mod prepare;

use anyhow::{Context, Result, ensure};
use minijinja::{Environment, UndefinedBehavior};
use otel_arrow_dfe_conformance_model::Operation;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// One entry in `fixtures/fixtures.yaml`. The URN selects the registered component,
/// the configuration installs it, and the probe defines what to observe.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    /// Resources the driver sets up for this fixture.
    #[serde(default)]
    pub services: Vec<Service>,
    pub urn: String,
    pub probe: Probe,
    /// Files installed into the stage directory before the receiver or consumer starts.
    #[serde(default)]
    pub(super) prepare: prepare::Prepare,
    /// Prerequisites the plan must find enabled before scheduling the fixture.
    #[serde(default)]
    pub requires: Vec<String>,
    config: Value,
    change_config: Option<Value>,
}

/// Optional peers or services needed by a component during deployment.
#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Service {
    Topics,
    AzureIdentity,
    Geneva,
    Kafka,
}

/// How the fixture wires its subject into the pipeline and verifies the result.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Probe {
    /// Use the inputs, peer nodes and expected output specified in YAML.
    PipelineNode(Box<super::pipeline_node::PipelineNodeProbe>),
    /// Exercise a receiver's authorization binding with accepted and denied requests.
    OtlpAuthorization,
    /// Send through the active listener and check that retired addresses refuse connections.
    SyslogTcp {
        connections: Vec<Value>,
        expect: ByOperation<ListenerCheck>,
    },
    /// Find the marker in the active exporter's file and check the retired outputs.
    JsonlFile {
        connections: Vec<Value>,
        expect: ByOperation<FileCheck>,
    },
    /// Inspect the header on a captured export as the extension binding changes.
    OtlpHttpHeader(Box<HeaderProbe>),
}

/// Required observations for each step of the shared rollout sequence.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByOperation<T> {
    add: T,
    change: T,
    remove: T,
}

impl<T> ByOperation<T> {
    pub fn at(&self, operation: Operation) -> &T {
        match operation {
            Operation::Add => &self.add,
            Operation::Change => &self.change,
            Operation::Remove => &self.remove,
        }
    }
}

/// A text field must equal the declared value or disappear from the marked record.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum ExpectedText {
    Absent,
    #[serde(untagged)]
    Equals {
        equals: String,
    },
}

impl ExpectedText {
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Absent => None,
            Self::Equals { equals } => Some(equals),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Listener {
    Baseline,
    Add,
    Change,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListenerCheck {
    pub delivers_on: Listener,
    pub closed: Option<Listener>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileCheck {
    pub delivers_to: String,
    pub absent_from: Vec<String>,
}

/// The exporter consuming the extension and the header expected on a marked export.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeaderProbe {
    consumer: Value,
    pub header: String,
    pub expect: ByOperation<ExpectedText>,
}

impl HeaderProbe {
    /// Use the same capture exporter across all stages. Drop only the binding to
    /// the subject when checking the baseline before installation or after removal.
    pub fn consumer(&self, url: &str, installed: bool) -> Result<Value> {
        let mut node = rendered(&self.consumer, minijinja::context!(http_url => url))?;
        let bindings = node["capabilities"]
            .as_object_mut()
            .context("consumer requires capabilities")?;
        ensure!(
            bindings.values().any(|v| v == "subject"),
            "consumer must bind the subject extension"
        );
        if !installed {
            bindings.retain(|_, extension| extension != "subject");
        }
        Ok(node)
    }
}

impl Probe {
    pub fn role(&self) -> &str {
        match self {
            Self::PipelineNode(probe) => &probe.role,
            Self::SyslogTcp { .. } => "receiver",
            Self::JsonlFile { .. } => "exporter",
            Self::OtlpHttpHeader(_) | Self::OtlpAuthorization => "extension",
        }
    }
}

/// Index the embedded fixtures by full component URN. Each URN has one rollout recipe.
pub fn catalog() -> Result<BTreeMap<String, Fixture>> {
    let mut catalog = BTreeMap::new();
    for fixture in serde_yaml::from_str::<Vec<Fixture>>(include_str!("../fixtures/fixtures.yaml"))?
    {
        let name = fixture.urn.clone();
        ensure!(
            catalog.insert(name.clone(), fixture).is_none(),
            "duplicate live fixture for {name}"
        );
    }
    Ok(catalog)
}

impl Fixture {
    /// Use `change_config` for the change stage when supplied. Both stages render
    /// their own variables, so the same template can produce different configurations.
    pub fn configuration(&self, change: bool, vars: impl Serialize) -> Result<Value> {
        let config = if change {
            self.change_config.as_ref().unwrap_or(&self.config)
        } else {
            &self.config
        };
        rendered(config, vars)
    }
}

fn template_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env
}

pub(super) fn render_text(template: &str, vars: &minijinja::Value) -> Result<String> {
    Ok(template_environment().render_str(template, vars)?)
}

/// Substitute variables in string values, preserving other JSON types. Unknown variables fail the fixture.
pub fn rendered(template: &Value, vars: impl Serialize) -> Result<Value> {
    fn render(env: &Environment<'_>, value: &mut Value, vars: &impl Serialize) -> Result<()> {
        match value {
            Value::String(s) => *s = env.render_str(s, vars)?,
            Value::Array(values) => {
                for value in values {
                    render(env, value, vars)?;
                }
            }
            Value::Object(map) => {
                for value in map.values_mut() {
                    render(env, value, vars)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let env = template_environment();
    let mut value = template.clone();
    render(&env, &mut value, &vars)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Scenario: load every catalog entry, including fixtures unavailable on this host.
    /// Guarantees: URNs are unique and each probe targets the declared component role.
    #[test]
    fn catalog_roles_match_urns() {
        for (urn, fixture) in catalog().expect("parse the YAML catalog") {
            assert_eq!(urn.split(':').nth(2), Some(fixture.probe.role()), "{urn}");
        }
    }

    /// Scenario: a processor changes a typed setting and a marked string in its second configuration.
    /// Guarantees: rendering preserves JSON types, selects the right phase, and rejects undefined variables.
    #[test]
    fn renders_typed_phase_configs() {
        let fixture: Fixture = serde_json::from_value(json!({
            "urn":"urn:otel:processor:test", "probe":{"kind":"pipeline-node","role":"processor"},
            "config":{"enabled":false,"count":1,"value":"{{ phase }}"},
            "change_config":{"enabled":true,"count":2,"value":"{{ phase }}"}
        }))
        .unwrap();
        for (change, phase, count) in [(false, "add", 1), (true, "change", 2)] {
            assert_eq!(
                fixture
                    .configuration(change, json!({"phase":phase}))
                    .unwrap(),
                json!({"enabled":change,"count":count,"value":phase})
            );
        }
        assert!(fixture.configuration(false, json!({})).is_err());
    }
}
