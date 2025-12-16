// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Log processors batch level configurations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for the batch log processor exporter.
#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq)]
pub enum LogBatchProcessorExporterConfig {
    /// Console log exporter
    Console,
    // TODO: Add OTLP exporter.
}

impl<'de> Deserialize<'de> for LogBatchProcessorExporterConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};

        struct LogBatchProcessorExporterConfigVisitor;

        impl<'de> Visitor<'de> for LogBatchProcessorExporterConfigVisitor {
            type Value = LogBatchProcessorExporterConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a map with an exporter type key (e.g., 'console')")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let key: String = map
                    .next_key()?
                    .ok_or_else(|| de::Error::custom("expected an exporter type key"))?;

                match key.as_str() {
                    "console" => {
                        // Console has no configuration, just consume the value (empty or null)
                        let _: de::IgnoredAny = map.next_value()?;
                        Ok(LogBatchProcessorExporterConfig::Console)
                    }
                    _ => Err(de::Error::unknown_variant(&key, &["console"])),
                }
            }
        }

        deserializer.deserialize_map(LogBatchProcessorExporterConfigVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_batch_processor_exporter_config_deserialize() {
        let yaml_str = r#"
            console:
            "#;
        let config: LogBatchProcessorExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        match config {
            LogBatchProcessorExporterConfig::Console => {}
        }
    }
}
