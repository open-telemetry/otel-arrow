// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/common/arrow/attributes.go#L40

use crate::otlp::attributes::parent_id::{ParentId, ParentIdEncoding};
use opentelemetry_proto::tonic::common::v1::any_value;

pub type Attrs16ParentIdDecoder = AttrsParentIdDecoder<u16>;
pub type Attrs32ParentIdDecoder = AttrsParentIdDecoder<u32>;

pub struct AttrsParentIdDecoder<T> {
    encoding_type: ParentIdEncoding,
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
            encoding_type: ParentIdEncoding::ParentIdDeltaGroupEncoding,
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
        match self.encoding_type {
            // Plain encoding
            ParentIdEncoding::ParentIdNoEncoding => delta_or_parent_id,
            // Simply delta
            ParentIdEncoding::ParentIdDeltaEncoding => {
                let decode_parent_id = self.prev_parent_id.add(delta_or_parent_id);
                self.prev_parent_id = decode_parent_id;
                decode_parent_id
            }
            // Key-value scoped delta.
            ParentIdEncoding::ParentIdDeltaGroupEncoding => {
                if self.prev_key.as_deref() == Some(key) && self.prev_value.as_ref() == Some(value)
                {
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
    }
}
