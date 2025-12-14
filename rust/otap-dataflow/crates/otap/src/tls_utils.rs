// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arc_swap::ArcSwap;
use base64::prelude::*;
use futures::{Stream, StreamExt};
use otap_df_config::tls::TlsServerConfig;
use rustls::RootCertStore;
use rustls::server::{ClientHello, ResolvesServerCert, WebPkiClientVerifier};
use rustls::sign::CertifiedKey;
use rustls_native_certs::load_native_certs;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime};
use tonic::transport::{Identity, ServerTlsConfig};

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

/// Returns true if the given URI uses the HTTPS scheme.
///
/// Trims leading whitespace before checking to handle common configuration errors.
/// Performs case-insensitive comparison per RFC 3986 (scheme is case-insensitive).
pub(crate) fn is_https_endpoint(uri: &str) -> bool {
    uri.trim_start()
        .get(..8)
        .is_some_and(|s| s.eq_ignore_ascii_case("https://"))
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
pub async fn load_client_tls_config(
    config: Option<&TlsClientConfig>,
    endpoint_uri: &str,
) -> Result<Option<ClientTlsConfig>, io::Error> {
    let endpoint_uses_tls = is_https_endpoint(endpoint_uri);

    let Some(config) = config else {
        if !endpoint_uses_tls {
            return Ok(None);
        }
        // HTTPS endpoints need a TLS connector. Build one with system roots.
        let mut tls = ClientTlsConfig::new();
        tls = add_system_trust_anchors_if_enabled(tls, true).await?;
        return Ok(Some(tls));
    };

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
    let has_ca_setting = config.ca_file.is_some()
        || config
            .ca_pem
            .as_ref()
            .is_some_and(|pem| !pem.trim().is_empty());
    let server_name_configured = config.server_name_override.is_some();
    let tls_configured = client_cert_configured
        || client_key_configured
        || has_ca_setting
        || server_name_configured
        || config.include_system_ca_certs_pool.is_some();

    if !endpoint_uses_tls && !tls_configured {
        return Ok(None);
    }

    if !endpoint_uses_tls && tls_configured {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "TLS is configured for a non-https endpoint; use an https:// endpoint when enabling TLS/mTLS",
        ));
    }

    let mut tls = ClientTlsConfig::new();

    // Domain name / SNI.
    if let Some(domain) = &config.server_name_override {
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
    static SYSTEM_ROOTS: OnceLock<Vec<CertificateDer<'static>>> = OnceLock::new();

    let roots = if let Some(roots) = SYSTEM_ROOTS.get() {
        roots.clone()
    } else {
        // Loading native certificates involves blocking I/O (e.g. reading from disk or
        // querying the OS keychain). We must offload this to a blocking thread to avoid
        // stalling the async runtime.
        tokio::task::spawn_blocking(move || {
            // Check cache again inside blocking task to avoid double-loading if raced
            if let Some(roots) = SYSTEM_ROOTS.get() {
                return Ok::<_, io::Error>(roots.clone());
            }

            let native = load_native_certs();
            if !native.errors.is_empty() {
                log::warn!(
                    "Errors while loading native certificates (count={}): first={:?}",
                    native.errors.len(),
                    native.errors.first()
                );
            }

            // Cache the loaded certs
            let _ = SYSTEM_ROOTS.set(native.certs.clone());
            Ok(native.certs)
        })
        .await
        .map_err(io::Error::other)??
    };

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
            check_interval_secs: check_interval.map(|d| d.as_secs()).unwrap_or(300), // Default: 5 minutes
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
/// If file paths are provided, it uses lazy reloading.
/// If PEM strings are provided, it uses static configuration.
pub async fn build_reloadable_server_config(
    config: &TlsServerConfig,
) -> Result<Arc<rustls::ServerConfig>, io::Error> {
    let check_interval = config.config.reload_interval;

    let builder = rustls::ServerConfig::builder();

    // Client Auth (mTLS)
    let mut client_ca_pem = Vec::new();

    // Load system roots if requested
    if config.include_system_ca_certs_pool == Some(true) {
        let cert_res = tokio::task::spawn_blocking(load_native_certs)
            .await
            .map_err(io::Error::other)?;

        for error in &cert_res.errors {
            log::warn!("Error loading native cert: {}", error);
        }
        for cert in cert_res.certs {
            let base64_cert = BASE64_STANDARD.encode(cert.as_ref());
            // Wrap base64 at 64 characters per line per RFC 7468
            let wrapped: String = base64_cert
                .as_bytes()
                .chunks(64)
                .map(|c| std::str::from_utf8(c).expect("base64 is valid utf8"))
                .collect::<Vec<_>>()
                .join("\n");
            let pem = format!(
                "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
                wrapped
            );
            client_ca_pem.extend_from_slice(pem.as_bytes());
        }
    }

    // Load user-provided CA
    if let Some(client_ca_file) = &config.client_ca_file {
        let ca = read_file_with_limit_async(client_ca_file).await?;
        client_ca_pem.extend_from_slice(&ca);
    } else if let Some(client_ca_pem_str) = &config.client_ca_pem {
        client_ca_pem.extend_from_slice(client_ca_pem_str.as_bytes());
    }

    let builder = if !client_ca_pem.is_empty() {
        log::info!(
            "Configuring mTLS with {} bytes of CA certs",
            client_ca_pem.len()
        );
        let mut roots = RootCertStore::empty();
        let mut reader = io::BufReader::new(&client_ca_pem[..]);
        let mut count = 0;
        for cert in CertificateDer::pem_reader_iter(&mut reader) {
            let cert = cert.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            roots
                .add(cert)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            count += 1;
        }
        log::info!("Loaded {} CA certificates", count);

        let verifier = WebPkiClientVerifier::builder(roots.into())
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        log::debug!(
            "Client auth mandatory: {}",
            verifier.client_auth_mandatory()
        );
        builder.with_client_cert_verifier(verifier)
    } else {
        // If system CAs were requested but no certs were loaded (and no user CA provided),
        // return an error.
        if config.include_system_ca_certs_pool == Some(true) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "include_system_ca_certs_pool is true, but no CA certificates were loaded. \
                 Cannot enable mTLS without CA certificates.",
            ));
        }
        builder.with_no_client_auth()
    };

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
            handshake_timeout: None,
        };

        let result = load_server_tls_config(&config).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
