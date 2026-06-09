// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use arrow::datatypes::DataType;
use datafusion::common::exec_err;
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::expr::ScalarFunction;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, Expr, Operator, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl,
    Signature, Volatility,
};

use crate::pipeline::filter::compare::compare;

/// Scalar UDF for invoking [`compare`].
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct CompareFunc {
    operator: Operator,
    signature: Signature,
}

impl CompareFunc {
    pub fn new(operator: Operator) -> Self {
        Self {
            operator,
            signature: Signature::any(2, Volatility::Immutable),
        }
    }

    /// return an [`Expr`] that invokes this function on the given args
    pub fn new_expr(left: Expr, operator: Operator, right: Expr) -> Expr {
        Expr::ScalarFunction(ScalarFunction::new_udf(
            Arc::new(ScalarUDF::new_from_shared_impl(Arc::new(CompareFunc::new(
                operator,
            )))),
            vec![left, right],
        ))
    }
}

impl ScalarUDFImpl for CompareFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "compare"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Boolean)
    }

    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        if args.args.len() != 2 {
            return exec_err!(
                "invalid number of args. received {} expected 2",
                args.args.len()
            );
        }

        let result = compare(&args.args[0], self.operator, &args.args[1])
            .map_err(|e| DataFusionError::Execution(e.to_string()))?;

        Ok(ColumnarValue::Array(Arc::new(result)))
    }

    fn documentation(&self) -> Option<&Documentation> {
        None
    }
}
