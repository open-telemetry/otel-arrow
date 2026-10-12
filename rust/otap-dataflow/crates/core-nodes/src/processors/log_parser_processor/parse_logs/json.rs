// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded JSON storage and duplicate-aware extraction.

use hashbrown::HashTable;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use smallvec::SmallVec;
use std::{collections::hash_map::RandomState, fmt, hash::BuildHasher, ops::Range};

#[derive(Debug)]
enum Value {
    Null,
    String(Range<usize>),
    Object,
    Array,
    Other,
}

#[derive(Debug)]
struct Node {
    parent: usize,
    key: Range<usize>,
    value: Value,
}

struct KeyIndexEntry {
    hash: u64,
    node_index: usize,
}

pub(super) struct Document {
    nodes: Vec<Node>,
    text: String,
    keys: HashTable<KeyIndexEntry>,
    inline_keys: SmallVec<[usize; 16]>,
    key_hasher: RandomState,
    max_entries: usize,
    max_depth: usize,
    limit_hit: bool,
}

impl Document {
    fn key_index_bound(entries: usize) -> Option<usize> {
        const MAX_BUCKETS_PER_ENTRY: usize = 4;
        const CONTROL_ALIGNMENT_ALLOWANCE: usize = 64;
        entries
            .checked_mul(MAX_BUCKETS_PER_ENTRY)?
            .checked_mul(size_of::<KeyIndexEntry>() + 1)?
            .checked_add(CONTROL_ALIGNMENT_ALLOWANCE)
    }

    pub(super) fn scratch_bound(input_len: usize, max_entries: usize) -> Option<usize> {
        let entries = max_entries.min(input_len);
        input_len
            .checked_mul(3)?
            .checked_add(entries.checked_add(1)?.checked_mul(size_of::<Node>())?)?
            .checked_add(Self::key_index_bound(entries)?)?
            .checked_add(512)
    }

    pub(super) fn parse(input: &str, max_entries: usize, max_depth: usize) -> Result<Self, bool> {
        let capacity = max_entries.min(input.len()).saturating_add(1);
        let mut document = Self {
            nodes: Vec::with_capacity(capacity),
            text: String::with_capacity(input.len()),
            keys: HashTable::new(),
            inline_keys: SmallVec::new(),
            key_hasher: RandomState::new(),
            max_entries: capacity - 1,
            max_depth,
            limit_hit: false,
        };
        let mut deserializer = serde_json::Deserializer::from_str(input);
        let result = Seed {
            document: &mut document,
            parent: usize::MAX,
            key: 0..0,
            depth: 1,
        }
        .deserialize(&mut deserializer)
        .and_then(|_| deserializer.end());
        if result.is_err() {
            return Err(document.limit_hit);
        }
        if !matches!(document.nodes[0].value, Value::Object) {
            return Err(false);
        }
        Ok(document)
    }

    fn store(&mut self, value: &str) -> Range<usize> {
        let start = self.text.len();
        self.text.push_str(value);
        start..self.text.len()
    }

    fn contains_key(&self, hash: u64, parent: usize, key: &str) -> bool {
        self.keys
            .find(hash, |entry| {
                let node = &self.nodes[entry.node_index];
                node.parent == parent && &self.text[node.key.clone()] == key
            })
            .is_some()
    }

    fn has_key(&self, parent: usize, key: &str) -> bool {
        if self.keys.capacity() == 0 {
            return self.inline_keys.iter().any(|node_index| {
                let node = &self.nodes[*node_index];
                node.parent == parent && &self.text[node.key.clone()] == key
            });
        }
        self.contains_key(self.key_hasher.hash_one((parent, key)), parent, key)
    }

    fn index_key(&mut self, node_index: usize) {
        if self.keys.capacity() == 0 && self.inline_keys.len() < self.inline_keys.inline_size() {
            self.inline_keys.push(node_index);
            return;
        }
        let Self {
            nodes,
            text,
            keys,
            inline_keys,
            key_hasher,
            ..
        } = self;
        let make_entry = |node_index: usize| {
            let node = &nodes[node_index];
            KeyIndexEntry {
                hash: key_hasher.hash_one((node.parent, &text[node.key.clone()])),
                node_index,
            }
        };
        if keys.capacity() == 0 {
            *keys = HashTable::with_capacity(inline_keys.len() + 1);
            for inline_index in inline_keys.drain(..) {
                let entry = make_entry(inline_index);
                let _ = keys.insert_unique(entry.hash, entry, |entry| entry.hash);
            }
        }
        let entry = make_entry(node_index);
        let _ = keys.insert_unique(entry.hash, entry, |entry| entry.hash);
    }

    pub(super) fn lookup(&self, tokens: &[String]) -> Result<Option<&str>, ()> {
        let mut current = 0;
        for token in tokens {
            let mut children = self
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| node.parent == current);
            let found = match self.nodes[current].value {
                Value::Object => children.find(|(_, node)| &self.text[node.key.clone()] == token),
                Value::Array => {
                    if token.is_empty()
                        || !token.bytes().all(|byte| byte.is_ascii_digit())
                        || (token.starts_with('0') && token.len() > 1)
                    {
                        return Ok(None);
                    }
                    let Some(index) = token.parse::<usize>().ok() else {
                        return Ok(None);
                    };
                    children.nth(index)
                }
                _ => return Ok(None),
            };
            let Some((index, _)) = found else {
                return Ok(None);
            };
            current = index;
        }
        match &self.nodes[current].value {
            Value::Null => Ok(None),
            Value::String(range) => Ok(Some(&self.text[range.clone()])),
            _ => Err(()),
        }
    }
}

struct Seed<'document> {
    document: &'document mut Document,
    parent: usize,
    key: Range<usize>,
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = ();

    fn deserialize<Deserializer: de::Deserializer<'de>>(
        self,
        deserializer: Deserializer,
    ) -> Result<(), Deserializer::Error> {
        if self.document.nodes.len() > self.document.max_entries {
            self.document.limit_hit = true;
            return Err(de::Error::custom("entry limit"));
        }
        let index = self.document.nodes.len();
        self.document.nodes.push(Node {
            parent: self.parent,
            key: self.key,
            value: Value::Null,
        });
        deserializer.deserialize_any(ValueVisitor {
            document: self.document,
            index,
            depth: self.depth,
        })
    }
}

struct ValueVisitor<'document> {
    document: &'document mut Document,
    index: usize,
    depth: usize,
}

impl ValueVisitor<'_> {
    fn check_depth<Error: de::Error>(&mut self) -> Result<(), Error> {
        if self.depth > self.document.max_depth {
            self.document.limit_hit = true;
            return Err(de::Error::custom("depth limit"));
        }
        Ok(())
    }
}

impl<'de> Visitor<'de> for ValueVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON value")
    }

    fn visit_unit<Error: de::Error>(self) -> Result<(), Error> {
        Ok(())
    }

    fn visit_bool<Error: de::Error>(self, _: bool) -> Result<(), Error> {
        self.document.nodes[self.index].value = Value::Other;
        Ok(())
    }

    fn visit_i64<Error: de::Error>(self, _: i64) -> Result<(), Error> {
        self.visit_bool(false)
    }
    fn visit_u64<Error: de::Error>(self, _: u64) -> Result<(), Error> {
        self.visit_bool(false)
    }
    fn visit_f64<Error: de::Error>(self, _: f64) -> Result<(), Error> {
        self.visit_bool(false)
    }

    fn visit_str<Error: de::Error>(self, value: &str) -> Result<(), Error> {
        let range = self.document.store(value);
        self.document.nodes[self.index].value = Value::String(range);
        Ok(())
    }

    fn visit_map<Access: MapAccess<'de>>(
        mut self,
        mut access: Access,
    ) -> Result<(), Access::Error> {
        self.check_depth()?;
        self.document.nodes[self.index].value = Value::Object;
        while let Some(key) = access.next_key_seed(KeySeed(self.document))? {
            if self
                .document
                .has_key(self.index, &self.document.text[key.clone()])
            {
                return Err(de::Error::custom("duplicate key"));
            }
            let node_index = self.document.nodes.len();
            access.next_value_seed(Seed {
                document: self.document,
                parent: self.index,
                key,
                depth: self.depth + 1,
            })?;
            self.document.index_key(node_index);
        }
        Ok(())
    }

    fn visit_seq<Access: SeqAccess<'de>>(
        mut self,
        mut access: Access,
    ) -> Result<(), Access::Error> {
        self.check_depth()?;
        self.document.nodes[self.index].value = Value::Array;
        while access
            .next_element_seed(Seed {
                document: self.document,
                parent: self.index,
                key: 0..0,
                depth: self.depth + 1,
            })?
            .is_some()
        {}
        Ok(())
    }
}

struct KeySeed<'document>(&'document mut Document);

impl<'de> DeserializeSeed<'de> for KeySeed<'_> {
    type Value = Range<usize>;

    fn deserialize<Deserializer: de::Deserializer<'de>>(
        self,
        deserializer: Deserializer,
    ) -> Result<Self::Value, Deserializer::Error> {
        deserializer.deserialize_str(self)
    }
}

impl Visitor<'_> for KeySeed<'_> {
    type Value = Range<usize>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("object key")
    }

    fn visit_str<Error: de::Error>(self, value: &str) -> Result<Self::Value, Error> {
        Ok(self.0.store(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Objects have few keys but short or long values under a large entry limit.
    /// Guarantees: Key-index allocation follows key count, not record bytes or the configured entry limit.
    #[test]
    fn key_index_allocation_tracks_key_count() {
        for fields in [8, 64, 1, 0, 16, 17] {
            let expected = HashTable::<KeyIndexEntry>::with_capacity(fields).allocation_size();
            for repeats in [1, 1024] {
                let object: serde_json::Map<String, serde_json::Value> = (0..fields)
                    .map(|index| {
                        (
                            format!("key{index:04}"),
                            serde_json::json!("value".repeat(repeats)),
                        )
                    })
                    .collect();
                let input = serde_json::to_string(&object).unwrap();
                let document = Document::parse(&input, 4096, 1).unwrap();
                assert_eq!(document.keys.len() + document.inline_keys.len(), fields);
                assert!(!document.inline_keys.spilled());
                if fields <= document.inline_keys.inline_size() {
                    assert_eq!(document.keys.allocation_size(), 0);
                }
                assert!(
                    document.keys.allocation_size() <= expected,
                    "{fields} keys in {} bytes allocated {} index bytes; expected at most {expected}",
                    input.len(),
                    document.keys.allocation_size(),
                );
            }
        }
    }

    /// Scenario: Repeated decoded names occur in sibling objects, array elements and parent-child scopes.
    /// Guarantees: Only duplicates in the same object are rejected, including escaped-equivalent names.
    #[test]
    fn duplicate_keys_are_scoped_to_their_object() {
        let document = Document::parse(
            r#"{"x":"outer","a":{"x":"first"},"b":[{"x":"second"},{"\u0078":"third"}]}"#,
            10,
            4,
        )
        .unwrap();
        assert_eq!(document.lookup(&["x".into()]), Ok(Some("outer")));
        assert_eq!(
            document.lookup(&["a".into(), "x".into()]),
            Ok(Some("first"))
        );
        assert_eq!(
            document.lookup(&["b".into(), "1".into(), "x".into()]),
            Ok(Some("third"))
        );
        for input in [
            r#"{"a":{"x":1,"\u0078":2}}"#,
            r#"{"a":{"x":1},"a":{"x":2}}"#,
            r#"{"a":[{"x":1,"x":2}]}"#,
            r#"{"":1,"":2}"#,
        ] {
            assert!(
                matches!(Document::parse(input, 10, 4), Err(false)),
                "{input}"
            );
        }
    }

    /// Scenario: An object fills or promotes the inline index before an escaped or parent-level duplicate arrives.
    /// Guarantees: Promotion preserves duplicate rejection and allows identical names in different objects.
    #[test]
    fn duplicate_keys_survive_index_promotion() {
        for fields in [16, 17, 32] {
            let object: serde_json::Map<String, serde_json::Value> = (0..fields)
                .map(|index| (format!("key{index:04}"), serde_json::json!("value")))
                .collect();
            let input = serde_json::to_string(&object).unwrap();
            let duplicate = format!(
                r#"{},"\u006bey0000":"duplicate"}}"#,
                input.strip_suffix('}').unwrap(),
            );
            assert!(matches!(Document::parse(&duplicate, 4096, 4), Err(false)));
            let parent_duplicate = format!(r#"{{"outer":{input},"outer":{{}}}}"#);
            assert!(matches!(
                Document::parse(&parent_duplicate, 4096, 4),
                Err(false)
            ));
            let siblings = format!(r#"{{"key0000":{input},"other":{input}}}"#);
            let document = Document::parse(&siblings, 4096, 4).unwrap();
            assert_eq!(
                document.lookup(&["key0000".into(), "key0000".into()]),
                Ok(Some("value")),
            );
        }
    }

    /// Scenario: Different names and parent objects are deliberately inserted with the same hash.
    /// Guarantees: A hash collision is never treated as a duplicate without parent and decoded-text equality.
    #[test]
    fn key_index_resolves_hash_collisions() {
        let mut document = Document::parse(r#"{"a":{"x":1},"b":2}"#, 4, 2).unwrap();
        document.keys.clear();
        for node_index in 1..document.nodes.len() {
            let entry = KeyIndexEntry {
                hash: 0,
                node_index,
            };
            let _ = document
                .keys
                .insert_unique(entry.hash, entry, |entry| entry.hash);
        }
        assert!(document.contains_key(0, 0, "a"));
        assert!(document.contains_key(0, 0, "b"));
        assert!(document.contains_key(0, 1, "x"));
        assert!(!document.contains_key(0, 0, "x"));
        assert!(!document.contains_key(0, 1, "b"));
        assert!(!document.contains_key(0, 0, "missing"));
    }

    /// Scenario: A flat JSON object has exactly 4096 unique keys, then one additional member.
    /// Guarantees: The growing index honors the exact entry boundary without exceeding its reservation.
    #[test]
    fn wide_object_entry_boundary() {
        let mut object: serde_json::Map<String, serde_json::Value> = (0..4096)
            .map(|index| (format!("key{index:04}"), serde_json::json!("value")))
            .collect();
        let input = serde_json::to_string(&object).unwrap();
        let document = Document::parse(&input, 4096, 1).unwrap();
        assert_eq!(document.nodes.len(), 4097);
        assert_eq!(document.keys.len(), 4096);
        assert_eq!(document.lookup(&["key4095".into()]), Ok(Some("value")));
        assert!(document.keys.allocation_size() <= Document::key_index_bound(4096).unwrap());
        assert!(matches!(Document::parse(&input, 4095, 1), Err(true)));
        let _ = object.insert("key4096".into(), serde_json::json!("value"));
        let over_limit = serde_json::to_string(&object).unwrap();
        assert!(matches!(Document::parse(&over_limit, 4096, 1), Err(true)));
    }

    /// Scenario: The index capacity crosses small-table and power-of-two allocation boundaries.
    /// Guarantees: Hashbrown's requested allocation fits the preflight reservation and overflow is rejected.
    #[test]
    fn key_index_allocation_bound() {
        for entries in [0, 1, 2, 3, 4, 7, 8, 14, 15, 16, 63, 64, 511, 512, 4096] {
            let index = HashTable::<KeyIndexEntry>::with_capacity(entries);
            assert!(index.allocation_size() <= Document::key_index_bound(entries).unwrap());
        }
        assert!(Document::key_index_bound(usize::MAX).is_none());
        assert!(Document::scratch_bound(usize::MAX, 4096).is_none());
    }

    /// Scenario: The promoted key index resizes while previously hashed key ranges are inaccessible.
    /// Guarantees: Growth reuses stored hashes without rereading old strings and preserves lookup after restoration.
    #[test]
    fn key_index_growth_does_not_read_existing_key_text() {
        let object: serde_json::Map<String, serde_json::Value> = (0..17)
            .map(|index| (format!("key{index:04}"), serde_json::json!("value")))
            .collect();
        let input = serde_json::to_string(&object).unwrap();
        let mut document = Document::parse(&input, 4096, 1).unwrap();
        let initial_capacity = document.keys.capacity();
        assert!(initial_capacity > 0);
        let original_keys: Vec<_> = document.nodes.iter().map(|node| node.key.clone()).collect();
        for node in document.nodes.iter_mut().skip(1) {
            node.key = usize::MAX..usize::MAX;
        }
        let extra_entries = initial_capacity + 1 - document.keys.len();
        for index in 0..extra_entries {
            let key = document.store(&format!("extra{index}"));
            let node_index = document.nodes.len();
            document.nodes.push(Node {
                parent: 0,
                key,
                value: Value::Null,
            });
            document.index_key(node_index);
        }
        assert!(document.keys.capacity() > initial_capacity);
        assert_eq!(document.keys.len(), initial_capacity + 1);
        for (node, key) in document.nodes.iter_mut().zip(original_keys) {
            node.key = key;
        }
        for index in 0..17 {
            assert!(document.has_key(0, &format!("key{index:04}")));
        }
        for index in 0..extra_entries {
            assert!(document.has_key(0, &format!("extra{index}")));
        }
    }

    /// Scenario: The key index grows through successive table capacities up to 4096 entries.
    /// Guarantees: Both tables' combined allocation during each resize fits the preflight reservation.
    #[test]
    fn key_index_growth_fits_reservation() {
        let mut index = HashTable::new();
        let hasher = RandomState::new();
        for node_index in 0..4096_usize {
            let before = index.allocation_size();
            let hash = hasher.hash_one(node_index);
            let _ =
                index.insert_unique(hash, KeyIndexEntry { hash, node_index }, |entry| entry.hash);
            let after = index.allocation_size();
            let peak = if before == after {
                after
            } else {
                before + after
            };
            assert!(
                peak <= Document::key_index_bound(node_index + 1).unwrap(),
                "{} entries require {peak} bytes during growth",
                node_index + 1,
            );
            assert_eq!(
                index
                    .find(hash, |entry| entry.node_index == node_index)
                    .map(|entry| entry.node_index),
                Some(node_index)
            );
        }
    }

    /// Scenario: JSON objects include escaped names, nested objects, array entries and nulls.
    /// Guarantees: Configured pointer tokens resolve strings without coercing other values.
    #[test]
    fn pointer_selection() {
        let document = Document::parse(
            r#"{"a/b":"yes","nested":[{"value":"ok"}],"missing":null,"number":42}"#,
            20,
            4,
        )
        .unwrap();
        assert_eq!(document.lookup(&["a/b".into()]), Ok(Some("yes")));
        assert_eq!(
            document.lookup(&["nested".into(), "0".into(), "value".into()]),
            Ok(Some("ok"))
        );
        assert_eq!(document.lookup(&["missing".into()]), Ok(None));
        assert_eq!(document.lookup(&["number".into()]), Err(()));
        assert_eq!(
            document.lookup(&["nested".into(), "+0".into(), "value".into()]),
            Ok(None)
        );
        assert_eq!(
            document.lookup(&["nested".into(), "00".into(), "value".into()]),
            Ok(None)
        );
    }

    /// Scenario: JSON contains duplicate keys, extra documents or too many nested entries.
    /// Guarantees: Extraction rejects invalid shapes and enforces exact entry/depth boundaries.
    #[test]
    fn invalid_shapes_and_limits() {
        for input in [
            r#"{"a":1,"a":2}"#,
            r#"{"a":1,"\u0061":2}"#,
            "{} {}",
            "[]",
            "null",
        ] {
            assert!(Document::parse(input, 10, 4).is_err(), "{input}");
        }
        assert!(Document::parse(r#"{"a":{"b":"ok"}}"#, 2, 2).is_ok());
        assert!(matches!(
            Document::parse(r#"{"a":{"b":"ok"}}"#, 1, 2),
            Err(true)
        ));
        assert!(matches!(
            Document::parse(r#"{"a":{"b":"ok"}}"#, 2, 1),
            Err(true)
        ));
    }
}
