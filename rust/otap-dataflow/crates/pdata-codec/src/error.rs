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
    /// Merge or split independently decodable batches.
    Batch,
}

impl std::fmt::Display for CodecOperation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Decode => "decode",
            Self::Encode => "encode",
            Self::Batch => "batch",
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
    /// More than one implementation declares itself the default for an encoding.
    #[error("multiple default pdata implementations for `{encoding}`")]
    DuplicateDefault {
        /// Duplicated identity.
        encoding: PdataEncoding,
    },
    /// An implementation identifier is invalid or registered more than once.
    #[error("invalid pdata implementation `{implementation}`: {reason}")]
    InvalidImplementation {
        /// Registered implementation ID.
        implementation: String,
        /// Registration error, without configuration values.
        reason: &'static str,
    },
    /// Several implementations exist without a unique declared default.
    #[error("pdata encoding `{encoding}` requires an explicit implementation selection")]
    AmbiguousImplementation {
        /// Encoding requiring selection.
        encoding: PdataEncoding,
    },
    /// No linked implementation has the requested ID.
    #[error("no pdata implementation registered as `{implementation}`")]
    ImplementationNotFound {
        /// Missing implementation ID.
        implementation: String,
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
    /// A decoder or native item batcher cannot report primary-signal item counts.
    #[error(
        "pdata encoding `{encoding}` requires an item counter for decoding or native item batching"
    )]
    MissingItemCounter {
        /// Invalid registration identity.
        encoding: PdataEncoding,
    },
    /// A native batching declaration is inconsistent.
    #[error("invalid pdata batching registration `{encoding}`: {reason}")]
    InvalidBatching {
        /// Invalid identity.
        encoding: PdataEncoding,
        /// Validation failure.
        reason: String,
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
    /// Invalid startup configuration, without option values or telemetry.
    #[error("invalid pdata codec configuration: {reason}")]
    Configuration {
        /// Configuration path and validation failure.
        reason: String,
    },
    /// Registry validation or lookup failed.
    #[error(transparent)]
    Registry(#[from] RegistryError),
    /// A codec-owned constructed batch violated its admission contract.
    #[error("invalid constructed pdata for `{encoding}`: {reason}")]
    InvalidConstruction {
        /// Target codec identity.
        encoding: PdataEncoding,
        /// Contract violation, without telemetry contents.
        reason: String,
    },
    /// A registered codec does not implement the requested operation.
    #[error("pdata codec `{encoding}` cannot {operation}")]
    UnsupportedCodecOperation {
        /// Codec identity.
        encoding: PdataEncoding,
        /// Requested operation.
        operation: CodecOperation,
    },
    /// A batching plan or codec batch output violated its contract.
    #[error("invalid pdata batching operation: {reason}")]
    InvalidBatch {
        /// Validation or execution failure.
        reason: String,
    },
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
    /// An encoded batch exceeds the output plan's size limit.
    #[error("pdata codec `{encoding}` output size {actual} exceeds limit {limit}")]
    EncodedSizeLimitExceeded {
        /// Output codec identity.
        encoding: PdataEncoding,
        /// Actual encoded size in bytes.
        actual: usize,
        /// Maximum permitted encoded size in bytes.
        limit: usize,
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
    /// Codec state is already borrowed by another synchronous operation.
    #[error(
        "pdata codec service is already in use; nested or concurrent codec access is unsupported"
    )]
    ServiceBusy,
    /// A panic occurred while codec state was borrowed.
    #[error("pdata codec service is unavailable after a panic while its state was borrowed")]
    ServicePoisoned,
}

impl CodecError {
    /// Reports a startup configuration error. Do not include option values.
    pub fn configuration(reason: impl Into<String>) -> Self {
        Self::Configuration {
            reason: reason.into(),
        }
    }

    /// Creates a representation-independent batching error.
    pub fn invalid_batch(reason: impl Into<String>) -> Self {
        Self::InvalidBatch {
            reason: reason.into(),
        }
    }

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

    /// Ensures an implementation failure carries service-owned diagnostics.
    pub(crate) fn with_operation_context(
        self,
        encoding: &PdataEncoding,
        operation: CodecOperation,
    ) -> Self {
        if matches!(
            &self,
            Self::Operation {
                encoding: error_encoding,
                operation: error_operation,
                ..
            } if error_encoding == encoding && *error_operation == operation
        ) {
            self
        } else {
            Self::operation(encoding, operation, self)
        }
    }
}
