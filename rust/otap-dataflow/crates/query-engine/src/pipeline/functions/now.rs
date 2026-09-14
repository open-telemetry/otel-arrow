// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::time::{SystemTime, UNIX_EPOCH};

use arrow::datatypes::{DataType, TimeUnit};
use datafusion::common::exec_err;
use datafusion::error::Result;
use datafusion::logical_expr::{ColumnarValue, ScalarUDFImpl, Signature, Volatility};
use datafusion::scalar::ScalarValue;

/// Scalar UDF implementation that evaluates to the current time.
///
/// Unlike the UDF datafusion `now` scalar built into datafusion, this does not try to inline
/// the expression to a static result representing when the query was planned. This means an
/// instance of the physical expr invoking this UDF can be reused for many invocations and will
/// evaluate to the actual time the function was invoked.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NowFunc {
    signature: Signature,
}

impl Default for NowFunc {
    fn default() -> Self {
        Self::new()
    }
}

impl NowFunc {
    pub fn new() -> Self {
        Self {
            signature: Signature::nullary(Volatility::Volatile),
        }
    }
}

impl ScalarUDFImpl for NowFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "now"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Ok(DataType::Timestamp(TimeUnit::Nanosecond, None))
    }

    fn invoke_with_args(
        &self,
        args: datafusion::logical_expr::ScalarFunctionArgs,
    ) -> Result<ColumnarValue> {
        if !args.args.is_empty() {
            return exec_err!(
                "{} function does not accept arguments, received {} args",
                self.name(),
                args.args.len()
            );
        }

        let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return exec_err!("system time before unix epoch");
        };

        Ok(ColumnarValue::Scalar(ScalarValue::TimestampNanosecond(
            i64::try_from(now.as_nanos()).ok(),
            None,
        )))
    }
}
