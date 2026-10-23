// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batch-local syslog attribute keys with direct appends for the fixed vocabulary.
//! Dynamic CEF keys share Arrow's interner, so duplicate keys and first-seen
//! ordering are identical to ordinary string dictionary construction.

use std::sync::Arc;

use arrow::{
    array::{
        ArrayRef, DictionaryArray, RecordBatch, StringArray, StringBuilder,
        StringDictionaryBuilder, UInt8Array, UInt16Array,
    },
    datatypes::{Field, Schema, UInt8Type, UInt16Type},
    error::ArrowError,
};
use otel_arrow_dfe_pdata::{
    encode::record::{
        array::{ArrayAppend, ArrayOptions, UInt16ArrayBuilder},
        attributes::AnyValuesRecordsBuilder,
    },
    schema::{FieldExt, consts},
};

/// The fixed syslog and CEF-header vocabulary; dynamic extensions use strings.
#[derive(Clone, Copy)]
pub(crate) enum KnownAttributeKey {
    Facility,
    Severity,
    HostName,
    Version,
    AppName,
    ProcessId,
    ProcessIdStr,
    MessageId,
    StructuredData,
    Message,
    Tag,
    Content,
    CefVersion,
    CefDeviceVendor,
    CefDeviceProduct,
    CefDeviceVersion,
    CefEventClassId,
    CefName,
    CefSeverity,
    InputFormat,
}

impl KnownAttributeKey {
    const COUNT: usize = Self::InputFormat as usize + 1;

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Facility => "syslog.facility",
            Self::Severity => "syslog.severity",
            Self::HostName => "syslog.host_name",
            Self::Version => "syslog.version",
            Self::AppName => "syslog.app_name",
            Self::ProcessId => "syslog.process_id",
            Self::ProcessIdStr => "syslog.process_id_str",
            Self::MessageId => "syslog.msg_id",
            Self::StructuredData => "syslog.structured_data",
            Self::Message => "syslog.message",
            Self::Tag => "syslog.tag",
            Self::Content => "syslog.content",
            Self::CefVersion => "cef.version",
            Self::CefDeviceVendor => "cef.device_vendor",
            Self::CefDeviceProduct => "cef.device_product",
            Self::CefDeviceVersion => "cef.device_version",
            Self::CefEventClassId => "cef.device_event_class_id",
            Self::CefName => "cef.name",
            Self::CefSeverity => "cef.severity",
            Self::InputFormat => "input.format",
        }
    }
}

enum KeyIndices {
    U8(Vec<u8>),
    U16(Vec<u16>),
}

impl KeyIndices {
    fn append(&mut self, index: u16) {
        match self {
            Self::U8(keys) => match u8::try_from(index) {
                Ok(index) => keys.push(index),
                Err(_) => {
                    let mut wide = Vec::with_capacity(keys.capacity());
                    wide.extend(keys.iter().copied().map(u16::from));
                    wide.push(index);
                    *self = Self::U16(wide);
                }
            },
            Self::U16(keys) => keys.push(index),
        }
    }

    fn append_to_native(&self, values: &StringArray, output: &mut StringBuilder) {
        match self {
            Self::U8(keys) => {
                for &key in keys {
                    output.append_value(values.value(usize::from(key)));
                }
            }
            Self::U16(keys) => {
                for &key in keys {
                    output.append_value(values.value(usize::from(key)));
                }
            }
        }
    }

    fn finish(self, values: ArrayRef) -> Result<ArrayRef, ArrowError> {
        match self {
            Self::U8(keys) => Ok(Arc::new(DictionaryArray::<UInt8Type>::try_new(
                UInt8Array::from(keys),
                values,
            )?)),
            Self::U16(keys) => Ok(Arc::new(DictionaryArray::<UInt16Type>::try_new(
                UInt16Array::from(keys),
                values,
            )?)),
        }
    }
}

enum KeyStorage {
    Dictionary {
        interner: StringDictionaryBuilder<UInt16Type>,
        indices: KeyIndices,
    },
    Native(StringBuilder),
}

struct AttributeKeys {
    storage: KeyStorage,
    // IDs never escape this batch, and the dictionary only grows. Native
    // fallback ignores these slots; a fresh builder always starts empty.
    known_ids: [Option<u16>; KnownAttributeKey::COUNT],
    #[cfg(test)]
    intern_calls: usize,
}

impl AttributeKeys {
    fn new() -> Self {
        Self {
            storage: KeyStorage::Dictionary {
                // The interner stores values only. Actual row indices are kept
                // separately, using u8 until the first index that needs u16.
                interner: StringDictionaryBuilder::with_capacity(0, 1024, 1024),
                indices: KeyIndices::U8(Vec::with_capacity(1024)),
            },
            known_ids: [None; KnownAttributeKey::COUNT],
            #[cfg(test)]
            intern_calls: 0,
        }
    }

    fn append_known(&mut self, key: KnownAttributeKey) {
        if let KeyStorage::Dictionary { indices, .. } = &mut self.storage
            && let Some(index) = self.known_ids[key as usize]
        {
            indices.append(index);
            return;
        }
        self.known_ids[key as usize] = self.append(key.as_str());
    }

    fn append(&mut self, key: &str) -> Option<u16> {
        match &mut self.storage {
            KeyStorage::Dictionary { interner, indices } => {
                // Arrow's append_n resolves and returns the dictionary ID even
                // when count is zero. This deliberately interns a value without
                // storing unused row indices. Equivalence tests protect this
                // dependency behavior, including empty strings and duplicates.
                #[cfg(test)]
                {
                    self.intern_calls += 1;
                }
                if let Ok(index) = interner.append_n(key, 0)
                    && index < u16::MAX
                {
                    indices.append(index);
                    return Some(index);
                }
                // Match the adaptive builder's max cardinality of 65,535.
                // Convert only previously appended rows, then append this key
                // once. Cached IDs must not be used after native fallback.
                let dictionary = interner.finish();
                // SAFETY: this private interner is a StringDictionaryBuilder,
                // so its values always have the Utf8 type required by StringArray.
                let values = StringArray::from(dictionary.values().to_data());
                let mut native = StringBuilder::new();
                indices.append_to_native(&values, &mut native);
                native.append_value(key);
                self.storage = KeyStorage::Native(native);
                None
            }
            KeyStorage::Native(native) => {
                native.append_value(key);
                None
            }
        }
    }

    fn finish(self) -> Result<ArrayRef, ArrowError> {
        match self.storage {
            KeyStorage::Dictionary {
                mut interner,
                indices,
            } => indices.finish(Arc::clone(interner.finish().values())),
            KeyStorage::Native(mut native) => Ok(Arc::new(native.finish())),
        }
    }
}

/// Syslog's attribute builder; values and parent IDs retain the common builders.
/// Consuming finish prevents dictionary IDs from leaking into a later batch.
pub(crate) struct SyslogAttributesBuilder {
    keys: AttributeKeys,
    parent_id: UInt16ArrayBuilder,
    pub(crate) any_values_builder: AnyValuesRecordsBuilder,
}

impl SyslogAttributesBuilder {
    pub(crate) fn new() -> Self {
        Self {
            keys: AttributeKeys::new(),
            parent_id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: false,
                ..Default::default()
            }),
            any_values_builder: AnyValuesRecordsBuilder::new(),
        }
    }

    pub(crate) fn append_known_key(&mut self, key: KnownAttributeKey) {
        self.keys.append_known(key);
    }

    pub(crate) fn append_key(&mut self, key: &str) {
        let _ = self.keys.append(key);
    }

    pub(crate) fn append_parent_id_n(&mut self, parent: &u16, count: usize) {
        self.parent_id.append_value_n(parent, count);
    }

    pub(crate) fn finish(mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::new();
        let mut columns = Vec::new();
        if let Some(parent_id) = self.parent_id.finish() {
            fields.push(
                Field::new(consts::PARENT_ID, parent_id.data_type().clone(), false)
                    .with_plain_encoding(),
            );
            columns.push(parent_id);
        }
        let keys = self.keys.finish()?;
        fields.push(Field::new(
            consts::ATTRIBUTE_KEY,
            keys.data_type().clone(),
            false,
        ));
        columns.push(keys);
        self.any_values_builder.finish(&mut columns, &mut fields)?;
        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::DataType;
    use otel_arrow_dfe_pdata::encode::record::{
        array::{ArrayAppendStr, StringArrayBuilder, dictionary::DictionaryOptions},
        attributes::StrKeysAttributesRecordBatchBuilder,
    };

    fn ordinary_keys() -> StringArrayBuilder {
        StringArrayBuilder::new(ArrayOptions {
            optional: false,
            dictionary_options: Some(DictionaryOptions::dict8()),
            ..Default::default()
        })
    }

    fn compare(keys: AttributeKeys, mut ordinary: StringArrayBuilder) -> ArrayRef {
        let keys = keys.finish().unwrap();
        let expected = ordinary.finish().unwrap();
        assert_eq!(keys.as_ref(), expected.as_ref());
        // Dictionary array equality compares decoded values. Check the actual
        // index buffers and dictionary value order as well as logical content.
        assert_eq!(keys.data_type(), expected.data_type());
        let actual_data = keys.to_data();
        let expected_data = expected.to_data();
        assert_eq!(actual_data.buffers(), expected_data.buffers());
        assert_eq!(actual_data.child_data(), expected_data.child_data());
        keys
    }

    /// Scenario: Dynamic CEF extensions collide with fixed keys before and after use, including a dictionary-width upgrade.
    /// Guarantees: Key identity, duplicates, empty names, and first-seen dictionary ordering match the general builder.
    #[test]
    fn known_and_dynamic_keys_preserve_dictionary_order() {
        for prefix in [0, 300] {
            let mut keys = AttributeKeys::new();
            let mut ordinary = ordinary_keys();
            keys.append_known(KnownAttributeKey::Facility);
            ordinary.append_str(KnownAttributeKey::Facility.as_str());
            for index in 0..prefix {
                let name = format!("extension-{index}");
                let _ = keys.append(&name);
                ordinary.append_str(&name);
            }
            for key in ["syslog.facility", "", "cef.name"] {
                let _ = keys.append(key);
                ordinary.append_str(key);
            }
            for _ in 0..3 {
                for key in [
                    KnownAttributeKey::CefName,
                    KnownAttributeKey::Facility,
                    KnownAttributeKey::Severity,
                    KnownAttributeKey::InputFormat,
                ] {
                    keys.append_known(key);
                    ordinary.append_str(key.as_str());
                }
                let _ = keys.append("syslog.severity");
                ordinary.append_str("syslog.severity");
            }
            let _ = compare(keys, ordinary);
        }
    }

    /// Scenario: Known-key appends repeat while dynamic appends reuse the same name.
    /// Guarantees: Fixed keys intern once per batch, dynamic names retain lookup, and the interner stores no row indices.
    #[test]
    fn known_keys_avoid_repeated_interning() {
        let mut keys = AttributeKeys::new();
        for _ in 0..100 {
            keys.append_known(KnownAttributeKey::Facility);
        }
        assert_eq!(keys.intern_calls, 1);
        let _ = keys.append(KnownAttributeKey::Facility.as_str());
        assert_eq!(keys.intern_calls, 2);
        let KeyStorage::Dictionary { interner, .. } = &mut keys.storage else {
            panic!("fixed vocabulary must remain dictionary encoded");
        };
        let dictionary = interner.finish_cloned();
        assert_eq!(dictionary.len(), 0);
        assert_eq!(dictionary.values().len(), 1);
    }

    /// Scenario: Attribute dictionaries are empty, small, or cross the u8 boundary up to maximum dictionary cardinality.
    /// Guarantees: The output uses exactly the general builder's u8/u16 width and dictionary contents.
    #[test]
    fn key_width_transitions_match_general_builder() {
        for count in [0, 1, 256, 257, 65535] {
            let mut keys = AttributeKeys::new();
            let mut ordinary = ordinary_keys();
            for index in 0..count {
                let key = format!("extension-{index}");
                let _ = keys.append(&key);
                ordinary.append_str(&key);
            }
            let result = compare(keys, ordinary);
            let width = if count <= 256 {
                DataType::UInt8
            } else {
                DataType::UInt16
            };
            assert_eq!(
                result.data_type(),
                &DataType::Dictionary(Box::new(width), Box::new(DataType::Utf8))
            );
        }
    }

    /// Scenario: Dynamic extensions exhaust the 65,535-value dictionary after a fixed key was cached.
    /// Guarantees: Native fallback neither duplicates nor drops the overflow row and cached keys remain correct afterward.
    #[test]
    fn key_native_fallback_preserves_cached_keys() {
        let mut keys = AttributeKeys::new();
        let mut ordinary = ordinary_keys();
        keys.append_known(KnownAttributeKey::Facility);
        ordinary.append_str(KnownAttributeKey::Facility.as_str());
        for index in 0..65535 {
            let key = format!("extension-{index}");
            let _ = keys.append(&key);
            ordinary.append_str(&key);
        }
        for key in [KnownAttributeKey::Facility, KnownAttributeKey::InputFormat] {
            keys.append_known(key);
            ordinary.append_str(key.as_str());
        }
        let result = compare(keys, ordinary);
        assert_eq!(result.data_type(), &DataType::Utf8);
        assert_eq!(result.len(), 65538);
    }

    /// Scenario: Two batches encounter the same fixed names in opposite order.
    /// Guarantees: Known-key IDs are local to each batch and never create stale or orphan dictionary entries.
    #[test]
    fn known_key_ids_are_batch_local() {
        for order in [
            [KnownAttributeKey::Severity, KnownAttributeKey::Facility],
            [KnownAttributeKey::Facility, KnownAttributeKey::Severity],
        ] {
            let mut keys = AttributeKeys::new();
            let mut ordinary = ordinary_keys();
            for key in order {
                keys.append_known(key);
                ordinary.append_str(key.as_str());
            }
            let _ = compare(keys, ordinary);
        }
    }

    /// Scenario: Mixed fixed and dynamic attributes use repeated parents and string, integer, and empty values.
    /// Guarantees: Complete attribute records retain exact fields, encoding metadata, row order, and values.
    #[test]
    fn syslog_attributes_match_general_builder() {
        for rows in [0, 1, 1000] {
            let mut actual = SyslogAttributesBuilder::new();
            let mut expected = StrKeysAttributesRecordBatchBuilder::<u16>::new();
            for parent in 0..rows {
                actual.append_known_key(KnownAttributeKey::Facility);
                expected.append_key(KnownAttributeKey::Facility.as_str());
                actual.any_values_builder.append_int(i64::from(parent));
                expected.any_values_builder.append_int(i64::from(parent));
                actual.append_key("dynamic");
                expected.append_key("dynamic");
                actual.any_values_builder.append_str(b"value");
                expected.any_values_builder.append_str(b"value");
                actual.append_known_key(KnownAttributeKey::InputFormat);
                expected.append_key(KnownAttributeKey::InputFormat.as_str());
                actual.any_values_builder.append_empty();
                expected.any_values_builder.append_empty();
                actual.append_parent_id_n(&parent, 3);
                expected.append_parent_id_n(&parent, 3);
            }
            assert_eq!(actual.finish().unwrap(), expected.finish().unwrap());
        }
    }
}
