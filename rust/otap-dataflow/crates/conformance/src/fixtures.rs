// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Fixtures describe the component configuration and its observable effect.
use anyhow::{Result, ensure};
use minijinja::{Environment, UndefinedBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    #[serde(default)]
    pub services: Vec<Service>,
    pub urn: String,
    pub probe: Probe,
    #[serde(default)]
    pub requires: Vec<String>,
    config: Value,
    change_config: Option<Value>,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Service {
    Topics,
    AzureIdentity,
    Geneva,
    Kafka,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Probe {
    Pipeline(Box<super::graph::Graph>),
    OtlpAuthorization,
    SyslogTcp,
    LogAttribute {
        key: String,
    },
    JsonlFile,
    OtlpHttpHeader {
        capability: String,
        header: String,
        #[serde(default)]
        expected: Option<String>,
    },
}

impl Probe {
    pub fn role(&self) -> &str {
        match self {
            Self::Pipeline(g) => &g.role,
            Self::SyslogTcp => "receiver",
            Self::LogAttribute { .. } => "processor",
            Self::JsonlFile => "exporter",
            Self::OtlpHttpHeader { .. } | Self::OtlpAuthorization => "extension",
        }
    }
}

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
    /// Render only string values. Paths and quoted attribute values retain their JSON types.
    pub fn configuration(&self, change: bool, vars: impl Serialize) -> Result<Value> {
        let config = if change {
            self.change_config.as_ref().unwrap_or(&self.config)
        } else {
            &self.config
        };
        rendered(config, vars)
    }
}

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
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
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
            "urn":"urn:otel:processor:test", "probe":{"kind":"log-attribute","key":"k"},
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
