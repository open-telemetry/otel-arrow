// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::args::ConnectionArgs;
use crate::error::CliError;
use otap_df_admin_api::config::tls::{TlsClientConfig, TlsConfig};
use otap_df_admin_api::{AdminEndpoint, HttpAdminClientSettings};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

pub const DEFAULT_LOCAL_URL: &str = "http://127.0.0.1:8085";

#[derive(Debug, Clone)]
pub struct ResolvedConnection {
    pub settings: HttpAdminClientSettings,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub url: Option<String>,
    #[serde(default, with = "humantime_serde")]
    pub connect_timeout: Option<Duration>,
    #[serde(default, with = "humantime_serde")]
    pub request_timeout: Option<Duration>,
    #[serde(default)]
    pub disable_request_timeout: bool,
    pub tcp_nodelay: Option<bool>,
    #[serde(default, with = "humantime_serde")]
    pub tcp_keepalive: Option<Duration>,
    #[serde(default, with = "humantime_serde")]
    pub tcp_keepalive_interval: Option<Duration>,
    pub ca_file: Option<PathBuf>,
    pub client_cert_file: Option<PathBuf>,
    pub client_key_file: Option<PathBuf>,
    pub include_system_ca_certs: Option<bool>,
    pub insecure_skip_verify: Option<bool>,
}

pub fn resolve_connection(args: &ConnectionArgs) -> Result<ResolvedConnection, CliError> {
    let profile = load_profile(args.profile_file.as_ref())?;

    let url = args
        .url
        .clone()
        .or(profile.url.clone())
        .unwrap_or_else(|| DEFAULT_LOCAL_URL.to_string());

    let endpoint = AdminEndpoint::from_url(&url)
        .map_err(|err| CliError::config(format!("invalid admin endpoint '{url}': {err}")))?;
    let mut settings = HttpAdminClientSettings::new(endpoint);

    if let Some(connect_timeout) = args.connect_timeout.or(profile.connect_timeout) {
        settings = settings.with_connect_timeout(connect_timeout);
    }

    if args.no_request_timeout || profile.disable_request_timeout {
        settings = settings.without_timeout();
    } else if let Some(request_timeout) = args.request_timeout.or(profile.request_timeout) {
        settings = settings.with_timeout(request_timeout);
    }

    if let Some(tcp_nodelay) = args.tcp_nodelay.or(profile.tcp_nodelay) {
        settings = settings.with_tcp_nodelay(tcp_nodelay);
    }

    if let Some(tcp_keepalive) = args.tcp_keepalive.or(profile.tcp_keepalive) {
        settings = settings.with_tcp_keepalive(Some(tcp_keepalive));
    }

    if let Some(tcp_keepalive_interval) = args
        .tcp_keepalive_interval
        .or(profile.tcp_keepalive_interval)
    {
        settings = settings.with_tcp_keepalive_interval(Some(tcp_keepalive_interval));
    }

    let ca_file = args.ca_file.clone().or(profile.ca_file.clone());
    let client_cert_file = args
        .client_cert_file
        .clone()
        .or(profile.client_cert_file.clone());
    let client_key_file = args
        .client_key_file
        .clone()
        .or(profile.client_key_file.clone());
    let include_system_ca_certs = args
        .include_system_ca_certs
        .or(profile.include_system_ca_certs);
    let insecure_skip_verify = args.insecure_skip_verify.or(profile.insecure_skip_verify);

    if ca_file.is_some()
        || client_cert_file.is_some()
        || client_key_file.is_some()
        || include_system_ca_certs.is_some()
        || insecure_skip_verify.is_some()
    {
        settings = settings.with_tls(TlsClientConfig {
            config: TlsConfig {
                cert_file: client_cert_file,
                key_file: client_key_file,
                ..TlsConfig::default()
            },
            ca_file,
            include_system_ca_certs_pool: include_system_ca_certs,
            insecure_skip_verify,
            ..TlsClientConfig::default()
        });
    }

    Ok(ResolvedConnection { settings })
}

fn load_profile(profile_file: Option<&PathBuf>) -> Result<ConnectionProfile, CliError> {
    let Some(path) = profile_file else {
        return Ok(ConnectionProfile::default());
    };

    let content = fs::read_to_string(path).map_err(|err| {
        CliError::config(format!(
            "failed to read profile file '{}': {err}",
            path.display()
        ))
    })?;
    serde_yaml::from_str(&content).map_err(|err| {
        CliError::config(format!(
            "failed to parse profile file '{}': {err}",
            path.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cli_values_override_profile_values() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("profile.yaml");
        fs::write(
            &path,
            "url: https://admin.example.com\nrequest_timeout: 10s\nconnect_timeout: 1s\n",
        )
        .expect("write profile");

        let args = ConnectionArgs {
            profile_file: Some(path),
            url: Some("http://127.0.0.1:8085/engine-a".to_string()),
            request_timeout: Some(Duration::from_secs(3)),
            ..ConnectionArgs::default()
        };

        let resolved = resolve_connection(&args).expect("resolve connection");
        assert_eq!(resolved.settings.connect_timeout, Duration::from_secs(1));
        assert_eq!(resolved.settings.timeout, Some(Duration::from_secs(3)));
        assert_eq!(
            resolved
                .settings
                .endpoint
                .base_path
                .as_deref()
                .expect("base path"),
            "/engine-a"
        );
    }
}
