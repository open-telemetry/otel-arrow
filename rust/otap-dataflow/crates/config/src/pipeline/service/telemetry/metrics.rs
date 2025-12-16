// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics level configurations.

pub mod readers;
pub mod views;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::service::telemetry::metrics::views::ViewConfig;

/// OpenTelemetry Metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct MetricsConfig {
    /// The list of metrics readers to configure.
    #[serde(default)]
    pub readers: Vec<readers::MetricsReaderConfig>,

    /// The metrics views configuration.
    #[serde(default)]
    pub views: Vec<ViewConfig>,
}

impl MetricsConfig {
    /// Returns `true` if there are any metric readers configured.
    #[must_use]
    pub fn has_readers(&self) -> bool {
        !self.readers.is_empty()
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
                    console:
                  interval: "10s"
            "#;

        let config: MetricsConfig = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(config.readers.len(), 1);

        if let readers::MetricsReaderConfig::Periodic(periodic_config) = &config.readers[0] {
            if readers::periodic::MetricsPeriodicExporterConfig::Console != periodic_config.exporter
            {
                panic!("Expected console exporter");
            }
            assert_eq!(periodic_config.interval.as_secs(), 10);
        } else {
            panic!("Expected periodic reader");
        }
    }
}
