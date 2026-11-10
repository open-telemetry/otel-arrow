// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration types for the Prometheus scrape receiver.

use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const fn default_scrape_interval() -> Duration {
    Duration::from_secs(15)
}

const fn default_scrape_timeout() -> Duration {
    Duration::from_secs(10)
}

const fn default_max_body_bytes() -> usize {
    16 * 1024 * 1024
}

const fn default_max_data_points_per_batch() -> usize {
    expfmt::arrow::DEFAULT_MAX_DATA_POINTS_PER_BATCH
}

fn default_job_name() -> String {
    "prometheus".to_owned()
}

fn default_metrics_path() -> String {
    "/metrics".to_owned()
}

const fn default_true() -> bool {
    true
}

/// Configuration for the Prometheus scrape receiver (`urn:otel:receiver:prometheus`).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // TODO(bwplotka): Populate service.name, service.instance.id, and relabeled target
    // attributes via ArrowDecoder once https://github.com/prometheus/scrape.rs/issues/35 lands.
    /// Logical job name associated with scraped targets.
    #[serde(default = "default_job_name")]
    pub job_name: String,

    /// Interval between scrape ticks.
    #[serde(default = "default_scrape_interval", with = "humantime_serde")]
    pub scrape_interval: Duration,

    /// Per-target HTTP request timeout. Must not exceed `scrape_interval`.
    #[serde(default = "default_scrape_timeout", with = "humantime_serde")]
    pub scrape_timeout: Duration,

    /// Optional delay before the first scrape tick.
    #[serde(default, with = "humantime_serde")]
    pub initial_delay: Duration,

    /// Default HTTP path appended to bare `host:port` targets.
    #[serde(default = "default_metrics_path")]
    pub metrics_path: String,

    /// Whether explicit sample timestamps in the payload are preserved.
    #[serde(default = "default_true")]
    pub honor_timestamps: bool,

    /// Whether to advertise OpenMetrics 1.0 in the HTTP `Accept` header.
    #[serde(default = "default_true")]
    pub enable_open_metrics: bool,

    /// Maximum HTTP response body size in bytes per target scrape (default 16 MiB).
    #[serde(default = "default_max_body_bytes")]
    pub max_body_bytes: usize,

    /// Maximum number of data points per emitted OTAP batch (default `16384`; `0` means unchunked).
    #[serde(default = "default_max_data_points_per_batch")]
    pub max_data_points_per_batch: usize,

    /// Static scrape target endpoints (`host:port` or full `http(s)://...` URLs).
    #[serde(default)]
    pub targets: Vec<String>,

    /// Headless DNS service endpoints (`host:port`) resolved to pod/host IPs on each scrape tick.
    #[serde(default)]
    pub dns_sd_targets: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            job_name: default_job_name(),
            scrape_interval: default_scrape_interval(),
            scrape_timeout: default_scrape_timeout(),
            initial_delay: Duration::ZERO,
            metrics_path: default_metrics_path(),
            honor_timestamps: true,
            enable_open_metrics: true,
            max_body_bytes: default_max_body_bytes(),
            max_data_points_per_batch: default_max_data_points_per_batch(),
            targets: Vec::new(),
            dns_sd_targets: Vec::new(),
        }
    }
}

// TODO(bwplotka): Replace static URL normalization with scrape.rs discovery once
// https://github.com/prometheus/scrape.rs/issues/36 lands.
/// Normalizes a static target (`host:port` or full URL) into an HTTP(S) URL.
pub(crate) fn normalize_target_url(
    raw_target: &str,
    default_metrics_path: &str,
) -> Result<Url, otel_arrow_dfe_config::error::Error> {
    let trimmed = raw_target.trim();
    if trimmed.is_empty() {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "target endpoint must not be empty".to_owned(),
        });
    }

    let candidate = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_owned()
    } else {
        format!("http://{trimmed}{default_metrics_path}")
    };

    let parsed = Url::parse(&candidate).map_err(|err| {
        otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: format!("invalid scrape target `{trimmed}`: {err}"),
        }
    })?;

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: format!(
                "unsupported scheme `{}` in scrape target `{trimmed}`",
                parsed.scheme()
            ),
        });
    }
    if parsed.host_str().is_none() {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: format!("scrape target `{trimmed}` is missing a host"),
        });
    }

    Ok(parsed)
}

/// Validates a `prometheus_receiver` configuration.
pub(crate) fn validate_config(config: &Config) -> Result<(), otel_arrow_dfe_config::error::Error> {
    if config.job_name.trim().is_empty() {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "job_name must not be empty".to_owned(),
        });
    }
    if config.scrape_interval.is_zero() {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "scrape_interval must be greater than zero".to_owned(),
        });
    }
    if config.scrape_timeout.is_zero() {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "scrape_timeout must be greater than zero".to_owned(),
        });
    }
    if config.scrape_timeout > config.scrape_interval {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "scrape_timeout must not exceed scrape_interval".to_owned(),
        });
    }
    if config.max_body_bytes == 0 {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "max_body_bytes must be greater than zero".to_owned(),
        });
    }
    if !config.metrics_path.starts_with('/') {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "metrics_path must start with '/'".to_owned(),
        });
    }
    if config.targets.is_empty() && config.dns_sd_targets.is_empty() {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "at least one entry in targets or dns_sd_targets must be configured".to_owned(),
        });
    }
    for target in &config.targets {
        let _ = normalize_target_url(target, &config.metrics_path)?;
    }
    for dns_target in &config.dns_sd_targets {
        let trimmed = dns_target.trim();
        if trimmed.is_empty() || !trimmed.contains(':') {
            return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: format!("dns_sd_targets entry `{trimmed}` must be in `host:port` format"),
            });
        }
    }
    Ok(())
}
