use thiserror::Error;

use crate::{Expression, QueryLocation};

#[derive(Error, Debug)]
pub enum ExpressionError {
    #[error("{1}")]
    TypeMismatch(QueryLocation, String),
}

impl Expression for ExpressionError {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            ExpressionError::TypeMismatch(l, _) => l,
        }
    }
}
