use std::{collections::HashMap, mem};

use data_engine_recordset::*;

use data_engine_expressions::*;

use crate::serializer::ProtobufField;

#[derive(Debug, Clone)]
pub struct InstrumentationScope {
    pub name: Option<StringValueStorage>,
    pub version: Option<StringValueStorage>,
    pub attributes: MapValueStorage<AnyValue>,
    pub(crate) extra_fields: Vec<ProtobufField>,
}

impl Default for InstrumentationScope {
    fn default() -> Self {
        Self::new()
    }
}

impl InstrumentationScope {
    pub fn new() -> InstrumentationScope {
        Self {
            name: None,
            version: None,
            attributes: MapValueStorage::new(HashMap::new()),
            extra_fields: Vec::new(),
        }
    }

    pub fn with_name(mut self, value: String) -> InstrumentationScope {
        self.name = Some(StringValueStorage::new(value));
        self
    }

    pub fn with_version(mut self, value: String) -> InstrumentationScope {
        self.version = Some(StringValueStorage::new(value));
        self
    }

    pub fn with_attribute(mut self, key: &str, value: AnyValue) -> InstrumentationScope {
        if !key.is_empty() {
            self.attributes.get_values_mut().insert(key.into(), value);
        }
        self
    }
}

#[derive(Debug, Clone)]
pub enum AnyValue {
    Null,
    Native(OtlpAnyValue),
    Extended(ExtendedValue),
}

impl AsStaticValueMut for AnyValue {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut> {
        match self {
            AnyValue::Native(n) => match n {
                OtlpAnyValue::StringValue(s) => Some(StaticValueMut::String(s)),
                OtlpAnyValue::ArrayValue(a) => Some(StaticValueMut::Array(a)),
                OtlpAnyValue::KvlistValue(m) => Some(StaticValueMut::Map(m)),
                OtlpAnyValue::BytesValue(b) => Some(StaticValueMut::Array(b)),
                _ => None,
            },
            _ => None,
        }
    }
}

impl AsStaticValue for AnyValue {
    fn to_static_value(&self) -> StaticValue {
        match self {
            AnyValue::Null => StaticValue::Null,
            AnyValue::Native(n) => match n {
                OtlpAnyValue::StringValue(s) => StaticValue::String(s),
                OtlpAnyValue::BoolValue(b) => StaticValue::Boolean(b),
                OtlpAnyValue::IntValue(i) => StaticValue::Integer(i),
                OtlpAnyValue::DoubleValue(d) => StaticValue::Double(d),
                OtlpAnyValue::ArrayValue(a) => StaticValue::Array(a),
                OtlpAnyValue::KvlistValue(m) => StaticValue::Map(m),
                OtlpAnyValue::BytesValue(b) => StaticValue::Array(b),
            },
            AnyValue::Extended(e) => match e {
                ExtendedValue::DateTime(d) => StaticValue::DateTime(d),
                ExtendedValue::Regex(r) => StaticValue::Regex(r),
            },
        }
    }
}

impl From<OwnedValue> for AnyValue {
    fn from(value: OwnedValue) -> Self {
        match value {
            OwnedValue::Array(a) => {
                if a.len() > 0 {
                    let mut byte_values = Vec::new();

                    let is_bytes = a.get_items(&mut IndexValueClosureCallback::new(|_, v| {
                        if let Value::Integer(i) = v {
                            let v = i.get_value();
                            if v >= u8::MIN as i64 && v <= u8::MAX as i64 {
                                byte_values.push(IntegerValueStorage::new(v as u8));
                                return true;
                            }
                        }

                        false
                    }));

                    if is_bytes {
                        return AnyValue::Native(OtlpAnyValue::BytesValue(
                            ByteArrayValueStorage::new(byte_values),
                        ));
                    }
                }
                AnyValue::Native(OtlpAnyValue::ArrayValue(a.into()))
            }
            OwnedValue::Boolean(b) => AnyValue::Native(OtlpAnyValue::BoolValue(b)),
            OwnedValue::DateTime(d) => AnyValue::Extended(ExtendedValue::DateTime(d)),
            OwnedValue::Double(d) => AnyValue::Native(OtlpAnyValue::DoubleValue(d)),
            OwnedValue::Integer(i) => AnyValue::Native(OtlpAnyValue::IntValue(i)),
            OwnedValue::Map(m) => AnyValue::Native(OtlpAnyValue::KvlistValue(m.into())),
            OwnedValue::Null => AnyValue::Null,
            OwnedValue::Regex(r) => AnyValue::Extended(ExtendedValue::Regex(r)),
            OwnedValue::String(s) => AnyValue::Native(OtlpAnyValue::StringValue(s)),
        }
    }
}

impl From<AnyValue> for OwnedValue {
    fn from(val: AnyValue) -> Self {
        match val {
            AnyValue::Null => OwnedValue::Null,
            AnyValue::Native(n) => match n {
                OtlpAnyValue::StringValue(s) => OwnedValue::String(s),
                OtlpAnyValue::BoolValue(b) => OwnedValue::Boolean(b),
                OtlpAnyValue::IntValue(i) => OwnedValue::Integer(i),
                OtlpAnyValue::DoubleValue(d) => OwnedValue::Double(d),
                OtlpAnyValue::ArrayValue(a) => OwnedValue::Array(a.into()),
                OtlpAnyValue::KvlistValue(k) => OwnedValue::Map(k.into()),
                OtlpAnyValue::BytesValue(mut b) => OwnedValue::Array(ArrayValueStorage::new(
                    b.values
                        .drain(..)
                        .map(|v| OwnedValue::Integer(IntegerValueStorage::new(v.get_value())))
                        .collect(),
                )),
            },
            AnyValue::Extended(e) => match e {
                ExtendedValue::DateTime(d) => OwnedValue::DateTime(d),
                ExtendedValue::Regex(r) => OwnedValue::Regex(r),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExtendedValue {
    DateTime(DateTimeValueStorage),
    Regex(RegexValueStorage),
}

#[derive(Debug, Clone)]
pub enum OtlpAnyValue {
    StringValue(StringValueStorage),
    BoolValue(BooleanValueStorage),
    IntValue(IntegerValueStorage<i64>),
    DoubleValue(DoubleValueStorage<f64>),
    ArrayValue(ArrayValueStorage<AnyValue>),
    KvlistValue(MapValueStorage<AnyValue>),
    BytesValue(ByteArrayValueStorage),
}

#[derive(Debug, Clone)]
pub struct ByteArrayValueStorage {
    values: Vec<IntegerValueStorage<u8>>,
}

impl ByteArrayValueStorage {
    pub fn new(values: Vec<IntegerValueStorage<u8>>) -> ByteArrayValueStorage {
        Self { values }
    }

    pub fn get_values(&self) -> &Vec<IntegerValueStorage<u8>> {
        &self.values
    }

    pub fn get_values_mut(&mut self) -> &mut Vec<IntegerValueStorage<u8>> {
        &mut self.values
    }
}

impl AsStaticValue for ByteArrayValueStorage {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::Array(self)
    }
}

impl AsStaticValueMut for ByteArrayValueStorage {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut> {
        Some(StaticValueMut::Array(self))
    }
}

impl ArrayValue for ByteArrayValueStorage {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn get(&self, index: usize) -> Option<&(dyn AsStaticValue + 'static)> {
        self.values.get(index).map(|v| v as &dyn AsStaticValue)
    }

    fn get_items(&self, item_callback: &mut dyn IndexValueCallback) -> bool {
        for (index, value) in self.values.iter().enumerate() {
            if !item_callback.next(index, Value::Integer(value)) {
                return false;
            }
        }

        true
    }
}

impl ArrayValueMut for ByteArrayValueStorage {
    fn get_mut(&mut self, _: usize) -> ValueMutGetResult {
        ValueMutGetResult::NotSupported("ByteArray items cannot be mutated individually".into())
    }

    fn set(&mut self, index: usize, value: ResolvedValue) -> ValueMutWriteResult {
        if let Value::Integer(i) = value.to_value() {
            let v = i.get_value();
            if v >= u8::MIN as i64 && v <= u8::MAX as i64 {
                match self.values.get_mut(index) {
                    Some(slot) => {
                        let old = mem::replace(slot, IntegerValueStorage::new(v as u8));
                        return ValueMutWriteResult::Updated(OwnedValue::Integer(
                            IntegerValueStorage::new(old.get_value()),
                        ));
                    }
                    None => {
                        return ValueMutWriteResult::NotFound;
                    }
                }
            }
        }

        ValueMutWriteResult::NotSupported(format!(
            "Cannot set '{:?}' value as index '{index}' because it is not a valid Byte",
            value.get_value_type()
        ))
    }

    fn push(&mut self, value: ResolvedValue) -> ValueMutWriteResult {
        if let Value::Integer(i) = value.to_value() {
            let v = i.get_value();
            if v >= u8::MIN as i64 && v <= u8::MAX as i64 {
                self.values.push(IntegerValueStorage::new(v as u8));
                return ValueMutWriteResult::Created;
            }
        }

        ValueMutWriteResult::NotSupported(format!(
            "Cannot add '{:?}' value to ByteArray because it is not a valid Byte",
            value.get_value_type()
        ))
    }

    fn insert(&mut self, index: usize, value: ResolvedValue) -> ValueMutWriteResult {
        if index > self.values.len() {
            return ValueMutWriteResult::NotFound;
        }

        if let Value::Integer(i) = value.to_value() {
            let v = i.get_value();
            if v >= u8::MIN as i64 && v <= u8::MAX as i64 {
                self.values.insert(index, IntegerValueStorage::new(v as u8));
                return ValueMutWriteResult::Created;
            }
        }

        ValueMutWriteResult::NotSupported(format!(
            "Cannot insert '{:?}' value at index '{index}' because it is not a valid Byte",
            value.get_value_type()
        ))
    }

    fn remove(&mut self, index: usize) -> ValueMutRemoveResult {
        if index >= self.values.len() {
            return ValueMutRemoveResult::NotFound;
        }

        let old = self.values.remove(index);

        ValueMutRemoveResult::Removed(OwnedValue::Integer(IntegerValueStorage::new(
            old.get_value(),
        )))
    }

    fn retain(&mut self, item_callback: &mut dyn IndexValueMutCallback) {
        let mut index = 0;
        self.values.retain_mut(|v| {
            let r = item_callback.next(index, v);
            index += 1;
            r
        });
    }
}
