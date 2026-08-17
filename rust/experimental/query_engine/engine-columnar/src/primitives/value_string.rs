// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::hash::{Hash, Hasher};
use std::rc::Rc;

use arrow::buffer::Buffer;
use data_engine_expressions::*;

use crate::*;

#[derive(Debug, Clone)]
pub enum StringValueOrRef<'a> {
    Empty,
    Ref(&'a str),
    Buffer(Utf8Buffer),
    Owned(Rc<String>),
    Slice(StringValueOrRefSlice<'a>),
}

impl StringValueOrRef<'_> {
    pub fn new_owned(value: String) -> StringValueOrRef<'static> {
        StringValueOrRef::Owned(value.into())
    }

    pub fn new_utf8(buffer: Buffer) -> StringValueOrRef<'static> {
        assert!(std::str::from_utf8(&buffer).is_ok(), "invalid UTF-8");
        StringValueOrRef::Buffer(Utf8Buffer { buffer })
    }

    /// # Safety
    ///
    /// The bytes passed in must be valid UTF-8.
    pub unsafe fn new_utf8_unvalidated(buffer: Buffer) -> StringValueOrRef<'static> {
        // Note: Debug assert here exists as a sanity check not enforcement
        debug_assert!(std::str::from_utf8(&buffer).is_ok(), "invalid UTF-8");
        StringValueOrRef::Buffer(Utf8Buffer { buffer })
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        match self {
            StringValueOrRef::Empty => 0,
            StringValueOrRef::Ref(s) => s.len(),
            StringValueOrRef::Buffer(b) => b.buffer.len(),
            StringValueOrRef::Owned(s) => s.len(),
            StringValueOrRef::Slice(s) => s.len(),
        }
    }

    pub fn char_len(&self) -> usize {
        match self {
            StringValueOrRef::Empty => 0,
            StringValueOrRef::Ref(s) => s.chars().count(),
            StringValueOrRef::Buffer(b) => unsafe { std::str::from_utf8_unchecked(&b.buffer) }
                .chars()
                .count(),
            StringValueOrRef::Owned(s) => s.chars().count(),
            StringValueOrRef::Slice(s) => s.char_len(),
        }
    }

    pub fn char_indices(&self) -> CharIndices<'_> {
        match self {
            StringValueOrRef::Empty => CharIndices::String("".char_indices()),
            StringValueOrRef::Ref(s) => CharIndices::String(s.char_indices()),
            StringValueOrRef::Buffer(b) => CharIndices::String(
                unsafe { std::str::from_utf8_unchecked(&b.buffer) }.char_indices(),
            ),
            StringValueOrRef::Owned(s) => CharIndices::String(s.char_indices()),
            StringValueOrRef::Slice(s) => s.char_indices(),
        }
    }

    pub fn append_to(self, value: &mut String) {
        match self {
            StringValueOrRef::Empty => {}
            StringValueOrRef::Ref(s) => value.push_str(s),
            StringValueOrRef::Buffer(b) => {
                value.push_str(unsafe { std::str::from_utf8_unchecked(&b.buffer) })
            }
            StringValueOrRef::Owned(s) => value.push_str(&s),
            StringValueOrRef::Slice(s) => s.append_to(value),
        }
    }
}

impl<'a> StringValueOrRef<'a> {
    pub fn new_ref(value: &'a str) -> StringValueOrRef<'a> {
        StringValueOrRef::Ref(value)
    }
}

impl AsRef<str> for StringValueOrRef<'_> {
    fn as_ref(&self) -> &str {
        match self {
            StringValueOrRef::Empty => "",
            StringValueOrRef::Ref(s) => s,
            StringValueOrRef::Buffer(b) => unsafe { std::str::from_utf8_unchecked(&b.buffer) },
            StringValueOrRef::Owned(s) => s,
            StringValueOrRef::Slice(s) => s.get_value(),
        }
    }
}

impl<'a> From<&ValueOrRef<'a>> for StringValueOrRef<'a> {
    fn from(value: &ValueOrRef<'a>) -> Self {
        match value {
            ValueOrRef::Null => StringValueOrRef::Empty,
            ValueOrRef::String(s) => s.clone(),
            v => StringValueOrRef::Owned(Rc::new(v.to_value().convert_to_string().into())),
        }
    }
}

impl From<StringValueOrRef<'_>> for String {
    fn from(value: StringValueOrRef) -> Self {
        match value {
            StringValueOrRef::Empty => String::new(),
            StringValueOrRef::Ref(s) => s.into(),
            StringValueOrRef::Buffer(b) => {
                unsafe { std::str::from_utf8_unchecked(&b.buffer) }.into()
            }
            StringValueOrRef::Owned(s) => match Rc::try_unwrap(s) {
                Ok(s) => s,
                Err(o) => (*o).clone(),
            },
            StringValueOrRef::Slice(s) => {
                let mut v = String::new();
                s.append_to(&mut v);
                v
            }
        }
    }
}

impl Hash for StringValueOrRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_value().hash(state);
    }
}

impl PartialEq for StringValueOrRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.get_value() == other.get_value()
    }
}

impl Eq for StringValueOrRef<'_> {}

#[derive(Debug, Clone)]
pub struct Utf8Buffer {
    pub(crate) buffer: Buffer,
}

#[derive(Debug, Clone)]
pub struct StringValueOrRefSlice<'a> {
    value: Box<StringValueOrRef<'a>>,
    byte_start_inclusive: usize,
    byte_end_exclusive: usize,
    char_len: usize,
}

impl StringValueOrRefSlice<'_> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.byte_end_exclusive - self.byte_start_inclusive
    }

    pub fn char_len(&self) -> usize {
        self.char_len
    }

    pub fn char_indices(&self) -> CharIndices<'_> {
        CharIndices::Slice(StringValueOrRefSliceCharIndices {
            source: self.value.as_ref().char_indices().into(),
            position: 0,
            start_byte_index: self.byte_start_inclusive,
            end_char_index_exclusive: self.char_len,
        })
    }

    pub fn append_to(self, value: &mut String) {
        value.reserve(self.len());
        for (_, c) in self.char_indices() {
            value.push(c);
        }
    }
}

impl StringValue for StringValueOrRefSlice<'_> {
    fn get_value(&self) -> &str {
        let value = self.value.get_value();

        &value[self.byte_start_inclusive..self.byte_end_exclusive]
    }
}

pub enum CharIndices<'a> {
    String(std::str::CharIndices<'a>),
    Slice(StringValueOrRefSliceCharIndices<'a>),
}

impl Iterator for CharIndices<'_> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CharIndices::String(c) => c.next(),
            CharIndices::Slice(c) => loop {
                let v = c.source.next()?;
                if v.0 < c.start_byte_index {
                    continue;
                }
                if c.position >= c.end_char_index_exclusive {
                    return None;
                }
                c.position += 1;
                return Some((v.0 - c.start_byte_index, v.1));
            },
        }
    }
}

pub struct StringValueOrRefSliceCharIndices<'a> {
    source: Box<CharIndices<'a>>,
    position: usize,
    start_byte_index: usize,
    end_char_index_exclusive: usize,
}
