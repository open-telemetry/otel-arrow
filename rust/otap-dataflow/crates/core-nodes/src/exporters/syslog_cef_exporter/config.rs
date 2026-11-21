// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_dfe_config::error::Error as ConfigError;
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

/// Default timeout for individual transport I/O operations (5 seconds).
pub const DEFAULT_IO_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for exporting an entire batch of messages (30 seconds).
pub const DEFAULT_BATCH_TIMEOUT: Duration = Duration::from_secs(30);

const fn default_io_timeout() -> Duration {
    DEFAULT_IO_TIMEOUT
}

const fn default_batch_timeout() -> Duration {
    DEFAULT_BATCH_TIMEOUT
}

/// Framing method for syslog messages over TCP.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TcpFraming {
    /// Delimited by a newline character (`\n`).
    #[default]
    Newline,
    /// RFC 6587 octet counting (message length prefix followed by space and message).
    OctetCounting,
}

/// TCP transport configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TcpConfig {
    /// Remote destination address formatted as `host:port` (accepts IP addresses and DNS hostnames).
    pub endpoint: String,
    /// Framing mode for TCP messages.
    #[serde(default)]
    pub framing: TcpFraming,
}

/// UDP transport configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UdpConfig {
    /// Remote destination address formatted as `host:port` (accepts IP addresses and DNS hostnames).
    pub endpoint: String,
}

/// Selected transport protocol.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    /// TCP transport.
    Tcp(TcpConfig),
    /// UDP transport.
    Udp(UdpConfig),
}

/// Configuration for the syslog/CEF exporter.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyslogCefExporterConfig {
    /// Transport protocol configuration (TCP or UDP).
    pub protocol: Protocol,
    /// Timeout for individual socket I/O operations (connect, send, write).
    #[serde(default = "default_io_timeout", with = "humantime_serde")]
    pub io_timeout: Duration,
    /// Overall timeout for exporting an entire batch.
    #[serde(default = "default_batch_timeout", with = "humantime_serde")]
    pub batch_timeout: Duration,
}

impl SyslogCefExporterConfig {
    /// Parses and validates user configuration from JSON.
    pub fn parse(value: &Value) -> Result<Self, ConfigError> {
        let mut config: Self = serde_json::from_value(value.clone()).map_err(|error| {
            ConfigError::InvalidUserConfig {
                error: format!("Invalid syslog_cef exporter config: {error}"),
            }
        })?;
        config.validate_and_sanitize()?;
        Ok(config)
    }

    /// Validates configuration fields.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut copy = self.clone();
        copy.validate_and_sanitize()
    }

    fn validate_and_sanitize(&mut self) -> Result<(), ConfigError> {
        match &mut self.protocol {
            Protocol::Tcp(tcp) => sanitize_and_validate_endpoint(&mut tcp.endpoint)?,
            Protocol::Udp(udp) => sanitize_and_validate_endpoint(&mut udp.endpoint)?,
        }
        if self.io_timeout.is_zero() {
            return Err(ConfigError::InvalidUserConfig {
                error: "io_timeout must be greater than zero".into(),
            });
        }
        if self.batch_timeout.is_zero() {
            return Err(ConfigError::InvalidUserConfig {
                error: "batch_timeout must be greater than zero".into(),
            });
        }
        Ok(())
    }
}

fn sanitize_and_validate_endpoint(endpoint: &mut String) -> Result<(), ConfigError> {
    *endpoint = endpoint.trim().to_string();
    if endpoint.is_empty() {
        return Err(ConfigError::InvalidUserConfig {
            error: "endpoint cannot be empty".into(),
        });
    }
    let Some((host, port_str)) = endpoint.rsplit_once(':') else {
        return Err(ConfigError::InvalidUserConfig {
            error: format!("endpoint '{endpoint}' missing port (expected 'host:port')"),
        });
    };
    let host = host.trim_matches(['[', ']']);
    if host.is_empty() {
        return Err(ConfigError::InvalidUserConfig {
            error: format!("endpoint '{endpoint}' has empty host"),
        });
    }
    let Ok(port) = port_str.parse::<u16>() else {
        return Err(ConfigError::InvalidUserConfig {
            error: format!("endpoint '{endpoint}' has invalid port (expected 1-65535)"),
        });
    };
    if port == 0 {
        return Err(ConfigError::InvalidUserConfig {
            error: format!("endpoint '{endpoint}' port cannot be 0 (expected 1-65535)"),
        });
    }
    Ok(())
}
