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
//! | `/path/to/config.yaml` | Bare path, treated as `file:` |
//! | `./relative/config.yaml` | Relative path, treated as `file:` |
//!
//! When no `--config` is provided, a default path in the current directory is tried.

use crate::error::Error;
use std::path::Path;

/// Fallback config path tried when `--config` is omitted.
const DEFAULT_CONFIG_PATH: &str = "config.yaml";

/// The result of resolving a config URI: the original source URI and the loaded content.
#[derive(Debug)]
pub struct ResolvedConfig {
    /// The original URI or path string used to load the config (e.g. "file:/etc/config.yaml", "env:MY_VAR").
    pub source: String,
    /// The raw configuration content (YAML or JSON string).
    pub content: String,
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
        Ok(ResolvedConfig {
            source: uri.to_string(),
            content,
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
        })
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

/// Returns a [`ConfigResolver`] with the default `file:` and `env:` providers.
#[must_use]
pub fn default_resolver() -> ConfigResolver {
    ConfigResolver::new(vec![
        Box::new(FileConfigProvider),
        Box::new(EnvConfigProvider),
    ])
}

/// Top-level entry point for resolving a config URI to content.
///
/// - `Some(uri)`: parse scheme and dispatch to the appropriate provider.
///   Bare paths (no scheme, starts with `/` or `.`) are treated as `file:`.
/// - `None`: try `DEFAULT_CONFIG_PATH` in the current working directory.
pub fn resolve_config(uri: Option<&str>) -> Result<ResolvedConfig, Error> {
    let resolver = default_resolver();

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
            .map_or(false, |c| c.is_ascii_alphabetic())
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

    #[test]
    fn file_provider_reads_temp_file() {
        let mut tmp = tempfile::NamedTempFile::new().expect("create temp file");
        write!(tmp, "hello: world").expect("write temp file");

        let provider = FileConfigProvider;
        let path = format!("file:{}", tmp.path().display());
        let resolved = provider.resolve(&path).expect("should read file");
        assert_eq!(resolved.content, "hello: world");
        assert_eq!(resolved.source, path);
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
        let resolved = provider
            .resolve(&uri)
            .expect("should read env var");
        assert_eq!(resolved.content, "version: v1");
        assert_eq!(resolved.source, uri);
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
        let mut tmp = tempfile::NamedTempFile::new().expect("create temp file");
        write!(tmp, "bare: path").expect("write temp file");

        let resolver = default_resolver();
        let resolved = resolver
            .resolve(&tmp.path().display().to_string())
            .expect("bare path should resolve as file");
        assert_eq!(resolved.content, "bare: path");
    }

    #[test]
    fn unknown_scheme_returns_error() {
        let resolver = default_resolver();
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
}
