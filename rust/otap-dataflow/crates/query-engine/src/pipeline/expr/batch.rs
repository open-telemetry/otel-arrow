// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains types and utilities for expressions that evaluate on an entire batch at a
//! time as opposed to (columns of data contained within the batch). For example, these types of
//! expressions could be evaluating the type of signal contained in the batch, retrieving values
//! from batch metadata. Generally when evaluated, these types of expressions will produce a scalar
//! value.

use datafusion::scalar::ScalarValue;

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};

use crate::error::Result;
use crate::pipeline::expr::eval::EvalContext;

mod metadata;

/// An evaluation of a property of the entire `OtapArrowRecords` batch.
///
/// Results are uniform across all rows.
pub(crate) trait BatchEval: std::fmt::Debug {
    /// Evaluate against the entire batch.
    fn evaluate(
        &self,
        otap_batch: &OtapArrowRecords,
        eval_ctx: &EvalContext<'_>,
    ) -> Result<ScalarValue>;
}

/// Checks whether the batch contains a specific signal type.
#[derive(Debug)]
pub(crate) struct SignalTypePredicate {
    pub signal_type: SignalType,
}

impl SignalTypePredicate {
    pub fn new(signal_type: SignalType) -> Self {
        Self { signal_type }
    }
}

impl BatchEval for SignalTypePredicate {
    fn evaluate(
        &self,
        otap_batch: &OtapArrowRecords,
        _eval_ctx: &EvalContext<'_>,
    ) -> Result<ScalarValue> {
        Ok(ScalarValue::Boolean(Some(
            otap_batch.signal_type() == self.signal_type,
        )))
    }
}
