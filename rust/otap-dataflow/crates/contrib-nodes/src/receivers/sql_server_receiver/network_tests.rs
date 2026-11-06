// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Loopback TDS peer for the real production native-TLS connector. No SQL Server,
//! container, global trust change, or process-wide Rustls provider is needed.

use super::{
    SqlServerAdapter, SqlServerError as Error, SqlServerReceiverConfig, TcpConnector,
    adapter_tests::{
        Credentials, attention_done, done, login, metadata_response, packet, raw_config,
        rows_response,
    },
};
use async_trait::async_trait;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::{
    control::{AckMsg, NodeControlMsg},
    error::Error as EngineError,
    local::receiver as local,
    memory_limiter::{LocalReceiverAdmissionState, MemoryPressureState},
    receiver::ReceiverWrapper,
    terminal_state::TerminalState,
    testing::{receiver::TestRuntime, test_node},
};
use otel_arrow_dfe_otap::{pdata::OtapPdata, testing::next_ack};
use otel_arrow_dfe_scraper::{
    CheckpointStore, DatabaseReceiver, SourceBinding,
    database::{Cursor, DriverAdapter, ScalarValue},
};
use rcgen::{
    BasicConstraints, CertificateParams, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use serde_json::{Value, json};
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread::JoinHandle,
    time::Duration,
};

struct Certificates {
    ca: rcgen::Certificate,
    issuer: Issuer<'static, KeyPair>,
    directory: tempfile::TempDir,
}

impl Certificates {
    fn new() -> Self {
        let mut params = CertificateParams::new(Vec::<String>::new()).unwrap();
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "SQL native test CA");
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        let key = KeyPair::generate().unwrap();
        let ca = params.self_signed(&key).unwrap();
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("ca.pem"), ca.pem()).unwrap();
        Self {
            ca,
            issuer: Issuer::new(params, key),
            directory,
        }
    }

    fn path(&self) -> PathBuf {
        self.directory.path().join("ca.pem")
    }

    fn server(&self, expired: bool) -> Arc<rustls::ServerConfig> {
        let key = KeyPair::generate().unwrap();
        let mut params = CertificateParams::new(vec!["localhost".to_owned()]).unwrap();
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "localhost");
        params.use_authority_key_identifier_extension = true;
        params.is_ca = IsCa::ExplicitNoCa;
        params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        if expired {
            params.not_before = rcgen::date_time_ymd(2000, 1, 1);
            params.not_after = rcgen::date_time_ymd(2001, 1, 1);
        }
        let cert = params.signed_by(&key, &self.issuer).unwrap();
        // Ring is only the test peer's explicitly selected server backend. The
        // client under test uses Tiberius native TLS, with no global provider.
        Arc::new(
            rustls::ServerConfig::builder_with_provider(Arc::new(
                rustls::crypto::ring::default_provider(),
            ))
            .with_protocol_versions(&[&rustls::version::TLS12])
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(
                vec![cert.der().clone(), self.ca.der().clone()],
                rustls::pki_types::PrivatePkcs8KeyDer::from(key.serialize_der()).into(),
            )
            .unwrap(),
        )
    }
}

// A blocking test-only TDS framing shim. Production framing remains entirely
// inside Tiberius. Socket timeouts and a joined owner bound all mock peer work.
struct HandshakeSocket {
    socket: TcpStream,
    framed: bool,
    remaining: usize,
}

impl Read for HandshakeSocket {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if !self.framed || buffer.is_empty() {
            return self.socket.read(buffer);
        }
        if self.remaining == 0 {
            let mut header = [0; 8];
            self.socket.read_exact(&mut header)?;
            if header[0] != 18 {
                return Err(io::Error::other("expected TLS prelogin frame"));
            }
            self.remaining = usize::from(u16::from_be_bytes([header[2], header[3]]))
                .checked_sub(8)
                .filter(|length| *length > 0)
                .ok_or_else(|| io::Error::other("invalid TLS frame length"))?;
        }
        let length = buffer.len().min(self.remaining);
        let read = self.socket.read(&mut buffer[..length])?;
        self.remaining -= read;
        Ok(read)
    }
}

impl Write for HandshakeSocket {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.framed {
            let mut frame = packet(buffer.to_vec());
            frame[0] = 18;
            self.socket.write_all(&frame)?;
            Ok(buffer.len())
        } else {
            self.socket.write(buffer)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.socket.flush()
    }
}

fn request(stream: &mut impl Read) -> io::Result<u8> {
    let mut total = 0usize;
    loop {
        let mut header = [0; 8];
        stream.read_exact(&mut header)?;
        let length = usize::from(u16::from_be_bytes([header[2], header[3]]));
        if length < 8 || total + length > 1024 * 1024 {
            return Err(io::Error::other("test request exceeds limit"));
        }
        total += length;
        let mut payload = vec![0; length - 8];
        stream.read_exact(&mut payload)?;
        if header[1] & 1 != 0 {
            return Ok(header[0]);
        }
    }
}

enum Step {
    Reply(Vec<u8>),
    SplitAttention,
    MissingAttention,
    LateAttention,
    IdleClose(mpsc::Receiver<()>),
}

struct Session {
    tls: Arc<rustls::ServerConfig>,
    drop_handshake: bool,
    steps: Vec<Step>,
}

impl Session {
    fn new(tls: Arc<rustls::ServerConfig>, replies: Vec<Vec<u8>>) -> Self {
        Self {
            tls,
            drop_handshake: false,
            steps: replies.into_iter().map(Step::Reply).collect(),
        }
    }
}

struct Peer {
    address: SocketAddr,
    stopping: Arc<AtomicBool>,
    connections: Arc<AtomicUsize>,
    requests: Arc<AtomicUsize>,
    thread: Option<JoinHandle<()>>,
}

impl Peer {
    fn start(sessions: Vec<Session>) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let stopping = Arc::new(AtomicBool::new(false));
        let connections = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(AtomicUsize::new(0));
        let (stop, calls, queries) = (
            Arc::clone(&stopping),
            Arc::clone(&connections),
            Arc::clone(&requests),
        );
        // A dedicated test peer thread models independent server progress while
        // the production pipeline remains current-thread/local. Atomics expose
        // only counters and shutdown; neither credentials nor rows are shared.
        let thread = std::thread::spawn(move || {
            for session in sessions {
                let socket = loop {
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }
                    match listener.accept() {
                        Ok((socket, _)) => break socket,
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(2))
                        }
                        Err(error) => panic!("accept failed: {error}"),
                    }
                };
                _ = calls.fetch_add(1, Ordering::Relaxed);
                socket.set_nonblocking(false).unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                socket
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                socket.set_nodelay(true).unwrap();
                serve(socket, session, &queries);
            }
        });
        Self {
            address,
            stopping,
            connections,
            requests,
            thread: Some(thread),
        }
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take()
            && let Err(panic) = thread.join()
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(panic);
        }
    }
}

fn serve(mut socket: TcpStream, session: Session, requests: &AtomicUsize) {
    assert_eq!(request(&mut socket).unwrap(), 18);
    socket
        .write_all(&packet(vec![1, 0, 6, 0, 1, 0xff, 1]))
        .unwrap();
    if session.drop_handshake {
        // Consume the actual ClientHello before closing; the failure is during
        // TLS, not TCP/prelogin, and no SQL query has been submitted.
        assert_eq!(request(&mut socket).unwrap(), 18);
        return;
    }
    let mut socket = HandshakeSocket {
        socket,
        framed: true,
        remaining: 0,
    };
    let mut connection = rustls::ServerConnection::new(session.tls).unwrap();
    while connection.is_handshaking() {
        if connection.complete_io(&mut socket).is_err() {
            return; // Negative certificate cases intentionally reject the peer.
        }
    }
    socket.framed = false;
    let mut stream = rustls::StreamOwned::new(connection, socket);
    match request(&mut stream) {
        Ok(kind) => assert_eq!(kind, 16),
        // Schannel can finish TLS records before rejecting the chain/name.
        // Negative tests assert the client's permanent TLS error and zero SQL.
        Err(_) => return,
    }
    stream.write_all(&login()).unwrap();
    stream.flush().unwrap();
    for step in session.steps {
        match step {
            Step::Reply(reply) => {
                assert_eq!(request(&mut stream).unwrap(), 3);
                _ = requests.fetch_add(1, Ordering::Relaxed);
                stream.write_all(&reply).unwrap();
                stream.flush().unwrap();
            }
            Step::SplitAttention | Step::MissingAttention | Step::LateAttention => {
                assert_eq!(request(&mut stream).unwrap(), 3);
                _ = requests.fetch_add(1, Ordering::Relaxed);
                // Send a row but no DONE; the timeout must not emit this partial page.
                let response = rows_response(&[8]);
                let mut unfinished = packet(response[8..response.len() - done().len()].to_vec());
                unfinished[1] = 0; // Response continues; no EOM/EOF completion.
                stream.write_all(&unfinished).unwrap();
                stream.flush().unwrap();
                assert_eq!(request(&mut stream).unwrap(), 6);
                _ = requests.fetch_add(1, Ordering::Relaxed);
                if matches!(step, Step::SplitAttention) {
                    let mut error = done();
                    error[1] = 2; // DONE_ERROR, separate TDS message from DONE_ATTN.
                    stream.write_all(&packet(error)).unwrap();
                    stream.flush().unwrap();
                    std::thread::sleep(Duration::from_millis(20));
                    // The published driver may already have closed; this is an expected
                    // peer-side error, not permission for the receiver to resume.
                    if stream.write_all(&attention_done()).is_ok() {
                        _ = stream.flush();
                    }
                } else if matches!(step, Step::LateAttention) {
                    std::thread::sleep(Duration::from_millis(1200));
                    if stream.write_all(&attention_done()).is_ok() {
                        _ = stream.flush();
                    }
                }
                assert!(
                    request(&mut stream).is_err(),
                    "no request after unconfirmed ATTENTION"
                );
                return;
            }
            Step::IdleClose(gate) => {
                gate.recv_timeout(Duration::from_secs(5)).unwrap();
                return;
            }
        }
    }
    assert!(request(&mut stream).is_err(), "no unscripted query");
}

fn configuration(peer: &Peer, ca: &Path) -> Value {
    let mut raw = raw_config(false, 4096);
    raw["connection"]["host"] = json!("127.0.0.1");
    raw["connection"]["port"] = json!(peer.address.port());
    raw["connection"]["connect_timeout"] = json!("5s");
    raw["connection"]["tls"] = json!({"ca_file":ca, "server_name_override":"localhost"});
    raw["query"]["catch_up"] = json!({"max_pages":1});
    raw
}

fn recovered(tls: Arc<rustls::ServerConfig>) -> Session {
    Session::new(
        tls,
        vec![
            metadata_response(),
            metadata_response(),
            rows_response(&[8, 9]),
        ],
    )
}

/// Scenario: Native TLS sees valid, unknown, mismatched, expired, and replaced private-CA certificates.
/// Guarantees: Only verified identities or explicit bypass succeed, with no global trust/provider changes.
#[tokio::test]
async fn native_certificate_policy_and_replacement() {
    let certs = Certificates::new();
    for case in [
        "valid",
        "unknown",
        "wrong-name",
        "expired",
        "replacement",
        "bypass",
    ] {
        let peer = Peer::start(vec![Session::new(
            certs.server(case == "expired"),
            vec![metadata_response()],
        )]);
        let mut raw = configuration(&peer, &certs.path());
        if case == "valid" {
            raw["connection"]["host"] = json!("localhost");
            raw["connection"]["tls"] = json!({"ca_file":certs.path()});
        } else if case == "unknown" {
            raw["connection"]["tls"] = json!({"server_name_override":"localhost"});
        } else if case == "wrong-name" {
            raw["connection"]["tls"]["server_name_override"] = json!("wrong.invalid");
        } else if case == "bypass" {
            raw["connection"]["tls"] = json!({"trust_server_certificate":true});
        }
        let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
        let query = config.query().clone();
        let mut adapter =
            SqlServerAdapter::new(config, TcpConnector::default(), Box::new(Credentials));
        _ = adapter.begin_operation().unwrap();
        let result = adapter.validate_query(&query).await;
        if matches!(case, "valid" | "replacement" | "bypass") {
            assert_eq!(
                result
                    .unwrap_or_else(|error| panic!("{case}: {error}"))
                    .len(),
                1
            );
        } else {
            assert_eq!(result, Err(Error::Tls), "{case}");
            assert!(!SqlServerAdapter::<TcpConnector>::is_retryable(&Error::Tls));
            assert_eq!(peer.requests.load(Ordering::Relaxed), 0);
        }
        adapter.shutdown().await.unwrap();
    }
}

/// Scenario: An embedding process has installed no Rustls provider at all.
/// Guarantees: Verified native TLS still connects and validates metadata using connection-local CA trust.
#[test]
fn native_tls_does_not_require_rustls_provider() {
    const CHILD: &str = "OTAP_NATIVE_TLS_NO_PROVIDER";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "receivers::sql_server_receiver::network_tests::native_tls_does_not_require_rustls_provider", "--nocapture"])
            .env(CHILD, "1").output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    assert!(!otel_arrow_dfe_otap::crypto::is_crypto_provider_installed());
    native_certificate_policy_and_replacement();
    assert!(!otel_arrow_dfe_otap::crypto::is_crypto_provider_installed());
}

struct Probe {
    receiver: DatabaseReceiver<SqlServerAdapter<TcpConnector>>,
    fails: bool,
    cleanup_fails: bool,
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for Probe {
    async fn start(
        self: Box<Self>,
        controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let result = tokio::time::timeout(
            Duration::from_secs(12),
            local::Receiver::start(Box::new(self.receiver), controls, effects),
        )
        .await
        .expect("receiver bounded termination");
        if self.fails {
            let error = result.err().expect("must fail closed");
            assert_eq!(
                error.to_string().contains("cleanup failed"),
                self.cleanup_fails,
                "{error}"
            );
        } else {
            _ = result.expect("confirmed shutdown");
        }
        Ok(TerminalState::default())
    }
}

fn controller_case(root: &Path, case: &str) {
    let certs = Certificates::new();
    let (close, gate) = mpsc::channel();
    let sessions = match case {
        "handshake" => vec![
            Session {
                tls: certs.server(false),
                drop_handshake: true,
                steps: vec![],
            },
            recovered(certs.server(false)),
        ],
        "certificate" => vec![recovered(certs.server(true))],
        "idle" => {
            let mut session = recovered(certs.server(false));
            session.steps.push(Step::IdleClose(gate));
            vec![session]
        }
        _ => {
            let mut session = Session::new(
                certs.server(false),
                vec![metadata_response(), metadata_response()],
            );
            session.steps.push(match case {
                "split" => Step::SplitAttention,
                "late" => Step::LateAttention,
                _ => Step::MissingAttention,
            });
            vec![session]
        }
    };
    let peer = Peer::start(sessions);
    let config: SqlServerReceiverConfig =
        serde_json::from_value(configuration(&peer, &certs.path())).unwrap();
    let query = config.query().clone();
    let store = CheckpointStore::new(
        root,
        "group",
        "pipeline",
        "native",
        "events",
        config.config_fingerprint().to_owned(),
    );
    let source = SourceBinding::acquire(store.clone()).unwrap();
    let previous = store
        .write(0, &Cursor::Scalar(ScalarValue::Int64(7)))
        .unwrap()
        .0;
    let receiver = DatabaseReceiver::new(
        SqlServerAdapter::new(config, TcpConnector::default(), Box::new(Credentials)),
        query,
        source,
        Duration::from_millis(1),
        5,
        LocalReceiverAdmissionState::from_process_state(&MemoryPressureState::default()),
        None,
    );
    let failed = case != "handshake";
    let quarantine = !matches!(case, "handshake" | "certificate");
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        Probe {
            receiver,
            fails: failed,
            cleanup_fails: quarantine,
        },
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            super::SQL_SERVER_RECEIVER_URN,
        )),
        runtime.config(),
    );
    let checkpoint = store.clone();
    let check = previous.clone();
    let idle = case == "idle";
    let interrupt = case == "dropped";
    let requests = Arc::clone(&peer.requests);
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            if interrupt {
                tokio::time::timeout(Duration::from_secs(3), async {
                    while requests.load(Ordering::Relaxed) < 4 {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                })
                .await
                .expect("TLS ATTENTION began");
                ctx.send_control_msg(NodeControlMsg::Shutdown {
                    deadline: std::time::Instant::now() + Duration::from_millis(100),
                    reason: "interrupt TLS cleanup".into(),
                })
                .await
                .unwrap();
            }
            if !failed || idle {
                let mut data = tokio::time::timeout(Duration::from_secs(8), ctx.recv())
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(data.num_items(), 2);
                assert_eq!(checkpoint.read().unwrap(), Some(check));
                let (_, ack) = next_ack(AckMsg::new(data)).unwrap();
                ctx.send_control_msg(NodeControlMsg::Ack(ack))
                    .await
                    .unwrap();
                tokio::time::timeout(Duration::from_secs(3), async {
                    while checkpoint.read().unwrap().unwrap().cursor
                        != Cursor::Scalar(ScalarValue::Int64(9))
                    {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                })
                .await
                .unwrap();
                if idle {
                    close.send(()).unwrap();
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    // Advance the scheduled interval only after TLS/query/ACK work
                    // is idle; never fake a handshake or cancellation deadline.
                    tokio::time::pause();
                    tokio::time::advance(Duration::from_secs(61)).await;
                    tokio::time::resume();
                } else {
                    ctx.send_control_msg(NodeControlMsg::Shutdown {
                        deadline: std::time::Instant::now() + Duration::from_secs(2),
                        reason: "native retry verified".into(),
                    })
                    .await
                    .unwrap();
                    return;
                }
            }
            assert!(
                tokio::time::timeout(Duration::from_secs(8), ctx.recv())
                    .await
                    .unwrap()
                    .is_err(),
                "no partial page"
            );
        });
    let state = store.read().unwrap().unwrap();
    assert_eq!(
        state.cursor,
        if idle || !failed {
            Cursor::Scalar(ScalarValue::Int64(9))
        } else {
            previous.cursor
        }
    );
    assert_eq!(
        state.revision,
        previous.revision + u64::from(idle || !failed)
    );
    assert_eq!(
        peer.connections.load(Ordering::Relaxed),
        if case == "handshake" { 2 } else { 1 }
    );
    assert_eq!(
        peer.requests.load(Ordering::Relaxed),
        match case {
            "certificate" => 0,
            "handshake" | "idle" => 3,
            _ => 4,
        },
        "no SQL after unconfirmed cleanup",
    );
    if quarantine {
        assert!(
            SourceBinding::acquire(store).is_err(),
            "ownership remains quarantined"
        );
    } else {
        drop(SourceBinding::acquire(store).expect("confirmed cleanup releases source"));
    }
}

/// Scenario: A peer closes after ClientHello, then a healthy connection serves the same source.
/// Guarantees: Shared retry follows confirmed worker cleanup/revalidation; only downstream ACK advances progress.
#[test]
fn handshake_network_failure_retries_through_controller() {
    let root = tempfile::tempdir().unwrap();
    controller_case(root.path(), "handshake");
}

/// Scenario: Native TLS rejects the server certificate before login.
/// Guarantees: Shared control terminates without retry, output, or checkpoint movement and releases clean ownership.
#[test]
fn certificate_failure_is_permanent_through_controller() {
    let root = tempfile::tempdir().unwrap();
    controller_case(root.path(), "certificate");
}

/// Scenario: TLS has split/missing/late/dropped ATTENTION or the peer closes idle after an ACK.
/// Guarantees: No uncertain work is reused/retried/emitted; saved progress is unchanged and ownership stays quarantined.
#[test]
fn native_tls_uncertainty_and_idle_close_quarantine() {
    const CHILD: &str = "OTAP_NATIVE_TLS_QUARANTINE";
    if let Some(root) = std::env::var_os(CHILD) {
        for case in ["split", "missing", "late", "dropped", "idle"] {
            let path = PathBuf::from(&root).join(case);
            std::fs::create_dir(&path).unwrap();
            controller_case(&path, case);
        }
        return;
    }
    // Quarantine intentionally keeps file handles until process exit. The
    // parent owns the scratch directory and waits for child exit before cleanup.
    let root = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "receivers::sql_server_receiver::network_tests::native_tls_uncertainty_and_idle_close_quarantine", "--nocapture"])
        .env(CHILD, root.path()).spawn().unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(45);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "native containment subprocess failed");
            break;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            _ = child.wait();
            panic!("native containment subprocess exceeded watchdog");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    root.close().unwrap();
}
