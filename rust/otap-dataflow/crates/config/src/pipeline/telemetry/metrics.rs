// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics level configurations.

pub mod readers;
pub mod views;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::telemetry::metrics::views::ViewConfig;

/// Backend used to publish internal metrics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MetricsProvider {
    /// Publish metrics through the OpenTelemetry client SDK.
    #[default]
    OpenTelemetry,
    /// Publish metrics through the engine's internal telemetry pipeline.
    Its,
    /// Do not publish internal metrics.
    None,
}

impl MetricsProvider {
    const fn as_str(self) -> &'static str {
        match self {
            Self::OpenTelemetry => "opentelemetry",
            Self::Its => "its",
            Self::None => "none",
        }
    }
}

/// Internal metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq)]
pub struct MetricsConfig {
    /// Backend used to publish internal metrics.
    #[serde(default)]
    pub provider: MetricsProvider,

    /// The list of metrics readers to configure.
    ///
    /// Readers are supported only by the `opentelemetry` provider. With the
    /// `its` provider, exporters are configured as nodes in the engine's
    /// observability pipeline instead.
    #[serde(default)]
    pub readers: Vec<readers::MetricsReaderConfig>,

    /// OpenTelemetry SDK metrics views.
    #[serde(default)]
    pub views: Vec<ViewConfig>,
}

impl MetricsConfig {
    /// Returns `true` when metrics are published through ITS.
    #[must_use]
    pub const fn uses_its_provider(&self) -> bool {
        matches!(self.provider, MetricsProvider::Its)
    }

    /// Returns `true` when metrics are published through the OpenTelemetry SDK.
    #[must_use]
    pub const fn uses_opentelemetry_provider(&self) -> bool {
        matches!(self.provider, MetricsProvider::OpenTelemetry)
    }

    /// Returns `true` if there are any metric readers configured.
    #[must_use]
    pub const fn has_readers(&self) -> bool {
        !self.readers.is_empty()
    }

    /// Validates every configured metric reader's exporter configuration.
    pub fn validate(&self) -> Result<(), crate::error::Error> {
        let mut errors = Vec::new();

        if !self.uses_opentelemetry_provider() && !self.readers.is_empty() {
            errors.push(crate::error::Error::InvalidUserConfig {
                error: format!(
                    "engine.telemetry.metrics.readers requires provider 'opentelemetry', got '{}'",
                    self.provider.as_str()
                ),
            });
        }
        if !self.uses_opentelemetry_provider() && !self.views.is_empty() {
            errors.push(crate::error::Error::InvalidUserConfig {
                error: format!(
                    "engine.telemetry.metrics.views requires provider 'opentelemetry', got '{}'",
                    self.provider.as_str()
                ),
            });
        }
        for reader in &self.readers {
            if let Err(e) = reader.validate() {
                errors.push(e);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(crate::error::Error::InvalidConfiguration { errors })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_deserialize() {
        let yaml_str = r#"
            readers:
              - periodic:
                  exporter:
                    type: console
                  interval: "10s"
            "#;

        let config: MetricsConfig = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(config.provider, MetricsProvider::OpenTelemetry);
        assert_eq!(config.readers.len(), 1);

        if let readers::MetricsReaderConfig::Periodic(periodic_config) = &config.readers[0] {
            if readers::periodic::MetricsPeriodicExporterType::Console
                != periodic_config.exporter.exporter_type
            {
                panic!("Expected console exporter");
            }
            assert_eq!(periodic_config.interval.as_secs(), 10);
        } else {
            panic!("Expected periodic reader");
        }
    }

    #[test]
    fn test_metrics_provider_deserialize_and_validate() {
        for (name, expected) in [
            ("opentelemetry", MetricsProvider::OpenTelemetry),
            ("its", MetricsProvider::Its),
            ("none", MetricsProvider::None),
        ] {
            let config: MetricsConfig = serde_yaml::from_str(&format!("provider: {name}"))
                .expect("provider should deserialize");
            assert_eq!(config.provider, expected);
            config.validate().expect("provider-only config is valid");
        }
    }

    #[test]
    fn test_non_sdk_provider_rejects_sdk_configuration() {
        let readers: MetricsConfig = serde_yaml::from_str(
            r#"
provider: its
readers:
  - periodic:
      exporter:
        type: console
"#,
        )
        .expect("config should deserialize before validation");
        assert!(
            readers
                .validate()
                .expect_err("ITS readers must be rejected")
                .to_string()
                .contains("readers requires provider 'opentelemetry'")
        );

        let views: MetricsConfig = serde_yaml::from_str(
            r#"
provider: none
views:
  - selector: {}
    stream: {}
"#,
        )
        .expect("config should deserialize before validation");
        assert!(
            views
                .validate()
                .expect_err("disabled metrics views must be rejected")
                .to_string()
                .contains("views requires provider 'opentelemetry'")
        );
    }
}
