// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Structural dispatch only: no HTTP serving, subscription advertisement, or
//! delivery acceptance. Response encoding never commits progress.

use super::{ADDRESSING, Envelope, Error, Route, SOAP, has_non_whitespace_text, scalar_text};
use crate::common::xml::Node;
use quick_xml::{
    Writer,
    events::{BytesEnd, BytesStart, BytesText, Event},
};
use std::io::{self, Write};
use uuid::Uuid;

const ENUMERATION: &str = "http://schemas.xmlsoap.org/ws/2004/09/enumeration";
const EVENTING: &str = "http://schemas.xmlsoap.org/ws/2004/08/eventing";
const WSMAN: &str = "http://schemas.dmtf.org/wbem/wsman/1/wsman.xsd";
const ANONYMOUS: &str = "http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous";
const ACK: &str = "http://schemas.dmtf.org/wbem/wsman/1/wsman/Ack";
const ENUMERATE_RESPONSE: &str =
    "http://schemas.xmlsoap.org/ws/2004/09/enumeration/EnumerateResponse";
const SUBSCRIPTION_RESOURCE: &str =
    "http://schemas.microsoft.com/wbem/wsman/1/SubscriptionManager/Subscription";
const FULL_DUPLEX_RESOURCE: &str = "http://schemas.microsoft.com/wbem/wsman/1/wsman/FullDuplex";

/// Content type for uncompressed, BOM-prefixed UTF-16LE SOAP responses.
pub const SOAP_CONTENT_TYPE: &str = "application/soap+xml;charset=UTF-16";

/// Supported incoming actions, independent of delivery acceptance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    /// Request the collector's subscriptions.
    Enumerate,
    /// End a full-duplex operation; the eventual transport replies with HTTP 204.
    End,
    /// Indicate that a source subscription remains active.
    Heartbeat,
    /// Deliver events; whole-batch downstream feedback is required before Ack.
    Events,
    /// Report source-side subscription termination.
    SubscriptionEnd,
}

impl Action {
    /// Return the exact, case-sensitive protocol action URI.
    #[must_use]
    pub fn uri(self) -> &'static str {
        match self {
            Self::Enumerate => "http://schemas.xmlsoap.org/ws/2004/09/enumeration/Enumerate",
            Self::End => "http://schemas.microsoft.com/wbem/wsman/1/wsman/End",
            Self::Heartbeat => "http://schemas.dmtf.org/wbem/wsman/1/wsman/Heartbeat",
            Self::Events => "http://schemas.dmtf.org/wbem/wsman/1/wsman/Events",
            Self::SubscriptionEnd => {
                "http://schemas.xmlsoap.org/ws/2004/08/eventing/SubscriptionEnd"
            }
        }
    }
}

/// Structurally checked request, not authorization to send a success response.
///
/// Callers must validate the destination, authenticate the TLS source, and check
/// the subscription Identifier. Event payload/bookmark and termination-status
/// validation are deferred to later increments, as is downstream delivery.
pub struct Request<'envelope, 'input> {
    envelope: &'envelope Envelope<'input>,
    action: Action,
    destination: String,
    identifier: Option<String>,
    ack_requested: bool,
    max_response_bytes: usize,
}

impl<'input> Envelope<'input> {
    /// Validate route/action compatibility, payload shape, and protocol headers.
    ///
    /// Only anonymous, same-connection replies and the ultimate-receiver role
    /// are supported. Unknown optional headers are ignored; unknown mandatory
    /// headers (including robust replay and Bookmark until implemented) fail.
    /// Delivery versions remain opaque and are not compared to Identifier.
    ///
    /// # Errors
    ///
    /// Rejects unsupported actions, ambiguous or malformed headers, unsupported
    /// mandatory headers/roles, and bodies incompatible with their action.
    pub fn request<'envelope>(
        &'envelope self,
        route: &Route<'_>,
        max_response_bytes: usize,
    ) -> Result<Request<'envelope, 'input>, Error> {
        let action = [
            Action::Enumerate,
            Action::End,
            Action::Heartbeat,
            Action::Events,
            Action::SubscriptionEnd,
        ]
        .into_iter()
        .find(|action| action.uri() == self.action())
        .ok_or(Error::UnsupportedAction)?;
        if !matches!(
            (route, action),
            (Route::SubscriptionManager, Action::Enumerate | Action::End)
                | (
                    Route::Delivery { .. },
                    Action::Heartbeat | Action::Events | Action::SubscriptionEnd | Action::End
                )
        ) {
            return Err(Error::UnsupportedAction);
        }
        let body = self.body();
        let mut elements = body.children().filter(Node::is_element);
        if action == Action::End {
            if elements.next().is_some() {
                return Err(Error::InvalidEnvelope("End requires an empty Body"));
            }
        } else {
            let expected = match action {
                Action::Enumerate => (ENUMERATION, "Enumerate"),
                Action::Heartbeat | Action::Events => (WSMAN, "Events"),
                Action::SubscriptionEnd => (EVENTING, "SubscriptionEnd"),
                Action::End => unreachable!(),
            };
            let payload = elements
                .next()
                .filter(|node| node.has_tag_name(expected))
                .ok_or(Error::InvalidEnvelope("body does not match Action"))?;
            if elements.next().is_some() {
                return Err(Error::InvalidEnvelope("multiple body payloads"));
            }
            if action == Action::Heartbeat {
                require_empty(payload)?;
            } else if action == Action::Enumerate {
                validate_enumeration(payload)?;
            }
        }
        let header = self
            .document
            .root_element()
            .first_element_child()
            .expect("validated Header");
        for element in header.children().filter(Node::is_element) {
            let mandatory = match element.attribute((SOAP, "mustUnderstand")) {
                None | Some("false" | "0") => false,
                Some("true" | "1") => true,
                _ => return Err(Error::InvalidEnvelope("invalid mustUnderstand")),
            };
            if element.attribute((SOAP, "role")).is_some_and(|role| {
                role != "http://www.w3.org/2003/05/soap-envelope/role/ultimateReceiver"
            }) {
                return Err(Error::InvalidEnvelope("unsupported SOAP role"));
            }
            let known = matches!(
                (element.tag_name().namespace(), element.tag_name().name()),
                (Some(ADDRESSING), "Action" | "MessageID" | "To" | "ReplyTo")
                    | (Some(WSMAN), "MaxEnvelopeSize" | "AckRequested")
                    | (Some(EVENTING), "Identifier")
            ) || (matches!(action, Action::Enumerate | Action::End)
                && element.has_tag_name((WSMAN, "ResourceURI")));
            if mandatory && !known {
                return Err(Error::MustUnderstand(
                    element.tag_name().name().chars().take(128).collect(),
                ));
            }
        }
        let to =
            unique_child(header, ADDRESSING, "To")?.ok_or(Error::InvalidEnvelope("missing To"))?;
        let destination = scalar_text(to, "To")?;
        if let Some(reply) = unique_child(header, ADDRESSING, "ReplyTo")? {
            reject_text(reply)?;
            let address = unique_child(reply, ADDRESSING, "Address")?
                .ok_or(Error::InvalidEnvelope("missing ReplyTo Address"))?;
            if reply.children().filter(Node::is_element).count() != 1
                || scalar_text(address, "ReplyTo Address")? != ANONYMOUS
            {
                return Err(Error::InvalidEnvelope(
                    "only anonymous ReplyTo is supported",
                ));
            }
        }
        if matches!(action, Action::Enumerate | Action::End) {
            let resource = unique_child(header, WSMAN, "ResourceURI")?
                .ok_or(Error::InvalidEnvelope("missing ResourceURI"))?;
            let expected = if action == Action::Enumerate {
                SUBSCRIPTION_RESOURCE
            } else {
                FULL_DUPLEX_RESOURCE
            };
            if scalar_text(resource, "ResourceURI")? != expected {
                return Err(Error::InvalidEnvelope("unsupported ResourceURI"));
            }
        }
        let max_response_bytes = match unique_child(header, WSMAN, "MaxEnvelopeSize")? {
            Some(element) => {
                let size = scalar_text(element, "MaxEnvelopeSize")?
                    .parse::<usize>()
                    .ok()
                    .filter(|size| *size > 0)
                    .ok_or(Error::InvalidEnvelope("invalid MaxEnvelopeSize"))?;
                max_response_bytes.min(size)
            }
            None => max_response_bytes,
        };
        let identifier = unique_child(header, EVENTING, "Identifier")?
            .map(|element| scalar_text(element, "Identifier"))
            .transpose()?;
        let ack = unique_child(header, WSMAN, "AckRequested")?;
        if let Some(element) = ack {
            require_empty(element)?;
        }
        if matches!(
            action,
            Action::Heartbeat | Action::Events | Action::SubscriptionEnd
        ) && (identifier.is_none() || ack.is_none())
        {
            return Err(Error::InvalidEnvelope(
                "delivery requires Identifier and AckRequested",
            ));
        }
        Ok(Request {
            envelope: self,
            action,
            destination,
            identifier,
            ack_requested: ack.is_some(),
            max_response_bytes,
        })
    }
}

impl Request<'_, '_> {
    /// Validate To against the exact HTTP path and, when configured, the public origin.
    ///
    /// `public_endpoint` must be a validated configuration origin. Without it,
    /// only HTTPS URL shape and path are checked, not the authority.
    ///
    /// # Errors
    ///
    /// Rejects credentials, queries/fragments, non-HTTPS destinations, normalized
    /// path aliases, and mismatched paths or configured origins.
    pub fn validate_destination(
        &self,
        public_endpoint: Option<&str>,
        path: &str,
    ) -> Result<(), Error> {
        let invalid = || Error::InvalidEnvelope("To does not match collector destination");
        let destination = url::Url::parse(&self.destination).map_err(|_| invalid())?;
        let uri = self
            .destination
            .parse::<http::Uri>()
            .map_err(|_| invalid())?;
        if destination.scheme() != "https"
            || destination.host_str().is_none()
            || !destination.username().is_empty()
            || destination.password().is_some()
            || destination.query().is_some()
            || destination.fragment().is_some()
            || uri
                .authority()
                .is_none_or(|authority| authority.as_str().contains('@'))
            || uri.path() != path
            || destination.path() != path
        {
            return Err(invalid());
        }
        if let Some(endpoint) = public_endpoint {
            let expected = url::Url::parse(endpoint).map_err(|_| invalid())?;
            if destination.origin() != expected.origin() {
                return Err(invalid());
            }
        }
        Ok(())
    }

    /// Return the classified action, not an acceptance decision.
    #[must_use]
    pub fn action(&self) -> Action {
        self.action
    }

    /// Return the subscription reference property, independent of the URL version.
    #[must_use]
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    /// Return whether acknowledgement was requested.
    #[must_use]
    pub fn ack_requested(&self) -> bool {
        self.ack_requested
    }

    /// Return the smaller of the local and peer-advertised response byte limits.
    #[must_use]
    pub fn max_response_bytes(&self) -> usize {
        self.max_response_bytes
    }

    /// Return the decoded, untrimmed request MessageID for response correlation.
    #[must_use]
    pub fn message_id(&self) -> &str {
        self.envelope.message_id()
    }

    /// Encode an Ack only after the caller has accepted the request.
    ///
    /// Events require whole-batch downstream Ack and bookmark commit first.
    /// SubscriptionEnd requires termination-body validation first. This utility
    /// does neither; it only encodes bytes and must never signal success on enqueue.
    /// The caller supplies a fresh response UUID and uses [`SOAP_CONTENT_TYPE`].
    ///
    /// # Errors
    ///
    /// Rejects non-Ack actions and responses exceeding the negotiated byte bound.
    pub fn encode_ack(&self, response_id: Uuid) -> Result<Vec<u8>, Error> {
        if !self.ack_requested
            || !matches!(
                self.action,
                Action::Heartbeat | Action::Events | Action::SubscriptionEnd
            )
        {
            return Err(Error::UnsupportedAction);
        }
        self.encode_response(response_id, false)
    }

    /// Encode an empty, completed enumeration without advertising event delivery.
    ///
    /// # Errors
    ///
    /// Rejects non-Enumerate actions and responses exceeding the negotiated bound.
    pub fn encode_empty_enumeration(&self, response_id: Uuid) -> Result<Vec<u8>, Error> {
        if self.action != Action::Enumerate {
            return Err(Error::UnsupportedAction);
        }
        self.encode_response(response_id, true)
    }

    fn encode_response(&self, response_id: Uuid, enumeration: bool) -> Result<Vec<u8>, Error> {
        // UTF-8 needs at most three bytes per UTF-16 code unit. Bound the
        // intermediate representation too, then check the exact wire size.
        let mut writer = Writer::new(BoundedXml {
            bytes: Vec::new(),
            limit: self
                .max_response_bytes
                .saturating_add(self.max_response_bytes / 2),
        });
        let result = (|| -> io::Result<()> {
            let mut envelope = BytesStart::new("s:Envelope");
            envelope.push_attribute(("xmlns:s", SOAP));
            envelope.push_attribute(("xmlns:a", ADDRESSING));
            writer.write_event(Event::Start(envelope))?;
            writer.write_event(Event::Start(BytesStart::new("s:Header")))?;
            for (name, value) in [
                (
                    "a:Action",
                    if enumeration { ENUMERATE_RESPONSE } else { ACK },
                ),
                ("a:MessageID", &format!("uuid:{response_id}")),
                ("a:To", ANONYMOUS),
                ("a:RelatesTo", self.message_id()),
            ] {
                let escaped = quick_xml::escape::escape(value).replace('\r', "&#13;");
                let _ = writer
                    .create_element(name)
                    .write_text_content(BytesText::from_escaped(escaped))?;
            }
            writer.write_event(Event::End(BytesEnd::new("s:Header")))?;
            if enumeration {
                writer.write_event(Event::Start(BytesStart::new("s:Body")))?;
                let mut response = BytesStart::new("n:EnumerateResponse");
                response.push_attribute(("xmlns:n", ENUMERATION));
                response.push_attribute(("xmlns:w", WSMAN));
                writer.write_event(Event::Start(response))?;
                writer.write_event(Event::Empty(BytesStart::new("n:EnumerationContext")))?;
                writer.write_event(Event::Empty(BytesStart::new("w:Items")))?;
                writer.write_event(Event::Empty(BytesStart::new("w:EndOfSequence")))?;
                writer.write_event(Event::End(BytesEnd::new("n:EnumerateResponse")))?;
                writer.write_event(Event::End(BytesEnd::new("s:Body")))?;
            } else {
                writer.write_event(Event::Empty(BytesStart::new("s:Body")))?;
            }
            writer.write_event(Event::End(BytesEnd::new("s:Envelope")))?;
            Ok(())
        })();
        result.map_err(|error| {
            if error.kind() == io::ErrorKind::FileTooLarge {
                Error::TooLarge
            } else {
                Error::ResponseWrite(error)
            }
        })?;
        let xml_bytes = writer.into_inner().bytes;
        let xml = std::str::from_utf8(&xml_bytes).map_err(|_| Error::InvalidEncoding)?;
        let size = xml
            .encode_utf16()
            .count()
            .checked_mul(2)
            .and_then(|size| size.checked_add(2))
            .ok_or(Error::TooLarge)?;
        if size > self.max_response_bytes {
            return Err(Error::TooLarge);
        }
        let mut body = Vec::with_capacity(size);
        body.extend_from_slice(&[0xff, 0xfe]);
        for unit in xml.encode_utf16() {
            body.extend_from_slice(&unit.to_le_bytes());
        }
        Ok(body)
    }
}

struct BoundedXml {
    bytes: Vec<u8>,
    limit: usize,
}

impl Write for BoundedXml {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::from(io::ErrorKind::FileTooLarge));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn unique_child<'document, 'input>(
    parent: Node<'document, 'input>,
    namespace: &str,
    name: &'static str,
) -> Result<Option<Node<'document, 'input>>, Error> {
    let mut matches = parent
        .children()
        .filter(|node| node.has_tag_name((namespace, name)));
    let first = matches.next();
    if matches.next().is_some() {
        return Err(Error::InvalidEnvelope(name));
    }
    Ok(first)
}

fn reject_text(element: Node<'_, '_>) -> Result<(), Error> {
    if has_non_whitespace_text(element) {
        return Err(Error::InvalidEnvelope("unexpected text content"));
    }
    Ok(())
}

fn require_empty(element: Node<'_, '_>) -> Result<(), Error> {
    reject_text(element)?;
    if element.children().any(|node| node.is_element()) {
        return Err(Error::InvalidEnvelope("expected empty element"));
    }
    Ok(())
}

fn validate_enumeration(payload: Node<'_, '_>) -> Result<(), Error> {
    reject_text(payload)?;
    if payload.children().filter(Node::is_element).any(|node| {
        !node.has_tag_name((WSMAN, "OptimizeEnumeration"))
            && !node.has_tag_name((WSMAN, "MaxElements"))
    }) {
        return Err(Error::InvalidEnvelope("unsupported enumeration option"));
    }
    if let Some(optimize) = unique_child(payload, WSMAN, "OptimizeEnumeration")? {
        require_empty(optimize)?;
    }
    if let Some(max_elements) = unique_child(payload, WSMAN, "MaxElements")?
        && scalar_text(max_elements, "MaxElements")?
            .parse::<u32>()
            .ok()
            .filter(|count| *count > 0)
            .is_none()
    {
        return Err(Error::InvalidEnvelope("invalid MaxElements"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{Encoding, decode_body};
    use super::*;

    const DELIVERY_HEADERS: &str =
        "<e:Identifier>reference-property</e:Identifier><w:AckRequested/>";

    fn request_xml(action: Action, body: &str, headers: &str) -> String {
        format!(
            r#"<s:Envelope xmlns:s="{SOAP}" xmlns:a="{ADDRESSING}" xmlns:w="{WSMAN}" xmlns:e="{EVENTING}" xmlns:n="{ENUMERATION}">
<s:Header><a:Action s:mustUnderstand="true">{}</a:Action><a:MessageID>uuid:AbC-123</a:MessageID>
<a:To>https://collector.example/wsman</a:To>{headers}</s:Header><s:Body>{body}</s:Body></s:Envelope>"#,
            action.uri()
        )
    }

    fn parse_envelope(xml: &str) -> Envelope<'_> {
        Envelope::parse(xml, 16384, 128).unwrap()
    }

    /// Scenario: all supported actions arrive at manager and delivery endpoints.
    /// Guarantees: the compatibility matrix is enforced without interpreting delivery versions.
    #[test]
    fn dispatches_actions() {
        let delivery = Route::Delivery {
            version: "opaque-old-version",
        };
        for (action, body, headers) in [
            (
                Action::Enumerate,
                "<n:Enumerate/>",
                format!("<w:ResourceURI>{SUBSCRIPTION_RESOURCE}</w:ResourceURI>"),
            ),
            (
                Action::End,
                "",
                format!("<w:ResourceURI>{FULL_DUPLEX_RESOURCE}</w:ResourceURI>"),
            ),
            (Action::Heartbeat, "<w:Events/>", DELIVERY_HEADERS.into()),
            (
                Action::Events,
                "<w:Events><w:Event/></w:Events>",
                DELIVERY_HEADERS.into(),
            ),
            (
                Action::SubscriptionEnd,
                "<e:SubscriptionEnd/>",
                DELIVERY_HEADERS.into(),
            ),
        ] {
            let xml = request_xml(action, body, &headers);
            let envelope = parse_envelope(&xml);
            for route in [&Route::SubscriptionManager, &delivery] {
                let allowed = match route {
                    Route::SubscriptionManager => matches!(action, Action::Enumerate | Action::End),
                    Route::Delivery { .. } => action != Action::Enumerate,
                };
                let request = envelope.request(route, 4096);
                assert_eq!(request.is_ok(), allowed, "{action:?} {route:?}");
                if let Ok(request) = request {
                    assert_eq!(request.action(), action);
                    assert_eq!(request.message_id(), "uuid:AbC-123");
                    assert_eq!(request.ack_requested(), headers == DELIVERY_HEADERS);
                    assert_eq!(
                        request.identifier(),
                        if headers == DELIVERY_HEADERS {
                            Some("reference-property")
                        } else {
                            None
                        }
                    );
                }
            }
        }
    }

    /// Scenario: heartbeat headers/body are malformed, duplicated, or have unsupported requirements.
    /// Guarantees: invalid requests fail before response encoding, including spoofed namespaces.
    #[test]
    fn rejects_invalid_requests() {
        let valid = request_xml(Action::Heartbeat, "<w:Events/>", DELIVERY_HEADERS);
        let route = Route::Delivery { version: "v" };
        for (from, to) in [
            ("<w:Events/>", "<w:Events><w:Event/></w:Events>"),
            ("<w:Events/>", "<w:Events>text</w:Events>"),
            ("<w:Events/>", "<w:Events>&#160;</w:Events>"),
            ("<w:Events/>", "<Events/>"),
            ("<w:Events/>", "<w:Events/><w:Events/>"),
            ("<w:Events/>", ""),
            ("<w:AckRequested/>", ""),
            ("<w:AckRequested/>", "<w:AckRequested/><w:AckRequested/>"),
            ("<w:AckRequested/>", "<w:AckRequested>yes</w:AckRequested>"),
            ("reference-property", "<e:Nested/>"),
            ("<e:Identifier>reference-property</e:Identifier>", ""),
            (
                "<e:Identifier>reference-property</e:Identifier>",
                "<e:Identifier/>",
            ),
            (
                "</s:Header>",
                "<e:Identifier>other</e:Identifier></s:Header>",
            ),
            ("</s:Header>", "<a:To>https://other/</a:To></s:Header>"),
            ("<a:To>https://collector.example/wsman</a:To>", ""),
            ("</s:Header>", "<a:ReplyTo/></s:Header>"),
            (
                "</s:Header>",
                "<a:ReplyTo><a:Address>https://elsewhere/</a:Address></a:ReplyTo></s:Header>",
            ),
            (
                "</s:Header>",
                "<a:ReplyTo><a:Address>bad</a:Address><a:Address>bad</a:Address></a:ReplyTo></s:Header>",
            ),
            (
                "</s:Header>",
                "<w:Unknown s:mustUnderstand='true'/></s:Header>",
            ),
            (
                "</s:Header>",
                "<e:AckRequested s:mustUnderstand='1'/></s:Header>",
            ),
            (
                "</s:Header>",
                "<w:OperationID s:mustUnderstand='1'/></s:Header>",
            ),
            ("true", "invalid"),
            ("s:mustUnderstand=\"true\"", "s:role='urn:other'"),
            (Action::Heartbeat.uri(), "urn:unknown:Heartbeat"),
        ] {
            let xml = valid.replace(from, to);
            assert!(
                parse_envelope(&xml).request(&route, 4096).is_err(),
                "accepted {xml}"
            );
        }
    }

    /// Scenario: optional extension headers, anonymous replies, and peer response bounds are present.
    /// Guarantees: optional headers are ignored, mandatory known headers work, and limits cannot increase.
    #[test]
    fn validates_header_policy_and_limits() {
        let route = Route::Delivery { version: "v" };
        let valid = request_xml(Action::Heartbeat, "<w:Events/>", DELIVERY_HEADERS);
        for boolean in ["false", "0"] {
            let xml = valid.replace("</s:Header>", &format!(
                "<w:Unknown s:mustUnderstand='{boolean}'/><a:ReplyTo><a:Address>{ANONYMOUS}</a:Address></a:ReplyTo></s:Header>"
            ));
            assert!(parse_envelope(&xml).request(&route, 4096).is_ok());
        }
        for boolean in ["true", "1"] {
            let xml = valid.replace(
                "s:mustUnderstand=\"true\"",
                &format!("s:mustUnderstand='{boolean}' s:role='{SOAP}/role/ultimateReceiver'"),
            );
            assert!(parse_envelope(&xml).request(&route, 4096).is_ok());
        }
        for value in [
            "0",
            "-1",
            "bad",
            "999999999999999999999999999999",
            "<w:Nested/>",
        ] {
            let xml = valid.replace(
                "</s:Header>",
                &format!("<w:MaxEnvelopeSize>{value}</w:MaxEnvelopeSize></s:Header>"),
            );
            assert!(parse_envelope(&xml).request(&route, 4096).is_err());
        }
        let xml = valid.replace(
            "</s:Header>",
            "<w:MaxEnvelopeSize>2048</w:MaxEnvelopeSize></s:Header>",
        );
        let envelope = parse_envelope(&xml);
        assert_eq!(
            envelope.request(&route, 4096).unwrap().max_response_bytes(),
            2048
        );
        assert_eq!(
            envelope.request(&route, 1024).unwrap().max_response_bytes(),
            1024
        );
        let long_name = "x".repeat(256);
        let xml = valid.replace(
            "</s:Header>",
            &format!("<w:{long_name} s:mustUnderstand='true'/></s:Header>"),
        );
        assert!(matches!(parse_envelope(&xml).request(&route, 4096),
            Err(Error::MustUnderstand(name)) if name.len() == 128));
    }

    /// Scenario: To uses alternate origins, credentials, query strings, or normalized path aliases.
    /// Guarantees: HTTPS and exact raw paths are required; configured origins compare semantically.
    #[test]
    fn validates_destination() {
        let path = "/wsman/subscriptions/subscription/old%2Fopaque";
        let route = Route::Delivery {
            version: "old%2Fopaque",
        };
        let xml = request_xml(Action::Heartbeat, "<w:Events/>", DELIVERY_HEADERS);
        let envelope = parse_envelope(&xml);
        let mut request = envelope.request(&route, 4096).unwrap();
        for origin in ["https://collector.example", "https://COLLECTOR.example:443"] {
            request.destination = format!("{origin}{path}");
            request
                .validate_destination(Some("https://collector.example/"), path)
                .unwrap();
            request.validate_destination(None, path).unwrap();
        }
        for destination in [
            format!("http://collector.example{path}"),
            format!("https://other.example{path}"),
            format!("https://collector.example:5986{path}"),
            format!("https://user@collector.example{path}"),
            format!("https://@collector.example{path}"),
            format!("https://collector.example{path}?query"),
            format!("https://collector.example{path}#fragment"),
            format!("https://collector.example/ignored/..{path}"),
            format!("https://collector.example{path}").replace("%2F", "%2f"),
            format!("https://collector.example{path}/"),
            path.to_owned(),
            "not a URL".into(),
        ] {
            request.destination = destination;
            assert!(
                request
                    .validate_destination(Some("https://collector.example"), path)
                    .is_err(),
                "{}",
                request.destination
            );
        }
        request.destination = format!("https://elsewhere{path}");
        assert!(request.validate_destination(None, path).is_ok());
    }

    /// Scenario: Enumerate and End contain missing resources or unsupported enumeration options.
    /// Guarantees: unsupported filtering/paging options are rejected instead of silently acknowledged.
    #[test]
    fn validates_manager_requests() {
        let headers = format!("<w:ResourceURI>{SUBSCRIPTION_RESOURCE}</w:ResourceURI>");
        for body in [
            "<n:Enumerate/>",
            "<n:Enumerate><w:OptimizeEnumeration/><w:MaxElements>1</w:MaxElements></n:Enumerate>",
        ] {
            let xml = request_xml(Action::Enumerate, body, &headers);
            assert!(
                parse_envelope(&xml)
                    .request(&Route::SubscriptionManager, 4096)
                    .is_ok()
            );
        }
        for option in [
            "text",
            "&#160;",
            "<n:Filter/>",
            "<w:MaxElements>0</w:MaxElements>",
            "<w:MaxElements>4294967296</w:MaxElements>",
            "<w:MaxElements>1</w:MaxElements><w:MaxElements>2</w:MaxElements>",
            "<w:OptimizeEnumeration>yes</w:OptimizeEnumeration>",
            "<w:OptimizeEnumeration/><w:OptimizeEnumeration/>",
        ] {
            let xml = request_xml(
                Action::Enumerate,
                &format!("<n:Enumerate>{option}</n:Enumerate>"),
                &headers,
            );
            assert!(
                parse_envelope(&xml)
                    .request(&Route::SubscriptionManager, 4096)
                    .is_err(),
                "{option}"
            );
        }
        for headers in [
            "".into(),
            "<w:ResourceURI>wrong</w:ResourceURI>".into(),
            format!("{headers}{headers}"),
        ] {
            let xml = request_xml(Action::Enumerate, "<n:Enumerate/>", &headers);
            assert!(
                parse_envelope(&xml)
                    .request(&Route::SubscriptionManager, 4096)
                    .is_err()
            );
        }
        let xml = request_xml(
            Action::End,
            "<w:Events/>",
            &format!("<w:ResourceURI>{FULL_DUPLEX_RESOURCE}</w:ResourceURI>"),
        );
        assert!(
            parse_envelope(&xml)
                .request(&Route::SubscriptionManager, 4096)
                .is_err()
        );
    }

    /// Scenario: Ack and empty enumeration include escaped Unicode/CR correlation values at byte limits.
    /// Guarantees: exact UTF-16 wire bounds include BOM, decoded correlation is preserved, and no subscriptions leak.
    #[test]
    fn encodes_bounded_responses() {
        for (action, body, headers, route) in [
            (
                Action::Heartbeat,
                "<w:Events/>",
                DELIVERY_HEADERS.into(),
                Route::Delivery { version: "v" },
            ),
            (
                Action::Enumerate,
                "<n:Enumerate/>",
                format!("<w:ResourceURI>{SUBSCRIPTION_RESOURCE}</w:ResourceURI>"),
                Route::SubscriptionManager,
            ),
        ] {
            let xml = request_xml(action, body, &headers).replace(
                "uuid:AbC-123",
                " uuid:AbC&amp;&lt;Tag&gt;&#13;\u{00e9}\u{1f600} ",
            );
            let envelope = parse_envelope(&xml);
            let response_id = Uuid::from_u128(123);
            let encode = |limit| {
                let request = envelope.request(&route, limit).unwrap();
                if action == Action::Enumerate {
                    request.encode_empty_enumeration(response_id)
                } else {
                    request.encode_ack(response_id)
                }
            };
            let bytes = encode(8192).unwrap();
            assert!(bytes.starts_with(&[0xff, 0xfe]));
            assert_eq!(SOAP_CONTENT_TYPE, "application/soap+xml;charset=UTF-16");
            let decoded = decode_body(&bytes, Encoding::Utf16Le, 16384).unwrap();
            let response = parse_envelope(&decoded);
            assert_eq!(
                response.action(),
                if action == Action::Enumerate {
                    ENUMERATE_RESPONSE
                } else {
                    ACK
                }
            );
            assert_eq!(response.message_id(), format!("uuid:{response_id}"));
            let header = response
                .document
                .root_element()
                .first_element_child()
                .unwrap();
            let relates = unique_child(header, ADDRESSING, "RelatesTo")
                .unwrap()
                .unwrap();
            assert_eq!(
                scalar_text(relates, "RelatesTo").unwrap(),
                envelope.message_id()
            );
            assert_eq!(header.children().filter(Node::is_element).count(), 4);
            if action == Action::Enumerate {
                let payload = response.body().first_element_child().unwrap();
                assert!(payload.has_tag_name((ENUMERATION, "EnumerateResponse")));
                assert_eq!(payload.children().filter(Node::is_element).count(), 3);
                for (ns, name) in [
                    (ENUMERATION, "EnumerationContext"),
                    (WSMAN, "Items"),
                    (WSMAN, "EndOfSequence"),
                ] {
                    require_empty(unique_child(payload, ns, name).unwrap().unwrap()).unwrap();
                }
                assert!(matches!(
                    envelope
                        .request(&route, 8192)
                        .unwrap()
                        .encode_ack(response_id),
                    Err(Error::UnsupportedAction)
                ));
            } else {
                require_empty(response.body()).unwrap();
                assert!(matches!(
                    envelope
                        .request(&route, 8192)
                        .unwrap()
                        .encode_empty_enumeration(response_id),
                    Err(Error::UnsupportedAction)
                ));
            }
            assert_eq!(encode(bytes.len()).unwrap(), bytes);
            for limit in [0, 32, bytes.len() - 1] {
                assert!(
                    matches!(encode(limit), Err(Error::TooLarge)),
                    "{action:?} limit {limit}"
                );
            }
        }
    }
}
