// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::otap::OtapBatchStore;
use crate::otap::transform::transport_optimize::Encoding;
use crate::record_batch;
use crate::views::otap::transport_guard_test_util::{assert_transport_error, set_column_encoding};

fn metrics_records() -> OtapArrowRecords {
    let store = crate::metrics!(
        (
            UnivariateMetrics,
            ("id", UInt16, vec![1u16, 2, 3, 4]),
            ("resource.id", UInt16, vec![1u16, 1, 2, 2]),
            ("scope.id", UInt16, vec![10u16, 10, 20, 20]),
            ("metric_type", UInt8, vec![1u8, 5, 3, 4])
        ),
        (ResourceAttrs, ("parent_id", UInt16, vec![1u16])),
        (ScopeAttrs, ("parent_id", UInt16, vec![10u16])),
        (MetricAttrs, ("parent_id", UInt16, vec![1u16])),
        (
            NumberDataPoints,
            ("id", UInt32, vec![10u32]),
            ("parent_id", UInt16, vec![1u16])
        ),
        (NumberDpAttrs, ("parent_id", UInt32, vec![10u32])),
        (
            NumberDpExemplars,
            ("id", UInt32, vec![11u32]),
            ("parent_id", UInt32, vec![10u32])
        ),
        (NumberDpExemplarAttrs, ("parent_id", UInt32, vec![11u32])),
        (
            SummaryDataPoints,
            ("id", UInt32, vec![20u32]),
            ("parent_id", UInt16, vec![2u16])
        ),
        (SummaryDpAttrs, ("parent_id", UInt32, vec![20u32])),
        (
            HistogramDataPoints,
            ("id", UInt32, vec![30u32]),
            ("parent_id", UInt16, vec![3u16])
        ),
        (HistogramDpAttrs, ("parent_id", UInt32, vec![30u32])),
        (
            HistogramDpExemplars,
            ("id", UInt32, vec![31u32]),
            ("parent_id", UInt32, vec![30u32])
        ),
        (HistogramDpExemplarAttrs, ("parent_id", UInt32, vec![31u32])),
        (
            ExpHistogramDataPoints,
            ("id", UInt32, vec![40u32]),
            ("parent_id", UInt16, vec![4u16])
        ),
        (ExpHistogramDpAttrs, ("parent_id", UInt32, vec![40u32])),
        (
            ExpHistogramDpExemplars,
            ("id", UInt32, vec![41u32]),
            ("parent_id", UInt32, vec![40u32])
        ),
        (
            ExpHistogramDpExemplarAttrs,
            ("parent_id", UInt32, vec![41u32])
        ),
    );
    store.into()
}

fn metrics_records_with_root_types(
    metric_types: Vec<u8>,
    aggregation_temporality: Vec<i32>,
    is_monotonic: Vec<bool>,
) -> OtapArrowRecords {
    let len = metric_types.len();
    let ids: Vec<u16> = (0..len).map(|idx| idx as u16 + 1).collect();
    let resource_ids: Vec<u16> = vec![1; len];
    let scope_ids: Vec<u16> = vec![1; len];

    crate::metrics!(
        (
            UnivariateMetrics,
            ("id", UInt16, ids),
            ("resource.id", UInt16, resource_ids),
            ("scope.id", UInt16, scope_ids),
            ("metric_type", UInt8, metric_types),
            ("aggregation_temporality", Int32, aggregation_temporality),
            ("is_monotonic", Boolean, is_monotonic)
        ),
        (
            NumberDataPoints,
            ("id", UInt32, vec![1u32]),
            ("parent_id", UInt16, vec![1u16])
        ),
    )
    .into()
}

#[test]
fn aggregatable_preflight_detects_gauge_without_transport_decode() {
    let mut records = metrics_records_with_root_types(
        vec![MetricType::Gauge as u8],
        vec![AggregationTemporality::Unspecified as i32],
        vec![false],
    );
    set_column_encoding(
        &mut records,
        ArrowPayloadType::NumberDataPoints,
        consts::PARENT_ID,
        Encoding::Delta,
    );

    assert!(otap_metrics_have_aggregatable_metrics(&records).unwrap());
}

#[test]
fn aggregatable_preflight_skips_delta_sum_without_transport_decode() {
    let mut records = metrics_records_with_root_types(
        vec![MetricType::Sum as u8],
        vec![AggregationTemporality::Delta as i32],
        vec![true],
    );
    set_column_encoding(
        &mut records,
        ArrowPayloadType::NumberDataPoints,
        consts::PARENT_ID,
        Encoding::Delta,
    );

    assert!(!otap_metrics_have_aggregatable_metrics(&records).unwrap());
}

#[test]
fn aggregatable_preflight_detects_cumulative_monotonic_sum() {
    let records = metrics_records_with_root_types(
        vec![MetricType::Sum as u8],
        vec![AggregationTemporality::Cumulative as i32],
        vec![true],
    );

    assert!(otap_metrics_have_aggregatable_metrics(&records).unwrap());
}

#[test]
fn rejects_encoded_metrics_root_id_columns() {
    for column in [consts::ID, RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH] {
        let mut records = metrics_records();
        set_column_encoding(
            &mut records,
            ArrowPayloadType::UnivariateMetrics,
            column,
            Encoding::Delta,
        );

        assert_transport_error(
            OtapMetricsView::try_from(&records),
            ArrowPayloadType::UnivariateMetrics,
            column,
        );
    }
}

#[test]
fn rejects_encoded_metrics_attr_parent_id_columns() {
    for payload_type in [
        ArrowPayloadType::ResourceAttrs,
        ArrowPayloadType::ScopeAttrs,
        ArrowPayloadType::MetricAttrs,
        ArrowPayloadType::NumberDpAttrs,
        ArrowPayloadType::SummaryDpAttrs,
        ArrowPayloadType::HistogramDpAttrs,
        ArrowPayloadType::ExpHistogramDpAttrs,
        ArrowPayloadType::NumberDpExemplarAttrs,
        ArrowPayloadType::HistogramDpExemplarAttrs,
        ArrowPayloadType::ExpHistogramDpExemplarAttrs,
    ] {
        let mut records = metrics_records();
        set_column_encoding(
            &mut records,
            payload_type,
            consts::PARENT_ID,
            Encoding::AttributeQuasiDelta,
        );

        assert_transport_error(
            OtapMetricsView::try_from(&records),
            payload_type,
            consts::PARENT_ID,
        );
    }
}

#[test]
fn rejects_encoded_metrics_child_id_columns() {
    for (payload_type, column) in [
        (ArrowPayloadType::NumberDataPoints, consts::ID),
        (ArrowPayloadType::NumberDataPoints, consts::PARENT_ID),
        (ArrowPayloadType::SummaryDataPoints, consts::ID),
        (ArrowPayloadType::SummaryDataPoints, consts::PARENT_ID),
        (ArrowPayloadType::HistogramDataPoints, consts::ID),
        (ArrowPayloadType::HistogramDataPoints, consts::PARENT_ID),
        (ArrowPayloadType::ExpHistogramDataPoints, consts::ID),
        (ArrowPayloadType::ExpHistogramDataPoints, consts::PARENT_ID),
        (ArrowPayloadType::NumberDpExemplars, consts::ID),
        (ArrowPayloadType::NumberDpExemplars, consts::PARENT_ID),
        (ArrowPayloadType::HistogramDpExemplars, consts::ID),
        (ArrowPayloadType::HistogramDpExemplars, consts::PARENT_ID),
        (ArrowPayloadType::ExpHistogramDpExemplars, consts::ID),
        (ArrowPayloadType::ExpHistogramDpExemplars, consts::PARENT_ID),
    ] {
        let mut records = metrics_records();
        set_column_encoding(&mut records, payload_type, column, Encoding::Delta);

        assert_transport_error(OtapMetricsView::try_from(&records), payload_type, column);
    }
}

#[test]
fn allows_metrics_view_after_decode() {
    let mut records = metrics_records();

    records.encode_transport_optimized().unwrap();
    assert!(matches!(
        OtapMetricsView::try_from(&records),
        Err(Error::TransportOptimizedIdsNotDecoded { .. })
    ));

    records.decode_transport_optimized_ids().unwrap();
    let _view = OtapMetricsView::try_from(&records)
        .expect("decoded transport IDs should allow metrics view creation");
}
