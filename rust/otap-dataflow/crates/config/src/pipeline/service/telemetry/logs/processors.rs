// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Log processors level configurations.

pub mod batch;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::service::telemetry::logs::processors::batch::LogBatchProcessorExporterConfig;

/// Supported log processors
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub enum LogProcessorConfig {
    /// Batch log processor
    Batch(BatchLogProcessorConfig),
}

impl<'de> Deserialize<'de> for LogProcessorConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};

        struct LogProcessorConfigVisitor;

        impl<'de> Visitor<'de> for LogProcessorConfigVisitor {
            type Value = LogProcessorConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a map with a processor type key (e.g., 'batch')")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let key: String = map
                    .next_key()?
                    .ok_or_else(|| de::Error::custom("expected a processor type key"))?;

                match key.as_str() {
                    "batch" => {
                        let config: BatchLogProcessorConfig = map.next_value()?;
                        Ok(LogProcessorConfig::Batch(config))
                    }
                    _ => Err(de::Error::unknown_variant(&key, &["batch"])),
                }
            }
        }

        deserializer.deserialize_map(LogProcessorConfigVisitor)
    }
}

/// Configuration for the batch log processor.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchLogProcessorConfig {
    /// The log exporter to use.
    pub exporter: LogBatchProcessorExporterConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_processor_config_console_deserialize() {
        let yaml_str = r#"
            batch:
              exporter:
                console:
            "#;
        let config: LogProcessorConfig = serde_yaml::from_str(yaml_str).unwrap();

        match config {
            LogProcessorConfig::Batch(batch_config) => match batch_config.exporter {
                LogBatchProcessorExporterConfig::Console => {}
                _ => {
                    panic!("Expected Console exporter.");
                }
            },
        }
    }

    #[test]
    fn test_log_processor_config_otlp_deserialize() {
        let yaml_str = r#"
            batch:
              exporter:
                otlp:
                  endpoint: "http://localhost:4317"
            "#;
        let config: LogProcessorConfig = serde_yaml::from_str(yaml_str).unwrap();
        match config {
            LogProcessorConfig::Batch(batch_config) => match batch_config.exporter {
                LogBatchProcessorExporterConfig::Otlp(_) => {}
                _ => {
                    panic!("Expected OTLP exporter.");
                }
            },
        }
    }

    #[test]
    fn test_log_processor_config_internal_deserialize() {
        let yaml_str = r#"
            batch:
              exporter:
                internal:
            "#;
        let config: LogProcessorConfig = serde_yaml::from_str(yaml_str).unwrap();
        match config {
            LogProcessorConfig::Batch(batch_config) => match batch_config.exporter {
                LogBatchProcessorExporterConfig::Internal => {}
                _ => {
                    panic!("Expected Internal exporter.");
                }
            },
        }
    }
}
