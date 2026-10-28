// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! SQL Server connection policy without blocking the pipeline's local executor.

use super::{
    SqlServerConnectionConfig, SqlServerConnector, SqlServerError as Error, adapter::configure_tls,
};
use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncWrite};
use std::{
    fs::File,
    io::{self, Read},
    net::ToSocketAddrs,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, OnceLock},
    task::{Context, Poll},
};
use tiberius::{Client, Config};
use tokio::{
    net::TcpStream,
    runtime::Handle,
    sync::{oneshot, watch},
};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use tracing::instrument::WithSubscriber;

const MAX_CA_BYTES: u64 = 4 * 1024 * 1024;

/// TCP/TLS connector defaulting to verified platform-native TLS.
///
/// DNS, CA loading and native TLS setup can perform synchronous work. One
/// bounded, adapter-owned thread handles connection establishment only, using
/// the existing Tokio reactor. After handoff, all query I/O remains local.
/// A dropped connect future requests cancellation; shutdown confirms completion.
#[derive(Default)]
pub struct TcpConnector {
    worker: Option<ConnectionWorker>,
    cleanup_failed: bool,
}

/// TCP transport retaining a safe read-failure kind across native TLS error erasure.
///
/// Tiberius owns the stream during setup and stringifies native TLS failures.
/// The shared, write-once cell lets the same setup worker recover a structured
/// socket cause without retaining diagnostics, packet contents, or credentials.
/// Tiberius requires a Send transport; queries still execute on the local core.
pub struct ObservedTcpStream {
    stream: Compat<TcpStream>,
    read_failure: Arc<OnceLock<io::ErrorKind>>,
}

impl std::fmt::Debug for ObservedTcpStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ObservedTcpStream(<redacted>)")
    }
}

impl AsyncRead for ObservedTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.stream).poll_read(cx, buffer);
        let failure = match &result {
            Poll::Ready(Ok(0)) if !buffer.is_empty() => Some(io::ErrorKind::UnexpectedEof),
            Poll::Ready(Err(error))
                if !matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                ) =>
            {
                Some(error.kind())
            }
            _ => None,
        };
        if let Some(kind) = failure {
            _ = this.read_failure.set(kind);
        }
        result
    }
}

impl AsyncWrite for ObservedTcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_write(cx, buffer)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_close(cx)
    }
}

struct ConnectionWorker {
    cancel: watch::Sender<bool>,
    exited: oneshot::Receiver<()>,
}

impl Drop for ConnectionWorker {
    fn drop(&mut self) {
        _ = self.cancel.send_replace(true);
    }
}

struct CancelOnDrop(watch::Sender<bool>);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        _ = self.0.send_replace(true);
    }
}

impl TcpConnector {
    async fn finish(&mut self) -> Result<(), Error> {
        if self.cleanup_failed {
            return Err(Error::Driver);
        }
        if let Some(worker) = self.worker.as_mut()
            && (&mut worker.exited).await.is_err()
        {
            self.cleanup_failed = true;
            self.worker = None;
            return Err(Error::Driver);
        }
        self.worker = None;
        Ok(())
    }
}

#[async_trait(?Send)]
impl SqlServerConnector for TcpConnector {
    type Stream = ObservedTcpStream;

    async fn connect(
        &mut self,
        endpoint: &SqlServerConnectionConfig,
        mut config: Config,
    ) -> Result<Client<Self::Stream>, Error> {
        configure_tls(endpoint, &mut config)?;
        // Join an interrupted attempt before admitting another connection.
        self.shutdown().await?;
        let host = endpoint.host().to_owned();
        let port = endpoint.port();
        let ca_file = endpoint.ca_file().map(PathBuf::from);
        let reactor = Handle::current();
        let (cancel, mut cancelled) = watch::channel(false);
        let (result_tx, result_rx) = oneshot::channel();
        let (exited_tx, exited) = oneshot::channel();
        let guard = CancelOnDrop(cancel.clone());

        // This thread owns synchronous DNS, CA loading and native TLS setup.
        // Socket registration uses the pipeline reactor, not a second runtime.
        // There is at most one attempt/thread per connector and no queued jobs.
        _ = std::thread::Builder::new()
            .name("sql-server-connect".into())
            .spawn(move || {
                let result = {
                    let _runtime = reactor.enter();
                    futures::executor::block_on(
                        async {
                            tokio::select! {
                                biased;
                                _ = cancelled.changed() => Err(Error::Cancelled),
                                result = establish(host, port, ca_file, config) => result,
                            }
                        }
                        .with_subscriber(tracing::subscriber::NoSubscriber::default()),
                    )
                };
                // Failed delivery drops the session here, before the exit acknowledgement.
                drop(result_tx.send(result));
                _ = exited_tx.send(());
            })
            .map_err(|_| Error::Driver)?;
        self.worker = Some(ConnectionWorker { cancel, exited });
        let result = result_rx.await.map_err(|_| Error::Driver)?;
        self.finish().await?;
        drop(guard);
        result
    }

    async fn shutdown(&mut self) -> Result<(), Error> {
        if let Some(worker) = &self.worker {
            _ = worker.cancel.send_replace(true);
        }
        self.finish().await
    }
}

async fn establish(
    host: String,
    port: u16,
    ca_file: Option<PathBuf>,
    mut config: Config,
) -> Result<Client<ObservedTcpStream>, Error> {
    if let Some(path) = ca_file {
        let mut bytes = Vec::new();
        _ = File::open(path)
            .map_err(|_| Error::Tls)?
            .take(MAX_CA_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::Tls)?;
        if bytes.is_empty() || bytes.len() as u64 > MAX_CA_BYTES {
            return Err(Error::Tls);
        }
        config.trust_cert_ca_bundle(bytes);
    }
    // Tokio's hostname lookup uses a blocking-pool task which can outlive a
    // dropped future. Resolve on our owned thread instead, so shutdown cannot
    // acknowledge exit while an orphaned DNS task is still outstanding.
    let addresses = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|error| Error::NameResolution(error.kind()))?;
    if addresses.as_slice().is_empty() {
        return Err(Error::NameResolution(io::ErrorKind::NotFound));
    }
    let stream = TcpStream::connect(addresses.as_slice())
        .await
        .map_err(|error| Error::Transport(error.kind()))?;
    stream
        .set_nodelay(true)
        .map_err(|error| Error::Transport(error.kind()))?;
    let read_failure = Arc::new(OnceLock::new());
    let stream = ObservedTcpStream {
        stream: stream.compat_write(),
        read_failure: Arc::clone(&read_failure),
    };
    Client::connect(config, stream).await.map_err(|error| {
        // Native TLS stringifies its source chain. Only an independently observed
        // socket read failure can recover a transport category. In particular,
        // a failed write of a certificate-rejection alert must not turn a trust
        // failure into a retry. All unclassified TLS failures remain permanent.
        if matches!(error, tiberius::error::Error::Tls(_))
            && let Some(kind) = read_failure.get()
        {
            Error::Transport(*kind)
        } else {
            Error::from(error)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Scenario: A worker exits without acknowledging cleanup, such as after a panic.
    /// Guarantees: Repeated shutdown attempts stay failed without polling a completed oneshot again.
    #[tokio::test]
    async fn unconfirmed_exit_remains_failed() {
        let (cancel, _cancelled) = watch::channel(false);
        let (tx, exited) = oneshot::channel();
        drop(tx);
        let mut connector = TcpConnector {
            worker: Some(ConnectionWorker { cancel, exited }),
            cleanup_failed: false,
        };
        assert_eq!(connector.shutdown().await, Err(Error::Driver));
        assert_eq!(connector.shutdown().await, Err(Error::Driver));
        let endpoint: SqlServerConnectionConfig = serde_json::from_value(serde_json::json!({
            "host":"127.0.0.1", "database":"fixture"
        }))
        .unwrap();
        assert_eq!(
            connector
                .connect(&endpoint, Config::new())
                .await
                .unwrap_err(),
            Error::Driver,
        );
        assert!(
            connector.worker.is_none(),
            "failed cleanup cannot spawn a replacement"
        );
    }

    /// Scenario: A TCP peer accepts the handshake but never answers prelogin.
    /// Guarantees: Dropping the connect future cancels and joins its owned worker without blocking the core.
    #[tokio::test]
    async fn abandoned_handshake_stops_owned_worker() {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let endpoint: SqlServerConnectionConfig = serde_json::from_value(serde_json::json!({
            "host":"127.0.0.1", "port":address.port(), "database":"fixture"
        }))
        .unwrap();
        let mut connector = TcpConnector::default();
        let mut config = Config::new();
        config.host("127.0.0.1");
        config.port(address.port());
        let mut socket = {
            let connection = connector.connect(&endpoint, config);
            tokio::pin!(connection);
            tokio::select! {
                result = &mut connection => panic!("unexpected handshake completion: {result:?}"),
                accepted = tokio::time::timeout(Duration::from_secs(5), listener.accept()) => {
                    accepted.unwrap().unwrap().0
                }
            }
            // Dropping the in-flight future signals cancellation while the peer stays open.
        };
        tokio::time::timeout(Duration::from_secs(2), connector.shutdown())
            .await
            .unwrap()
            .unwrap();
        assert!(connector.worker.is_none());
        let mut prelogin = Vec::new();
        _ = tokio::time::timeout(
            Duration::from_secs(2),
            tokio::io::AsyncReadExt::read_to_end(&mut socket, &mut prelogin),
        )
        .await
        .expect("cancelled connector closes the socket")
        .unwrap();
    }

    /// Scenario: A cancelled worker has not yet acknowledged exit and shutdown is interrupted.
    /// Guarantees: Cleanup retains that worker until acknowledgement, without admitting another job.
    #[tokio::test(start_paused = true)]
    async fn interrupted_shutdown_retains_worker_until_confirmed() {
        let (cancel, cancelled) = watch::channel(false);
        let (acknowledge, exited) = oneshot::channel();
        let mut connector = TcpConnector {
            worker: Some(ConnectionWorker { cancel, exited }),
            cleanup_failed: false,
        };
        assert!(
            tokio::time::timeout(Duration::from_millis(10), connector.shutdown())
                .await
                .is_err()
        );
        assert!(*cancelled.borrow());
        assert!(connector.worker.is_some());
        let endpoint: SqlServerConnectionConfig = serde_json::from_value(serde_json::json!({
            "host":"127.0.0.1", "database":"fixture"
        }))
        .unwrap();
        {
            let attempt = connector.connect(&endpoint, Config::new());
            tokio::pin!(attempt);
            assert!(futures::poll!(&mut attempt).is_pending());
        }
        assert!(
            connector.worker.is_some(),
            "replacement waits for the original worker"
        );
        acknowledge.send(()).unwrap();
        connector.shutdown().await.unwrap();
        assert!(connector.worker.is_none());
        connector.shutdown().await.unwrap();
    }

    /// Scenario: The configured CA file is missing, empty, or exceeds the 4 MiB loading limit.
    /// Guarantees: Connection fails with a redacted TLS error and confirms worker cleanup.
    #[tokio::test]
    async fn invalid_ca_files_fail_closed_and_clean_up() {
        let directory = tempfile::tempdir().unwrap();
        let ca = directory.path().join("sensitive-ca.pem");
        for size in [None, Some(0), Some(MAX_CA_BYTES + 1)] {
            if let Some(size) = size {
                File::create(&ca).unwrap().set_len(size).unwrap();
            }
            let endpoint: SqlServerConnectionConfig = serde_json::from_value(serde_json::json!({
                "host":"127.0.0.1", "database":"fixture", "tls":{"ca_file":ca}
            }))
            .unwrap();
            let mut connector = TcpConnector::default();
            let error = tokio::time::timeout(
                Duration::from_secs(5),
                connector.connect(&endpoint, Config::new()),
            )
            .await
            .unwrap()
            .unwrap_err();
            assert_eq!(error, Error::Tls);
            assert!(!format!("{error:?} {error}").contains("sensitive"));
            connector.shutdown().await.unwrap();
            assert!(connector.worker.is_none());
        }
    }
}
