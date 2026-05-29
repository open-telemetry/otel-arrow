// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use arrow::array::RecordBatch;

use crate::error::Error;
use crate::otap::transform::transport_optimize::RESOURCE_ID_COL_PATH;
use crate::otlp::attributes::Attribute16Arrays;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use crate::views::otap::common::{
    OtapAttributeIter, OtapAttributeView, RowGroup, build_attribute_index,
    ensure_plain_encoded_columns, group_by_resource_id,
};
use otap_df_pdata_views::views::resource::ResourceView;

/// Resource-only view over OTAP logs Arrow RecordBatches.
pub struct OtapLogsResourcesView<'a> {
    resource_attrs: Option<Attribute16Arrays<'a>>,
    resource_groups: Vec<(u16, RowGroup)>,
    resource_attrs_map: BTreeMap<u16, Vec<usize>>,
}

impl<'a> OtapLogsResourcesView<'a> {
    pub(super) fn new(
        logs_batch: &'a RecordBatch,
        resource_attrs: Option<&'a RecordBatch>,
    ) -> Result<Self, Error> {
        ensure_plain_encoded_columns(ArrowPayloadType::Logs, logs_batch, &[RESOURCE_ID_COL_PATH])?;
        if let Some(batch) = resource_attrs {
            ensure_plain_encoded_columns(
                ArrowPayloadType::ResourceAttrs,
                batch,
                &[consts::PARENT_ID],
            )?;
        }

        let resource_groups = group_by_resource_id(logs_batch);
        let resource_attrs_map = resource_attrs
            .map(build_attribute_index)
            .unwrap_or_default();

        Ok(Self {
            resource_attrs: resource_attrs
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            resource_groups,
            resource_attrs_map,
        })
    }

    /// Iterate over distinct resources in the logs batch.
    #[must_use]
    pub fn resources(&self) -> OtapLogsResourcesIter<'_> {
        OtapLogsResourcesIter {
            view: self,
            resource_groups: self.resource_groups.iter(),
        }
    }
}

/// Iterator over resource-level log groups.
pub struct OtapLogsResourcesIter<'a> {
    view: &'a OtapLogsResourcesView<'a>,
    resource_groups: std::slice::Iter<'a, (u16, RowGroup)>,
}

impl<'a> Iterator for OtapLogsResourcesIter<'a> {
    type Item = OtapLogsResource<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (resource_id, _) = self.resource_groups.next()?;
        Some(OtapLogsResource {
            view: self.view,
            resource_id: *resource_id,
        })
    }
}

/// Resource-level entry in an OTAP logs batch.
pub struct OtapLogsResource<'a> {
    view: &'a OtapLogsResourcesView<'a>,
    resource_id: u16,
}

impl<'a> OtapLogsResource<'a> {
    /// Access this log group's resource.
    #[must_use]
    pub fn resource(&self) -> OtapLogsResourceOnlyResourceView<'_> {
        OtapLogsResourceOnlyResourceView {
            view: self.view,
            resource_id: self.resource_id,
        }
    }
}

/// Resource view backed by the resource-only logs view.
pub struct OtapLogsResourceOnlyResourceView<'a> {
    view: &'a OtapLogsResourcesView<'a>,
    resource_id: u16,
}

impl<'a> ResourceView for OtapLogsResourceOnlyResourceView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;

    type AttributesIter<'att>
        = OtapAttributeIter<'att>
    where
        Self: 'att;

    #[inline]
    fn attributes(&self) -> Self::AttributesIter<'_> {
        let matching_rows = self
            .view
            .resource_attrs_map
            .get(&self.resource_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        OtapAttributeIter {
            attrs: self.view.resource_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        0
    }
}
