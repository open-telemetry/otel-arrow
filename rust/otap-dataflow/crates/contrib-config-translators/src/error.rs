// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors produced while translating a vendor configuration into an
//! [`OtelDataflowSpec`](otel_arrow_dfe_config::engine::OtelDataflowSpec).

/// Errors that can occur while translating a vendor configuration.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The supplied configuration could not be deserialized.
    #[error("failed to deserialize {format} configuration: {details}")]
    Deserialization {
        /// The serialization format that was attempted (for example `JSON`).
        format: &'static str,
        /// The underlying parser error.
        details: String,
    },

    /// The translated pipeline specification could not be serialized to YAML.
    #[error("failed to serialize the generated pipeline specification to YAML: {details}")]
    Serialization {
        /// The underlying serializer error.
        details: String,
    },

    /// The translated configuration did not produce a usable pipeline.
    #[error("the configuration did not yield any pipeline nodes: {details}")]
    EmptyPipeline {
        /// Why no nodes could be produced.
        details: String,
    },

    /// The generated pipeline specification was rejected by the engine config model.
    #[error("the generated pipeline specification is invalid: {0}")]
    InvalidPipeline(#[from] Box<otel_arrow_dfe_config::error::Error>),

    /// A listener address could not be resolved to a socket address.
    #[error("cannot resolve listener address `{host}:{port}`: {details}")]
    InvalidListenerAddress {
        /// The configured host.
        host: String,
        /// The configured port.
        port: u16,
        /// Why resolution failed.
        details: String,
    },
}

impl From<otel_arrow_dfe_config::error::Error> for Error {
    fn from(value: otel_arrow_dfe_config::error::Error) -> Self {
        Self::InvalidPipeline(Box::new(value))
    }
}
