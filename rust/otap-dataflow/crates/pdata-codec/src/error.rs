// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::error::Error as StdError;

use crate::PdataEncoding;

/// Operation being performed by a codec.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecOperation {
    /// Convert encoded bytes to native OTAP.
    Decode,
    /// Convert native OTAP to encoded bytes.
    Encode,
}

impl std::fmt::Display for CodecOperation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Decode => "decode",
            Self::Encode => "encode",
        })
    }
}

/// Invalid link-time codec registry.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RegistryError {
    /// An encoding name does not satisfy the identity contract.
    #[error("invalid pdata encoding `{encoding}`: {reason}")]
    InvalidIdentity {
        /// Invalid identity.
        encoding: PdataEncoding,
        /// Validation failure.
        reason: &'static str,
    },
    /// More than one extension registered the same identity.
    #[error("duplicate pdata encoding registration `{encoding}`")]
    Duplicate {
        /// Duplicated identity.
        encoding: PdataEncoding,
    },
    /// A registration cannot perform any operation.
    #[error("pdata encoding `{encoding}` has no decoder or encoder")]
    EmptyCapabilities {
        /// Empty registration identity.
        encoding: PdataEncoding,
    },
    /// A registration has no supported signals.
    #[error("pdata encoding `{encoding}` has no supported signals")]
    EmptySignals {
        /// Invalid registration identity.
        encoding: PdataEncoding,
    },
    /// The requested identity is not linked into this binary.
    #[error("no pdata codec registered for `{encoding}`")]
    NotFound {
        /// Missing identity.
        encoding: PdataEncoding,
    },
}

/// Codec resolution or execution failure.
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    /// Registry validation or lookup failed.
    #[error(transparent)]
    Registry(#[from] RegistryError),
    /// A registered codec does not support an operation for this signal.
    #[error("pdata codec `{encoding}` cannot {operation} {signal:?}")]
    Unsupported {
        /// Codec identity.
        encoding: PdataEncoding,
        /// Requested operation.
        operation: CodecOperation,
        /// Requested signal.
        signal: otel_arrow_dfe_config::SignalType,
    },
    /// A codec failed while processing telemetry.
    #[error("pdata codec `{encoding}` failed to {operation}: {source}")]
    Operation {
        /// Codec identity.
        encoding: PdataEncoding,
        /// Failed operation.
        operation: CodecOperation,
        /// Original error, retained for diagnostics.
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
    /// A decoder returned a different signal from the admitted envelope.
    #[error("pdata codec `{encoding}` decoded {actual:?}, expected {expected:?}")]
    SignalChanged {
        /// Codec identity.
        encoding: PdataEncoding,
        /// Signal admitted by the receiver.
        expected: otel_arrow_dfe_config::SignalType,
        /// Signal produced by the decoder.
        actual: otel_arrow_dfe_config::SignalType,
    },
}

impl CodecError {
    /// Wraps a codec implementation error without losing its source chain.
    pub fn operation(
        encoding: &PdataEncoding,
        operation: CodecOperation,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self::Operation {
            encoding: encoding.clone(),
            operation,
            source: Box::new(source),
        }
    }
}
