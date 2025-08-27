// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/common/arrow/attributes.go#L40

use crate::otlp::attributes::parent_id::ParentId;
use crate::proto::opentelemetry::common::v1::any_value;
pub type Attrs16ParentIdDecoder = AttrsParentIdDecoder<u16>;
pub type Attrs32ParentIdDecoder = AttrsParentIdDecoder<u32>;

// AttrsParentIdDecoder implements parent_id decoding for attribute
// sets.  The parent_id in this case is the entity which refers to the
// set of attributes (e.g., a resource, a scope, a metric) contained
// in a RecordBatch.
//
// Phase 1 note: there were several experimental encoding schemes
// tested.  Two schemes named "ParentIdDeltaEncoding",
// "ParentIdNoEncoding" have been removed.
pub struct AttrsParentIdDecoder<T> {
    prev_parent_id: T,
    prev_key: Option<String>,
    prev_value: Option<any_value::Value>,
}

impl<T> Default for AttrsParentIdDecoder<T>
where
    T: ParentId,
{
    fn default() -> Self {
        Self {
            prev_parent_id: T::default(),
            prev_key: None,
            prev_value: None,
        }
    }
}

impl<T> AttrsParentIdDecoder<T>
where
    T: ParentId,
{
    pub fn decode(&mut self, delta_or_parent_id: T, key: &str, value: &any_value::Value) -> T {
        if self.prev_key.as_deref() == Some(key) && self.prev_value.as_ref() == Some(value) {
            let parent_id = self.prev_parent_id.add(delta_or_parent_id);
            self.prev_parent_id = parent_id;
            parent_id
        } else {
            self.prev_key = Some(key.to_string());
            self.prev_value = Some(value.clone());
            self.prev_parent_id = delta_or_parent_id;
            delta_or_parent_id
        }
    }
}
