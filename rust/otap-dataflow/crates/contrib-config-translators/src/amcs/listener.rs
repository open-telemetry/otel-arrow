// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP listener discovery.
//!
//! Port of `AMCSParser.GetOtlpEventListenerInfo` / `AMCSParser.TryAddListener` from the .NET
//! `AMCSConfiguration` project, extended with Agent Settings support per
//! `Telemetry-Collection-Spec/AMACoreAgent/otel-port-configuration.md` (owner: Ragu Marimuthu).
//!
//! Listeners are **global to the host**: they come from environment variables and the agent
//! settings rule, not from a data-source rule. Every Data Collection Rule on the machine shares
//! the same listening sockets, which is why the generated pipeline has a single receiver.
//!
//! # Port precedence
//!
//! Highest to lowest, per the specification:
//!
//! 1. **Environment variable** -- always wins. If set, the agent settings value is ignored.
//! 2. **Agent Settings DCR** -- `OtlpGrpcLogsTracesPort` / `OtlpHttpProtobufLogsTracesPort`.
//! 3. **Hardcoded default** -- 4319 (gRPC) and 4320 (HTTP/protobuf).
//!
//! | Source | Variable / setting | Default |
//! |---|---|---|
//! | env | `OTLP_GRPC_LOGS_TRACES_PORT` | -- |
//! | agent settings | `OtlpGrpcLogsTracesPort` | -- |
//! | built in | -- | `4319` |
//! | env | `OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT` | -- |
//! | agent settings | `OtlpHttpProtobufLogsTracesPort` | -- |
//! | built in | -- | `4320` |
//!
//! Value handling, also per the specification:
//!
//! - `-1` ([`PORT_IGNORE`]) disables that listener outright, overriding any agent settings value.
//! - Any other invalid value -- unparseable, or outside `[1, 65535]` -- is treated as **unset**, so
//!   resolution falls through to the next source. With no agent settings rule present this is
//!   indistinguishable from the .NET behaviour of falling back to the default.
//! - A blank or unset host falls back to [`DEFAULT_HOST`]. Host has **no** agent settings
//!   equivalent; `OTLP_GRPC_LOGS_TRACES_HOST` and `OTLP_HTTP_PROTOBUF_LOGS_TRACES_HOST` are the
//!   only way to override it.
//!
//! Note that discovering a listener here does not by itself open a port: the specification
//! requires an OTel data-source rule to be present, which in this crate means
//! [`extract_configuration`](crate::amcs::extract::extract_configuration) produced at least one
//! binding.

use otel_arrow_dfe_telemetry::otel_warn;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

/// Host used when the host environment variable is unset or blank.
pub const DEFAULT_HOST: &str = "localhost";

/// Sentinel port value that disables a listener.
pub const PORT_IGNORE: i64 = -1;

/// Default port for the OTLP/gRPC logs and traces listener.
pub const DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT: u16 = 4319;

/// Default port for the OTLP/HTTP-protobuf logs and traces listener.
pub const DEFAULT_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT: u16 = 4320;

/// Environment variable naming the host for the OTLP/gRPC listener.
pub const ENV_OTLP_GRPC_LOGS_TRACES_HOST: &str = "OTLP_GRPC_LOGS_TRACES_HOST";

/// Environment variable naming the host for the OTLP/HTTP-protobuf listener.
pub const ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_HOST: &str = "OTLP_HTTP_PROTOBUF_LOGS_TRACES_HOST";

/// Environment variable naming the port for the OTLP/gRPC listener.
pub const ENV_OTLP_GRPC_LOGS_TRACES_PORT: &str = "OTLP_GRPC_LOGS_TRACES_PORT";

/// Environment variable naming the port for the OTLP/HTTP-protobuf listener.
pub const ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT: &str = "OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT";

/// Agent settings name carrying the OTLP/gRPC listener port.
pub const SETTING_OTLP_GRPC_LOGS_TRACES_PORT: &str = "OtlpGrpcLogsTracesPort";

/// Agent settings name carrying the OTLP/HTTP-protobuf listener port.
pub const SETTING_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT: &str = "OtlpHttpProtobufLogsTracesPort";

/// Lowest valid TCP port.
const MIN_PORT: i64 = 1;

/// Highest valid TCP port.
const MAX_PORT: i64 = 65535;

/// Supplies environment variables to the translator.
///
/// Injected rather than read directly from the process so that tests can vary the environment
/// without `unsafe` calls to [`std::env::set_var`], which is neither sound nor race-free when
/// tests run in parallel. This mirrors the .NET `IEnvironmentVariableProvider` abstraction and its
/// `UnitTestEnvironmentVariableProvider` implementation.
pub trait EnvironmentProvider {
    /// Return the value of `key`, or `None` when it is not set.
    fn get(&self, key: &str) -> Option<String>;
}

/// Reads environment variables from the current process.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessEnvironment;

impl EnvironmentProvider for ProcessEnvironment {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// An in-memory [`EnvironmentProvider`] for tests and for callers that already hold the values.
#[derive(Debug, Clone, Default)]
pub struct StaticEnvironment {
    vars: HashMap<String, String>,
}

impl StaticEnvironment {
    /// Create an empty environment, in which every lookup misses and defaults apply.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable, returning `self` so calls can be chained.
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let _ = self.vars.insert(key.into(), value.into());
        self
    }
}

impl EnvironmentProvider for StaticEnvironment {
    fn get(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }
}

/// The wire protocol a listener accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OtlpProtocol {
    /// OTLP over gRPC.
    Grpc,
    /// OTLP over HTTP with protobuf encoding.
    HttpProtobuf,
}

impl OtlpProtocol {
    /// A stable lowercase name, used in diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grpc => "grpc",
            Self::HttpProtobuf => "http_protobuf",
        }
    }
}

/// A resolved OTLP listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtlpEventListenerInfo {
    /// The configured host, verbatim (for example `localhost` or `0.0.0.0`).
    pub host: String,
    /// The resolved port.
    pub port: u16,
    /// The protocol this listener accepts.
    pub protocol: OtlpProtocol,
}

impl OtlpEventListenerInfo {
    /// Resolve this listener to a [`SocketAddr`] suitable for the OTLP receiver's
    /// `listening_addr` field.
    ///
    /// The receiver deserializes `listening_addr` as a [`SocketAddr`], which does **not** accept
    /// hostnames -- so the default host `localhost` has to be resolved to a literal address before
    /// it reaches the generated YAML. Resolution prefers IPv4 to match the .NET agent's binding
    /// behaviour, falling back to IPv6 when no IPv4 address is available.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidListenerAddress`](crate::Error::InvalidListenerAddress) when the
    /// host cannot be resolved.
    pub fn socket_addr(&self) -> Result<SocketAddr, crate::Error> {
        if let Ok(ip) = self.host.parse::<IpAddr>() {
            return Ok(SocketAddr::new(ip, self.port));
        }

        let resolved: Vec<SocketAddr> = (self.host.as_str(), self.port)
            .to_socket_addrs()
            .map_err(|e| crate::Error::InvalidListenerAddress {
                host: self.host.clone(),
                port: self.port,
                details: e.to_string(),
            })?
            .collect();

        resolved
            .iter()
            .find(|addr| addr.is_ipv4())
            .or_else(|| resolved.first())
            .copied()
            .ok_or_else(|| crate::Error::InvalidListenerAddress {
                host: self.host.clone(),
                port: self.port,
                details: "host resolved to no addresses".to_string(),
            })
    }
}

/// Agent settings values relevant to listener discovery.
///
/// Built from the payload's agent settings rule, if one is present. Values are kept as raw
/// strings because that is how they arrive on the wire, and because an unparseable value must be
/// treated as absent rather than rejected.
#[derive(Debug, Clone, Default)]
pub struct AgentSettings {
    /// Value of the `OtlpGrpcLogsTracesPort` setting, if present.
    pub grpc_port: Option<String>,
    /// Value of the `OtlpHttpProtobufLogsTracesPort` setting, if present.
    pub http_protobuf_port: Option<String>,
}

impl AgentSettings {
    /// Extract the listener-relevant settings from a parsed AMCS payload.
    ///
    /// Returns an empty set when the payload carries no agent settings rule.
    #[must_use]
    pub fn from_configurations(config: &crate::amcs::schema::Configurations) -> Self {
        Self {
            grpc_port: config
                .agent_setting(SETTING_OTLP_GRPC_LOGS_TRACES_PORT)
                .map(str::to_string),
            http_protobuf_port: config
                .agent_setting(SETTING_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT)
                .map(str::to_string),
        }
    }
}

/// How a candidate port value was interpreted.
enum PortValue {
    /// A usable port.
    Port(u16),
    /// The sentinel `-1`, disabling the listener.
    Disabled,
    /// Absent, blank, unparseable, or out of range -- fall through to the next source.
    Unset,
}

/// Interpret one raw port value.
///
/// Anything that is not a valid port and is not the disable sentinel counts as [`PortValue::Unset`],
/// so resolution continues to the next source in the precedence chain. The specification is
/// explicit about this: *"Any other invalid input is treated as an empty string (not `-1`) and the
/// listener starts accordingly."*
fn interpret_port(raw: Option<&str>) -> PortValue {
    let Some(trimmed) = raw.map(str::trim).filter(|v| !v.is_empty()) else {
        return PortValue::Unset;
    };

    let Ok(value) = trimmed.parse::<i64>() else {
        return PortValue::Unset;
    };

    if value == PORT_IGNORE {
        return PortValue::Disabled;
    }

    if (MIN_PORT..=MAX_PORT).contains(&value) {
        // Range-checked immediately above, so this conversion cannot fail.
        u16::try_from(value).map_or(PortValue::Unset, PortValue::Port)
    } else {
        PortValue::Unset
    }
}

/// Discover the OTLP listeners configured for this host.
///
/// Returns at most two listeners (gRPC and HTTP/protobuf), in that order. Either or both may be
/// absent when disabled via [`PORT_IGNORE`]. An empty result means OTLP ingestion is switched off
/// entirely, which callers must treat as "no pipeline to build".
///
/// `settings` supplies the agent settings rule values; pass
/// [`AgentSettings::default`] when the payload has no such rule.
#[must_use]
pub fn discover_listeners<E: EnvironmentProvider + ?Sized>(
    env: &E,
    settings: &AgentSettings,
) -> Vec<OtlpEventListenerInfo> {
    let mut listeners = Vec::with_capacity(2);

    try_add_listener(
        env,
        ENV_OTLP_GRPC_LOGS_TRACES_HOST,
        ENV_OTLP_GRPC_LOGS_TRACES_PORT,
        SETTING_OTLP_GRPC_LOGS_TRACES_PORT,
        settings.grpc_port.as_deref(),
        DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT,
        OtlpProtocol::Grpc,
        &mut listeners,
    );

    try_add_listener(
        env,
        ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_HOST,
        ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT,
        SETTING_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT,
        settings.http_protobuf_port.as_deref(),
        DEFAULT_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT,
        OtlpProtocol::HttpProtobuf,
        &mut listeners,
    );

    listeners
}

/// Resolve one listener and append it to `listeners`, unless it is disabled.
#[allow(clippy::too_many_arguments)]
fn try_add_listener<E: EnvironmentProvider + ?Sized>(
    env: &E,
    host_env_var: &str,
    port_env_var: &str,
    port_setting_name: &str,
    setting_port: Option<&str>,
    default_port: u16,
    protocol: OtlpProtocol,
    listeners: &mut Vec<OtlpEventListenerInfo>,
) {
    let env_port = env.get(port_env_var);

    // 1. Environment variable always wins.
    let port = match interpret_port(env_port.as_deref()) {
        PortValue::Disabled => {
            otel_warn!(
                "amcs.listener.disabled",
                protocol = protocol.as_str(),
                env_var = port_env_var,
                message = "OTLP listener disabled because its environment variable is set to -1"
            );
            return;
        }
        PortValue::Port(port) => port,
        // 2. Fall through to the agent settings rule.
        PortValue::Unset => match interpret_port(setting_port) {
            PortValue::Disabled => {
                otel_warn!(
                    "amcs.listener.disabled",
                    protocol = protocol.as_str(),
                    setting = port_setting_name,
                    message = "OTLP listener disabled because its agent setting is set to -1"
                );
                return;
            }
            PortValue::Port(port) => port,
            // 3. Fall back to the built-in default.
            PortValue::Unset => {
                if setting_port.is_some() {
                    otel_warn!(
                        "amcs.listener.invalid_agent_setting",
                        protocol = protocol.as_str(),
                        setting = port_setting_name,
                        default_port = i64::from(default_port),
                        message =
                            "agent setting port is not usable; falling back to the default port"
                    );
                }
                default_port
            }
        },
    };

    // Host is environment-only; the agent settings rule has no equivalent.
    let host = env
        .get(host_env_var)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_HOST.to_string());

    listeners.push(OtlpEventListenerInfo {
        host,
        port,
        protocol,
    });
}

/// The loopback address `localhost` resolves to when no resolver is available.
#[allow(dead_code)]
pub(crate) const LOOPBACK_V4: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

#[cfg(test)]
mod tests {
    use super::*;

    /// No agent settings rule present.
    fn no_settings() -> AgentSettings {
        AgentSettings::default()
    }

    /// An agent settings rule supplying both ports.
    fn settings(grpc: &str, http: &str) -> AgentSettings {
        AgentSettings {
            grpc_port: Some(grpc.to_string()),
            http_protobuf_port: Some(http.to_string()),
        }
    }

    /// With no environment set at all, both listeners appear on their default ports.
    #[test]
    fn defaults_when_environment_is_empty() {
        let env = StaticEnvironment::new();
        let listeners = discover_listeners(&env, &no_settings());

        assert_eq!(listeners.len(), 2);
        assert_eq!(
            listeners[0],
            OtlpEventListenerInfo {
                host: DEFAULT_HOST.to_string(),
                port: DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT,
                protocol: OtlpProtocol::Grpc,
            }
        );
        assert_eq!(
            listeners[1],
            OtlpEventListenerInfo {
                host: DEFAULT_HOST.to_string(),
                port: DEFAULT_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT,
                protocol: OtlpProtocol::HttpProtobuf,
            }
        );
    }

    /// Mirrors `OtlpEventInfoTest` permutation 1: gRPC disabled, custom HTTP host and port.
    #[test]
    fn grpc_disabled_with_custom_http_endpoint() {
        let env = StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "-1")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "12345")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_HOST, "0.0.0.0");

        let listeners = discover_listeners(&env, &no_settings());

        assert_eq!(listeners.len(), 1);
        assert_eq!(listeners[0].host, "0.0.0.0");
        assert_eq!(listeners[0].port, 12345);
        assert_eq!(listeners[0].protocol, OtlpProtocol::HttpProtobuf);
    }

    /// Mirrors permutation 2: port `0` is out of range so the default applies, and a blank host
    /// falls back to `localhost`.
    #[test]
    fn zero_port_and_blank_host_fall_back_to_defaults() {
        let env = StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "-1")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "0")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_HOST, "");

        let listeners = discover_listeners(&env, &no_settings());

        assert_eq!(listeners.len(), 1);
        assert_eq!(listeners[0].host, DEFAULT_HOST);
        assert_eq!(
            listeners[0].port,
            DEFAULT_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT
        );
        assert_eq!(listeners[0].protocol, OtlpProtocol::HttpProtobuf);
    }

    /// Mirrors permutation 3: HTTP disabled, gRPC falls back to its default.
    #[test]
    fn http_disabled_leaves_grpc_on_default_port() {
        let env = StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_HOST, "")
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "0")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1");

        let listeners = discover_listeners(&env, &no_settings());

        assert_eq!(listeners.len(), 1);
        assert_eq!(listeners[0].host, DEFAULT_HOST);
        assert_eq!(listeners[0].port, DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT);
        assert_eq!(listeners[0].protocol, OtlpProtocol::Grpc);
    }

    /// Mirrors permutation 4: an unparseable port falls back to the default.
    #[test]
    fn unparseable_port_falls_back_to_default() {
        let env = StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_HOST, "")
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "Junk")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1");

        let listeners = discover_listeners(&env, &no_settings());

        assert_eq!(listeners.len(), 1);
        assert_eq!(listeners[0].port, DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT);
        assert_eq!(listeners[0].protocol, OtlpProtocol::Grpc);
    }

    /// Both protocols disabled yields no listeners at all.
    #[test]
    fn both_protocols_disabled_yields_empty_list() {
        let env = StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "-1")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1");

        assert!(discover_listeners(&env, &no_settings()).is_empty());
    }

    /// A port above the valid range falls back to the default, as with `0`.
    #[test]
    fn port_above_range_falls_back_to_default() {
        let env = StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "70000")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1");

        let listeners = discover_listeners(&env, &no_settings());

        assert_eq!(listeners.len(), 1);
        assert_eq!(listeners[0].port, DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT);
    }

    /// Custom hosts and ports are honoured on both protocols.
    #[test]
    fn custom_hosts_and_ports() {
        let env = StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_HOST, "127.0.0.2")
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "5000")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_HOST, "127.0.0.3")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "5001");

        let listeners = discover_listeners(&env, &no_settings());

        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].host, "127.0.0.2");
        assert_eq!(listeners[0].port, 5000);
        assert_eq!(listeners[1].host, "127.0.0.3");
        assert_eq!(listeners[1].port, 5001);
    }

    // ---------------------------------------------------------------------------------------
    // Agent Settings precedence, per
    // Telemetry-Collection-Spec/AMACoreAgent/otel-port-configuration.md.
    // Scenario numbers below refer to that document's "Port opening behavior by scenario" table.
    // Scenarios 1, 2, 5, 6, 9, 10, 13 and 14 concern the absence of an OTel data-source rule,
    // which is enforced one level up in `extract_configuration`; they are covered in
    // `tests/translate.rs`.
    // ---------------------------------------------------------------------------------------

    /// Scenario 4: both env vars set and agent settings present -- the environment wins.
    #[test]
    fn environment_overrides_agent_settings() {
        let env = StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "4319")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "4320");

        let listeners = discover_listeners(&env, &settings("4329", "4330"));

        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].port, 4319);
        assert_eq!(listeners[1].port, 4320);
    }

    /// Scenario 8: no environment variables -- the agent settings values are used.
    #[test]
    fn agent_settings_used_when_environment_is_unset() {
        let listeners = discover_listeners(&StaticEnvironment::new(), &settings("4329", "4330"));

        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].port, 4329);
        assert_eq!(listeners[0].protocol, OtlpProtocol::Grpc);
        assert_eq!(listeners[1].port, 4330);
        assert_eq!(listeners[1].protocol, OtlpProtocol::HttpProtobuf);
    }

    /// Scenario 7: no environment variables and no agent settings -- defaults apply.
    #[test]
    fn defaults_when_neither_source_supplies_a_port() {
        let listeners = discover_listeners(&StaticEnvironment::new(), &no_settings());

        assert_eq!(listeners[0].port, DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT);
        assert_eq!(
            listeners[1].port,
            DEFAULT_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT
        );
    }

    /// Scenario 12: only the HTTP env var is set, so gRPC comes from agent settings and HTTP from
    /// the environment.
    #[test]
    fn per_protocol_precedence_is_independent() {
        let env = StaticEnvironment::new().with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "4321");

        let listeners = discover_listeners(&env, &settings("4329", "4330"));

        assert_eq!(listeners.len(), 2);
        assert_eq!(
            listeners[0].port, 4329,
            "gRPC should come from agent settings"
        );
        assert_eq!(
            listeners[1].port, 4321,
            "HTTP should come from the environment"
        );
    }

    /// Scenario 11: only the HTTP env var is set and there are no agent settings, so gRPC falls
    /// back to its default.
    #[test]
    fn partial_environment_without_agent_settings_uses_defaults() {
        let env = StaticEnvironment::new().with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "4321");

        let listeners = discover_listeners(&env, &no_settings());

        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].port, DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT);
        assert_eq!(listeners[1].port, 4321);
    }

    /// Scenario 16: `-1` in the environment disables the listener even when agent settings supply
    /// a port.
    #[test]
    fn environment_disable_overrides_agent_settings() {
        let env = StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "-1")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1");

        assert!(discover_listeners(&env, &settings("4329", "4330")).is_empty());
    }

    /// An invalid environment value is treated as unset, so resolution falls through to the agent
    /// settings rather than jumping straight to the default.
    #[test]
    fn invalid_environment_value_falls_through_to_agent_settings() {
        let env = StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "Junk")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "0");

        let listeners = discover_listeners(&env, &settings("4329", "4330"));

        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].port, 4329);
        assert_eq!(listeners[1].port, 4330);
    }

    /// An invalid agent settings value falls through to the default.
    #[test]
    fn invalid_agent_setting_falls_through_to_default() {
        let listeners = discover_listeners(&StaticEnvironment::new(), &settings("Junk", "99999"));

        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].port, DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT);
        assert_eq!(
            listeners[1].port,
            DEFAULT_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT
        );
    }

    /// `-1` in the agent settings disables the listener when the environment says nothing.
    #[test]
    fn agent_setting_can_disable_a_listener() {
        let listeners = discover_listeners(&StaticEnvironment::new(), &settings("-1", "4330"));

        assert_eq!(listeners.len(), 1);
        assert_eq!(listeners[0].port, 4330);
        assert_eq!(listeners[0].protocol, OtlpProtocol::HttpProtobuf);
    }

    /// Host has no agent settings equivalent, so it is always environment or default.
    #[test]
    fn host_is_not_configurable_through_agent_settings() {
        let env = StaticEnvironment::new().with(ENV_OTLP_GRPC_LOGS_TRACES_HOST, "0.0.0.0");

        let listeners = discover_listeners(&env, &settings("4329", "4330"));

        assert_eq!(listeners[0].host, "0.0.0.0");
        assert_eq!(listeners[0].port, 4329);
        assert_eq!(listeners[1].host, DEFAULT_HOST);
    }

    /// Agent settings are read out of the payload's agent settings rule.
    #[test]
    fn agent_settings_are_read_from_the_payload() {
        let json = r#"{
            "configurations": [{
                "configurationId": "dcr-settings",
                "content": {
                    "kind": "AgentSettings",
                    "settings": [
                        { "name": "MaxDiskQuotaInMB", "value": "10240" },
                        { "name": "OtlpGrpcLogsTracesPort", "value": "4329" },
                        { "name": "OtlpHttpProtobufLogsTracesPort", "value": "4330" }
                    ]
                }
            }]
        }"#;

        let config = crate::amcs::schema::Configurations::from_json(json).expect("should parse");
        let extracted = AgentSettings::from_configurations(&config);

        assert_eq!(extracted.grpc_port.as_deref(), Some("4329"));
        assert_eq!(extracted.http_protobuf_port.as_deref(), Some("4330"));

        let listeners = discover_listeners(&StaticEnvironment::new(), &extracted);
        assert_eq!(listeners[0].port, 4329);
        assert_eq!(listeners[1].port, 4330);
    }

    /// A payload with no agent settings rule yields empty settings and therefore defaults.
    #[test]
    fn payload_without_agent_settings_yields_defaults() {
        let json = r#"{ "configurations": [{ "configurationId": "dcr-1", "content": {} }] }"#;

        let config = crate::amcs::schema::Configurations::from_json(json).expect("should parse");
        let extracted = AgentSettings::from_configurations(&config);

        assert!(extracted.grpc_port.is_none());
        assert!(extracted.http_protobuf_port.is_none());

        let listeners = discover_listeners(&StaticEnvironment::new(), &extracted);
        assert_eq!(listeners[0].port, DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT);
    }

    /// A literal IP address is used directly, without going through name resolution.
    #[test]
    fn literal_ip_resolves_without_lookup() {
        let listener = OtlpEventListenerInfo {
            host: "0.0.0.0".to_string(),
            port: 4319,
            protocol: OtlpProtocol::Grpc,
        };

        let addr = listener.socket_addr().expect("literal IP should resolve");
        assert_eq!(addr.to_string(), "0.0.0.0:4319");
    }

    /// `localhost` must resolve to a literal address, because the receiver's `listening_addr`
    /// field is a `SocketAddr` and cannot accept a hostname.
    #[test]
    fn localhost_resolves_to_loopback() {
        let listener = OtlpEventListenerInfo {
            host: DEFAULT_HOST.to_string(),
            port: 4319,
            protocol: OtlpProtocol::Grpc,
        };

        let addr = listener.socket_addr().expect("localhost should resolve");
        assert!(addr.ip().is_loopback(), "expected loopback, got {addr}");
        assert_eq!(addr.port(), 4319);
    }

    /// An unresolvable host surfaces as an error rather than a panic.
    #[test]
    fn unresolvable_host_is_an_error() {
        let listener = OtlpEventListenerInfo {
            host: "host.invalid.".to_string(),
            port: 4319,
            protocol: OtlpProtocol::Grpc,
        };

        assert!(matches!(
            listener.socket_addr(),
            Err(crate::Error::InvalidListenerAddress { .. })
        ));
    }
}
