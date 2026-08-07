// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmark-only access to the ClickHouse logs transformation implementations.

use arrow::array::RecordBatch;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use super::transform::logs_fast::{LogsFastTransform, LogsFastTransformer};
use super::transform::transform_batch::BatchTransformer;

/// Owns reusable state for comparing the generic and specialized logs transformers.
#[derive(Default)]
pub struct LogsTransformBenchmark {
    generic: BatchTransformer,
    fast: LogsFastTransformer,
}

impl LogsTransformBenchmark {
    /// Transform one logs batch with the generic transformation plan.
    #[must_use]
    pub fn transform_generic(&mut self, records: OtapArrowRecords) -> RecordBatch {
        self.generic
            .apply_plan(records)
            .expect("generic ClickHouse logs transform")
            .remove(&ArrowPayloadType::Logs)
            .expect("generic transform produced a logs batch")
    }

    /// Transform one decoded canonical OTAP logs batch with the specialized path.
    #[must_use]
    pub fn transform_fast(&mut self, records: &OtapArrowRecords) -> RecordBatch {
        match self
            .fast
            .try_apply(records)
            .expect("specialized ClickHouse logs transform")
        {
            LogsFastTransform::Applied(batch) => batch,
            LogsFastTransform::NotApplicable(reason) => {
                panic!("benchmark input is not supported by the specialized transform: {reason}")
            }
        }
    }
}
