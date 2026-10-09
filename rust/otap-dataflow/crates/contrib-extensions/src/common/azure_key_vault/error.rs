// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Safe, bounded acquisition diagnostics. Raw SDK errors are never retained.

use azure_core::error::ErrorKind;
use otel_arrow_dfe_telemetry_macros::AttributeEnum;

/// Which part of startup acquisition failed, without exposing secret names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum Stage {
    /// Azure identity initialization.
    Identity,
    /// Key Vault client initialization.
    Client,
    /// Username secret read or validation.
    Username,
    /// Password secret read or validation.
    Password,
    /// Whole credential acquisition.
    Acquisition,
}

/// Closed error categories safe for events, errors, and measurement attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum FailureKind {
    /// Identity could not authenticate.
    Authentication,
    /// Identity lacks secret-read access.
    Authorization,
    /// Secret or selected version does not exist.
    NotFound,
    /// Service throttling.
    Throttled,
    /// Network or server failure.
    Transient,
    /// Missing, empty, disabled, expired, or not-yet-valid secret value.
    InvalidValue,
    /// Malformed response or authentication challenge.
    InvalidResponse,
    /// Acquisition exceeded the startup timeout.
    Timeout,
    /// Client construction failed.
    Configuration,
    /// Another service/SDK failure.
    Service,
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Stage {
    /// Stable stage label for operational events.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Client => "client",
            Self::Username => "username",
            Self::Password => "password",
            Self::Acquisition => "acquisition",
        }
    }
}

impl std::fmt::Display for FailureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FailureKind {
    /// Stable category label, with no response body, token, or SDK error text.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authentication => "authentication",
            Self::Authorization => "authorization",
            Self::NotFound => "not_found",
            Self::Throttled => "throttled",
            Self::Transient => "transient",
            Self::InvalidValue => "invalid_value",
            Self::InvalidResponse => "invalid_response",
            Self::Timeout => "timeout",
            Self::Configuration => "configuration",
            Self::Service => "service",
        }
    }
}

/// An acquisition failure containing only a stage and a safe category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Azure Key Vault {stage} acquisition failed ({kind})")]
pub struct Error {
    /// Failing acquisition stage.
    pub stage: Stage,
    /// Safe error category.
    pub kind: FailureKind,
}

impl Error {
    /// Emits a protocol-neutral failure event containing only bounded labels.
    pub fn log_failure(&self) {
        otel_arrow_dfe_telemetry::otel_warn!(
            "azure_key_vault.acquisition_failed",
            stage = self.stage.as_str(),
            category = self.kind.as_str(),
        );
    }

    /// Drops all raw SDK error data, including its source chain and response.
    #[must_use]
    pub fn from_sdk(stage: Stage, error: azure_core::Error) -> Self {
        let kind = match error.kind() {
            ErrorKind::Credential => FailureKind::Authentication,
            ErrorKind::HttpResponse { status, .. } => match u16::from(*status) {
                401 => FailureKind::Authentication,
                403 => FailureKind::Authorization,
                404 => FailureKind::NotFound,
                408 | 500..=599 => FailureKind::Transient,
                429 => FailureKind::Throttled,
                _ => FailureKind::Service,
            },
            ErrorKind::Connection | ErrorKind::Io => FailureKind::Transient,
            ErrorKind::DataConversion => FailureKind::InvalidResponse,
            _ => FailureKind::Service,
        };
        Self { stage, kind }
    }
}
