// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arc_swap::ArcSwap;
use base64::prelude::*;
use futures::{Stream, StreamExt};
use notify::{Event, RecursiveMode, Watcher};
use otap_df_config::tls::TlsServerConfig;
use rustls::RootCertStore;
use rustls::pki_types::CertificateDer;
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::server::{ClientHello, ResolvesServerCert, WantsServerCert, WebPkiClientVerifier};
use rustls::sign::CertifiedKey;
use rustls::{
    ConfigBuilder, DigitallySignedStruct, DistinguishedName, ServerConfig, SignatureScheme,
    WantsVerifier,
};
use rustls_native_certs::load_native_certs;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{PrivateKeyDer, UnixTime};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime};
use tonic::transport::{Identity, ServerTlsConfig};

#[cfg(feature = "experimental-tls")]
use tokio::sync::OnceCell;

#[cfg(feature = "experimental-tls")]
use otap_df_config::tls::TlsClientConfig;

#[cfg(feature = "experimental-tls")]
use tonic::transport::{Certificate, ClientTlsConfig};

/// Maximum allowed size for TLS certificate and key files (4MB).
/// This limit is chosen to be generous enough for typical certificate chains (which are usually < 10KB)
/// while preventing potential OOM issues from loading extremely large files.
const MAX_TLS_FILE_SIZE: u64 = 4 * 1024 * 1024; // 4MB

/// Maximum number of concurrent TLS handshakes per receiver instance.
///
/// This is a conservative default that balances concurrency with resource usage:
/// - Allows concurrent handshakes to prevent slow clients from blocking others
/// - Limits memory overhead for pending handshake state
/// - May need adjustment based on actual workload characteristics
const MAX_CONCURRENT_HANDSHAKES: usize = 64;

/// Default interval between certificate reload checks (5 minutes).
/// This is used when no explicit reload_interval is configured.
const DEFAULT_RELOAD_INTERVAL_SECS: u64 = 300;

/// Minimum interval between CA certificate reloads to prevent rapid successive reloads.
/// Events arriving within this window after a reload will be debounced.
const CA_RELOAD_DEBOUNCE_SECS: u64 = 1;

/// Delay before reading file metadata after receiving a filesystem event.
/// This allows atomic rename operations to fully complete before we check the file identity.
/// On macOS, kqueue events can arrive before the rename operation is visible to stat().
const FS_EVENT_SETTLE_DELAY_MS: u64 = 50;

/// Converts native system certificates to PEM format.
///
/// Takes a `CertificateResult` from `rustls_native_certs::load_native_certs()` and
/// converts each certificate to PEM format with 64-character line wrapping.
/// Any errors encountered during loading are logged as warnings.
fn convert_native_certs_to_pem(cert_res: &rustls_native_certs::CertificateResult) -> Vec<u8> {
    let mut pem_data = Vec::new();

    for error in &cert_res.errors {
        log::warn!("Error loading native cert: {}", error);
    }

    for cert in &cert_res.certs {
        let base64_cert = BASE64_STANDARD.encode(cert.as_ref());
        // BASE64_STANDARD produces only ASCII, so working with chars is safe.
        // Wrap at 64 characters per line for PEM format.
        let wrapped: String = base64_cert
            .chars()
            .collect::<Vec<_>>()
            .chunks(64)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        let pem = format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
            wrapped
        );
        pem_data.extend_from_slice(pem.as_bytes());
    }

    pem_data
}

/// Loads TLS configuration for a server.
///
/// Returns `Ok(None)` when no cert/key material is provided, indicating TLS is disabled.
pub async fn load_server_tls_config(
    config: &TlsServerConfig,
) -> Result<Option<ServerTlsConfig>, io::Error> {
    // If neither cert nor key is provided, we assume TLS is disabled.
    // However, if one is provided, the other must be too.
    let (cert, key) = match (
        &config.config.cert_file,
        &config.config.key_file,
        &config.config.cert_pem,
        &config.config.key_pem,
    ) {
        (Some(cert_file), Some(key_file), _, _) => {
            let cert = read_file_with_limit_async(cert_file).await.map_err(|e| {
                log::error!("Failed to read cert file {:?}: {}", cert_file, e);
                e
            })?;
            let key = read_file_with_limit_async(key_file).await.map_err(|e| {
                log::error!("Failed to read key file {:?}: {}", key_file, e);
                e
            })?;
            (cert, key)
        }
        (None, None, Some(cert_pem), Some(key_pem)) => {
            (cert_pem.as_bytes().to_vec(), key_pem.as_bytes().to_vec())
        }
        (None, None, None, None) => {
            return Ok(None);
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "TLS configuration error: both certificate and key must be provided. \
                     Found cert_file={:?}, key_file={:?}, cert_pem={:?}, key_pem={:?}",
                    config.config.cert_file.is_some(),
                    config.config.key_file.is_some(),
                    config.config.cert_pem.is_some(),
                    config.config.key_pem.is_some()
                ),
            ));
        }
    };

    let identity = Identity::from_pem(cert, key);
    let tls_builder = ServerTlsConfig::new().identity(identity);

    // Note: Client CA/mTLS support is handled by build_reloadable_server_config instead.

    Ok(Some(tls_builder))
}

/// Loads TLS configuration for a client.
///
/// This is used by **exporters** and other components that initiate TLS connections.
///
/// Returns `Ok(None)` when TLS settings are empty and the endpoint URI is not `https://`.
///
/// # Known Limitations
///
/// **TODO: Hot Reload Not Implemented**
///
/// Unlike the receiver implementation (which uses `LazyReloadableCertResolver` for automatic
/// certificate reloading), exporter TLS configuration is static and loaded once at startup.
/// The `reload_interval` field in `TlsConfig` is present but currently unused for clients.
///
/// **Impact:** Exporters with expiring client certificates require process restart. This creates
/// a feature parity gap with receivers and an operational burden for long-running exporters
/// with short-lived certificates (e.g., certificates rotated every 24 hours).
///
/// **Implementation Complexity:** Adding hot reload for exporters requires either:
/// - Recreating the gRPC channel when certificates expire (may disrupt in-flight requests)
/// - Implementing a custom TLS connector with lazy certificate loading (complex integration
///   with tonic's transport layer)
///
/// Consider implementing certificate hot reload if this becomes an operational requirement.
#[cfg(feature = "experimental-tls")]
pub(crate) async fn load_client_tls_config(
    config: Option<&TlsClientConfig>,
    endpoint_uri: &str,
) -> Result<Option<ClientTlsConfig>, io::Error> {
    let wants_tls = endpoint_uri.starts_with("https://");

    let Some(config) = config else {
        // Go collector behavior: absence of a TLS block means "use scheme defaults".
        // - https:// => TLS enabled with default trust anchors
        // - http://  => plaintext
        if !wants_tls {
            return Ok(None);
        }

        let mut tls = ClientTlsConfig::new();
        tls = add_system_trust_anchors_if_enabled(tls, true).await?;
        return Ok(Some(tls));
    };

    let insecure = config.insecure.unwrap_or(false);
    let custom_ca_configured = config.ca_file.is_some()
        || config
            .ca_pem
            .as_ref()
            .is_some_and(|pem| !pem.trim().is_empty());

    // Align with Go configtls.ClientConfig.LoadTLSConfig:
    // when insecure=true and no custom CA is configured, return None and let the
    // endpoint scheme decide whether the connection is plaintext or TLS.
    if insecure && !custom_ca_configured {
        return Ok(None);
    }

    if let Some(true) = config.insecure_skip_verify {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "TLS configuration error: insecure_skip_verify=true is not supported by the current Rust OTLP client implementation (tonic/rustls). \
             Remove insecure_skip_verify or set it to false.\n\n\
             TODO: Implement this only with an explicit, clearly-labeled dangerous verifier override.",
        ));
    }

    let client_cert_configured = config.config.cert_file.is_some()
        || config
            .config
            .cert_pem
            .as_ref()
            .is_some_and(|pem| !pem.trim().is_empty());
    let client_key_configured = config.config.key_file.is_some()
        || config
            .config
            .key_pem
            .as_ref()
            .is_some_and(|pem| !pem.trim().is_empty());

    // Note: Providing a TLS config block forces TLS regardless of scheme.

    let mut tls = ClientTlsConfig::new();

    // Domain name / SNI.
    if let Some(domain) = &config.server_name {
        tls = tls.domain_name(domain.clone());
    }

    // Validate trust anchors are configured.
    let include_system = config.include_system_ca_certs_pool.unwrap_or(true);
    let ca_configured = config.ca_file.is_some()
        || config
            .ca_pem
            .as_ref()
            .is_some_and(|pem| !pem.trim().is_empty());

    if !include_system && !ca_configured {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "TLS configuration error: no trust anchors configured. \
             Either provide ca_file/ca_pem or set include_system_ca_certs_pool to true (or omit it).",
        ));
    }

    // System CA pool.
    tls = add_system_trust_anchors_if_enabled(tls, include_system).await?;

    // Custom CA.
    if let Some(ca_file) = &config.ca_file {
        let ca_pem = read_file_with_limit_async(ca_file).await.map_err(|e| {
            log::error!("Failed to read CA file {:?}: {}", ca_file, e);
            e
        })?;
        tls = tls.ca_certificate(Certificate::from_pem(ca_pem));
    }
    if let Some(ca_pem) = &config.ca_pem {
        if ca_pem.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "TLS configuration error: ca_pem is set but empty or contains only whitespace",
            ));
        }
        tls = tls.ca_certificate(Certificate::from_pem(ca_pem.as_bytes()));
    }

    // Client identity (mTLS).
    if client_cert_configured || client_key_configured {
        if !(client_cert_configured && client_key_configured) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "TLS configuration error: both client certificate and key must be provided for mTLS",
            ));
        }

        // Match on all combinations of cert/key sources to avoid unnecessary allocations.
        // When using PEM strings, pass as_bytes() directly instead of copying to Vec.
        tls = match (
            (&config.config.cert_file, &config.config.cert_pem),
            (&config.config.key_file, &config.config.key_pem),
        ) {
            ((Some(cert_path), _), (Some(key_path), _)) => {
                let cert = read_file_with_limit_async(cert_path).await.map_err(|e| {
                    log::error!("Failed to read client cert file {:?}: {}", cert_path, e);
                    e
                })?;
                let key = read_file_with_limit_async(key_path).await.map_err(|e| {
                    log::error!("Failed to read client key file {:?}: {}", key_path, e);
                    e
                })?;
                tls.identity(Identity::from_pem(cert, key))
            }
            ((Some(cert_path), _), (None, Some(key_pem))) => {
                let cert = read_file_with_limit_async(cert_path).await.map_err(|e| {
                    log::error!("Failed to read client cert file {:?}: {}", cert_path, e);
                    e
                })?;
                tls.identity(Identity::from_pem(cert, key_pem.as_bytes()))
            }
            ((None, Some(cert_pem)), (Some(key_path), _)) => {
                let key = read_file_with_limit_async(key_path).await.map_err(|e| {
                    log::error!("Failed to read client key file {:?}: {}", key_path, e);
                    e
                })?;
                tls.identity(Identity::from_pem(cert_pem.as_bytes(), key))
            }
            ((None, Some(cert_pem)), (None, Some(key_pem))) => {
                tls.identity(Identity::from_pem(cert_pem.as_bytes(), key_pem.as_bytes()))
            }
            _ => unreachable!("validation ensures both cert and key are configured"),
        };
    }

    Ok(Some(tls))
}

#[cfg(feature = "experimental-tls")]
async fn add_system_trust_anchors_if_enabled(
    tls: ClientTlsConfig,
    include_system: bool,
) -> Result<ClientTlsConfig, io::Error> {
    if !include_system {
        return Ok(tls);
    }

    // Use cached system roots if available, otherwise load them.
    // Cloning the Vec<CertificateDer> is cheap (ref-counted inner data).
    // OnceCell ensures only one task loads the certificates, preventing race conditions.
    static SYSTEM_ROOTS: OnceCell<Vec<CertificateDer<'static>>> = OnceCell::const_new();

    let roots = SYSTEM_ROOTS
        .get_or_try_init(|| async {
            // Loading native certificates involves blocking I/O (e.g. reading from disk or
            // querying the OS keychain). We must offload this to a blocking thread to avoid
            // stalling the async runtime.
            tokio::task::spawn_blocking(|| {
                let native = load_native_certs();
                if !native.errors.is_empty() {
                    log::warn!(
                        "Errors while loading native certificates (count={}): first={:?}",
                        native.errors.len(),
                        native.errors.first()
                    );
                }
                native.certs
            })
            .await
            .map_err(io::Error::other)
        })
        .await?
        .clone();

    let mut store = RootCertStore::empty();
    // Best-effort: accept that some system certs might not parse.
    let (added, ignored) = store.add_parsable_certificates(roots);
    log::debug!(
        "Loaded {} system CA certificates ({} ignored)",
        added,
        ignored
    );

    Ok(tls.trust_anchors(store.roots))
}

/// Creates a TLS stream from a TCP listener stream and a TLS acceptor.
///
/// This function handles the TLS handshake for each incoming connection.
/// TLS handshake failures are logged and filtered out (non-fatal).
/// Transport-level listener errors are propagated to terminate the server.
///
/// # Concurrency
///
/// TLS handshakes are performed concurrently (up to `MAX_CONCURRENT_HANDSHAKES`) to prevent
/// slow or malicious clients from blocking other connections. This is important because
/// TLS handshakes involve network round-trips and can take significant time.
///
/// When the maximum concurrent handshakes limit is reached, backpressure is applied:
/// new connections wait in the OS TCP accept queue until a handshake slot becomes available.
/// This prevents unbounded resource consumption while maintaining high throughput.
pub fn create_tls_stream<S, T>(
    listener_stream: S,
    tls_acceptor: tokio_rustls::TlsAcceptor,
    handshake_timeout: Option<Duration>,
) -> impl Stream<Item = Result<tokio_rustls::server::TlsStream<T>, io::Error>>
where
    S: Stream<Item = Result<T, io::Error>> + Send + 'static,
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Unpin + 'static,
{
    listener_stream
        .map(move |conn_res| {
            let acceptor = tls_acceptor.clone();
            async move {
                match conn_res {
                    Ok(conn) => {
                        // Try TLS handshake
                        let handshake_future = acceptor.accept(conn);
                        let timeout_duration = handshake_timeout.unwrap_or(Duration::from_secs(10));

                        match tokio::time::timeout(timeout_duration, handshake_future).await {
                            Ok(Ok(stream)) => Some(Ok::<_, io::Error>(stream)),
                            Ok(Err(e)) => {
                                // TLS handshake failed - log and continue
                                log::warn!("TLS handshake failed: {}", e);
                                None
                            }
                            Err(_) => {
                                log::warn!("TLS handshake timed out");
                                None
                            }
                        }
                    }
                    Err(e) => {
                        // Transport-level listener error - propagate to terminate server
                        Some(Err(e))
                    }
                }
            }
        })
        // Allow concurrent handshakes to prevent slow/malicious clients from blocking others
        .buffer_unordered(MAX_CONCURRENT_HANDSHAKES)
        // Filter out failed handshakes (None values)
        .filter_map(|res| async move { res })
}

/// A certificate resolver that lazily reloads TLS certificates with throttled file modification time (mtime) checks.
///
/// # Performance characteristics
///
/// - Mtime checks for the certificate and key files are throttled by a configurable interval (`check_interval_secs`)
///   to minimize filesystem operations. This means the filesystem is not polled on every handshake, but only after
///   the specified interval has elapsed since the last check.
/// - If a file change is detected during a TLS handshake (i.e., the mtime has changed since the last check),
///   the certificate and key are reloaded asynchronously. This ensures that new certificates are picked up without
///   requiring a server restart, while avoiding excessive filesystem access.
/// - Reloads happen in the context of TLS handshakes. Certificate updates are applied to subsequent connections after
///   the async reload completes, which typically happens within a short time after file modification is detected.
#[derive(Debug)]
pub struct LazyReloadableCertResolver {
    /// Current certificate
    cert_key: Arc<ArcSwap<CertifiedKey>>,

    /// File paths
    cert_path: PathBuf,
    key_path: PathBuf,

    /// Last known modification times (Arc for sharing with async reload tasks)
    cert_mtime: Arc<AtomicU64>,
    key_mtime: Arc<AtomicU64>,

    /// Last time we checked mtime (unix timestamp in seconds)
    last_check_time: AtomicU64,

    /// Minimum interval between mtime checks (seconds)
    check_interval_secs: u64,

    /// Reload lock (Arc for sharing with async reload tasks)
    is_reloading: Arc<AtomicBool>,
}

impl LazyReloadableCertResolver {
    /// Creates a new LazyReloadableCertResolver
    pub fn new(
        cert_path: PathBuf,
        key_path: PathBuf,
        check_interval: Option<Duration>,
    ) -> Result<Self, io::Error> {
        let cert_key = load_certified_key_sync(&cert_path, &key_path)?;
        let cert_mtime = get_mtime(&cert_path)?;
        let key_mtime = get_mtime(&key_path)?;
        let now = current_timestamp();

        Ok(Self {
            cert_key: Arc::new(ArcSwap::from_pointee(cert_key)),
            cert_path,
            key_path,
            cert_mtime: Arc::new(AtomicU64::new(cert_mtime)),
            key_mtime: Arc::new(AtomicU64::new(key_mtime)),
            last_check_time: AtomicU64::new(now),
            check_interval_secs: check_interval
                .map(|d| d.as_secs())
                .unwrap_or(DEFAULT_RELOAD_INTERVAL_SECS), // Default: 5 minutes
            is_reloading: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Check if enough time has passed, then check mtime and reload if needed.
    pub fn check_and_reload_if_interval_expired(&self) -> bool {
        let now = current_timestamp();
        let last_check = self.last_check_time.load(Ordering::Relaxed);

        // Fast path: interval not expired yet
        if now.saturating_sub(last_check) < self.check_interval_secs {
            return false; // Skip mtime check entirely
        }

        // Interval expired - try to win the check race (leader election)
        if self
            .last_check_time
            .compare_exchange(last_check, now, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Another thread won - they'll handle the check.
            // Re-check the fast path: if the winner updated last_check_time, interval should be unexpired.
            let updated_last_check = self.last_check_time.load(Ordering::Relaxed);
            if now.saturating_sub(updated_last_check) < self.check_interval_secs {
                return false;
            }
            // If still expired, we can retry once (rare), or just return false to avoid spinning.
            // For simplicity, just return false here; a loop could be added for more aggressive detection.
            return false;
        }

        // We won - check mtimes
        let current_cert_mtime = match get_mtime(&self.cert_path) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to check cert mtime: {}", e);
                return false;
            }
        };

        let current_key_mtime = match get_mtime(&self.key_path) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to check key mtime: {}", e);
                return false;
            }
        };

        // Compare with cached mtimes
        let last_cert_mtime = self.cert_mtime.load(Ordering::Relaxed);
        let last_key_mtime = self.key_mtime.load(Ordering::Relaxed);

        if current_cert_mtime == last_cert_mtime && current_key_mtime == last_key_mtime {
            return false; // No change
        }

        // Files changed! Spawn async reload task
        if self
            .is_reloading
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return false; // Another thread already reloading
        }

        // Clone what we need for the async task
        let cert_path = self.cert_path.clone();
        let key_path = self.key_path.clone();
        let cert_key = Arc::clone(&self.cert_key);
        let cert_mtime = Arc::clone(&self.cert_mtime);
        let key_mtime = Arc::clone(&self.key_mtime);
        let is_reloading = Arc::clone(&self.is_reloading);

        // Spawn async reload - doesn't block the current handshake
        // Fire-and-forget: we intentionally don't await the task.
        // Using drop() to explicitly ignore the JoinHandle and satisfy clippy::let_underscore_future.
        drop(tokio::spawn(async move {
            match load_certified_key_async(&cert_path, &key_path).await {
                Ok(new_cert) => {
                    cert_key.store(Arc::new(new_cert));
                    cert_mtime.store(current_cert_mtime, Ordering::Relaxed);
                    key_mtime.store(current_key_mtime, Ordering::Relaxed);
                    log::info!(
                        "TLS certificate reloaded asynchronously: cert={:?}, key={:?}",
                        cert_path,
                        key_path
                    );
                }
                Err(e) => {
                    log::error!(
                        "Failed to reload cert asynchronously (keeping current): {}",
                        e
                    );
                }
            }
            is_reloading.store(false, Ordering::Release);
        }));

        // Return immediately - current handshake uses existing (valid) cert
        false
    }

    /// Returns the currently loaded certified key
    pub fn current_cert_key(&self) -> Arc<CertifiedKey> {
        self.cert_key.load_full()
    }
}

impl ResolvesServerCert for LazyReloadableCertResolver {
    fn resolve(&self, _client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        // Lazy check: only if interval expired (no overhead on most requests)
        let _ = self.check_and_reload_if_interval_expired();

        // Return current cert (wait-free)
        Some(self.cert_key.load_full())
    }
}

/// Internal state for the CA file watcher callback.
///
/// # Design: Why Blocking I/O Here is Acceptable
///
/// This callback runs in the notify crate's dedicated OS thread, not in the tokio
/// async runtime. The blocking operations (std::fs::metadata, std::thread::sleep)
/// only affect the watcher thread, which exists solely to monitor file changes.
///
/// Since CA reloads are rare (minutes/hours apart), the performance impact is
/// negligible. The TLS handshake path remains wait-free - just an atomic pointer load.
///
/// Alternative: A channel-based bridge to a tokio worker task would eliminate blocking
/// entirely, but adds complexity for minimal benefit in this use case.
struct CaWatcherState {
    /// The verifier to update on reload (shared with ReloadableClientCaVerifier).
    /// Arc allows sharing between watcher and verifier, ArcSwap enables atomic updates.
    inner: Arc<ArcSwap<Arc<dyn ClientCertVerifier>>>,
    /// Canonical path to match against events
    watched_path: PathBuf,
    /// Original path for reloading the file
    reload_path: PathBuf,
    /// Whether to include system CAs
    include_system_cas: bool,
    /// Last known file identity (inode on Unix)
    last_identity: Arc<AtomicU64>,
    /// Timestamp of last reload (for debouncing)
    last_reload: Arc<AtomicU64>,
    /// Lock to prevent concurrent reloads
    is_reloading: Arc<AtomicBool>,
}

impl CaWatcherState {
    /// Create a new watcher state.
    fn new(
        ca_file_path: &Path,
        inner: Arc<ArcSwap<Arc<dyn ClientCertVerifier>>>,
        ca_path: PathBuf,
        include_system_cas: bool,
    ) -> Result<Self, io::Error> {
        let watched_path = std::fs::canonicalize(&ca_path).unwrap_or_else(|_| ca_path.clone());
        let initial_identity = get_file_identity(ca_file_path).unwrap_or(0);

        Ok(Self {
            inner,
            watched_path,
            reload_path: ca_path,
            include_system_cas,
            last_identity: Arc::new(AtomicU64::new(initial_identity)),
            last_reload: Arc::new(AtomicU64::new(0)),
            is_reloading: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Handle a file system event.
    fn handle_event(&self, res: Result<Event, notify::Error>) {
        match res {
            Ok(event) => self.process_event(event),
            Err(e) => log::warn!("File watcher error: {}", e),
        }
    }

    /// Process a file system event, potentially triggering a reload.
    fn process_event(&self, event: Event) {
        log::debug!("File watcher event: {:?}", event);

        // Filter out irrelevant event types early (before expensive path checks)
        if matches!(event.kind, notify::EventKind::Access(_)) {
            return;
        }

        if !self.is_event_for_watched_file(&event) {
            return;
        }

        log::debug!("Event matches our CA file, proceeding with reload check");

        // Small delay to allow filesystem operations to complete (e.g., atomic renames).
        // This blocks the notify thread briefly, but is acceptable because:
        // - CA reloads are rare (days/weeks apart)
        // - notify buffers events internally
        // - 50ms won't cause meaningful event loss
        std::thread::sleep(Duration::from_millis(FS_EVENT_SETTLE_DELAY_MS));

        if !self.should_reload() {
            return;
        }

        self.perform_reload();
    }

    /// Check if the event is for the file we're watching.
    fn is_event_for_watched_file(&self, event: &Event) -> bool {
        let is_match = event.paths.iter().any(|p| {
            if p == &self.watched_path {
                return true;
            }
            // Try canonicalizing the event path if direct match fails
            std::fs::canonicalize(p)
                .map(|canon_p| canon_p == self.watched_path)
                .unwrap_or(false)
        });

        if !is_match {
            log::debug!(
                "Event not for our file. Event paths: {:?}, watched: {:?}",
                event.paths,
                self.watched_path
            );
        }

        is_match
    }

    /// Check if we should reload based on file identity and debouncing.
    fn should_reload(&self) -> bool {
        // Check if file identity (inode) has changed
        let current_identity = match get_file_identity(&self.reload_path) {
            Ok(id) => id,
            Err(e) => {
                log::debug!("Failed to get file identity, skipping reload: {}", e);
                return false;
            }
        };

        let prev_identity = self.last_identity.load(Ordering::Relaxed);
        if current_identity == prev_identity {
            log::debug!("File identity unchanged, skipping reload");
            return false;
        }
        log::debug!(
            "File identity changed from {} to {}",
            prev_identity,
            current_identity
        );

        // Check debounce window
        let now = current_timestamp();
        let last = self.last_reload.load(Ordering::Relaxed);
        if now.saturating_sub(last) < CA_RELOAD_DEBOUNCE_SECS {
            log::debug!("Debouncing CA file change event");
            return false;
        }

        // Try to acquire reload lock
        if self
            .is_reloading
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            log::debug!("CA reload already in progress, skipping");
            return false;
        }

        true
    }

    /// Perform the actual CA certificate reload.
    fn perform_reload(&self) {
        log::info!(
            "CA certificate file changed, reloading: {:?}",
            self.reload_path
        );

        // Note: There's a theoretical TOCTOU between getting identity and reading the file.
        // If the file changes in between, we may store a stale identity, causing one extra
        // reload on the next event. This is harmless and the debounce prevents rapid retries.
        let current_identity = get_file_identity(&self.reload_path).unwrap_or(0);
        let now = current_timestamp();

        match reload_ca_verifier(&self.reload_path, self.include_system_cas) {
            Ok(new_verifier) => {
                self.inner.store(Arc::new(new_verifier));
                self.last_identity
                    .store(current_identity, Ordering::Relaxed);
                self.last_reload.store(now, Ordering::Relaxed);
                log::info!("Successfully reloaded client CA certificates");
            }
            Err(e) => {
                log::error!("Failed to reload CA certificates (keeping previous): {}", e);
            }
        }

        self.is_reloading.store(false, Ordering::Release);
    }
}

/// A dynamically reloadable client certificate verifier for mTLS.
///
/// This verifier supports zero-downtime hot-reload of client CA certificates through
/// file system watching. When the CA certificate file is modified, the verifier
/// automatically reloads the CA store without interrupting existing connections.
///
/// # Performance
///
/// The TLS handshake hot path is wait-free - just an atomic pointer load with no
/// filesystem access or blocking. Reload operations happen in a separate watcher
/// thread and don't impact concurrent connections.
///
/// # Industry Standard Approach
///
/// This implementation follows the patterns used by Envoy, Linkerd, and other
/// production service meshes:
///
/// 1. **File System Watching**: Uses OS-native file notifications (inotify on Linux,
///    kqueue on macOS, FSEvents on macOS) for immediate detection of certificate changes.
///
/// 2. **Atomic Swap**: Uses `ArcSwap` for lock-free, wait-free reads during TLS handshakes.
///    New connections get the updated CA store; existing connections are unaffected.
///
/// 3. **Debouncing**: Multiple rapid file changes are coalesced to avoid excessive reloads.
///
/// 4. **Graceful Degradation**: If reload fails, the previous valid CA store is retained.
///
/// # Usage
///
/// ```ignore
/// let verifier = ReloadableClientCaVerifier::new_with_file_watch(
///     ca_file_path,
///     include_system_cas,
/// )?;
/// ```
pub struct ReloadableClientCaVerifier {
    /// The current client certificate verifier (atomically swappable).
    ///
    /// Architecture: Arc<ArcSwap<Arc<dyn ClientCertVerifier>>>
    ///
    /// Why three layers of Arc?
    /// 1. **Outer Arc**: Shared ownership between the verifier and watcher callback.
    ///    Multiple threads need access to the same ArcSwap (hot path + reload path).
    ///
    /// 2. **ArcSwap**: Atomic pointer swap for lock-free reads during TLS handshakes.
    ///    Allows updating the verifier without blocking concurrent connections.
    ///
    /// 3. **Inner Arc**: rustls requires ClientCertVerifier to be in an Arc for trait
    ///    object lifetime management. The WebPkiClientVerifier we swap is Arc<dyn ...>.
    ///
    /// Performance: Hot path is just `inner.load()` - a single atomic pointer load.
    inner: Arc<ArcSwap<Arc<dyn ClientCertVerifier>>>,

    /// Path to the CA file being watched (if file-based)
    ca_file_path: Option<PathBuf>,

    /// Whether to include system CA certificates
    include_system_cas: bool,

    /// Static CA PEM data (if not file-based)
    static_ca_pem: Option<Vec<u8>>,

    /// File watcher handle (kept alive to continue watching)
    /// Using Box<dyn Watcher> to support both RecommendedWatcher and PollWatcher
    #[allow(dead_code)]
    watcher: Option<Box<dyn Watcher + Send + Sync>>,
}

impl fmt::Debug for ReloadableClientCaVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReloadableClientCaVerifier")
            .field("ca_file_path", &self.ca_file_path)
            .field("include_system_cas", &self.include_system_cas)
            .field("has_static_ca", &self.static_ca_pem.is_some())
            .field("watching", &self.watcher.is_some())
            .finish()
    }
}

impl ReloadableClientCaVerifier {
    /// Creates a new reloadable client CA verifier with file system watching.
    ///
    /// This is the recommended method for production mTLS deployments where
    /// CA certificates may be rotated frequently (e.g., short-lived certificates
    /// from SPIFFE/SPIRE or cert-manager).
    ///
    /// # Arguments
    ///
    /// * `ca_file_path` - Path to the PEM-encoded CA certificate file
    /// * `include_system_cas` - Whether to include system root CA certificates
    ///
    /// # Returns
    ///
    /// Returns the verifier wrapped in an `Arc` for thread-safe sharing.
    pub fn new_with_file_watch(
        ca_file_path: PathBuf,
        include_system_cas: bool,
    ) -> Result<Arc<Self>, io::Error> {
        // Initial load
        let ca_pem = read_file_with_limit_sync(&ca_file_path)?;
        log::debug!("Initial CA PEM size: {} bytes", ca_pem.len());
        let verifier = build_webpki_verifier(&ca_pem, include_system_cas)?;

        let inner = Arc::new(ArcSwap::from_pointee(verifier));
        let inner_for_watcher = Arc::clone(&inner);
        let ca_path_for_watcher = ca_file_path.clone();
        let include_system_for_watcher = include_system_cas;

        // Set up file watcher
        let watcher = Self::setup_file_watcher(
            &ca_file_path,
            inner_for_watcher,
            ca_path_for_watcher,
            include_system_for_watcher,
        )?;

        Ok(Arc::new(Self {
            inner,
            ca_file_path: Some(ca_file_path),
            include_system_cas,
            static_ca_pem: None,
            watcher: Some(watcher),
        }))
    }

    /// Creates a verifier from in-memory PEM data (no file watching).
    ///
    /// Use this when CA certificates are provided via configuration strings
    /// rather than files. This verifier will not support hot-reload.
    pub fn new_from_pem(ca_pem: Vec<u8>, include_system_cas: bool) -> Result<Arc<Self>, io::Error> {
        let verifier = build_webpki_verifier(&ca_pem, include_system_cas)?;

        Ok(Arc::new(Self {
            inner: Arc::new(ArcSwap::from_pointee(verifier)),
            ca_file_path: None,
            include_system_cas,
            static_ca_pem: Some(ca_pem),
            watcher: None,
        }))
    }

    /// Creates a verifier with interval-based polling (fallback for systems without file watching).
    ///
    /// This is less efficient than file watching but works on all platforms and file systems.
    pub fn new_with_polling(
        ca_file_path: PathBuf,
        include_system_cas: bool,
        poll_interval: Duration,
    ) -> Result<Arc<Self>, io::Error> {
        // Initial load
        let ca_pem = read_file_with_limit_sync(&ca_file_path)?;
        let verifier = build_webpki_verifier(&ca_pem, include_system_cas)?;

        let inner = Arc::new(ArcSwap::from_pointee(verifier));
        let inner_for_poll = Arc::clone(&inner);
        let ca_path_for_poll = ca_file_path.clone();

        // Set up polling watcher
        let watcher = Self::setup_polling_watcher(
            &ca_file_path,
            inner_for_poll,
            ca_path_for_poll,
            include_system_cas,
            poll_interval,
        )?;

        Ok(Arc::new(Self {
            inner,
            ca_file_path: Some(ca_file_path),
            include_system_cas,
            static_ca_pem: None,
            watcher: Some(watcher),
        }))
    }

    /// Sets up a file system watcher using OS-native notifications.
    fn setup_file_watcher(
        ca_file_path: &Path,
        inner: Arc<ArcSwap<Arc<dyn ClientCertVerifier>>>,
        ca_path: PathBuf,
        include_system_cas: bool,
    ) -> Result<Box<dyn Watcher + Send + Sync>, io::Error> {
        // Initialize watcher state
        let state = CaWatcherState::new(ca_file_path, inner, ca_path.clone(), include_system_cas)?;

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            state.handle_event(res);
        })
        .map_err(io::Error::other)?;

        // Watch the parent directory instead of the file itself. This is necessary because:
        // 1. Atomic file replacements (mv tmp ca.crt) create a new inode - watching the old
        //    file would lose track when it's replaced.
        // 2. Kubernetes ConfigMaps/Secrets use symlink swapping, which also requires
        //    watching the parent directory to detect changes.
        // 3. Many editors (vim, etc.) use atomic save patterns that replace the file.
        // Events are filtered in `is_event_for_watched_file()` to only process changes
        // to our specific CA file.
        let parent_dir = ca_file_path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "CA file has no parent directory",
            )
        })?;
        let parent_dir =
            std::fs::canonicalize(parent_dir).unwrap_or_else(|_| parent_dir.to_path_buf());

        watcher
            .watch(&parent_dir, RecursiveMode::NonRecursive)
            .map_err(io::Error::other)?;

        log::info!(
            "File watcher set up for CA certificates: {:?} (watching parent: {:?})",
            ca_file_path,
            parent_dir
        );

        Ok(Box::new(watcher))
    }

    /// Sets up a polling-based watcher for environments where native watching isn't reliable.
    fn setup_polling_watcher(
        ca_file_path: &Path,
        inner: Arc<ArcSwap<Arc<dyn ClientCertVerifier>>>,
        ca_path: PathBuf,
        include_system_cas: bool,
        poll_interval: Duration,
    ) -> Result<Box<dyn Watcher + Send + Sync>, io::Error> {
        let is_reloading = Arc::new(AtomicBool::new(false));

        let config = notify::Config::default().with_poll_interval(poll_interval);

        let mut watcher = notify::PollWatcher::new(
            move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    if !matches!(
                        event.kind,
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                    ) {
                        return;
                    }

                    if is_reloading
                        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                        .is_err()
                    {
                        return;
                    }

                    log::info!(
                        "CA certificate file changed (polling), reloading: {:?}",
                        ca_path
                    );

                    match reload_ca_verifier(&ca_path, include_system_cas) {
                        Ok(new_verifier) => {
                            inner.store(Arc::new(new_verifier));
                            log::info!("Successfully reloaded client CA certificates");
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to reload CA certificates (keeping previous): {}",
                                e
                            );
                        }
                    }

                    is_reloading.store(false, Ordering::Release);
                }
                Err(e) => {
                    log::warn!("Poll watcher error: {}", e);
                }
            },
            config,
        )
        .map_err(io::Error::other)?;

        watcher
            .watch(ca_file_path, RecursiveMode::NonRecursive)
            .map_err(io::Error::other)?;

        log::info!(
            "Poll watcher set up for CA certificates: {:?} (interval: {:?})",
            ca_file_path,
            poll_interval
        );

        Ok(Box::new(watcher))
    }

    /// Returns the current verifier for debugging/testing purposes.
    #[allow(dead_code)]
    #[must_use]
    pub fn current_verifier(&self) -> Arc<Arc<dyn ClientCertVerifier>> {
        self.inner.load_full()
    }
}

/// Implements the ClientCertVerifier trait by delegating to the inner verifier.
///
/// This trait is from rustls's `danger` module, which requires careful handling
/// to ensure security. Safety is ensured by delegating all verification to a
/// properly constructed `WebPkiClientVerifier` which performs full certificate
/// chain validation.
impl ClientCertVerifier for ReloadableClientCaVerifier {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        // Root hints are CA Distinguished Names sent in CertificateRequest to help
        // clients choose which certificate to send (e.g., if client has multiple certs).
        //
        // Problem: This method requires returning a &'static slice, but our CA verifier
        // can change at any time due to hot-reload. We can't safely return a reference
        // to the current CA's DNs because they might be swapped out mid-handshake.
        //
        // Solution: Return empty slice. Clients will send their certificate anyway (they
        // just won't have the hint about which CA we trust). This is TLS-spec compliant
        // and what Envoy does for dynamic CAs.
        //
        // Impact: Clients with multiple certificates might send the wrong one first,
        // causing a retry. In practice, most clients have only one cert, so no issue.
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        now: UnixTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        // Load current verifier (wait-free, atomic)
        let verifier = self.inner.load();
        verifier.verify_client_cert(end_entity, intermediates, now)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        let verifier = self.inner.load();
        verifier.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        let verifier = self.inner.load();
        verifier.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        let verifier = self.inner.load();
        verifier.supported_verify_schemes()
    }

    fn client_auth_mandatory(&self) -> bool {
        let verifier = self.inner.load();
        verifier.client_auth_mandatory()
    }
}

/// Builds a WebPkiClientVerifier from PEM-encoded CA certificates.
fn build_webpki_verifier(
    ca_pem: &[u8],
    include_system_cas: bool,
) -> Result<Arc<dyn ClientCertVerifier>, io::Error> {
    let mut roots = RootCertStore::empty();

    // Add system CAs if requested
    if include_system_cas {
        let system_certs = load_native_certs();
        for error in &system_certs.errors {
            log::warn!("Error loading native cert: {}", error);
        }
        for cert in system_certs.certs {
            if let Err(e) = roots.add(cert) {
                log::warn!("Failed to add system cert: {}", e);
            }
        }
    }

    // Add user-provided CAs
    let mut reader = io::BufReader::new(ca_pem);
    let mut count = 0;
    for cert in CertificateDer::pem_reader_iter(&mut reader) {
        let cert = cert.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        roots
            .add(cert)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        count += 1;
    }

    if roots.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No CA certificates loaded",
        ));
    }

    log::debug!("Built verifier with {} CA certificates", count);

    WebPkiClientVerifier::builder(roots.into())
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Reloads the CA verifier from a file path.
fn reload_ca_verifier(
    ca_path: &Path,
    include_system_cas: bool,
) -> Result<Arc<dyn ClientCertVerifier>, io::Error> {
    let ca_pem = read_file_with_limit_sync(ca_path)?;
    log::debug!("Reloaded CA PEM size: {} bytes", ca_pem.len());
    build_webpki_verifier(&ca_pem, include_system_cas)
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn get_mtime(path: &Path) -> Result<u64, io::Error> {
    std::fs::metadata(path)?
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(io::Error::other)
}

/// Get a unique file identifier that changes when the file is replaced.
/// On Unix, this uses the inode number which changes on atomic rename.
/// On other platforms, falls back to mtime.
#[cfg(unix)]
fn get_file_identity(path: &Path) -> Result<u64, io::Error> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.ino())
}

#[cfg(not(unix))]
fn get_file_identity(path: &Path) -> Result<u64, io::Error> {
    // On non-Unix platforms, fall back to mtime
    get_mtime(path)
}

/// Parses a certified key from PEM-encoded certificate and key bytes.
fn parse_certified_key(
    cert_pem: &[u8],
    key_pem: &[u8],
    cert_path_debug: &Path,
) -> Result<CertifiedKey, io::Error> {
    use std::io::BufReader;

    let certs: Vec<_> = CertificateDer::pem_reader_iter(&mut BufReader::new(cert_pem))
        .collect::<Result<_, _>>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if certs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("No certificates found in file: {:?}", cert_path_debug),
        ));
    }

    let key = PrivateKeyDer::from_pem_reader(&mut BufReader::new(key_pem))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let signing_key = rustls::crypto::ring::sign::any_supported_type(&key)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(CertifiedKey::new(certs, signing_key))
}

/// Async version for background reloads - doesn't block handshakes
async fn load_certified_key_async(
    cert_path: &Path,
    key_path: &Path,
) -> Result<CertifiedKey, io::Error> {
    // Use async file I/O - doesn't block
    let cert_pem = read_file_with_limit_async(cert_path).await?;
    let key_pem = read_file_with_limit_async(key_path).await?;

    parse_certified_key(&cert_pem, &key_pem, cert_path)
}

/// Sync version for initial load in constructor
fn load_certified_key_sync(cert_path: &Path, key_path: &Path) -> Result<CertifiedKey, io::Error> {
    let cert_pem = read_file_with_limit_sync(cert_path)?;
    let key_pem = read_file_with_limit_sync(key_path)?;

    parse_certified_key(&cert_pem, &key_pem, cert_path)
}

/// Builds a reloadable server config from the given configuration.
///
/// This function creates a TLS server configuration with support for:
///
/// 1. **Server Certificate Hot-Reload**: Using `LazyReloadableCertResolver` for
///    automatic reloading of server cert/key when files change.
///
/// 2. **Client CA Certificate Hot-Reload (mTLS)**: Using `ReloadableClientCaVerifier`
///    for zero-downtime reload of client CA certificates. This is particularly useful
///    for environments with frequently rotating certificates (SPIFFE/SPIRE, cert-manager).
///
/// # Hot-Reload Behavior
///
/// - **Server certificates**: Checked on a configurable interval (`reload_interval`).
/// - **Client CA certificates**: Reloaded immediately when file changes are detected
///   via file system notifications (when `watch_client_ca` is enabled), or on the
///   same interval as server certificates (when `watch_client_ca` is disabled).
///
/// # Arguments
///
/// * `config` - TLS server configuration containing paths or PEM data for certificates
///
/// # Returns
///
/// An `Arc<ServerConfig>` ready for use with `TlsAcceptor`.
pub async fn build_reloadable_server_config(
    config: &TlsServerConfig,
) -> Result<Arc<ServerConfig>, io::Error> {
    let check_interval = config.config.reload_interval;

    let builder = ServerConfig::builder();

    // Determine client auth configuration
    let builder = build_client_auth(config, builder).await?;

    // Cert resolver
    let mut server_config = if let (Some(cert_path), Some(key_path)) =
        (&config.config.cert_file, &config.config.key_file)
    {
        // File-based: use lazy reloader
        let cert_resolver = Arc::new(LazyReloadableCertResolver::new(
            cert_path.clone(),
            key_path.clone(),
            check_interval,
        )?);
        builder.with_cert_resolver(cert_resolver)
    } else if let (Some(cert_pem), Some(key_pem)) =
        (&config.config.cert_pem, &config.config.key_pem)
    {
        // PEM-based: static
        let certs = CertificateDer::pem_reader_iter(&mut io::BufReader::new(cert_pem.as_bytes()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let key = PrivateKeyDer::from_pem_reader(&mut io::BufReader::new(key_pem.as_bytes()))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        builder
            .with_single_cert(certs, key)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "TLS requires either cert_file/key_file or cert_pem/key_pem",
        ));
    };

    server_config.alpn_protocols = vec![b"h2".to_vec()];

    Ok(Arc::new(server_config))
}

/// Builds the client authentication (mTLS) configuration.
///
/// This function handles three modes of client CA certificate management:
///
/// 1. **File-based with file watching** (`watch_client_ca: true`):
///    Uses OS-native file notifications for immediate reload when CA files change.
///    Best for environments with frequent certificate rotation.
///
/// 2. **File-based with interval polling** (`watch_client_ca: false`, `client_ca_file` set):
///    Falls back to interval-based checking (using `reload_interval`).
///    More compatible with all file systems but less responsive.
///
/// 3. **PEM-based static** (`client_ca_pem` set):
///    CA certificates provided inline; no hot-reload supported.
async fn build_client_auth(
    config: &TlsServerConfig,
    builder: ConfigBuilder<ServerConfig, WantsVerifier>,
) -> Result<ConfigBuilder<ServerConfig, WantsServerCert>, io::Error> {
    let include_system_cas = config.include_system_ca_certs_pool.unwrap_or(false);
    let watch_enabled = config.watch_client_ca;
    let check_interval = config
        .config
        .reload_interval
        .unwrap_or(Duration::from_secs(DEFAULT_RELOAD_INTERVAL_SECS));

    // Check if we have any CA configuration
    let has_ca_file = config.client_ca_file.is_some();
    let has_ca_pem = config.client_ca_pem.is_some();

    if !has_ca_file && !has_ca_pem && !include_system_cas {
        // No client auth configured
        log::debug!("No client CA configured, disabling client authentication");
        return Ok(builder.with_no_client_auth());
    }

    // Build the appropriate verifier based on configuration
    if let Some(ca_file) = &config.client_ca_file {
        // File-based CA configuration
        let verifier = if watch_enabled {
            log::info!(
                "Configuring mTLS with file watching for CA certificates: {:?}",
                ca_file
            );
            ReloadableClientCaVerifier::new_with_file_watch(ca_file.clone(), include_system_cas)?
        } else {
            log::info!(
                "Configuring mTLS with polling for CA certificates: {:?} (interval: {:?})",
                ca_file,
                check_interval
            );
            ReloadableClientCaVerifier::new_with_polling(
                ca_file.clone(),
                include_system_cas,
                check_interval,
            )?
        };

        Ok(builder.with_client_cert_verifier(verifier))
    } else if let Some(ca_pem) = &config.client_ca_pem {
        // PEM-based (static) CA configuration
        log::info!("Configuring mTLS with static PEM CA certificates");

        // For PEM-based, we need to combine with system CAs if requested
        let mut combined_pem = Vec::new();

        if include_system_cas {
            let cert_res = tokio::task::spawn_blocking(load_native_certs)
                .await
                .map_err(io::Error::other)?;
            combined_pem.extend_from_slice(&convert_native_certs_to_pem(&cert_res));
        }

        combined_pem.extend_from_slice(ca_pem.as_bytes());

        // Pass false because system CAs are already included in combined_pem above
        let verifier = ReloadableClientCaVerifier::new_from_pem(combined_pem, false)?;
        Ok(builder.with_client_cert_verifier(verifier))
    } else if include_system_cas {
        // Only system CAs (no user-provided CA)
        log::info!("Configuring mTLS with system CA certificates only");

        let cert_res = tokio::task::spawn_blocking(load_native_certs)
            .await
            .map_err(io::Error::other)?;
        let system_pem = convert_native_certs_to_pem(&cert_res);

        if system_pem.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "include_system_ca_certs_pool is true, but no CA certificates were loaded. \
                 Cannot enable mTLS without CA certificates.",
            ));
        }

        // Pass false because system CAs are already in system_pem
        let verifier = ReloadableClientCaVerifier::new_from_pem(system_pem, false)?;
        Ok(builder.with_client_cert_verifier(verifier))
    } else {
        // This shouldn't happen given the guard at the top, but handle it
        Ok(builder.with_no_client_auth())
    }
}

/// Builds a TLS acceptor from optional TLS configuration.
///
/// Returns `Ok(None)` if `tls_config` is `None` (TLS disabled).
/// Returns `Ok(Some(TlsAcceptor))` if TLS is successfully configured.
/// Returns `Err` if TLS configuration fails.
pub async fn build_tls_acceptor(
    tls_config: Option<&TlsServerConfig>,
) -> Result<Option<tokio_rustls::TlsAcceptor>, io::Error> {
    match tls_config {
        Some(config) => {
            let server_config = build_reloadable_server_config(config).await?;
            Ok(Some(tokio_rustls::TlsAcceptor::from(server_config)))
        }
        None => Ok(None),
    }
}

async fn read_file_with_limit_async(path: &Path) -> Result<Vec<u8>, io::Error> {
    let metadata = tokio::fs::metadata(path).await?;
    if metadata.len() > MAX_TLS_FILE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "File {:?} is too large ({} bytes). Max allowed is {} bytes.",
                path,
                metadata.len(),
                MAX_TLS_FILE_SIZE
            ),
        ));
    }
    tokio::fs::read(path).await
}

fn read_file_with_limit_sync(path: &Path) -> Result<Vec<u8>, io::Error> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_TLS_FILE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "File {:?} is too large ({} bytes). Max allowed is {} bytes.",
                path,
                metadata.len(),
                MAX_TLS_FILE_SIZE
            ),
        ));
    }
    std::fs::read(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::tls::TlsConfig;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    /// Check if OpenSSL CLI is available on the system.
    /// Returns `true` if `openssl version` succeeds, `false` otherwise.
    fn is_openssl_available() -> bool {
        Command::new("openssl")
            .arg("version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Skips the test if OpenSSL is not available, printing a clear message.
    /// Returns `true` if the test should be skipped.
    fn skip_if_no_openssl() -> bool {
        if !is_openssl_available() {
            eprintln!(
                "SKIPPED: OpenSSL CLI not found. Install OpenSSL to run this test. \
                 On macOS: `brew install openssl`, on Ubuntu: `apt-get install openssl`"
            );
            true
        } else {
            false
        }
    }

    /// Generate a self-signed certificate using OpenSSL CLI.
    ///
    /// # Panics
    /// Panics if OpenSSL is not installed or if cert generation fails.
    /// Tests using this should call `skip_if_no_openssl()` first for graceful handling.
    fn generate_cert(dir: &Path, name: &str, cn: &str) {
        // Generate Key and Cert in one go (self-signed)
        let output = Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-keyout",
                &format!("{}.key", name),
                "-out",
                &format!("{}.crt", name),
                "-days",
                "1",
                "-nodes",
                "-subj",
                &format!("/CN={}", cn),
                "-addext",
                "basicConstraints=critical,CA:TRUE",
            ])
            .current_dir(dir)
            .output()
            .expect(
                "Failed to execute openssl. Ensure OpenSSL CLI is installed: \
                 macOS: `brew install openssl`, Ubuntu: `apt-get install openssl`",
            );

        if !output.status.success() {
            panic!(
                "Certificate generation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[tokio::test]
    async fn test_lazy_reload_resolver() {
        if skip_if_no_openssl() {
            return;
        }
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        let cert_path = path.join("server.crt");
        let key_path = path.join("server.key");

        // 1. Generate initial cert
        generate_cert(path, "cert1", "localhost");
        let _ = fs::copy(path.join("cert1.crt"), &cert_path).expect("Copy cert1.crt");
        let _ = fs::copy(path.join("cert1.key"), &key_path).expect("Copy cert1.key");

        // 2. Create resolver with short interval
        let resolver = LazyReloadableCertResolver::new(
            cert_path.clone(),
            key_path.clone(),
            Some(Duration::from_millis(500)),
        )
        .expect("Failed to create resolver");

        let initial_cert = resolver.current_cert_key();
        assert!(!initial_cert.cert.is_empty());

        // 3. Wait for interval to expire
        tokio::time::sleep(Duration::from_millis(600)).await;

        // 4. Update cert file (ensure mtime changes)
        // Sleep a bit to ensure FS mtime granularity (some systems are 1s)
        tokio::time::sleep(Duration::from_millis(1100)).await;

        generate_cert(path, "cert2", "otherhost");
        let _ = fs::copy(path.join("cert2.crt"), &cert_path).expect("Copy cert2.crt");
        let _ = fs::copy(path.join("cert2.key"), &key_path).expect("Copy cert2.key");

        // 5. Trigger reload (async - returns false immediately)
        let reloaded = resolver.check_and_reload_if_interval_expired();
        assert!(!reloaded, "Async reload returns false immediately");

        // 6. Wait for async reload to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        let new_cert = resolver.current_cert_key();
        assert_ne!(initial_cert.cert, new_cert.cert, "Cert should have changed");

        // 7. Trigger again immediately - should not reload (interval not expired)
        let reloaded_again = resolver.check_and_reload_if_interval_expired();
        assert!(!reloaded_again, "Should not reload again immediately");
    }

    #[tokio::test]
    async fn test_load_server_tls_config_missing_key() {
        let config = TlsServerConfig {
            config: TlsConfig {
                cert_pem: Some("fake cert".to_string()),
                key_pem: None,
                cert_file: None,
                key_file: None,
                reload_interval: None,
            },
            client_ca_file: None,
            client_ca_pem: None,
            include_system_ca_certs_pool: None,
            watch_client_ca: false,
            handshake_timeout: None,
        };

        let result = load_server_tls_config(&config).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("TLS configuration error"));
    }

    #[tokio::test]
    async fn test_load_server_tls_config_missing_cert() {
        let config = TlsServerConfig {
            config: TlsConfig {
                cert_pem: None,
                key_pem: Some("fake key".to_string()),
                cert_file: None,
                key_file: None,
                reload_interval: None,
            },
            client_ca_file: None,
            client_ca_pem: None,
            include_system_ca_certs_pool: None,
            watch_client_ca: false,
            handshake_timeout: None,
        };

        let result = load_server_tls_config(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_server_tls_config_success_pem() {
        if skip_if_no_openssl() {
            return;
        }
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        generate_cert(path, "server", "localhost");

        let cert_pem = fs::read_to_string(path.join("server.crt")).expect("Failed to read cert");
        let key_pem = fs::read_to_string(path.join("server.key")).expect("Failed to read key");

        let config = TlsServerConfig {
            config: TlsConfig {
                cert_pem: Some(cert_pem),
                key_pem: Some(key_pem),
                cert_file: None,
                key_file: None,
                reload_interval: None,
            },
            client_ca_file: None,
            client_ca_pem: None,
            include_system_ca_certs_pool: None,
            watch_client_ca: false,
            handshake_timeout: None,
        };

        let result = load_server_tls_config(&config).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_load_server_tls_config_success_file() {
        if skip_if_no_openssl() {
            return;
        }
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        generate_cert(path, "server", "localhost");

        let cert_path = path.join("server.crt");
        let key_path = path.join("server.key");

        let config = TlsServerConfig {
            config: TlsConfig {
                cert_file: Some(cert_path),
                key_file: Some(key_path),
                cert_pem: None,
                key_pem: None,
                reload_interval: None,
            },
            client_ca_file: None,
            client_ca_pem: None,
            include_system_ca_certs_pool: None,
            watch_client_ca: false,
            handshake_timeout: None,
        };

        let result = load_server_tls_config(&config).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_reloadable_client_ca_verifier_file_watch() {
        if skip_if_no_openssl() {
            return;
        }
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        let ca_path = path.join("ca.crt");

        // 1. Generate initial CA cert
        generate_cert(path, "ca1", "TestCA1");
        let _ = fs::copy(path.join("ca1.crt"), &ca_path).expect("Copy ca1.crt");

        // 2. Create verifier with file watching
        let verifier = ReloadableClientCaVerifier::new_with_file_watch(ca_path.clone(), false)
            .expect("Failed to create verifier");

        // Verify it was created successfully
        assert!(verifier.client_auth_mandatory());

        // 3. Wait a bit for the watcher to be set up
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 4. Update CA file (ensure file system detects change)
        tokio::time::sleep(Duration::from_millis(1100)).await; // Ensure mtime changes

        generate_cert(path, "ca2", "TestCA2");
        let _ = fs::copy(path.join("ca2.crt"), &ca_path).expect("Copy ca2.crt");

        // 5. Wait for file watcher to trigger reload
        tokio::time::sleep(Duration::from_millis(500)).await;

        // The verifier should still work (we can't easily verify the CA changed,
        // but we can verify no errors occurred during reload)
        assert!(verifier.client_auth_mandatory());
    }

    #[tokio::test]
    async fn test_reloadable_client_ca_verifier_from_pem() {
        if skip_if_no_openssl() {
            return;
        }
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();

        // Generate a CA cert
        generate_cert(path, "ca", "TestCA");
        let ca_pem = fs::read(path.join("ca.crt")).expect("Failed to read CA cert");

        // Create verifier from PEM
        let verifier = ReloadableClientCaVerifier::new_from_pem(ca_pem, false)
            .expect("Failed to create verifier");

        // Verify it was created successfully
        assert!(verifier.client_auth_mandatory());
    }

    // Skipping on Windows and macOS due to flakiness: https://github.com/open-telemetry/otel-arrow/issues/1614
    #[tokio::test]
    #[cfg_attr(any(target_os = "windows", target_os = "macos"), ignore = "Skipping on Windows and macOS due to flakiness")]
    async fn test_build_reloadable_server_config_with_mtls() {
        if skip_if_no_openssl() {
            return;
        }
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();

        // Generate server cert and CA cert
        generate_cert(path, "server", "localhost");
        generate_cert(path, "ca", "TestCA");

        let config = TlsServerConfig {
            config: TlsConfig {
                cert_file: Some(path.join("server.crt")),
                key_file: Some(path.join("server.key")),
                cert_pem: None,
                key_pem: None,
                reload_interval: Some(Duration::from_secs(1)),
            },
            client_ca_file: Some(path.join("ca.crt")),
            client_ca_pem: None,
            include_system_ca_certs_pool: None,
            watch_client_ca: true,
            handshake_timeout: None,
        };

        let result = build_reloadable_server_config(&config).await;
        assert!(result.is_ok());

        let _server_config = result.unwrap();
        // Verify mTLS config was created successfully
        // Note: ServerConfig doesn't expose client_auth_mandatory directly,
        // but if we got here without error, mTLS is configured.
    }
}
