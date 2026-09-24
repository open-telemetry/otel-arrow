// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::xml::Node;
use quick_xml::{
    Writer,
    events::{BytesEnd, BytesStart, BytesText, Event},
};
use std::io::{self, Write};
use uuid::Uuid;

use super::{ADDRESSING, Envelope, Error, Route, SOAP, scalar_text};

const ENUMERATION: &str = "http://schemas.xmlsoap.org/ws/2004/09/enumeration";
const EVENTING: &str = "http://schemas.xmlsoap.org/ws/2004/08/eventing";
const WSMAN: &str = "http://schemas.dmtf.org/wbem/wsman/1/wsman.xsd";
const WSMAN_FAULT: &str = "http://schemas.microsoft.com/wbem/wsman/1/wsmanfault";
const ANONYMOUS: &str = "http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous";
const ACK: &str = "http://schemas.dmtf.org/wbem/wsman/1/wsman/Ack";

/// Bounded extraction of rendered event batches and opaque bookmarks.
pub mod delivery;
/// Configured subscription advertised inside an Enumerate response.
pub mod subscription;
use subscription::Advertisement;

/// Content type of uncompressed, BOM-prefixed UTF-16 acknowledgement responses.
pub const SOAP_CONTENT_TYPE: &str = "application/soap+xml;charset=UTF-16";

/// Supported incoming actions; dispatch never acknowledges an event batch itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    /// Request the collector's configured subscription.
    Enumerate,
    /// End a full-duplex operation; the transport replies with HTTP 204.
    End,
    /// Indicate that a source subscription remains active.
    Heartbeat,
    /// Deliver events requiring whole-batch downstream feedback before Ack.
    Events,
    /// Report source-side subscription termination.
    SubscriptionEnd,
}

impl Action {
    /// Return the exact protocol action URI.
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

/// Source-reported termination category, without unbounded diagnostic labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubscriptionEndStatus {
    /// The source could not deliver notifications.
    DeliveryFailure,
    /// The source is shutting down normally.
    SourceShuttingDown,
    /// The source cancelled the subscription for another reason.
    SourceCancelling,
    /// An extension status URI not defined by WS-Eventing.
    Other,
}

/// Structurally checked request, still requiring action-specific business processing.
///
/// Enumeration options, event records, SubscriptionEnd status, and subscription
/// reference properties must be processed before sending any success response.
pub struct Request<'envelope, 'input> {
    envelope: &'envelope Envelope<'input>,
    action: Action,
    destination: String,
    identifier: Option<String>,
    ack_requested: bool,
    max_response_bytes: usize,
}

impl<'input> Envelope<'input> {
    /// Check endpoint/action compatibility, body shape, and supported mandatory headers.
    ///
    /// Only anonymous, same-connection replies are supported. Optional robust-connection
    /// headers are ignored; mandatory replay requests fail until response caching exists.
    /// This does not compare delivery versions or authenticate SOAP identity claims.
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
        reject_text(body)?;
        let mut elements = body.children().filter(Node::is_element);
        match action {
            Action::End => {
                if elements.next().is_some() {
                    return Err(Error::InvalidEnvelope("End requires an empty Body"));
                }
            }
            _ => {
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
                }
            }
        }
        let header = self
            .document
            .root_element()
            .first_element_child()
            .expect("Header validated");
        reject_text(header)?;
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
                && element.has_tag_name((WSMAN, "ResourceURI")))
                || (action == Action::Events && element.has_tag_name((WSMAN, "Bookmark")));
            if mandatory && !known {
                return Err(Error::MustUnderstand(element.tag_name().name().to_owned()));
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
                "http://schemas.microsoft.com/wbem/wsman/1/SubscriptionManager/Subscription"
            } else {
                "http://schemas.microsoft.com/wbem/wsman/1/wsman/FullDuplex"
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
    /// Validate a bounded termination notification without changing delivery state.
    /// SubscriptionManager is descriptive only: its address is never contacted.
    pub fn subscription_end(&self) -> Result<SubscriptionEndStatus, Error> {
        if self.action != Action::SubscriptionEnd {
            return Err(Error::UnsupportedAction);
        }
        let payload = self
            .envelope
            .body()
            .first_element_child()
            .expect("validated payload");
        reject_text(payload)?;
        if let Some(node) = payload.children().filter(Node::is_element).find(|node| {
            !node.has_tag_name((EVENTING, "SubscriptionManager"))
                && !node.has_tag_name((EVENTING, "Status"))
                && !node.has_tag_name((EVENTING, "Reason"))
                && !node.has_tag_name((WSMAN_FAULT, "WSManFault"))
        }) {
            return Err(Error::UnsupportedSubscriptionEndElement {
                namespace: node
                    .tag_name()
                    .namespace()
                    .unwrap_or_default()
                    .chars()
                    .take(128)
                    .collect(),
                name: node.tag_name().name().chars().take(128).collect(),
            });
        }
        let manager = unique_child(payload, EVENTING, "SubscriptionManager")?
            .ok_or(Error::InvalidEnvelope("missing SubscriptionManager"))?;
        reject_text(manager)?;
        let address = unique_child(manager, ADDRESSING, "Address")?.ok_or(
            Error::InvalidEnvelope("missing SubscriptionManager Address"),
        )?;
        let address = scalar_text(address, "SubscriptionManager Address")?;
        validate_absolute_uri(&address)?;
        for name in ["ReferenceProperties", "ReferenceParameters"] {
            if let Some(properties) = unique_child(manager, ADDRESSING, name)? {
                reject_text(properties)?;
            }
        }
        let status = unique_child(payload, EVENTING, "Status")?
            .ok_or(Error::InvalidEnvelope("missing SubscriptionEnd Status"))?;
        let status = scalar_text(status, "SubscriptionEnd Status")?;
        validate_absolute_uri(&status)?;
        if let Some(fault) = unique_child(payload, WSMAN_FAULT, "WSManFault")? {
            reject_text(fault)?;
            if fault
                .attribute("Code")
                .and_then(|code| code.trim().parse::<u32>().ok())
                .is_none()
                || fault.attribute("Machine").is_none()
                || fault
                    .children()
                    .filter(Node::is_element)
                    .any(|node| !node.has_tag_name((WSMAN_FAULT, "Message")))
            {
                return Err(Error::InvalidEnvelope("invalid SubscriptionEnd WSManFault"));
            }
            if let Some(message) = unique_child(fault, WSMAN_FAULT, "Message")? {
                if message
                    .children()
                    .filter(Node::is_element)
                    .any(|node| !node.has_tag_name((WSMAN_FAULT, "ProviderFault")))
                {
                    return Err(Error::InvalidEnvelope("invalid WSManFault Message"));
                }
                let _ = unique_child(message, WSMAN_FAULT, "ProviderFault")?;
            }
        }
        if let Some(reason) = unique_child(payload, EVENTING, "Reason")? {
            if reason.children().any(|node| node.is_element()) {
                return Err(Error::InvalidEnvelope("nested SubscriptionEnd Reason"));
            }
            if reason
                .attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
                .is_some_and(|language| language.trim().is_empty())
            {
                return Err(Error::InvalidEnvelope(
                    "empty SubscriptionEnd Reason language",
                ));
            }
        }
        Ok(match status.strip_prefix(EVENTING) {
            Some("/DeliveryFailure") => SubscriptionEndStatus::DeliveryFailure,
            Some("/SourceShuttingDown") => SubscriptionEndStatus::SourceShuttingDown,
            Some("/SourceCancelling") => SubscriptionEndStatus::SourceCancelling,
            _ => SubscriptionEndStatus::Other,
        })
    }

    /// Validate the HTTPS destination against the configured origin and exact HTTP route.
    /// Without a public origin, only URL shape and route can be checked.
    pub fn validate_destination(
        &self,
        public_endpoint: Option<&str>,
        path: &str,
    ) -> Result<(), Error> {
        let invalid = || Error::InvalidEnvelope("To does not match collector destination");
        let destination = url::Url::parse(&self.destination).map_err(|_| invalid())?;
        let uri = self
            .destination
            .parse::<hyper::Uri>()
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

    /// Return the classified action; Events still requires downstream acceptance.
    #[must_use]
    pub fn action(&self) -> Action {
        self.action
    }

    /// Return reference-property identity separately from the opaque delivery URL version.
    #[must_use]
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    /// Return whether the source requested acknowledgement.
    #[must_use]
    pub fn ack_requested(&self) -> bool {
        self.ack_requested
    }

    /// Return the smaller of the local and source-advertised response limits.
    #[must_use]
    pub fn max_response_bytes(&self) -> usize {
        self.max_response_bytes
    }

    /// Preserve the request MessageID verbatim for response correlation.
    #[must_use]
    pub fn message_id(&self) -> &str {
        self.envelope.message_id()
    }

    /// Encode an Ack after the caller has accepted the request, never on enqueue alone.
    ///
    /// For Events, call only after whole-batch downstream Ack and bookmark commit.
    /// This encoder does not verify delivery or mutate progress. The caller supplies
    /// a fresh response UUID and sends the result with SOAP_CONTENT_TYPE. No robust-
    /// connection headers are advertised until response replay is implemented.
    pub fn encode_ack(&self, response_id: Uuid) -> Result<Vec<u8>, Error> {
        if !self.ack_requested
            || !matches!(
                self.action,
                Action::Heartbeat | Action::Events | Action::SubscriptionEnd
            )
        {
            return Err(Error::UnsupportedAction);
        }
        self.encode_response(response_id, ACK, false, None)
    }

    /// Return no subscriptions while testing the authenticated manager endpoint.
    ///
    /// This deliberately does not advertise event delivery before its runtime exists.
    pub fn encode_empty_enumeration(&self, response_id: Uuid) -> Result<Vec<u8>, Error> {
        self.encode_enumeration(response_id, None)
    }

    /// Encode a subscription with its verified peer's issuer thumbprint, or an empty list.
    pub fn encode_enumeration(
        &self,
        response_id: Uuid,
        subscription: Option<(&Advertisement, &[u8; 20])>,
    ) -> Result<Vec<u8>, Error> {
        if self.action != Action::Enumerate {
            return Err(Error::UnsupportedAction);
        }
        let payload = self
            .envelope
            .body()
            .first_element_child()
            .expect("validated payload");
        reject_text(payload)?;
        if payload.children().filter(Node::is_element).any(|node| {
            !node.has_tag_name((WSMAN, "OptimizeEnumeration"))
                && !node.has_tag_name((WSMAN, "MaxElements"))
        }) {
            return Err(Error::InvalidEnvelope("unsupported enumeration option"));
        }
        if let Some(optimize) = unique_child(payload, WSMAN, "OptimizeEnumeration")? {
            require_empty(optimize)?;
        } else if subscription.is_some() {
            return Err(Error::InvalidEnvelope(
                "subscription enumeration requires OptimizeEnumeration",
            ));
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
        self.encode_response(
            response_id,
            "http://schemas.xmlsoap.org/ws/2004/09/enumeration/EnumerateResponse",
            true,
            subscription,
        )
    }

    fn encode_response(
        &self,
        response_id: Uuid,
        action: &str,
        enumeration: bool,
        subscription: Option<(&Advertisement, &[u8; 20])>,
    ) -> Result<Vec<u8>, Error> {
        let output = BoundedXml {
            bytes: Vec::new(),
            limit: self
                .max_response_bytes
                .saturating_add(self.max_response_bytes / 2),
        };
        let mut writer = Writer::new(output);
        let result = (|| -> io::Result<()> {
            let mut envelope = BytesStart::new("s:Envelope");
            envelope.push_attribute(("xmlns:s", SOAP));
            envelope.push_attribute(("xmlns:a", ADDRESSING));
            writer.write_event(Event::Start(envelope))?;
            writer.write_event(Event::Start(BytesStart::new("s:Header")))?;
            for (name, value) in [
                ("a:Action", action),
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
                writer.write_event(Event::Start(BytesStart::new("w:Items")))?;
                if let Some((subscription, certificate_thumbprint)) = subscription {
                    subscription.write(&mut writer, certificate_thumbprint)?;
                }
                writer.write_event(Event::End(BytesEnd::new("w:Items")))?;
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
        let response_size = xml
            .encode_utf16()
            .count()
            .checked_mul(2)
            .and_then(|size| size.checked_add(2))
            .ok_or(Error::TooLarge)?;
        if response_size > self.max_response_bytes {
            return Err(Error::TooLarge);
        }
        let mut body = Vec::with_capacity(response_size);
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

fn validate_absolute_uri(value: &str) -> Result<(), Error> {
    if value.chars().any(char::is_whitespace) || url::Url::parse(value).is_err() {
        return Err(Error::InvalidEnvelope("expected absolute URI"));
    }
    Ok(())
}

fn reject_text(element: Node<'_, '_>) -> Result<(), Error> {
    if element
        .children()
        .any(|node| node.is_text() && node.text().is_some_and(|text| !text.trim().is_empty()))
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn request_xml(action: Action, body: &str, headers: &str) -> String {
        format!(
            r#"<s:Envelope xmlns:s="{SOAP}" xmlns:a="{ADDRESSING}" xmlns:w="{WSMAN}" xmlns:e="{EVENTING}" xmlns:n="{ENUMERATION}">
<s:Header><a:Action s:mustUnderstand="true">{}</a:Action><a:MessageID>uuid:AbC-123</a:MessageID>
<a:To>https://collector.example/wsman</a:To>{headers}</s:Header><s:Body>{body}</s:Body></s:Envelope>"#,
            action.uri()
        )
    }

    const DELIVERY_HEADERS: &str =
        "<e:Identifier>reference-property</e:Identifier><w:AckRequested/>";

    /// Scenario: a source reports each standard termination category or an extension status.
    /// Guarantees: only well-formed bodies are accepted and extension statuses stay bounded labels.
    #[test]
    fn validates_subscription_end() {
        let manager = "<e:SubscriptionManager><a:Address>https://source.example/wsman</a:Address><a:ReferenceProperties><e:Identifier>source-side-id</e:Identifier></a:ReferenceProperties></e:SubscriptionManager>";
        for (status, expected) in [
            (
                format!("{EVENTING}/DeliveryFailure"),
                SubscriptionEndStatus::DeliveryFailure,
            ),
            (
                format!("{EVENTING}/SourceShuttingDown"),
                SubscriptionEndStatus::SourceShuttingDown,
            ),
            (
                format!("{EVENTING}/SourceCancelling"),
                SubscriptionEndStatus::SourceCancelling,
            ),
            (
                "urn:example:termination".to_owned(),
                SubscriptionEndStatus::Other,
            ),
        ] {
            let body = format!(
                "<e:SubscriptionEnd>{manager}<e:Status>{status}</e:Status><e:Reason xml:lang=\"en-US\">Stopped &amp; restarting</e:Reason></e:SubscriptionEnd>"
            );
            let xml = request_xml(Action::SubscriptionEnd, &body, DELIVERY_HEADERS);
            let envelope = Envelope::parse(&xml, 8192).unwrap();
            let request = envelope
                .request(&Route::Delivery { version: "opaque" }, 8192)
                .unwrap();
            assert_eq!(request.subscription_end().unwrap(), expected);
            assert!(request.encode_ack(Uuid::nil()).is_ok());
        }
        for content in [
            manager.to_owned(),
            "<e:Status>urn:example:end</e:Status>".to_owned(),
            format!("{manager}{manager}<e:Status>urn:example:end</e:Status>"),
            format!("{manager}<e:Status/>"),
            format!("{manager}<e:Status>relative</e:Status>"),
            format!("{manager}<e:Status>urn:bad status</e:Status>"),
            format!("{manager}<e:Status><e:Status>urn:example:end</e:Status></e:Status>"),
            format!(
                "{manager}<e:Status>urn:example:end</e:Status><e:Status>urn:example:other</e:Status>"
            ),
            format!(
                "{manager}<e:Status>urn:example:end</e:Status><e:Reason><e:Nested/></e:Reason>"
            ),
            format!("{manager}<e:Status>urn:example:end</e:Status><e:Reason/><e:Reason/>"),
            format!("{manager}<e:Status>urn:example:end</e:Status><e:Reason xml:lang=\"\"/>"),
            "<e:SubscriptionManager/><e:Status>urn:example:end</e:Status>".to_owned(),
        ] {
            let xml = request_xml(
                Action::SubscriptionEnd,
                &format!("<e:SubscriptionEnd>{content}</e:SubscriptionEnd>"),
                DELIVERY_HEADERS,
            );
            let envelope = Envelope::parse(&xml, 8192).unwrap();
            let request = envelope
                .request(&Route::Delivery { version: "opaque" }, 8192)
                .unwrap();
            assert!(request.subscription_end().is_err(), "{content}");
        }
    }

    /// Scenario: Windows extends a termination notice with descriptive WSManFault information.
    /// Guarantees: only the exact bounded extension is accepted; malformed or duplicate faults fail.
    #[test]
    fn validates_subscription_end_wsman_fault() {
        let manager = "<e:SubscriptionManager><a:Address>https://source.example/wsman</a:Address></e:SubscriptionManager>";
        let fault = format!(
            "<f:WSManFault xmlns:f=\"{WSMAN_FAULT}\" Code=\"2150859027\" Machine=\"untrusted-machine\"><f:Message>Delivery failed<f:ProviderFault providerId=\"00000000-0000-0000-0000-000000000000\"><detail xmlns=\"urn:example\">descriptive only</detail></f:ProviderFault></f:Message></f:WSManFault>"
        );
        for (extension, valid) in [
            (fault.clone(), true),
            (fault.replace("2150859027", "4294967295"), true),
            (
                format!("<f:WSManFault xmlns:f=\"{WSMAN_FAULT}\" Code=\"0\" Machine=\"host\"/>"),
                true,
            ),
            (fault.replace(" Code=\"2150859027\"", ""), false),
            (fault.replace(" Machine=\"untrusted-machine\"", ""), false),
            (fault.replace("2150859027", "4294967296"), false),
            (fault.replace("2150859027", "-1"), false),
            (fault.replace("2150859027", "not-a-number"), false),
            (format!("{fault}{fault}"), false),
            (fault.replace(WSMAN_FAULT, "urn:wrong-namespace"), false),
            (fault.replace("WSManFault", "UnknownFault"), false),
            (
                fault.replace("<f:Message>", "unexpected text<f:Message>"),
                false,
            ),
            (
                fault.replace("</f:Message>", "</f:Message><f:Message/>"),
                false,
            ),
            (
                fault.replace("</f:ProviderFault>", "</f:ProviderFault><f:ProviderFault/>"),
                false,
            ),
            (fault.replace("ProviderFault", "UnknownProvider"), false),
        ] {
            let body = format!(
                "<e:SubscriptionEnd>{manager}<e:Status>{EVENTING}/DeliveryFailure</e:Status>{extension}</e:SubscriptionEnd>"
            );
            let xml = request_xml(Action::SubscriptionEnd, &body, DELIVERY_HEADERS);
            let envelope = Envelope::parse(&xml, 8192).unwrap();
            let request = envelope
                .request(&Route::Delivery { version: "opaque" }, 8192)
                .unwrap();
            if valid {
                assert_eq!(
                    request.subscription_end().unwrap(),
                    SubscriptionEndStatus::DeliveryFailure
                );
                assert!(request.encode_ack(Uuid::nil()).is_ok());
            } else {
                assert!(request.subscription_end().is_err(), "{extension}");
            }
        }
    }

    /// Scenario: a termination notification contains an unknown child with oversized names.
    /// Guarantees: refusal identifies bounded qualified names without logging attributes or content.
    #[test]
    fn subscription_end_diagnostic_is_bounded() {
        let namespace = format!("urn:unknown:{}", "n".repeat(1024));
        let name = "Unknown".repeat(100);
        let body = format!(
            "<e:SubscriptionEnd><x:{name} xmlns:x=\"{namespace}\" secret=\"private-attribute\">private-body</x:{name}></e:SubscriptionEnd>"
        );
        let xml = request_xml(Action::SubscriptionEnd, &body, DELIVERY_HEADERS);
        let envelope = Envelope::parse(&xml, 8192).unwrap();
        let request = envelope
            .request(&Route::Delivery { version: "opaque" }, 8192)
            .unwrap();
        let error = request.subscription_end().unwrap_err();
        assert!(!error.to_string().contains("private"));
        let Error::UnsupportedSubscriptionEndElement {
            namespace: actual_namespace,
            name: actual_name,
        } = error
        else {
            panic!("expected unsupported termination element");
        };
        assert_eq!(
            actual_namespace,
            namespace.chars().take(128).collect::<String>()
        );
        assert_eq!(actual_name, name.chars().take(128).collect::<String>());
    }

    /// Scenario: optimized enumeration advertises configured settings and either initial-read mode.
    /// Guarantees: Subscribe is bounded, namespace-correct, selects the peer's issuer, and omits SLDC.
    #[test]
    fn encodes_subscription() {
        use crate::receivers::windows_event_forwarding_receiver::config::{Config, InitialRead};
        let mut config: Config = serde_json::from_value(serde_json::json!({
            "endpoint": "127.0.0.1:5986", "public_endpoint": "https://collector.example:5986",
            "tls": {"cert_file": "server.pem", "key_file": "key.pem", "client_ca_files": ["ca.pem"]},
            "auth": {"allowed_sources": ["host.example.com"]},
            "subscriptions": [{"name": "audit & <test>", "query": "<?xml version=\"1.0\"?><QueryList><Query Id=\"0\"><Select Path=\"Application\">*[System[EventID &lt; 42]]</Select></Query></QueryList>", "heartbeat": "1501ms", "connection_retry_count": 3}]
        })).unwrap();
        let xml = request_xml(
            Action::Enumerate,
            "<n:Enumerate><w:OptimizeEnumeration/><w:MaxElements>1</w:MaxElements></n:Enumerate>",
            "<w:ResourceURI>http://schemas.microsoft.com/wbem/wsman/1/SubscriptionManager/Subscription</w:ResourceURI>",
        );
        let envelope = Envelope::parse(&xml, 8192).unwrap();
        let request = envelope
            .request(&Route::SubscriptionManager, 32768)
            .unwrap();
        for initial_read in [InitialRead::NewEvents, InitialRead::AllExistingEvents] {
            config.subscriptions[0].initial_read = initial_read;
            let advertisement = Advertisement::from_config(&config, "receiver-one")
                .unwrap()
                .unwrap();
            let (identifier, version) =
                config.subscriptions[0].identifiers("receiver-one").unwrap();
            let bytes = request
                .encode_enumeration(Uuid::nil(), Some((&advertisement, &[0xab; 20])))
                .unwrap();
            let xml =
                super::super::decode_body(&bytes, super::super::Encoding::Utf16Le, 32768).unwrap();
            let response = Envelope::parse(&xml, 32768).unwrap();
            let root = response.body().first_element_child().unwrap();
            let items = unique_child(root, WSMAN, "Items").unwrap().unwrap();
            assert_eq!(items.children().filter(Node::is_element).count(), 1);
            let item = items.first_element_child().unwrap();
            assert!(item.has_tag_name((
                "http://schemas.microsoft.com/wbem/wsman/1/subscription",
                "Subscription"
            )));
            assert_eq!(
                item.first_element_child().unwrap().text().unwrap(),
                format!("uuid:{version}")
            );
            let nested = unique_child(item, SOAP, "Envelope").unwrap().unwrap();
            let body = unique_child(nested, SOAP, "Body").unwrap().unwrap();
            let subscribe = unique_child(body, EVENTING, "Subscribe").unwrap().unwrap();
            let delivery = unique_child(subscribe, EVENTING, "Delivery")
                .unwrap()
                .unwrap();
            assert_eq!(delivery.attribute("Mode"), Some(Action::Events.uri()));
            assert_eq!(
                unique_child(delivery, WSMAN, "Heartbeats")
                    .unwrap()
                    .unwrap()
                    .text(),
                Some("PT1.501S")
            );
            assert_eq!(
                unique_child(delivery, WSMAN, "MaxTime")
                    .unwrap()
                    .unwrap()
                    .text(),
                Some("PT30.000S")
            );
            let retry = unique_child(delivery, WSMAN, "ConnectionRetry")
                .unwrap()
                .unwrap();
            assert_eq!(retry.attribute("Total"), Some("3"));
            assert_eq!(retry.text(), Some("PT60.000S"));
            assert!(
                nested
                    .descendants()
                    .any(|node| node.attribute("Name") == Some("SubscriptionName")
                        && node.text() == Some("audit & <test>"))
            );
            assert!(
                !nested
                    .descendants()
                    .any(|node| node.attribute("Name") == Some("Compression"))
            );
            let policy_namespace = "http://schemas.xmlsoap.org/ws/2002/12/policy";
            let authentication_namespace =
                "http://schemas.microsoft.com/wbem/wsman/1/authentication";
            let notify_to = unique_child(delivery, EVENTING, "NotifyTo")
                .unwrap()
                .unwrap();
            let policy = unique_child(notify_to, policy_namespace, "Policy")
                .unwrap()
                .unwrap();
            let exactly_one = unique_child(policy, policy_namespace, "ExactlyOne")
                .unwrap()
                .unwrap();
            let assertions = unique_child(exactly_one, policy_namespace, "All")
                .unwrap()
                .unwrap();
            let authentication =
                unique_child(assertions, authentication_namespace, "Authentication")
                    .unwrap()
                    .unwrap();
            for parent in [policy, exactly_one, assertions, authentication] {
                assert_eq!(parent.children().filter(Node::is_element).count(), 1);
            }
            assert_eq!(
                authentication.attribute("Profile"),
                Some("http://schemas.dmtf.org/wbem/wsman/1/wsman/secprofile/https/mutual")
            );
            let certificate = unique_child(
                authentication,
                authentication_namespace,
                "ClientCertificate",
            )
            .unwrap()
            .unwrap();
            let thumbprint = unique_child(certificate, authentication_namespace, "Thumbprint")
                .unwrap()
                .unwrap();
            assert_eq!(thumbprint.attribute("Role"), Some("issuer"));
            assert_eq!(
                thumbprint.text(),
                Some("ABABABABABABABABABABABABABABABABABABABAB")
            );
            for name in ["EndTo", "NotifyTo"] {
                let endpoint = nested
                    .descendants()
                    .find(|node| node.has_tag_name((EVENTING, name)))
                    .unwrap();
                assert_eq!(
                    unique_child(endpoint, ADDRESSING, "Address")
                        .unwrap()
                        .unwrap()
                        .text(),
                    Some(advertisement.delivery_url())
                );
                assert_eq!(
                    endpoint
                        .descendants()
                        .find(|node| node.has_tag_name((EVENTING, "Identifier")))
                        .unwrap()
                        .text()
                        .unwrap(),
                    identifier.to_string()
                );
            }
            let filter = unique_child(subscribe, WSMAN, "Filter").unwrap().unwrap();
            assert!(
                filter
                    .first_element_child()
                    .unwrap()
                    .has_tag_name("QueryList")
            );
            assert_eq!(
                filter
                    .descendants()
                    .find(|node| node.has_tag_name("Select"))
                    .unwrap()
                    .text(),
                Some("*[System[EventID < 42]]")
            );
            let bookmark = unique_child(subscribe, WSMAN, "Bookmark").unwrap();
            if initial_read == InitialRead::AllExistingEvents {
                assert_eq!(
                    bookmark.unwrap().text(),
                    Some("http://schemas.dmtf.org/wbem/wsman/1/wsman/bookmark/earliest")
                );
            } else {
                assert!(bookmark.is_none());
            }
            assert!(
                unique_child(subscribe, WSMAN, "SendBookmarks")
                    .unwrap()
                    .is_some()
            );
            assert!(
                envelope
                    .request(&Route::SubscriptionManager, bytes.len())
                    .unwrap()
                    .encode_enumeration(Uuid::nil(), Some((&advertisement, &[0xab; 20])))
                    .is_ok()
            );
            assert!(matches!(
                envelope
                    .request(&Route::SubscriptionManager, bytes.len() - 1)
                    .unwrap()
                    .encode_enumeration(Uuid::nil(), Some((&advertisement, &[0xab; 20]))),
                Err(Error::TooLarge)
            ));
        }
        for query in [
            "<bad/>",
            "<QueryList>",
            "<!DOCTYPE QueryList [<!ENTITY test 'value'>]><QueryList>&test;</QueryList>",
        ] {
            config.subscriptions[0].query = query.into();
            assert!(Advertisement::from_config(&config, "receiver-one").is_err());
        }
    }

    /// Scenario: every supported action arrives at its intended endpoint with its body shape.
    /// Guarantees: dispatch separates manager and delivery actions while accepting End on both.
    #[test]
    fn dispatches_actions() {
        let delivery = Route::Delivery {
            version: "opaque-old-version",
        };
        for (action, body, headers, route) in [
            (
                Action::Enumerate,
                "<n:Enumerate/>",
                "<w:ResourceURI>http://schemas.microsoft.com/wbem/wsman/1/SubscriptionManager/Subscription</w:ResourceURI>",
                &Route::SubscriptionManager,
            ),
            (
                Action::End,
                "",
                "<w:ResourceURI>http://schemas.microsoft.com/wbem/wsman/1/wsman/FullDuplex</w:ResourceURI>",
                &Route::SubscriptionManager,
            ),
            (
                Action::End,
                "",
                "<w:ResourceURI>http://schemas.microsoft.com/wbem/wsman/1/wsman/FullDuplex</w:ResourceURI>",
                &delivery,
            ),
            (
                Action::Heartbeat,
                "<w:Events/>",
                DELIVERY_HEADERS,
                &delivery,
            ),
            (
                Action::Events,
                "<w:Events><w:Event><![CDATA[<Event/>]]></w:Event></w:Events>",
                DELIVERY_HEADERS,
                &delivery,
            ),
            (
                Action::SubscriptionEnd,
                "<e:SubscriptionEnd/>",
                DELIVERY_HEADERS,
                &delivery,
            ),
        ] {
            let xml = request_xml(action, body, headers);
            let envelope = Envelope::parse(&xml, 8192).unwrap();
            let request = envelope.request(route, 4096).unwrap();
            assert_eq!(request.action(), action);
            assert_eq!(request.message_id(), "uuid:AbC-123");
            if matches!(
                action,
                Action::Heartbeat | Action::Events | Action::SubscriptionEnd
            ) {
                assert_eq!(request.identifier(), Some("reference-property"));
                assert!(request.ack_requested());
                assert!(matches!(
                    envelope.request(&Route::SubscriptionManager, 4096),
                    Err(Error::UnsupportedAction)
                ));
            }
        }
    }

    /// Scenario: SOAP destinations differ in origin, URL shape, or the opaque delivery path.
    /// Guarantees: Only HTTPS destinations for the configured origin and exact incoming path pass.
    #[test]
    fn validates_destination() {
        let path = "/wsman/subscriptions/subscription/old%2Fopaque";
        let route = Route::Delivery {
            version: "old%2Fopaque",
        };
        let xml = request_xml(Action::Heartbeat, "<w:Events/>", DELIVERY_HEADERS);
        let envelope = Envelope::parse(&xml, 8192).unwrap();
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
            format!("https://collector.example{path}/"),
            format!("https://collector.example/ignored/..{path}"),
            format!("https://collector.example{path}").replace("%2F", "%2f"),
            format!("https://collector.example{path}").replace("old%2Fopaque", "current"),
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
        request.destination = "https://elsewhere/wsman/SubscriptionManager/WEC".into();
        request
            .validate_destination(None, "/wsman/SubscriptionManager/WEC")
            .unwrap();
        assert!(request.validate_destination(None, path).is_err());
        request.destination = "http://elsewhere/wsman/SubscriptionManager/WEC".into();
        assert!(
            request
                .validate_destination(None, "/wsman/SubscriptionManager/WEC")
                .is_err()
        );
    }

    /// Scenario: a heartbeat contains data, a spoofed body, ambiguous headers, or unknown requirements.
    /// Guarantees: malformed control requests cannot be dispatched as successful heartbeats.
    #[test]
    fn rejects_invalid_dispatch() {
        let valid = request_xml(Action::Heartbeat, "<w:Events/>", DELIVERY_HEADERS);
        let route = Route::Delivery { version: "any" };
        for invalid in [
            valid.replace("<w:Events/>", "<w:Events><w:Event/></w:Events>"),
            valid.replace("<w:Events/>", "<Events/>"),
            valid.replace("<w:Events/>", "<w:Events/><w:Events/>"),
            valid.replace("<w:AckRequested/>", ""),
            valid.replace("<w:AckRequested/>", "<w:AckRequested/><w:AckRequested/>"),
            valid.replace("<e:Identifier>reference-property</e:Identifier>", ""),
            valid.replace(
                "</s:Header>",
                "<e:Identifier>duplicate</e:Identifier></s:Header>",
            ),
            valid.replace(
                "</s:Header>",
                "<w:Unknown s:mustUnderstand=\"true\"/></s:Header>",
            ),
            valid.replace("s:mustUnderstand=\"true\"", "s:mustUnderstand=\"invalid\""),
            valid.replace(
                "</s:Header>",
                "<a:ReplyTo><a:Address>https://elsewhere/</a:Address></a:ReplyTo></s:Header>",
            ),
            valid.replace(
                "</s:Header>",
                "<w:MaxEnvelopeSize>0</w:MaxEnvelopeSize></s:Header>",
            ),
            valid.replace(
                "</s:Header>",
                "<w:MaxEnvelopeSize>bad</w:MaxEnvelopeSize></s:Header>",
            ),
            valid.replace(Action::Heartbeat.uri(), "urn:unknown:Heartbeat"),
        ] {
            let envelope = Envelope::parse(&invalid, 8192).unwrap();
            assert!(
                envelope.request(&route, 4096).is_err(),
                "accepted {invalid}"
            );
        }
        let optional = valid.replace("</s:Header>", "<w:Unknown s:mustUnderstand=\"false\"/><w:MaxEnvelopeSize>2048</w:MaxEnvelopeSize></s:Header>");
        let envelope = Envelope::parse(&optional, 8192).unwrap();
        assert_eq!(
            envelope.request(&route, 4096).unwrap().max_response_bytes(),
            2048
        );
        assert_eq!(
            envelope.request(&route, 1024).unwrap().max_response_bytes(),
            1024
        );
    }

    /// Scenario: an accepted request contains mixed-case correlation text and XML metacharacters.
    /// Guarantees: the bounded UTF-16 Ack preserves RelatesTo without allowing XML injection.
    #[test]
    fn encodes_bounded_ack() {
        let route = Route::Delivery {
            version: "different-from-identifier",
        };
        let response_id = Uuid::from_u128(123);
        let xml = request_xml(Action::Heartbeat, "<w:Events/>", DELIVERY_HEADERS)
            .replace("uuid:AbC-123", "uuid:AbC&amp;&lt;Tag&gt;&#13;\u{00e9}");
        let envelope = Envelope::parse(&xml, 8192).unwrap();
        let request = envelope.request(&route, 4096).unwrap();
        let bytes = request.encode_ack(response_id).unwrap();
        assert!(bytes.starts_with(&[0xff, 0xfe]));
        let decoded =
            super::super::decode_body(&bytes, super::super::Encoding::Utf16Le, 8192).unwrap();
        let response = Envelope::parse(&decoded, 8192).unwrap();
        assert_eq!(response.action(), ACK);
        assert_eq!(response.message_id(), format!("uuid:{response_id}"));
        let header = response
            .document
            .root_element()
            .first_element_child()
            .unwrap();
        let relates_to = unique_child(header, ADDRESSING, "RelatesTo")
            .unwrap()
            .unwrap();
        assert_eq!(
            scalar_text(relates_to, "RelatesTo").unwrap(),
            "uuid:AbC&<Tag>\r\u{00e9}"
        );
        assert!(
            header
                .children()
                .all(|node| node.tag_name().name() != "OperationID")
        );
        assert!(response.body().children().next().is_none());
        assert!(
            envelope
                .request(&route, bytes.len())
                .unwrap()
                .encode_ack(response_id)
                .is_ok()
        );
        assert!(matches!(
            envelope
                .request(&route, bytes.len() - 1)
                .unwrap()
                .encode_ack(response_id),
            Err(Error::TooLarge)
        ));
        assert!(matches!(
            envelope
                .request(&route, 32)
                .unwrap()
                .encode_ack(response_id),
            Err(Error::TooLarge)
        ));
        let end_xml = request_xml(
            Action::End,
            "",
            "<w:ResourceURI>http://schemas.microsoft.com/wbem/wsman/1/wsman/FullDuplex</w:ResourceURI>",
        );
        let end = Envelope::parse(&end_xml, 8192).unwrap();
        assert!(matches!(
            end.request(&route, 4096).unwrap().encode_ack(response_id),
            Err(Error::UnsupportedAction)
        ));
    }
}
