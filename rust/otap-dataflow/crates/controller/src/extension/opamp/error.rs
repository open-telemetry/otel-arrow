// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration type definitions for OpAMP Controller extension.

/// Error type used in OpAMP Controller Extension
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("instance_uid is not valid UUID: {reason}")]
    InvalidInstanceUid { reason: String },

    #[error("endpoint is not valid: {reason}")]
    InvalidEndpoint { reason: String },
}
