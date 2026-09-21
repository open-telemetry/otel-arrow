// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementation of the Views traits for OTAP Arrow RecordBatches.
//!
//! This provides zero-copy views over OTAP columnar data, abstracting away the internal
//! structure of Arrow RecordBatches. It enables direct iteration over the data using
//! a hierarchical OTLP-like interface (Resource -> Scope -> LogRecord) without exposing
//! the complexity of the raw Arrow batches or requiring conversion to intermediate formats.

use std::borrow::Cow;

use crate::error::Result;
use crate::otap::OtapArrowRecords;
use crate::otap::transform::transport_optimize::first_transport_encoded_id_column;

pub mod common;
pub(crate) mod logs;
pub(crate) mod metrics;
pub(crate) mod traces;

pub use logs::{DecodedOtapLogsResources, OtapLogsResourcesView, OtapLogsView};
pub use metrics::{OtapMetricsView, otap_metrics_have_aggregatable_metrics};
pub use traces::OtapTracesView;

/// Keeps an OTAP batch available after transport-optimized IDs have been decoded.
///
/// Views borrow from their input records. This wrapper lets callers preserve an
/// original payload for forwarding or NACK handling. Plain records are borrowed
/// directly; transport-optimized records are cloned and decoded.
pub struct DecodedOtapArrowRecords<'a> {
    records: Cow<'a, OtapArrowRecords>,
}

impl<'a> DecodedOtapArrowRecords<'a> {
    /// Borrow plain records, or clone and decode every transport-optimized ID column.
    pub fn clone_and_decode(records: &'a OtapArrowRecords) -> Result<Self> {
        if transport_ids_are_plain(records) {
            return Ok(Self {
                records: Cow::Borrowed(records),
            });
        }

        let mut records = records.clone();
        records.decode_transport_optimized_ids()?;
        Ok(Self {
            records: Cow::Owned(records),
        })
    }

    /// Decode owned records without an additional clone.
    pub fn decode(mut records: OtapArrowRecords) -> Result<Self> {
        records.decode_transport_optimized_ids()?;
        Ok(Self {
            records: Cow::Owned(records),
        })
    }

    /// Borrow the decoded records.
    #[must_use]
    pub fn records(&self) -> &OtapArrowRecords {
        self.records.as_ref()
    }

    /// Build a logs view over the decoded records.
    pub fn logs_view(&self) -> Result<OtapLogsView<'_>> {
        OtapLogsView::try_from(self.records.as_ref())
    }

    /// Build a metrics view over the decoded records.
    pub fn metrics_view(&self) -> Result<OtapMetricsView<'_>> {
        OtapMetricsView::try_from(self.records.as_ref())
    }

    /// Build a traces view over the decoded records.
    pub fn traces_view(&self) -> Result<OtapTracesView<'_>> {
        OtapTracesView::try_from(self.records.as_ref())
    }

    #[cfg(test)]
    fn is_borrowed(&self) -> bool {
        matches!(&self.records, Cow::Borrowed(_))
    }

    #[cfg(test)]
    fn is_owned(&self) -> bool {
        matches!(&self.records, Cow::Owned(_))
    }
}

fn transport_ids_are_plain(records: &OtapArrowRecords) -> bool {
    records.allowed_payload_types().iter().all(|payload_type| {
        records.get(*payload_type).is_none_or(|record_batch| {
            first_transport_encoded_id_column(*payload_type, record_batch.schema_ref()).is_none()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otap::{Logs, Metrics, OtapBatchStore, Traces, from_record_messages};
    use crate::otlp::metrics::MetricType;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::record_batch;
    use crate::schema::{consts, update_field_metadata};
    use crate::testing::{fixtures, round_trip};
    use crate::{Consumer, Producer};
    use otel_arrow_dfe_pdata_views::views::logs::{
        LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
    };
    use otel_arrow_dfe_pdata_views::views::metrics::MetricsView;
    use otel_arrow_dfe_pdata_views::views::trace::TracesView;

    fn transport_round_trip(mut records: OtapArrowRecords) -> OtapArrowRecords {
        let mut producer = Producer::new();
        let mut bar = producer
            .produce_bar(&mut records)
            .expect("encode OTAP transport batch");
        let messages = Consumer::default()
            .consume_bar(&mut bar)
            .expect("decode OTAP transport batch");

        match records {
            OtapArrowRecords::Logs(_) => {
                OtapArrowRecords::Logs(from_record_messages::<Logs>(messages).expect("logs"))
            }
            OtapArrowRecords::Metrics(_) => OtapArrowRecords::Metrics(
                from_record_messages::<Metrics>(messages).expect("metrics"),
            ),
            OtapArrowRecords::Traces(_) => {
                OtapArrowRecords::Traces(from_record_messages::<Traces>(messages).expect("traces"))
            }
        }
    }

    /// Scenario: Clone-and-decode receives records whose canonical transport IDs are plain.
    /// Guarantees: The wrapper borrows the input instead of cloning its batch store.
    #[test]
    fn decoded_owner_borrows_plain_records() {
        let records = round_trip::encode_logs(&fixtures::logs_with_full_resource_and_scope());

        let decoded = DecodedOtapArrowRecords::clone_and_decode(&records).expect("plain records");

        assert!(decoded.is_borrowed());
        assert!(std::ptr::eq(decoded.records(), &records));
    }

    /// Scenario: Wire-decoded logs retain transport-optimized IDs and include record attributes.
    /// Guarantees: Direct view construction fails, while the decoded owner restores attribute links.
    #[test]
    fn decoded_owner_makes_transport_logs_safe_to_view() {
        let records = transport_round_trip(round_trip::encode_logs(
            &fixtures::logs_with_varying_attributes_and_properties(4),
        ));

        assert!(matches!(
            OtapLogsView::try_from(&records),
            Err(crate::error::Error::TransportOptimizedIdsNotDecoded { .. })
        ));

        let decoded = DecodedOtapArrowRecords::clone_and_decode(&records).expect("decode IDs");
        assert!(decoded.is_owned());
        let view = decoded.logs_view().expect("decoded logs view");
        let mut attribute_count = 0;
        for resource in view.resources() {
            for scope in resource.scopes() {
                for record in scope.log_records() {
                    attribute_count += record.attributes().count();
                }
            }
        }
        assert!(attribute_count > 0, "record attributes must remain linked");
    }

    /// Scenario: Only the log-attribute parent IDs are marked transport optimized.
    /// Guarantees: The centralized guard identifies the exact child payload and column.
    #[test]
    fn logs_view_rejects_transport_encoded_child_parent_ids() {
        let mut records =
            round_trip::encode_logs(&fixtures::logs_with_varying_attributes_and_properties(2));
        let attrs = records
            .get(ArrowPayloadType::LogAttrs)
            .expect("fixture has log attributes");
        let schema = update_field_metadata(
            attrs.schema_ref(),
            consts::PARENT_ID,
            consts::metadata::COLUMN_ENCODING,
            consts::metadata::encodings::QUASI_DELTA,
        );
        records
            .set(
                ArrowPayloadType::LogAttrs,
                arrow::record_batch::RecordBatch::try_new(
                    std::sync::Arc::new(schema),
                    attrs.columns().to_vec(),
                )
                .expect("metadata-only schema replacement"),
            )
            .expect("replace log attributes");

        assert!(matches!(
            OtapLogsView::try_from(&records),
            Err(crate::error::Error::TransportOptimizedIdsNotDecoded {
                payload_type: ArrowPayloadType::LogAttrs,
                column,
            }) if column == consts::PARENT_ID
        ));
    }

    /// Scenario: Wire-decoded metrics contain transport-optimized hierarchy and child IDs.
    /// Guarantees: Direct view construction fails, while the decoded owner produces a usable view.
    #[test]
    fn decoded_owner_makes_transport_metrics_safe_to_view() {
        let records = transport_round_trip(round_trip::encode_metrics(
            &fixtures::metrics_sum_with_full_resource_and_scope(),
        ));

        assert!(matches!(
            OtapMetricsView::try_from(&records),
            Err(crate::error::Error::TransportOptimizedIdsNotDecoded { .. })
        ));

        let decoded = DecodedOtapArrowRecords::clone_and_decode(&records).expect("decode IDs");
        let view = decoded.metrics_view().expect("decoded metrics view");
        assert!(view.resources().count() > 0);
    }

    /// Scenario: Wire-decoded traces contain transport-optimized hierarchy and child IDs.
    /// Guarantees: Direct view construction fails, while the decoded owner produces a usable view.
    #[test]
    fn decoded_owner_makes_transport_traces_safe_to_view() {
        let records = transport_round_trip(round_trip::encode_traces(
            &fixtures::traces_with_full_resource_and_scope(),
        ));

        assert!(matches!(
            OtapTracesView::try_from(&records),
            Err(crate::error::Error::TransportOptimizedIdsNotDecoded { .. })
        ));

        let decoded = DecodedOtapArrowRecords::clone_and_decode(&records).expect("decode IDs");
        let view = decoded.traces_view().expect("decoded traces view");
        assert!(view.resources().count() > 0);
    }

    /// Scenario: Transport-optimized traces contain empty event and link child batches.
    /// Guarantees: Decoding marks their parent IDs plain and permits traces view construction.
    #[test]
    fn decoded_owner_accepts_empty_transport_trace_children() {
        let mut encoded: OtapArrowRecords = crate::traces!(
            (
                Spans,
                ("id", UInt16, vec![1u16]),
                ("scope.id", UInt16, vec![1u16]),
                ("resource.id", UInt16, vec![1u16])
            ),
            (
                SpanEvents,
                ("id", UInt32, vec![1u32]),
                ("parent_id", UInt16, vec![1u16])
            ),
            (
                SpanLinks,
                ("id", UInt32, vec![1u32]),
                ("parent_id", UInt16, vec![1u16])
            ),
        )
        .into();
        encoded
            .encode_transport_optimized()
            .expect("transport optimize traces");

        for payload_type in [ArrowPayloadType::SpanEvents, ArrowPayloadType::SpanLinks] {
            let mut records = encoded.clone();
            let empty_batch = records
                .get(payload_type)
                .expect("child payload")
                .slice(0, 0);
            records
                .set(payload_type, empty_batch)
                .expect("replace child payload");

            let decoded =
                DecodedOtapArrowRecords::clone_and_decode(&records).expect("decode empty child");
            let _view = decoded.traces_view().expect("traces view");
        }
    }

    /// Scenario: Transport-optimized metrics contain an empty exemplar child batch.
    /// Guarantees: Decoding marks its parent ID plain and permits metrics view construction.
    #[test]
    fn decoded_owner_accepts_empty_transport_exemplars() {
        let mut records: OtapArrowRecords = crate::metrics!(
            (
                UnivariateMetrics,
                ("id", UInt16, vec![1u16]),
                ("resource.id", UInt16, vec![1u16]),
                ("scope.id", UInt16, vec![1u16]),
                ("metric_type", UInt8, vec![MetricType::Gauge as u8])
            ),
            (
                NumberDataPoints,
                ("id", UInt32, vec![1u32]),
                ("parent_id", UInt16, vec![1u16])
            ),
            (
                NumberDpExemplars,
                ("id", UInt32, vec![1u32]),
                ("parent_id", UInt32, vec![1u32])
            ),
        )
        .into();
        records
            .encode_transport_optimized()
            .expect("transport optimize metrics");
        let empty_batch = records
            .get(ArrowPayloadType::NumberDpExemplars)
            .expect("exemplar payload")
            .slice(0, 0);
        records
            .set(ArrowPayloadType::NumberDpExemplars, empty_batch)
            .expect("replace exemplar payload");

        let decoded =
            DecodedOtapArrowRecords::clone_and_decode(&records).expect("decode empty exemplar");
        let _view = decoded.metrics_view().expect("metrics view");
    }
}
