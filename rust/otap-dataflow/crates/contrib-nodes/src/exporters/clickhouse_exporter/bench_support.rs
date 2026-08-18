// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmark-only access to the ClickHouse logs transformation implementations.
//!
//! Criterion benchmarks are compiled as separate crates, so they cannot access the exporter's
//! crate-private transformers directly. This feature-gated bridge exposes only the operations
//! needed to compare the generic and specialized implementations without making those
//! implementation details part of the production API.

use arrow::array::RecordBatch;
use bytes::Bytes;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::{OtapArrowRecords, OtapPayload, OtlpProtoBytes, TryIntoWithOptions};

use super::transform::logs_fast::{LogsFastTransform, LogsFastTransformer};
use super::transform::logs_otlp::OtlpLogsTransformer;
use super::transform::transform_batch::BatchTransformer;

/// Owns reusable state for comparing the generic and specialized logs transformers.
#[derive(Default)]
pub struct LogsTransformBenchmark {
    generic: BatchTransformer,
    fast: LogsFastTransformer,
    otlp: OtlpLogsTransformer,
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

    /// Transform raw OTLP logs through the legacy OTAP Arrow conversion path.
    #[must_use]
    pub fn transform_otlp_legacy(&mut self, request: Bytes) -> RecordBatch {
        let payload: OtapPayload = OtlpProtoBytes::ExportLogsRequest(request).into();
        let mut records: OtapArrowRecords = payload
            .try_into_with_default()
            .expect("convert benchmark OTLP logs to OTAP Arrow");
        records
            .decode_transport_optimized_ids()
            .expect("decode benchmark transport-optimized IDs");
        self.transform_generic(records)
    }

    /// Transform raw OTLP logs directly to ClickHouse columns.
    #[must_use]
    pub fn transform_otlp_direct(&mut self, request: &[u8]) -> RecordBatch {
        self.otlp
            .transform(request)
            .expect("direct ClickHouse OTLP logs transform")
            .expect("benchmark input contains logs")
    }
}
