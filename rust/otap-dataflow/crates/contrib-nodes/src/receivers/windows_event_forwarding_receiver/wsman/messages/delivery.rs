// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded extraction of rendered event XML from WS-Man Event wrappers.
//!
//! Extraction is not Windows event validation or delivery acceptance. Callers
//! must authenticate the source, validate destination and subscription identity,
//! and convert the complete batch before submitting it downstream. Neither this
//! module nor successful conversion authorizes an Ack or commits progress.
//! Bookmark-bearing deliveries remain unsupported until bookmark integration.

use super::{Action, Error, Request, WSMAN, reject_text};
use crate::common::xml::Node;
use crate::receivers::windows_event_forwarding_receiver::{config::Limits, event::MAX_EVENTS};

// Rendered XML is scalar content of a WS-Man Event, not nested SOAP markup.
const EVENT_ACTION: &str = "http://schemas.dmtf.org/wbem/wsman/1/wsman/Event";

impl Request<'_, '_> {
    /// Extract a nonempty rendered-text batch in source order.
    ///
    /// CDATA and escaped text are decoded once by the enclosing XML parser.
    /// Each wrapper must have the supported Event action and scalar content.
    /// Count, per-event UTF-8 bytes, and aggregate UTF-8 bytes are checked before
    /// allocating extracted strings. The envelope's byte/node limits separately
    /// bound the already parsed SOAP document.
    ///
    /// # Errors
    ///
    /// Returns no partial batch for unsupported actions, malformed wrappers,
    /// empty content, Bookmark headers, or count/byte-limit violations.
    pub fn event_batch(&self, limits: &Limits) -> Result<Vec<String>, Error> {
        if self.action() != Action::Events {
            return Err(Error::UnsupportedAction);
        }
        let header = self
            .envelope
            .document
            .root_element()
            .first_element_child()
            .expect("validated Header");
        if header
            .children()
            .any(|node| node.has_tag_name((WSMAN, "Bookmark")))
        {
            return Err(Error::InvalidEnvelope(
                "Bookmark extraction is not supported",
            ));
        }
        let payload = self
            .envelope
            .body()
            .first_element_child()
            .expect("validated Events");
        reject_text(payload)?;
        let mut events = Vec::new();
        let mut total_bytes = 0usize;
        for event in payload.children().filter(Node::is_element) {
            if events.len() == MAX_EVENTS {
                return Err(Error::TooLarge);
            }
            if !event.has_tag_name((WSMAN, "Event"))
                || event.attribute("Action") != Some(EVENT_ACTION)
                || event.first_element_child().is_some()
            {
                return Err(Error::InvalidEnvelope("unsupported event format"));
            }
            let mut bytes = 0usize;
            for text in event.children().filter(Node::is_text) {
                let len = text.text().unwrap_or_default().len();
                bytes = bytes.checked_add(len).ok_or(Error::TooLarge)?;
            }
            if bytes > limits.max_event_bytes
                || bytes > limits.max_decompressed_bytes.saturating_sub(total_bytes)
            {
                return Err(Error::TooLarge);
            }
            let mut xml = String::with_capacity(bytes);
            for text in event.children().filter(Node::is_text) {
                if let Some(text) = text.text() {
                    xml.push_str(text);
                }
            }
            if xml.trim().is_empty() {
                return Err(Error::InvalidEnvelope("empty event"));
            }
            total_bytes += bytes;
            events.push(xml);
        }
        if events.is_empty() {
            return Err(Error::InvalidEnvelope("empty event batch"));
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receivers::windows_event_forwarding_receiver::wsman::{Envelope, Route};

    fn envelope(events: &str, headers: &str) -> String {
        format!(
            r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing" xmlns:w="{WSMAN}" xmlns:e="http://schemas.xmlsoap.org/ws/2004/08/eventing"><s:Header><a:Action>{}</a:Action><a:MessageID>message</a:MessageID><a:To>https://collector/delivery</a:To><e:Identifier>subscription</e:Identifier><w:AckRequested/>{headers}</s:Header><s:Body><w:Events>{events}</w:Events></s:Body></s:Envelope>"#,
            Action::Events.uri()
        )
    }

    fn extract(xml: &str, limits: &Limits) -> Result<Vec<String>, Error> {
        Envelope::parse(xml, xml.len(), xml.len())?
            .request(&Route::Delivery { version: "opaque" }, 8192)?
            .event_batch(limits)
    }

    /// Scenario: CDATA and escaped XML span multiple text nodes separated by comments and PIs.
    /// Guarantees: wrapper extraction preserves order and whitespace, decodes once, and bounds UTF-8 bytes exactly.
    #[test]
    fn extracts_scalar_events_with_exact_byte_limits() {
        let contents = [
            " <![CDATA[<Event>]]><!--split--><![CDATA[\u{e9}&amp;</Event>]]> ",
            "&lt;Event&gt;<?split x?>&amp;lt;&lt;/Event&gt;",
        ];
        let wrappers: String = contents
            .iter()
            .map(|content| format!(r#"<w:Event Action="{EVENT_ACTION}">{content}</w:Event>"#))
            .collect();
        let xml = envelope(&wrappers, "");
        let expected = [" <Event>\u{e9}&amp;</Event> ", "<Event>&lt;</Event>"];
        let mut limits = Limits {
            max_event_bytes: expected.iter().map(|text| text.len()).max().unwrap(),
            max_decompressed_bytes: expected.iter().map(|text| text.len()).sum(),
            ..Limits::default()
        };
        assert_eq!(extract(&xml, &limits).unwrap(), expected);
        limits.max_event_bytes -= 1;
        assert!(matches!(extract(&xml, &limits), Err(Error::TooLarge)));
        limits.max_event_bytes += 1;
        limits.max_decompressed_bytes -= 1;
        assert!(matches!(extract(&xml, &limits), Err(Error::TooLarge)));
    }

    /// Scenario: an Events body contains malformed wrappers, mixed text, no events, or a Bookmark header.
    /// Guarantees: unsupported formats and deferred bookmarks reject the entire extraction, including a valid prefix.
    #[test]
    fn rejects_invalid_delivery_shapes() {
        let valid = format!(r#"<w:Event Action="{EVENT_ACTION}"><![CDATA[<Event/>]]></w:Event>"#);
        for invalid in [
            "<Event/>".to_owned(),
            "<w:Event/>".to_owned(),
            format!(r#"<x:Event xmlns:x="urn:wrong" Action="{EVENT_ACTION}">xml</x:Event>"#),
            format!(r#"<w:Event w:Action="{EVENT_ACTION}">xml</w:Event>"#),
            r#"<w:Event Action="urn:wrong">xml</w:Event>"#.to_owned(),
            format!(r#"<w:Event Action="{EVENT_ACTION}"><Event/></w:Event>"#),
            format!(r#"<w:Event Action="{EVENT_ACTION}"> &#13; </w:Event>"#),
            "unexpected text".to_owned(),
        ] {
            assert!(
                matches!(
                    extract(
                        &envelope(&format!("{valid}{invalid}"), ""),
                        &Limits::default()
                    ),
                    Err(Error::InvalidEnvelope(_))
                ),
                "{invalid}"
            );
        }
        assert!(extract(&envelope("", ""), &Limits::default()).is_err());
        assert!(matches!(
            extract(&envelope(&valid, "<w:Bookmark/>"), &Limits::default()),
            Err(Error::InvalidEnvelope(
                "Bookmark extraction is not supported"
            ))
        ));
        let heartbeat = envelope("", "").replace(Action::Events.uri(), Action::Heartbeat.uri());
        assert!(matches!(
            extract(&heartbeat, &Limits::default()),
            Err(Error::UnsupportedAction)
        ));
    }

    /// Scenario: a delivery reaches the OTAP input-count ceiling and then adds one more wrapper.
    /// Guarantees: exactly 65,535 wrappers are accepted and the next is rejected independently of bytes.
    #[test]
    fn bounds_event_count_before_conversion() {
        let wrapper = format!(r#"<w:Event Action="{EVENT_ACTION}">&lt;E/&gt;</w:Event>"#);
        let mut events = wrapper.repeat(MAX_EVENTS);
        assert_eq!(
            extract(&envelope(&events, ""), &Limits::default())
                .unwrap()
                .len(),
            MAX_EVENTS
        );
        events.push_str(&wrapper);
        assert!(matches!(
            extract(&envelope(&events, ""), &Limits::default()),
            Err(Error::TooLarge)
        ));
    }
}
