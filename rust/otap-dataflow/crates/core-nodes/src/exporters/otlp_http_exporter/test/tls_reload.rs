// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::io::Read;
use std::path::Path;

use futures::FutureExt;
use http_body_util::BodyExt;
use otel_arrow_dfe_engine::control::pipeline_completion_msg_channel;
use otel_arrow_dfe_engine::message::Receiver;
use otel_arrow_dfe_engine::shared::message::SharedReceiver;
use otel_arrow_dfe_test_tls_certs::GeneratedCa;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;

const POOL_SIZE: NonZeroUsize = NonZeroUsize::new(2).unwrap();

pub(super) struct RunningExporter {
    pdata: mpsc::Sender<OtapPdata>,
    control: mpsc::Sender<NodeControlMsg<OtapPdata>>,
    pub(super) completions:
        otel_arrow_dfe_engine::control::PipelineCompletionMsgReceiver<OtapPdata>,
    task: tokio::task::JoinHandle<Result<TerminalState, EngineError>>,
}

impl RunningExporter {
    pub(super) fn start(config: Config, pipeline: &PipelineContext) -> Self {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let exporter = Box::new(OtlpHttpExporter {
            config,
            metrics: OtlpHttpExporterMetrics::register(pipeline, None),
            auth_provider: None,
        });
        let (completion_tx, completions) = pipeline_completion_msg_channel(10);
        let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut effects = EffectHandler::new(
            test_node("tls-test"),
            reporter,
            otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
        );
        effects.set_pipeline_completion_msg_sender(completion_tx);
        let (control, control_rx) = mpsc::channel(10);
        let (pdata, pdata_rx) = mpsc::channel(10);
        let inbox = ExporterInbox::new(
            Receiver::Shared(SharedReceiver::mpsc(control_rx)),
            Receiver::Shared(SharedReceiver::mpsc(pdata_rx)),
            0,
            Interests::empty(),
        );
        let task = tokio::task::spawn_local(exporter.start(inbox, effects));
        Self {
            pdata,
            control,
            completions,
            task,
        }
    }

    pub(super) async fn send(&self, pdata: OtapPdata) {
        self.pdata.send(pdata).await.unwrap();
    }

    pub(super) async fn stop(self) {
        self.control
            .send(NodeControlMsg::Shutdown {
                deadline: Instant::now() + Duration::from_secs(1),
                reason: "test complete".into(),
            })
            .await
            .unwrap();
        let _terminal = timeout(Duration::from_secs(5), self.task)
            .await
            .expect("exporter shutdown deadline")
            .expect("exporter task")
            .expect("exporter shutdown");
    }
}

fn write_material(path: &Path, ca: &GeneratedCa, name: &str) {
    ca.write_cert_to_dir(path, "ca");
    ca.issue_leaf(name, None, Some(ExtendedKeyUsage::ClientAuth))
        .write_to_dir(path, "client");
}

struct TlsFixture {
    dir: TempDir,
    ca: GeneratedCa,
    settings: HttpClientSettings,
    provider: Rc<ClientTlsProvider>,
}

impl TlsFixture {
    async fn new() -> Self {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temporary directory");
        let ca = generate_ca("HTTP reload CA");
        write_material(dir.path(), &ca, "client-0");
        let settings = HttpClientSettings {
            tls: Some(TlsClientConfig {
                config: TlsConfig {
                    cert_file: Some(dir.path().join("client.crt")),
                    key_file: Some(dir.path().join("client.key")),
                    reload_interval: Some(Duration::from_millis(10)),
                    ..Default::default()
                },
                ca_file: Some(dir.path().join("ca.crt")),
                include_system_ca_certs_pool: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        };
        let provider = Rc::new(
            ClientTlsProvider::new_http(settings.tls.as_ref())
                .await
                .expect("load material")
                .expect("TLS provider"),
        );
        Self {
            dir,
            ca,
            settings,
            provider,
        }
    }

    async fn pool(&self) -> HttpClientPool {
        HttpClientPool::try_new(&self.settings, POOL_SIZE, Some(self.provider.current()))
            .await
            .expect("initial client pool")
    }

    async fn rotate(&self, name: &str) -> Arc<ClientTlsGeneration> {
        let mut updates = self.provider.subscribe();
        write_material(self.dir.path(), &self.ca, name);
        timeout(Duration::from_secs(5), async {
            tokio::select! {
                result = updates.changed() => result.expect("generation publication"),
                () = self.provider.run(std::future::pending()) => {
                    panic!("reload task unexpectedly stopped");
                }
            }
        })
        .await
        .expect("material reload deadline");
        self.provider.current()
    }

    fn failing_settings(&self) -> HttpClientSettings {
        HttpClientSettings {
            user_agent: Some("invalid\nuser-agent".into()),
            ..self.settings.clone()
        }
    }
}

/// Scenario: installing one HTTP TLS generation fails repeatedly and is retried
/// after the builder error is removed, without another material publication.
/// Guarantees: failure preserves the active pool and round-robin position,
/// retries back off at 5/10/20/40/60 seconds, and success clears pending state.
#[tokio::test]
async fn failed_installation_retries_without_another_publication() {
    let fixture = TlsFixture::new().await;
    let mut pool = fixture.pool().await;
    drop(pool.get_client());
    let generation = fixture.rotate("client-1").await;
    let updates = fixture.provider.subscribe();
    let mut installation = TlsPoolInstallation::new();
    for seconds in [5, 10, 20, 40, 60, 60] {
        let before = tokio::time::Instant::now();
        installation
            .install(
                &fixture.failing_settings(),
                POOL_SIZE,
                generation.clone(),
                &mut pool,
            )
            .await;
        let after = tokio::time::Instant::now();
        let delay = Duration::from_secs(seconds);
        assert!(installation.retry_at >= before + delay);
        assert!(installation.retry_at <= after + delay);
        assert_eq!(pool.generation, 0);
        assert_eq!(pool.pool.len(), POOL_SIZE.get());
        assert_eq!(pool.next_client, 1);
        assert!(Arc::ptr_eq(
            installation.pending.as_ref().unwrap(),
            &generation
        ));
        assert!(
            tokio::time::sleep_until(installation.retry_at)
                .now_or_never()
                .is_none()
        );
    }
    installation
        .install(&fixture.settings, POOL_SIZE, generation.clone(), &mut pool)
        .await;
    assert_eq!(pool.generation, generation.number());
    assert_eq!(pool.next_client, 0);
    assert!(installation.pending.is_none());
    assert_eq!(installation.retry_delay, TLS_INSTALL_RETRY_MIN);
    assert!(!updates.has_changed().expect("live generation subscription"));
}

/// Scenario: a new TLS generation arrives while an older HTTP installation
/// candidate is backed off, followed by many successful rotations.
/// Guarantees: the latest candidate installs without waiting for the old
/// deadline, only the configured number of clients is retained, and obsolete
/// snapshots are released instead of accumulating historical generations.
#[tokio::test]
async fn newer_generations_supersede_failed_candidates_and_remain_bounded() {
    let fixture = TlsFixture::new().await;
    let mut pool = fixture.pool().await;
    let generation = fixture.rotate("client-1").await;
    let retired = Arc::downgrade(&generation);
    let mut installation = TlsPoolInstallation::new();
    installation
        .install(
            &fixture.failing_settings(),
            POOL_SIZE,
            generation,
            &mut pool,
        )
        .await;
    let deadline = installation.retry_at;
    for index in 2..18 {
        let generation = fixture.rotate(&format!("client-{index}")).await;
        installation
            .install(&fixture.settings, POOL_SIZE, generation, &mut pool)
            .await;
        assert_eq!(pool.generation, index);
        assert_eq!(pool.pool.len(), POOL_SIZE.get());
        assert!(installation.pending.is_none());
        assert!(retired.upgrade().is_none());
        if index == 2 {
            assert!(tokio::time::Instant::now() < deadline);
        }
    }
}

/// Scenario: shutdown or exporter replacement interrupts reload work while an
/// HTTP pool installation is waiting for its next retry.
/// Guarantees: shutdown does not wait for backoff, pending snapshots are freed,
/// and repeated task replacement releases all provider references.
#[tokio::test]
async fn shutdown_and_replacement_release_reload_and_retry_state() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let fixture = TlsFixture::new().await;
            let mut pool = fixture.pool().await;
            let generation = fixture.rotate("client-1").await;
            let retired = Arc::downgrade(&generation);
            let mut installation = TlsPoolInstallation::new();
            installation
                .install(
                    &fixture.failing_settings(),
                    POOL_SIZE,
                    generation,
                    &mut pool,
                )
                .await;
            let mut task = LocalTlsReloadTask::start(fixture.provider.clone());
            timeout(Duration::from_secs(1), task.stop())
                .await
                .expect("shutdown deadline")
                .expect("reload task stopped");
            assert!(tokio::time::Instant::now() < installation.retry_at);
            assert!(task.handle.is_none());
            drop(installation);
            let _generation = fixture.rotate("client-2").await;
            assert!(retired.upgrade().is_none());
            for _ in 0..12 {
                let task = LocalTlsReloadTask::start(fixture.provider.clone());
                tokio::task::yield_now().await;
                assert_eq!(Rc::strong_count(&fixture.provider), 2);
                drop(task);
                timeout(Duration::from_secs(1), async {
                    while Rc::strong_count(&fixture.provider) != 1 {
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .expect("replacement releases provider");
            }
        })
        .await;
}

struct HeldRequest {
    identity: Vec<u8>,
    handshake: Option<rustls::HandshakeKind>,
    headers: HeaderMap,
    path: String,
    body: Bytes,
    response: oneshot::Sender<StatusCode>,
}

struct TestServer {
    endpoint: String,
    requests: mpsc::Receiver<HeldRequest>,
    closed: mpsc::Receiver<Vec<u8>>,
    config: watch::Sender<Arc<rustls::ServerConfig>>,
    cancel: CancellationToken,
    task: tokio::task::JoinHandle<()>,
}

fn server_config(ca: &GeneratedCa) -> Arc<rustls::ServerConfig> {
    let server = ca.issue_leaf(
        "server",
        Some("localhost"),
        Some(ExtendedKeyUsage::ServerAuth),
    );
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from_pem_slice(ca.cert_pem.as_bytes()).unwrap())
        .unwrap();
    let verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .unwrap();
    Arc::new(
        rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS12])
            .with_client_cert_verifier(verifier)
            .with_single_cert(
                vec![CertificateDer::from_pem_slice(server.cert_pem.as_bytes()).unwrap()],
                PrivateKeyDer::from_pem_slice(server.key_pem.as_bytes()).unwrap(),
            )
            .unwrap(),
    )
}

impl TestServer {
    async fn start(ca: &GeneratedCa) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!(
            "https://localhost:{}",
            listener.local_addr().unwrap().port()
        );
        let (requests_tx, requests) = mpsc::channel(32);
        let (closed_tx, closed) = mpsc::channel(32);
        let (config, current) = watch::channel(server_config(ca));
        let cancel = CancellationToken::new();
        let stopped = cancel.clone();
        // The backend runs concurrently with the single-core exporter under test.
        let task = tokio::spawn(async move {
            let tracker = TaskTracker::new();
            loop {
                let (stream, _) = tokio::select! {
                    () = stopped.cancelled() => break,
                    accepted = listener.accept() => accepted.unwrap(),
                };
                let acceptor = TlsAcceptor::from(current.borrow().clone());
                let requests = requests_tx.clone();
                let closed = closed_tx.clone();
                let stopped = stopped.clone();
                drop(tracker.spawn(async move {
                    let stream = tokio::select! {
                        () = stopped.cancelled() => return,
                        stream = acceptor.accept(stream) => match stream {
                            Ok(stream) => stream,
                            Err(error) => {
                                otel_warn!("http.test.tls_handshake_rejected", error = %error);
                                return;
                            }
                        },
                    };
                    let identity = stream.get_ref().1.peer_certificates().unwrap()[0]
                        .as_ref().to_vec();
                    let handshake = stream.get_ref().1.handshake_kind();
                    let closed_identity = identity.clone();
                    let service = service_fn(move |request: hyper::Request<hyper::body::Incoming>| {
                        let requests = requests.clone();
                        let identity = identity.clone();
                        async move {
                            let (parts, body) = request.into_parts();
                            let body = body.collect().await.unwrap().to_bytes();
                            let (response, result) = oneshot::channel();
                            requests.send(HeldRequest {
                                identity, handshake, headers: parts.headers,
                                path: parts.uri.path().into(), body, response,
                            }).await.unwrap();
                            let status = result.await.expect("controlled HTTP response");
                            Ok::<_, std::convert::Infallible>(
                                Response::builder().status(status)
                                    .body(Full::new(Bytes::new())).unwrap()
                            )
                        }
                    });
                    tokio::select! {
                        () = stopped.cancelled() => {},
                        result = http1::Builder::new().serve_connection(TokioIo::new(stream), service) => {
                            result.expect("HTTP connection");
                        }
                    }
                    closed.send(closed_identity).await.expect("connection observer");
                }));
            }
            let _ = tracker.close();
            tracker.wait().await;
        });
        Self {
            endpoint,
            requests,
            closed,
            config,
            cancel,
            task,
        }
    }

    async fn request(&mut self) -> HeldRequest {
        timeout(Duration::from_secs(5), self.requests.recv())
            .await
            .expect("request deadline")
            .expect("live backend")
    }

    async fn stop(self) {
        self.cancel.cancel();
        timeout(Duration::from_secs(5), self.task)
            .await
            .expect("server shutdown deadline")
            .expect("backend task");
    }
}

/// Scenario: a client populates its TLS session cache, then a rotated pool
/// connects to the same backend while the backend retains old sessions.
/// Guarantees: the replacement performs a full handshake with the new mTLS
/// identity, and dropping the retired pool closes its idle connection.
#[tokio::test]
async fn replacement_does_not_reuse_retired_tls_sessions_or_connections() {
    let fixture = TlsFixture::new().await;
    let mut server = TestServer::start(&fixture.ca).await;
    let mut pool = fixture.pool().await;
    let old = pool.get_client();
    let endpoint = server.endpoint.clone();
    let send = tokio::spawn(async move {
        old.post(endpoint)
            .header(header::CONNECTION, "close")
            .send()
            .await
            .unwrap()
    });
    let first = server.request().await;
    let first_identity = first.identity;
    assert_eq!(first.handshake, Some(rustls::HandshakeKind::Full));
    first.response.send(StatusCode::OK).unwrap();
    drop(send.await.unwrap());
    let _ = timeout(Duration::from_secs(5), server.closed.recv())
        .await
        .unwrap()
        .unwrap();

    // Round-robin back to the same client to exercise its populated cache.
    drop(pool.get_client());
    let old = pool.get_client();
    let endpoint = server.endpoint.clone();
    let send = tokio::spawn(async move { old.post(endpoint).send().await.unwrap() });
    let resumed = server.request().await;
    assert_eq!(resumed.handshake, Some(rustls::HandshakeKind::Resumed));
    assert_eq!(resumed.identity, first_identity);
    resumed.response.send(StatusCode::OK).unwrap();
    drop(send.await.unwrap());

    let generation = fixture.rotate("client-1").await;
    TlsPoolInstallation::new()
        .install(&fixture.settings, POOL_SIZE, generation, &mut pool)
        .await;
    let old_closed = timeout(Duration::from_secs(5), server.closed.recv())
        .await
        .expect("retired idle connection deadline")
        .unwrap();
    assert_eq!(old_closed, first_identity);
    let new = pool.get_client();
    let endpoint = server.endpoint.clone();
    let send = tokio::spawn(async move { new.post(endpoint).send().await.unwrap() });
    let rotated = server.request().await;
    assert_eq!(rotated.handshake, Some(rustls::HandshakeKind::Full));
    assert_ne!(rotated.identity, first_identity);
    rotated.response.send(StatusCode::OK).unwrap();
    drop(send.await.unwrap());
    drop(pool);
    server.stop().await;
}

/// Scenario: the running HTTP exporter rotates both CA and client identity
/// while an older export is held in flight, using an overridden logs endpoint.
/// Guarantees: new requests use the new identity without restart, compression
/// and sensitive headers survive rotation, and old success/new failure still
/// route Ack/Nack with their original payloads before clean shutdown.
#[test]
fn exporter_rotates_mtls_with_an_inflight_request() {
    run_exporter_rotation(|_| {}, write_material);
}

/// Scenario: Kubernetes replaces a projected Secret's ..data symlink while the
/// HTTP exporter has an in-flight request on the previous CA and identity.
/// Guarantees: coherent projected material installs, the old request completes,
/// and new exports use the new identity without restarting the exporter.
#[cfg(unix)]
#[test]
fn exporter_rotates_projected_mtls_with_an_inflight_request() {
    fn setup(path: &Path) {
        use std::os::unix::fs::symlink;
        let generation = path.join("..gen0");
        std::fs::create_dir(&generation).unwrap();
        for name in ["ca.crt", "client.crt", "client.key"] {
            std::fs::rename(path.join(name), generation.join(name)).unwrap();
            symlink(Path::new("..data").join(name), path.join(name)).unwrap();
        }
        symlink("..gen0", path.join("..data")).unwrap();
    }
    fn rotate(path: &Path, ca: &GeneratedCa, name: &str) {
        use std::os::unix::fs::symlink;
        let generation = path.join("..gen1");
        std::fs::create_dir(&generation).unwrap();
        write_material(&generation, ca, name);
        symlink("..gen1", path.join("..next")).unwrap();
        std::fs::rename(path.join("..next"), path.join("..data")).unwrap();
    }
    run_exporter_rotation(setup, rotate);
}

fn run_exporter_rotation(setup: fn(&Path), rotate: fn(&Path, &GeneratedCa, &str)) {
    let runtime = Runtime::new().unwrap();
    let mut fixture = runtime.block_on(TlsFixture::new());
    setup(fixture.dir.path());
    let mut server = runtime.block_on(TestServer::start(&fixture.ca));
    fixture.settings.user_agent = Some("http-tls-reload-test".into());
    fixture.settings.compression = Some(otel_arrow_dfe_otap::compression::CompressionMethod::Gzip);
    let _ = fixture
        .settings
        .headers
        .insert("authorization".into(), "Bearer test-token".into());
    let _ = fixture
        .settings
        .headers
        .insert("x-route".into(), "test-route".into());
    let config = Config {
        http: fixture.settings.clone(),
        logs_endpoint: Some(format!("{}/custom/logs", server.endpoint)),
        max_in_flight: 2,
        ..default_test_config("http://127.0.0.1:1".into())
    };
    let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_INPUT_METRICS);
    let (logs, _, _) = gen_batches_for_each_signal_type();
    let body = logs.encode_to_vec();
    let pdata =
        OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(Bytes::from(body.clone())).into());
    let old_pdata = pdata.clone().test_subscribe_to(
        Interests::ACKS | Interests::NACKS | Interests::RETURN_DATA,
        TestCallData::default().into(),
        123,
    );
    let new_pdata = pdata.test_subscribe_to(
        Interests::ACKS | Interests::NACKS | Interests::RETURN_DATA,
        TestCallData::default().into(),
        456,
    );
    runtime.block_on(tokio::task::LocalSet::new().run_until(async move {
        let mut running = RunningExporter::start(config, &pipeline_ctx);
        running.send(old_pdata).await;
        let old = server.request().await;
        let ca = generate_ca("rotated HTTP CA");
        rotate(fixture.dir.path(), &ca, "client-1");
        let _ = server.config.send_replace(server_config(&ca));
        let new = timeout(Duration::from_secs(10), async {
            loop {
                running.send(new_pdata.clone()).await;
                let request = tokio::select! {
                    request = server.request() => Some(request),
                    completed = running.completions.recv() => {
                        let PipelineCompletionMsg::DeliverNack { nack } = completed.unwrap() else {
                            panic!("only the retrying export can complete before rotation");
                        };
                        assert!(!nack.permanent);
                        None
                    }
                };
                let Some(request) = request else {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    continue;
                };
                if request.identity != old.identity {
                    break request;
                }
                request.response.send(StatusCode::OK).unwrap();
                assert!(matches!(
                    running.completions.recv().await.unwrap(),
                    PipelineCompletionMsg::DeliverAck { .. }
                ));
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("new identity installed");
        for request in [&old, &new] {
            assert_eq!(request.path, "/custom/logs");
            assert_eq!(request.headers[header::AUTHORIZATION], "Bearer test-token");
            assert_eq!(request.headers[header::USER_AGENT], "http-tls-reload-test");
            assert_eq!(request.headers["x-route"], "test-route");
            assert_eq!(request.headers[header::CONTENT_TYPE], PROTOBUF_CONTENT_TYPE);
            assert_eq!(request.headers[header::ACCEPT], PROTOBUF_CONTENT_TYPE);
            assert_eq!(request.headers[header::CONTENT_ENCODING], "gzip");
            let mut decoded = Vec::new();
            let _ = flate2::read::GzDecoder::new(request.body.as_ref())
                .read_to_end(&mut decoded)
                .unwrap();
            assert_eq!(decoded, body);
        }
        old.response.send(StatusCode::OK).unwrap();
        let ack = timeout(Duration::from_secs(5), running.completions.recv())
            .await
            .unwrap()
            .unwrap();
        let PipelineCompletionMsg::DeliverAck { mut ack } = ack else {
            panic!("old export must Ack")
        };
        assert_eq!(ack.accepted.num_items(), 1);
        assert_eq!(otel_arrow_dfe_otap::testing::next_ack(ack).unwrap().0, 123);
        new.response.send(StatusCode::SERVICE_UNAVAILABLE).unwrap();
        let nack = timeout(Duration::from_secs(5), running.completions.recv())
            .await
            .unwrap()
            .unwrap();
        let PipelineCompletionMsg::DeliverNack { mut nack } = nack else {
            panic!("new export must Nack")
        };
        assert!(!nack.permanent);
        assert_eq!(nack.refused.num_items(), 1);
        assert_eq!(
            otel_arrow_dfe_otap::testing::next_nack(nack).unwrap().0,
            456
        );
        running.stop().await;
        server.stop().await;
    }));
}
