// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration type definitions for OpAMP Controller extension.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::de::{self, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use crate::extension::opamp::error::Error;
use crate::extension::opamp::proto::opamp::v1::{AnyValue, any_value::Value};
use crate::extension::opamp::util::ExponentialBackoff;

/// Configuration for OpAMP Controller Extension
#[derive(Deserialize, Serialize)]
pub struct Config {
    /// Initial instance_uid.
    ///
    /// May be overridden at runtime by server sending a new agent identity.
    ///
    /// If not provided, a UUIDv7 will be generated.
    pub instance_uid: Option<String>,

    /// Endpoint for OpAMP server.
    ///
    /// e.g. http://127.0.0.1:4320/opamp/v1 or ws://127.0.0.1:4320/opamp/v1
    ///
    /// The URL scheme will be used to identify the transport mechanism.
    /// Use `ws://` for [WebSocket transport](https://opentelemetry.io/docs/specs/opamp/#websocket-transport)
    /// or `http://` for [HTTP Transport](https://opentelemetry.io/docs/specs/opamp/#plain-http-transport).
    pub endpoint: String,

    /// Options for backoff timeout when connecting the websocket fails and will be retried.
    #[serde(default = "default_backoff")]
    pub connect_retry: ExponentialBackoff,

    /// Options for backoff timeout when retrying a request
    #[serde(default = "default_backoff")]
    pub request_retry: ExponentialBackoff,

    /// Duration in which the controller extension will perform shutdown procedures, such as
    /// sending close frame on its WebSocket connection.
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout: Duration,

    /// Interval at which the agent sends heartbeat to server. The agent during periods where
    /// there is no immediate stimulus to force sending messages.
    ///
    /// Default = 30s (recommended by OpAMP spec)
    #[serde(with = "humantime_serde", default = "default_heartbeat_interval")]
    pub heartbeat_interval: Duration,

    /// Configuration of the agent_description.
    ///
    /// If set, this will be used to configure the agent_description field on the
    /// AgentToServerMessage. Otherwise, this field will not be set on the message
    /// sent to the server.
    pub agent_description: Option<AgentIdentityConfig>,

    /// Configuration options for controlling reconciliation of dataflow engine config when a new
    /// config is received from the OpAMP server
    #[serde(default = "default_reconcile_config")]
    pub reconcile: EngineReconcileConfig,

    /// The key in the remote config map which will be expected to contain the remote config.
    /// Different servers may use different keys to send the config, such as "desired_state" or
    /// may omit the key and simply send an empty string.
    #[serde(default)]
    pub remote_config_key: String,
}

impl Config {
    /// validates this instance of config. Returns an error if the config is not valid.
    pub fn validate(&self) -> Result<(), Error> {
        // ensure if instance_uid is passed that it can be parsed
        if let Some(instance_uid) = &self.instance_uid {
            _ = uuid::Uuid::parse_str(instance_uid).map_err(|e| Error::InvalidInstanceUid {
                reason: e.to_string(),
            })?
        }

        let parsed_endpoint = Url::parse(&self.endpoint).map_err(|e| Error::InvalidEndpoint {
            reason: format!("could not parse \"{}\": {e}", self.endpoint),
        })?;

        match parsed_endpoint.scheme() {
            "ws" => {
                // this is are acceptable schemes.
            }

            "http" => {
                // return this error temporarily until we actually support OpAMP over HTTP.
                // Currently only websocket is supported
                return Err(Error::InvalidEndpoint {
                    reason: "OpAMP over plain HTTP not yet supported".into(),
                });
            }
            other => {
                return Err(Error::InvalidEndpoint {
                    reason: format!(
                        "invalid URL scheme {other}. Acceptable schemes \"ws\" or \"http\""
                    ),
                });
            }
        }

        Ok(())
    }
}

fn default_backoff() -> ExponentialBackoff {
    ExponentialBackoff::new(Duration::from_millis(250), Duration::from_secs(15))
}

fn default_heartbeat_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_shutdown_timeout() -> Duration {
    Duration::from_secs(2)
}

fn default_reconcile_config() -> EngineReconcileConfig {
    EngineReconcileConfig {
        step_timeout_secs: default_reconcile_timeout(),
        drain_timeout_secs: default_reconcile_timeout(),
        delete_timeout_secs: default_reconcile_timeout(),
        delete_missing: default_reconcile_delete_missing(),
    }
}

/// Configuration for controlling engine reconciliation behaviour
#[derive(Deserialize, Serialize)]
pub struct EngineReconcileConfig {
    /// timeout for reconcile step
    #[serde(default = "default_reconcile_timeout")]
    pub step_timeout_secs: u64,

    /// timeout for drain
    #[serde(default = "default_reconcile_timeout")]
    pub drain_timeout_secs: u64,

    /// timeout for pipeline delete
    #[serde(default = "default_reconcile_timeout")]
    pub delete_timeout_secs: u64,

    /// Whether to delete any pipelines that are missing from the config received from the server.
    /// When `true`, any currently running pipeline not in the received config is drained/deleted.
    /// When `false` received remote configs are treated as additive/partial updates.
    #[serde(default = "default_reconcile_delete_missing")]
    pub delete_missing: bool,
}

fn default_reconcile_timeout() -> u64 {
    10
}

const fn default_reconcile_delete_missing() -> bool {
    true
}

/// Configuration for agent description
#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct AgentIdentityConfig {
    /// Defines identifying attributes
    pub identifying_attributes: Option<BTreeMap<String, AgentDescriptionAttribute>>,

    /// Defines non-identifying attributes
    pub non_identifying_attributes: Option<BTreeMap<String, AgentDescriptionAttribute>>,
}

/// Config definition of a value of an attribute used in the agent description.
///
/// Can be deserialized into an AnyValue from JSON primitive. For example:
/// ```yaml
/// # in config:
/// agent_description:
///   identifying_attributes:
///     attr1: "string_val"
///     attr2: 514
///     attr3: 418.0
///     attr4: true
/// ```
#[derive(Debug, PartialEq)]
pub enum AgentDescriptionAttribute {
    /// String attribute value
    String(String),

    /// Int attribute value
    Int(i64),

    /// Boolean attribute value
    Bool(bool),

    /// Float attribute value
    Float(f64),
}

impl<'de> Deserialize<'de> for AgentDescriptionAttribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) => Ok(AgentDescriptionAttribute::String(s)),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    // safety: we've checked the type
                    Ok(AgentDescriptionAttribute::Int(n.as_i64().expect("is i64")))
                } else if n.is_f64() {
                    // safety: we've checked the type
                    Ok(AgentDescriptionAttribute::Float(
                        n.as_f64().expect("is f64"),
                    ))
                } else {
                    Err(de::Error::invalid_type(
                        Unexpected::Other("Number"),
                        &"integer or float",
                    ))
                }
            }
            serde_json::Value::Bool(b) => Ok(AgentDescriptionAttribute::Bool(b)),
            _ => Err(de::Error::invalid_type(
                Unexpected::Other(&format!("{:?}", value)),
                &"a string, integer, float, or boolean",
            )),
        }
    }
}

impl Serialize for AgentDescriptionAttribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Float(f) => serializer.serialize_f64(*f),
            Self::Int(i) => serializer.serialize_i64(*i),
            Self::String(s) => serializer.serialize_str(s),
        }
    }
}

impl From<&AgentDescriptionAttribute> for AnyValue {
    fn from(value: &AgentDescriptionAttribute) -> Self {
        match value {
            AgentDescriptionAttribute::String(s) => Self {
                value: Some(Value::StringValue(s.clone())),
            },
            AgentDescriptionAttribute::Int(i) => Self {
                value: Some(Value::IntValue(*i)),
            },
            AgentDescriptionAttribute::Float(f) => Self {
                value: Some(Value::DoubleValue(*f)),
            },
            AgentDescriptionAttribute::Bool(b) => Self {
                value: Some(Value::BoolValue(*b)),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_validate_returns_accepts_valid_instance_uid_from_uuid() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": "8be4df61-93ca-11d2-aa0d-00e098032b8c",
            "endpoint": "ws://127.0.0.1:4320"
        }))
        .unwrap();

        config.validate().unwrap();
    }

    #[test]
    fn test_validate_returns_error_for_unparsable_instance_uid() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "instance_uid": "invalid1234",
            "endpoint": "ws://127.0.0.1:4320"
        }))
        .unwrap();

        let error = config.validate().unwrap_err();
        assert_eq!(
            error,
            Error::InvalidInstanceUid {
                reason: "invalid character: found `i` at 0".into()
            }
        );
    }

    #[test]
    fn test_validate_rejects_unparsable_endpoint() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "endpoint": "this URL will not parse!"
        }))
        .unwrap();

        let error = config.validate().unwrap_err();
        assert_eq!(
            error,
            Error::InvalidEndpoint {
                reason: "could not parse \"this URL will not parse!\": relative URL without a base"
                    .into(),
            }
        );
    }

    #[test]
    fn test_validate_rejects_url_with_invalid_scheme() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "endpoint": "ftp://127.0.0.1:4320"
        }))
        .unwrap();

        let error = config.validate().unwrap_err();
        assert_eq!(
            error,
            Error::InvalidEndpoint {
                reason: "invalid URL scheme ftp. Acceptable schemes \"ws\" or \"http\"".into(),
            }
        );
    }
}
