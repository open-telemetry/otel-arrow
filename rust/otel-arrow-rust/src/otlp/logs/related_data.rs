// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::error;
use crate::otap::OtapBatch;
use crate::otlp::attributes::store::AttributeStoreV2;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

pub struct RelatedData<'a> {
    pub(crate) log_record_id: u16,

    pub(crate) res_attr_map_store: Option<AttributeStoreV2<'a, u16>>,
    pub(crate) scope_attr_map_store: Option<AttributeStoreV2<'a, u16>>,
    pub(crate) log_record_attr_map_store: Option<AttributeStoreV2<'a, u16>>,
}

impl<'a> TryFrom<&'a OtapBatch> for RelatedData<'a> {
    type Error = error::Error;

    fn try_from(otap_batch: &'a OtapBatch) -> error::Result<Self> {
        Ok(Self {
            log_record_id: 0,
            res_attr_map_store: otap_batch
                .get(ArrowPayloadType::ResourceAttrs)
                .map(AttributeStoreV2::try_from)
                .transpose()?,
            scope_attr_map_store: otap_batch
                .get(ArrowPayloadType::ScopeAttrs)
                .map(AttributeStoreV2::try_from)
                .transpose()?,
            log_record_attr_map_store: otap_batch
                .get(ArrowPayloadType::LogAttrs)
                .map(AttributeStoreV2::try_from)
                .transpose()?,
        })
    }
}

impl<'a> RelatedData<'a> {
    pub fn log_record_id_from_delta(&mut self, delta: u16) -> u16 {
        self.log_record_id += delta;
        self.log_record_id
    }
}
