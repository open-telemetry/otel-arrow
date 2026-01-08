// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Configuration for the KQL processor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordsetKqlProcessorConfig {
    /// KQL query to apply to the data
    pub query: String,
    /// Optional bridge options for KQL processing
    #[serde(default)]
    pub bridge_options: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use crate::experimental::recordset_kql_processor::processor::RecordsetKqlProcessor;

    use super::*;
    use otap_df_engine::context::{ControllerContext, PipelineContext};
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn create_test_pipeline_context() -> PipelineContext {
        let metrics_registry = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry);
        controller_ctx.pipeline_context_with("test_grp".into(), "test_pipeline".into(), 0, 0)
    }

    #[test]
    fn test_config_parsing() {
        let config_json = json!({
            "query": "source | where true"
        });

        let config: RecordsetKqlProcessorConfig = serde_json::from_value(config_json).unwrap();
        assert_eq!(config.query, "source | where true");
        assert_eq!(config.bridge_options, None);
    }

    #[test]
    fn test_config_parsing_with_bridge_options() {
        let config_json = json!({
            "query": "source | where true",
            "bridge_options": {
                "attributes_schema": {
                    "double_key": "Double",
                    "string_key": "String"
                }
            }
        });

        let config: RecordsetKqlProcessorConfig = serde_json::from_value(config_json).unwrap();
        assert_eq!(config.query, "source | where true");
        assert!(config.bridge_options.is_some());

        // Verify bridge options can be parsed
        let bridge_options = RecordsetKqlProcessor::parse_bridge_options(&config.bridge_options);
        assert!(bridge_options.is_ok());
        assert!(bridge_options.unwrap().is_some());
    }

    #[test]
    fn test_processor_creation() {
        let pipeline_ctx = create_test_pipeline_context();
        let config = RecordsetKqlProcessorConfig {
            query: "source | where true".to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_processor_creation_with_bridge_options() {
        let pipeline_ctx = create_test_pipeline_context();
        let bridge_options_json = json!({
            "attributes_schema": {
                "custom_field": "String",
                "metric_value": "Double"
            }
        });

        let config = RecordsetKqlProcessorConfig {
            query: "source | where true".to_string(),
            bridge_options: Some(bridge_options_json),
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_query() {
        let pipeline_ctx = create_test_pipeline_context();
        let config = RecordsetKqlProcessorConfig {
            query: "invalid kql syntax <<<".to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_err());
    }

    #[test]
    fn test_conditional_extend_vendor_info() {
        // Test demonstrating vendor-specific enrichment based on EventName or Body matching
        let pipeline_ctx = create_test_pipeline_context();

        // Query that matches on EventName or Body and extends with vendor info
        let query = r#"
            source
            | extend vendor = case(
                EventName == "payment.completed" or Body has "payment", "payment-processor",
                EventName == "user.login" or Body has "login", "auth-service",
                EventName == "metric.emitted" or Body has "metric", "observability",
                "unknown"
            )
            | extend priority = case(
                EventName has "error" or SeverityNumber >= 17, "high",
                EventName has "warning" or SeverityNumber >= 13, "medium",
                "low"
            )
            | extend processed_at = now()
        "#;

        let config = RecordsetKqlProcessorConfig {
            query: query.to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok(), "Query should parse successfully");
    }

    #[test]
    fn test_multiple_conditions_with_or() {
        let pipeline_ctx = create_test_pipeline_context();

        // Query demonstrating OR conditions and statement-by-statement enrichment
        let query = r#"
            source
            | where EventName == "critical.alert" or Body has "CRITICAL" or SeverityNumber >= 21
            | extend alert_type = case(
                Body has "OutOfMemory" or EventName has "oom", "memory",
                Body has "DiskFull" or EventName has "disk", "storage",
                Body has "NetworkTimeout" or EventName has "timeout", "network",
                "generic"
            )
            | extend team = case(
                alert_type == "memory", "infrastructure",
                alert_type == "storage", "storage-ops",
                alert_type == "network", "network-ops",
                "on-call"
            )
            | extend escalation_minutes = case(
                alert_type == "memory", 5,
                alert_type == "storage", 15,
                30
            )
        "#;

        let config = RecordsetKqlProcessorConfig {
            query: query.to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok(), "Complex OR and case query should parse");
    }

    #[test]
    fn test_pattern_matching_extend() {
        let pipeline_ctx = create_test_pipeline_context();

        // Query using pattern matching with 'has' operator
        let query = r#"
            source
            | extend service_type = case(
                EventName has "http" or Body has "HTTP", "web-service",
                EventName has "grpc" or Body has "gRPC", "rpc-service",
                EventName has "kafka" or Body has "Kafka", "messaging",
                EventName has "postgres" or Body has "PostgreSQL", "database",
                "other"
            )
            | extend vendor_category = case(
                service_type == "web-service", "frontend",
                service_type == "rpc-service", "backend",
                service_type == "messaging", "integration",
                service_type == "database", "data-layer",
                "uncategorized"
            )
            | project-keep EventName, Body, service_type, vendor_category, SeverityText
        "#;

        let config = RecordsetKqlProcessorConfig {
            query: query.to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(
            processor.is_ok(),
            "Pattern matching query should parse successfully"
        );
    }

    #[test]
    fn test_body_modification_with_strcat() {
        let pipeline_ctx = create_test_pipeline_context();

        // Query that appends supplemental information to the Body
        let query = r#"
            source
            | extend Body = case(
                EventName == "payment.failed" or Body has "payment failed",
                strcat(Body, "\n\nTroubleshooting: https://docs.example.com/payment-errors\nSupport: https://support.example.com/ticket"),
                EventName == "auth.error" or Body has "authentication",
                strcat(Body, "\n\nAuth Guide: https://docs.example.com/auth\nReset Password: https://portal.example.com/reset"),
                EventName has "error" or SeverityNumber >= 17,
                strcat(Body, "\n\nError Guide: https://docs.example.com/errors"),
                Body
            )
        "#;

        let config = RecordsetKqlProcessorConfig {
            query: query.to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(
            processor.is_ok(),
            "Body modification with strcat should parse successfully"
        );
    }

    #[test]
    fn test_body_enrichment_with_vendor_links() {
        let pipeline_ctx = create_test_pipeline_context();

        // Query that adds vendor-specific links and guidelines based on content
        let query = r#"
            source
            | extend vendor_type = case(
                Body has "AWS" or Body has "Amazon", "aws",
                Body has "Azure" or Body has "Microsoft", "azure",
                Body has "GCP" or Body has "Google", "gcp",
                "other"
            )
            | extend Body = case(
                vendor_type == "aws",
                strcat(Body, "\n\n[AWS Guidelines]\nDocs: https://docs.aws.amazon.com\nSupport: https://console.aws.amazon.com/support\nStatus: https://health.aws.amazon.com"),
                vendor_type == "azure",
                strcat(Body, "\n\n[Azure Guidelines]\nDocs: https://learn.microsoft.com/azure\nSupport: https://portal.azure.com/#blade/Microsoft_Azure_Support\nStatus: https://status.azure.com"),
                vendor_type == "gcp",
                strcat(Body, "\n\n[GCP Guidelines]\nDocs: https://cloud.google.com/docs\nSupport: https://console.cloud.google.com/support\nStatus: https://status.cloud.google.com"),
                Body
            )
        "#;

        let config = RecordsetKqlProcessorConfig {
            query: query.to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(
            processor.is_ok(),
            "Body enrichment with vendor links should parse successfully"
        );
    }
}
