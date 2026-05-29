// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::RecordBatch;

use crate::error::Error;
use crate::otap::OtapArrowRecords;
use crate::otap::transform::transport_optimize::{
    remove_transport_optimized_encodings, remove_transport_optimized_resource_id,
};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use super::view::OtapLogsResourcesView;

/// Owns the decoded batches needed for resource-level logs inspection.
///
/// This intentionally decodes only `Logs.resource.id` and `ResourceAttrs.parent_id`
/// so routing and validation paths can inspect resources without materializing
/// scope/log attribute IDs or constructing a full logs view.
pub struct DecodedOtapLogsResources {
    logs_batch: RecordBatch,
    resource_attrs: Option<RecordBatch>,
}

impl DecodedOtapLogsResources {
    /// Clone the resource-level batches from borrowed OTAP logs records and decode
    /// the transport-optimized IDs needed to build a resource-only view.
    pub fn clone_and_decode(records: &OtapArrowRecords) -> Result<Self, Error> {
        let logs_batch = records
            .get(ArrowPayloadType::Logs)
            .ok_or(Error::RecordBatchNotFound {
                payload_type: ArrowPayloadType::Logs,
            })?;
        let logs_batch = remove_transport_optimized_resource_id(logs_batch)?;

        let resource_attrs = records
            .get(ArrowPayloadType::ResourceAttrs)
            .map(|batch| {
                remove_transport_optimized_encodings(ArrowPayloadType::ResourceAttrs, batch)
            })
            .transpose()?;

        Ok(Self {
            logs_batch,
            resource_attrs,
        })
    }

    /// Create a resource-only logs view over the decoded batches.
    pub fn resources_view(&self) -> Result<OtapLogsResourcesView<'_>, Error> {
        OtapLogsResourcesView::new(&self.logs_batch, self.resource_attrs.as_ref())
    }
}
