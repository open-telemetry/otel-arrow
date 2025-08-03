use data_engine_expressions::*;
use data_engine_recordset::*;

use crate::*;

pub(crate) struct OtlpAttachedRecords<'a> {
    resource: Option<&'a (dyn MapValue + 'static)>,
    scope: Option<&'a (dyn MapValue + 'static)>,
}

impl<'a> OtlpAttachedRecords<'a> {
    pub fn new(
        resource: Option<&'a (dyn MapValue + 'static)>,
        scope: Option<&'a (dyn MapValue + 'static)>,
    ) -> OtlpAttachedRecords<'a> {
        Self { resource, scope }
    }
}

impl AttachedRecords for OtlpAttachedRecords<'_> {
    fn get_attached_record(&self, name: &str) -> Option<&(dyn MapValue + 'static)> {
        match name {
            "resource" => self.resource,
            "instrumentation_scope" | "scope" => self.scope,
            _ => None,
        }
    }
}

impl AsStaticValue for Resource {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::Map(self)
    }
}

impl MapValue for Resource {
    fn is_empty(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        1
    }

    fn contains_key(&self, key: &str) -> bool {
        key == "Attributes"
    }

    fn get(&self, key: &str) -> Option<&(dyn AsStaticValue + 'static)> {
        if key == "Attributes" {
            return Some(&self.attributes as &dyn AsStaticValue);
        }

        None
    }

    fn get_items(&self, item_callback: &mut dyn KeyValueCallback) -> bool {
        item_callback.next("Attributes", Value::Map(&self.attributes))
    }
}

impl AsStaticValue for InstrumentationScope {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::Map(self)
    }
}

impl MapValue for InstrumentationScope {
    fn is_empty(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        (self.name.is_some() as usize) + (self.version.is_some() as usize) + 1
    }

    fn contains_key(&self, key: &str) -> bool {
        matches!(key, "Attributes" | "Name" | "Version")
    }

    fn get(&self, key: &str) -> Option<&(dyn AsStaticValue + 'static)> {
        match key {
            "Attributes" => Some(&self.attributes as &dyn AsStaticValue),
            "Name" => self.name.as_ref().map(|v| v as &dyn AsStaticValue),
            "Version" => self.version.as_ref().map(|v| v as &dyn AsStaticValue),
            _ => None,
        }
    }

    fn get_items(&self, item_callback: &mut dyn KeyValueCallback) -> bool {
        if let Some(v) = &self.name {
            if !item_callback.next("Name", Value::String(v)) {
                return false;
            }
        }
        if let Some(v) = &self.version {
            if !item_callback.next("Version", Value::String(v)) {
                return false;
            }
        }
        item_callback.next("Attributes", Value::Map(&self.attributes))
    }
}
