// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Endpoint and auth configuration for the admin SDK.

use crate::error::EndpointError;
use std::net::SocketAddr;
use url::Url;

/// Supported admin endpoint schemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminScheme {
    /// Plain HTTP.
    Http,
    /// HTTPS.
    Https,
}

impl AdminScheme {
    /// Returns the URL scheme string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            AdminScheme::Http => "http",
            AdminScheme::Https => "https",
        }
    }
}

/// Future-extensible authentication configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AdminAuth {
    /// No authentication.
    #[default]
    None,
}

/// Admin endpoint location and URL composition settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminEndpoint {
    /// URL scheme.
    pub scheme: AdminScheme,
    /// Hostname or IP address.
    pub host: String,
    /// TCP port.
    pub port: u16,
    /// Optional URL base path prepended before `/api/v1/...`.
    pub base_path: Option<String>,
}

impl AdminEndpoint {
    /// Creates a new endpoint.
    pub fn new(
        scheme: AdminScheme,
        host: impl Into<String>,
        port: u16,
    ) -> Result<Self, EndpointError> {
        let endpoint = Self {
            scheme,
            host: host.into(),
            port,
            base_path: None,
        };
        endpoint.validate()?;
        Ok(endpoint)
    }

    /// Creates an HTTP endpoint.
    #[must_use]
    pub fn http(host: impl Into<String>, port: u16) -> Self {
        Self {
            scheme: AdminScheme::Http,
            host: host.into(),
            port,
            base_path: None,
        }
    }

    /// Creates an HTTPS endpoint.
    #[must_use]
    pub fn https(host: impl Into<String>, port: u16) -> Self {
        Self {
            scheme: AdminScheme::Https,
            host: host.into(),
            port,
            base_path: None,
        }
    }

    /// Creates an endpoint from a socket address using HTTP.
    #[must_use]
    pub fn from_socket_addr(addr: SocketAddr) -> Self {
        Self::http(addr.ip().to_string(), addr.port())
    }

    /// Creates an endpoint from a full base URL.
    pub fn from_url(url: &str) -> Result<Self, EndpointError> {
        let parsed = Url::parse(url).map_err(|err| EndpointError::UrlParse {
            url: url.to_string(),
            details: err.to_string(),
        })?;

        if parsed.query().is_some() || parsed.fragment().is_some() {
            return Err(EndpointError::InvalidBasePath {
                base_path: parsed.path().to_string(),
                reason: "query strings and fragments are not supported in admin endpoint URLs"
                    .to_string(),
            });
        }

        let scheme = match parsed.scheme() {
            "http" => AdminScheme::Http,
            "https" => AdminScheme::Https,
            other => {
                return Err(EndpointError::InvalidScheme {
                    scheme: other.to_string(),
                });
            }
        };

        let host = parsed
            .host_str()
            .ok_or_else(|| EndpointError::InvalidHost {
                host: url.to_string(),
            })?;
        let port = parsed
            .port_or_known_default()
            .ok_or(EndpointError::InvalidPort { port: 0 })?;

        let mut endpoint = Self::new(scheme, host.to_string(), port)?;
        let path = parsed.path().trim_end_matches('/');
        if !path.is_empty() && path != "/" {
            endpoint.base_path = Some(path.to_string());
        }
        endpoint.validate()?;
        Ok(endpoint)
    }

    /// Sets the base path used for URL construction.
    pub fn with_base_path(mut self, base_path: impl Into<String>) -> Result<Self, EndpointError> {
        self.base_path = Some(base_path.into());
        self.validate()?;
        Ok(self)
    }

    /// Validates the endpoint fields.
    pub fn validate(&self) -> Result<(), EndpointError> {
        if self.host.trim().is_empty() {
            return Err(EndpointError::EmptyHost);
        }
        if let Some(base_path) = &self.base_path {
            if !base_path.is_empty() && !base_path.starts_with('/') {
                return Err(EndpointError::InvalidBasePath {
                    base_path: base_path.clone(),
                    reason: "base path must start with '/'".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Builds a URL for the provided path segments.
    pub fn url_for_segments<'a, I>(&self, segments: I) -> Result<Url, EndpointError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        self.validate()?;

        let mut url = Url::parse("http://localhost/").map_err(|err| EndpointError::UrlBuild {
            details: err.to_string(),
        })?;
        url.set_scheme(self.scheme.as_str())
            .map_err(|_| EndpointError::InvalidScheme {
                scheme: self.scheme.as_str().to_string(),
            })?;
        url.set_host(Some(&self.host))
            .map_err(|_| EndpointError::InvalidHost {
                host: self.host.clone(),
            })?;
        url.set_port(Some(self.port))
            .map_err(|_| EndpointError::InvalidPort { port: self.port })?;

        {
            let mut path_segments =
                url.path_segments_mut()
                    .map_err(|_| EndpointError::InvalidBasePath {
                        base_path: self.base_path.clone().unwrap_or_default(),
                        reason: "endpoint cannot be used as a base URL".to_string(),
                    })?;
            _ = path_segments.clear();

            if let Some(base_path) = &self.base_path {
                for segment in base_path.split('/').filter(|segment| !segment.is_empty()) {
                    _ = path_segments.push(segment);
                }
            }

            for segment in segments {
                _ = path_segments.push(segment);
            }
        }

        Ok(url)
    }
}
