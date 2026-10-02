// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::str::CharIndices;

use arrow::buffer::Buffer;
use otel_arrow_contrib_data_engine_expressions::*;

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
            StringValueOrRef::Empty => "".char_indices(),
            StringValueOrRef::Ref(s) => s.char_indices(),
            StringValueOrRef::Buffer(b) => {
                unsafe { std::str::from_utf8_unchecked(&b.buffer) }.char_indices()
            }
            StringValueOrRef::Owned(s) => s.char_indices(),
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
        self.get_value().char_indices()
    }

    pub fn append_to(self, value: &mut String) {
        value.push_str(self.get_value());
    }
}

impl StringValue for StringValueOrRefSlice<'_> {
    fn get_value(&self) -> &str {
        let value = self.value.get_value();

        &value[self.byte_start_inclusive..self.byte_end_exclusive]
    }
}

#[cfg(test)]
mod tests {
    use arrow::buffer::Buffer;

    use super::*;

    /// Scenario: Equivalent text is stored as an empty, borrowed, Arrow-buffered, or owned value.
    /// Guarantees: All representations report consistent byte and character lengths and convert to the same text.
    #[test]
    fn string_representations_have_consistent_behavior() {
        let values = [
            StringValueOrRef::Empty,
            StringValueOrRef::new_ref(""),
            StringValueOrRef::new_utf8(Buffer::from("".as_bytes())),
            StringValueOrRef::new_owned(String::new()),
        ];

        for value in values {
            assert!(value.is_empty());
            assert_eq!(value.len(), 0);
            assert_eq!(value.char_len(), 0);
            assert_eq!(String::from(value), "");
        }

        let text = "h\u{e9}llo";
        let values = [
            StringValueOrRef::new_ref(text),
            StringValueOrRef::new_utf8(Buffer::from(text.as_bytes())),
            StringValueOrRef::new_owned(text.to_string()),
        ];

        for value in values {
            assert_eq!(value.len(), 6);
            assert_eq!(value.char_len(), 5);
            assert_eq!(value.char_indices().collect::<Vec<_>>()[1], (1, '\u{e9}'));
            assert_eq!(String::from(value), text);
        }
    }

    /// Scenario: A slice spans multiple variable-width UTF-8 characters inside an owned string.
    /// Guarantees: Slice access, character indexes, and append operations remain relative to the slice.
    #[test]
    fn string_slice_handles_variable_width_characters() {
        let slice = StringValueOrRefSlice {
            value: Box::new(StringValueOrRef::new_owned("a\u{e9}\u{65e5}z".to_string())),
            byte_start_inclusive: 1,
            byte_end_exclusive: 6,
            char_len: 2,
        };

        assert!(!slice.is_empty());
        assert_eq!(slice.len(), 5);
        assert_eq!(slice.char_len(), 2);
        assert_eq!(slice.get_value(), "\u{e9}\u{65e5}");
        assert_eq!(
            slice.char_indices().collect::<Vec<_>>(),
            vec![(0, '\u{e9}'), (2, '\u{65e5}')]
        );

        let mut output = String::from("prefix:");
        slice.append_to(&mut output);
        assert_eq!(output, "prefix:\u{e9}\u{65e5}");
    }

    /// Scenario: Empty slices are taken at the beginning, middle, and end of a UTF-8 string.
    /// Guarantees: Character iteration is empty and appending preserves the destination at every valid boundary.
    #[test]
    fn empty_string_slices_have_no_characters_to_append() {
        for byte_index in [0, 1, 3, 6] {
            let slice = StringValueOrRefSlice {
                value: Box::new(StringValueOrRef::new_ref("a\u{e9}\u{65e5}")),
                byte_start_inclusive: byte_index,
                byte_end_exclusive: byte_index,
                char_len: 0,
            };

            assert!(slice.is_empty());
            assert_eq!(slice.char_indices().collect::<Vec<_>>(), vec![]);

            let mut output = String::from("existing");
            slice.append_to(&mut output);
            assert_eq!(output, "existing");
        }
    }

    /// Scenario: The same interior UTF-8 slice is backed by borrowed, Arrow-buffered, owned, and sliced values.
    /// Guarantees: Character indexes remain byte-relative to the outer slice and appending produces identical text.
    #[test]
    fn string_slice_behavior_is_consistent_across_backing_values() {
        let text = "a\u{e9}\u{65e5}z";
        let nested = StringValueOrRefSlice {
            value: Box::new(StringValueOrRef::new_ref(text)),
            byte_start_inclusive: 0,
            byte_end_exclusive: text.len(),
            char_len: 4,
        };
        let values = [
            StringValueOrRef::new_ref(text),
            StringValueOrRef::new_utf8(Buffer::from(text.as_bytes())),
            StringValueOrRef::new_owned(text.to_string()),
            StringValueOrRef::Slice(nested),
        ];

        for value in values {
            let slice = StringValueOrRefSlice {
                value: Box::new(value),
                byte_start_inclusive: 1,
                byte_end_exclusive: 6,
                char_len: 2,
            };

            assert_eq!(
                slice.char_indices().collect::<Vec<_>>(),
                vec![(0, '\u{e9}'), (2, '\u{65e5}')]
            );

            let mut output = String::from("prefix:");
            slice.append_to(&mut output);
            assert_eq!(output, "prefix:\u{e9}\u{65e5}");
        }
    }

    /// Scenario: A slice selects one multibyte character from another non-zero-offset slice.
    /// Guarantees: Nested character indexes reset to zero and appending excludes both inner-slice neighbors.
    #[test]
    fn nested_string_slice_indices_are_relative_to_outer_slice() {
        let inner = StringValueOrRefSlice {
            value: Box::new(StringValueOrRef::new_ref("x\u{e9}\u{65e5}y")),
            byte_start_inclusive: 1,
            byte_end_exclusive: 6,
            char_len: 2,
        };
        let outer = StringValueOrRefSlice {
            value: Box::new(StringValueOrRef::Slice(inner)),
            byte_start_inclusive: 2,
            byte_end_exclusive: 5,
            char_len: 1,
        };

        assert_eq!(
            outer.char_indices().collect::<Vec<_>>(),
            vec![(0, '\u{65e5}')]
        );

        let mut output = String::from("prefix:");
        outer.append_to(&mut output);
        assert_eq!(output, "prefix:\u{65e5}");
    }

    /// Scenario: An Arrow buffer contains bytes that are not valid UTF-8.
    /// Guarantees: The checked constructor rejects invalid text before it can be exposed as a string.
    #[test]
    #[should_panic(expected = "invalid UTF-8")]
    fn checked_utf8_constructor_rejects_invalid_bytes() {
        let _ = StringValueOrRef::new_utf8(Buffer::from(vec![0xff]));
    }
}
