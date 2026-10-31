// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! One immutable subscription advertised inside an optimized EnumerateResponse.
//!
//! The nested Subscribe envelope instructs the authenticated Windows source to
//! deliver events; it does not initiate a connection from the collector.
//! Query XML is validated at construction and streamed as markup, not escaped
//! text. Construction caps queries at 1 MiB and 65,536 XML nodes. The enclosing
//! response writer applies the negotiated byte budget to all generated content.
//!
//! The issuer SHA-1 thumbprint is a Windows certificate-selection hint, not an
//! authentication decision. It must come from the already verified client's
//! issuing CA, never the client leaf or collector server certificate.
//! Committed bookmarks and actual receiver activation are not implemented here.

use super::{ADDRESSING, ANONYMOUS, BoundedXml, EVENTING, SOAP, WSMAN, write_escaped_text};
use crate::{
    common::xml::Document,
    receivers::windows_event_forwarding_receiver::config::{Config, InitialRead, Subscription},
};
use quick_xml::{
    Reader, Writer,
    events::{BytesEnd, BytesStart, Event},
};
use std::{io, time::Duration};
use uuid::Uuid;

// At most 1 MiB of configured query XML is parsed and retained per advertisement.
const MAX_QUERY_BYTES: usize = 1024 * 1024;
// Cap query XML nodes independently of bytes to bound parser bookkeeping.
const MAX_QUERY_NODES: usize = 65_536;

/// Validated advertisement shared across sources, without mutable delivery state.
pub struct Advertisement {
    subscription: Subscription,
    identifier: Uuid,
    version: Uuid,
    delivery_url: String,
    public_origin: url::Origin,
}

impl Advertisement {
    /// Validate settings and derive stable identities using the receiver instance name.
    ///
    /// No public endpoint means no advertisement, after configuration validation.
    /// Startup callers can retain [`Self::public_origin`] for request validation.
    ///
    /// # Errors
    ///
    /// Rejects invalid configuration, oversized or malformed query XML, and names
    /// containing characters forbidden in XML 1.0. Does not load credentials.
    pub fn from_config(config: &Config, receiver_name: &str) -> Result<Option<Self>, String> {
        if config.public_endpoint.is_some()
            && let Some(subscription) = config.subscriptions.first()
        {
            if subscription.query.len() > MAX_QUERY_BYTES {
                return Err("subscription query exceeds 1 MiB".into());
            }
            let _ = Document::parse_bounded(&subscription.query, MAX_QUERY_NODES)
                .map_err(|error| format!("invalid subscription query XML: {error}"))?;
            if !subscription.name.chars().all(|c| {
                matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}'
                    | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
            }) {
                return Err("subscription name contains a prohibited XML character".into());
            }
        }
        config.validate()?;
        let Some(endpoint) = &config.public_endpoint else {
            return Ok(None);
        };
        let subscription = &config.subscriptions[0];
        let (identifier, version) = subscription
            .identifiers(receiver_name)
            .map_err(|error| error.to_string())?;
        let mut address = url::Url::parse(endpoint).map_err(|error| error.to_string())?;
        let public_origin = address.origin();
        address.set_path(&format!("/wsman/subscriptions/{identifier}/{version}"));
        Ok(Some(Self {
            subscription: subscription.clone(),
            identifier,
            version,
            delivery_url: address.to_string(),
            public_origin,
        }))
    }

    /// Stable subscription identifier carried in endpoint reference properties.
    #[must_use]
    pub fn identifier(&self) -> Uuid {
        self.identifier
    }

    /// Version derived from canonical subscription settings.
    #[must_use]
    pub fn version(&self) -> Uuid {
        self.version
    }

    /// HTTPS destination used by both NotifyTo and EndTo.
    #[must_use]
    pub fn delivery_url(&self) -> &str {
        &self.delivery_url
    }

    /// Pre-parsed origin for validating incoming destinations without reparsing configuration.
    #[must_use]
    pub fn public_origin(&self) -> &url::Origin {
        &self.public_origin
    }

    pub(super) fn write(
        &self,
        writer: &mut Writer<BoundedXml>,
        issuer_thumbprint: &[u8; 20],
        subscribe_id: Uuid,
    ) -> io::Result<()> {
        start(
            writer,
            "m:Subscription",
            &[(
                "xmlns:m",
                "http://schemas.microsoft.com/wbem/wsman/1/subscription",
            )],
        )?;
        text(writer, "m:Version", &format!("uuid:{}", self.version), &[])?;
        start(
            writer,
            "s:Envelope",
            &[
                ("xmlns:s", SOAP),
                ("xmlns:a", ADDRESSING),
                ("xmlns:e", EVENTING),
                ("xmlns:w", WSMAN),
                ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
            ],
        )?;
        start(writer, "s:Header", &[])?;
        text(writer, "a:To", ANONYMOUS, &[])?;
        text(
            writer,
            "w:ResourceURI",
            "http://schemas.microsoft.com/wbem/wsman/1/windows/EventLog",
            &[("s:mustUnderstand", "true")],
        )?;
        start(writer, "a:ReplyTo", &[])?;
        text(writer, "a:Address", ANONYMOUS, &[])?;
        end(writer, "a:ReplyTo")?;
        text(
            writer,
            "a:Action",
            "http://schemas.xmlsoap.org/ws/2004/08/eventing/Subscribe",
            &[("s:mustUnderstand", "true")],
        )?;
        text(writer, "a:MessageID", &format!("uuid:{subscribe_id}"), &[])?;
        text(
            writer,
            "w:MaxEnvelopeSize",
            &self.subscription.max_envelope_size_bytes.to_string(),
            &[("s:mustUnderstand", "true")],
        )?;
        start(writer, "w:OptionSet", &[])?;
        text(
            writer,
            "w:Option",
            &self.subscription.name,
            &[("Name", "SubscriptionName")],
        )?;
        text(
            writer,
            "w:Option",
            "",
            &[("Name", "CDATA"), ("xsi:nil", "true")],
        )?;
        text(
            writer,
            "w:Option",
            "RenderedText",
            &[("Name", "ContentFormat")],
        )?;
        end(writer, "w:OptionSet")?;
        end(writer, "s:Header")?;
        start(writer, "s:Body", &[])?;
        start(writer, "e:Subscribe", &[])?;
        start(writer, "e:EndTo", &[])?;
        self.endpoint_reference(writer)?;
        end(writer, "e:EndTo")?;
        start(
            writer,
            "e:Delivery",
            &[("Mode", "http://schemas.dmtf.org/wbem/wsman/1/wsman/Events")],
        )?;
        text(
            writer,
            "w:Heartbeats",
            &duration(self.subscription.heartbeat),
            &[],
        )?;
        start(writer, "e:NotifyTo", &[])?;
        self.endpoint_reference(writer)?;
        start(
            writer,
            "c:Policy",
            &[
                ("xmlns:c", "http://schemas.xmlsoap.org/ws/2002/12/policy"),
                (
                    "xmlns:auth",
                    "http://schemas.microsoft.com/wbem/wsman/1/authentication",
                ),
            ],
        )?;
        start(writer, "c:ExactlyOne", &[])?;
        start(writer, "c:All", &[])?;
        start(
            writer,
            "auth:Authentication",
            &[(
                "Profile",
                "http://schemas.dmtf.org/wbem/wsman/1/wsman/secprofile/https/mutual",
            )],
        )?;
        start(writer, "auth:ClientCertificate", &[])?;
        let thumbprint: String = issuer_thumbprint
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect();
        text(
            writer,
            "auth:Thumbprint",
            &thumbprint,
            &[("Role", "issuer")],
        )?;
        end(writer, "auth:ClientCertificate")?;
        end(writer, "auth:Authentication")?;
        end(writer, "c:All")?;
        end(writer, "c:ExactlyOne")?;
        end(writer, "c:Policy")?;
        end(writer, "e:NotifyTo")?;
        text(
            writer,
            "w:ConnectionRetry",
            &duration(self.subscription.connection_retry),
            &[(
                "Total",
                &self.subscription.connection_retry_count.to_string(),
            )],
        )?;
        text(
            writer,
            "w:MaxTime",
            &duration(self.subscription.max_time),
            &[],
        )?;
        text(
            writer,
            "w:MaxEnvelopeSize",
            &self.subscription.max_envelope_size_bytes.to_string(),
            &[("Policy", "Notify")],
        )?;
        text(writer, "w:ContentEncoding", "UTF-16", &[])?;
        end(writer, "e:Delivery")?;
        start(
            writer,
            "w:Filter",
            &[(
                "Dialect",
                "http://schemas.microsoft.com/win/2004/08/events/eventquery",
            )],
        )?;
        let mut reader = Reader::from_str(&self.subscription.query);
        loop {
            match reader.read_event().map_err(io::Error::other)? {
                Event::Eof => break,
                Event::Decl(_) | Event::PI(_) => {}
                event => writer.write_event(event)?,
            }
        }
        end(writer, "w:Filter")?;
        if self.subscription.initial_read == InitialRead::AllExistingEvents {
            text(
                writer,
                "w:Bookmark",
                "http://schemas.dmtf.org/wbem/wsman/1/wsman/bookmark/earliest",
                &[],
            )?;
        }
        writer.write_event(Event::Empty(BytesStart::new("w:SendBookmarks")))?;
        end(writer, "e:Subscribe")?;
        end(writer, "s:Body")?;
        end(writer, "s:Envelope")?;
        end(writer, "m:Subscription")
    }

    fn endpoint_reference(&self, writer: &mut Writer<BoundedXml>) -> io::Result<()> {
        text(writer, "a:Address", &self.delivery_url, &[])?;
        start(writer, "a:ReferenceProperties", &[])?;
        text(writer, "e:Identifier", &self.identifier.to_string(), &[])?;
        end(writer, "a:ReferenceProperties")
    }
}

fn start(
    writer: &mut Writer<BoundedXml>,
    name: &str,
    attributes: &[(&str, &str)],
) -> io::Result<()> {
    let mut element = BytesStart::new(name);
    let _ = element.extend_attributes(attributes.iter().copied());
    writer.write_event(Event::Start(element))
}

fn end(writer: &mut Writer<BoundedXml>, name: &str) -> io::Result<()> {
    writer.write_event(Event::End(BytesEnd::new(name)))
}

fn text(
    writer: &mut Writer<BoundedXml>,
    name: &str,
    value: &str,
    attributes: &[(&str, &str)],
) -> io::Result<()> {
    start(writer, name, attributes)?;
    write_escaped_text(writer.get_mut(), value)?;
    end(writer, name)
}

fn duration(value: Duration) -> String {
    format!("PT{}.{:03}S", value.as_secs(), value.subsec_millis())
}

#[cfg(test)]
mod tests {
    use super::super::{Action, Envelope, Error, Route, unique_child};
    use super::*;
    use crate::common::xml::Node;
    use crate::receivers::windows_event_forwarding_receiver::wsman::{Encoding, decode_body};

    fn config() -> Config {
        serde_json::from_value(serde_json::json!({
            "endpoint": "127.0.0.1:0",
            "public_endpoint": "https://collector.example:5986",
            "tls": {"cert_file": "server.pem", "key_file": "key.pem", "client_ca_files": ["ca.pem"]},
            "auth": {"allowed_sources": ["host.example.com"]},
            "subscriptions": [{
                "name": "audit & <test>\r\u{e9}",
                "query": "<?xml version=\"1.0\"?><?query test?><QueryList><Query Id=\"0\"><Select Path=\"Application\">*[System[EventID &lt; 42]]</Select></Query></QueryList>",
                "heartbeat": "1501ms", "connection_retry": "2750ms",
                "connection_retry_count": 3, "max_time": "12ms",
                "max_envelope_size_bytes": 100000
            }]
        })).unwrap()
    }

    fn enumerate_xml() -> String {
        format!(
            r#"<s:Envelope xmlns:s="{SOAP}" xmlns:a="{ADDRESSING}" xmlns:w="{WSMAN}" xmlns:n="http://schemas.xmlsoap.org/ws/2004/09/enumeration">
<s:Header><a:Action>{}</a:Action><a:MessageID>uuid:request&amp;&#13;</a:MessageID><a:To>https://collector.example:5986/wsman/SubscriptionManager</a:To><w:ResourceURI>http://schemas.microsoft.com/wbem/wsman/1/SubscriptionManager/Subscription</w:ResourceURI></s:Header>
<s:Body><n:Enumerate><w:OptimizeEnumeration/><w:MaxElements>1</w:MaxElements></n:Enumerate></s:Body></s:Envelope>"#,
            Action::Enumerate.uri()
        )
    }

    fn child<'d, 'i>(node: Node<'d, 'i>, ns: &str, name: &'static str) -> Node<'d, 'i> {
        unique_child(node, ns, name).unwrap().unwrap()
    }

    /// Scenario: optimized enumeration advertises one subscription in either configured initial-read mode.
    /// Guarantees: identities, endpoints, settings, XML query, and issuer authentication policy match configuration.
    #[test]
    fn encodes_subscription_contract() {
        let mut config = config();
        let request_xml = enumerate_xml();
        let envelope = Envelope::parse(&request_xml, 8192, 128).unwrap();
        let request = envelope
            .request(&Route::SubscriptionManager, 32768)
            .unwrap();
        for mode in [InitialRead::NewEvents, InitialRead::AllExistingEvents] {
            config.subscriptions[0].initial_read = mode;
            let advertisement = Advertisement::from_config(&config, "receiver-one")
                .unwrap()
                .unwrap();
            let (identifier, version) =
                config.subscriptions[0].identifiers("receiver-one").unwrap();
            assert_eq!(advertisement.identifier(), identifier);
            assert_eq!(advertisement.version(), version);
            assert_eq!(
                advertisement.delivery_url(),
                format!(
                    "https://collector.example:5986/wsman/subscriptions/{identifier}/{version}"
                )
            );
            request
                .validate_destination(
                    Some(advertisement.public_origin()),
                    "/wsman/SubscriptionManager",
                )
                .unwrap();
            let bytes = request
                .encode_enumeration(
                    Uuid::from_u128(1),
                    &advertisement,
                    &[0xab; 20],
                    Uuid::from_u128(2),
                )
                .unwrap();
            assert!(bytes.starts_with(&[0xff, 0xfe]));
            let xml = decode_body(&bytes, Encoding::Utf16Le, 32768).unwrap();
            assert!(!xml.contains("<?xml"));
            assert!(!xml.contains("<?query"));
            let response = Envelope::parse(&xml, 32768, 512).unwrap();
            assert_eq!(response.action(), super::super::ENUMERATE_RESPONSE);
            assert_eq!(
                response.message_id(),
                format!("uuid:{}", Uuid::from_u128(1))
            );
            let outer_header = child(response.document.root_element(), SOAP, "Header");
            assert_eq!(
                child(outer_header, ADDRESSING, "RelatesTo").text(),
                Some(envelope.message_id())
            );
            let enumeration = response.body().first_element_child().unwrap();
            let items = child(enumeration, WSMAN, "Items");
            assert_eq!(items.children().filter(Node::is_element).count(), 1);
            let item = child(
                items,
                "http://schemas.microsoft.com/wbem/wsman/1/subscription",
                "Subscription",
            );
            assert_eq!(
                item.first_element_child().unwrap().text().unwrap(),
                format!("uuid:{version}")
            );
            let nested = child(item, SOAP, "Envelope");
            let header = child(nested, SOAP, "Header");
            assert_eq!(child(header, ADDRESSING, "To").text(), Some(ANONYMOUS));
            assert_eq!(
                child(child(header, ADDRESSING, "ReplyTo"), ADDRESSING, "Address").text(),
                Some(ANONYMOUS)
            );
            assert_eq!(
                child(header, ADDRESSING, "Action").text(),
                Some("http://schemas.xmlsoap.org/ws/2004/08/eventing/Subscribe")
            );
            assert_eq!(
                child(header, ADDRESSING, "MessageID").text().unwrap(),
                format!("uuid:{}", Uuid::from_u128(2))
            );
            assert_eq!(
                child(header, WSMAN, "ResourceURI").text(),
                Some("http://schemas.microsoft.com/wbem/wsman/1/windows/EventLog")
            );
            let options = child(header, WSMAN, "OptionSet");
            for (name, value) in [
                ("SubscriptionName", config.subscriptions[0].name.as_str()),
                ("ContentFormat", "RenderedText"),
            ] {
                assert!(
                    options
                        .children()
                        .any(|node| node.attribute("Name") == Some(name)
                            && node.text() == Some(value))
                );
            }
            assert!(
                options
                    .children()
                    .any(|node| node.attribute("Name") == Some("CDATA")
                        && node.attribute(("http://www.w3.org/2001/XMLSchema-instance", "nil"))
                            == Some("true"))
            );
            assert_eq!(options.children().filter(Node::is_element).count(), 3);
            let subscribe = child(child(nested, SOAP, "Body"), EVENTING, "Subscribe");
            let delivery = child(subscribe, EVENTING, "Delivery");
            assert_eq!(delivery.attribute("Mode"), Some(Action::Events.uri()));
            for (name, value) in [
                ("Heartbeats", "PT1.501S"),
                ("MaxTime", "PT0.012S"),
                ("ConnectionRetry", "PT2.750S"),
                ("ContentEncoding", "UTF-16"),
            ] {
                assert_eq!(child(delivery, WSMAN, name).text(), Some(value));
            }
            assert_eq!(
                child(delivery, WSMAN, "ConnectionRetry").attribute("Total"),
                Some("3")
            );
            for parent in [header, delivery] {
                assert_eq!(
                    child(parent, WSMAN, "MaxEnvelopeSize").text(),
                    Some("100000")
                );
            }
            assert_eq!(
                child(delivery, WSMAN, "MaxEnvelopeSize").attribute("Policy"),
                Some("Notify")
            );
            for endpoint in [
                child(subscribe, EVENTING, "EndTo"),
                child(delivery, EVENTING, "NotifyTo"),
            ] {
                assert_eq!(
                    child(endpoint, ADDRESSING, "Address").text(),
                    Some(advertisement.delivery_url())
                );
                assert_eq!(
                    child(
                        child(endpoint, ADDRESSING, "ReferenceProperties"),
                        EVENTING,
                        "Identifier"
                    )
                    .text()
                    .unwrap(),
                    identifier.to_string()
                );
            }
            let policy_ns = "http://schemas.xmlsoap.org/ws/2002/12/policy";
            let auth_ns = "http://schemas.microsoft.com/wbem/wsman/1/authentication";
            let notify = child(delivery, EVENTING, "NotifyTo");
            let policy = child(notify, policy_ns, "Policy");
            let exactly_one = child(policy, policy_ns, "ExactlyOne");
            let all = child(exactly_one, policy_ns, "All");
            let auth = child(all, auth_ns, "Authentication");
            for parent in [policy, exactly_one, all, auth] {
                assert_eq!(parent.children().filter(Node::is_element).count(), 1);
            }
            assert_eq!(
                auth.attribute("Profile"),
                Some("http://schemas.dmtf.org/wbem/wsman/1/wsman/secprofile/https/mutual")
            );
            let thumbprint = child(
                child(auth, auth_ns, "ClientCertificate"),
                auth_ns,
                "Thumbprint",
            );
            assert_eq!(thumbprint.attribute("Role"), Some("issuer"));
            assert_eq!(
                thumbprint.text(),
                Some("ABABABABABABABABABABABABABABABABABABABAB")
            );
            let filter = child(subscribe, WSMAN, "Filter");
            assert_eq!(
                filter.attribute("Dialect"),
                Some("http://schemas.microsoft.com/win/2004/08/events/eventquery")
            );
            let query_list = filter.first_element_child().unwrap();
            assert!(query_list.has_tag_name("QueryList"));
            let query = query_list.first_element_child().unwrap();
            assert!(query.has_tag_name("Query"));
            let select = query.first_element_child().unwrap();
            assert!(select.has_tag_name("Select"));
            assert_eq!(select.text(), Some("*[System[EventID < 42]]"));
            assert_eq!(select.attribute("Path"), Some("Application"));
            let bookmark = unique_child(subscribe, WSMAN, "Bookmark").unwrap();
            match mode {
                InitialRead::NewEvents => assert!(bookmark.is_none()),
                InitialRead::AllExistingEvents => assert_eq!(
                    bookmark.unwrap().text(),
                    Some("http://schemas.dmtf.org/wbem/wsman/1/wsman/bookmark/earliest")
                ),
            }
            assert!(
                unique_child(subscribe, WSMAN, "SendBookmarks")
                    .unwrap()
                    .is_some()
            );
            assert!(
                unique_child(enumeration, WSMAN, "EndOfSequence")
                    .unwrap()
                    .is_some()
            );
        }
    }

    /// Scenario: full advertisements reach exact output limits or contain large escaped names/queries.
    /// Guarantees: final UTF-16 limits include the BOM; oversized responses fail without partial success.
    #[test]
    fn bounds_subscription_responses() {
        let xml = enumerate_xml();
        let envelope = Envelope::parse(&xml, 8192, 128).unwrap();
        let mut config = config();
        let advertisement = Advertisement::from_config(&config, "receiver")
            .unwrap()
            .unwrap();
        let encode = |limit| {
            envelope
                .request(&Route::SubscriptionManager, limit)
                .unwrap()
                .encode_enumeration(Uuid::nil(), &advertisement, &[0; 20], Uuid::nil())
        };
        let bytes = encode(32768).unwrap();
        assert_eq!(encode(bytes.len()).unwrap(), bytes);
        for limit in [0, 4096, bytes.len() - 1] {
            assert!(matches!(encode(limit), Err(Error::TooLarge)));
        }
        let limited = xml.replace(
            "</s:Header>",
            "<w:MaxEnvelopeSize>4096</w:MaxEnvelopeSize></s:Header>",
        );
        let limited = Envelope::parse(&limited, 8192, 128).unwrap();
        assert!(matches!(
            limited
                .request(&Route::SubscriptionManager, 32768)
                .unwrap()
                .encode_enumeration(Uuid::nil(), &advertisement, &[0; 20], Uuid::nil()),
            Err(Error::TooLarge)
        ));
        for large_name in [true, false] {
            if large_name {
                config.subscriptions[0].name = "&".repeat(1024 * 1024);
            } else {
                config.subscriptions[0].name = "test".into();
                config.subscriptions[0].query =
                    format!("<QueryList>{}</QueryList>", " ".repeat(128 * 1024));
            }
            let advertisement = Advertisement::from_config(&config, "receiver")
                .unwrap()
                .unwrap();
            assert!(matches!(
                envelope
                    .request(&Route::SubscriptionManager, 4096)
                    .unwrap()
                    .encode_enumeration(Uuid::nil(), &advertisement, &[0; 20], Uuid::nil()),
                Err(Error::TooLarge)
            ));
        }
    }

    /// Scenario: configuration disables advertisement or supplies malformed and excessive XML.
    /// Guarantees: invalid input never produces an advertisement; absent public origin preserves empty mode.
    #[test]
    fn validates_advertisement_configuration() {
        let mut config = config();
        config.public_endpoint = None;
        assert!(
            Advertisement::from_config(&config, "receiver")
                .unwrap()
                .is_none()
        );
        config.auth.allowed_sources.clear();
        assert!(Advertisement::from_config(&config, "receiver").is_err());
        let mut config = self::config();
        for query in [
            "<Other/>".into(),
            "<QueryList>".into(),
            "<!DOCTYPE QueryList><QueryList/>".into(),
            format!("<QueryList>{}</QueryList>", " ".repeat(MAX_QUERY_BYTES)),
            format!(
                "<QueryList>{}</QueryList>",
                "<Query/>".repeat(MAX_QUERY_NODES)
            ),
        ] {
            config.subscriptions[0].query = query;
            assert!(Advertisement::from_config(&config, "receiver").is_err());
        }
        config.subscriptions[0].query = "<QueryList/>".into();
        config.subscriptions[0].name = "invalid\u{0}name".into();
        assert!(Advertisement::from_config(&config, "receiver").is_err());
    }

    /// Scenario: query documents reach the independent construction byte and node limits exactly.
    /// Guarantees: each exact limit is accepted, and one additional byte or node is rejected.
    #[test]
    fn enforces_exact_query_limits() {
        let mut config = config();
        let wrapper = "<QueryList></QueryList>";
        config.subscriptions[0].query = format!(
            "<QueryList>{}</QueryList>",
            " ".repeat(MAX_QUERY_BYTES - wrapper.len())
        );
        assert_eq!(config.subscriptions[0].query.len(), MAX_QUERY_BYTES);
        assert!(Advertisement::from_config(&config, "receiver").is_ok());
        config.subscriptions[0].query.push(' ');
        assert!(Advertisement::from_config(&config, "receiver").is_err());
        config.subscriptions[0].query = format!(
            "<QueryList>{}</QueryList>",
            "<Query/>".repeat(MAX_QUERY_NODES - 1)
        );
        assert!(Advertisement::from_config(&config, "receiver").is_ok());
        config.subscriptions[0].query = config.subscriptions[0]
            .query
            .replace("</QueryList>", "<Query/></QueryList>");
        assert!(Advertisement::from_config(&config, "receiver").is_err());
    }

    /// Scenario: enumeration has oversized correlation, or a delivery action asks for an advertisement.
    /// Guarantees: full advertisements retain bounded correlation escaping and action restrictions.
    #[test]
    fn rejects_invalid_advertisement_requests() {
        let advertisement = Advertisement::from_config(&config(), "receiver")
            .unwrap()
            .unwrap();
        let large = enumerate_xml().replace(
            "uuid:request&amp;&#13;",
            &format!("<![CDATA[{}]]>", "&".repeat(1024 * 1024)),
        );
        let envelope = Envelope::parse(&large, large.len(), 128).unwrap();
        assert!(matches!(
            envelope
                .request(&Route::SubscriptionManager, 4096)
                .unwrap()
                .encode_enumeration(Uuid::nil(), &advertisement, &[0; 20], Uuid::nil()),
            Err(Error::TooLarge)
        ));
        let heartbeat = enumerate_xml()
            .replace(Action::Enumerate.uri(), Action::Heartbeat.uri())
            .replace(
                "<n:Enumerate><w:OptimizeEnumeration/><w:MaxElements>1</w:MaxElements></n:Enumerate>",
                "<w:Events/>",
            )
            .replace("</s:Header>", &format!(
                "<e:Identifier xmlns:e=\"{EVENTING}\">subscription</e:Identifier><w:AckRequested/></s:Header>"
            ));
        let envelope = Envelope::parse(&heartbeat, 8192, 128).unwrap();
        assert!(matches!(
            envelope
                .request(&Route::Delivery { version: "version" }, 32768)
                .unwrap()
                .encode_enumeration(Uuid::nil(), &advertisement, &[0; 20], Uuid::nil()),
            Err(Error::UnsupportedAction)
        ));
    }

    /// Scenario: the advertised origin or receiver name changes without modifying subscription settings.
    /// Guarantees: public-origin edits preserve identity/version; receiver-name edits isolate identities.
    #[test]
    fn preserves_subscription_identity() {
        let mut config = config();
        let first = Advertisement::from_config(&config, "receiver")
            .unwrap()
            .unwrap();
        config.public_endpoint = Some("https://other.example:8443".into());
        let second = Advertisement::from_config(&config, "receiver")
            .unwrap()
            .unwrap();
        assert_eq!(first.identifier(), second.identifier());
        assert_eq!(first.version(), second.version());
        assert_ne!(first.delivery_url(), second.delivery_url());
        let third = Advertisement::from_config(&config, "other-receiver")
            .unwrap()
            .unwrap();
        assert_ne!(first.identifier(), third.identifier());
        assert_ne!(first.version(), third.version());
    }
}
