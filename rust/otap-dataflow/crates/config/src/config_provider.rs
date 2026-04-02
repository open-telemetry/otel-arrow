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
//! When no `--config` is provided, well-known paths are tried in order.

use crate::error::Error;
use std::path::Path;

/// Well-known config file paths tried when `--config` is omitted.
const WELL_KNOWN_PATHS: &[&str] = &[
    "/etc/o11y-gateway/config.yaml",
    "/etc/data-plane/configs/pipeline_config.yaml",
    "/app/config.yaml",
];

/// A provider that can resolve configuration content from a URI with a specific scheme.
pub trait ConfigProvider {
    /// The URI scheme this provider handles (e.g. "file", "env").
    fn scheme(&self) -> &str;

    /// Resolve the given URI to configuration content as a string.
    ///
    /// The `uri` is the full original URI string (including the scheme prefix).
    /// Implementations should strip their own scheme prefix.
    fn resolve(&self, uri: &str) -> Result<String, Error>;
}

/// Reads configuration from a local file path.
pub struct FileConfigProvider;

impl ConfigProvider for FileConfigProvider {
    fn scheme(&self) -> &str {
        "file"
    }

    fn resolve(&self, uri: &str) -> Result<String, Error> {
        let path = uri.strip_prefix("file:").unwrap_or(uri);
        std::fs::read_to_string(path).map_err(|e| Error::FileReadError {
            context: crate::error::Context::default(),
            details: format!("{path}: {e}"),
        })
    }
}

/// Reads configuration from an environment variable.
pub struct EnvConfigProvider;

impl ConfigProvider for EnvConfigProvider {
    fn scheme(&self) -> &str {
        "env"
    }

    fn resolve(&self, uri: &str) -> Result<String, Error> {
        let var = uri.strip_prefix("env:").unwrap_or(uri);
        std::env::var(var).map_err(|_| Error::ConfigEnvVarNotSet {
            var: var.to_string(),
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
    pub fn resolve(&self, uri: &str) -> Result<String, Error> {
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
/// - `None`: try well-known paths in order, returning the first found.
pub fn resolve_config(uri: Option<&str>) -> Result<String, Error> {
    let resolver = default_resolver();

    match uri {
        Some(u) => resolver.resolve(u),
        None => {
            for path in WELL_KNOWN_PATHS {
                if Path::new(path).exists() {
                    return resolver.resolve(path);
                }
            }
            Err(Error::ConfigNoFileFound {
                searched: crate::error::PathBulletList(
                    WELL_KNOWN_PATHS.iter().map(|p| (*p).to_string()).collect(),
                ),
            })
        }
    }
}

/// Parse the URI scheme, if any.
///
/// Returns `None` for bare paths (starts with `/`, `.`, or no `:` before a path separator).
/// Returns `Some(scheme)` for `scheme:rest` patterns.
fn parse_scheme(uri: &str) -> Option<&str> {
    // Bare absolute or relative paths have no scheme.
    if uri.starts_with('/') || uri.starts_with('.') {
        return None;
    }

    // Look for a colon. If there is no colon, it is a bare filename.
    let colon_pos = uri.find(':')?;

    // If a path separator appears before the colon, treat as bare path.
    let before_colon = &uri[..colon_pos];
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
        let content = provider.resolve(&path).expect("should read file");
        assert_eq!(content, "hello: world");
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
        let content = provider
            .resolve(&format!("env:{var_name}"))
            .expect("should read env var");
        assert_eq!(content, "version: v1");
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
        let content = resolver
            .resolve(&tmp.path().display().to_string())
            .expect("bare path should resolve as file");
        assert_eq!(content, "bare: path");
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
    fn resolve_config_none_with_no_well_known_paths() {
        let result = resolve_config(None);
        // On most test machines none of the well-known paths exist.
        if let Err(err) = result {
            match err {
                Error::ConfigNoFileFound { searched } => {
                    assert_eq!(searched.0.len(), WELL_KNOWN_PATHS.len());
                }
                other => panic!("expected ConfigNoFileFound, got {other:?}"),
            }
        }
        // If one of them happens to exist (e.g. in a container), that is fine too.
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
        let content =
            resolve_config(Some(&format!("env:{var_name}"))).expect("should resolve env config");
        // Verify it can be parsed as an OtelDataflowSpec.
        let spec = crate::engine::OtelDataflowSpec::from_yaml(&content);
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
    }
}
