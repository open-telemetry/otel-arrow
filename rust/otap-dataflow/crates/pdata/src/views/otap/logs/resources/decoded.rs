// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{RecordBatch, UInt32Array};
use arrow::compute::take_record_batch;

use crate::arrays::{StringArrayAccessor, get_required_array};
use crate::error::Error;
use crate::otap::OtapArrowRecords;
use crate::otap::transform::transport_optimize::{
    remove_transport_optimized_encodings, remove_transport_optimized_resource_id,
};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;

use super::view::OtapLogsResourcesView;

/// Owns the decoded batches needed for resource-level logs inspection.
///
/// This intentionally decodes only `Logs.resource.id` and
/// `ResourceAttrs.parent_id`, avoiding unrelated scope and log attributes.
pub struct DecodedOtapLogsResources {
    logs_batch: Option<RecordBatch>,
    resource_attrs: Option<RecordBatch>,
}

impl DecodedOtapLogsResources {
    /// Clone and decode the resource-level batches in borrowed OTAP logs records.
    pub fn clone_and_decode(records: &OtapArrowRecords) -> Result<Self, Error> {
        Self::clone_and_decode_with_resource_attrs(records, |batch| {
            remove_transport_optimized_encodings(ArrowPayloadType::ResourceAttrs, batch)
        })
    }

    /// Clone and decode resource-level batches after filtering attributes to one key.
    pub fn clone_and_decode_keyed(
        records: &OtapArrowRecords,
        resource_attr_key: &[u8],
    ) -> Result<Self, Error> {
        Self::clone_and_decode_with_resource_attrs(records, |batch| {
            decode_resource_attrs_for_key(batch, resource_attr_key)
        })
    }

    fn clone_and_decode_with_resource_attrs<F>(
        records: &OtapArrowRecords,
        decode_resource_attrs: F,
    ) -> Result<Self, Error>
    where
        F: FnOnce(&RecordBatch) -> Result<RecordBatch, Error>,
    {
        let Some(logs_batch) = records.get(ArrowPayloadType::Logs) else {
            return Ok(Self {
                logs_batch: None,
                resource_attrs: None,
            });
        };
        let logs_batch = remove_transport_optimized_resource_id(logs_batch)?;
        let resource_attrs = records
            .get(ArrowPayloadType::ResourceAttrs)
            .map(decode_resource_attrs)
            .transpose()?;

        Ok(Self {
            logs_batch: Some(logs_batch),
            resource_attrs,
        })
    }

    /// Create a resource-only logs view over the decoded batches.
    pub fn resources_view(&self) -> Result<OtapLogsResourcesView<'_>, Error> {
        OtapLogsResourcesView::new(self.logs_batch.as_ref(), self.resource_attrs.as_ref())
    }
}

fn decode_resource_attrs_for_key(
    batch: &RecordBatch,
    resource_attr_key: &[u8],
) -> Result<RecordBatch, Error> {
    let batch = filter_resource_attrs_by_key(batch, resource_attr_key)?;
    remove_transport_optimized_encodings(ArrowPayloadType::ResourceAttrs, &batch)
}

fn filter_resource_attrs_by_key(
    batch: &RecordBatch,
    resource_attr_key: &[u8],
) -> Result<RecordBatch, Error> {
    if batch.num_rows() == 0 {
        return Ok(batch.clone());
    }

    let key_array =
        StringArrayAccessor::try_new(get_required_array(batch, consts::ATTRIBUTE_KEY)?)?;
    let mut indices = Vec::new();
    for row_idx in 0..batch.num_rows() {
        if key_array
            .str_at(row_idx)
            .is_some_and(|key| key.as_bytes() == resource_attr_key)
        {
            let index = u32::try_from(row_idx).map_err(|_| Error::UnexpectedRecordBatchState {
                reason: format!("resource attribute row index {row_idx} exceeds u32::MAX"),
            })?;
            indices.push(index);
        }
    }

    if indices.len() == batch.num_rows() {
        return Ok(batch.clone());
    }

    let indices = UInt32Array::from_iter_values(indices);
    take_record_batch(batch, &indices).map_err(|err| Error::UnexpectedRecordBatchState {
        reason: format!("failed to filter resource attributes by key: {err}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otap::OtapBatchStore;
    use crate::record_batch;
    use crate::testing::{fixtures, round_trip};
    use otel_arrow_dfe_pdata_views::views::common::{AnyValueView, AttributeView};
    use otel_arrow_dfe_pdata_views::views::resource::ResourceView;

    /// Scenario: A resource-only view filters transport-optimized attributes to one key.
    /// Guarantees: The requested resource attribute remains linked and unrelated keys are absent.
    #[test]
    fn keyed_decode_preserves_requested_resource_attribute() {
        let mut records = round_trip::encode_logs(&fixtures::logs_with_full_resource_and_scope());
        records
            .encode_transport_optimized()
            .expect("transport optimize logs");

        let decoded = DecodedOtapLogsResources::clone_and_decode_keyed(&records, b"res.id")
            .expect("decode keyed resource data");
        let view = decoded.resources_view().expect("resource view");
        let resources = view.resources().collect::<Vec<_>>();
        assert_eq!(resources.len(), 1);

        let resource = resources[0].resource();
        let attributes = resource.attributes().collect::<Vec<_>>();
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key(), b"res.id");
        assert_eq!(
            attributes[0]
                .value()
                .and_then(|value| value.as_string().map(|value| value.as_ref().to_vec())),
            Some(b"self".to_vec())
        );
    }

    /// Scenario: A nonempty logs batch has a non-struct resource column.
    /// Guarantees: Resource-only view construction reports a conversion error.
    #[test]
    fn keyed_decode_rejects_malformed_resource_column() {
        let records: OtapArrowRecords = crate::logs!((
            Logs,
            ("id", UInt16, vec![1u16]),
            ("resource", UInt16, vec![1u16])
        ))
        .into();

        let decoded = DecodedOtapLogsResources::clone_and_decode_keyed(&records, b"res.id")
            .expect("select resource batches");

        assert!(decoded.resources_view().is_err());
    }
}
