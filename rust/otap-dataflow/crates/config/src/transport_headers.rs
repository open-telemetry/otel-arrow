// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transport headers carried through the pipeline context.
//!
//! Headers can come from gRPC metadata, HTTP headers, or other transports.
//!
//! Preserves:
//! - Duplicate header names
//! - Text and binary values
//! - Original wire names when required
//! - Stored context entry names for policy matching

use crate::context::ContextEntryName;
use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

const PACKED_HEADER_LEN: usize = 28;

/// Kind of header value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ValueKind {
    /// UTF-8 text value.
    Text,
    /// Arbitrary binary value (e.g. gRPC `-bin` metadata).
    Binary,
}

impl ValueKind {
    fn decode(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Text),
            1 => Some(Self::Binary),
            _ => None,
        }
    }
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueKind::Text => write!(f, "text"),
            ValueKind::Binary => write!(f, "binary"),
        }
    }
}

/// A header value and its optional original name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportHeaderValue {
    /// Original wire name when required and different from the stored name.
    pub original_name: Option<Box<str>>,
    /// Whether the value is text or binary.
    pub value_kind: ValueKind,
    /// Raw value bytes.
    pub bytes: Box<[u8]>,
}

/// A transport header stored in context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportHeader {
    /// Stored context entry name.
    pub name: ContextEntryName,
    /// Value and optional original wire name.
    pub value: TransportHeaderValue,
}

impl TransportHeader {
    /// Creates a header without an original wire name.
    #[must_use]
    pub fn new<V: Into<Box<[u8]>>>(
        name: ContextEntryName,
        value_kind: ValueKind,
        value: V,
    ) -> Self {
        TransportHeader {
            name,
            value: TransportHeaderValue {
                original_name: None,
                value_kind,
                bytes: value.into(),
            },
        }
    }

    /// Creates a captured header. Keeps a distinct original name when requested.
    #[must_use]
    pub fn captured<V: Into<Box<[u8]>>>(
        name: ContextEntryName,
        wire_name: &str,
        preserve_original_name: bool,
        value_kind: ValueKind,
        value: V,
    ) -> Self {
        let original_name = (preserve_original_name && name.as_str() != wire_name)
            .then(|| wire_name.to_owned().into_boxed_str());
        Self {
            name,
            value: TransportHeaderValue {
                original_name,
                value_kind,
                bytes: value.into(),
            },
        }
    }

    /// Creates a text header.
    #[must_use]
    pub fn text(name: ContextEntryName, value: impl Into<Vec<u8>>) -> Self {
        Self::new(name, ValueKind::Text, value.into())
    }

    /// Creates a binary header.
    #[must_use]
    pub fn binary(name: ContextEntryName, value: impl Into<Vec<u8>>) -> Self {
        Self::new(name, ValueKind::Binary, value.into())
    }

    /// Returns the original wire name or the stored name.
    #[must_use]
    pub fn wire_name(&self) -> &str {
        self.value.original_name.as_deref().unwrap_or(&self.name)
    }

    /// Returns the value as a UTF-8 string, if it is valid text.
    #[must_use]
    pub fn value_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.value.bytes).ok()
    }
}

/// Borrowed configured context entry name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContextEntryNameRef<'a>(&'a str);

impl<'a> ContextEntryNameRef<'a> {
    /// Returns the configured name.
    #[must_use]
    pub const fn as_str(&self) -> &'a str {
        self.0
    }
}

impl AsRef<str> for ContextEntryNameRef<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Deref for ContextEntryNameRef<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Hash for ContextEntryNameRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq<str> for ContextEntryNameRef<'_> {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ContextEntryNameRef<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Borrowed value stored in packed transport-header context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransportHeaderValueRef<'a> {
    /// Original wire name when retained and distinct from the stored name.
    pub original_name: Option<&'a str>,
    /// Whether the value is text or binary.
    pub value_kind: ValueKind,
    /// Raw value bytes.
    pub bytes: &'a [u8],
}

/// Borrowed view of one packed transport header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransportHeaderRef<'a> {
    /// Stored context entry name.
    pub name: ContextEntryNameRef<'a>,
    /// Value and optional original wire name.
    pub value: TransportHeaderValueRef<'a>,
}

impl<'a> TransportHeaderRef<'a> {
    /// Returns the original wire name or the stored name.
    #[must_use]
    pub fn wire_name(&self) -> &'a str {
        self.value.original_name.unwrap_or(self.name.0)
    }

    /// Returns the value as UTF-8 when valid.
    #[must_use]
    pub fn value_as_str(&self) -> Option<&'a str> {
        std::str::from_utf8(self.value.bytes).ok()
    }
}

impl PartialEq<TransportHeader> for TransportHeaderRef<'_> {
    fn eq(&self, other: &TransportHeader) -> bool {
        *self == TransportHeaderRef::from(other)
    }
}

impl PartialEq<&TransportHeader> for TransportHeaderRef<'_> {
    fn eq(&self, other: &&TransportHeader) -> bool {
        *self == TransportHeaderRef::from(*other)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PackedTransportHeaders {
    bytes: Box<[u8]>,
    count: usize,
}

pub(crate) struct CapturedTransportHeader<'name, 'value> {
    pub(crate) name: &'name ContextEntryName,
    pub(crate) wire_name: &'value str,
    pub(crate) preserve_original_name: bool,
    pub(crate) value_kind: ValueKind,
    pub(crate) value: Cow<'value, [u8]>,
}

#[derive(Debug, PartialEq, Eq)]
enum TransportHeadersStorage {
    Owned(Vec<TransportHeader>),
    Packed(PackedTransportHeaders),
}

/// An ordered collection of captured transport headers.
///
/// Captured names, values, and descriptors share one immutable packed byte
/// block. Cloning is a reference-count bump, while iteration returns borrowed
/// views without rebuilding owned headers.
#[derive(Clone, Default)]
pub struct TransportHeaders {
    storage: Option<Arc<TransportHeadersStorage>>,
}

impl fmt::Debug for TransportHeaders {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

impl PartialEq for TransportHeaders {
    fn eq(&self, other: &Self) -> bool {
        match (&self.storage, &other.storage) {
            (Some(left), Some(right)) if Arc::ptr_eq(left, right) => true,
            _ => self.len() == other.len() && self.iter().eq(other.iter()),
        }
    }
}

impl Eq for TransportHeaders {}

impl TransportHeaders {
    /// Create an empty header collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty header collection with space for at least `capacity` headers.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::default();
        }
        Self {
            storage: Some(Arc::new(TransportHeadersStorage::Owned(
                Vec::with_capacity(capacity),
            ))),
        }
    }

    /// Add a header to the collection.
    pub fn push(&mut self, header: TransportHeader) {
        if let Some(TransportHeadersStorage::Owned(headers)) =
            self.storage.as_mut().and_then(Arc::get_mut)
        {
            headers.push(header);
            return;
        }

        let mut headers = self.iter().map(TransportHeader::from).collect::<Vec<_>>();
        headers.push(header);
        self.storage = Some(Arc::new(TransportHeadersStorage::Owned(headers)));
    }

    /// Replaces this collection with packed headers.
    pub(crate) fn replace(&mut self, headers: Vec<TransportHeader>) {
        self.storage = PackedTransportHeaders::build(headers)
            .map(TransportHeadersStorage::Packed)
            .map(Arc::new);
    }

    /// Replaces this collection by packing borrowed capture results directly.
    pub(crate) fn replace_captured(&mut self, headers: &[CapturedTransportHeader<'_, '_>]) {
        self.storage = PackedTransportHeaders::build_captured(headers)
            .map(TransportHeadersStorage::Packed)
            .map(Arc::new);
    }

    /// Returns `true` if there are no headers.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of headers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.storage.as_deref().map_or(0, |storage| match storage {
            TransportHeadersStorage::Owned(headers) => headers.len(),
            TransportHeadersStorage::Packed(packed) => packed.count,
        })
    }

    /// Iterate over all headers.
    #[must_use]
    pub fn iter(&self) -> TransportHeadersIter<'_> {
        TransportHeadersIter {
            storage: self.storage.as_deref(),
            index: 0,
        }
    }

    /// Returns the header at `index`.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<TransportHeaderRef<'_>> {
        match self.storage.as_deref()? {
            TransportHeadersStorage::Owned(headers) => {
                headers.get(index).map(TransportHeaderRef::from)
            }
            TransportHeadersStorage::Packed(packed) => packed.get(index),
        }
    }

    /// Finds headers by exact stored name.
    /// Uses a linear scan for validation.
    pub fn find_by_name<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = TransportHeaderRef<'a>> {
        self.iter().filter(move |header| header.name.0 == name)
    }
}

impl From<TransportHeaderRef<'_>> for TransportHeader {
    fn from(header: TransportHeaderRef<'_>) -> Self {
        Self {
            name: ContextEntryName::try_from(header.name.0)
                .expect("packed context names were validated during construction"),
            value: TransportHeaderValue {
                original_name: header.value.original_name.map(Box::from),
                value_kind: header.value.value_kind,
                bytes: header.value.bytes.into(),
            },
        }
    }
}

impl PackedTransportHeaders {
    fn build(headers: Vec<TransportHeader>) -> Option<Self> {
        if headers.is_empty() {
            return None;
        }

        let descriptor_len = headers
            .len()
            .checked_mul(PACKED_HEADER_LEN)
            .expect("transport header descriptor length overflow");
        let blob_len = headers
            .iter()
            .try_fold(0usize, |total, header| {
                total
                    .checked_add(header.name.as_str().len())
                    .and_then(|total| {
                        total.checked_add(header.value.original_name.as_deref().map_or(0, str::len))
                    })
                    .and_then(|total| total.checked_add(header.value.bytes.len()))
            })
            .expect("transport header blob length overflow");
        let mut bytes = vec![
            0;
            descriptor_len
                .checked_add(blob_len)
                .expect("transport header packed length overflow")
        ];
        let mut blob_at = descriptor_len;

        for (index, header) in headers.into_iter().enumerate() {
            let descriptor_at = index * PACKED_HEADER_LEN;
            let stored = write_blob(&mut bytes, &mut blob_at, header.name.as_str().as_bytes())
                .expect("preallocated transport header name range");
            let original = header.value.original_name.as_deref().map(|name| {
                write_blob(&mut bytes, &mut blob_at, name.as_bytes())
                    .expect("preallocated original transport header name range")
            });
            let value = write_blob(&mut bytes, &mut blob_at, &header.value.bytes)
                .expect("preallocated transport header value range");

            write_range(&mut bytes, descriptor_at, stored)
                .expect("transport header name range fits packed descriptor");
            write_range(&mut bytes, descriptor_at + 8, original.unwrap_or((0, 0)))
                .expect("original header name range fits packed descriptor");
            write_range(&mut bytes, descriptor_at + 16, value)
                .expect("transport header value range fits packed descriptor");
            bytes[descriptor_at + 24] = header.value.value_kind as u8;
        }

        Some(Self {
            bytes: bytes.into_boxed_slice(),
            count: descriptor_len / PACKED_HEADER_LEN,
        })
    }

    fn build_captured(headers: &[CapturedTransportHeader<'_, '_>]) -> Option<Self> {
        if headers.is_empty() {
            return None;
        }

        let descriptor_len = headers
            .len()
            .checked_mul(PACKED_HEADER_LEN)
            .expect("transport header descriptor length overflow");
        let blob_len = headers
            .iter()
            .try_fold(0usize, |total, header| {
                total
                    .checked_add(header.name.as_str().len())
                    .and_then(|total| {
                        total.checked_add(
                            if header.preserve_original_name
                                && header.name.as_str() != header.wire_name
                            {
                                header.wire_name.len()
                            } else {
                                0
                            },
                        )
                    })
                    .and_then(|total| total.checked_add(header.value.len()))
            })
            .expect("transport header blob length overflow");
        let mut bytes = vec![
            0;
            descriptor_len
                .checked_add(blob_len)
                .expect("transport header packed length overflow")
        ];
        let mut blob_at = descriptor_len;

        for (index, header) in headers.iter().enumerate() {
            let descriptor_at = index * PACKED_HEADER_LEN;
            let stored = write_blob(&mut bytes, &mut blob_at, header.name.as_str().as_bytes())
                .expect("preallocated transport header name range");
            let original = (header.preserve_original_name
                && header.name.as_str() != header.wire_name)
                .then(|| {
                    write_blob(&mut bytes, &mut blob_at, header.wire_name.as_bytes())
                        .expect("preallocated original transport header name range")
                });
            let value = write_blob(&mut bytes, &mut blob_at, header.value.as_ref())
                .expect("preallocated transport header value range");

            write_range(&mut bytes, descriptor_at, stored)
                .expect("transport header name range fits packed descriptor");
            write_range(&mut bytes, descriptor_at + 8, original.unwrap_or((0, 0)))
                .expect("original header name range fits packed descriptor");
            write_range(&mut bytes, descriptor_at + 16, value)
                .expect("transport header value range fits packed descriptor");
            bytes[descriptor_at + 24] = header.value_kind as u8;
        }

        Some(Self {
            bytes: bytes.into_boxed_slice(),
            count: headers.len(),
        })
    }

    fn get(&self, index: usize) -> Option<TransportHeaderRef<'_>> {
        if index >= self.count {
            return None;
        }
        let descriptor_at = index.checked_mul(PACKED_HEADER_LEN)?;
        let stored = read_range(&self.bytes, descriptor_at)?;
        let original = read_range(&self.bytes, descriptor_at + 8)?;
        let value = read_range(&self.bytes, descriptor_at + 16)?;
        let stored = read_str(&self.bytes, stored)?;
        let original_name = if original.1 == 0 {
            None
        } else {
            Some(read_str(&self.bytes, original)?)
        };
        Some(TransportHeaderRef {
            name: ContextEntryNameRef(stored),
            value: TransportHeaderValueRef {
                original_name,
                value_kind: ValueKind::decode(*self.bytes.get(descriptor_at + 24)?)?,
                bytes: read_bytes(&self.bytes, value)?,
            },
        })
    }
}

/// Iterator over packed transport headers.
pub struct TransportHeadersIter<'a> {
    storage: Option<&'a TransportHeadersStorage>,
    index: usize,
}

impl<'a> Iterator for TransportHeadersIter<'a> {
    type Item = TransportHeaderRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let storage = self.storage?;
        let count = match storage {
            TransportHeadersStorage::Owned(headers) => headers.len(),
            TransportHeadersStorage::Packed(packed) => packed.count,
        };
        if self.index >= count {
            return None;
        }

        let index = self.index;
        self.index += 1;
        let header = match storage {
            TransportHeadersStorage::Owned(headers) => {
                headers.get(index).map(TransportHeaderRef::from)
            }
            TransportHeadersStorage::Packed(packed) => packed.get(index),
        };
        if header.is_none() {
            tracing::error!(
                name: "context.transport_headers.decode_failed",
                header_index = index,
                header_count = count,
                message = "Packed transport header could not be decoded",
            );
            debug_assert!(
                header.is_some(),
                "packed transport header {index} of {count} must decode"
            );
            self.index = count;
        }
        header
    }
}

impl<'a> From<&'a TransportHeader> for TransportHeaderRef<'a> {
    fn from(header: &'a TransportHeader) -> Self {
        Self {
            name: ContextEntryNameRef(header.name.as_str()),
            value: TransportHeaderValueRef {
                original_name: header.value.original_name.as_deref(),
                value_kind: header.value.value_kind,
                bytes: &header.value.bytes,
            },
        }
    }
}

fn write_blob(bytes: &mut [u8], at: &mut usize, value: &[u8]) -> Option<(usize, usize)> {
    let start = *at;
    let end = start.checked_add(value.len())?;
    bytes.get_mut(start..end)?.copy_from_slice(value);
    *at = end;
    Some((start, value.len()))
}

fn write_range(bytes: &mut [u8], at: usize, range: (usize, usize)) -> Option<()> {
    write_u32(bytes, at, range.0)?;
    write_u32(bytes, at + 4, range.1)
}

fn write_u32(bytes: &mut [u8], at: usize, value: usize) -> Option<()> {
    bytes
        .get_mut(at..at.checked_add(4)?)?
        .copy_from_slice(&u32::try_from(value).ok()?.to_le_bytes());
    Some(())
}

fn read_range(bytes: &[u8], at: usize) -> Option<(usize, usize)> {
    Some((read_u32(bytes, at)?, read_u32(bytes, at + 4)?))
}

fn read_u32(bytes: &[u8], at: usize) -> Option<usize> {
    Some(u32::from_le_bytes(bytes.get(at..at.checked_add(4)?)?.try_into().ok()?) as usize)
}

fn read_bytes(bytes: &[u8], range: (usize, usize)) -> Option<&[u8]> {
    bytes.get(range.0..range.0.checked_add(range.1)?)
}

fn read_str(bytes: &[u8], range: (usize, usize)) -> Option<&str> {
    std::str::from_utf8(read_bytes(bytes, range)?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport_headers_policy::{
        CaptureDefaults, CaptureRule, HeaderCapturePolicy, HeaderPropagationPolicy, NameStrategy,
        PropagationAction, PropagationDefault, PropagationMatch, PropagationOverride,
        PropagationSelector, PropagationSelectorType,
    };

    fn context_name(raw: &str) -> ContextEntryName {
        ContextEntryName::try_from(raw).expect("valid test context entry name")
    }

    fn header(normal: &str, wire_name: &str, value: impl AsRef<[u8]>) -> TransportHeader {
        TransportHeader::captured(
            context_name(normal),
            wire_name,
            true,
            ValueKind::Text,
            value.as_ref(),
        )
    }

    /// Scenario: matching and nonmatching headers are interleaved.
    /// Guarantees: lookup preserves all matching values in order.
    #[test]
    fn find_by_name_returns_matching_headers() {
        let mut headers = TransportHeaders::new();
        headers.push(header("tenant", "X-Tenant", b"a"));
        headers.push(header("request-id", "X-Request-Id", b"b"));
        headers.push(header("tenant", "X-Tenant", b"c"));

        let tenants: Vec<_> = headers.find_by_name("tenant").collect();
        assert_eq!(tenants.len(), 2);
        assert_eq!(tenants[0].value.bytes, b"a");
        assert_eq!(tenants[1].value.bytes, b"c");
    }

    /// Scenario: two headers have the same name.
    /// Guarantees: neither header replaces the other.
    #[test]
    fn duplicate_names_preserved() {
        let mut headers = TransportHeaders::new();
        headers.push(header("key", "Key", b"val1"));
        headers.push(header("key", "Key", b"val2"));
        assert_eq!(headers.len(), 2);
    }

    /// Scenario: a text header contains valid UTF-8.
    /// Guarantees: the string view preserves the text.
    #[test]
    fn value_as_str_for_text() {
        let h = header("name", "Name", b"hello");
        assert_eq!(h.value_as_str(), Some("hello"));
    }

    /// Scenario: a binary header contains invalid UTF-8.
    /// Guarantees: the string view returns `None`.
    #[test]
    fn value_as_str_for_invalid_utf8() {
        let h = TransportHeader::binary(context_name("name-bin"), vec![0xFF, 0xFE]);
        assert_eq!(h.value_as_str(), None);
    }

    // -- Capture engine tests ------------------------------------------------

    fn make_capture_policy(rules: Vec<CaptureRule>) -> HeaderCapturePolicy {
        HeaderCapturePolicy::new(CaptureDefaults::default(), rules)
    }

    fn rule(names: &[&str], store_as: Option<&str>) -> CaptureRule {
        CaptureRule {
            match_names: names.iter().map(|n| context_name(n)).collect(),
            store_as: store_as.map(context_name),
            sensitive: false,
            value_kind: None,
        }
    }

    /// Scenario: headers arrive with no capture rules.
    /// Guarantees: capture returns no headers or limit errors.
    #[test]
    fn capture_empty_policy_captures_nothing() {
        let policy = HeaderCapturePolicy::default().compile(|_| true);
        let pairs = vec![("X-Tenant-Id", b"abc" as &[u8])];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert!(result.is_empty());
        assert!(stats.is_none());
    }

    /// Scenario: capture rules select and rename incoming headers.
    /// Guarantees: only matches are stored. Values and original names are retained.
    #[test]
    fn capture_matching_headers() {
        let policy = make_capture_policy(vec![
            rule(&["x-tenant-id"], Some("tenant_id")),
            rule(&["x-request-id"], None),
        ])
        .compile(|_| true);

        let pairs: Vec<(&str, &[u8])> = vec![
            ("X-Tenant-Id", b"t-123"),
            ("X-Request-Id", b"r-456"),
            ("X-Unmatched", b"ignored"),
        ];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert!(stats.is_none());
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).expect("first header").name, "tenant_id");
        assert_eq!(
            result.get(0).expect("first header").wire_name(),
            "X-Tenant-Id"
        );
        assert_eq!(result.get(0).expect("first header").value.bytes, b"t-123");
        assert_eq!(result.get(1).expect("second header").name, "x-request-id");
        assert!(matches!(
            result.storage.as_deref(),
            Some(TransportHeadersStorage::Packed(_))
        ));
    }

    /// Scenario: a packed header descriptor is corrupted below its declared count.
    /// Guarantees: iteration detects the violated decode invariant instead of failing silently.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "packed transport header 0 of 1 must decode")]
    fn packed_header_decode_failure_triggers_debug_assertion() {
        let mut headers = TransportHeaders::new();
        headers.replace(vec![header("tenant", "X-Tenant", b"acme")]);
        let storage = Arc::get_mut(headers.storage.as_mut().expect("packed storage is present"))
            .expect("packed storage is uniquely owned");
        let TransportHeadersStorage::Packed(packed) = storage else {
            panic!("expected packed transport header storage");
        };
        packed.bytes[0..4].copy_from_slice(&u32::MAX.to_le_bytes());

        let _ = headers.iter().next();
    }

    /// Scenario: a cloned manually-built collection is mutated.
    /// Guarantees: copy-on-write preserves the original collection and both borrowed views.
    #[test]
    fn pushed_headers_preserve_copy_on_write() {
        let mut original = TransportHeaders::new();
        original.push(header("tenant", "X-Tenant", b"a"));
        let mut changed = original.clone();

        changed.push(header("request", "X-Request", b"b"));

        assert_eq!(original.len(), 1);
        assert_eq!(original.get(0).expect("original header").value.bytes, b"a");
        assert_eq!(changed.len(), 2);
        assert_eq!(changed.get(1).expect("appended header").value.bytes, b"b");
    }

    /// Scenario: a wire name uses different casing from its capture rule.
    /// Guarantees: it matches the rule and retains its wire spelling.
    #[test]
    fn capture_case_insensitive_matching() {
        let policy = make_capture_policy(vec![rule(&["x-tenant-id"], None)]).compile(|_| true);

        let pairs: Vec<(&str, &[u8])> = vec![("X-TENANT-ID", b"val")];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert!(stats.is_none());
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).expect("captured header").name, "x-tenant-id");
        assert_eq!(
            result.get(0).expect("captured header").wire_name(),
            "X-TENANT-ID"
        );
    }

    /// Scenario: one capture rule matches several wire names.
    /// Guarantees: every alias is captured under the same stored name.
    #[test]
    fn capture_rule_supports_multiple_match_names() {
        let policy = make_capture_policy(vec![rule(&["x-first", "x-second"], Some("combined"))])
            .compile(|_| false);

        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(
            [
                ("X-First", b"first".as_slice()),
                ("X-Second", b"second".as_slice()),
            ]
            .into_iter(),
            &mut result,
        );

        assert!(stats.is_none());
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|header| header.name == "combined"));
    }

    /// Scenario: matching headers exceed the entry limit.
    /// Guarantees: capture respects the limit and counts excess matches.
    #[test]
    fn capture_respects_max_entries() {
        let mut policy = make_capture_policy(vec![rule(&["x-key"], None)]);
        policy.defaults.max_entries = 2;
        let policy = policy.compile(|_| true);

        let pairs: Vec<(&str, &[u8])> = vec![("x-key", b"1"), ("x-key", b"2"), ("x-key", b"3")];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert_eq!(result.len(), 2);
        let stats = stats.expect("should report skipped headers");
        assert_eq!(stats.skipped_max_entries, 1);
        assert_eq!(stats.skipped_name_too_long, 0);
        assert_eq!(stats.skipped_value_too_long, 0);
    }

    /// Scenario: an oversized value precedes a valid value.
    /// Guarantees: the oversized value is counted and dropped. The valid value is captured.
    #[test]
    fn capture_drops_oversized_value() {
        let mut policy = make_capture_policy(vec![rule(&["x-key"], None)]);
        policy.defaults.max_value_bytes = 3;
        let policy = policy.compile(|_| true);

        let pairs: Vec<(&str, &[u8])> = vec![("x-key", b"toolong"), ("x-key", b"ok")];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).expect("captured header").value.bytes, b"ok");
        let stats = stats.expect("should report skipped headers");
        assert_eq!(stats.skipped_value_too_long, 1);
        assert_eq!(stats.skipped_max_entries, 0);
        assert_eq!(stats.skipped_name_too_long, 0);
    }

    /// Scenario: a `-bin` header contains non-text bytes.
    /// Guarantees: capture accepts the bytes and marks the value as binary.
    #[test]
    fn capture_binary_detection() {
        let policy = make_capture_policy(vec![rule(&["auth-token-bin"], None)]).compile(|_| true);

        let pairs: Vec<(&str, &[u8])> = vec![("auth-token-bin", &[0xFF, 0x00])];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert!(stats.is_none());
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.get(0).expect("captured header").value.value_kind,
            ValueKind::Binary
        );
    }

    // -- Propagation policy tests --------------------------------------------

    /// Scenario: propagation selects all captured headers.
    /// Guarantees: every header keeps its original wire name.
    #[test]
    fn propagate_all_captured_default() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![],
        );
        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 2);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
        assert_eq!(propagated[1].header_name, "X-Request-Id");
    }

    /// Scenario: a drop override excludes authorization.
    /// Guarantees: other headers still propagate with their original names.
    #[test]
    fn propagate_override_drops_auth() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec![context_name("authorization")],
                },
                action: PropagationAction::Drop,
                name: None,
                on_error: None,
            }],
        );

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("authorization", "Authorization", b"Bearer secret"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }

    /// Scenario: a `none` selector has one propagation override.
    /// Guarantees: only the overridden header is propagated.
    #[test]
    fn propagate_selector_none_drops_all_unless_override() {
        let policy = HeaderPropagationPolicy {
            default: PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::None,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            overrides: vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec![context_name("tenant_id")],
                },
                action: PropagationAction::Propagate,
                name: None,
                on_error: None,
            }],
        };

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }

    /// Scenario: propagation uses `StoredName` for a renamed header.
    /// Guarantees: the stored name replaces the original wire name.
    #[test]
    fn propagate_stored_name_strategy() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                name: NameStrategy::StoredName,
                ..PropagationDefault::default()
            },
            vec![],
        );

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "tenant_id");
    }

    /// Scenario: a named selector lists one captured entry.
    /// Guarantees: only that entry is propagated.
    #[test]
    fn propagate_named_selector() {
        let policy = HeaderPropagationPolicy {
            default: PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::Named,
                    named: Some(vec![context_name("tenant_id")]),
                },
                ..PropagationDefault::default()
            },
            overrides: vec![],
        };

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }
}
