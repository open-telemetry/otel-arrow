// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

use crate::QueryLocation;

#[derive(Error, Debug)]
pub enum ExpressionError {
    #[error("{1}")]
    TypeMismatch(QueryLocation, String),

    #[error("{1}")]
    ValidationFailure(QueryLocation, String),

    #[error("{1}")]
    ParseError(QueryLocation, String),

    #[error("{1}")]
    NotSupported(QueryLocation, String),
}

impl ExpressionError {
    pub fn get_query_location(&self) -> &QueryLocation {
        match self {
            ExpressionError::TypeMismatch(l, _) => l,
            ExpressionError::ValidationFailure(l, _) => l,
            ExpressionError::ParseError(l, _) => l,
            ExpressionError::NotSupported(l, _) => l,
        }
    }
}
