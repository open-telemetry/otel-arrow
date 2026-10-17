// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded hot-reload of client TLS material for OTLP exporters.
//!
//! Detects changes to file-backed client TLS material by periodically
//! re-reading and re-validating the configured sources, then publishes each
//! validated generation through an [`ArcSwap`] so exporters can rebuild their
//! transports without a process restart.
//!
//! The loop is transport-agnostic: it produces validated
//! [`LoadedClientTlsMaterial`], and callers build tonic channels or reqwest
//! clients from it. Keeping it independent of `tonic`/`reqwest` is deliberate so
//! the engine-owned TLS reload service can host it later without touching
//! exporter code.
//!
//! Change detection is content-based. For each Kubernetes projected volume, the
//! loader resolves `..data` once per attempt and reads direct or nested files
//! from that captured generation. Independent paths are read normally. Changed
//! candidates are parsed and validated before publication, including the full
//! certificate chain and certificate/key match; on any failure the
//! last-known-good generation is retained.
//!
//! Explicit `reload_interval: null` disables the loop, while zero is rejected
//! for configurations that create a provider. Overlapping attempts are
//! coalesced so generation numbers and notifications cannot regress.

// Wired into the OTLP/gRPC and OTLP/HTTP exporters in the follow-up stages of
// #4160; the reload machinery lands first so it can be reviewed on its own.
#![allow(dead_code)]

use std::future::Future;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use arc_swap::ArcSwap;
use otel_arrow_dfe_config::tls::TlsClientConfig;
use otel_arrow_dfe_telemetry::{otel_info, otel_warn};

use crate::tls_utils::{
    LoadedClientTlsMaterial, capture_client_tls_material, load_client_tls_material,
};

/// An immutable, validated generation of client TLS material.
///
/// Published atomically; readers always observe one consistent generation.
#[derive(Debug)]
pub(crate) struct ClientTlsGeneration {
    /// Monotonic generation number; `0` is the initial load, incremented on each
    /// successful reload. Lets a consumer cheaply detect that it must rebuild.
    pub(crate) number: u64,
    /// The validated material for this generation.
    pub(crate) material: LoadedClientTlsMaterial,
}

/// Outcome of a single reload attempt.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ReloadOutcome {
    /// Another reload attempt is already in progress, so this request was
    /// coalesced with it.
    Busy,
    /// Material was byte-for-byte identical to the current generation.
    Unchanged,
    /// A new generation was validated and published; carries its number.
    Updated(u64),
    /// The attempt failed; the last-known-good generation was retained.
    Failed,
}

/// Owns the current client TLS generation and reloads it on a bounded schedule.
pub(crate) struct ClientTlsProvider {
    /// Owned copy of the exporter's TLS configuration (source of the paths).
    config: Option<TlsClientConfig>,
    /// Endpoint URI, used to reproduce the scheme-driven load decisions.
    endpoint_uri: String,
    /// Interval between reload checks, or `None` when reload is disabled.
    reload_interval: Option<Duration>,
    /// Prevents overlapping loads from publishing generations out of order.
    /// Contention is coalesced instead of queued because the next scheduled
    /// poll will re-read the latest contents.
    reload_guard: tokio::sync::Mutex<()>,
    /// Current generation. `ArcSwap` gives exporters a lock-free read on their
    /// hot path while the guarded reload attempt publishes new generations; the
    /// two run on different tasks, so shared interior mutability is required.
    current: Arc<ArcSwap<ClientTlsGeneration>>,
    /// Next generation number to assign.
    next_number: AtomicU64,
    /// Notifies subscribers of the latest published generation number.
    notify: tokio::sync::watch::Sender<u64>,
}

impl ClientTlsProvider {
    /// Loads the initial generation and returns a provider, or `Ok(None)` when
    /// the connection does not use a configured TLS block (plaintext `http://`
    /// or `insecure` with no custom CA) -- the same decision as
    /// [`load_client_tls_material`].
    ///
    /// Fails if the initial material is missing or invalid, or if an active
    /// provider is configured with a zero reload interval. In particular, the
    /// shared loader rejects empty or malformed custom CA bundles.
    pub(crate) async fn new(
        config: Option<&TlsClientConfig>,
        endpoint_uri: &str,
    ) -> Result<Option<Self>, io::Error> {
        let Some(material) = load_client_tls_material(config, endpoint_uri).await? else {
            return Ok(None);
        };

        let reload_interval = config.and_then(|c| c.config.reload_interval);
        if reload_interval.is_some_and(|interval| interval.is_zero()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "TLS configuration error: reload_interval must be greater than zero or null to disable reload",
            ));
        }

        let generation = Arc::new(ClientTlsGeneration {
            number: 0,
            material,
        });
        let (notify, _rx) = tokio::sync::watch::channel(0);

        Ok(Some(Self {
            config: config.cloned(),
            endpoint_uri: endpoint_uri.to_string(),
            reload_interval,
            reload_guard: tokio::sync::Mutex::new(()),
            current: Arc::new(ArcSwap::from(generation)),
            next_number: AtomicU64::new(1),
            notify,
        }))
    }

    /// Returns the current generation (lock-free).
    pub(crate) fn current(&self) -> Arc<ClientTlsGeneration> {
        self.current.load_full()
    }

    /// Subscribes to published generation numbers. A consumer can await
    /// `changed()` and then rebuild its transport from [`current`].
    pub(crate) fn subscribe(&self) -> tokio::sync::watch::Receiver<u64> {
        self.notify.subscribe()
    }

    /// The configured reload interval, or `None` when periodic reload is
    /// disabled.
    pub(crate) fn reload_interval(&self) -> Option<Duration> {
        self.reload_interval
    }

    /// Whether any configured source is file-backed. Inline-only PEM material
    /// never changes at runtime, so a caller can skip running the loop.
    pub(crate) fn is_file_backed(&self) -> bool {
        self.config.as_ref().is_some_and(|c| {
            c.ca_file.is_some() || c.config.cert_file.is_some() || c.config.key_file.is_some()
        })
    }

    /// Performs one reload attempt: re-read and re-validate the configured
    /// sources, publishing a new generation only when the content changed.
    ///
    /// On validation/read failure, or if the configuration now resolves to
    /// no-TLS, the current generation is retained. If another attempt is in
    /// progress, this request returns [`ReloadOutcome::Busy`] without queueing.
    pub(crate) async fn poll_once(&self) -> ReloadOutcome {
        let Ok(_guard) = self.reload_guard.try_lock() else {
            return ReloadOutcome::Busy;
        };

        match capture_client_tls_material(self.config.as_ref(), &self.endpoint_uri).await {
            Ok(Some(candidate)) if candidate.matches(&self.current.load().material) => {
                ReloadOutcome::Unchanged
            }
            Ok(Some(candidate)) => match candidate.validate() {
                Ok(material) => {
                    let number = self.next_number.fetch_add(1, Ordering::Relaxed);
                    self.current
                        .store(Arc::new(ClientTlsGeneration { number, material }));
                    let _previous_number = self.notify.send_replace(number);
                    otel_info!("tls.client_reload.updated", generation = number);
                    ReloadOutcome::Updated(number)
                }
                Err(error) => {
                    otel_warn!(
                        "tls.client_reload.failed",
                        error = %error,
                        message = "client TLS reload failed; keeping last-known-good"
                    );
                    ReloadOutcome::Failed
                }
            },
            Ok(None) => {
                otel_warn!(
                    "tls.client_reload.became_plaintext",
                    message = "client TLS material now resolves to no-TLS; keeping last-known-good"
                );
                ReloadOutcome::Unchanged
            }
            Err(error) => {
                otel_warn!(
                    "tls.client_reload.failed",
                    error = %error,
                    message = "client TLS reload failed; keeping last-known-good"
                );
                ReloadOutcome::Failed
            }
        }
    }

    /// Runs the bounded reload loop until `cancel` resolves.
    ///
    /// Returns immediately when periodic reload is disabled. Otherwise, each
    /// tick performs at most one [`poll_once`]; cancellation is checked with
    /// priority so shutdown is prompt. The caller owns how this future is driven
    /// (for example, spawned local to the exporter's core), keeping the loop free
    /// of any engine-specific task machinery.
    pub(crate) async fn run<C>(&self, cancel: C)
    where
        C: Future<Output = ()>,
    {
        self.run_with_poll(cancel, || self.poll_once()).await;
    }

    async fn run_with_poll<C, P, F>(&self, cancel: C, mut poll: P)
    where
        C: Future<Output = ()>,
        P: FnMut() -> F,
        F: Future<Output = ReloadOutcome>,
    {
        let Some(reload_interval) = self.reload_interval else {
            return;
        };
        tokio::pin!(cancel);
        loop {
            tokio::select! {
                biased;
                () = &mut cancel => break,
                _ = async {
                    tokio::time::sleep(reload_interval).await;
                    poll().await
                } => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::tls::{TlsClientConfig, TlsConfig};
    use otel_arrow_dfe_test_tls_certs as tls_certs;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    const ENDPOINT: &str = "https://backend.example.com:4317";

    /// Writes a CA, client cert, and client key into `dir` and returns a
    /// file-backed client config that trusts only that CA (no system store, so
    /// the test is hermetic).
    fn write_material(dir: &Path, ca_cn: &str, leaf_cn: &str) -> TlsClientConfig {
        let ca = tls_certs::generate_self_signed_cert(ca_cn, Some(ca_cn), true);
        let leaf = tls_certs::generate_self_signed_cert(leaf_cn, Some(leaf_cn), false);
        fs::write(dir.join("ca.crt"), &ca.cert_pem).expect("write ca");
        fs::write(dir.join("tls.crt"), &leaf.cert_pem).expect("write cert");
        fs::write(dir.join("tls.key"), &leaf.key_pem).expect("write key");
        file_backed_config(dir.join("ca.crt"), dir.join("tls.crt"), dir.join("tls.key"))
    }

    fn file_backed_config(ca: PathBuf, cert: PathBuf, key: PathBuf) -> TlsClientConfig {
        TlsClientConfig {
            config: TlsConfig {
                cert_file: Some(cert),
                cert_pem: None,
                key_file: Some(key),
                key_pem: None,
                reload_interval: Some(Duration::from_millis(10)),
            },
            ca_file: Some(ca),
            ca_pem: None,
            include_system_ca_certs_pool: Some(false),
            server_name: None,
            insecure: None,
            insecure_skip_verify: None,
        }
    }

    /// Scenario: build a provider for a plaintext `http://` endpoint with no TLS
    /// block.
    /// Guarantees: no provider is created, so plaintext connections never spin up
    /// a reload loop.
    #[tokio::test]
    async fn new_returns_none_for_plaintext() {
        let provider = ClientTlsProvider::new(None, "http://backend:4317")
            .await
            .expect("load must succeed");
        assert!(provider.is_none());
    }

    /// Scenario: configure a zero reload interval on an insecure TLS block that
    /// does not create a client TLS provider.
    /// Guarantees: provider-specific interval validation does not reject a
    /// configuration that preserves the existing no-provider behavior.
    #[tokio::test]
    async fn zero_reload_interval_is_ignored_without_provider() {
        let config = TlsClientConfig {
            config: TlsConfig {
                reload_interval: Some(Duration::ZERO),
                ..TlsConfig::default()
            },
            insecure: Some(true),
            ..TlsClientConfig::default()
        };

        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("no-provider configuration must remain valid");
        assert!(provider.is_none());
    }

    /// Scenario: configure an explicit null reload interval for valid file-backed
    /// TLS material.
    /// Guarantees: reload remains disabled and running the provider returns
    /// without polling or waiting for cancellation.
    #[tokio::test]
    async fn null_reload_interval_disables_reload() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let mut config = write_material(dir.path(), "ca-1", "client-1");
        config.config.reload_interval = None;

        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");
        assert_eq!(provider.reload_interval(), None);
        tokio::time::timeout(
            Duration::from_millis(100),
            provider.run(std::future::pending()),
        )
        .await
        .expect("disabled reload must return immediately");
    }

    /// Scenario: configure a zero reload interval for valid TLS material.
    /// Guarantees: the provider rejects the configuration instead of creating a
    /// busy polling and validation loop.
    #[tokio::test]
    async fn zero_reload_interval_is_rejected() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let mut config = write_material(dir.path(), "ca-1", "client-1");
        config.config.reload_interval = Some(Duration::ZERO);

        let error = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .err()
            .expect("zero reload interval must fail");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    /// Scenario: rotate the client certificate and key to a new valid pair, then
    /// poll.
    /// Guarantees: the change is detected and a new generation is published with
    /// the rotated bytes, without recreating the provider.
    #[tokio::test]
    async fn poll_once_publishes_new_generation_on_rotation() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");

        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");
        assert_eq!(provider.current().number, 0);
        let mut updates = provider.subscribe();

        // Rotate to a new matching cert/key pair at the same paths.
        let rotated = tls_certs::generate_self_signed_cert("client-2", Some("client-2"), false);
        fs::write(dir.path().join("tls.crt"), &rotated.cert_pem).expect("rewrite cert");
        fs::write(dir.path().join("tls.key"), &rotated.key_pem).expect("rewrite key");

        assert_eq!(provider.poll_once().await, ReloadOutcome::Updated(1));
        let generation = provider.current();
        assert_eq!(generation.number, 1);
        assert_eq!(
            generation
                .material
                .client_identity
                .as_ref()
                .unwrap()
                .cert_pem,
            rotated.cert_pem.into_bytes()
        );
        assert!(updates.has_changed().unwrap());
        assert_eq!(*updates.borrow_and_update(), 1);
    }

    /// Scenario: publish a TLS rotation before any notification receiver has
    /// subscribed.
    /// Guarantees: a later subscriber observes the latest generation number
    /// rather than the constructor's initial value.
    #[tokio::test]
    async fn late_subscriber_observes_latest_generation() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");
        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");

        let rotated = tls_certs::generate_self_signed_cert("client-2", Some("client-2"), false);
        fs::write(dir.path().join("tls.crt"), &rotated.cert_pem).expect("rewrite cert");
        fs::write(dir.path().join("tls.key"), &rotated.key_pem).expect("rewrite key");
        assert_eq!(provider.poll_once().await, ReloadOutcome::Updated(1));

        let updates = provider.subscribe();
        assert_eq!(*updates.borrow(), 1);
    }

    /// Scenario: request another reload while one attempt already holds the
    /// provider's publication guard.
    /// Guarantees: overlapping attempts are coalesced, so generations cannot be
    /// published out of order.
    #[tokio::test]
    async fn poll_once_coalesces_concurrent_attempts() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");
        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");
        let _guard = provider.reload_guard.lock().await;

        assert_eq!(provider.poll_once().await, ReloadOutcome::Busy);
        assert_eq!(provider.current().number, 0);
    }

    /// Scenario: poll when nothing on disk has changed.
    /// Guarantees: no new generation is published, so unchanged files do not
    /// churn the transport.
    #[tokio::test]
    async fn poll_once_is_unchanged_when_content_is_stable() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");

        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");

        assert_eq!(provider.poll_once().await, ReloadOutcome::Unchanged);
        assert_eq!(provider.current().number, 0);
    }

    /// Scenario: replace the client certificate with unparseable bytes, then
    /// poll.
    /// Guarantees: the reload fails and the last-known-good generation is
    /// retained, so invalid material never replaces working material.
    #[tokio::test]
    async fn poll_once_keeps_last_good_on_invalid_material() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");

        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");
        let good = provider.current();

        fs::write(
            dir.path().join("tls.crt"),
            b"-----BEGIN CERTIFICATE-----\nnotpem\n",
        )
        .expect("corrupt cert");

        assert_eq!(provider.poll_once().await, ReloadOutcome::Failed);
        let still = provider.current();
        assert_eq!(still.number, good.number);
        let still_identity = still
            .material
            .client_identity
            .as_ref()
            .expect("current identity");
        let good_identity = good
            .material
            .client_identity
            .as_ref()
            .expect("last-known-good identity");
        assert_eq!(still_identity.cert_pem, good_identity.cert_pem);
        assert_eq!(still_identity.key_pem, good_identity.key_pem);
    }

    /// Scenario: append a PEM certificate containing malformed DER to an
    /// otherwise valid client certificate chain, then poll.
    /// Guarantees: every certificate in the chain is validated before
    /// publication and a malformed intermediate cannot replace last-known-good
    /// material.
    #[tokio::test]
    async fn poll_once_rejects_malformed_intermediate_certificate() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");
        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");

        let mut chain = fs::read_to_string(dir.path().join("tls.crt")).expect("read cert");
        chain.push_str("-----BEGIN CERTIFICATE-----\nAQID\n-----END CERTIFICATE-----\n");
        fs::write(dir.path().join("tls.crt"), chain).expect("append malformed intermediate");

        assert_eq!(provider.poll_once().await, ReloadOutcome::Failed);
        assert_eq!(provider.current().number, 0);
    }

    /// Scenario: prepend a PEM certificate containing invalid DER before a
    /// valid client leaf certificate and matching key, then poll.
    /// Guarantees: an invalid DER leaf is rejected instead of being treated as
    /// an inconclusive key-match check and published.
    #[tokio::test]
    async fn poll_once_rejects_invalid_der_leaf_certificate() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");
        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");

        let valid_leaf = fs::read_to_string(dir.path().join("tls.crt")).expect("read cert");
        let chain =
            format!("-----BEGIN CERTIFICATE-----\nAQID\n-----END CERTIFICATE-----\n{valid_leaf}");
        fs::write(dir.path().join("tls.crt"), chain).expect("prepend invalid leaf");

        assert_eq!(provider.poll_once().await, ReloadOutcome::Failed);
        assert_eq!(provider.current().number, 0);
    }

    /// Scenario: replace a valid configured CA file with an empty bundle, then
    /// poll.
    /// Guarantees: invalid trust material fails validation and cannot replace
    /// the last-known-good generation.
    #[tokio::test]
    async fn poll_once_keeps_last_good_on_invalid_ca() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");
        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");

        fs::write(dir.path().join("ca.crt"), []).expect("empty CA file");

        assert_eq!(provider.poll_once().await, ReloadOutcome::Failed);
        assert_eq!(provider.current().number, 0);
    }

    /// Scenario: rotate only the certificate, leaving the previous key in place.
    /// Guarantees: the mismatched pair is rejected and the last-known-good
    /// generation is retained, covering partial rotations.
    #[tokio::test]
    async fn poll_once_rejects_partial_rotation() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");

        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");

        // New cert, but the on-disk key still belongs to the old cert.
        let rotated = tls_certs::generate_self_signed_cert("client-2", Some("client-2"), false);
        fs::write(dir.path().join("tls.crt"), &rotated.cert_pem).expect("rewrite cert only");

        assert_eq!(provider.poll_once().await, ReloadOutcome::Failed);
        assert_eq!(provider.current().number, 0);
    }

    /// Scenario: Kubernetes-style rotation that swaps a `..data` symlink to a new
    /// timestamped directory instead of rewriting the leaf files.
    /// Guarantees: the reload follows the current symlink target and publishes
    /// the new material, so projected-Secret rotations apply without a restart.
    #[cfg(unix)]
    #[tokio::test]
    async fn poll_once_follows_projected_secret_symlink_swap() {
        use std::os::unix::fs::symlink;

        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let root = dir.path();

        // Kubelet AtomicWriter layout: leaf files are symlinks through ..data.
        let v1 = root.join("..2026_09_17_08_06_30.1");
        fs::create_dir(&v1).expect("v1 dir");
        let ca = tls_certs::generate_self_signed_cert("ca-1", Some("ca-1"), true);
        let leaf1 = tls_certs::generate_self_signed_cert("client-1", Some("client-1"), false);
        fs::write(v1.join("ca.crt"), &ca.cert_pem).expect("v1 ca");
        fs::write(v1.join("tls.crt"), &leaf1.cert_pem).expect("v1 cert");
        fs::write(v1.join("tls.key"), &leaf1.key_pem).expect("v1 key");
        symlink(&v1, root.join("..data")).expect("..data -> v1");
        symlink("..data/ca.crt", root.join("ca.crt")).expect("ca symlink");
        symlink("..data/tls.crt", root.join("tls.crt")).expect("cert symlink");
        symlink("..data/tls.key", root.join("tls.key")).expect("key symlink");

        let config = file_backed_config(
            root.join("ca.crt"),
            root.join("tls.crt"),
            root.join("tls.key"),
        );
        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");
        assert_eq!(provider.current().number, 0);

        // Rotate: new timestamped dir, then atomically swap the ..data symlink.
        let v2 = root.join("..2026_09_18_08_06_30.2");
        fs::create_dir(&v2).expect("v2 dir");
        let leaf2 = tls_certs::generate_self_signed_cert("client-2", Some("client-2"), false);
        fs::write(v2.join("ca.crt"), &ca.cert_pem).expect("v2 ca");
        fs::write(v2.join("tls.crt"), &leaf2.cert_pem).expect("v2 cert");
        fs::write(v2.join("tls.key"), &leaf2.key_pem).expect("v2 key");
        let tmp = root.join("..data_tmp");
        symlink(&v2, &tmp).expect("..data_tmp -> v2");
        fs::rename(&tmp, root.join("..data")).expect("swap ..data over");
        fs::remove_dir_all(&v1).expect("remove old v1");

        assert_eq!(provider.poll_once().await, ReloadOutcome::Updated(1));
        assert_eq!(
            provider
                .current()
                .material
                .client_identity
                .as_ref()
                .unwrap()
                .cert_pem,
            leaf2.cert_pem.into_bytes()
        );
    }

    /// Scenario: run the reload loop with an already-resolved cancel future.
    /// Guarantees: the loop observes cancellation and returns promptly instead of
    /// waiting a full interval, so shutdown is not delayed.
    #[tokio::test]
    async fn run_returns_when_cancelled() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");
        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");

        // Ready-immediately cancel; run must not block on the reload interval.
        tokio::time::timeout(Duration::from_secs(5), provider.run(std::future::ready(())))
            .await
            .expect("run must return once cancelled");
    }

    /// Scenario: cancellation resolves after the reload interval has elapsed and
    /// a poll attempt is still pending.
    /// Guarantees: the reload loop races cancellation against active polling and
    /// returns without waiting for the filesystem operation to finish.
    #[tokio::test]
    async fn run_returns_when_cancelled_during_poll() {
        crate::crypto::ensure_crypto_provider();
        let dir = TempDir::new().expect("temp dir");
        let config = write_material(dir.path(), "ca-1", "client-1");
        let provider = ClientTlsProvider::new(Some(&config), ENDPOINT)
            .await
            .expect("load")
            .expect("provider present");
        let poll_started = Arc::new(tokio::sync::Notify::new());
        let cancel = Arc::new(tokio::sync::Notify::new());

        let poll_started_by_loop = Arc::clone(&poll_started);
        let cancel_for_loop = Arc::clone(&cancel);
        let run = provider.run_with_poll(cancel_for_loop.notified(), move || {
            let poll_started = Arc::clone(&poll_started_by_loop);
            async move {
                poll_started.notify_one();
                std::future::pending::<ReloadOutcome>().await
            }
        });
        let trigger_cancel = async {
            poll_started.notified().await;
            cancel.notify_one();
        };

        tokio::time::timeout(Duration::from_secs(1), async {
            tokio::join!(run, trigger_cancel);
        })
        .await
        .expect("run must return when cancellation wins an active poll");
    }
}
