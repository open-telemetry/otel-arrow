//! HTTPS transport and request dispatch for source-initiated Windows forwarding.
//!
//! # Trust and routing
//!
//! TLS certificate verification and source authorization complete before HTTP
//! parsing. Requests use that authenticated identity for progress and event
//! attribution; SOAP fields do not establish source identity. Authentication
//! failures close the connection rather than returning HTTP 401 or 403.
//!
//! The manager endpoint is `/wsman/SubscriptionManager/WEC`. When a public origin
//! enables advertisement, delivery routes are validated against the advertised
//! subscription identifier. SOAP destinations are checked against the configured
//! origin and request path. Query strings are rejected and only POST is accepted.
//! The delivery version is opaque and passed unchanged to event conversion.
//!
//! # HTTP response mapping
//!
//! | Status | Handler outcome |
//! | --- | --- |
//! | 200 | SOAP enumeration response or successful delivery/control Ack. |
//! | 204 | Accepted WS-Man End action; no SOAP response body. |
//! | 400 | Invalid body, XML, envelope, action, destination, or event batch. |
//! | 404 | Unknown route or any query string. |
//! | 405 | Non-POST request to a recognized route; includes `Allow: POST`. |
//! | 408 | Per-request handler deadline expired, if a response can still be sent. |
//! | 413 | HTTP body limit, protocol `TooLarge`, or admission payload budget exceeded. |
//! | 415 | Content-Encoding present, invalid Content-Type, or unsupported charset. |
//! | 503 | Admission refusal, unavailable delivery queue/feedback, or downstream Nack. |
//! | 504 | Downstream feedback deadline expired. |
//!
//! Validation runs in order: memory-pressure shedding precedes routing and body
//! parsing, so it can return 503 even for an otherwise invalid request. Framework
//! throttling adds `Retry-After`; capacity refusals and downstream failures do not.
//! Event decoding errors become 400 even when the underlying event validation
//! failed a size bound; they are distinct from protocol `TooLarge` errors.
//! Error responses have empty bodies, not SOAP faults. All responses constructed
//! here include `Connection: close`; HTTP parser errors are handled by Hyper.
//!
//! # Encoding and delivery
//!
//! Exactly one `application/soap+xml` Content-Type header is required. An absent
//! charset means UTF-8; explicit UTF-8, UTF-16LE, and UTF-16BE are supported.
//! Generic UTF-16 requires a byte-order mark. Content-Encoding is rejected even
//! when its value is `identity`; no decompression occurs here.
//!
//! An event Ack may be serialized before admission, but is returned only after
//! successful delivery and bookmark resolution. Marker-only batches commit
//! locally under the same progress bounds. Heartbeats and valid SubscriptionEnd
//! notices are acknowledged without changing bookmarks; termination is
//! informational and does not cancel pending event delivery.
//!
//! # Deadlines and cancellation
//!
//! The TLS layer owns the handshake timeout. After authentication,
//! `request_timeout` bounds header reading, each handler, and the entire HTTP
//! connection, including time awaiting downstream feedback. It is not merely an
//! idle timeout. `feedback_timeout` separately bounds prepared event delivery.
//! The outer connection deadline may win either race and close the socket without
//! a 408 or 504 response. Dropping a pending delivery releases its reservations
//! without advancing progress, but cannot retract already forwarded records.

use super::{
    config::Config,
    identity::SourceIdentity,
    metrics::{ProtocolAction, RequestObservation},
    runtime::{DeliveryBridge, PreparationError},
    tls::{SourceAcceptor, accept_source, build_acceptor},
    wsman::{
        self, Encoding, Envelope, Route,
        messages::{Action, SOAP_CONTENT_TYPE, subscription::Advertisement},
    },
};
use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::{
    Request, Response, StatusCode, body::Incoming, header, server::conn::http1, service::service_fn,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use otel_arrow_dfe_otap::{otlp_http::HttpServerSettings, socket_options::apply_socket_options};
use otel_arrow_dfe_telemetry::{otel_info, otel_warn};
use std::{convert::Infallible, future::Future, io, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinSet,
    time::timeout,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Serve bounded, authenticated connections until ingress shutdown or accept failure.
///
/// Configuration must already be validated, including the single subscription and
/// nonzero limits. Advertisement and TLS setup finish before the accept loop.
/// Without a public origin, only the manager route is available and enumeration
/// advertises no subscription.
/// The caller must run on the pipeline's LocalSet; connection tasks remain local
/// and do not require Send or move work between runtime threads.
///
/// `max_sources` also caps connection tasks, including handshakes. At capacity,
/// accepting pauses rather than producing an HTTP rejection for queued sockets.
/// Normal shutdown drops the listener, cancels handshakes, and gracefully drains
/// authenticated connections within their existing deadlines. `bridge.force_stop`
/// interrupts this drain; remaining tasks are aborted and joined. An accept error
/// also aborts remaining tasks. Individual connection failures are logged and do
/// not stop the listener.
pub(super) async fn serve_manager(
    listener: TcpListener,
    config: Config,
    receiver_name: &str,
    bridge: DeliveryBridge,
    shutdown: impl Future<Output = ()>,
) -> io::Result<()> {
    let advertisement = Advertisement::from_config(&config, receiver_name)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?
        .map(Arc::new);
    let acceptor = build_acceptor(&config).await?;
    let config = Arc::new(config);
    let mut connections = JoinSet::new();
    let draining = CancellationToken::new();
    let tcp = HttpServerSettings::default();
    tokio::pin!(shutdown);
    otel_info!("wef.manager.listening", endpoint = %listener.local_addr()?, subscription_advertised = advertisement.is_some());
    let result = loop {
        tokio::select! {
            biased;
            _ = &mut shutdown => break Ok(()),
            completed = connections.join_next(), if !connections.is_empty() => {
                if let Some(Err(error)) = completed {
                    otel_warn!("wef.connection.task_failed", error = %error);
                }
            }
            accepted = listener.accept(), if connections.len() < config.limits.max_sources => {
                let (stream, peer) = match accepted {
                    Ok(connection) => connection,
                    Err(error) => break Err(error),
                };
                let stream = match apply_socket_options(stream, tcp.tcp_nodelay, tcp.tcp_keepalive, tcp.tcp_keepalive_interval, tcp.tcp_keepalive_retries) {
                    Ok(stream) => stream,
                    Err(error) => {
                        otel_warn!("wef.socket_options.failed", peer = %peer, error = %error);
                        continue;
                    }
                };
                let config = Arc::clone(&config);
                let acceptor = acceptor.clone();
                let advertisement = advertisement.clone();
                let bridge = bridge.clone();
                let draining = draining.clone();
                let _ = connections.spawn_local(async move {
                    if let Err(error) = serve_connection(stream, &acceptor, config, advertisement, bridge, draining).await {
                        otel_warn!("wef.connection.rejected", peer = %peer, error = %error);
                    }
                });
            }
        }
    };
    drop(listener);
    draining.cancel();
    if result.is_ok() {
        while !connections.is_empty() {
            tokio::select! {
                biased;
                _ = bridge.force_stop.cancelled() => break,
                _ = connections.join_next() => {}
            }
        }
    }
    connections.shutdown().await;
    result
}

/// Authenticate one source, then serve HTTP/1 under request and connection bounds.
///
/// The peer attribute is the socket IP, without its port. When advertising a
/// subscription, obtain the verified issuer thumbprint for the Windows source's
/// certificate configuration. The thumbprint does not replace TLS verification.
/// Drain cancels an unfinished handshake or requests graceful HTTP shutdown;
/// the manager remains responsible for forced task cancellation.
async fn serve_connection(
    stream: TcpStream,
    acceptor: &SourceAcceptor,
    config: Arc<Config>,
    advertisement: Option<Arc<Advertisement>>,
    bridge: DeliveryBridge,
    draining: CancellationToken,
) -> io::Result<()> {
    let peer = stream.peer_addr()?.ip().to_string();
    let (stream, identity) = tokio::select! {
        biased;
        _ = draining.cancelled() => return Ok(()),
        accepted = accept_source(stream, acceptor, &config) => accepted?,
    };
    let certificates = stream.get_ref().1.peer_certificates().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            "missing verified peer certificate",
        )
    })?;
    let issuer_thumbprint = advertisement
        .as_ref()
        .map(|_| acceptor.issuer_thumbprint(certificates))
        .transpose()?;
    otel_info!("wef.source.authenticated", source = identity.as_str());
    let request_timeout = config.limits.request_timeout;
    let service = service_fn(move |request| {
        let identity = identity.clone();
        let config = Arc::clone(&config);
        let advertisement = advertisement.clone();
        let bridge = bridge.clone();
        let peer = peer.clone();
        async move {
            let response = timeout(
                request_timeout,
                handle(
                    request,
                    &identity,
                    &config,
                    advertisement.as_deref(),
                    issuer_thumbprint.as_ref(),
                    &bridge,
                    &peer,
                ),
            )
            .await
            .unwrap_or_else(|_| status(StatusCode::REQUEST_TIMEOUT));
            Ok::<_, Infallible>(response)
        }
    });
    let mut builder = http1::Builder::new();
    let _ = builder
        .timer(TokioTimer::new())
        .header_read_timeout(request_timeout);
    timeout(request_timeout, async {
        let connection = builder.serve_connection(TokioIo::new(stream), service);
        tokio::pin!(connection);
        tokio::select! {
            result = &mut connection => result,
            _ = draining.cancelled() => {
                connection.as_mut().graceful_shutdown();
                connection.await
            }
        }
    })
    .await
    .map_err(|_| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            "manager connection deadline exceeded",
        )
    })?
    .map_err(io::Error::other)
}

/// Build an empty 503 response with a framework-supplied retry delay in seconds.
fn unavailable(retry_after_secs: u32) -> Response<Full<Bytes>> {
    let mut response = status(StatusCode::SERVICE_UNAVAILABLE);
    let _ = response
        .headers_mut()
        .insert(header::RETRY_AFTER, retry_after_secs.into());
    response
}

/// Validate and dispatch a request from an already authenticated connection.
///
/// Body collection is byte-bounded before Unicode decoding and SOAP parsing.
/// Event extraction runs through bridge admission; queueing and downstream
/// feedback must complete before returning the prebuilt Ack. The caller owns the
/// overall request deadline, while this handler applies the feedback deadline.
///
/// Manager advertisements use a snapshot of the source's committed bookmark,
/// never its pending candidate. Normally returned responses complete the request
/// observation here; cancellation drops the observation with the handler future.
async fn handle(
    request: Request<Incoming>,
    identity: &SourceIdentity,
    config: &Config,
    advertisement: Option<&Advertisement>,
    issuer_thumbprint: Option<&[u8; 20]>,
    bridge: &DeliveryBridge,
    peer: &str,
) -> Response<Full<Bytes>> {
    let mut observation = bridge.observe_request();
    let response = async {
    if let Some(retry_after_secs) = bridge.pressure_retry_after() {
        observation.refused = true;
        return unavailable(retry_after_secs);
    }
    if request.uri().query().is_some() {
        return status(StatusCode::NOT_FOUND);
    }
    let path = request.uri().path().to_owned();
    let route = match advertisement {
        Some(advertisement) => Route::parse(&path, advertisement.identifier()),
        None if path == "/wsman/SubscriptionManager/WEC" => Ok(Route::SubscriptionManager),
        None => Err(wsman::Error::UnknownRoute),
    };
    let Ok(route) = route else {
        return status(StatusCode::NOT_FOUND);
    };
    if request.method() != hyper::Method::POST {
        let mut response = status(StatusCode::METHOD_NOT_ALLOWED);
        let _ = response
            .headers_mut()
            .insert(header::ALLOW, "POST".parse().expect("static header"));
        return response;
    }
    if request.headers().contains_key(header::CONTENT_ENCODING) {
        return status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    if request
        .headers()
        .get_all(header::CONTENT_TYPE)
        .iter()
        .count()
        != 1
    {
        return status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<mime::Mime>().ok());
    let Some(content_type) =
        content_type.filter(|value| value.essence_str() == "application/soap+xml")
    else {
        return status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    };
    let collected = match Limited::new(request.into_body(), config.limits.max_request_bytes)
        .collect()
        .await
    {
        Ok(body) => body.to_bytes(),
        Err(error) => {
            return status(if error.is::<http_body_util::LengthLimitError>() {
                StatusCode::PAYLOAD_TOO_LARGE
            } else {
                StatusCode::BAD_REQUEST
            });
        }
    };
    let encoding = match content_type
        .get_param(mime::CHARSET)
        .map(|value| value.as_str().to_ascii_lowercase())
    {
        None => Encoding::Utf8,
        Some(charset) => match charset.as_str() {
            "utf-8" => Encoding::Utf8,
            "utf-16le" => Encoding::Utf16Le,
            "utf-16be" => Encoding::Utf16Be,
            "utf-16" if collected.starts_with(&[0xfe, 0xff]) => Encoding::Utf16Be,
            "utf-16" if collected.starts_with(&[0xff, 0xfe]) => Encoding::Utf16Le,
            _ => return status(StatusCode::UNSUPPORTED_MEDIA_TYPE),
        },
    };
    if let Route::Delivery { version } = route {
        let parsed = (|| {
            let xml =
                wsman::decode_body(&collected, encoding, config.limits.max_decompressed_bytes)?;
            Ok::<_, wsman::Error>(xml)
        })();
        let xml = match parsed {
            Ok(xml) => xml,
            Err(error) => return invalid_request(identity, error),
        };
        let envelope = match Envelope::parse(&xml, config.limits.max_decompressed_bytes) {
            Ok(envelope) => envelope,
            Err(error) => return invalid_request(identity, error),
        };
        observation.action = ProtocolAction::from_uri(envelope.action());
        let parsed = (|| {
            let request =
                envelope.request(&route, config.subscriptions[0].max_envelope_size_bytes)?;
            request.validate_destination(config.public_endpoint.as_deref(), &path)?;
            if request.action() == Action::End {
                return Ok((request, false));
            }
            if request
                .identifier()
                .and_then(|value| Uuid::parse_str(value).ok())
                != advertisement.map(Advertisement::identifier)
            {
                return Err(wsman::Error::InvalidEnvelope(
                    "subscription Identifier mismatch",
                ));
            }
            match request.action() {
                Action::Events => Ok((request, true)),
                Action::Heartbeat | Action::SubscriptionEnd => Ok((request, false)),
                _ => Err(wsman::Error::UnsupportedAction),
            }
        })();
        let (request, batch) = match parsed {
            Ok(parsed) => parsed,
            Err(error) => return invalid_request(identity, error),
        };
        if request.action() == Action::End {
            return status(StatusCode::NO_CONTENT);
        }
        let termination = if request.action() == Action::SubscriptionEnd {
            match request.subscription_end() {
                Ok(termination) => Some(termination),
                Err(error) => return invalid_request(identity, error),
            }
        } else {
            None
        };
        let ack = match request.encode_ack(Uuid::new_v4()) {
            Ok(ack) => ack,
            Err(error) => return invalid_request(identity, error),
        };
        if let Some(termination) = termination {
            otel_info!(
                "wef.subscription.ended",
                source = identity.as_str(),
                subscription_id = %advertisement.expect("delivery advertisement").identifier(),
                status = ?termination
            );
            return soap_response(ack);
        }
        if batch {
            let decoded = bridge.prepare(identity.clone(), collected.len(), || {
                let batch = request
                    .event_batch(&config.limits)
                    .map_err(|error| error.to_string())?;
                let records = super::event::encode_events(
                    &batch.events,
                    identity.as_str(),
                    peer,
                    &config.subscriptions[0].name,
                    advertisement.expect("delivery advertisement").identifier(),
                    version,
                    config.include_event_original,
                )?;
                Ok::<_, String>((batch.bookmark, records))
            });
            let prepared = match decoded {
                Ok(decoded) => decoded,
                Err(PreparationError::Invalid(error)) => {
                    otel_warn!("wef.events.invalid", source = identity.as_str(), error = %error);
                    return status(StatusCode::BAD_REQUEST);
                }
                Err(PreparationError::Refused(error)) => {
                    observation.refused = true;
                    otel_warn!("wef.admission.refused", source = identity.as_str(), error = %error);
                    return status(StatusCode::SERVICE_UNAVAILABLE);
                }
                Err(PreparationError::Throttled { retry_after_secs }) => {
                    observation.refused = true;
                    return unavailable(retry_after_secs);
                }
                Err(PreparationError::Oversized) => return status(StatusCode::PAYLOAD_TOO_LARGE),
            };
            let marker_only = prepared.is_marker_only();
            match timeout(config.limits.feedback_timeout, prepared.deliver()).await {
                Ok(Ok(())) => { observation.marker_only = marker_only; }
                Ok(Err(error)) => {
                    otel_warn!("wef.delivery.rejected", source = identity.as_str(), error = %error);
                    return status(StatusCode::SERVICE_UNAVAILABLE);
                }
                Err(_) => {
                    observation.feedback_timeout = true;
                    return status(StatusCode::GATEWAY_TIMEOUT);
                }
            }
        }
        otel_info!(
            "wef.delivery.accepted",
            source = identity.as_str(),
            action = request.action().uri()
        );
        return soap_response(ack);
    }
    let advertisement =
        advertisement.map(|advertisement| advertisement.with_bookmark(bridge.bookmark(identity)));
    let result = manager_response(
        &collected,
        encoding,
        config,
        advertisement.as_ref().zip(issuer_thumbprint),
        &mut observation,
    );
    match result {
        Ok((action, body)) => {
            otel_info!(
                "wef.manager.request",
                source = identity.as_str(),
                action = action.uri()
            );
            match body {
                Some(body) => soap_response(body),
                None => status(StatusCode::NO_CONTENT),
            }
        }
        Err(error) => invalid_request(identity, error),
    }
    }.await;
    observation.complete(response.status());
    response
}

/// Validate a manager envelope and encode enumeration, or accept End without a body.
///
/// Advertisement requires both subscription settings and the source certificate's
/// issuer thumbprint. Without that pair, enumeration is empty. Delivery actions
/// are not accepted on the manager route. The supplied advertisement already
/// contains the authenticated source's committed bookmark snapshot.
fn manager_response(
    body: &[u8],
    encoding: Encoding,
    config: &Config,
    advertisement: Option<(&Advertisement, &[u8; 20])>,
    observation: &mut RequestObservation,
) -> Result<(Action, Option<Vec<u8>>), wsman::Error> {
    let xml = wsman::decode_body(body, encoding, config.limits.max_decompressed_bytes)?;
    let envelope = Envelope::parse(&xml, config.limits.max_decompressed_bytes)?;
    observation.action = ProtocolAction::from_uri(envelope.action());
    let request = envelope.request(
        &Route::SubscriptionManager,
        config.subscriptions[0].max_envelope_size_bytes,
    )?;
    request.validate_destination(
        config.public_endpoint.as_deref(),
        "/wsman/SubscriptionManager/WEC",
    )?;
    match request.action() {
        Action::Enumerate => {
            let response = request.encode_enumeration(Uuid::new_v4(), advertisement)?;
            observation.advertised = advertisement.is_some();
            Ok((Action::Enumerate, Some(response)))
        }
        Action::End => Ok((Action::End, None)),
        _ => Err(wsman::Error::UnsupportedAction),
    }
}

/// Build an empty response that closes the connection, including successful 204s.
fn status(code: StatusCode) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::new()));
    *response.status_mut() = code;
    let _ = response
        .headers_mut()
        .insert(header::CONNECTION, "close".parse().expect("static header"));
    response
}

/// Return serialized SOAP as HTTP 200 with its content type and connection close.
fn soap_response(body: Vec<u8>) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::from(body)));
    let _ = response.headers_mut().insert(
        header::CONTENT_TYPE,
        SOAP_CONTENT_TYPE.parse().expect("static content type"),
    );
    let _ = response
        .headers_mut()
        .insert(header::CONNECTION, "close".parse().expect("static header"));
    response
}

/// Log a protocol rejection and map `TooLarge` to 413, all other errors to 400.
///
/// Unsupported termination elements include their expanded name in diagnostics.
/// Error details are not returned to the source, and no SOAP fault is generated.
fn invalid_request(identity: &SourceIdentity, error: wsman::Error) -> Response<Full<Bytes>> {
    if let wsman::Error::UnsupportedSubscriptionEndElement { namespace, name } = &error {
        otel_warn!(
            "wef.request.invalid",
            source = identity.as_str(),
            element_name = ?name,
            element_namespace = ?namespace,
            error = %error
        );
    } else {
        otel_warn!("wef.request.invalid", source = identity.as_str(), error = %error);
    }
    status(if matches!(error, wsman::Error::TooLarge) {
        StatusCode::PAYLOAD_TOO_LARGE
    } else {
        StatusCode::BAD_REQUEST
    })
}

#[cfg(test)]
pub(super) mod tests {
    use super::super::xml::Document;
    use super::*;
    use otel_arrow_dfe_test_tls_certs::{ExtendedKeyUsage, GeneratedCert, generate_ca};
    use rustls::{
        ClientConfig, RootCertStore,
        pki_types::{CertificateDer, PrivateKeyDer, ServerName, pem::PemObject},
    };
    use sha1::{Digest, Sha1};
    use std::{net::SocketAddr, time::Duration};
    use tokio_rustls::TlsConnector;

    pub(crate) fn connector(ca: &str, client: Option<&GeneratedCert>) -> TlsConnector {
        let mut roots = RootCertStore::empty();
        roots
            .add(CertificateDer::from_pem_slice(ca.as_bytes()).unwrap())
            .unwrap();
        let builder = ClientConfig::builder().with_root_certificates(roots);
        let config = match client {
            Some(client) => builder
                .with_client_auth_cert(
                    vec![CertificateDer::from_pem_slice(client.cert_pem.as_bytes()).unwrap()],
                    PrivateKeyDer::from_pem_slice(client.key_pem.as_bytes()).unwrap(),
                )
                .unwrap(),
            None => builder.with_no_client_auth(),
        };
        TlsConnector::from(Arc::new(config))
    }

    pub(crate) async fn exchange(
        address: SocketAddr,
        connector: &TlsConnector,
        request: Request<Full<Bytes>>,
    ) -> Result<(StatusCode, hyper::HeaderMap, Bytes), Box<dyn std::error::Error + Send + Sync>>
    {
        let stream = connector
            .connect(
                ServerName::try_from("localhost")?,
                TcpStream::connect(address).await?,
            )
            .await?;
        let (mut sender, connection) =
            hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;
        let response = async {
            let response = sender.send_request(request).await?;
            let (parts, body) = response.into_parts();
            let bytes = body.collect().await?.to_bytes();
            Ok::<_, hyper::Error>((parts.status, parts.headers, bytes))
        };
        let (response, _) = tokio::join!(response, connection);
        Ok(response?)
    }

    fn envelope(action: Action) -> Vec<u8> {
        let resource = if action == Action::End {
            "http://schemas.microsoft.com/wbem/wsman/1/wsman/FullDuplex"
        } else {
            "http://schemas.microsoft.com/wbem/wsman/1/SubscriptionManager/Subscription"
        };
        let body = if action == Action::End {
            ""
        } else {
            "<n:Enumerate><w:OptimizeEnumeration/><w:MaxElements>100</w:MaxElements></n:Enumerate>"
        };
        let xml = format!(
            r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
            xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing"
            xmlns:w="http://schemas.dmtf.org/wbem/wsman/1/wsman.xsd"
            xmlns:n="http://schemas.xmlsoap.org/ws/2004/09/enumeration">
            <s:Header><a:Action>{}</a:Action><a:MessageID>uuid:ABC-123</a:MessageID>
            <a:To>https://localhost/wsman/SubscriptionManager/WEC</a:To>
            <w:ResourceURI>{resource}</w:ResourceURI>
            <w:MachineID>spoofed.example.com</w:MachineID></s:Header>
            <s:Body>{body}</s:Body></s:Envelope>"#,
            action.uri()
        );
        [
            vec![0xff, 0xfe],
            xml.encode_utf16().flat_map(u16::to_le_bytes).collect(),
        ]
        .concat()
    }

    pub(crate) fn request(
        method: &str,
        path: &str,
        content_type: &str,
        body: Vec<u8>,
    ) -> Request<Full<Bytes>> {
        Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, "localhost")
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONNECTION, "close")
            .body(Full::new(Bytes::from(body)))
            .unwrap()
    }

    /// Scenario: authenticated TCP clients enumerate, deliver, and report source termination.
    /// Guarantees: identity and destination checks gate replies; termination preserves pending feedback and bookmarks.
    #[tokio::test]
    async fn authenticated_manager_over_tcp() {
        tokio::task::LocalSet::new()
            .run_until(run_authenticated_manager_over_tcp())
            .await;
    }

    async fn run_authenticated_manager_over_tcp() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let directory = tempfile::tempdir().unwrap();
        let ca = generate_ca("manager test CA");
        ca.write_cert_to_dir(directory.path(), "ca");
        ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        )
        .write_to_dir(directory.path(), "server");
        let config: Config = serde_json::from_value(serde_json::json!({
            "endpoint": "127.0.0.1:0",
            "tls": {"cert_file": directory.path().join("server.crt"),
                "key_file": directory.path().join("server.key"), "client_ca_files": [directory.path().join("ca.crt")]},
            "auth": {"allowed_sources": ["host.example.com"]},
            "subscriptions": [{"name": "test", "query": "<QueryList/>"}],
            "limits": {"max_request_bytes": 4096, "request_timeout": "2s"}
        })).unwrap();
        let trusted = ca.issue_leaf(
            "ignored-cn",
            Some("HOST.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let unauthorized = ca.issue_leaf(
            "host.example.com",
            Some("other.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let wrong_ca = generate_ca("untrusted CA");
        let untrusted = wrong_ca.issue_leaf(
            "host",
            Some("host.example.com"),
            Some(ExtendedKeyUsage::ClientAuth),
        );
        let listener = TcpListener::bind(config.endpoint).await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let mut advertised_config = config.clone();
        advertised_config.public_endpoint = Some("https://localhost".into());
        let (bridge, _deliveries) = DeliveryBridge::new(&config.limits);
        let metrics = super::super::metrics::tests::registered();
        let bridge = bridge.with_metrics(Arc::clone(&metrics));
        let server =
            tokio::task::spawn_local(serve_manager(listener, config, "test", bridge, async {
                let _ = stopped.await;
            }));
        let path = "/wsman/SubscriptionManager/WEC";
        let trusted_connector = connector(&ca.cert_pem, Some(&trusted));
        for client in [None, Some(&untrusted), Some(&unauthorized)] {
            let result = timeout(
                Duration::from_secs(5),
                exchange(
                    address,
                    &connector(&ca.cert_pem, client),
                    request("POST", path, SOAP_CONTENT_TYPE, envelope(Action::Enumerate)),
                ),
            )
            .await
            .unwrap();
            assert!(
                result.is_err(),
                "unauthenticated or unauthorized client reached HTTP"
            );
        }
        let (code, headers, bytes) = timeout(
            Duration::from_secs(5),
            exchange(
                address,
                &trusted_connector,
                request("POST", path, SOAP_CONTENT_TYPE, envelope(Action::Enumerate)),
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(code, StatusCode::OK);
        assert_eq!(headers[header::CONTENT_TYPE], SOAP_CONTENT_TYPE);
        let xml = wsman::decode_body(&bytes, Encoding::Utf16Le, 8192).unwrap();
        let response = Envelope::parse(&xml, 8192).unwrap();
        assert_eq!(
            response.action(),
            "http://schemas.xmlsoap.org/ws/2004/09/enumeration/EnumerateResponse"
        );
        let document = Document::parse(&xml).unwrap();
        assert!(document.descendants().any(|node| node.has_tag_name((
            "http://schemas.xmlsoap.org/ws/2004/08/addressing",
            "RelatesTo"
        )) && node.text() == Some("uuid:ABC-123")));
        assert!(document.descendants().any(|node| node.has_tag_name((
            "http://schemas.dmtf.org/wbem/wsman/1/wsman.xsd",
            "EndOfSequence"
        ))));
        assert!(
            !document
                .descendants()
                .any(|node| node.tag_name().name() == "Subscribe")
        );
        for (method, target, content_type, body, expected) in [
            (
                "POST",
                path,
                SOAP_CONTENT_TYPE,
                envelope(Action::End),
                StatusCode::NO_CONTENT,
            ),
            (
                "GET",
                path,
                SOAP_CONTENT_TYPE,
                vec![],
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            (
                "POST",
                "/wsman/subscriptions/unused/version",
                SOAP_CONTENT_TYPE,
                vec![],
                StatusCode::NOT_FOUND,
            ),
            (
                "POST",
                path,
                "text/plain",
                vec![],
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ),
            (
                "POST",
                path,
                SOAP_CONTENT_TYPE,
                vec![0; 4097],
                StatusCode::PAYLOAD_TOO_LARGE,
            ),
            (
                "POST",
                path,
                "application/soap+xml;charset=utf-8",
                b"invalid XML".to_vec(),
                StatusCode::BAD_REQUEST,
            ),
        ] {
            let (code, _, _) = timeout(
                Duration::from_secs(5),
                exchange(
                    address,
                    &trusted_connector,
                    request(method, target, content_type, body),
                ),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(code, expected);
        }
        stop.send(()).unwrap();
        timeout(Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        super::super::metrics::tests::assert_counts(
            &metrics,
            &[
                ("enumerate", "success", 1),
                ("end", "success", 1),
                ("other", "refused", 5),
            ],
            [0, 0, 0],
        );

        let advertisement = Advertisement::from_config(&advertised_config, "test")
            .unwrap()
            .unwrap();
        let listener = TcpListener::bind(advertised_config.endpoint).await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, stopped) = tokio::sync::oneshot::channel();
        advertised_config.limits.feedback_timeout = Duration::from_millis(200);
        let (bridge, mut deliveries) = DeliveryBridge::new(&advertised_config.limits);
        let bridge = bridge.with_metrics(Arc::clone(&metrics));
        let observed_bridge = bridge.clone();
        let source = SourceIdentity::from_verified_certificate(
            CertificateDer::from_pem_slice(trusted.cert_pem.as_bytes())
                .unwrap()
                .as_ref(),
            &advertised_config.auth,
        )
        .unwrap();
        let server = tokio::task::spawn_local(serve_manager(
            listener,
            advertised_config,
            "test",
            bridge,
            async {
                let _ = stopped.await;
            },
        ));
        for action in [Action::Enumerate, Action::End] {
            for destination in [
                "https://elsewhere/wsman/SubscriptionManager/WEC",
                "https://localhost/wrong-path",
            ] {
                let xml = wsman::decode_body(&envelope(action), Encoding::Utf16Le, 8192)
                    .unwrap()
                    .replace(
                        "https://localhost/wsman/SubscriptionManager/WEC",
                        destination,
                    );
                let (code, _, body) = timeout(
                    Duration::from_secs(5),
                    exchange(
                        address,
                        &trusted_connector,
                        request(
                            "POST",
                            path,
                            "application/soap+xml;charset=utf-8",
                            xml.into_bytes(),
                        ),
                    ),
                )
                .await
                .unwrap()
                .unwrap();
                assert_eq!(code, StatusCode::BAD_REQUEST);
                assert!(body.is_empty());
            }
        }
        let (code, _, bytes) = timeout(
            Duration::from_secs(5),
            exchange(
                address,
                &trusted_connector,
                request("POST", path, SOAP_CONTENT_TYPE, envelope(Action::Enumerate)),
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(code, StatusCode::OK);
        let xml = wsman::decode_body(&bytes, Encoding::Utf16Le, 16384).unwrap();
        let document = Document::parse(&xml).unwrap();
        let certificate = CertificateDer::from_pem_slice(ca.cert_pem.as_bytes()).unwrap();
        let expected_thumbprint: String = Sha1::digest(certificate.as_ref())
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect();
        let thumbprint = document
            .descendants()
            .find(|node| {
                node.has_tag_name((
                    "http://schemas.microsoft.com/wbem/wsman/1/authentication",
                    "Thumbprint",
                ))
            })
            .unwrap();
        assert_eq!(thumbprint.attribute("Role"), Some("issuer"));
        assert_eq!(thumbprint.text(), Some(expected_thumbprint.as_str()));
        assert!(document.descendants().any(|node| node.has_tag_name((
            "http://schemas.xmlsoap.org/ws/2004/08/eventing",
            "Subscribe"
        ))));
        assert!(
            document
                .descendants()
                .any(|node| node.text() == Some(advertisement.delivery_url()))
        );
        let delivery_path = format!(
            "/wsman/subscriptions/{}/opaque-version",
            advertisement.identifier()
        );
        let (code, _, bytes) = timeout(
            Duration::from_secs(5),
            exchange(
                address,
                &trusted_connector,
                request("POST", &delivery_path, SOAP_CONTENT_TYPE, vec![]),
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(code, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert!(bytes.is_empty());
        let content_type = "application/soap+xml;charset=utf-8";
        let ordinary_event = r#"<w:Event Action="http://schemas.dmtf.org/wbem/wsman/1/wsman/Event"><![CDATA[<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event"><System><EventID>42</EventID><Level>4</Level><TimeCreated SystemTime="2026-09-22T19:28:11Z"/><Channel>Application</Channel><Computer>spoofed</Computer></System><RenderingInfo><Message>test message</Message></RenderingInfo></Event>]]></w:Event>"#;
        let bookmark_event = r#"<w:Event Action="http://schemas.dmtf.org/wbem/wsman/1/wsman/Event"><![CDATA[<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event"><System><Provider Name="Microsoft-Windows-EventForwarder"/><EventID>111</EventID><TimeCreated SystemTime="2026-09-22T19:44:13.247Z"/><Computer>host</Computer></System><SubscriptionBookmarkEvent><SubscriptionId/></SubscriptionBookmarkEvent></Event>]]></w:Event>"#;
        let mixed_events = format!("{bookmark_event}{ordinary_event}");
        let make_delivery = |action: Action, cursor: &str, events: &str| {
            let bookmark = if action == Action::Events {
                format!(
                    r#"<w:Bookmark s:mustUnderstand="true"><b:Cursor>{cursor}</b:Cursor></w:Bookmark>"#
                )
            } else {
                String::new()
            };
            format!(r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing" xmlns:w="http://schemas.dmtf.org/wbem/wsman/1/wsman.xsd" xmlns:e="http://schemas.xmlsoap.org/ws/2004/08/eventing" xmlns:b="urn:bookmark"><s:Header><a:Action>{}</a:Action><a:MessageID>uuid:delivery</a:MessageID><a:To>https://localhost{delivery_path}</a:To><e:Identifier>{}</e:Identifier><w:AckRequested/>{bookmark}</s:Header><s:Body><w:Events>{events}</w:Events></s:Body></s:Envelope>"#, action.uri(), advertisement.identifier()).into_bytes()
        };
        let make_termination = || {
            String::from_utf8(make_delivery(Action::SubscriptionEnd, "", ""))
                .unwrap()
                .replace(
                    "<w:Events></w:Events>",
                    "<e:SubscriptionEnd><e:SubscriptionManager><a:Address>https://source.example/wsman</a:Address><a:ReferenceProperties><e:Identifier>source-side-id</e:Identifier></a:ReferenceProperties></e:SubscriptionManager><e:Status>http://schemas.xmlsoap.org/ws/2004/08/eventing/SourceShuttingDown</e:Status><e:Reason xml:lang=\"en-US\">Stopping</e:Reason></e:SubscriptionEnd>",
                )
        };
        let (code, _, body) = exchange(
            address,
            &trusted_connector,
            request(
                "POST",
                &delivery_path,
                content_type,
                make_delivery(Action::Heartbeat, "", ""),
            ),
        )
        .await
        .unwrap();
        assert_eq!(code, StatusCode::OK);
        assert!(
            wsman::decode_body(&body, Encoding::Utf16Le, 8192)
                .unwrap()
                .contains("/wsman/Ack")
        );
        assert!(deliveries.try_recv().is_err());
        for (cursor, outcome) in [
            ("accepted", Some(super::super::bookmark::BatchOutcome::Ack)),
            ("rejected", Some(super::super::bookmark::BatchOutcome::Nack)),
            ("timeout", None),
        ] {
            let mut response = Box::pin(exchange(
                address,
                &trusted_connector,
                request(
                    "POST",
                    &delivery_path,
                    content_type,
                    make_delivery(Action::Events, cursor, &mixed_events),
                ),
            ));
            let delivery = tokio::select! {
                queued = deliveries.recv() => queued.unwrap(),
                response = &mut response => panic!("response preceded downstream feedback: {response:?}"),
            };
            assert!(
                timeout(Duration::from_millis(10), &mut response)
                    .await
                    .is_err()
            );
            assert_eq!(delivery.records.get(otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs).unwrap().num_rows(), 1);
            let bookmark_before = observed_bridge.bookmark(&source);
            for termination in [
                make_termination(),
                make_termination()
                    .replace("SourceShuttingDown", "DeliveryFailure")
                    .replace(
                        "</e:SubscriptionEnd>",
                        "<f:WSManFault xmlns:f=\"http://schemas.microsoft.com/wbem/wsman/1/wsmanfault\" Code=\"2150859027\" Machine=\"untrusted-machine\"><f:Message>Delivery failed</f:Message></f:WSManFault></e:SubscriptionEnd>",
                    ),
            ] {
            let (code, _, body) = timeout(
                Duration::from_secs(2),
                exchange(
                    address,
                    &trusted_connector,
                    request(
                        "POST",
                        &delivery_path,
                        content_type,
                        termination.into_bytes(),
                    ),
                ),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(code, StatusCode::OK);
            let reply = wsman::decode_body(&body, Encoding::Utf16Le, 8192).unwrap();
            assert!(reply.contains("/wsman/Ack"));
            assert!(reply.contains("uuid:delivery"));
            assert_eq!(observed_bridge.bookmark(&source), bookmark_before);
            assert!(deliveries.try_recv().is_err());
            assert!(
                timeout(Duration::from_millis(10), &mut response)
                    .await
                    .is_err()
            );
            }
            if cursor == "accepted" {
                let (code, _, _) = exchange(
                    address,
                    &trusted_connector,
                    request(
                        "POST",
                        &delivery_path,
                        content_type,
                        make_delivery(Action::Events, "overtaking", bookmark_event),
                    ),
                )
                .await
                .unwrap();
                assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
            }
            if let Some(outcome) = outcome {
                delivery.feedback.send(outcome).unwrap();
            }
            let (code, _, body) = timeout(Duration::from_secs(2), response)
                .await
                .unwrap()
                .unwrap();
            match outcome {
                Some(super::super::bookmark::BatchOutcome::Ack) => {
                    assert_eq!(code, StatusCode::OK);
                    assert!(
                        wsman::decode_body(&body, Encoding::Utf16Le, 8192)
                            .unwrap()
                            .contains("uuid:delivery")
                    );
                }
                Some(super::super::bookmark::BatchOutcome::Nack) => {
                    assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE)
                }
                None => assert_eq!(code, StatusCode::GATEWAY_TIMEOUT),
            }
            let (code, _, bytes) = exchange(
                address,
                &trusted_connector,
                request("POST", path, SOAP_CONTENT_TYPE, envelope(Action::Enumerate)),
            )
            .await
            .unwrap();
            assert_eq!(code, StatusCode::OK);
            let xml = wsman::decode_body(&bytes, Encoding::Utf16Le, 16384).unwrap();
            let document = Document::parse(&xml).unwrap();
            assert!(
                document
                    .descendants()
                    .any(|node| node.has_tag_name(("urn:bookmark", "Cursor"))
                        && node.text() == Some("accepted"))
            );
        }
        let (code, _, body) = exchange(
            address,
            &trusted_connector,
            request(
                "POST",
                &delivery_path,
                content_type,
                make_delivery(Action::Events, "marker-only", bookmark_event),
            ),
        )
        .await
        .unwrap();
        assert_eq!(code, StatusCode::OK);
        assert!(
            wsman::decode_body(&body, Encoding::Utf16Le, 8192)
                .unwrap()
                .contains("/wsman/Ack")
        );
        assert!(deliveries.try_recv().is_err());
        let (code, _, bytes) = exchange(
            address,
            &trusted_connector,
            request("POST", path, SOAP_CONTENT_TYPE, envelope(Action::Enumerate)),
        )
        .await
        .unwrap();
        assert_eq!(code, StatusCode::OK);
        let xml = wsman::decode_body(&bytes, Encoding::Utf16Le, 16384).unwrap();
        let document = Document::parse(&xml).unwrap();
        assert!(
            document
                .descendants()
                .any(|node| node.has_tag_name(("urn:bookmark", "Cursor"))
                    && node.text() == Some("marker-only"))
        );
        for action in [Action::Events, Action::Heartbeat] {
            for destination in [
                format!("https://elsewhere{delivery_path}"),
                format!("https://localhost{delivery_path}-different-version"),
            ] {
                let events = if action == Action::Events {
                    ordinary_event
                } else {
                    ""
                };
                let xml = String::from_utf8(make_delivery(action, "wrong-destination", events))
                    .unwrap()
                    .replace(&format!("https://localhost{delivery_path}"), &destination);
                let (code, _, body) = timeout(
                    Duration::from_secs(5),
                    exchange(
                        address,
                        &trusted_connector,
                        request("POST", &delivery_path, content_type, xml.into_bytes()),
                    ),
                )
                .await
                .unwrap()
                .unwrap();
                assert_eq!(code, StatusCode::BAD_REQUEST);
                assert!(body.is_empty());
                assert!(deliveries.try_recv().is_err());
            }
        }
        let (code, _, bytes) = exchange(
            address,
            &trusted_connector,
            request("POST", path, SOAP_CONTENT_TYPE, envelope(Action::Enumerate)),
        )
        .await
        .unwrap();
        assert_eq!(code, StatusCode::OK);
        let xml = wsman::decode_body(&bytes, Encoding::Utf16Le, 16384).unwrap();
        assert!(xml.contains("marker-only"));
        assert!(!xml.contains("wrong-destination"));
        let invalid_identifier =
            String::from_utf8(make_delivery(Action::Events, "invalid", &mixed_events))
                .unwrap()
                .replace(
                    &advertisement.identifier().to_string(),
                    &Uuid::nil().to_string(),
                )
                .into_bytes();
        let (code, _, _) = exchange(
            address,
            &trusted_connector,
            request("POST", &delivery_path, content_type, invalid_identifier),
        )
        .await
        .unwrap();
        assert_eq!(code, StatusCode::BAD_REQUEST);
        assert!(deliveries.try_recv().is_err());
        let bookmark_before = observed_bridge.bookmark(&source);
        for invalid in [
            make_termination()
                .replace("<e:Status>", "<e:MissingStatus>")
                .replace("</e:Status>", "</e:MissingStatus>"),
            make_termination().replace(
                &format!(
                    "<e:Identifier>{}</e:Identifier>",
                    advertisement.identifier()
                ),
                &format!("<e:Identifier>{}</e:Identifier>", Uuid::nil()),
            ),
            make_termination().replace(
                &format!("https://localhost{delivery_path}"),
                "https://elsewhere/wrong",
            ),
            make_termination().replace("<w:AckRequested/>", ""),
        ] {
            let (code, _, body) = exchange(
                address,
                &trusted_connector,
                request("POST", &delivery_path, content_type, invalid.into_bytes()),
            )
            .await
            .unwrap();
            assert_eq!(code, StatusCode::BAD_REQUEST);
            assert!(body.is_empty());
            assert!(deliveries.try_recv().is_err());
            assert_eq!(observed_bridge.bookmark(&source), bookmark_before);
        }
        let (code, _, _) = exchange(
            address,
            &trusted_connector,
            request("POST", path, content_type, make_termination().into_bytes()),
        )
        .await
        .unwrap();
        assert_eq!(code, StatusCode::BAD_REQUEST);
        stop.send(()).unwrap();
        timeout(Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        super::super::metrics::tests::assert_counts(
            &metrics,
            &[
                ("enumerate", "success", 6),
                ("enumerate", "refused", 2),
                ("end", "refused", 2),
                ("other", "refused", 1),
                ("heartbeat", "success", 1),
                ("heartbeat", "refused", 2),
                ("events", "success", 2),
                ("events", "failure", 2),
                ("events", "refused", 4),
                ("subscription_end", "success", 6),
                ("subscription_end", "refused", 5),
            ],
            [6, 1, 1],
        );
    }
}
