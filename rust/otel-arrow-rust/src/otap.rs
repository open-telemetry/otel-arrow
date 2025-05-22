// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains various types and methods for interacting with and manipulating
//! OTAP data / record batches

use arrow::array::RecordBatch;

use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

enum OtapBatch {
    Logs(Logs),
}

struct Logs {
    batches: [RecordBatch; 4],
}

impl Logs {
    // TODO reconfirm these const types are only defined once statically
    const TYPE_MASK: u64 = 0u64
        + (1 << ArrowPayloadType::ResourceAttrs as u64)
        + (1 << ArrowPayloadType::ScopeAttrs as u64)
        + (1 << ArrowPayloadType::Logs as u64)
        + (1 << ArrowPayloadType::LogAttrs as u64);

    fn is_valid_type(payload_type: ArrowPayloadType) -> bool {
        Self::TYPE_MASK & (1 << payload_type as u64) != 0
    }

    // TODO think a bit deeper about if this should warn/error, or just ignore the invalid type
    fn set(&mut self, payload_type: ArrowPayloadType, record_batch: RecordBatch) {
        if Self::is_valid_type(payload_type) {
            self.batches[POSITION_LOOKUP[payload_type as usize]] = record_batch;
        }
    }

    fn get(&self, payload_type: ArrowPayloadType) -> Option<&RecordBatch> {
        if !Self::is_valid_type(payload_type) {
            None
        } else {
            self.batches.get(POSITION_LOOKUP[payload_type as usize])
        }
    }
}

// TODO comment on what is going on here
const POSITION_LOOKUP: &[usize] = &[
    // Unknown shouldn't ever ben in record batche
    // (so doesn't really matter what value is here)
    99, // Unknown = 0,
    // common:
    0, // ResourceAttrs = 1,
    1, // ScopeAttrs = 2,
    // metrics:
    2,  // UnivariateMetrics = 10,
    3,  // NumberDataPoints = 11,
    4,  // SummaryDataPoints = 12,
    5,  // HistogramDataPoints = 13,
    6,  // ExpHistogramDataPoints = 14,
    7,  // NumberDpAttrs = 15,
    8,  // SummaryDpAttrs = 16,
    9,  // HistogramDpAttrs = 17,
    10, // ExpHistogramDpAttrs = 18,
    11, // NumberDpExemplars = 19,
    12, // HistogramDpExemplars = 20,
    13, // ExpHistogramDpExemplars = 21,
    14, // NumberDpExemplarAttrs = 22,
    15, // HistogramDpExemplarAttrs = 23,
    16, // ExpHistogramDpExemplarAttrs = 24,
    17, // MultivariateMetrics = 25,
    // logs:
    2, // Logs = 30,
    3, // LogAttrs = 31,
    // traces:
    2, // Spans = 40,
    3, // SpanAttrs = 41,
    4, // SpanEvents = 42,
    5, // SpanLinks = 43,
    6, // SpanEventAttrs = 44,
    7, // SpanLinkAttrs = 45,
];
