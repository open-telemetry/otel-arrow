// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal gRPC-style status type to avoid pulling in tonic for the OTEL receiver.
//! The receiver only needs a handful of status codes plus message formatting to
//! drive trailer generation and logging.

use std::error::Error;
use std::fmt;

/// Subset of gRPC status codes used by the lightweight OTEL receiver.
///
/// This lets us avoid pulling in `tonic` only to represent status values.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StatusCode {
    #[allow(unused)]
    Ok = 0,
    #[allow(unused)]
    Cancelled = 1,
    #[allow(unused)]
    Unknown = 2,
    InvalidArgument = 3,
    DeadlineExceeded = 4,
    ResourceExhausted = 8,
    Unimplemented = 12,
    Internal = 13,
    Unavailable = 14,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Status {
    code: StatusCode,
    message: String,
}

impl Status {
    pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> StatusCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    #[allow(unused)]
    pub fn into_parts(self) -> (StatusCode, String) {
        (self.code, self.message)
    }

    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new(StatusCode::InvalidArgument, message)
    }

    pub fn unimplemented(message: impl Into<String>) -> Self {
        Self::new(StatusCode::Unimplemented, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::Internal, message)
    }

    pub fn deadline_exceeded(message: impl Into<String>) -> Self {
        Self::new(StatusCode::DeadlineExceeded, message)
    }

    pub fn resource_exhausted(message: impl Into<String>) -> Self {
        Self::new(StatusCode::ResourceExhausted, message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::Unavailable, message)
    }

    #[allow(unused)]
    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::new(StatusCode::Cancelled, message)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            write!(f, "{:?}", self.code)
        } else {
            write!(f, "{:?}: {}", self.code, self.message)
        }
    }
}

impl Error for Status {}
