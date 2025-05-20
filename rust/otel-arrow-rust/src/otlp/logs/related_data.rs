// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::decode::record_message::RecordMessage;
use crate::error;
use crate::otap::OtapBatch;
use crate::otlp::attributes::store::Attribute16Store;
use crate::otlp::attributes::store::AttributeStoreV2;
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

pub struct RelatedDataV2<'a> {
    pub(crate) log_record_id: u16,

    pub(crate) res_attr_map_store: AttributeStoreV2<'a, u16>,
    pub(crate) scope_attr_map_store: AttributeStoreV2<'a, u16>,
    pub(crate) log_record_attr_map_store: AttributeStoreV2<'a, u16>,
    // pub(crate)
}

impl<'a> RelatedDataV2<'a> {
    pub fn log_record_id_from_delta(&mut self, delta: u16) -> u16 {
        self.log_record_id += delta;
        self.log_record_id
    }

    pub fn from_record_messages(rbs: &'a [RecordMessage]) -> error::Result<(Self, Option<usize>)> {
        let mut res_attr_map_store = None;
        let mut scope_attr_map_store = None;
        let mut log_record_attr_map_store = None;
        let mut logs_record_idx: Option<usize> = None;

        for (idx, rm) in rbs.iter().enumerate() {
            match rm.payload_type {
                ArrowPayloadType::Logs => logs_record_idx = Some(idx),
                ArrowPayloadType::ResourceAttrs => {
                    res_attr_map_store = Some(AttributeStoreV2::try_from(&rm.record)?);
                }
                ArrowPayloadType::ScopeAttrs => {
                    scope_attr_map_store = Some(AttributeStoreV2::try_from(&rm.record)?);
                }
                ArrowPayloadType::LogAttrs => {
                    log_record_attr_map_store = Some(AttributeStoreV2::try_from(&rm.record)?);
                }
                _ => {
                    return error::UnsupportedPayloadTypeSnafu {
                        actual: rm.payload_type,
                    }
                    .fail();
                }
            }
        }

        let related_data = Self {
            log_record_id: 0,
            // TODO handle unwrap gracefully here if the message isn't in the list of batches
            res_attr_map_store: res_attr_map_store.unwrap(),
            scope_attr_map_store: scope_attr_map_store.unwrap(),
            log_record_attr_map_store: log_record_attr_map_store.unwrap(),
        };

        Ok((related_data, logs_record_idx))
    }
}
