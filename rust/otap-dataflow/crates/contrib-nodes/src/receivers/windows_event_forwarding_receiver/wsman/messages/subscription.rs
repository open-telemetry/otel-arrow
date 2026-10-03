// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Build and serialize the subscription advertised during WEF enumeration.
//!
//! An advertisement is an `m:Subscription` item containing a version and a nested
//! SOAP Subscribe envelope. It is not the outer enumeration response and does not
//! initiate a connection to Windows. The authenticated source reads these settings
//! and sends events to the advertised HTTPS delivery endpoint.
//!
//! # Identity and source-specific state
//!
//! Configuration validation belongs to `Config::validate`. Construction additionally
//! derives deterministic UUID v5 identities from the actual receiver instance name,
//! subscription name, and canonical subscription settings. The subscription ID
//! remains stable across settings edits; the version tracks those settings. Neither
//! the public origin nor a source's bookmark participates in these UUIDs.
//! The delivery path contains both UUIDs, while endpoint reference properties
//! carry the subscription ID. Each serialization generates a fresh SOAP MessageID;
//! the advertisement is therefore not a cached, byte-identical protocol response.
//!
//! The base advertisement is shared across sources. `with_bookmark` creates a copy
//! containing one source's committed progress without changing identity or version.
//! A supplied bookmark takes precedence over `initial_read`. Without one,
//! `all_existing_events` emits the WS-Man earliest-bookmark URI and `new_events`
//! omits the Bookmark element. Every subscription requests subsequent bookmarks
//! through `w:SendBookmarks`; this module does not persist or advance progress.
//!
//! # Advertised contract
//!
//! - Both EndTo and NotifyTo use the same delivery URL and subscription identifier.
//! - ContentFormat is RenderedText, with CDATA requested for event documents.
//! - ContentEncoding is UTF-16; this is a character encoding, not compression.
//! - Heartbeats, ConnectionRetry (interval and count), MaxTime, and MaxEnvelopeSize
//!   describe source delivery behavior, not the collector's HTTP/feedback deadlines.
//! - NotifyTo advertises mandatory mutual TLS with an uppercase hexadecimal
//!   certificate thumbprint marked `Role="issuer"`. The caller supplies the SHA-1
//!   thumbprint of the verified client certificate's issuer, not the client leaf
//!   or collector server certificate. This certificate-selection hint does not
//!   replace chain verification or source authorization at the HTTPS endpoint.
//!
//! QueryList XML is embedded as markup inside the Windows event-query Filter;
//! XML declarations and processing instructions are omitted. Bookmarks are copied
//! as markup, including their wrapper, and must already be validated and
//! self-contained. The enclosing response writer owns the output byte limit.

use super::{ADDRESSING, ANONYMOUS, EVENTING, SOAP, WSMAN};
use crate::receivers::windows_event_forwarding_receiver::config::{
    Config, InitialRead, Subscription,
};
use quick_xml::{
    Reader, Writer,
    events::{BytesEnd, BytesStart, BytesText, Event},
};
use std::{
    io::{self, Write},
    time::Duration,
};
use uuid::Uuid;

const MUTUAL_TLS: &str = "http://schemas.dmtf.org/wbem/wsman/1/wsman/secprofile/https/mutual";

/// Immutable, validated subscription parameters shared across authenticated sources.
///
/// A per-source copy may include the last committed bookmark. Construction and
/// copying do not perform network I/O or mutate the receiver's progress store.
#[derive(Clone)]
pub struct Advertisement {
    subscription: Subscription,
    identifier: Uuid,
    version: Uuid,
    delivery_url: String,
    bookmark: Option<String>,
}

impl Advertisement {
    /// Validate settings, then build stable identities using the actual receiver name.
    ///
    /// Configuration-only callers should use `Config::validate` instead of
    /// constructing a disposable advertisement. An absent public origin keeps
    /// enumeration empty and does not derive subscription identities.
    ///
    /// Returns `Ok(None)` only after configuration validation succeeds. Otherwise
    /// builds the single configured subscription's HTTPS URL and starts without a
    /// source bookmark. Validation, identity serialization, and URL parsing failures
    /// are returned as diagnostic strings. No certificate files are loaded here.
    pub fn from_config(config: &Config, receiver_name: &str) -> Result<Option<Self>, String> {
        config.validate()?;
        let Some(origin) = &config.public_endpoint else {
            return Ok(None);
        };
        let subscription = &config.subscriptions[0];
        let (identifier, version) = subscription
            .identifiers(receiver_name)
            .map_err(|error| error.to_string())?;
        let mut address = url::Url::parse(origin).map_err(|error| error.to_string())?;
        address.set_path(&format!("/wsman/subscriptions/{identifier}/{version}"));
        Ok(Some(Self {
            subscription: subscription.clone(),
            identifier,
            version,
            delivery_url: address.to_string(),
            bookmark: None,
        }))
    }

    /// Stable subscription identity used for delivery routing, not source ownership.
    /// Changing the receiver or subscription name changes this identity.
    #[must_use]
    pub fn identifier(&self) -> Uuid {
        self.identifier
    }

    /// Address advertised to Windows for event delivery and termination notifications.
    /// Contains the configured public origin and `/wsman/subscriptions/{id}/{version}`.
    #[must_use]
    pub fn delivery_url(&self) -> &str {
        &self.delivery_url
    }

    /// Attach a source's committed bookmark snapshot without modifying the base item.
    ///
    /// The caller must supply validated, self-contained WS-Man Bookmark XML, as
    /// produced by delivery extraction. This method neither parses that XML nor
    /// checks its size. `None` restores the configured initial-read behavior in
    /// this copy; it does not clear any bookmark in the progress store.
    pub(in crate::receivers::windows_event_forwarding_receiver) fn with_bookmark(
        &self,
        bookmark: Option<String>,
    ) -> Self {
        Self {
            bookmark,
            ..self.clone()
        }
    }

    /// Append one subscription item and nested Subscribe envelope to the response.
    ///
    /// `certificate_thumbprint` identifies the authenticated source certificate's
    /// issuer. Scalar values are escaped, while query and bookmark fragments are
    /// copied as XML events. A committed bookmark is emitted before considering
    /// the initial-read fallback, and SendBookmarks is always requested.
    ///
    /// No local output-size budget is imposed: the enclosing writer must enforce
    /// it. Errors propagate from fragment parsing or writing and may leave partial
    /// output in the writer; the caller must discard a failed response.
    pub(super) fn write(
        &self,
        writer: &mut Writer<impl Write>,
        certificate_thumbprint: &[u8; 20],
    ) -> io::Result<()> {
        let mut item = BytesStart::new("m:Subscription");
        item.push_attribute((
            "xmlns:m",
            "http://schemas.microsoft.com/wbem/wsman/1/subscription",
        ));
        writer.write_event(Event::Start(item))?;
        text(writer, "m:Version", &format!("uuid:{}", self.version), &[])?;
        let mut envelope = BytesStart::new("s:Envelope");
        for attribute in [
            ("xmlns:s", SOAP),
            ("xmlns:a", ADDRESSING),
            ("xmlns:e", EVENTING),
            ("xmlns:w", WSMAN),
            (
                "xmlns:p",
                "http://schemas.microsoft.com/wbem/wsman/1/wsman.xsd",
            ),
            ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
        ] {
            envelope.push_attribute(attribute);
        }
        writer.write_event(Event::Start(envelope))?;
        start(writer, "s:Header")?;
        text(writer, "a:To", ANONYMOUS, &[])?;
        text(
            writer,
            "w:ResourceURI",
            "http://schemas.microsoft.com/wbem/wsman/1/windows/EventLog",
            &[("s:mustUnderstand", "true")],
        )?;
        start(writer, "a:ReplyTo")?;
        text(writer, "a:Address", ANONYMOUS, &[])?;
        end(writer, "a:ReplyTo")?;
        text(
            writer,
            "a:Action",
            "http://schemas.xmlsoap.org/ws/2004/08/eventing/Subscribe",
            &[("s:mustUnderstand", "true")],
        )?;
        text(
            writer,
            "a:MessageID",
            &format!("uuid:{}", Uuid::new_v4()),
            &[],
        )?;
        text(
            writer,
            "w:MaxEnvelopeSize",
            &self.subscription.max_envelope_size_bytes.to_string(),
            &[("s:mustUnderstand", "true")],
        )?;
        start(writer, "w:OptionSet")?;
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
        start(writer, "s:Body")?;
        start(writer, "e:Subscribe")?;
        start(writer, "e:EndTo")?;
        self.endpoint_reference(writer)?;
        end(writer, "e:EndTo")?;
        let mut delivery = BytesStart::new("e:Delivery");
        delivery.push_attribute(("Mode", "http://schemas.dmtf.org/wbem/wsman/1/wsman/Events"));
        writer.write_event(Event::Start(delivery))?;
        text(
            writer,
            "w:Heartbeats",
            &duration(self.subscription.heartbeat),
            &[],
        )?;
        start(writer, "e:NotifyTo")?;
        self.endpoint_reference(writer)?;
        let mut policy = BytesStart::new("c:Policy");
        policy.push_attribute(("xmlns:c", "http://schemas.xmlsoap.org/ws/2002/12/policy"));
        policy.push_attribute((
            "xmlns:auth",
            "http://schemas.microsoft.com/wbem/wsman/1/authentication",
        ));
        writer.write_event(Event::Start(policy))?;
        start(writer, "c:ExactlyOne")?;
        start(writer, "c:All")?;
        let mut authentication = BytesStart::new("auth:Authentication");
        authentication.push_attribute(("Profile", MUTUAL_TLS));
        writer.write_event(Event::Start(authentication))?;
        start(writer, "auth:ClientCertificate")?;
        let thumbprint: String = certificate_thumbprint
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
        let mut filter = BytesStart::new("w:Filter");
        filter.push_attribute((
            "Dialect",
            "http://schemas.microsoft.com/win/2004/08/events/eventquery",
        ));
        writer.write_event(Event::Start(filter))?;
        let mut reader = Reader::from_str(&self.subscription.query);
        loop {
            match reader.read_event().map_err(io::Error::other)? {
                Event::Eof => break,
                Event::Decl(_) | Event::PI(_) => {}
                event => writer.write_event(event)?,
            }
        }
        end(writer, "w:Filter")?;
        if let Some(bookmark) = &self.bookmark {
            let mut reader = Reader::from_str(bookmark);
            loop {
                match reader.read_event().map_err(io::Error::other)? {
                    Event::Eof => break,
                    event => writer.write_event(event)?,
                }
            }
        } else if self.subscription.initial_read == InitialRead::AllExistingEvents {
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

    /// Write the shared Address and Identifier children for EndTo or NotifyTo.
    /// The caller owns the surrounding endpoint-reference element.
    fn endpoint_reference(&self, writer: &mut Writer<impl Write>) -> io::Result<()> {
        text(writer, "a:Address", &self.delivery_url, &[])?;
        start(writer, "a:ReferenceProperties")?;
        text(writer, "e:Identifier", &self.identifier.to_string(), &[])?;
        end(writer, "a:ReferenceProperties")
    }
}

/// Open a protocol element using a caller-supplied qualified name.
fn start(writer: &mut Writer<impl Write>, name: &str) -> io::Result<()> {
    writer.write_event(Event::Start(BytesStart::new(name)))
}

/// Close a protocol element; the caller is responsible for matching its start.
fn end(writer: &mut Writer<impl Write>, name: &str) -> io::Result<()> {
    writer.write_event(Event::End(BytesEnd::new(name)))
}

/// Write scalar text, escaping XML delimiters and preserving carriage returns.
///
/// Carriage returns use character references so XML newline normalization does
/// not turn them into line feeds. Attribute values are supplied separately by
/// callers; this helper's explicit text escaping does not apply to attributes.
fn text(
    writer: &mut Writer<impl Write>,
    name: &str,
    value: &str,
    attributes: &[(&str, &str)],
) -> io::Result<()> {
    let escaped = quick_xml::escape::escape(value).replace('\r', "&#13;");
    let _ = writer
        .create_element(name)
        .with_attributes(attributes.iter().copied())
        .write_text_content(BytesText::from_escaped(escaped))?;
    Ok(())
}

/// Format validated whole-millisecond durations as `PT<seconds>.<millis>S`.
/// Sub-millisecond precision would be truncated; configuration rejects it earlier.
fn duration(value: Duration) -> String {
    format!("PT{}.{:03}S", value.as_secs(), value.subsec_millis())
}
