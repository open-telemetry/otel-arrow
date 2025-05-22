// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::decode::record_message::RecordMessage;
use crate::error;
use crate::otap::OtapBatch;
use crate::otlp::attributes::store::Attribute16Store;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

#[derive(Default)]
pub struct RelatedData {
    pub(crate) log_record_id: u16,

    pub(crate) res_attr_map_store: Attribute16Store,
    pub(crate) scope_attr_map_store: Attribute16Store,
    pub(crate) log_record_attr_map_store: Attribute16Store,
}

impl RelatedData {
    pub fn log_record_id_from_delta(&mut self, delta: u16) -> u16 {
        self.log_record_id += delta;
        self.log_record_id
    }
}

impl TryFrom<&OtapBatch> for RelatedData {
    type Error = error::Error;

    fn try_from(otap_batch: &OtapBatch) -> error::Result<Self> {
        let mut related_data = Self::default();

        if let Some(rb) = otap_batch.get(ArrowPayloadType::ResourceAttrs) {
            related_data.res_attr_map_store = Attribute16Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::ScopeAttrs) {
            related_data.scope_attr_map_store = Attribute16Store::try_from(rb)?;
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::LogAttrs) {
            related_data.log_record_attr_map_store = Attribute16Store::try_from(rb)?;
        }

        Ok(related_data)
    }
}
