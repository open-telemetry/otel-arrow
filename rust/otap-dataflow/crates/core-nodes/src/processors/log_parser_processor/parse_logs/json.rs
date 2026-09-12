//! Bounded JSON storage and duplicate-aware extraction.

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::{fmt, ops::Range};

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

pub(super) struct Document {
    nodes: Vec<Node>,
    text: String,
    max_entries: usize,
    max_depth: usize,
    limit_hit: bool,
}

impl Document {
    pub(super) fn scratch_bound(input_len: usize, max_entries: usize) -> Option<usize> {
        input_len
            .checked_mul(3)?
            .checked_add(
                max_entries
                    .min(input_len)
                    .checked_add(1)?
                    .checked_mul(size_of::<Node>())?,
            )?
            .checked_add(512)
    }

    pub(super) fn parse(input: &str, max_entries: usize, max_depth: usize) -> Result<Self, bool> {
        let capacity = max_entries.min(input.len()).saturating_add(1);
        let mut document = Self {
            nodes: Vec::with_capacity(capacity),
            text: String::with_capacity(input.len()),
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
            if self.document.nodes.iter().any(|node| {
                node.parent == self.index
                    && self.document.text[node.key.clone()] == self.document.text[key.clone()]
            }) {
                return Err(de::Error::custom("duplicate key"));
            }
            access.next_value_seed(Seed {
                document: self.document,
                parent: self.index,
                key,
                depth: self.depth + 1,
            })?;
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
