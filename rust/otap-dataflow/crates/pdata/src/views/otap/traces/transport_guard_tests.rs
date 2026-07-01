// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::otap::OtapBatchStore;
use crate::otap::transform::transport_optimize::Encoding;
use crate::record_batch;
use crate::views::otap::transport_guard_test_util::{assert_transport_error, set_column_encoding};

fn traces_records() -> OtapArrowRecords {
    let store = crate::traces!(
        (
            Spans,
            ("id", UInt16, vec![1u16, 2]),
            ("resource.id", UInt16, vec![1u16, 1]),
            ("scope.id", UInt16, vec![10u16, 10])
        ),
        (ResourceAttrs, ("parent_id", UInt16, vec![1u16])),
        (ScopeAttrs, ("parent_id", UInt16, vec![10u16])),
        (SpanAttrs, ("parent_id", UInt16, vec![1u16])),
        (
            SpanEvents,
            ("id", UInt32, vec![100u32]),
            ("parent_id", UInt16, vec![1u16]),
            ("name", Utf8, vec!["event"])
        ),
        (SpanEventAttrs, ("parent_id", UInt32, vec![100u32])),
        (
            SpanLinks,
            ("id", UInt32, vec![200u32]),
            ("parent_id", UInt16, vec![1u16])
        ),
        (SpanLinkAttrs, ("parent_id", UInt32, vec![200u32])),
    );
    store.into()
}

#[test]
fn rejects_encoded_trace_root_id_columns() {
    for column in [consts::ID, RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH] {
        let mut records = traces_records();
        set_column_encoding(
            &mut records,
            ArrowPayloadType::Spans,
            column,
            Encoding::Delta,
        );

        assert_transport_error(
            OtapTracesView::try_from(&records),
            ArrowPayloadType::Spans,
            column,
        );
    }
}

#[test]
fn rejects_encoded_trace_attr_parent_id_columns() {
    for payload_type in [
        ArrowPayloadType::ResourceAttrs,
        ArrowPayloadType::ScopeAttrs,
        ArrowPayloadType::SpanAttrs,
        ArrowPayloadType::SpanEventAttrs,
        ArrowPayloadType::SpanLinkAttrs,
    ] {
        let mut records = traces_records();
        set_column_encoding(
            &mut records,
            payload_type,
            consts::PARENT_ID,
            Encoding::AttributeQuasiDelta,
        );

        assert_transport_error(
            OtapTracesView::try_from(&records),
            payload_type,
            consts::PARENT_ID,
        );
    }
}

#[test]
fn rejects_encoded_trace_event_and_link_id_columns() {
    for (payload_type, column) in [
        (ArrowPayloadType::SpanEvents, consts::ID),
        (ArrowPayloadType::SpanEvents, consts::PARENT_ID),
        (ArrowPayloadType::SpanLinks, consts::ID),
        (ArrowPayloadType::SpanLinks, consts::PARENT_ID),
    ] {
        let mut records = traces_records();
        set_column_encoding(&mut records, payload_type, column, Encoding::Delta);

        assert_transport_error(OtapTracesView::try_from(&records), payload_type, column);
    }
}

#[test]
fn allows_traces_view_after_decode() {
    let mut records = traces_records();

    records.encode_transport_optimized().unwrap();
    assert!(matches!(
        OtapTracesView::try_from(&records),
        Err(Error::TransportOptimizedIdsNotDecoded { .. })
    ));

    records.decode_transport_optimized_ids().unwrap();
    let _view = OtapTracesView::try_from(&records)
        .expect("decoded transport IDs should allow traces view creation");
}
