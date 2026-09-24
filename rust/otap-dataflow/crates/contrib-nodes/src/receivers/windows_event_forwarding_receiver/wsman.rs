// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::xml::{Document, Node};
use uuid::Uuid;

const SOAP: &str = "http://www.w3.org/2003/05/soap-envelope";
const ADDRESSING: &str = "http://schemas.xmlsoap.org/ws/2004/08/addressing";

/// Action-specific request checks and response encoding.
pub mod messages;

/// Errors detected before dispatching a WS-Man request.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Request path does not identify an advertised endpoint.
    #[error("unknown WEF request path")]
    UnknownRoute,
    /// Input or decoded XML exceeds the configured bound.
    #[error("SOAP body exceeds configured size limit")]
    TooLarge,
    /// Body is not valid in the encoding declared by the transport.
    #[error("invalid SOAP text encoding")]
    InvalidEncoding,
    /// XML is malformed or contains a prohibited DTD.
    #[error("invalid SOAP XML: {0}")]
    InvalidXml(#[from] super::xml::Error),
    /// SOAP structure or required addressing headers are invalid.
    #[error("invalid SOAP envelope: {0}")]
    InvalidEnvelope(&'static str),
    /// A termination child is unsupported; names are truncated before diagnostic formatting.
    #[error("unsupported SubscriptionEnd element: namespace={namespace:?}, name={name:?}")]
    UnsupportedSubscriptionEndElement {
        /// At most 128 characters of the namespace, never element content.
        namespace: String,
        /// At most 128 characters of the local name.
        name: String,
    },
    /// The action URI is unknown or not supported at this endpoint.
    #[error("unsupported WS-Man action for this endpoint")]
    UnsupportedAction,
    /// A mandatory SOAP header cannot be processed by this implementation.
    #[error("unsupported mandatory SOAP header: {0}")]
    MustUnderstand(String),
    /// Response serialization could not write its XML bytes.
    #[error("cannot encode SOAP response: {0}")]
    ResponseWrite(#[from] std::io::Error),
}

/// An endpoint identified from the HTTP URI path, without its query string.
#[derive(Debug, Eq, PartialEq)]
pub enum Route<'path> {
    /// Subscription enumeration and full-duplex termination endpoint.
    SubscriptionManager,
    /// Delivery endpoint for the configured subscription and an opaque version.
    Delivery {
        /// Exact version path segment, never interpreted as a UUID or compared.
        version: &'path str,
    },
}

impl<'path> Route<'path> {
    /// Match a path against the single configured subscription.
    ///
    /// The caller must authenticate and authorize the TLS source independently.
    /// Version text is retained as it appears in the URI path, without decoding.
    pub fn parse(path: &'path str, subscription_id: Uuid) -> Result<Self, Error> {
        if path == "/wsman/SubscriptionManager/WEC" {
            return Ok(Self::SubscriptionManager);
        }
        let suffix = path
            .strip_prefix("/wsman/subscriptions/")
            .ok_or(Error::UnknownRoute)?;
        let (identifier, version) = suffix.split_once('/').ok_or(Error::UnknownRoute)?;
        if Uuid::parse_str(identifier).ok() != Some(subscription_id)
            || version.is_empty()
            || version.contains('/')
        {
            return Err(Error::UnknownRoute);
        }
        Ok(Self::Delivery { version })
    }
}

/// Text encoding selected from the HTTP content type by the transport.
#[derive(Clone, Copy, Debug)]
pub enum Encoding {
    /// UTF-8, with an optional byte-order mark.
    Utf8,
    /// UTF-16 little endian, with an optional matching byte-order mark.
    Utf16Le,
    /// UTF-16 big endian, with an optional matching byte-order mark.
    Utf16Be,
}

/// Decode an already decompressed body without replacing invalid Unicode.
///
/// Both input bytes and decoded UTF-8 bytes are bounded. The HTTP layer must also
/// bound wire bytes and decompression before invoking this function.
pub fn decode_body(body: &[u8], encoding: Encoding, max_bytes: usize) -> Result<String, Error> {
    if body.len() > max_bytes {
        return Err(Error::TooLarge);
    }
    if matches!(encoding, Encoding::Utf8) {
        let body = body.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(body);
        return std::str::from_utf8(body)
            .map(str::to_owned)
            .map_err(|_| Error::InvalidEncoding);
    }
    if !body.len().is_multiple_of(2) {
        return Err(Error::InvalidEncoding);
    }
    let little_endian = matches!(encoding, Encoding::Utf16Le);
    let units = body.as_chunks::<2>().0.iter().map(|bytes| {
        if little_endian {
            u16::from_le_bytes(*bytes)
        } else {
            u16::from_be_bytes(*bytes)
        }
    });
    let mut decoded = String::new();
    for (index, character) in char::decode_utf16(units).enumerate() {
        let character = character.map_err(|_| Error::InvalidEncoding)?;
        if index == 0 {
            if character == '\u{feff}' {
                continue;
            }
            if character == '\u{fffe}' {
                return Err(Error::InvalidEncoding);
            }
        }
        if character.len_utf8() > max_bytes.saturating_sub(decoded.len()) {
            return Err(Error::TooLarge);
        }
        decoded.push(character);
    }
    Ok(decoded)
}

/// Namespace-checked SOAP 1.2 envelope borrowing the decoded request XML.
pub struct Envelope<'input> {
    document: Document<'input>,
    action: String,
    message_id: String,
}

impl<'input> Envelope<'input> {
    /// Parse a bounded envelope with exactly one Header followed by one Body.
    ///
    /// DTDs are disabled. Required WS-Addressing headers must be unique, nonempty
    /// scalar elements. Action-specific body and mustUnderstand checks are left
    /// to the dispatcher; this parser alone does not authorize an acknowledgement.
    pub fn parse(xml: &'input str, max_bytes: usize) -> Result<Self, Error> {
        if xml.len() > max_bytes {
            return Err(Error::TooLarge);
        }
        let document = Document::parse_bounded(xml, max_bytes)?;
        let root = document.root_element();
        if !root.has_tag_name((SOAP, "Envelope")) {
            return Err(Error::InvalidEnvelope("expected SOAP 1.2 Envelope"));
        }
        let header = {
            let mut elements = root.children().filter(Node::is_element);
            let header = elements
                .next()
                .filter(|node| node.has_tag_name((SOAP, "Header")))
                .ok_or(Error::InvalidEnvelope("expected Header"))?;
            let _body = elements
                .next()
                .filter(|node| node.has_tag_name((SOAP, "Body")))
                .ok_or(Error::InvalidEnvelope("expected Body"))?;
            if elements.next().is_some()
                || root.children().any(|node| {
                    node.is_text() && node.text().is_some_and(|text| !text.trim().is_empty())
                })
            {
                return Err(Error::InvalidEnvelope("unexpected Envelope content"));
            }
            header
        };
        let action = addressing_header(header, "Action")?;
        let message_id = addressing_header(header, "MessageID")?;
        Ok(Self {
            document,
            action,
            message_id,
        })
    }

    /// Return the action URI without case folding or whitespace normalization.
    #[must_use]
    pub fn action(&self) -> &str {
        &self.action
    }

    /// Return the original MessageID value for exact Ack RelatesTo correlation.
    #[must_use]
    pub fn message_id(&self) -> &str {
        &self.message_id
    }

    /// Return the validated Body element for action-specific parsing.
    #[must_use]
    pub fn body(&self) -> Node<'_, 'input> {
        self.document
            .root_element()
            .children()
            .find(|node| node.has_tag_name((SOAP, "Body")))
            .expect("Body validated during parsing")
    }
}

fn addressing_header(header: Node<'_, '_>, name: &'static str) -> Result<String, Error> {
    let mut matches = header
        .children()
        .filter(|node| node.has_tag_name((ADDRESSING, name)));
    let element = matches.next().ok_or(Error::InvalidEnvelope(name))?;
    if matches.next().is_some() {
        return Err(Error::InvalidEnvelope(name));
    }
    scalar_text(element, name)
}

fn scalar_text(element: Node<'_, '_>, name: &'static str) -> Result<String, Error> {
    if element.children().any(|node| node.is_element()) {
        return Err(Error::InvalidEnvelope(name));
    }
    let value: String = element
        .children()
        .filter(|node| node.is_text())
        .filter_map(|node| node.text())
        .collect();
    if value.trim().is_empty() {
        return Err(Error::InvalidEnvelope(name));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing">
<s:Header><a:Action>urn:test:Events</a:Action><a:MessageID>uuid:AbC-123</a:MessageID></s:Header>
<s:Body><Events><![CDATA[<Event>example</Event>]]></Events></s:Body></s:Envelope>"#;

    /// Scenario: deliveries use old, unknown, or non-UUID versions and incorrect routes.
    /// Guarantees: only the configured subscription is routed; version text stays opaque.
    #[test]
    fn routes_preserve_opaque_versions() {
        let subscription = Uuid::from_u128(1);
        assert_eq!(
            Route::parse("/wsman/SubscriptionManager/WEC", subscription).unwrap(),
            Route::SubscriptionManager
        );
        for version in ["old", "Unknown-VERSION", "not-a-uuid", "%41"] {
            let path = format!("/wsman/subscriptions/{subscription}/{version}");
            assert_eq!(
                Route::parse(&path, subscription).unwrap(),
                Route::Delivery { version }
            );
        }
        for path in [
            format!("/wsman/subscriptions/{subscription}/"),
            format!("/wsman/subscriptions/{subscription}/version/extra"),
            format!("/wsman/subscriptions/{}/version", Uuid::from_u128(2)),
            "/wsman/subscriptions/invalid/version".into(),
            "/unknown".into(),
        ] {
            assert!(matches!(
                Route::parse(&path, subscription),
                Err(Error::UnknownRoute)
            ));
        }
    }

    /// Scenario: SOAP arrives as UTF-8 or UTF-16 in either byte order, with or without BOM.
    /// Guarantees: decoding preserves Unicode, MessageID case, XML namespaces, and CDATA.
    #[test]
    fn decodes_and_parses_envelopes() {
        let xml = XML.replace("example", "\u{1f600}\u{00e9}");
        for (encoding, little_endian) in [(Encoding::Utf16Le, true), (Encoding::Utf16Be, false)] {
            for bom in [false, true] {
                let input = if bom {
                    format!("\u{feff}{xml}")
                } else {
                    xml.clone()
                };
                let bytes: Vec<u8> = input
                    .encode_utf16()
                    .flat_map(|unit| {
                        if little_endian {
                            unit.to_le_bytes()
                        } else {
                            unit.to_be_bytes()
                        }
                    })
                    .collect();
                let decoded = decode_body(&bytes, encoding, 8192).unwrap();
                assert_eq!(decoded, xml);
                let envelope = Envelope::parse(&decoded, 8192).unwrap();
                assert_eq!(envelope.action(), "urn:test:Events");
                assert_eq!(envelope.message_id(), "uuid:AbC-123");
                assert_eq!(
                    envelope.body().first_element_child().unwrap().text(),
                    Some("<Event>\u{1f600}\u{00e9}</Event>")
                );
            }
        }
        for input in [xml.clone(), format!("\u{feff}{xml}")] {
            assert_eq!(
                decode_body(input.as_bytes(), Encoding::Utf8, 8192).unwrap(),
                xml
            );
        }
        let renamed = XML
            .replace("<s:", "<soap:")
            .replace("</s:", "</soap:")
            .replace("xmlns:s=", "xmlns:soap=")
            .replace("<a:", "<address:")
            .replace("</a:", "</address:")
            .replace("xmlns:a=", "xmlns:address=");
        assert_eq!(
            Envelope::parse(&renamed, 8192).unwrap().message_id(),
            "uuid:AbC-123"
        );
    }

    /// Scenario: text contains invalid Unicode, mismatched BOMs, or exceeds byte limits.
    /// Guarantees: decoding fails without replacement characters or unbounded decoded output.
    #[test]
    fn rejects_invalid_or_oversized_text() {
        for (bytes, encoding) in [
            (vec![0xff], Encoding::Utf8),
            (vec![0x00], Encoding::Utf16Le),
            (vec![0x00, 0xd8], Encoding::Utf16Le),
            (vec![0xdc, 0x00], Encoding::Utf16Be),
            (vec![0xfe, 0xff], Encoding::Utf16Le),
            (vec![0xff, 0xfe], Encoding::Utf16Be),
        ] {
            assert!(matches!(
                decode_body(&bytes, encoding, 8192),
                Err(Error::InvalidEncoding)
            ));
        }
        assert!(matches!(
            decode_body(b"123", Encoding::Utf8, 2),
            Err(Error::TooLarge)
        ));
        assert!(matches!(
            decode_body(&[0x00, 0x08], Encoding::Utf16Le, 2),
            Err(Error::TooLarge)
        ));
        assert!(matches!(
            Envelope::parse(XML, XML.len() - 1),
            Err(Error::TooLarge)
        ));
    }

    /// Scenario: SOAP structure, namespaces, or addressing headers are malformed or ambiguous.
    /// Guarantees: duplicate, nested, empty, and spoofed required headers and DTDs are rejected.
    #[test]
    fn rejects_ambiguous_envelopes() {
        for xml in [
            XML.replace("2003/05/soap-envelope", "wrong-soap"),
            XML.replace("2004/08/addressing", "wrong-addressing"),
            XML.replace("<a:Action>urn:test:Events</a:Action>", ""),
            XML.replace("</s:Header>", "<a:Action>duplicate</a:Action></s:Header>"),
            XML.replace(
                "</s:Header>",
                "<a:MessageID>duplicate</a:MessageID></s:Header>",
            ),
            XML.replace("uuid:AbC-123", " "),
            XML.replace("uuid:AbC-123", "<nested>value</nested>"),
            XML.replace("<s:Body>", "<s:Body/><s:Body>"),
            XML.replace("<s:Header>", "unexpected<s:Header>"),
            format!("<!DOCTYPE Envelope [<!ENTITY test 'value'>]>{XML}"),
            "<broken".into(),
        ] {
            assert!(Envelope::parse(&xml, 8192).is_err(), "accepted {xml}");
        }
        let split_text = XML.replace("uuid:AbC-123", "uuid:AbC<!--comment-->-123");
        assert_eq!(
            Envelope::parse(&split_text, 8192).unwrap().message_id(),
            "uuid:AbC-123"
        );
    }
}
