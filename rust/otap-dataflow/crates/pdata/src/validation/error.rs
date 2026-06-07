// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub type Result<T> = std::result::Result<T, Error>;

/// Errors in the validation tests
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(
        "Receiver operation timed out after {:?}: {:?}",
        super::collector::RECEIVER_TIMEOUT_SECONDS,
        source
    )]
    ReceiverTimeout { source: tokio::time::error::Elapsed },

    #[error(
        "Test timed out after {:?}: {:?}",
        super::collector::TEST_TIMEOUT_SECONDS,
        source
    )]
    TestTimeout { source: tokio::time::error::Elapsed },

    #[error(
        "Collector not ready after {:?}: {:?}",
        super::collector::READY_TIMEOUT_SECONDS,
        source
    )]
    ReadyTimeout { source: tokio::time::error::Elapsed },

    #[error("Channel closed: {}", source)]
    ChannelClosed {
        source: tokio::sync::oneshot::error::RecvError,
    },

    #[error("No response received")]
    NoResponse,

    #[error("Collector exit status: {:?}", code)]
    BadExitStatus { code: Option<i32> },

    #[error("Could not kill: {:?}", source)]
    SignalNotDelivered { source: nix::errno::Errno },

    #[error("Input/output error: {:?}", source)]
    InputOutput {
        desc: &'static str,
        source: std::io::Error,
    },

    #[error("File is not available: {:?}", desc)]
    FileNotAvailable { desc: &'static str },

    #[error("gRPC transport error: {:?}", source)]
    TonicTransport { source: tonic::transport::Error },

    #[error("gRPC status: {:?}", source)]
    TonicStatus { source: tonic::Status },

    #[error("Test pattern {:?} not found in input {:?}", pattern, input)]
    PatternNotFound { pattern: String, input: String },

    #[error("Invalid payload type {:?}", source)]
    InvalidPayload {
        #[from]
        source: prost::UnknownEnumValue,
    },

    #[error("OTel-Arrow error {:?}", source)]
    OTelArrow {
        #[from]
        source: crate::error::Error,
    },

    #[error("Tokio error {:?}", source)]
    Join {
        #[from]
        source: tokio::task::JoinError,
    },
}
