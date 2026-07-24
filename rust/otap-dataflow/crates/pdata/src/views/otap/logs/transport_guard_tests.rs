// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::test_util::encoded_field;
use super::*;
use arrow::array::{ArrayRef, StringArray, StructArray, UInt8Array, UInt16Array};
use arrow::datatypes::{DataType, Field, Schema};
use otap_df_pdata_views::views::common::{AnyValueView, AttributeView};
use otap_df_pdata_views::views::resource::ResourceView;
use std::sync::Arc;

fn logs_batch_with_id_encodings(
    log_id_encoding: &str,
    resource_id_encoding: &str,
    scope_id_encoding: &str,
) -> RecordBatch {
    let log_id_field = encoded_field(consts::ID, DataType::UInt16, false, log_id_encoding);
    let resource_id_field =
        encoded_field(consts::ID, DataType::UInt16, false, resource_id_encoding);
    let scope_id_field = encoded_field(consts::ID, DataType::UInt16, false, scope_id_encoding);

    let schema = Arc::new(Schema::new(vec![
        log_id_field,
        Field::new(
            consts::RESOURCE,
            DataType::Struct(vec![resource_id_field.clone()].into()),
            false,
        ),
        Field::new(
            consts::SCOPE,
            DataType::Struct(vec![scope_id_field.clone()].into()),
            false,
        ),
    ]));

    let resource_struct = StructArray::from(vec![(
        Arc::new(resource_id_field),
        Arc::new(UInt16Array::from(vec![1, 1])) as ArrayRef,
    )]);
    let scope_struct = StructArray::from(vec![(
        Arc::new(scope_id_field),
        Arc::new(UInt16Array::from(vec![1, 1])) as ArrayRef,
    )]);

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt16Array::from(vec![1, 2])),
            Arc::new(resource_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap()
}

fn logs_batch_with_plain_resource_ids(resource_ids: &[u16]) -> RecordBatch {
    let log_id_field = encoded_field(
        consts::ID,
        DataType::UInt16,
        false,
        consts::metadata::encodings::PLAIN,
    );
    let resource_id_field = encoded_field(
        consts::ID,
        DataType::UInt16,
        false,
        consts::metadata::encodings::PLAIN,
    );
    let scope_id_field = encoded_field(
        consts::ID,
        DataType::UInt16,
        false,
        consts::metadata::encodings::PLAIN,
    );

    let schema = Arc::new(Schema::new(vec![
        log_id_field,
        Field::new(
            consts::RESOURCE,
            DataType::Struct(vec![resource_id_field.clone()].into()),
            false,
        ),
        Field::new(
            consts::SCOPE,
            DataType::Struct(vec![scope_id_field.clone()].into()),
            false,
        ),
    ]));

    let resource_struct = StructArray::from(vec![(
        Arc::new(resource_id_field),
        Arc::new(UInt16Array::from_iter_values(resource_ids.iter().copied())) as ArrayRef,
    )]);
    let scope_struct = StructArray::from(vec![(
        Arc::new(scope_id_field),
        Arc::new(UInt16Array::from_iter_values(std::iter::repeat_n(
            1,
            resource_ids.len(),
        ))) as ArrayRef,
    )]);

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt16Array::from_iter_values(
                (1..=resource_ids.len()).map(|idx| u16::try_from(idx).unwrap()),
            )),
            Arc::new(resource_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap()
}

fn attrs_batch_with_parent_encoding_and_values(
    encoding: &str,
    attrs: &[(u16, &str, &str)],
) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        encoded_field(consts::PARENT_ID, DataType::UInt16, false, encoding),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt16Array::from_iter_values(
                attrs.iter().map(|(parent_id, _, _)| *parent_id),
            )),
            Arc::new(StringArray::from(
                attrs.iter().map(|(_, key, _)| *key).collect::<Vec<_>>(),
            )),
            Arc::new(UInt8Array::from_iter_values(
                attrs.iter().map(|_| AttributeValueType::Str as u8),
            )),
            Arc::new(StringArray::from(
                attrs
                    .iter()
                    .map(|(_, _, value)| Some(*value))
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .unwrap()
}

fn attrs_batch_with_parent_encoding(encoding: &str) -> RecordBatch {
    attrs_batch_with_parent_encoding_and_values(encoding, &[(1, "service.name", "test-service")])
}

fn empty_attrs_batch_with_parent_encoding(encoding: &str) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        encoded_field(consts::PARENT_ID, DataType::UInt16, false, encoding),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt16Array::from_iter_values(vec![])),
            Arc::new(StringArray::from(Vec::<Option<&str>>::new())),
            Arc::new(UInt8Array::from_iter_values(vec![])),
            Arc::new(StringArray::from(Vec::<Option<&str>>::new())),
        ],
    )
    .unwrap()
}

fn logs_batch_with_plain_ids() -> RecordBatch {
    logs_batch_with_id_encodings(
        consts::metadata::encodings::PLAIN,
        consts::metadata::encodings::PLAIN,
        consts::metadata::encodings::PLAIN,
    )
}

fn assert_transport_error<T>(
    result: Result<T, Error>,
    expected_payload_type: ArrowPayloadType,
    expected_column: &str,
) {
    match result {
        Err(Error::TransportOptimizedIdsNotDecoded {
            payload_type,
            column,
        }) => {
            assert_eq!(payload_type, expected_payload_type);
            assert_eq!(column, expected_column);
        }
        Err(err) => panic!("unexpected error: {err:?}"),
        Ok(_) => panic!("expected transport-optimized ID guard error"),
    }
}

/// Scenario: A logs view is constructed with an encoded root ID column.
/// Guarantees: Construction reports the offending root column in a typed transport error.
#[test]
fn rejects_encoded_log_id_columns() {
    for (log_id_encoding, resource_id_encoding, scope_id_encoding, expected_column) in [
        (
            consts::metadata::encodings::DELTA,
            consts::metadata::encodings::PLAIN,
            consts::metadata::encodings::PLAIN,
            consts::ID,
        ),
        (
            consts::metadata::encodings::PLAIN,
            consts::metadata::encodings::DELTA,
            consts::metadata::encodings::PLAIN,
            RESOURCE_ID_COL_PATH,
        ),
        (
            consts::metadata::encodings::PLAIN,
            consts::metadata::encodings::PLAIN,
            consts::metadata::encodings::DELTA,
            SCOPE_ID_COL_PATH,
        ),
    ] {
        let batch =
            logs_batch_with_id_encodings(log_id_encoding, resource_id_encoding, scope_id_encoding);

        assert_transport_error(
            OtapLogsView::new(Some(&batch), None, None, None),
            ArrowPayloadType::Logs,
            expected_column,
        );
    }
}

/// Scenario: A logs view is constructed with an encoded attribute parent ID column.
/// Guarantees: Construction reports the offending attribute payload in a typed transport error.
#[test]
fn rejects_encoded_attr_parent_id_columns() {
    let logs_batch = logs_batch_with_plain_ids();
    let resource_attrs = attrs_batch_with_parent_encoding(consts::metadata::encodings::QUASI_DELTA);
    let scope_attrs = attrs_batch_with_parent_encoding(consts::metadata::encodings::QUASI_DELTA);
    let log_attrs = attrs_batch_with_parent_encoding(consts::metadata::encodings::QUASI_DELTA);

    assert_transport_error(
        OtapLogsView::new(Some(&logs_batch), Some(&resource_attrs), None, None),
        ArrowPayloadType::ResourceAttrs,
        consts::PARENT_ID,
    );
    assert_transport_error(
        OtapLogsView::new(Some(&logs_batch), None, Some(&scope_attrs), None),
        ArrowPayloadType::ScopeAttrs,
        consts::PARENT_ID,
    );

    assert_transport_error(
        OtapLogsView::new(Some(&logs_batch), None, None, Some(&log_attrs)),
        ArrowPayloadType::LogAttrs,
        consts::PARENT_ID,
    );
}

/// Scenario: Transport-optimized log records are decoded before view construction.
/// Guarantees: Construction fails before decoding and succeeds after all transport IDs are plain.
#[test]
fn allows_view_after_decode() {
    let mut otap_records = OtapArrowRecords::Logs(Default::default());
    otap_records
        .set(ArrowPayloadType::Logs, logs_batch_with_plain_ids())
        .unwrap();
    otap_records
        .set(
            ArrowPayloadType::ResourceAttrs,
            attrs_batch_with_parent_encoding(consts::metadata::encodings::PLAIN),
        )
        .unwrap();
    otap_records
        .set(
            ArrowPayloadType::ScopeAttrs,
            attrs_batch_with_parent_encoding(consts::metadata::encodings::PLAIN),
        )
        .unwrap();
    otap_records
        .set(
            ArrowPayloadType::LogAttrs,
            attrs_batch_with_parent_encoding(consts::metadata::encodings::PLAIN),
        )
        .unwrap();

    otap_records.encode_transport_optimized().unwrap();
    assert!(matches!(
        OtapLogsView::try_from(&otap_records),
        Err(Error::TransportOptimizedIdsNotDecoded { .. })
    ));

    otap_records.decode_transport_optimized_ids().unwrap();
    let _logs_view = OtapLogsView::try_from(&otap_records)
        .expect("decoded transport IDs should allow logs view creation");
}

/// Scenario: An empty encoded log-attribute batch is decoded before view construction.
/// Guarantees: Decoding marks the empty parent ID column plain and permits view construction.
#[test]
fn allows_view_after_decode_with_empty_attr_batch() {
    let mut otap_records = OtapArrowRecords::Logs(Default::default());
    otap_records
        .set(ArrowPayloadType::Logs, logs_batch_with_plain_ids())
        .unwrap();
    otap_records
        .set(
            ArrowPayloadType::LogAttrs,
            empty_attrs_batch_with_parent_encoding(consts::metadata::encodings::QUASI_DELTA),
        )
        .unwrap();

    otap_records.decode_transport_optimized_ids().unwrap();
    let _logs_view = OtapLogsView::try_from(&otap_records)
        .expect("decoded empty transport child batches should allow logs view creation");
}

/// Scenario: A resource-only view is built from records with other encoded child payloads.
/// Guarantees: Only resource data is decoded and the expected resource attribute remains readable.
#[test]
fn resource_only_view_decodes_only_resource_batches() {
    let mut otap_records = OtapArrowRecords::Logs(Default::default());
    otap_records
        .set(ArrowPayloadType::Logs, logs_batch_with_plain_ids())
        .unwrap();
    otap_records
        .set(
            ArrowPayloadType::ResourceAttrs,
            attrs_batch_with_parent_encoding(consts::metadata::encodings::PLAIN),
        )
        .unwrap();
    otap_records
        .set(
            ArrowPayloadType::ScopeAttrs,
            attrs_batch_with_parent_encoding(consts::metadata::encodings::PLAIN),
        )
        .unwrap();
    otap_records
        .set(
            ArrowPayloadType::LogAttrs,
            attrs_batch_with_parent_encoding(consts::metadata::encodings::PLAIN),
        )
        .unwrap();

    otap_records.encode_transport_optimized().unwrap();
    assert!(matches!(
        OtapLogsView::try_from(&otap_records),
        Err(Error::TransportOptimizedIdsNotDecoded { .. })
    ));

    let decoded_resources = DecodedOtapLogsResources::clone_and_decode(&otap_records)
        .expect("resource-only decode should ignore scope/log child batches");
    let resources_view = decoded_resources
        .resources_view()
        .expect("resource-only view should only require resource ID columns");

    let mut resources = resources_view.resources();
    let resource_logs = resources.next().expect("expected one resource");
    let resource = resource_logs.resource();
    let mut attrs = resource.attributes();
    let attr = attrs.next().expect("expected resource attribute");

    assert_eq!(attr.key(), b"service.name");
    let value = attr.value().expect("attribute should have a value");
    assert_eq!(value.as_string(), Some("test-service".as_bytes()));
    assert!(attrs.next().is_none());
    assert!(resources.next().is_none());
}

/// Scenario: Resource attributes are decoded while filtering for one requested key.
/// Guarantees: Each resource exposes only the requested attribute and its decoded value.
#[test]
fn resource_only_view_keyed_decode_keeps_only_requested_resource_attr() {
    let mut otap_records = OtapArrowRecords::Logs(Default::default());
    otap_records
        .set(
            ArrowPayloadType::Logs,
            logs_batch_with_plain_resource_ids(&[1, 2]),
        )
        .unwrap();
    otap_records
        .set(
            ArrowPayloadType::ResourceAttrs,
            attrs_batch_with_parent_encoding_and_values(
                consts::metadata::encodings::PLAIN,
                &[
                    (1, "deployment.environment", "prod"),
                    (2, "deployment.environment", "prod"),
                    (1, "service.name", "shared-service"),
                    (2, "service.name", "shared-service"),
                ],
            ),
        )
        .unwrap();

    otap_records.encode_transport_optimized().unwrap();

    let decoded_resources =
        DecodedOtapLogsResources::clone_and_decode_keyed(&otap_records, b"service.name")
            .expect("keyed resource-only decode should decode matching resource attrs");
    let resources_view = decoded_resources
        .resources_view()
        .expect("keyed resource-only view should decode parent IDs for matching attrs");

    let mut resources = resources_view.resources();
    for _ in 0..2 {
        let resource_logs = resources.next().expect("expected resource");
        let resource = resource_logs.resource();
        let mut attrs = resource.attributes();
        let attr = attrs.next().expect("expected requested resource attribute");

        assert_eq!(attr.key(), b"service.name");
        let value = attr.value().expect("attribute should have a value");
        assert_eq!(value.as_string(), Some("shared-service".as_bytes()));
        assert!(attrs.next().is_none());
    }
    assert!(resources.next().is_none());
}

/// Scenario: A keyed resource decode requests an attribute absent from every resource.
/// Guarantees: The resource view remains valid and exposes empty attribute iterators.
#[test]
fn resource_only_view_keyed_decode_allows_missing_requested_resource_attr() {
    let mut otap_records = OtapArrowRecords::Logs(Default::default());
    otap_records
        .set(
            ArrowPayloadType::Logs,
            logs_batch_with_plain_resource_ids(&[1, 2]),
        )
        .unwrap();
    otap_records
        .set(
            ArrowPayloadType::ResourceAttrs,
            attrs_batch_with_parent_encoding_and_values(
                consts::metadata::encodings::PLAIN,
                &[
                    (1, "deployment.environment", "prod"),
                    (2, "deployment.environment", "prod"),
                ],
            ),
        )
        .unwrap();

    otap_records.encode_transport_optimized().unwrap();

    let decoded_resources =
        DecodedOtapLogsResources::clone_and_decode_keyed(&otap_records, b"service.name")
            .expect("keyed resource-only decode should allow missing requested attrs");
    let resources_view = decoded_resources
        .resources_view()
        .expect("keyed resource-only view should allow an empty filtered attr batch");

    let mut resources = resources_view.resources();
    for _ in 0..2 {
        let resource_logs = resources.next().expect("expected resource");
        let resource = resource_logs.resource();
        assert!(resource.attributes().next().is_none());
    }
    assert!(resources.next().is_none());
}

/// Scenario: A keyed resource-only view is built without a root Logs payload.
/// Guarantees: The missing root is treated as an empty resource set instead of an error.
#[test]
fn resource_only_view_missing_logs_batch_yields_empty_view() {
    let otap_records = OtapArrowRecords::Logs(Default::default());

    let decoded_resources =
        DecodedOtapLogsResources::clone_and_decode_keyed(&otap_records, b"service.name")
            .expect("missing root should decode as an empty resource set");
    let resources_view = decoded_resources
        .resources_view()
        .expect("missing root should yield an empty resource-only view");

    assert!(resources_view.resources().next().is_none());
}
