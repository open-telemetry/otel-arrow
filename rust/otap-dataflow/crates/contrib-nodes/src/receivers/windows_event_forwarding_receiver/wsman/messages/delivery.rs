// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extract rendered event documents and an optional bookmark from an Events request.
//!
//! This is the protocol extraction step, not Windows event decoding or delivery
//! acceptance. The enclosing `Request` has already validated the SOAP action and
//! body shape. HTTP handling remains responsible for authentication, destination
//! and subscription checks, and withholding the protocol Ack until delivery
//! completes. Event conversion subsequently validates each inner Windows XML
//! document and removes recognized synthetic bookmark events from emitted logs.
//!
//! # Event representation and bounds
//!
//! Every direct element inside `w:Events` must be `w:Event` in the WS-Man namespace
//! with the exact supported Event action URI. Its scalar content carries an inner
//! XML document as text, including CDATA or escaped XML, rather than nested SOAP
//! elements. Extraction preserves input event order and returns no partial batch
//! if any entry or bookmark fails validation.
//!
//! `max_event_bytes` applies to each extracted UTF-8 string, not its original
//! encoded HTTP representation. This check occurs after scalar extraction. The
//! HTTP and envelope layers bound the overall request; event conversion enforces
//! the record-count bound. An empty `w:Events` collection is invalid, even though
//! a nonempty collection of synthetic markers may later produce no log records.
//!
//! # Bookmark retention
//!
//! At most one WS-Man Bookmark header is extracted. Its payload remains opaque:
//! this module checks its container shape but does not interpret cursor values.
//! Inherited namespace bindings are copied onto the Bookmark root when absent
//! there, allowing later reuse outside the original envelope. Existing root
//! declarations and the original inner XML events are retained; the output is
//! self-contained XML, not a byte-for-byte copy or a canonical representation.
//!
//! `max_bookmark_bytes` is checked both on the original UTF-8 fragment and after
//! namespace expansion. The expanded output is allocated before the second check.
//! Extraction does not store or commit the bookmark. The runtime owns ordering
//! and progress, including the local commit path for validated marker-only batches.

use super::{Action, Error, Request, WSMAN, reject_text, scalar_text, unique_child};
use crate::receivers::windows_event_forwarding_receiver::config::Limits;
use crate::receivers::windows_event_forwarding_receiver::xml::Node;
use quick_xml::{Reader, Writer, events::Event};

/// A complete rendered-text delivery, not yet accepted by the pipeline.
///
/// Event documents have passed wrapper and size checks only; their Windows event
/// schema and fields still need validation by the event encoder.
pub struct EventBatch {
    /// Extracted Windows Event XML strings in source order, including synthetic markers.
    pub events: Vec<String>,
    /// Self-contained WS-Man Bookmark header, including its wrapper element.
    ///
    /// Absence supplies no new bookmark candidate; it is not a request to clear
    /// committed progress. Runtime acceptance determines whether a candidate commits.
    pub bookmark: Option<String>,
}

impl Request<'_, '_> {
    /// Extract the advertised rendered-text format without authorizing an acknowledgement.
    ///
    /// Requires an Events action and at least one supported Event wrapper. Rejects
    /// non-whitespace text between wrappers, unsupported element names/actions,
    /// nested elements in event scalar content, duplicate Bookmark headers, and
    /// invalid bookmark shape. Inner event XML is not parsed by this method.
    ///
    /// Returns `UnsupportedAction` for other request actions, `TooLarge` for event
    /// or bookmark byte-limit violations, and validation/serialization errors for
    /// malformed content. HTTP status mapping is the caller's responsibility.
    pub fn event_batch(&self, limits: &Limits) -> Result<EventBatch, Error> {
        if self.action() != Action::Events {
            return Err(Error::UnsupportedAction);
        }
        let payload = self
            .envelope
            .body()
            .first_element_child()
            .expect("validated Events");
        reject_text(payload)?;
        let mut events = Vec::new();
        for event in payload.children().filter(Node::is_element) {
            if !event.has_tag_name((WSMAN, "Event"))
                || event.attribute("Action")
                    != Some("http://schemas.dmtf.org/wbem/wsman/1/wsman/Event")
            {
                return Err(Error::InvalidEnvelope("unsupported event format"));
            }
            let xml = scalar_text(event, "Event")?;
            if xml.len() > limits.max_event_bytes {
                return Err(Error::TooLarge);
            }
            events.push(xml);
        }
        if events.is_empty() {
            return Err(Error::InvalidEnvelope("empty event batch"));
        }
        let header = self
            .envelope
            .document
            .root_element()
            .first_element_child()
            .expect("validated Header");
        let bookmark = unique_child(header, WSMAN, "Bookmark")?
            .map(|node| bookmark_xml(node, limits.max_bookmark_bytes))
            .transpose()?;
        Ok(EventBatch { events, bookmark })
    }
}

/// Copy a parsed Bookmark fragment and materialize its in-scope namespaces.
///
/// Allows an empty or text-only bookmark, or at most one direct child element.
/// With a child element, non-whitespace text alongside it is rejected. Descendant
/// cursor structure is otherwise opaque. The node comes from a validated XML
/// document; the streaming reader/writer copies its fragment without decoding
/// bookmark semantics. Explicit root namespace declarations are not replaced.
///
/// Both the source fragment and the resulting standalone XML must fit `limit`.
fn bookmark_xml(node: Node<'_, '_>, limit: usize) -> Result<String, Error> {
    let source = node.source();
    if source.len() > limit {
        return Err(Error::TooLarge);
    }
    if node.children().filter(Node::is_element).count() > 1 {
        return Err(Error::InvalidEnvelope("multiple bookmark roots"));
    }
    if node.first_element_child().is_some() {
        reject_text(node)?;
    }
    let mut reader = Reader::from_str(source);
    let mut writer = Writer::new(Vec::new());
    let first = reader
        .read_event()
        .map_err(|_| Error::InvalidEnvelope("invalid bookmark"))?;
    let empty = matches!(first, Event::Empty(_));
    let mut root = match first {
        Event::Start(root) | Event::Empty(root) => root.into_owned(),
        _ => return Err(Error::InvalidEnvelope("invalid bookmark")),
    };
    for namespace in node.namespaces() {
        let name = namespace
            .name()
            .map_or_else(|| "xmlns".to_owned(), |prefix| format!("xmlns:{prefix}"));
        if root
            .try_get_attribute(name.as_str())
            .map_err(|_| Error::InvalidEnvelope("invalid bookmark attribute"))?
            .is_none()
        {
            root.push_attribute((name.as_str(), namespace.uri()));
        }
    }
    writer.write_event(if empty {
        Event::Empty(root)
    } else {
        Event::Start(root)
    })?;
    loop {
        match reader
            .read_event()
            .map_err(|_| Error::InvalidEnvelope("invalid bookmark"))?
        {
            Event::Eof => break,
            event => writer.write_event(event)?,
        }
    }
    let bytes = writer.into_inner();
    if bytes.len() > limit {
        return Err(Error::TooLarge);
    }
    String::from_utf8(bytes).map_err(|_| Error::InvalidEncoding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receivers::windows_event_forwarding_receiver::wsman::{Envelope, Route};

    /// Scenario: rendered deliveries contain multiple event documents and an inherited-namespace bookmark.
    /// Guarantees: boundaries and bookmark namespaces survive extraction, while malformed and oversized batches fail.
    #[test]
    fn extracts_bounded_rendered_batch() {
        let make_xml = |events: &str, bookmark: &str| {
            format!(
                r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing" xmlns:w="{WSMAN}" xmlns:e="http://schemas.xmlsoap.org/ws/2004/08/eventing" xmlns:b="urn:bookmark"><s:Header><a:Action>{}</a:Action><a:MessageID>message</a:MessageID><a:To>https://collector/delivery</a:To><e:Identifier>subscription</e:Identifier><w:AckRequested/>{bookmark}</s:Header><s:Body><w:Events>{events}</w:Events></s:Body></s:Envelope>"#,
                Action::Events.uri()
            )
        };
        let event = r#"<w:Event Action="http://schemas.dmtf.org/wbem/wsman/1/wsman/Event"><![CDATA[<Event/>]]></w:Event>"#;
        let xml = make_xml(
            &event.repeat(2),
            "<w:Bookmark><b:Cursor>one</b:Cursor></w:Bookmark>",
        );
        let envelope = Envelope::parse(&xml, 8192).unwrap();
        let request = envelope
            .request(&Route::Delivery { version: "opaque" }, 8192)
            .unwrap();
        let batch = request.event_batch(&Limits::default()).unwrap();
        assert_eq!(batch.events, ["<Event/>", "<Event/>"]);
        let bookmark = crate::receivers::windows_event_forwarding_receiver::xml::Document::parse(
            batch.bookmark.as_ref().unwrap(),
        )
        .unwrap();
        assert!(
            bookmark
                .root_element()
                .first_element_child()
                .unwrap()
                .has_tag_name(("urn:bookmark", "Cursor"))
        );
        assert!(matches!(
            request.event_batch(&Limits {
                max_event_bytes: 1,
                ..Limits::default()
            }),
            Err(Error::TooLarge)
        ));
        assert!(matches!(
            request.event_batch(&Limits {
                max_bookmark_bytes: 1,
                ..Limits::default()
            }),
            Err(Error::TooLarge)
        ));
        for (events, bookmark) in [
            ("", ""),
            ("<w:Unknown/>", ""),
            (event, "<w:Bookmark><b:One/><b:Two/></w:Bookmark>"),
            (event, "<w:Bookmark>mixed<b:One/></w:Bookmark>"),
        ] {
            let xml = make_xml(events, bookmark);
            let envelope = Envelope::parse(&xml, 8192).unwrap();
            assert!(
                envelope
                    .request(&Route::Delivery { version: "opaque" }, 8192)
                    .unwrap()
                    .event_batch(&Limits::default())
                    .is_err()
            );
        }
    }
}
