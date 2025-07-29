// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::error;
use crate::otap::OtapArrowRecords;
use crate::otlp::attributes::store::Attribute16Store;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

pub struct RelatedData {
    pub(crate) log_record_id: u16,

    pub(crate) res_attr_map_store: Option<Attribute16Store>,
    pub(crate) scope_attr_map_store: Option<Attribute16Store>,
    pub(crate) log_record_attr_map_store: Option<Attribute16Store>,
}

impl<'a> TryFrom<&'a OtapArrowRecords> for RelatedData {
    type Error = error::Error;

    fn try_from(otap_batch: &'a OtapArrowRecords) -> error::Result<Self> {
        Ok(Self {
            log_record_id: 0,
            res_attr_map_store: otap_batch
                .get(ArrowPayloadType::ResourceAttrs)
                .map(Attribute16Store::try_from)
                .transpose()?,
            scope_attr_map_store: otap_batch
                .get(ArrowPayloadType::ScopeAttrs)
                .map(Attribute16Store::try_from)
                .transpose()?,
            log_record_attr_map_store: otap_batch
                .get(ArrowPayloadType::LogAttrs)
                .map(Attribute16Store::try_from)
                .transpose()?,
        })
    }
}

impl RelatedData {
    pub fn log_record_id_from_delta(&mut self, delta: u16) -> u16 {
        self.log_record_id += delta;
        self.log_record_id
    }
}
