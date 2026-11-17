// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Config URI providers for resolving pipeline configuration from various sources.
//!
//! Implements an OTel Collector-compatible `--config` URI pattern where the
//! configuration source is identified by a URI scheme:
//!
//! | URI | Behaviour |
//! |---|---|
//! | `file:/path/to/config.yaml` | Read config from a local file |
//! | `env:MY_VAR` | Read config from an environment variable |
//! | `yaml:key::value` | Inline YAML with `::` as nested-key separator |
//! | `http://host/path` | Fetch config via unauthenticated HTTP GET (30s per-request timeout, retries with exponential backoff) |
//! | `https://host/path` | As HTTP, with server certificate verification and HTTPS-only redirects |
//! | `/path/to/config.yaml` | Bare path, treated as `file:` |
//! | `./relative/config.yaml` | Relative path, treated as `file:` |
//!
//! When no `--config` is provided, a default path in the current directory is tried.
//!
//! Authentication (Bearer token, mTLS) and multi-config merge are deferred to a
//! future phase.

use crate::error::Error;
use std::path::Path;
use std::time::Duration;

/// Fallback config path tried when `--config` is omitted.
const DEFAULT_CONFIG_PATH: &str = "config.yaml";

/// The serialization format of resolved configuration content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// YAML format (the default for most config sources).
    Yaml,
    /// JSON format (detected by a `.json` file extension).
    Json,
}

/// The result of resolving a config URI: the original source URI, the loaded
/// content, and the detected serialization format.
#[derive(Debug)]
pub struct ResolvedConfig {
    /// The original URI or path string used to load the config
    /// (e.g. "file:/etc/config.yaml", "env:MY_VAR").
    pub source: String,
    /// The raw configuration content (YAML or JSON string).
    pub content: String,
    /// The serialization format of `content`, derived from the source URI.
    pub format: ConfigFormat,
}

/// A provider that can resolve configuration content from a URI with a specific scheme.
pub trait ConfigProvider {
    /// The URI scheme this provider handles (e.g. "file", "env").
    fn scheme(&self) -> &str;

    /// Resolve the given URI to configuration content.
    ///
    /// The `uri` is the full original URI string (including the scheme prefix).
    /// Implementations should strip their own scheme prefix.
    fn resolve(&self, uri: &str) -> Result<ResolvedConfig, Error>;
}

/// Reads configuration from a local file path.
pub struct FileConfigProvider;

impl ConfigProvider for FileConfigProvider {
    fn scheme(&self) -> &str {
        "file"
    }

    fn resolve(&self, uri: &str) -> Result<ResolvedConfig, Error> {
        let path = uri.strip_prefix("file:").unwrap_or(uri);
        let content = std::fs::read_to_string(path).map_err(|e| Error::FileReadError {
            context: crate::error::Context::default(),
            details: format!("{path}: {e}"),
        })?;
        let format = if path.ends_with(".json") {
            ConfigFormat::Json
        } else {
            ConfigFormat::Yaml
        };
        Ok(ResolvedConfig {
            source: uri.to_string(),
            content,
            format,
        })
    }
}

/// Reads configuration from an environment variable.
pub struct EnvConfigProvider;

impl ConfigProvider for EnvConfigProvider {
    fn scheme(&self) -> &str {
        "env"
    }

    fn resolve(&self, uri: &str) -> Result<ResolvedConfig, Error> {
        let var = uri.strip_prefix("env:").unwrap_or(uri);
        let content = std::env::var(var).map_err(|_| Error::ConfigEnvVarNotSet {
            var: var.to_string(),
        })?;
        Ok(ResolvedConfig {
            source: uri.to_string(),
            content,
            // Env vars are assumed to contain YAML; JSON is not common here.
            format: ConfigFormat::Yaml,
        })
    }
}

/// Reads configuration from an inline YAML string on the CLI.
///
/// Follows the OTel Collector `yaml:` convention: `::` is a path separator
/// that expands into nested YAML, and the segment after the final `::` is a
/// trailing YAML fragment (typically `key: value`). With no `::` the content
/// is passed through as literal YAML.
///
/// # Examples
///
/// | URI | Resolved YAML |
/// |---|---|
/// | `yaml:version: otel_dataflow/v1` | `version: otel_dataflow/v1` |
/// | `yaml:exporters::debug::verbosity: detailed` | `exporters:\n  debug:\n    verbosity: detailed` |
/// | `yaml:engine::{}` | `engine:\n  {}` |
pub struct YamlConfigProvider;

impl YamlConfigProvider {
    /// Expand `key1::key2::...::trailing` into indented nested YAML. If the
    /// input contains no `::`, returns it unchanged.
    fn expand_key_path(body: &str) -> String {
        let Some((path_part, trailing)) = body.rsplit_once("::") else {
            return body.to_string();
        };
        let segments: Vec<&str> = path_part.split("::").collect();
        let mut out = String::new();
        for (depth, seg) in segments.iter().enumerate() {
            for _ in 0..(depth * 2) {
                out.push(' ');
            }
            out.push_str(seg);
            out.push_str(":\n");
        }
        for _ in 0..(segments.len() * 2) {
            out.push(' ');
        }
        out.push_str(trailing);
        out
    }
}

impl ConfigProvider for YamlConfigProvider {
    fn scheme(&self) -> &str {
        "yaml"
    }

    fn resolve(&self, uri: &str) -> Result<ResolvedConfig, Error> {
        let body = uri.strip_prefix("yaml:").unwrap_or(uri);
        Ok(ResolvedConfig {
            source: uri.to_string(),
            content: Self::expand_key_path(body),
            format: ConfigFormat::Yaml,
        })
    }
}

/// Reads configuration from an unauthenticated HTTP GET.
///
/// The response body is treated as YAML unless the `Content-Type` header is
/// `application/json`. A 30-second per-request timeout is applied and standard
/// HTTP redirects are followed by the underlying client. For HTTPS-only
/// configuration sources, use [`HttpsConfigProvider`]. Authentication and
/// custom timeouts are deferred to a future phase.
///
/// On a network error or a non-2xx status (404, 5xx, etc.) the provider retries
/// with exponential backoff starting at 500ms and capped at 60s per sleep. The
/// default maximum attempt count is [`u64::MAX`] so startup blocks until the
/// remote config server becomes available; use [`Self::with_max_attempts`] to
/// bound this.
pub struct HttpConfigProvider {
    client: reqwest::blocking::Client,
    max_attempts: u64,
}

impl HttpConfigProvider {
    /// Start delay for exponential backoff between retries.
    const BASE_BACKOFF: Duration = Duration::from_millis(500);
    /// Upper bound on the sleep between retries regardless of attempt count.
    const MAX_BACKOFF: Duration = Duration::from_secs(60);

    /// Build a provider that retries failed fetches indefinitely with
    /// exponential backoff (default: [`u64::MAX`] attempts).
    ///
    /// Returns [`Error::ConfigHttpClientBuildFailed`] if the underlying
    /// HTTP client cannot be constructed (for example when no rustls
    /// crypto provider has been installed).
    pub fn new() -> Result<Self, Error> {
        Self::with_max_attempts(u64::MAX)
    }

    /// Build a provider with an explicit cap on retry attempts. `max_attempts`
    /// counts the first try as an attempt, so `1` disables retries entirely.
    ///
    /// See [`Self::new`] for the set of errors this can return.
    pub fn with_max_attempts(max_attempts: u64) -> Result<Self, Error> {
        Self::with_client_builder(reqwest::blocking::Client::builder(), max_attempts)
    }

    fn with_client_builder(
        builder: reqwest::blocking::ClientBuilder,
        max_attempts: u64,
    ) -> Result<Self, Error> {
        let client = builder
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| Error::ConfigHttpClientBuildFailed {
                details: e.to_string(),
            })?;
        Ok(Self {
            client,
            max_attempts,
        })
    }

    fn try_fetch(&self, uri: &str) -> Result<ResolvedConfig, Error> {
        let response = self
            .client
            .get(uri)
            .send()
            .map_err(|e| Error::ConfigHttpRequestFailed {
                uri: uri.to_string(),
                // Include the source chain so TLS verification failures are actionable.
                details: format!("{e:?}"),
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(Error::ConfigHttpRequestFailed {
                uri: uri.to_string(),
                details: format!("unexpected HTTP status {status}"),
            });
        }

        let format = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .filter(|ct| ct.starts_with("application/json"))
            .map_or(ConfigFormat::Yaml, |_| ConfigFormat::Json);

        let content = response
            .text()
            .map_err(|e| Error::ConfigHttpRequestFailed {
                uri: uri.to_string(),
                details: format!("read body: {e}"),
            })?;

        Ok(ResolvedConfig {
            source: uri.to_string(),
            content,
            format,
        })
    }

    /// Sleep duration before the next attempt, doubling until capped.
    fn backoff_for(attempt: u64) -> Duration {
        let shift = attempt.min(20) as u32;
        let multiplier = 1u32 << shift;
        Self::BASE_BACKOFF
            .checked_mul(multiplier)
            .unwrap_or(Self::MAX_BACKOFF)
            .min(Self::MAX_BACKOFF)
    }
}

impl ConfigProvider for HttpConfigProvider {
    fn scheme(&self) -> &str {
        "http"
    }

    fn resolve(&self, uri: &str) -> Result<ResolvedConfig, Error> {
        if self.max_attempts == 0 {
            return Err(Error::ConfigHttpRequestFailed {
                uri: uri.to_string(),
                details: "max_attempts is 0".to_string(),
            });
        }

        let mut last_err: Option<Error> = None;
        for attempt in 0..self.max_attempts {
            match self.try_fetch(uri) {
                Ok(resolved) => return Ok(resolved),
                Err(e) => {
                    last_err = Some(e);
                    // Don't sleep after the final attempt.
                    if attempt + 1 < self.max_attempts {
                        std::thread::sleep(Self::backoff_for(attempt));
                    }
                }
            }
        }
        Err(last_err.expect("loop ran at least once so last_err is set"))
    }
}

/// Reads configuration from an unauthenticated HTTPS GET.
///
/// Uses the same content-type detection, 30-second timeout, and retry/backoff
/// behavior as [`HttpConfigProvider`]. Server certificates and hostnames are
/// verified using reqwest's default trust configuration and the crypto provider
/// installed at process startup. No client certificate is configured.
///
/// Redirects to HTTPS are followed with the client's default redirect limit;
/// redirects to plaintext HTTP are rejected before contacting the target.
pub struct HttpsConfigProvider(HttpConfigProvider);

impl HttpsConfigProvider {
    /// Build a provider with the same retry defaults as [`HttpConfigProvider::new`].
    ///
    /// Returns [`Error::ConfigHttpClientBuildFailed`] if the client cannot be built.
    pub fn new() -> Result<Self, Error> {
        Self::with_max_attempts(u64::MAX)
    }

    /// Build a provider with an explicit cap on attempts; `1` disables retries.
    ///
    /// See [`Self::new`] for the set of errors this can return.
    pub fn with_max_attempts(max_attempts: u64) -> Result<Self, Error> {
        HttpConfigProvider::with_client_builder(Self::client_builder(), max_attempts).map(Self)
    }

    fn client_builder() -> reqwest::blocking::ClientBuilder {
        reqwest::blocking::Client::builder().https_only(true)
    }
}

impl ConfigProvider for HttpsConfigProvider {
    fn scheme(&self) -> &str {
        "https"
    }

    fn resolve(&self, uri: &str) -> Result<ResolvedConfig, Error> {
        self.0.resolve(uri)
    }
}

/// Dispatches config resolution to the appropriate [`ConfigProvider`] based on URI scheme.
pub struct ConfigResolver {
    providers: Vec<Box<dyn ConfigProvider>>,
}

impl ConfigResolver {
    /// Create a new resolver with the given providers.
    #[must_use]
    pub fn new(providers: Vec<Box<dyn ConfigProvider>>) -> Self {
        Self { providers }
    }

    /// Resolve a URI to config content by matching the scheme to a registered provider.
    pub fn resolve(&self, uri: &str) -> Result<ResolvedConfig, Error> {
        // Determine the scheme from the URI.
        let scheme = parse_scheme(uri);

        match scheme {
            Some(s) => {
                for provider in &self.providers {
                    if provider.scheme() == s {
                        return provider.resolve(uri);
                    }
                }
                Err(Error::ConfigUriUnknownScheme {
                    scheme: s.to_string(),
                })
            }
            // No scheme detected means it is a bare file path.
            None => {
                for provider in &self.providers {
                    if provider.scheme() == "file" {
                        return provider.resolve(uri);
                    }
                }
                Err(Error::ConfigUriUnknownScheme {
                    scheme: "file".to_string(),
                })
            }
        }
    }
}

/// Returns a [`ConfigResolver`] with the default providers: `file:`, `env:`,
/// `yaml:`, `http:`, and `https:`.
///
/// Can fail if a client used by the `http:` or `https:` provider cannot be
/// constructed (see [`HttpConfigProvider::new`]).
pub fn default_resolver() -> Result<ConfigResolver, Error> {
    Ok(ConfigResolver::new(vec![
        Box::new(FileConfigProvider),
        Box::new(EnvConfigProvider),
        Box::new(YamlConfigProvider),
        Box::new(HttpConfigProvider::new()?),
        Box::new(HttpsConfigProvider::new()?),
    ]))
}

/// Top-level entry point for resolving a config URI to content.
///
/// - `Some(uri)`: parse scheme and dispatch to the appropriate provider.
///   Bare paths (no scheme, starts with `/` or `.`) are treated as `file:`.
/// - `None`: try `DEFAULT_CONFIG_PATH` in the current working directory.
pub fn resolve_config(uri: Option<&str>) -> Result<ResolvedConfig, Error> {
    let resolver = default_resolver()?;

    match uri {
        Some(u) => resolver.resolve(u),
        None => {
            if Path::new(DEFAULT_CONFIG_PATH).exists() {
                return resolver.resolve(DEFAULT_CONFIG_PATH);
            }
            Err(Error::ConfigNoFileFound {
                path: DEFAULT_CONFIG_PATH.to_string(),
            })
        }
    }
}

/// Parse the URI scheme, if any.
///
/// Returns `None` for bare paths (absolute, relative starting with `.`, or no `:` before
/// a path separator). Also returns `None` for Windows drive-letter paths like `C:\foo`.
/// Returns `Some(scheme)` for `scheme:rest` patterns.
fn parse_scheme(uri: &str) -> Option<&str> {
    // Absolute paths (works cross-platform: /foo on Unix, C:\foo on Windows).
    if Path::new(uri).is_absolute() {
        return None;
    }

    // Relative paths starting with ./ or ../
    if uri.starts_with('.') {
        return None;
    }

    // Look for a colon. If there is no colon, it is a bare filename.
    let colon_pos = uri.find(':')?;
    let before_colon = &uri[..colon_pos];

    // Single-character "scheme" is a Windows drive letter (e.g. C:), not a URI scheme.
    if before_colon.len() == 1
        && before_colon
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
    {
        return None;
    }

    // If a path separator appears before the colon, treat as bare path.
    if before_colon.contains('/') || before_colon.contains('\\') {
        return None;
    }

    Some(before_colon)
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Write;
    use std::sync::Once;

    /// Install a rustls crypto provider so reqwest can build a blocking client.
    /// Production code installs this at process startup; tests do it lazily.
    static CRYPTO_INIT: Once = Once::new();
    fn ensure_crypto_provider() {
        CRYPTO_INIT.call_once(|| {
            let _ = rustls::crypto::ring::default_provider().install_default();
        });
    }

    #[test]
    fn file_provider_reads_temp_file() {
        let mut tmp = tempfile::NamedTempFile::new().expect("create temp file");
        write!(tmp, "hello: world").expect("write temp file");

        let provider = FileConfigProvider;
        let path = format!("file:{}", tmp.path().display());
        let resolved = provider.resolve(&path).expect("should read file");
        assert_eq!(resolved.content, "hello: world");
        assert_eq!(resolved.source, path);
        assert_eq!(resolved.format, ConfigFormat::Yaml);
    }

    #[test]
    fn file_provider_detects_json_format() {
        let mut tmp = tempfile::Builder::new()
            .suffix(".json")
            .tempfile()
            .expect("create temp json file");
        write!(tmp, r#"{{"key": "value"}}"#).expect("write temp file");

        let provider = FileConfigProvider;
        let path = format!("file:{}", tmp.path().display());
        let resolved = provider.resolve(&path).expect("should read json file");
        assert_eq!(resolved.format, ConfigFormat::Json);
    }

    #[test]
    fn file_provider_errors_on_missing_file() {
        let provider = FileConfigProvider;
        let result = provider.resolve("file:/nonexistent/path/config.yaml");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::FileReadError { details, .. } => {
                assert!(details.contains("/nonexistent/path/config.yaml"));
            }
            other => panic!("expected FileReadError, got {other:?}"),
        }
    }

    #[test]
    fn env_provider_reads_var() {
        let var_name = "OTAP_TEST_CFG_PROVIDER_READ";
        unsafe {
            env::set_var(var_name, "version: v1");
        }
        let provider = EnvConfigProvider;
        let uri = format!("env:{var_name}");
        let resolved = provider.resolve(&uri).expect("should read env var");
        assert_eq!(resolved.content, "version: v1");
        assert_eq!(resolved.source, uri);
        assert_eq!(resolved.format, ConfigFormat::Yaml);
        unsafe {
            env::remove_var(var_name);
        }
    }

    #[test]
    fn env_provider_errors_when_not_set() {
        let var_name = "OTAP_TEST_CFG_PROVIDER_UNSET";
        unsafe {
            env::remove_var(var_name);
        }
        let provider = EnvConfigProvider;
        let result = provider.resolve(&format!("env:{var_name}"));
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ConfigEnvVarNotSet { var } => {
                assert_eq!(var, var_name);
            }
            other => panic!("expected ConfigEnvVarNotSet, got {other:?}"),
        }
    }

    #[test]
    fn bare_path_treated_as_file() {
        ensure_crypto_provider();
        let mut tmp = tempfile::NamedTempFile::new().expect("create temp file");
        write!(tmp, "bare: path").expect("write temp file");

        let resolver = default_resolver().expect("default resolver should build");
        let resolved = resolver
            .resolve(&tmp.path().display().to_string())
            .expect("bare path should resolve as file");
        assert_eq!(resolved.content, "bare: path");
    }

    #[test]
    fn unknown_scheme_returns_error() {
        ensure_crypto_provider();
        let resolver = default_resolver().expect("default resolver should build");
        let result = resolver.resolve("ftp://example.com/config.yaml");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ConfigUriUnknownScheme { scheme } => {
                assert_eq!(scheme, "ftp");
            }
            other => panic!("expected ConfigUriUnknownScheme, got {other:?}"),
        }
    }

    #[test]
    fn resolve_config_none_with_no_default_config() {
        ensure_crypto_provider();
        // Run from a temp directory to ensure config.yaml does not exist.
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let _guard = env::current_dir().ok();
        env::set_current_dir(tmp_dir.path()).expect("set cwd to temp dir");

        let result = resolve_config(None);
        match result {
            Err(Error::ConfigNoFileFound { path }) => {
                assert_eq!(path, "config.yaml");
            }
            Err(other) => panic!("expected ConfigNoFileFound, got {other:?}"),
            Ok(_) => panic!("expected error when no config.yaml exists"),
        }
    }

    #[test]
    fn integration_env_var_to_yaml() {
        ensure_crypto_provider();
        let var_name = "OTAP_TEST_CFG_INTEGRATION";
        let yaml = r#"
version: otel_dataflow/v1
engine: {}
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;
        unsafe {
            env::set_var(var_name, yaml);
        }
        let resolved =
            resolve_config(Some(&format!("env:{var_name}"))).expect("should resolve env config");
        // Verify it can be parsed as an OtelDataflowSpec.
        let spec = crate::engine::OtelDataflowSpec::from_yaml(&resolved.content);
        assert!(spec.is_ok(), "env config should parse as valid YAML spec");
        unsafe {
            env::remove_var(var_name);
        }
    }

    #[test]
    fn parse_scheme_cases() {
        assert_eq!(parse_scheme("file:/etc/config.yaml"), Some("file"));
        assert_eq!(parse_scheme("env:MY_VAR"), Some("env"));
        assert_eq!(parse_scheme("yaml:foo::bar"), Some("yaml"));
        assert_eq!(parse_scheme("/absolute/path.yaml"), None);
        assert_eq!(parse_scheme("./relative/path.yaml"), None);
        assert_eq!(parse_scheme("config.yaml"), None);
        assert_eq!(parse_scheme("some/path:with:colons"), None);
        assert_eq!(parse_scheme("http://example.com"), Some("http"));
        // Windows drive-letter paths: the single-char scheme guard catches these
        // even on Unix where Path::is_absolute() would return false for them.
        assert_eq!(parse_scheme("C:\\config.yaml"), None);
        assert_eq!(parse_scheme("D:/config.yaml"), None);
    }

    #[test]
    fn yaml_provider_literal_content() {
        let provider = YamlConfigProvider;
        let resolved = provider
            .resolve("yaml:version: otel_dataflow/v1")
            .expect("literal yaml should resolve");
        assert_eq!(resolved.content, "version: otel_dataflow/v1");
        assert_eq!(resolved.format, ConfigFormat::Yaml);
    }

    #[test]
    fn yaml_provider_single_key_path() {
        let provider = YamlConfigProvider;
        let resolved = provider
            .resolve("yaml:version::otel_dataflow/v1")
            .expect("single-level key path should resolve");
        assert_eq!(resolved.content, "version:\n  otel_dataflow/v1");
    }

    #[test]
    fn yaml_provider_nested_key_path() {
        let provider = YamlConfigProvider;
        let resolved = provider
            .resolve("yaml:exporters::debug::verbosity: detailed")
            .expect("nested key path should resolve");
        assert_eq!(
            resolved.content,
            "exporters:\n  debug:\n    verbosity: detailed"
        );
    }

    #[test]
    fn yaml_provider_flow_value() {
        let provider = YamlConfigProvider;
        let resolved = provider
            .resolve("yaml:engine::{}")
            .expect("flow-style value should resolve");
        assert_eq!(resolved.content, "engine:\n  {}");
    }

    #[test]
    fn yaml_provider_parses_as_yaml() {
        ensure_crypto_provider();
        let resolver = default_resolver().expect("default resolver should build");
        let resolved = resolver
            .resolve("yaml:exporters::debug::verbosity: detailed")
            .expect("should dispatch to yaml provider");
        let value: serde_yaml::Value =
            serde_yaml::from_str(&resolved.content).expect("expanded content should be valid YAML");
        let verbosity = value
            .get("exporters")
            .and_then(|v| v.get("debug"))
            .and_then(|v| v.get("verbosity"))
            .and_then(|v| v.as_str());
        assert_eq!(verbosity, Some("detailed"));
    }

    #[tokio::test]
    async fn http_provider_fetches_yaml_body() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        ensure_crypto_provider();
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pipeline.yaml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "application/yaml")
                    .set_body_string("version: otel_dataflow/v1\n"),
            )
            .mount(&mock_server)
            .await;

        let uri = format!("{}/pipeline.yaml", mock_server.uri());
        // reqwest blocking must run off the tokio runtime. spawn_blocking keeps
        // the async wiremock server alive while the sync client runs.
        let resolved = tokio::task::spawn_blocking(move || {
            HttpConfigProvider::new()
                .expect("provider should build")
                .resolve(&uri)
                .expect("http provider should fetch body")
        })
        .await
        .expect("spawn_blocking join");
        assert_eq!(resolved.content, "version: otel_dataflow/v1\n");
        assert_eq!(resolved.format, ConfigFormat::Yaml);
    }

    #[tokio::test]
    async fn http_provider_detects_json_content_type() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        ensure_crypto_provider();
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pipeline.json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(r#"{"version":"v1"}"#.as_bytes(), "application/json"),
            )
            .mount(&mock_server)
            .await;

        let uri = format!("{}/pipeline.json", mock_server.uri());
        let resolved = tokio::task::spawn_blocking(move || {
            HttpConfigProvider::new()
                .expect("provider should build")
                .resolve(&uri)
                .expect("should detect json content type")
        })
        .await
        .expect("spawn_blocking join");
        assert_eq!(resolved.format, ConfigFormat::Json);
    }

    #[tokio::test]
    async fn http_provider_errors_on_non_success_status() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        ensure_crypto_provider();
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/missing"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let uri = format!("{}/missing", mock_server.uri());
        // `with_max_attempts(1)` disables retries; otherwise the default
        // `u64::MAX` would keep the test running forever.
        let result = tokio::task::spawn_blocking(move || {
            HttpConfigProvider::with_max_attempts(1)
                .expect("provider should build")
                .resolve(&uri)
        })
        .await
        .expect("spawn_blocking join");
        match result {
            Err(Error::ConfigHttpRequestFailed { details, .. }) => {
                assert!(
                    details.contains("404"),
                    "details should mention status: {details}"
                );
            }
            other => panic!("expected ConfigHttpRequestFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn http_provider_retries_on_transient_failure() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        ensure_crypto_provider();
        let mock_server = MockServer::start().await;
        // First two requests return 503, then fall through to the success mock.
        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"version: v1\n".to_vec(), "application/yaml"),
            )
            .mount(&mock_server)
            .await;

        let uri = format!("{}/flaky", mock_server.uri());
        let resolved = tokio::task::spawn_blocking(move || {
            HttpConfigProvider::with_max_attempts(4)
                .expect("provider should build")
                .resolve(&uri)
                .expect("third attempt should succeed")
        })
        .await
        .expect("spawn_blocking join");
        assert_eq!(resolved.content, "version: v1\n");
    }

    #[test]
    fn http_provider_backoff_grows_and_caps() {
        assert_eq!(
            HttpConfigProvider::backoff_for(0),
            Duration::from_millis(500)
        );
        assert_eq!(HttpConfigProvider::backoff_for(1), Duration::from_secs(1));
        assert_eq!(HttpConfigProvider::backoff_for(2), Duration::from_secs(2));
        // Beyond ~7 attempts the 60s cap takes over.
        assert_eq!(HttpConfigProvider::backoff_for(20), Duration::from_secs(60));
        assert_eq!(
            HttpConfigProvider::backoff_for(u64::MAX),
            Duration::from_secs(60)
        );
    }

    #[test]
    fn http_provider_new_defaults_to_unbounded_attempts() {
        ensure_crypto_provider();
        let p = HttpConfigProvider::new().expect("provider should build");
        assert_eq!(p.max_attempts, u64::MAX);
        assert_eq!(p.scheme(), "http");
    }

    #[test]
    fn http_provider_zero_attempts_returns_error() {
        ensure_crypto_provider();
        let p = HttpConfigProvider::with_max_attempts(0).expect("provider should build");
        let result = p.resolve("http://127.0.0.1:1/never-called");
        match result {
            Err(Error::ConfigHttpRequestFailed { details, .. }) => {
                assert!(
                    details.contains("max_attempts is 0"),
                    "details should explain the zero-attempts case: {details}"
                );
            }
            other => panic!("expected ConfigHttpRequestFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn http_provider_surfaces_network_error() {
        ensure_crypto_provider();
        // Bind a listener to grab a free port, then drop it so the port is
        // closed when the resolver tries to connect.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind free port");
        let addr = listener.local_addr().expect("local addr");
        drop(listener);
        let uri = format!("http://{addr}/nope");

        let result = tokio::task::spawn_blocking(move || {
            HttpConfigProvider::with_max_attempts(1)
                .expect("provider should build")
                .resolve(&uri)
        })
        .await
        .expect("spawn_blocking join");
        match result {
            Err(Error::ConfigHttpRequestFailed { details, .. }) => {
                assert!(
                    !details.is_empty(),
                    "details should surface the network error"
                );
            }
            other => panic!("expected ConfigHttpRequestFailed, got {other:?}"),
        }
    }

    mod https {
        use super::*;
        use otel_arrow_dfe_test_tls_certs::{ExtendedKeyUsage, GeneratedCert, generate_ca};
        use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio_rustls::TlsAcceptor;

        async fn server(
            cert: GeneratedCert,
            responses: Vec<String>,
        ) -> (String, impl Future<Output = std::io::Result<Vec<String>>>) {
            ensure_crypto_provider();
            let config = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(
                    CertificateDer::pem_slice_iter(cert.cert_pem.as_bytes())
                        .collect::<Result<_, _>>()
                        .expect("parse server certificate"),
                    PrivateKeyDer::from_pem_slice(cert.key_pem.as_bytes())
                        .expect("parse server key"),
                )
                .expect("build TLS server");
            // TlsAcceptor requires an Arc for its immutable TLS configuration.
            let acceptor = TlsAcceptor::from(std::sync::Arc::new(config));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind HTTPS fixture");
            let uri = format!("https://{}/pipeline", listener.local_addr().unwrap());
            let serve = async move {
                let mut requests = Vec::new();
                for response in responses {
                    let (tcp, _) = listener.accept().await?;
                    let mut stream = acceptor.accept(tcp).await?;
                    let mut request = Vec::new();
                    let mut buf = [0; 1024];
                    while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                        let count = stream.read(&mut buf).await?;
                        if count == 0 || request.len() + count > 16 * 1024 {
                            return Err(std::io::Error::other("incomplete or oversized request"));
                        }
                        request.extend_from_slice(&buf[..count]);
                    }
                    requests.push(String::from_utf8(request).expect("HTTP request is UTF-8"));
                    stream.write_all(response.as_bytes()).await?;
                    stream.shutdown().await?;
                }
                Ok(requests)
            };
            (uri, serve)
        }

        fn response(status: u16, headers: &str, body: &str) -> String {
            format!(
                "HTTP/1.1 {status} Response\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
        }

        fn resolve(
            uri: &str,
            trusted_cert: Option<&str>,
            max_attempts: u64,
        ) -> Result<ResolvedConfig, Error> {
            let mut builder = HttpsConfigProvider::client_builder().no_proxy();
            if let Some(pem) = trusted_cert {
                builder = builder
                    .tls_certs_only([reqwest::Certificate::from_pem(pem.as_bytes()).unwrap()]);
            }
            let provider = HttpsConfigProvider(
                HttpConfigProvider::with_client_builder(builder, max_attempts)
                    .expect("build HTTPS provider"),
            );
            ConfigResolver::new(vec![Box::new(provider)]).resolve(uri)
        }

        /// Scenario: HTTPS serves valid YAML, JSON, or YAML without a content-type.
        /// Guarantees: HTTPS is registered by default and loads parseable configs without credentials.
        #[tokio::test]
        async fn https_loads_yaml_and_json() {
            ensure_crypto_provider();
            // Local certificates are trusted only in the injected test client, not the OS store.
            tokio::task::spawn_blocking(|| {
                let resolver = default_resolver().expect("build default resolver");
                assert!(resolver.providers.iter().any(|p| p.scheme() == "https"));
            })
            .await
            .unwrap();
            let ca = generate_ca("config test CA");
            for (headers, body, format) in [
                (
                    "Content-Type: application/yaml\r\n",
                    "version: otel_dataflow/v1\n",
                    ConfigFormat::Yaml,
                ),
                (
                    "Content-Type: application/json; charset=utf-8\r\n",
                    r#"{"version":"otel_dataflow/v1"}"#,
                    ConfigFormat::Json,
                ),
                ("", "version: otel_dataflow/v1\n", ConfigFormat::Yaml),
            ] {
                let cert = ca.issue_leaf(
                    "config server",
                    Some("127.0.0.1"),
                    Some(ExtendedKeyUsage::ServerAuth),
                );
                let (uri, serve) = server(cert, vec![response(200, headers, body)]).await;
                let expected_uri = uri.clone();
                let pem = ca.cert_pem.clone();
                // The synchronous config resolver must run outside the async fixture's runtime.
                let fetch = tokio::task::spawn_blocking(move || resolve(&uri, Some(&pem), 1));
                let (requests, resolved) =
                    tokio::join!(tokio::time::timeout(Duration::from_secs(60), serve), fetch);
                let requests = requests
                    .expect("HTTPS fixture deadline")
                    .expect("serve config");
                assert!(requests[0].starts_with("GET /pipeline HTTP/1.1\r\n"));
                assert!(
                    !requests[0]
                        .to_ascii_lowercase()
                        .contains("\r\nauthorization:")
                );
                let resolved = resolved.unwrap().expect("resolve HTTPS config");
                assert_eq!(resolved.source, expected_uri);
                assert_eq!(resolved.content, body);
                assert_eq!(resolved.format, format);
                let _spec = match format {
                    ConfigFormat::Yaml => {
                        crate::engine::OtelDataflowSpec::from_yaml(&resolved.content)
                    }
                    ConfigFormat::Json => {
                        crate::engine::OtelDataflowSpec::from_json(&resolved.content)
                    }
                }
                .expect("remote config parses as a valid root spec");
            }
        }

        /// Scenario: a trusted HTTPS endpoint redirects to another HTTPS path.
        /// Guarantees: secure redirects load the final body while preserving the original source URI.
        #[tokio::test]
        async fn https_follows_secure_redirect() {
            let ca = generate_ca("redirect test CA");
            let cert = ca.issue_leaf(
                "config server",
                Some("127.0.0.1"),
                Some(ExtendedKeyUsage::ServerAuth),
            );
            let (uri, serve) = server(
                cert,
                vec![
                    response(302, "Location: /actual\r\n", ""),
                    response(200, "", "version: otel_dataflow/v1\n"),
                ],
            )
            .await;
            let source = uri.clone();
            let fetch = tokio::task::spawn_blocking(move || resolve(&uri, Some(&ca.cert_pem), 1));
            let (requests, resolved) =
                tokio::join!(tokio::time::timeout(Duration::from_secs(60), serve), fetch);
            let requests = requests.unwrap().expect("serve redirect and config");
            assert_eq!(requests.len(), 2);
            assert!(requests[1].starts_with("GET /actual HTTP/1.1\r\n"));
            let resolved = resolved.unwrap().expect("follow HTTPS redirect");
            assert_eq!(resolved.source, source);
            assert_eq!(resolved.content, "version: otel_dataflow/v1\n");
        }

        /// Scenario: a trusted HTTPS endpoint redirects to a listening plaintext HTTP endpoint.
        /// Guarantees: resolution reports the forbidden scheme without contacting the HTTP target.
        #[tokio::test]
        async fn https_rejects_plaintext_redirect() {
            let target = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let headers = format!(
                "Location: http://{}/plaintext\r\n",
                target.local_addr().unwrap()
            );
            let ca = generate_ca("downgrade test CA");
            let cert = ca.issue_leaf(
                "config server",
                Some("127.0.0.1"),
                Some(ExtendedKeyUsage::ServerAuth),
            );
            let (uri, serve) = server(cert, vec![response(302, &headers, "")]).await;
            let source = uri.clone();
            let fetch = tokio::task::spawn_blocking(move || resolve(&uri, Some(&ca.cert_pem), 1));
            let (requests, resolved) =
                tokio::join!(tokio::time::timeout(Duration::from_secs(60), serve), fetch);
            assert_eq!(requests.unwrap().expect("serve redirect").len(), 1);
            match resolved.unwrap() {
                Err(Error::ConfigHttpRequestFailed { uri, details }) => {
                    assert_eq!(uri, source);
                    assert!(details.to_ascii_lowercase().contains("scheme"), "{details}");
                }
                other => panic!("expected downgrade rejection, got {other:?}"),
            }
            target.set_nonblocking(true).unwrap();
            assert_eq!(
                target.accept().unwrap_err().kind(),
                std::io::ErrorKind::WouldBlock
            );
        }

        /// Scenario: HTTPS presents an untrusted, hostname-mismatched, or expired certificate.
        /// Guarantees: verification fails before any config is served and the TLS cause reaches callers.
        #[tokio::test]
        async fn https_rejects_invalid_certificates() {
            let ca = generate_ca("verification test CA");
            let mut expired = rcgen::CertificateParams::new(vec!["127.0.0.1".to_string()]).unwrap();
            expired.not_before = rcgen::date_time_ymd(2000, 1, 1);
            expired.not_after = rcgen::date_time_ymd(2001, 1, 1);
            let key = rcgen::KeyPair::generate().unwrap();
            let expired = GeneratedCert {
                cert_pem: expired.self_signed(&key).unwrap().pem(),
                key_pem: key.serialize_pem(),
            };
            let expired_pem = expired.cert_pem.clone();
            for (cert, trusted, cause) in [
                (
                    ca.issue_leaf(
                        "config server",
                        Some("127.0.0.1"),
                        Some(ExtendedKeyUsage::ServerAuth),
                    ),
                    None,
                    "UnknownIssuer",
                ),
                (
                    ca.issue_leaf(
                        "wrong server",
                        Some("wrong.example"),
                        Some(ExtendedKeyUsage::ServerAuth),
                    ),
                    Some(ca.cert_pem.clone()),
                    "NotValidForName",
                ),
                (expired, Some(expired_pem), "Expired"),
            ] {
                let (uri, serve) = server(cert, vec![response(200, "", "must not load")]).await;
                let source = uri.clone();
                let fetch =
                    tokio::task::spawn_blocking(move || resolve(&uri, trusted.as_deref(), 1));
                let (served, resolved) =
                    tokio::join!(tokio::time::timeout(Duration::from_secs(60), serve), fetch);
                assert!(served.expect("TLS rejection deadline").is_err());
                match resolved.unwrap() {
                    Err(Error::ConfigHttpRequestFailed { uri, details }) => {
                        assert_eq!(uri, source);
                        assert!(details.contains(cause), "expected {cause}: {details}");
                    }
                    other => panic!("expected TLS verification failure, got {other:?}"),
                }
            }
        }

        /// Scenario: an HTTPS config endpoint returns transient 503s or a final 404.
        /// Guarantees: HTTPS shares HTTP retry handling and reports unsuccessful status codes.
        #[tokio::test]
        async fn https_retries_and_reports_http_errors() {
            let ca = generate_ca("retry test CA");
            for (responses, max_attempts, succeeds) in [
                (
                    vec![
                        response(503, "", ""),
                        response(200, "", "version: otel_dataflow/v1\n"),
                    ],
                    2,
                    true,
                ),
                (vec![response(404, "", "")], 1, false),
            ] {
                let expected_requests = responses.len();
                let cert = ca.issue_leaf(
                    "config server",
                    Some("127.0.0.1"),
                    Some(ExtendedKeyUsage::ServerAuth),
                );
                let (uri, serve) = server(cert, responses).await;
                let pem = ca.cert_pem.clone();
                // Real I/O uses real time; backoff duration is covered by the existing unit test.
                let fetch =
                    tokio::task::spawn_blocking(move || resolve(&uri, Some(&pem), max_attempts));
                let (requests, resolved) =
                    tokio::join!(tokio::time::timeout(Duration::from_secs(60), serve), fetch);
                assert_eq!(
                    requests.unwrap().expect("serve HTTPS responses").len(),
                    expected_requests
                );
                let resolved = resolved.unwrap();
                if succeeds {
                    assert_eq!(
                        resolved.expect("retry succeeds").content,
                        "version: otel_dataflow/v1\n"
                    );
                } else {
                    assert!(
                        matches!(resolved, Err(Error::ConfigHttpRequestFailed { details, .. }) if details.contains("404"))
                    );
                }
            }
        }
    }
}
