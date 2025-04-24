// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display(
        "Receiver operation timed out after {:?}: {:?}",
        super::collector::RECEIVER_TIMEOUT_SECONDS,
        source,
    ))]
    ReceiverTimeout { source: tokio::time::error::Elapsed },

    #[snafu(display(
        "Test timed out after {:?}: {:?}",
        super::collector::TEST_TIMEOUT_SECONDS,
        source,
    ))]
    TestTimeout { source: tokio::time::error::Elapsed },

    #[snafu(display(
        "Collector not ready after {:?}: {:?}",
        super::collector::READY_TIMEOUT_SECONDS,
        source,
    ))]
    ReadyTimeout { source: tokio::time::error::Elapsed },

    #[snafu(display("Channel closed: {}", source))]
    ChannelClosed {
        source: tokio::sync::oneshot::error::RecvError,
    },

    #[snafu(display("No response received"))]
    NoResponse {},

    //#[snafu(display("Collector did not exit: {:?}", source))]
    //UnsuccessfulExit { source: std::process::ExitStatus },

    #[snafu(display("Collector exit status: {:?}", code))]
    BadExitStatus { code: Option<i32> },

    #[snafu(display("Could not kill: {:?}", source))]
    SignalNotDelivered { source: nix::errno::Errno },

    #[snafu(display("Input/output error: {:?}", source))]
    InputOutput {
        desc: &'static str,
        source: std::io::Error,
    },

    #[snafu(display("File is not available: {:?}", desc))]
    FileNotAvailable { desc: &'static str },

    #[snafu(display("gRPC transport error: {:?}", source))]
    TonicTransport { source: tonic::transport::Error },

    #[snafu(display("gRPC status: {:?}", source))]
    TonicStatus { source: tonic::Status },

    #[snafu(display("Test pattern {:?} not found in input {:?}", pattern, input))]
    PatternNotFound { pattern: String, input: String },

    #[snafu(display("Arrow error {:?}", source))]
    Arrow { source: arrow::error::ArrowError },

    #[snafu(display("Empty batch"))]
    EmptyBatch { },

    #[snafu(display("Invalid payload type {:?}", source))]
    InvalidPayload {
	source: prost::UnknownEnumValue,
    },

    #[snafu(display("OTel-Arrow error {:?}", source))]
    OTelArrow {
	source: crate::error::Error,
    },

    #[snafu(display("Tokio error {:?}", source))]
    Join{
	source: tokio::task::JoinError,
    },
}
