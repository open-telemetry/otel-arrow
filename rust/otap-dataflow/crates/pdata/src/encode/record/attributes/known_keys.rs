// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opt-in key-column construction for repeated attribute vocabularies.
//!
//! Intern a known name on first use, then append its batch-local dictionary ID
//! directly. Dynamic names share Arrow's interner. Separate row indices retain
//! u8/u16 widening and the ordinary attribute builder's plain-string fallback.
//! Vocabulary slots are reusable; internal dictionary IDs never escape a batch.

use super::AttributeKeyColumnBuilder;
use arrow::{
    array::{
        ArrayRef, DictionaryArray, StringArray, StringBuilder, StringDictionaryBuilder, UInt8Array,
        UInt16Array,
    },
    datatypes::{UInt8Type, UInt16Type},
    error::ArrowError,
};
use std::sync::Arc;

/// A position in a configured vocabulary, not an Arrow dictionary ID.
///
/// Obtain valid positions from [`Self::slots`]. Handles may be reused across
/// batches: the receiving builder resolves the name at that position in its
/// own vocabulary and owns the batch-local dictionary cache.
#[derive(Clone, Copy)]
pub struct KnownKey<const N: usize> {
    index: usize,
}

impl<const N: usize> KnownKey<N> {
    /// Returns all valid positions for a vocabulary of `N` names.
    #[must_use]
    pub const fn slots() -> [Self; N] {
        let mut result = [Self { index: 0 }; N];
        let mut index = 0;
        while index < N {
            result[index] = Self { index };
            index += 1;
        }
        result
    }
}

/// Per-attribute row references into the shared key dictionary.
/// Starts with u8 indices and widens to u16 when necessary without renumbering
/// dictionary values, so already cached known-key IDs remain valid.
enum KeyIndices {
    U8(Vec<u8>),
    U16(Vec<u16>),
}

impl KeyIndices {
    // Receiver crates instantiate the cached-key path; keep this small append
    // available for inlining across the crate boundary.
    #[inline]
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

/// Physical key-column representation for the current batch.
/// Uses dictionary encoding until the ordinary builder's cardinality limit,
/// then expands existing rows into plain UTF-8 strings for the rest of the batch.
enum KeyStorage {
    Uninitialized,
    Dictionary {
        // Stores unique strings only; append_n(key, 0) does not add row indices.
        interner: StringDictionaryBuilder<UInt16Type>,
        // Actual row indices, independently sized as u8 or u16.
        indices: KeyIndices,
    },
    Native(StringBuilder),
}

/// Opt-in string key-column builder with a fixed vocabulary and dynamic keys.
///
/// Known keys bypass repeated hashing after their first occurrence in a batch.
/// All keys share one interner, preserving first-seen order and duplicate names.
/// Cached dictionary IDs are private, used only while dictionary encoded, and
/// cleared by [`Self::finish`]. The ordinary string builder is unaffected.
pub struct KnownKeyArrayBuilder<const N: usize> {
    names: &'static [&'static str; N],
    storage: KeyStorage,
    // IDs never escape this batch, and the dictionary only grows. Native
    // fallback ignores these slots; a fresh builder always starts empty.
    known_ids: [Option<u16>; N],
    #[cfg(test)]
    intern_calls: usize,
}

impl<const N: usize> KnownKeyArrayBuilder<N> {
    /// Creates an opt-in builder for a fixed vocabulary, without pre-interning it.
    #[must_use]
    pub fn new(names: &'static [&'static str; N]) -> Self {
        Self {
            names,
            storage: KeyStorage::Uninitialized,
            known_ids: [None; N],
            #[cfg(test)]
            intern_calls: 0,
        }
    }

    /// Appends a vocabulary slot, caching its dictionary ID for this batch only.
    pub fn append_known_key(&mut self, key: KnownKey<N>) {
        if let KeyStorage::Dictionary { indices, .. } = &mut self.storage
            && let Some(index) = self.known_ids[key.index]
        {
            indices.append(index);
            return;
        }
        self.known_ids[key.index] = self.append(self.names[key.index]);
    }

    /// Appends a dynamic key through the same interner as known keys.
    pub fn append_key(&mut self, key: &str) {
        let _ = self.append(key);
    }

    fn append(&mut self, key: &str) -> Option<u16> {
        match &mut self.storage {
            KeyStorage::Uninitialized => {
                self.storage = KeyStorage::Dictionary {
                    // Only values are interned; compact row indices live separately.
                    interner: StringDictionaryBuilder::with_capacity(0, 1024, 1024),
                    indices: KeyIndices::U8(Vec::with_capacity(1024)),
                };
                self.append(key)
            }
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

    /// Finishes one batch and clears cached IDs and encoding state.
    /// The next append starts a fresh dictionary lazily; resetting the cache
    /// does not allocate a replacement interner.
    pub fn finish(&mut self) -> Result<ArrayRef, ArrowError> {
        self.known_ids.fill(None);
        match std::mem::replace(&mut self.storage, KeyStorage::Uninitialized) {
            KeyStorage::Uninitialized => {
                KeyIndices::U8(Vec::new()).finish(Arc::new(StringArray::from(Vec::<&str>::new())))
            }
            KeyStorage::Dictionary {
                mut interner,
                indices,
            } => indices.finish(Arc::clone(interner.finish().values())),
            KeyStorage::Native(mut native) => Ok(Arc::new(native.finish())),
        }
    }
}

impl<const N: usize> AttributeKeyColumnBuilder for KnownKeyArrayBuilder<N> {
    fn append_key(&mut self, key: &str) {
        self.append_key(key);
    }

    fn finish_keys(&mut self) -> Result<Option<ArrayRef>, ArrowError> {
        self.finish().map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::DataType;

    const NAMES: [&str; 4] = ["known.first", "known.second", "known.third", "known.fourth"];
    const SLOTS: [KnownKey<4>; 4] = KnownKey::slots();
    const FIRST: KnownKey<4> = SLOTS[0];
    const SECOND: KnownKey<4> = SLOTS[1];
    const THIRD: KnownKey<4> = SLOTS[2];
    const FOURTH: KnownKey<4> = SLOTS[3];
    use crate::encode::record::array::{
        ArrayAppendStr, ArrayOptions, StringArrayBuilder, dictionary::DictionaryOptions,
    };

    fn ordinary_keys() -> StringArrayBuilder {
        StringArrayBuilder::new(ArrayOptions {
            optional: false,
            dictionary_options: Some(DictionaryOptions::dict8()),
            ..Default::default()
        })
    }

    fn compare<const N: usize>(
        keys: &mut KnownKeyArrayBuilder<N>,
        mut ordinary: StringArrayBuilder,
    ) -> ArrayRef {
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

    /// Scenario: Dynamic attributes collide with fixed keys before and after use, including a dictionary-width upgrade.
    /// Guarantees: Key identity, duplicates, empty names, and first-seen dictionary ordering match the general builder.
    #[test]
    fn known_and_dynamic_keys_preserve_dictionary_order() {
        for prefix in [0, 300] {
            let mut keys = KnownKeyArrayBuilder::new(&NAMES);
            let mut ordinary = ordinary_keys();
            keys.append_known_key(FIRST);
            ordinary.append_str(NAMES[FIRST.index]);
            for index in 0..prefix {
                let name = format!("extension-{index}");
                let _ = keys.append(&name);
                ordinary.append_str(&name);
            }
            for key in ["known.first", "", "known.third"] {
                let _ = keys.append(key);
                ordinary.append_str(key);
            }
            for _ in 0..3 {
                for key in [THIRD, FIRST, SECOND, FOURTH] {
                    keys.append_known_key(key);
                    ordinary.append_str(NAMES[key.index]);
                }
                let _ = keys.append("known.second");
                ordinary.append_str("known.second");
            }
            let _ = compare(&mut keys, ordinary);
        }
    }

    /// Scenario: Known-key appends repeat while dynamic appends reuse the same name.
    /// Guarantees: Fixed keys intern once per batch, dynamic names retain lookup, and the interner stores no row indices.
    #[test]
    fn known_keys_avoid_repeated_interning() {
        let mut keys = KnownKeyArrayBuilder::new(&NAMES);
        for _ in 0..100 {
            keys.append_known_key(FIRST);
        }
        assert_eq!(keys.intern_calls, 1);
        let _ = keys.append(NAMES[FIRST.index]);
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
            let mut keys = KnownKeyArrayBuilder::new(&NAMES);
            let mut ordinary = ordinary_keys();
            for index in 0..count {
                let key = format!("extension-{index}");
                let _ = keys.append(&key);
                ordinary.append_str(&key);
            }
            let result = compare(&mut keys, ordinary);
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
        let mut keys = KnownKeyArrayBuilder::new(&NAMES);
        let mut ordinary = ordinary_keys();
        keys.append_known_key(FIRST);
        ordinary.append_str(NAMES[FIRST.index]);
        for index in 0..65535 {
            let key = format!("extension-{index}");
            let _ = keys.append(&key);
            ordinary.append_str(&key);
        }
        for key in [FIRST, FOURTH] {
            keys.append_known_key(key);
            ordinary.append_str(NAMES[key.index]);
        }
        let result = compare(&mut keys, ordinary);
        assert_eq!(result.data_type(), &DataType::Utf8);
        assert_eq!(result.len(), 65538);
        // Reusing the builder after fallback must start a fresh dictionary.
        keys.append_known_key(SECOND);
        keys.append_known_key(FIRST);
        let mut ordinary = ordinary_keys();
        ordinary.append_str(NAMES[SECOND.index]);
        ordinary.append_str(NAMES[FIRST.index]);
        let result = compare(&mut keys, ordinary);
        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        );
    }

    /// Scenario: A reused builder encounters fixed names in opposite order after finish.
    /// Guarantees: Cached IDs reset between batches and preserve physical first-seen ordering.
    #[test]
    fn known_key_ids_are_batch_local() {
        let mut keys = KnownKeyArrayBuilder::new(&NAMES);
        for order in [[SECOND, FIRST], [FIRST, SECOND]] {
            let mut ordinary = ordinary_keys();
            for key in order {
                keys.append_known_key(key);
                ordinary.append_str(NAMES[key.index]);
            }
            let _ = compare(&mut keys, ordinary);
        }
    }

    /// Scenario: A vocabulary is empty or contains two slots naming the same key.
    /// Guarantees: Dynamic-only batches and vocabulary aliases retain ordinary dictionary contents and row indices.
    #[test]
    fn empty_and_aliased_vocabularies_match_general_builder() {
        let mut empty = KnownKeyArrayBuilder::new(&[]);
        let mut ordinary = ordinary_keys();
        for key in ["", "dynamic", "dynamic"] {
            empty.append_key(key);
            ordinary.append_str(key);
        }
        let _ = compare(&mut empty, ordinary);
        let mut aliases = KnownKeyArrayBuilder::new(&["same", "same"]);
        let slots = KnownKey::<2>::slots();
        let mut ordinary = ordinary_keys();
        for slot in [slots[1], slots[0], slots[1]] {
            aliases.append_known_key(slot);
            ordinary.append_str("same");
        }
        let _ = compare(&mut aliases, ordinary);
    }
}
