// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Owned, receiver-independent representation of a decoded Windows event.
//!
//! This is neither an XML tree nor an OTAP record. XML parsing is one way to
//! build it (see [`super::parse`]); native Windows Event Log APIs can populate
//! the same structure directly. Attribute naming, severity mapping, and OTAP
//! encoding belong to the consuming receiver.
//!
//! The fields follow the Windows Event Schema (`Event`, `System`, `EventData`,
//! `UserData`, `RenderingInfo`). See
//! <https://learn.microsoft.com/en-us/windows/win32/wes/eventschema-eventtype-complextype>
//! and <https://learn.microsoft.com/en-us/windows/win32/wes/eventschema-schema>.

/// A decoded Windows event independent of its transport and output encoding.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowsEvent {
    /// System-rendered metadata.
    pub system: System,
    /// Publisher-rendered message and level text, when present.
    pub rendering: Option<RenderingInfo>,
    /// The event's payload section, when present.
    pub payload: Option<Payload>,
}

/// The event's payload section: a schema choice of mutually exclusive variants.
///
/// `EventData` and `UserData` are structured. Other defined sections (for example
/// `DebugData`, `ProcessingErrorData`, `BinaryEventData`) and any unrecognized
/// section are retained verbatim as [`Payload::Other`] rather than dropped or
/// rejected; the original section name is preserved in [`Element::name`].
/// Dedicated variants for those sections may be added later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    /// Structured `EventData` payload.
    EventData(EventData),
    /// Namespace-aware `UserData` element tree.
    UserData(Element),
    /// Any other payload section, retained as a raw element tree until a dedicated
    /// variant (e.g. `DebugData`, `ProcessingErrorData`, `BinaryEventData`) is added.
    Other(Element),
}

/// System-section fields. Numeric identifiers are parsed; opaque values stay strings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct System {
    /// Publisher name, when present.
    pub provider_name: Option<String>,
    /// Publisher GUID, when present.
    pub provider_guid: Option<String>,
    /// Provider-defined event identifier.
    pub event_id: u32,
    /// Raw Windows level; severity derivation is a caller concern.
    pub level: u8,
    /// Event creation time in nanoseconds since the Unix epoch.
    pub time_created_unix_nano: i64,
    /// Channel name, when present.
    pub channel: Option<String>,
    /// Monotonic event record identifier, when present.
    pub record_id: Option<u64>,
    /// Task value, when present.
    pub task: Option<u32>,
    /// Opcode value, when present.
    pub opcode: Option<u32>,
    /// Raw keyword bitmask text (e.g. `0x8000000000000000`), preserved verbatim.
    pub keywords: Option<String>,
    /// Source computer name, when present.
    pub computer: Option<String>,
    /// Security user SID, when present.
    pub user_sid: Option<String>,
}

/// Publisher rendering information.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenderingInfo {
    /// Localized message; `Some("")` distinguishes an empty message from absence.
    pub message: Option<String>,
    /// Localized level name usable as severity text.
    pub level: Option<String>,
}

/// `EventData` payload, retaining every variant rather than dropping unsupported ones.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventData {
    /// `Data` entries in source order, retaining repeated names.
    pub entries: Vec<DataEntry>,
    /// `ComplexData` elements as namespace-aware trees, in source order.
    pub complex: Vec<Element>,
    /// `Binary` payload as its raw hex string, when present.
    pub binary: Option<String>,
}

/// One `EventData/Data` entry. Unnamed entries use a one-based `paramN` name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataEntry {
    /// Entry name: the `Name` attribute, or `paramN` for an unnamed entry.
    pub name: String,
    /// Decoded scalar value.
    pub value: String,
}

/// Namespace-aware element retained from `UserData`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Element {
    /// Local element name without its namespace prefix.
    pub name: String,
    /// Resolved namespace URI, when the element is namespace-qualified.
    pub namespace: Option<String>,
    /// Element attributes in source order.
    pub attributes: Vec<Attribute>,
    /// Ordered mixed element/text content.
    pub content: Vec<Content>,
}

/// An attribute on a `UserData` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// Local attribute name without its namespace prefix.
    pub name: String,
    /// Resolved namespace URI, when the attribute is namespace-qualified.
    pub namespace: Option<String>,
    /// Decoded attribute value.
    pub value: String,
}

/// Ordered mixed content within a `UserData` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    /// A nested child element.
    Element(Element),
    /// A decoded text run.
    Text(String),
}
