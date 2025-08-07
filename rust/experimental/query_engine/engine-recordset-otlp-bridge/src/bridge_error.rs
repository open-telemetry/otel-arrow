use thiserror::Error;

use crate::SerializerError;

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Pipeline '{0}' could not be found")]
    PipelineNotFound(usize),

    #[error("Error encountered reading OTLP Protobuf request: {0}")]
    OtlpProtobufReadError(SerializerError),

    #[error("Error encountered writing OTLP Protobuf response: {0}")]
    OtlpProtobufWriteError(SerializerError),
}
