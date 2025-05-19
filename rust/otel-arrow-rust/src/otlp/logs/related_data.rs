// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::decode::record_message::RecordMessage;
use crate::error;
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

    /// returns an instance of self and also the index of the main log record
    /// in the input array.
    pub fn from_record_messages(rbs: &[RecordMessage]) -> error::Result<(Self, Option<usize>)> {
        let mut related_data = RelatedData::default();
        let mut logs_record_idx: Option<usize> = None;

        for (idx, rm) in rbs.iter().enumerate() {
            match rm.payload_type {
                ArrowPayloadType::Logs => logs_record_idx = Some(idx),
                ArrowPayloadType::ResourceAttrs => {
                    related_data.res_attr_map_store = Attribute16Store::try_from(&rm.record)?;
                }
                ArrowPayloadType::ScopeAttrs => {
                    related_data.scope_attr_map_store = Attribute16Store::try_from(&rm.record)?;
                }
                ArrowPayloadType::LogAttrs => {
                    related_data.log_record_attr_map_store =
                        Attribute16Store::try_from(&rm.record)?;
                }
                _ => {
                    return error::UnsupportedPayloadTypeSnafu {
                        actual: rm.payload_type,
                    }
                    .fail();
                }
            }
        }

        Ok((related_data, logs_record_idx))
    }
}
