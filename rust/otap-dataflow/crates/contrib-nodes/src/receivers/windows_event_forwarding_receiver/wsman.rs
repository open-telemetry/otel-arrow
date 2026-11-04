// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::common::xml::{Document, Node};
use uuid::Uuid;

const SOAP: &str = "http://www.w3.org/2003/05/soap-envelope";
const ADDRESSING: &str = "http://schemas.xmlsoap.org/ws/2004/08/addressing";

/// Request validation and bounded basic response encoding.
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
    /// XML is malformed or violates the shared XML parser's policy or limits.
    #[error("invalid SOAP XML: {0}")]
    InvalidXml(#[from] crate::common::xml::Error),
    /// SOAP structure or required addressing headers are invalid.
    #[error("invalid SOAP envelope: {0}")]
    InvalidEnvelope(&'static str),
    /// The action URI is unknown or not supported at this endpoint.
    #[error("unsupported WS-Man action for this endpoint")]
    UnsupportedAction,
    /// An unsupported mandatory header was encountered; the local name is bounded.
    #[error("unsupported mandatory SOAP header: {0}")]
    MustUnderstand(String),
    /// Response serialization failed.
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
    ///
    /// # Errors
    ///
    /// Returns `UnknownRoute` for unmatched paths, another subscription, or a
    /// missing version or extra path segment.
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
/// Both input bytes (including a BOM) and decoded UTF-8 bytes are bounded.
/// The HTTP layer must also bound wire bytes and decompression before invoking
/// this function. This does not parse XML or infer encoding from its declaration.
///
/// # Errors
///
/// Returns `TooLarge` when either byte limit is exceeded, or `InvalidEncoding`
/// for malformed Unicode, odd UTF-16 byte counts, or a mismatched UTF-16 BOM.
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
    /// `max_bytes` bounds input size independently of `max_nodes`, which bounds
    /// arena entries (elements, text nodes, and comment/PI placeholders).
    /// Header blocks must be namespace-qualified. Envelope, Header, and Body
    /// may contain only XML whitespace between their child elements.
    ///
    /// DTDs are disabled. Required WS-Addressing Action and MessageID headers
    /// must be unique, nonempty scalar elements. Action-specific body and
    /// mustUnderstand checks are left to [`Self::request`]; this parser
    /// alone does not authorize an acknowledgement.
    ///
    /// # Errors
    ///
    /// Rejects oversized input, invalid XML, unexpected envelope structure, and
    /// missing, duplicate, nested, or empty required addressing headers.
    pub fn parse(xml: &'input str, max_bytes: usize, max_nodes: usize) -> Result<Self, Error> {
        if xml.len() > max_bytes {
            return Err(Error::TooLarge);
        }
        let document = Document::parse_bounded(xml, max_nodes)?;
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
            let body = elements
                .next()
                .filter(|node| node.has_tag_name((SOAP, "Body")))
                .ok_or(Error::InvalidEnvelope("expected Body"))?;
            if elements.next().is_some() || has_non_whitespace_text(root) {
                return Err(Error::InvalidEnvelope("unexpected Envelope content"));
            }
            if has_non_whitespace_text(body) {
                return Err(Error::InvalidEnvelope("unexpected Body content"));
            }
            header
        };
        if has_non_whitespace_text(header) {
            return Err(Error::InvalidEnvelope("unexpected Header content"));
        }
        if header
            .children()
            .filter(Node::is_element)
            .any(|node| node.tag_name().namespace().is_none())
        {
            return Err(Error::InvalidEnvelope("unqualified Header block"));
        }
        let action = addressing_header(header, "Action")?;
        let message_id = addressing_header(header, "MessageID")?;
        Ok(Self {
            document,
            action,
            message_id,
        })
    }

    /// Return the decoded action value without case folding or trimming.
    #[must_use]
    pub fn action(&self) -> &str {
        &self.action
    }

    /// Return the decoded MessageID without trimming, for exact response correlation.
    #[must_use]
    pub fn message_id(&self) -> &str {
        &self.message_id
    }

    /// Return the structurally validated Body element for action-specific parsing.
    #[must_use]
    pub fn body(&self) -> Node<'_, 'input> {
        self.document
            .root_element()
            .children()
            .find(|node| node.has_tag_name((SOAP, "Body")))
            .expect("Body validated during parsing")
    }
}

fn has_non_whitespace_text(element: Node<'_, '_>) -> bool {
    element.children().any(|node| {
        node.is_text()
            && node
                .text()
                .is_some_and(|text| !text.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')))
    })
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

    const MAX_TEST_NODES: usize = 64;

    const XML: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing">
<s:Header><a:Action>urn:test:Events</a:Action><a:MessageID>uuid:AbC-123</a:MessageID></s:Header>
<s:Body><Events><![CDATA[<Event>example</Event>]]></Events></s:Body></s:Envelope>"#;

    fn utf16(input: &str, little_endian: bool) -> Vec<u8> {
        input
            .encode_utf16()
            .flat_map(|unit| {
                if little_endian {
                    unit.to_le_bytes()
                } else {
                    unit.to_be_bytes()
                }
            })
            .collect()
    }

    /// Scenario: deliveries use old, unknown, or encoded versions and incorrect routes.
    /// Guarantees: only the configured subscription is routed; version text stays opaque.
    #[test]
    fn routes_preserve_opaque_versions() {
        let subscription = Uuid::from_u128(0xabcdef);
        assert_eq!(
            Route::parse("/wsman/SubscriptionManager/WEC", subscription).unwrap(),
            Route::SubscriptionManager
        );
        for version in ["old", "Unknown-VERSION", "not-a-uuid", "%41", "%2F"] {
            let path = format!("/wsman/subscriptions/{subscription}/{version}");
            assert_eq!(
                Route::parse(&path, subscription).unwrap(),
                Route::Delivery { version }
            );
        }
        let uppercase = format!("/wsman/subscriptions/{subscription}/v")
            .to_uppercase()
            .replace("/WSMAN/SUBSCRIPTIONS/", "/wsman/subscriptions/");
        assert_eq!(
            Route::parse(&uppercase, subscription).unwrap(),
            Route::Delivery { version: "V" }
        );
        for path in [
            format!("/wsman/subscriptions/{subscription}"),
            format!("/wsman/subscriptions/{subscription}/"),
            format!("/wsman/subscriptions/{subscription}/version/extra"),
            format!("/wsman/subscriptions/{}/version", Uuid::from_u128(2)),
            "/wsman/subscriptions/invalid/version".into(),
            "/wsman/SubscriptionManager/WEC/".into(),
            "/wsman/subscriptionmanager/WEC".into(),
            "/unknown".into(),
        ] {
            assert!(
                matches!(Route::parse(&path, subscription), Err(Error::UnknownRoute)),
                "{path}"
            );
        }
    }

    /// Scenario: SOAP arrives as UTF-8 or either UTF-16 byte order, with or without a BOM.
    /// Guarantees: decoding preserves Unicode, MessageID case, namespaces, and CDATA content.
    #[test]
    fn decodes_and_parses_envelopes() {
        let xml = XML.replace("example", "\u{1f600}\u{00e9}");
        for encoding in [Encoding::Utf8, Encoding::Utf16Le, Encoding::Utf16Be] {
            for bom in [false, true] {
                let input = if bom {
                    format!("\u{feff}{xml}")
                } else {
                    xml.clone()
                };
                let bytes = match encoding {
                    Encoding::Utf8 => input.into_bytes(),
                    Encoding::Utf16Le => utf16(&input, true),
                    Encoding::Utf16Be => utf16(&input, false),
                };
                let decoded = decode_body(&bytes, encoding, bytes.len()).unwrap();
                assert_eq!(decoded, xml);
                let envelope = Envelope::parse(&decoded, decoded.len(), MAX_TEST_NODES).unwrap();
                assert_eq!(envelope.action(), "urn:test:Events");
                assert_eq!(envelope.message_id(), "uuid:AbC-123");
                assert_eq!(
                    envelope.body().first_element_child().unwrap().text(),
                    Some("<Event>\u{1f600}\u{00e9}</Event>")
                );
            }
        }
    }

    /// Scenario: wire bytes or decoded UTF-8 reach and exceed their exact limits.
    /// Guarantees: input BOM bytes count, UTF-16 expansion is bounded, and equality is accepted.
    #[test]
    fn enforces_exact_byte_limits() {
        for (bytes, encoding, decoded) in [
            (b"123".to_vec(), Encoding::Utf8, "123"),
            (b"\xef\xbb\xbf123".to_vec(), Encoding::Utf8, "123"),
            (utf16("123", true), Encoding::Utf16Le, "123"),
            (utf16("123", false), Encoding::Utf16Be, "123"),
        ] {
            assert_eq!(decode_body(&bytes, encoding, bytes.len()).unwrap(), decoded);
            assert!(matches!(
                decode_body(&bytes, encoding, bytes.len() - 1),
                Err(Error::TooLarge)
            ));
        }
        for (encoding, little_endian) in [(Encoding::Utf16Le, true), (Encoding::Utf16Be, false)] {
            let bytes = utf16("\u{0800}\u{0800}", little_endian);
            assert_eq!(
                decode_body(&bytes, encoding, 6).unwrap(),
                "\u{0800}\u{0800}"
            );
            assert!(matches!(
                decode_body(&bytes, encoding, 5),
                Err(Error::TooLarge)
            ));
            assert_eq!(decode_body(&[], encoding, 0).unwrap(), "");
        }
        assert_eq!(decode_body(&[], Encoding::Utf8, 0).unwrap(), "");
        assert!(Envelope::parse(XML, XML.len(), MAX_TEST_NODES).is_ok());
        assert!(matches!(
            Envelope::parse(XML, XML.len() - 1, MAX_TEST_NODES),
            Err(Error::TooLarge)
        ));
        assert!(Envelope::parse("", 0, MAX_TEST_NODES).is_err());
    }

    /// Scenario: text contains invalid UTF-8, odd UTF-16, unpaired surrogates, or opposite BOMs.
    /// Guarantees: invalid encodings fail explicitly rather than producing replacement characters.
    #[test]
    fn rejects_invalid_text() {
        for (bytes, encoding) in [
            (vec![0xff], Encoding::Utf8),
            (vec![0xc0, 0xaf], Encoding::Utf8),
            (vec![0xe2, 0x82], Encoding::Utf8),
            (vec![0x00], Encoding::Utf16Le),
            (vec![0x00], Encoding::Utf16Be),
            (vec![0x00, 0xd8], Encoding::Utf16Le),
            (vec![0xd8, 0x00], Encoding::Utf16Be),
            (vec![0x00, 0xdc], Encoding::Utf16Le),
            (vec![0xdc, 0x00], Encoding::Utf16Be),
            (vec![0xfe, 0xff], Encoding::Utf16Le),
            (vec![0xff, 0xfe], Encoding::Utf16Be),
        ] {
            assert!(
                matches!(
                    decode_body(&bytes, encoding, 8192),
                    Err(Error::InvalidEncoding)
                ),
                "{bytes:?} {encoding:?}"
            );
        }
    }

    /// Scenario: SOAP structure, namespaces, or required addressing headers are ambiguous.
    /// Guarantees: missing, duplicate, nested, empty, spoofed, or reordered structural fields fail.
    #[test]
    fn rejects_ambiguous_envelopes() {
        for xml in [
            XML.replace("2003/05/soap-envelope", "wrong-soap"),
            XML.replace("2004/08/addressing", "wrong-addressing"),
            XML.replace("<s:Header>", "<s:Header/><s:Header>"),
            XML.replace("<s:Header>", "<s:Body/><s:Header>"),
            XML.replace("</s:Envelope>", "<extra/></s:Envelope>"),
            XML.replace("<s:Body>", "<s:Body/><s:Body>"),
            XML.replace("<s:Header>", "unexpected<s:Header>"),
            XML.replace("<s:Header>", "\u{00a0}<s:Header>"),
            XML.replace("<s:Header>", "<s:Header>unexpected"),
            XML.replace("<s:Header>", "<s:Header>\u{00a0}"),
            format!("<s:Envelope xmlns:s='{SOAP}'/>"),
            format!("<s:Envelope xmlns:s='{SOAP}'><s:Header/></s:Envelope>"),
        ] {
            assert!(
                matches!(
                    Envelope::parse(&xml, xml.len(), MAX_TEST_NODES),
                    Err(Error::InvalidEnvelope(_))
                ),
                "accepted {xml}"
            );
        }
        for (name, value) in [("Action", "urn:test:Events"), ("MessageID", "uuid:AbC-123")] {
            let field = format!("<a:{name}>{value}</a:{name}>");
            for replacement in [
                String::new(),
                format!("{field}{field}"),
                format!("<a:{name}/>"),
                format!("<a:{name}> \t\n</a:{name}>"),
                format!("<a:{name}><nested>value</nested></a:{name}>"),
                format!("<{name}>{value}</{name}>"),
            ] {
                let xml = XML.replace(&field, &replacement);
                assert!(
                    matches!(
                        Envelope::parse(&xml, xml.len(), MAX_TEST_NODES),
                        Err(Error::InvalidEnvelope(_))
                    ),
                    "accepted {xml}"
                );
            }
        }
    }

    /// Scenario: prefixes change and scalar fields contain references, CDATA, and comments.
    /// Guarantees: expanded names drive matching and decoded correlation text is not trimmed.
    #[test]
    fn preserves_decoded_addressing_values() {
        let xml = XML
            .replace("<s:", "<soap:")
            .replace("</s:", "</soap:")
            .replace("xmlns:s=", "xmlns:soap=")
            .replace("<a:", "<address:")
            .replace("</a:", "</address:")
            .replace("xmlns:a=", "xmlns:address=")
            .replace(
                "uuid:AbC-123",
                " uuid:AbC<!--comment--><![CDATA[-]]>&#49;23 ",
            )
            .replace("urn:test:Events", " urn:test:Events ");
        let envelope = Envelope::parse(&xml, xml.len(), MAX_TEST_NODES).unwrap();
        assert_eq!(envelope.action(), " urn:test:Events ");
        assert_eq!(envelope.message_id(), " uuid:AbC-123 ");
        let default_namespace = XML
            .replace("xmlns:s=", "xmlns=")
            .replace("<s:", "<")
            .replace("</s:", "</");
        assert!(
            Envelope::parse(&default_namespace, default_namespace.len(), MAX_TEST_NODES).is_ok()
        );
    }

    /// Scenario: input contains malformed XML, a DTD, unknown entities, or multiple roots.
    /// Guarantees: shared parser policy errors propagate as InvalidXml before dispatch.
    #[test]
    fn rejects_invalid_xml() {
        for xml in [
            format!("<!DOCTYPE Envelope [<!ENTITY test 'value'>]>{XML}"),
            XML.replace("uuid:AbC-123", "&unknown;"),
            XML.replace("uuid:AbC-123", "&#0;"),
            format!("{XML}{XML}"),
            "<broken".into(),
        ] {
            assert!(
                matches!(
                    Envelope::parse(&xml, xml.len(), MAX_TEST_NODES),
                    Err(Error::InvalidXml(_))
                ),
                "accepted {xml}"
            );
        }
    }

    /// Scenario: a valid envelope contains direct Body text, including CDATA or character references.
    /// Guarantees: only XML whitespace is allowed between Body elements; payload text stays valid.
    #[test]
    fn rejects_direct_body_text() {
        for content in ["unexpected", "<![CDATA[unexpected]]>", "&#160;"] {
            let xml = XML.replace("<s:Body>", &format!("<s:Body>{content}"));
            assert!(matches!(
                Envelope::parse(&xml, xml.len(), MAX_TEST_NODES),
                Err(Error::InvalidEnvelope("unexpected Body content"))
            ));
        }
        let xml = XML.replace("<s:Body>", "<s:Body> \t\r\n<!--comment-->");
        let envelope = Envelope::parse(&xml, xml.len(), MAX_TEST_NODES).unwrap();
        assert_eq!(
            envelope.body().first_element_child().unwrap().text(),
            Some("<Event>example</Event>")
        );
    }

    /// Scenario: an extra header block is unqualified despite valid Action and MessageID fields.
    /// Guarantees: unqualified blocks fail, while prefixed and default-namespace blocks remain valid.
    #[test]
    fn requires_qualified_header_blocks() {
        for block in ["<Extra/>", "<Extra xmlns=''/>"] {
            let xml = XML.replace("</s:Header>", &format!("{block}</s:Header>"));
            assert!(matches!(
                Envelope::parse(&xml, xml.len(), MAX_TEST_NODES),
                Err(Error::InvalidEnvelope("unqualified Header block"))
            ));
        }
        for block in [
            "<e:Extra xmlns:e='urn:extension'/>",
            "<Extra xmlns='urn:extension'/>",
        ] {
            let xml = XML.replace("</s:Header>", &format!("{block}</s:Header>"));
            assert!(Envelope::parse(&xml, xml.len(), MAX_TEST_NODES).is_ok());
        }
    }

    /// Scenario: comment padding exhausts the node budget while input remains below the byte limit.
    /// Guarantees: node limits are independent, accept equality, and reject one extra node or zero capacity.
    #[test]
    fn enforces_independent_node_limit() {
        // Five elements and two scalar text nodes, with no inter-element whitespace.
        let base = format!(
            "<s:Envelope xmlns:s='{SOAP}' xmlns:a='{ADDRESSING}'><s:Header>\
                 <a:Action>urn:test</a:Action><a:MessageID>id</a:MessageID>\
                 </s:Header><s:Body/></s:Envelope>"
        );
        let max_nodes = 10;
        let max_bytes = 1024;
        assert!(Envelope::parse(&base, max_bytes, 7).is_ok());
        assert!(matches!(
            Envelope::parse(&base, max_bytes, 0),
            Err(Error::InvalidXml(crate::common::xml::Error::Invalid(
                "node limit exceeded"
            )))
        ));
        let at_limit = base.replace("<s:Body/>", "<s:Body><!--1--><!--2--><!--3--></s:Body>");
        assert!(at_limit.len() < max_bytes);
        assert!(Envelope::parse(&at_limit, max_bytes, max_nodes).is_ok());
        let over_limit = at_limit.replace("</s:Body>", "<!--4--></s:Body>");
        assert!(over_limit.len() < max_bytes);
        assert!(matches!(
            Envelope::parse(&over_limit, max_bytes, max_nodes),
            Err(Error::InvalidXml(crate::common::xml::Error::Invalid(
                "node limit exceeded"
            )))
        ));
        assert!(Envelope::parse(&over_limit, max_bytes, max_nodes + 1).is_ok());
    }
}
