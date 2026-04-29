use std::collections::HashMap;

use data_engine_recordset::*;

use crate::{proto::common::AnyValue, serializer::ProtobufField};

#[derive(Debug, Clone)]
pub struct Resource {
    pub attributes: MapValueStorage<AnyValue>,
    pub(crate) extra_fields: Vec<ProtobufField>,
}

impl Default for Resource {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource {
    pub fn new() -> Resource {
        Self {
            attributes: MapValueStorage::new(HashMap::new()),
            extra_fields: Vec::new(),
        }
    }

    pub fn with_attribute(mut self, key: &str, value: AnyValue) -> Resource {
        if !key.is_empty() {
            self.attributes.get_values_mut().insert(key.into(), value);
        }
        self
    }
}
