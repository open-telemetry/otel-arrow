// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::{
    ExportLogsServiceResponse, logs_service_server::LogsService,
};
use tokio::sync::{mpsc, oneshot};

struct TlsFixture {
    dir: TempDir,
    ca: GeneratedCa,
    settings: GrpcClientSettings,
    provider: Rc<ClientTlsProvider>,
}

impl TlsFixture {
    async fn new() -> Self {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let ca = generate_ca("reload test CA");
        write_client_tls_material(dir.path(), &ca, "client-0");
        let settings = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".into(),
            tls: Some(TlsClientConfig {
                config: TlsConfig {
                    cert_file: Some(dir.path().join("client.crt")),
                    key_file: Some(dir.path().join("client.key")),
                    reload_interval: Some(Duration::from_millis(10)),
                    ..TlsConfig::default()
                },
                ca_file: Some(dir.path().join("ca.crt")),
                include_system_ca_certs_pool: Some(false),
                server_name: Some("localhost".into()),
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };
        let provider = Rc::new(
            ClientTlsProvider::new(settings.tls.as_ref(), &settings.grpc_endpoint)
                .await
                .expect("load TLS")
                .expect("TLS provider"),
        );
        Self {
            dir,
            ca,
            settings,
            provider,
        }
    }

    async fn rotate(&self, name: &str) -> Arc<ClientTlsGeneration> {
        let mut updates = self.provider.subscribe();
        write_client_tls_material(self.dir.path(), &self.ca, name);
        timeout(Duration::from_secs(2), async {
            tokio::select! {
                result = updates.changed() => result.expect("generation publication"),
                () = self.provider.run(std::future::pending()) => {
                    panic!("enabled reload loop returned");
                }
            }
        })
        .await
        .expect("TLS reload timeout");
        self.provider.current()
    }

    async fn pool(&self) -> GrpcClientPool {
        build_grpc_client_pool(
            &self.settings,
            2,
            3,
            Some(CompressionEncoding::Gzip),
            Some(&self.provider.current()),
        )
        .await
        .expect("initial pool")
    }

    fn failing_settings(&self) -> GrpcClientSettings {
        GrpcClientSettings {
            user_agent: Some("invalid\nuser-agent".into()),
            ..self.settings.clone()
        }
    }
}

/// Scenario: a replacement pool cannot be built, then the local build failure
/// is removed without publishing another TLS generation.
/// Guarantees: the installed pool stays intact on failure and the same pending
/// generation installs successfully when its retry deadline arrives.
#[tokio::test]
async fn failed_installation_retries_without_another_material_update() {
    let fixture = TlsFixture::new().await;
    let mut pool = fixture.pool().await;
    let generation = fixture.rotate("client-1").await;
    let updates = fixture.provider.subscribe();
    let mut installation = TlsPoolInstallation::new();
    installation
        .install(
            &fixture.failing_settings(),
            2,
            3,
            Some(CompressionEncoding::Gzip),
            generation.clone(),
            &mut pool,
        )
        .await;

    assert_eq!(pool.generation, 0);
    assert_eq!(pool.channels.len(), 2);
    assert_eq!(pool.compression, Some(CompressionEncoding::Gzip));
    assert_eq!(pool.logs.len(), 3);
    assert_eq!(pool.metrics.len(), 3);
    assert_eq!(pool.traces.len(), 3);
    assert!(Arc::ptr_eq(
        installation.pending.as_ref().expect("pending generation"),
        &generation
    ));
    assert!(
        tokio::time::sleep_until(installation.retry_at)
            .now_or_never()
            .is_none(),
        "failed installations must not busy-retry"
    );

    tokio::time::sleep_until(installation.retry_at).await;
    let pending = installation
        .pending
        .as_ref()
        .expect("retry candidate")
        .clone();
    installation
        .install(
            &fixture.settings,
            2,
            3,
            Some(CompressionEncoding::Gzip),
            pending,
            &mut pool,
        )
        .await;
    assert_eq!(pool.generation, generation.number());
    assert!(installation.pending.is_none());
    assert_eq!(installation.retry_delay, TLS_INSTALL_RETRY_MIN);
    assert_eq!(fixture.provider.current().number(), generation.number());
    assert!(!updates.has_changed().expect("live generation subscription"));
}

/// Scenario: an older generation repeatedly fails installation and a newer
/// generation is published before its backed-off retry deadline.
/// Guarantees: the newer generation installs immediately, clears the failed
/// candidate, resets backoff, and releases the obsolete material snapshot.
#[tokio::test]
async fn newer_generation_supersedes_a_failed_installation() {
    let fixture = TlsFixture::new().await;
    let mut pool = fixture.pool().await;
    let generation = fixture.rotate("client-1").await;
    let retired = Arc::downgrade(&generation);
    let mut installation = TlsPoolInstallation::new();
    for _ in 0..3 {
        installation
            .install(
                &fixture.failing_settings(),
                2,
                3,
                None,
                generation.clone(),
                &mut pool,
            )
            .await;
    }
    let old_deadline = installation.retry_at;
    assert_eq!(installation.retry_delay, Duration::from_secs(40));
    drop(generation);

    let newer = fixture.rotate("client-2").await;
    installation
        .install(&fixture.settings, 2, 3, None, newer.clone(), &mut pool)
        .await;

    assert_eq!(pool.generation, newer.number());
    assert!(tokio::time::Instant::now() < old_deadline);
    assert!(installation.pending.is_none());
    assert_eq!(installation.retry_delay, TLS_INSTALL_RETRY_MIN);
    assert!(retired.upgrade().is_none());
}

/// Scenario: the same generation fails installation more times than necessary
/// to reach the retry cap.
/// Guarantees: retry deadlines follow 5/10/20/40/60-second backoff, remain
/// capped, preserve the installed pool, and retain only one pending snapshot.
#[tokio::test]
async fn installation_failures_have_bounded_backoff_and_pending_state() {
    let fixture = TlsFixture::new().await;
    let mut pool = fixture.pool().await;
    let generation = fixture.rotate("client-1").await;
    let mut installation = TlsPoolInstallation::new();
    for seconds in [5, 10, 20, 40, 60, 60, 60] {
        let before = tokio::time::Instant::now();
        installation
            .install(
                &fixture.failing_settings(),
                2,
                3,
                None,
                generation.clone(),
                &mut pool,
            )
            .await;
        let after = tokio::time::Instant::now();
        let delay = Duration::from_secs(seconds);
        assert!(installation.retry_at >= before + delay);
        assert!(installation.retry_at <= after + delay);
        assert_eq!(pool.generation, 0);
        assert_eq!(pool.channels.len(), 2);
        assert_eq!(pool.logs.len(), 3);
        assert_eq!(Arc::strong_count(&generation), 3);
    }
    assert_eq!(installation.retry_delay, TLS_INSTALL_RETRY_MAX);

    let newer = fixture.rotate("client-2").await;
    let before = tokio::time::Instant::now();
    installation
        .install(&fixture.failing_settings(), 2, 3, None, newer, &mut pool)
        .await;
    assert!(installation.retry_at <= before + TLS_INSTALL_RETRY_MIN + Duration::from_secs(1));
    assert_eq!(installation.retry_delay, TLS_INSTALL_RETRY_MIN * 2);
}

/// Scenario: many successive generations replace a pool while clients checked
/// out from each retired pool are returned after the swap.
/// Guarantees: all three signal pools stay bounded, retired material is
/// released, and retired channel workers terminate instead of accumulating.
#[tokio::test]
async fn repeated_rotations_release_retired_pools_and_channel_tasks() {
    let fixture = TlsFixture::new().await;
    let mut pool = fixture.pool().await;
    let mut installation = TlsPoolInstallation::new();
    let baseline_tasks = tokio::runtime::Handle::current()
        .metrics()
        .num_alive_tasks();
    for index in 1..=12 {
        let retired = Arc::downgrade(&fixture.provider.current());
        let logs = pool.take_logs();
        let metrics = pool.take_metrics();
        let traces = pool.take_traces();
        let generation = fixture.rotate(&format!("client-{index}")).await;
        installation
            .install(
                &fixture.settings,
                2,
                3,
                Some(CompressionEncoding::Gzip),
                generation,
                &mut pool,
            )
            .await;
        pool.release(logs);
        pool.release(metrics);
        pool.release(traces);

        assert_eq!(pool.generation, index);
        assert_eq!(pool.channels.len(), 2);
        assert_eq!(pool.logs.len(), 3);
        assert_eq!(pool.metrics.len(), 3);
        assert_eq!(pool.traces.len(), 3);
        assert!(installation.pending.is_none());
        assert!(retired.upgrade().is_none());
        timeout(Duration::from_secs(2), async {
            while tokio::runtime::Handle::current()
                .metrics()
                .num_alive_tasks()
                > baseline_tasks
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("retired channel workers must terminate");
    }
}

/// Scenario: shutdown stops a live reload task with an installation retry
/// pending, and repeated exporter replacements drop the reload guard.
/// Guarantees: stopping does not wait for the retry deadline, dropping pending
/// state releases its snapshot, and every stopped or aborted task releases its provider.
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
                    2,
                    3,
                    None,
                    generation,
                    &mut pool,
                )
                .await;
            let mut task = LocalTlsReloadTask::start(fixture.provider.clone());
            tokio::task::yield_now().await;
            timeout(Duration::from_secs(1), task.stop())
                .await
                .expect("shutdown must not wait for installation retry")
                .expect("reload task stopped cleanly");
            assert!(tokio::time::Instant::now() < installation.retry_at);
            assert!(task.handle.is_none());
            assert_eq!(Rc::strong_count(&fixture.provider), 1);
            drop(installation);
            let _newer = fixture.rotate("client-2").await;
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
                .expect("replacement must release the aborted reload task");
            }
        })
        .await;
}

struct HeldRequest {
    identity: Vec<u8>,
    metadata: MetadataMap,
    response: oneshot::Sender<Result<(), tonic::Status>>,
}

struct HeldLogsService {
    requests: mpsc::Sender<HeldRequest>,
}

#[tonic::async_trait]
impl LogsService for HeldLogsService {
    async fn export(
        &self,
        request: tonic::Request<ExportLogsServiceRequest>,
    ) -> Result<tonic::Response<ExportLogsServiceResponse>, tonic::Status> {
        let identity = request.peer_certs().expect("mTLS peer identity")[0]
            .as_ref()
            .to_vec();
        let (response, result) = oneshot::channel();
        self.requests
            .send(HeldRequest {
                identity,
                metadata: request.metadata().clone(),
                response,
            })
            .await
            .expect("request observer");
        result.await.expect("controlled RPC response")?;
        Ok(tonic::Response::new(ExportLogsServiceResponse::default()))
    }
}

fn prepared_logs(
    metrics: &mut OtlpGrpcExporterMetrics,
    metadata: Option<MetadataMap>,
) -> EncodedExport {
    let (context, saved_payload) = subscribed_empty_logs().into_parts();
    EncodedExport {
        bytes: Bytes::new(),
        context,
        saved_payload,
        signal_type: SignalType::Logs,
        attempt: metrics.boundary.attempt(SignalType::Logs),
        metadata: RequestMetadata {
            metadata,
            auth_generation: None,
        },
    }
}

/// Scenario: an mTLS RPC remains blocked on the old pool while a new identity
/// installs and starts a second RPC using configured metadata and compression.
/// Guarantees: new RPCs present the rotated identity, old RPCs finish and Ack,
/// new failures Nack, and only current-generation clients return to the pool.
#[tokio::test]
async fn inflight_rpc_survives_rotation_and_preserves_completion_routing() {
    let mut fixture = TlsFixture::new().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind server");
    fixture.settings.grpc_endpoint = format!(
        "https://localhost:{}",
        listener.local_addr().expect("server address").port()
    );
    fixture.settings.user_agent = Some("tls-rotation-test".into());
    let _ = fixture.settings.headers.insert(
        "authorization".into(),
        secrecy::SecretString::from("Bearer test-token"),
    );
    let (requests, mut received) = mpsc::channel(2);
    let (shutdown, stopped) = oneshot::channel();
    let tls = mtls_server_config(&fixture.ca, "server");
    // The test server must run concurrently with the controlled RPCs.
    let server = tokio::spawn(async move {
        Server::builder()
            .tls_config(tls)
            .expect("server TLS")
            .add_service(
                LogsServiceServer::new(HeldLogsService { requests })
                    .accept_compressed(CompressionEncoding::Gzip),
            )
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                stopped.await.expect("server shutdown signal");
            })
            .await
            .expect("mTLS server");
    });
    let mut pool = fixture.pool().await;
    let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_INPUT_METRICS);
    let mut metrics = OtlpGrpcExporterMetrics::register(&pipeline_ctx, None);
    let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(4);
    let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
    let mut effects = EffectHandler::new(
        test_node("tls-test"),
        reporter,
        otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
    );
    effects.set_pipeline_completion_msg_sender(completion_tx);
    let mut inflight = InFlightExports::new();
    inflight.push(make_export_future(
        prepared_logs(&mut metrics, fixture.settings.build_static_metadata()),
        pool.take_logs(),
    ));
    let old = timeout(Duration::from_secs(2), async {
        tokio::select! {
            request = received.recv() => request.expect("old request"),
            _ = inflight.next_completion() => panic!("old RPC must remain in flight"),
        }
    })
    .await
    .expect("old RPC reached server");

    let generation = fixture.rotate("client-1").await;
    let mut installation = TlsPoolInstallation::new();
    installation
        .install(
            &fixture.settings,
            2,
            3,
            Some(CompressionEncoding::Gzip),
            generation,
            &mut pool,
        )
        .await;
    assert_eq!(pool.generation, 1);
    assert_eq!(pool.channels.len(), 2);
    assert_eq!(pool.compression, Some(CompressionEncoding::Gzip));
    inflight.push(make_export_future(
        prepared_logs(&mut metrics, fixture.settings.build_static_metadata()),
        pool.take_logs(),
    ));
    let new = timeout(Duration::from_secs(2), async {
        tokio::select! {
            request = received.recv() => request.expect("new request"),
            _ = inflight.next_completion() => panic!("both RPCs must remain in flight"),
        }
    })
    .await
    .expect("rotated RPC reached server");
    assert_ne!(old.identity, new.identity);
    for request in [&old, &new] {
        assert_eq!(
            request.metadata.get("authorization").expect("auth"),
            "Bearer test-token"
        );
        assert!(
            request
                .metadata
                .get("user-agent")
                .expect("user-agent")
                .to_str()
                .expect("ASCII user-agent")
                .starts_with("tls-rotation-test")
        );
    }

    old.response.send(Ok(())).expect("finish old RPC");
    let completed = timeout(Duration::from_secs(2), inflight.next_completion())
        .await
        .expect("old RPC completion")
        .expect("completed old RPC");
    let (client, _) = finalize_completed_export(completed, &effects, &mut metrics).await;
    assert_eq!(client.generation, 0);
    pool.release(client);
    assert_eq!(pool.logs.len(), 2);
    wait_for_ack_or_nack(&mut completion_rx, true, 123, "old-generation RPC")
        .await
        .expect("old RPC Ack");

    new.response
        .send(Err(tonic::Status::unavailable("retry new-generation RPC")))
        .expect("fail new RPC");
    let completed = timeout(Duration::from_secs(2), inflight.next_completion())
        .await
        .expect("new RPC completion")
        .expect("completed new RPC");
    let (client, _) = finalize_completed_export(completed, &effects, &mut metrics).await;
    assert_eq!(client.generation, 1);
    pool.release(client);
    assert_eq!(pool.logs.len(), 3);
    wait_for_ack_or_nack(&mut completion_rx, false, 123, "new-generation RPC")
        .await
        .expect("new RPC Nack");
    assert!(inflight.is_empty());
    drop(pool);
    shutdown.send(()).expect("stop server");
    timeout(Duration::from_secs(2), server)
        .await
        .expect("server shutdown timeout")
        .expect("server task");
}

/// Scenario: a running exporter observes regular-file CA and client-identity
/// rotation while the backend switches to a different issuing CA.
/// Guarantees: exports before and after rotation Ack without restart on both
/// Windows and Unix, and exporter shutdown terminates its local reload task.
#[test]
fn grpc_exporter_hot_reloads_regular_file_mtls_generation() {
    fn copy_material(root: &Path, generation: &Path) {
        for name in ["ca.crt", "client.crt", "client.key"] {
            let _ =
                std::fs::copy(generation.join(name), root.join(name)).expect("copy TLS material");
        }
    }
    run_grpc_exporter_mtls_rotation(copy_material, copy_material);
}
