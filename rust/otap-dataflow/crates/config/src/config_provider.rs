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
use std::cell::{OnceCell, RefCell};
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

/// Fallback config path tried when `--config` is omitted.
const DEFAULT_CONFIG_PATH: &str = "config.yaml";

/// Resource limits for HTTP and HTTPS configuration sources.
#[derive(Clone, Copy, Debug)]
pub struct RemoteConfigOptions {
    /// Total fetch attempts, including the first request. Must be nonzero.
    pub max_attempts: u64,
    /// Maximum response-body bytes, including bodies without a declared length.
    /// Must be nonzero.
    pub max_bytes: usize,
}

impl Default for RemoteConfigOptions {
    fn default() -> Self {
        Self {
            max_attempts: u64::MAX,
            max_bytes: 5 * 1024 * 1024,
        }
    }
}

/// Replace supplied credentials, queries, and fragments with `[REDACTED]` markers.
/// Preserve which components were supplied, but hide query names as well as values.
/// The original source must still be used when issuing a request.
#[must_use]
pub fn redact_config_source(source: &str) -> String {
    let Some(scheme) = parse_scheme(source).filter(|scheme| {
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
    }) else {
        return source.to_string();
    };
    let Ok(mut url) = reqwest::Url::parse(source) else {
        return format!("{scheme}:<invalid URI>");
    };
    // Render userinfo separately so URL serialization does not encode the marker's brackets.
    let userinfo = match (!url.username().is_empty(), url.password().is_some()) {
        (true, true) => "[REDACTED]:[REDACTED]@",
        (true, false) => "[REDACTED]@",
        (false, true) => ":[REDACTED]@",
        (false, false) => "",
    };
    url.set_username("").expect("HTTP URLs support userinfo");
    url.set_password(None).expect("HTTP URLs support userinfo");
    url.set_query(url.query().map(|_| "[REDACTED]"));
    url.set_fragment(url.fragment().map(|_| "[REDACTED]"));
    let (scheme, rest) = url
        .as_str()
        .split_once("://")
        .expect("HTTP URLs have an authority");
    format!("{scheme}://{userinfo}{rest}")
}

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

    /// Resolve while reporting retry failures. Providers without retries ignore
    /// the reporter. Remote providers report immediately before the first retry,
    /// then at most once every 30 seconds.
    fn resolve_with_reporter(
        &self,
        uri: &str,
        _report_failure: &dyn Fn(&Error),
    ) -> Result<ResolvedConfig, Error> {
        self.resolve(uri)
    }
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
    client: OnceCell<Result<reqwest::blocking::Client, String>>,
    client_builder: RefCell<Option<reqwest::blocking::ClientBuilder>>,
    options: RemoteConfigOptions,
}

impl HttpConfigProvider {
    /// Start delay for exponential backoff between retries.
    const BASE_BACKOFF: Duration = Duration::from_millis(500);
    /// Upper bound on the sleep between retries regardless of attempt count.
    const MAX_BACKOFF: Duration = Duration::from_secs(60);
    const REPORT_INTERVAL: Duration = Duration::from_secs(30);

    /// Build a provider that retries failed fetches indefinitely with
    /// exponential backoff (default: [`u64::MAX`] attempts).
    ///
    /// Client construction is deferred until resolution; registering an unused
    /// provider does not initialize TLS or start a blocking-client runtime.
    pub fn new() -> Result<Self, Error> {
        Self::with_options(RemoteConfigOptions::default())
    }

    /// Build a provider with an explicit cap on retry attempts. `max_attempts`
    /// counts the first try as an attempt, so `1` disables retries entirely.
    ///
    /// Returns an error when the attempt count is zero.
    pub fn with_max_attempts(max_attempts: u64) -> Result<Self, Error> {
        Self::with_options(RemoteConfigOptions {
            max_attempts,
            ..RemoteConfigOptions::default()
        })
    }

    /// Register a lazy HTTP client with explicit retry and response-size limits.
    pub fn with_options(options: RemoteConfigOptions) -> Result<Self, Error> {
        Self::with_client_builder(Self::client_builder(), options)
    }

    fn client_builder() -> reqwest::blocking::ClientBuilder {
        let default_redirects = reqwest::redirect::Policy::default();
        reqwest::blocking::Client::builder()
            .referer(false)
            .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                if has_credentials(attempt.url()) {
                    attempt.error("URL credentials are not supported by config providers")
                } else {
                    default_redirects.redirect(attempt)
                }
            }))
    }

    fn with_client_builder(
        builder: reqwest::blocking::ClientBuilder,
        options: RemoteConfigOptions,
    ) -> Result<Self, Error> {
        if options.max_attempts == 0 || options.max_bytes == 0 {
            return Err(Error::ConfigRemoteOptionsInvalid);
        }
        Ok(Self {
            client: OnceCell::new(),
            client_builder: RefCell::new(Some(builder.timeout(Duration::from_secs(30)))),
            options,
        })
    }

    fn client(&self) -> Result<&reqwest::blocking::Client, Error> {
        self.client
            .get_or_init(|| {
                self.client_builder
                    .borrow_mut()
                    .take()
                    .expect("client builder is consumed only during initialization")
                    .build()
                    .map_err(|error| remote_error_details(&error))
            })
            .as_ref()
            .map_err(|details| Error::ConfigHttpClientBuildFailed {
                details: details.clone(),
            })
    }

    fn try_fetch(
        &self,
        client: &reqwest::blocking::Client,
        uri: &str,
    ) -> Result<ResolvedConfig, Error> {
        let response = client
            .get(uri)
            // Blocking-client timeouts reset on each Read. Also set the async
            // request deadline so a slow stream cannot keep extending the fetch.
            .timeout(Duration::from_secs(30))
            .send()
            .map_err(|e| Error::ConfigHttpRequestFailed {
                uri: redact_config_source(uri),
                details: remote_error_details(&e),
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(Error::ConfigHttpRequestFailed {
                uri: redact_config_source(uri),
                details: format!("unexpected HTTP status {status}"),
            });
        }

        let format = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .filter(|ct| ct.starts_with("application/json"))
            .map_or(ConfigFormat::Yaml, |_| ConfigFormat::Json);

        let limit = u64::try_from(self.options.max_bytes).expect("usize fits in u64");
        let too_large = || Error::ConfigHttpResponseTooLarge {
            uri: redact_config_source(uri),
            max_bytes: self.options.max_bytes,
        };
        if response.content_length().is_some_and(|size| size > limit) {
            return Err(too_large());
        }
        let headers = response.headers().clone();
        let mut body = Vec::new();
        let _ = response
            .take(limit.saturating_add(1))
            .read_to_end(&mut body)
            .map_err(|error| Error::ConfigHttpRequestFailed {
                uri: redact_config_source(uri),
                details: format!("read body: {}", remote_error_details(&error)),
            })?;
        if body.len() > self.options.max_bytes {
            return Err(too_large());
        }

        // Decode only the bounded body, retaining reqwest's charset/BOM handling.
        let mut bounded_response = http::Response::new(body);
        *bounded_response.headers_mut() = headers;
        let content = reqwest::blocking::Response::from(bounded_response)
            .text()
            .map_err(|error| Error::ConfigHttpRequestFailed {
                uri: redact_config_source(uri),
                details: format!("decode body: {}", remote_error_details(&error)),
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

    fn should_report(last_reported: &mut Option<Instant>, now: Instant) -> bool {
        if last_reported.is_none_or(|last| now.duration_since(last) >= Self::REPORT_INTERVAL) {
            *last_reported = Some(now);
            true
        } else {
            false
        }
    }

    fn resolve_inner(
        &self,
        uri: &str,
        report_failure: Option<&dyn Fn(&Error)>,
    ) -> Result<ResolvedConfig, Error> {
        let url = reqwest::Url::parse(uri).map_err(|error| Error::ConfigRemoteUriInvalid {
            uri: redact_config_source(uri),
            details: error.to_string(),
        })?;
        if has_credentials(&url) {
            return Err(Error::ConfigRemoteUriInvalid {
                uri: redact_config_source(uri),
                details: "URL credentials are not supported; use an unauthenticated endpoint"
                    .to_string(),
            });
        }
        let client = self.client()?;
        let mut last_reported = None;
        for attempt in 0..self.options.max_attempts {
            match self.try_fetch(client, uri) {
                Ok(resolved) => return Ok(resolved),
                Err(error @ Error::ConfigHttpResponseTooLarge { .. }) => return Err(error),
                Err(error) => {
                    if attempt + 1 == self.options.max_attempts {
                        return Err(error);
                    }
                    if let Some(report) = report_failure
                        && Self::should_report(&mut last_reported, Instant::now())
                    {
                        report(&error);
                    }
                    std::thread::sleep(Self::backoff_for(attempt));
                }
            }
        }
        unreachable!("remote options require at least one attempt")
    }
}

impl ConfigProvider for HttpConfigProvider {
    fn scheme(&self) -> &str {
        "http"
    }

    fn resolve(&self, uri: &str) -> Result<ResolvedConfig, Error> {
        self.resolve_inner(uri, None)
    }

    fn resolve_with_reporter(
        &self,
        uri: &str,
        report_failure: &dyn Fn(&Error),
    ) -> Result<ResolvedConfig, Error> {
        self.resolve_inner(uri, Some(report_failure))
    }
}

fn has_credentials(url: &reqwest::Url) -> bool {
    !url.username().is_empty() || url.password().is_some()
}

fn remote_error_details(error: &(dyn std::error::Error + 'static)) -> String {
    let mut details = Vec::new();
    let mut cause = Some(error);
    while let Some(error) = cause {
        if let Some(error) = error.downcast_ref::<reqwest::Error>() {
            // Both outer and nested reqwest errors may carry a secret-bearing URL.
            details.push(
                if error.is_timeout() {
                    "request timed out"
                } else if error.is_redirect() {
                    "redirect failed"
                } else if error.is_connect() {
                    "connection failed"
                } else {
                    "HTTP request failed"
                }
                .to_string(),
            );
        } else if let Some(inner) = error
            .downcast_ref::<std::io::Error>()
            .and_then(std::io::Error::get_ref)
        {
            cause = Some(inner);
            continue;
        } else {
            details.push(error.to_string());
        }
        cause = error.source();
    }
    details.join(": ")
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
    /// Client construction is deferred until the first resolution.
    pub fn new() -> Result<Self, Error> {
        Self::with_options(RemoteConfigOptions::default())
    }

    /// Register a lazy HTTPS client with explicit retry and response-size limits.
    pub fn with_options(options: RemoteConfigOptions) -> Result<Self, Error> {
        HttpConfigProvider::with_client_builder(Self::client_builder(), options).map(Self)
    }

    fn client_builder() -> reqwest::blocking::ClientBuilder {
        HttpConfigProvider::client_builder().https_only(true)
    }
}

impl ConfigProvider for HttpsConfigProvider {
    fn scheme(&self) -> &str {
        "https"
    }

    fn resolve(&self, uri: &str) -> Result<ResolvedConfig, Error> {
        self.0.resolve(uri)
    }

    fn resolve_with_reporter(
        &self,
        uri: &str,
        report_failure: &dyn Fn(&Error),
    ) -> Result<ResolvedConfig, Error> {
        self.0.resolve_with_reporter(uri, report_failure)
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
        self.provider(uri)?.resolve(uri)
    }

    /// Resolve with an optional retry-failure reporter for remote sources.
    pub fn resolve_with_reporter(
        &self,
        uri: &str,
        report_failure: &dyn Fn(&Error),
    ) -> Result<ResolvedConfig, Error> {
        self.provider(uri)?
            .resolve_with_reporter(uri, report_failure)
    }

    fn provider(&self, uri: &str) -> Result<&dyn ConfigProvider, Error> {
        let scheme = parse_scheme(uri).unwrap_or("file");
        self.providers
            .iter()
            .find(|provider| provider.scheme() == scheme)
            .map(|provider| provider.as_ref())
            .ok_or_else(|| Error::ConfigUriUnknownScheme {
                scheme: scheme.to_string(),
            })
    }
}

/// Returns a [`ConfigResolver`] with the default providers: `file:`, `env:`,
/// `yaml:`, `http:`, and `https:`.
///
/// Network clients are not constructed until their provider is selected.
pub fn default_resolver() -> Result<ConfigResolver, Error> {
    resolver_with_options(RemoteConfigOptions::default())
}

/// Register the default providers with explicit remote-source limits.
pub fn resolver_with_options(options: RemoteConfigOptions) -> Result<ConfigResolver, Error> {
    Ok(ConfigResolver::new(vec![
        Box::new(FileConfigProvider),
        Box::new(EnvConfigProvider),
        Box::new(YamlConfigProvider),
        Box::new(HttpConfigProvider::with_options(options)?),
        Box::new(HttpsConfigProvider::with_options(options)?),
    ]))
}

/// Top-level entry point for resolving a config URI to content.
///
/// - `Some(uri)`: parse scheme and dispatch to the appropriate provider.
///   Bare paths (no scheme, starts with `/` or `.`) are treated as `file:`.
/// - `None`: try `DEFAULT_CONFIG_PATH` in the current working directory.
pub fn resolve_config(uri: Option<&str>) -> Result<ResolvedConfig, Error> {
    resolve_config_with_options(uri, RemoteConfigOptions::default(), None)
}

/// Resolve with remote-source limits and an optional rate-limited retry reporter.
/// Final failures are returned to the caller rather than reported as retries.
pub fn resolve_config_with_options(
    uri: Option<&str>,
    options: RemoteConfigOptions,
    report_failure: Option<&dyn Fn(&Error)>,
) -> Result<ResolvedConfig, Error> {
    let resolver = resolver_with_options(options)?;
    let uri = match uri {
        Some(uri) => uri,
        None => {
            if !Path::new(DEFAULT_CONFIG_PATH).exists() {
                return Err(Error::ConfigNoFileFound {
                    path: DEFAULT_CONFIG_PATH.to_string(),
                });
            }
            DEFAULT_CONFIG_PATH
        }
    };
    match report_failure {
        Some(report) => resolver.resolve_with_reporter(uri, report),
        None => resolver.resolve(uri),
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

    /// Scenario: an unsupported config URI scheme is resolved.
    /// Guarantees: the rendered diagnostic includes HTTPS among the supported sources.
    #[test]
    fn unknown_scheme_returns_error() {
        ensure_crypto_provider();
        let resolver = default_resolver().expect("default resolver should build");
        let result = resolver.resolve("ftp://example.com/config.yaml");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("http:, https:"));
        match error {
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

    /// Scenario: a remote provider is registered without selecting a remote source.
    /// Guarantees: local resolution works inside a runtime without constructing blocking/TLS clients.
    #[tokio::test]
    async fn local_sources_do_not_build_remote_clients() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "version: v1").unwrap();
        let resolver = default_resolver().expect("register providers without building clients");
        assert_eq!(
            resolver
                .resolve(&file.path().display().to_string())
                .unwrap()
                .content,
            "version: v1\n"
        );
        assert_eq!(
            resolver.resolve("yaml:engine::{}").unwrap().content,
            "engine:\n  {}"
        );
        assert!(matches!(
            resolver.resolve("env:"),
            Err(Error::ConfigEnvVarNotSet { .. })
        ));

        let provider = HttpConfigProvider::with_client_builder(
            HttpConfigProvider::client_builder().user_agent("\n"),
            RemoteConfigOptions::default(),
        )
        .expect("defer invalid client construction");
        assert!(provider.client.get().is_none());
        // A selected client still reports build failures, before issuing a request.
        let error =
            tokio::task::spawn_blocking(move || provider.resolve("http://127.0.0.1/pipeline"))
                .await
                .unwrap()
                .unwrap_err();
        assert!(matches!(error, Error::ConfigHttpClientBuildFailed { .. }));
    }

    /// Scenario: HTTP(S) sources have different combinations of sensitive URL components.
    /// Guarantees: markers preserve parsed component presence, including empty queries and fragments, without exposing contents.
    #[test]
    fn remote_source_redaction() {
        for (source, expected) in [
            (
                "https://user-secret:password-secret@example.com:8443/pipeline?signature=query-secret#fragment-secret",
                "https://[REDACTED]:[REDACTED]@example.com:8443/pipeline?[REDACTED]#[REDACTED]",
            ),
            (
                "HTTP://user-secret:password-secret@example.com/pipeline?signature=query-secret#fragment-secret",
                "http://[REDACTED]:[REDACTED]@example.com/pipeline?[REDACTED]#[REDACTED]",
            ),
            (
                "https://user-secret@example.com/pipeline",
                "https://[REDACTED]@example.com/pipeline",
            ),
            (
                "https://:password-secret@example.com/pipeline",
                "https://:[REDACTED]@example.com/pipeline",
            ),
            (
                "https://example.com/pipeline?signature=query-secret",
                "https://example.com/pipeline?[REDACTED]",
            ),
            (
                "https://example.com/pipeline#fragment-secret",
                "https://example.com/pipeline#[REDACTED]",
            ),
            // URL parsing normalizes an empty password to no password.
            (
                "https://user-secret:@example.com/pipeline?#",
                "https://[REDACTED]@example.com/pipeline?[REDACTED]#[REDACTED]",
            ),
            (
                "https://[::1]:8443/pipeline?#",
                "https://[::1]:8443/pipeline?[REDACTED]#[REDACTED]",
            ),
            (
                "https://example.com/pipeline",
                "https://example.com/pipeline",
            ),
        ] {
            let redacted = redact_config_source(source);
            assert_eq!(redacted, expected);
            assert_eq!(redact_config_source(&redacted), expected);
            for secret in [
                "user-secret",
                "password-secret",
                "signature",
                "query-secret",
                "fragment-secret",
            ] {
                assert!(!redacted.contains(secret), "{redacted}");
            }
        }
        assert_eq!(
            redact_config_source("https://[invalid?secret"),
            "https:<invalid URI>"
        );
        assert_eq!(redact_config_source("file:config.yaml"), "file:config.yaml");
    }

    /// Scenario: retry failures arrive before, at, and after the diagnostic interval.
    /// Guarantees: the first failure is immediately reportable and later reports are 30 seconds apart.
    #[test]
    fn remote_retry_reports_are_rate_limited() {
        let now = Instant::now();
        let mut last = None;
        assert!(HttpConfigProvider::should_report(&mut last, now));
        assert!(!HttpConfigProvider::should_report(
            &mut last,
            now + Duration::from_secs(29)
        ));
        assert!(HttpConfigProvider::should_report(
            &mut last,
            now + Duration::from_secs(30)
        ));
        assert!(!HttpConfigProvider::should_report(
            &mut last,
            now + Duration::from_secs(31)
        ));
        assert!(HttpConfigProvider::should_report(
            &mut last,
            now + Duration::from_secs(60)
        ));
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

    /// Scenario: an HTTP provider is registered using its default retry policy.
    /// Guarantees: startup retains indefinite retries without eagerly creating a network client.
    #[test]
    fn http_provider_new_defaults_to_unbounded_attempts() {
        let p = HttpConfigProvider::new().expect("provider should build");
        assert_eq!(p.options.max_attempts, u64::MAX);
        assert!(p.client.get().is_none());
        assert_eq!(p.scheme(), "http");
    }

    /// Scenario: a library caller sets a zero retry count or response-size limit.
    /// Guarantees: invalid options fail before client initialization or network access.
    #[test]
    fn http_provider_zero_attempts_returns_error() {
        assert!(matches!(
            HttpConfigProvider::with_max_attempts(0),
            Err(Error::ConfigRemoteOptionsInvalid)
        ));
        assert!(matches!(
            HttpsConfigProvider::with_options(RemoteConfigOptions {
                max_bytes: 0,
                ..RemoteConfigOptions::default()
            }),
            Err(Error::ConfigRemoteOptionsInvalid)
        ));
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
            resolve_with_options(
                uri,
                trusted_cert,
                RemoteConfigOptions {
                    max_attempts,
                    ..RemoteConfigOptions::default()
                },
                None,
            )
        }

        fn resolve_with_options(
            uri: &str,
            trusted_cert: Option<&str>,
            options: RemoteConfigOptions,
            report_failure: Option<&dyn Fn(&Error)>,
        ) -> Result<ResolvedConfig, Error> {
            let mut builder = HttpsConfigProvider::client_builder().no_proxy();
            if let Some(pem) = trusted_cert {
                builder = builder
                    .tls_certs_only([reqwest::Certificate::from_pem(pem.as_bytes()).unwrap()]);
            }
            let provider = HttpsConfigProvider(
                HttpConfigProvider::with_client_builder(builder, options)
                    .expect("build HTTPS provider"),
            );
            let resolver = ConfigResolver::new(vec![Box::new(provider)]);
            match report_failure {
                Some(report) => resolver.resolve_with_reporter(uri, report),
                None => resolver.resolve(uri),
            }
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

        /// Scenario: a signed HTTPS URL redirects to a distinct HTTPS origin.
        /// Guarantees: the original query is requested but no Referer or Authorization reaches either origin.
        #[tokio::test]
        async fn https_follows_secure_redirect() {
            let ca = generate_ca("redirect test CA");
            let cert = ca.issue_leaf(
                "config server",
                Some("127.0.0.1"),
                Some(ExtendedKeyUsage::ServerAuth),
            );
            let target_cert = ca.issue_leaf(
                "redirect target",
                Some("127.0.0.1"),
                Some(ExtendedKeyUsage::ServerAuth),
            );
            let (target_uri, serve_target) = server(
                target_cert,
                vec![response(200, "", "version: otel_dataflow/v1\n")],
            )
            .await;
            let (uri, serve_source) = server(
                cert,
                vec![response(302, &format!("Location: {target_uri}\r\n"), "")],
            )
            .await;
            assert_ne!(
                reqwest::Url::parse(&uri).unwrap().origin(),
                reqwest::Url::parse(&target_uri).unwrap().origin()
            );
            let uri = format!("{uri}?signature=source-secret#fragment-secret");
            let source = uri.clone();
            let fetch = tokio::task::spawn_blocking(move || resolve(&uri, Some(&ca.cert_pem), 1));
            let (source_requests, target_requests, resolved) = tokio::join!(
                tokio::time::timeout(Duration::from_secs(60), serve_source),
                tokio::time::timeout(Duration::from_secs(60), serve_target),
                fetch
            );
            let source_requests = source_requests.unwrap().expect("serve source redirect");
            let target_requests = target_requests.unwrap().expect("serve redirect target");
            assert!(
                source_requests[0]
                    .starts_with("GET /pipeline?signature=source-secret HTTP/1.1\r\n")
            );
            assert!(target_requests[0].starts_with("GET /pipeline HTTP/1.1\r\n"));
            for requests in [&source_requests, &target_requests] {
                assert_eq!(requests.len(), 1);
                let request = requests[0].to_ascii_lowercase();
                assert!(!request.contains("\r\nreferer:"), "{request}");
                assert!(!request.contains("\r\nauthorization:"), "{request}");
            }
            let resolved = resolved.unwrap().expect("follow HTTPS redirect");
            assert_eq!(resolved.source, source);
            assert_eq!(resolved.content, "version: otel_dataflow/v1\n");
        }

        /// Scenario: HTTPS config URLs include a username, password, or both.
        /// Guarantees: credentials fail before fetching, with markers showing which sensitive fields were supplied.
        #[test]
        fn https_rejects_url_credentials_before_fetch() {
            let target = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            for (userinfo, redacted_userinfo) in [
                ("user-secret@", "[REDACTED]@"),
                (":password-secret@", ":[REDACTED]@"),
                ("user-secret:password-secret@", "[REDACTED]:[REDACTED]@"),
            ] {
                let uri = format!(
                    "https://{userinfo}{}/pipeline?signature=query-secret#fragment-secret",
                    target.local_addr().unwrap()
                );
                let provider = HttpsConfigProvider::new().unwrap();
                let error = provider.resolve(&uri).unwrap_err();
                assert!(provider.0.client.get().is_none());
                match &error {
                    Error::ConfigRemoteUriInvalid { uri, .. } => assert_eq!(
                        uri,
                        &format!(
                            "https://{redacted_userinfo}{}/pipeline?[REDACTED]#[REDACTED]",
                            target.local_addr().unwrap()
                        )
                    ),
                    other => panic!("expected credential rejection, got {other:?}"),
                }
                for rendered in [error.to_string(), format!("{error:?}")] {
                    assert!(rendered.contains("credentials"), "{rendered}");
                    assert!(!rendered.contains("-secret"), "{rendered}");
                }
            }
            target.set_nonblocking(true).unwrap();
            assert_eq!(
                target.accept().unwrap_err().kind(),
                std::io::ErrorKind::WouldBlock
            );
        }

        /// Scenario: a trusted server redirects to a credentialed, signed HTTPS URL.
        /// Guarantees: the redirect is rejected without connecting or displaying either URL's secrets.
        #[tokio::test]
        async fn https_rejects_credentialed_redirects() {
            let target = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let ca = generate_ca("credential redirect CA");
            let cert = ca.issue_leaf(
                "source",
                Some("127.0.0.1"),
                Some(ExtendedKeyUsage::ServerAuth),
            );
            let location = format!(
                "Location: https://user-secret:password-secret@{}/pipeline?signature=target-secret\r\n",
                target.local_addr().unwrap()
            );
            let (uri, serve) = server(cert, vec![response(302, &location, "")]).await;
            let uri = format!("{uri}?signature=source-secret");
            let fetch = tokio::task::spawn_blocking(move || resolve(&uri, Some(&ca.cert_pem), 1));
            let (served, result) =
                tokio::join!(tokio::time::timeout(Duration::from_secs(60), serve), fetch);
            assert_eq!(
                served.unwrap().expect("serve credentialed redirect").len(),
                1
            );
            let error = result.unwrap().unwrap_err();
            assert!(error.to_string().contains("credentials"), "{error}");
            for rendered in [error.to_string(), format!("{error:?}")] {
                assert!(!rendered.contains("-secret"), "{rendered}");
            }
            target.set_nonblocking(true).unwrap();
            assert_eq!(
                target.accept().unwrap_err().kind(),
                std::io::ErrorKind::WouldBlock
            );
        }

        /// Scenario: a signed HTTPS source redirects to a signed target with the wrong TLS hostname.
        /// Guarantees: nested errors retain the TLS reason and mark supplied fields without exposing signed queries.
        #[tokio::test]
        async fn https_redacts_redirect_tls_errors() {
            let ca = generate_ca("redirect verification CA");
            let target_cert = ca.issue_leaf(
                "target",
                Some("wrong.example"),
                Some(ExtendedKeyUsage::ServerAuth),
            );
            let (target_uri, serve_target) =
                server(target_cert, vec![response(200, "", "must not load")]).await;
            let cert = ca.issue_leaf(
                "source",
                Some("127.0.0.1"),
                Some(ExtendedKeyUsage::ServerAuth),
            );
            let location = format!(
                "Location: {target_uri}?signature=target-secret#target-fragment-secret\r\n"
            );
            let (uri, serve_source) = server(cert, vec![response(302, &location, "")]).await;
            let uri = format!("{uri}?signature=source-secret#source-fragment-secret");
            let fetch = tokio::task::spawn_blocking(move || resolve(&uri, Some(&ca.cert_pem), 1));
            let (source, target, result) = tokio::join!(
                tokio::time::timeout(Duration::from_secs(60), serve_source),
                tokio::time::timeout(Duration::from_secs(60), serve_target),
                fetch
            );
            assert_eq!(source.unwrap().expect("serve redirect").len(), 1);
            assert!(target.expect("TLS rejection deadline").is_err());
            let error = result.unwrap().unwrap_err();
            assert!(
                error.to_string().contains("certificate not valid for name"),
                "{error}"
            );
            assert!(
                error.to_string().contains("?[REDACTED]#[REDACTED]"),
                "{error}"
            );
            for rendered in [error.to_string(), format!("{error:?}")] {
                assert!(!rendered.contains("-secret"), "{rendered}");
            }
        }

        /// Scenario: fixed-length, EOF-delimited, and chunked bodies meet or exceed the byte cap.
        /// Guarantees: exactly the limit succeeds; oversized bodies fail without retries, even without a length.
        #[tokio::test]
        async fn https_limits_streamed_response_bodies() {
            let ca = generate_ca("bounded body CA");
            let limit = "version: v1\n".len();
            for framing in ["length", "eof", "chunked"] {
                for oversized in [false, true] {
                    let body = if oversized {
                        "version: v1\n "
                    } else {
                        "version: v1\n"
                    };
                    let reply = match framing {
                        "length" => response(200, "", body),
                        "eof" => format!("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{body}"),
                        "chunked" => format!(
                            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n{body}\r\n0\r\n\r\n",
                            body.len()
                        ),
                        _ => unreachable!(),
                    };
                    let cert = ca.issue_leaf(
                        "source",
                        Some("127.0.0.1"),
                        Some(ExtendedKeyUsage::ServerAuth),
                    );
                    let (uri, serve) = server(cert, vec![reply]).await;
                    let uri = format!("{uri}?signature=body-secret");
                    let pem = ca.cert_pem.clone();
                    let fetch = tokio::task::spawn_blocking(move || {
                        resolve_with_options(
                            &uri,
                            Some(&pem),
                            RemoteConfigOptions {
                                max_attempts: 2,
                                max_bytes: limit,
                            },
                            None,
                        )
                    });
                    let (served, result) =
                        tokio::join!(tokio::time::timeout(Duration::from_secs(60), serve), fetch);
                    assert_eq!(served.unwrap().expect("serve bounded body").len(), 1);
                    let result = result.unwrap();
                    if oversized {
                        let error = result.unwrap_err();
                        assert!(
                            matches!(error, Error::ConfigHttpResponseTooLarge { max_bytes, .. } if max_bytes == limit)
                        );
                        assert!(!error.to_string().contains("body-secret"));
                    } else {
                        assert_eq!(result.expect("accept exactly the limit").content, body);
                    }
                }
            }
        }

        /// Scenario: a trusted HTTPS endpoint redirects to a listening plaintext HTTP endpoint.
        /// Guarantees: resolution reports the forbidden scheme and redacted component presence without contacting HTTP.
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
            let uri = format!("{uri}?signature=downgrade-secret#fragment-secret");
            let fetch = tokio::task::spawn_blocking(move || resolve(&uri, Some(&ca.cert_pem), 1));
            let (requests, resolved) =
                tokio::join!(tokio::time::timeout(Duration::from_secs(60), serve), fetch);
            assert_eq!(requests.unwrap().expect("serve redirect").len(), 1);
            let result = resolved.unwrap();
            let error = result.as_ref().unwrap_err();
            for rendered in [error.to_string(), format!("{error:?}")] {
                assert!(!rendered.contains("-secret"), "{rendered}");
            }
            match result {
                Err(Error::ConfigHttpRequestFailed { uri, details }) => {
                    assert_eq!(uri, format!("{source}?[REDACTED]#[REDACTED]"));
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
        /// Guarantees: verification fails before serving config and reports the TLS cause with redacted URL markers.
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
                    "certificate not valid for name",
                ),
                (expired, Some(expired_pem), "certificate expired"),
            ] {
                let (uri, serve) = server(cert, vec![response(200, "", "must not load")]).await;
                let source = uri.clone();
                let uri = format!("{uri}?signature=certificate-secret#fragment-secret");
                let fetch =
                    tokio::task::spawn_blocking(move || resolve(&uri, trusted.as_deref(), 1));
                let (served, resolved) =
                    tokio::join!(tokio::time::timeout(Duration::from_secs(60), serve), fetch);
                assert!(served.expect("TLS rejection deadline").is_err());
                let result = resolved.unwrap();
                let error = result.as_ref().unwrap_err();
                for rendered in [error.to_string(), format!("{error:?}")] {
                    assert!(!rendered.contains("-secret"), "{rendered}");
                }
                match result {
                    Err(Error::ConfigHttpRequestFailed { uri, details }) => {
                        assert_eq!(uri, format!("{source}?[REDACTED]#[REDACTED]"));
                        assert!(details.contains(cause), "expected {cause}: {details}");
                    }
                    other => panic!("expected TLS verification failure, got {other:?}"),
                }
            }
        }

        /// Scenario: an HTTPS config endpoint returns transient 503s or a final 404.
        /// Guarantees: HTTPS retries within its budget and marks supplied queries in retry and final diagnostics.
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
                (vec![response(503, "", ""), response(404, "", "")], 2, false),
            ] {
                let expected_requests = responses.len();
                let cert = ca.issue_leaf(
                    "config server",
                    Some("127.0.0.1"),
                    Some(ExtendedKeyUsage::ServerAuth),
                );
                let (uri, serve) = server(cert, responses).await;
                let uri = format!("{uri}?signature=retry-secret");
                let pem = ca.cert_pem.clone();
                // Real I/O uses real time; backoff duration is covered by the existing unit test.
                let fetch = tokio::task::spawn_blocking(move || {
                    let reports = RefCell::new(Vec::new());
                    let result = resolve_with_options(
                        &uri,
                        Some(&pem),
                        RemoteConfigOptions {
                            max_attempts,
                            ..RemoteConfigOptions::default()
                        },
                        Some(&|error| reports.borrow_mut().push(error.to_string())),
                    );
                    (result, reports.into_inner())
                });
                let (requests, resolved) =
                    tokio::join!(tokio::time::timeout(Duration::from_secs(60), serve), fetch);
                assert_eq!(
                    requests.unwrap().expect("serve HTTPS responses").len(),
                    expected_requests
                );
                let (resolved, reports) = resolved.unwrap();
                if succeeds {
                    assert_eq!(reports.len(), 1);
                    assert!(reports[0].contains("503"));
                    assert!(reports[0].contains("?[REDACTED]"));
                    assert!(!reports[0].contains("retry-secret"));
                    assert_eq!(
                        resolved.expect("retry succeeds").content,
                        "version: otel_dataflow/v1\n"
                    );
                } else {
                    assert_eq!(
                        reports.len(),
                        expected_requests - 1,
                        "final errors are returned, not reported as retries"
                    );
                    let error = resolved.unwrap_err();
                    let rendered = error.to_string();
                    assert!(rendered.contains("404"), "{rendered}");
                    assert!(rendered.contains("HTTP and HTTPS"), "{rendered}");
                    assert!(rendered.contains("?[REDACTED]"), "{rendered}");
                    assert!(!rendered.contains("not supported yet"), "{rendered}");
                    assert!(!rendered.contains("retry-secret"), "{rendered}");
                }
            }
        }
    }
}
