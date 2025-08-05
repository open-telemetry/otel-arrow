// SPDX-License-Identifier: Apache-2.0

//! # Basic Usage Example for SignalTypeRouter
//!
//! This example demonstrates how to create and configure a SignalTypeRouter processor.
//! It shows both direct processor instantiation and how the router would be configured
//! within a complete OTAP pipeline using the proper YAML configuration format.

use log::{debug, error, info};
use otap_df_otlp::grpc::OTLPData;
use otap_df_otlp::proto::opentelemetry::collector::{
    logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
    trace::v1::ExportTraceServiceRequest,
};
use otap_df_signal_type_router::{SignalTypeRouter, SignalTypeRouterConfig};
use serde_json::json;

fn main() {
    // Initialize logging
    env_logger::init();

    info!("SignalTypeRouter Example");
    info!("========================");

    // Example 1: Create router with default configuration
    info!("1. Creating router with default configuration...");
    let default_config = SignalTypeRouterConfig::default();
    let _router = SignalTypeRouter::new(default_config);
    info!("   Router created successfully with drop_unknown_signals = false");

    // Example 2: Create router with custom configuration
    info!("2. Creating router with custom configuration...");
    let custom_config = SignalTypeRouterConfig {
        drop_unknown_signals: true,
    };
    let _router = SignalTypeRouter::new(custom_config);
    info!("   Router created successfully with drop_unknown_signals = true");

    // Example 3: Configuration serialization/deserialization
    info!("3. Testing configuration serialization...");
    let config_json = json!({
        "drop_unknown_signals": false
    });

    let deserialized_config: SignalTypeRouterConfig =
        serde_json::from_value(config_json).expect("Failed to deserialize config");

    info!(
        "   Configuration deserialized: drop_unknown_signals = {}",
        deserialized_config.drop_unknown_signals
    );

    // Example 4: Signal type detection examples
    info!("4. Signal type detection examples...");

    // Create sample signals
    let traces_signal = OTLPData::Traces(ExportTraceServiceRequest {
        resource_spans: vec![],
    });

    let metrics_signal = OTLPData::Metrics(ExportMetricsServiceRequest {
        resource_metrics: vec![],
    });

    let logs_signal = OTLPData::Logs(ExportLogsServiceRequest {
        resource_logs: vec![],
    });

    debug!("   Created sample signals:");
    debug!(
        "   - Traces signal: {:?}",
        matches!(traces_signal, OTLPData::Traces(_))
    );
    debug!(
        "   - Metrics signal: {:?}",
        matches!(metrics_signal, OTLPData::Metrics(_))
    );
    debug!(
        "   - Logs signal: {:?}",
        matches!(logs_signal, OTLPData::Logs(_))
    );

    // Example 5: Factory usage
    info!("5. Using the processor factory...");
    let factory_config = json!({
        "drop_unknown_signals": false
    });

    let processor_config = otap_df_engine::config::ProcessorConfig::new("signal_router_example");

    match otap_df_signal_type_router::create_signal_type_router(&factory_config, &processor_config)
    {
        Ok(_wrapper) => info!("   Processor wrapper created successfully!"),
        Err(e) => error!("   Error creating processor wrapper: {e}"),
    }

    // Example 6: Explicit configuration principles
    info!("6. Explicit configuration principles...");
    show_explicit_configuration_principles();

    // Example 7: Complete pipeline configuration example
    info!("7. Complete pipeline configuration example...");
    show_pipeline_configuration_example();

    info!("Example completed successfully!");
    info!("Next steps:");
    info!("- Use the shown YAML configuration in your pipeline files");
    info!("- Configure output ports for different signal types");
    info!("- Implement advanced routing strategies");
}

/// Displays a complete pipeline configuration example showing how to use SignalTypeRouter
/// in a real OTAP pipeline with proper YAML configuration format.
fn show_pipeline_configuration_example() {
    let pipeline_yaml = r#"
# Example OTAP Pipeline Configuration with SignalTypeRouter
# This shows the complete pipeline structure format used in this codebase

type: otap
description: "Multi-signal processing pipeline with signal type routing"
settings:
  default_control_channel_size: 100
  default_pdata_channel_size: 100

nodes:
  # OTLP receiver that accepts all signal types
  otlp_receiver:
    kind: receiver
    plugin_urn: "urn:otap:receiver:otlp"
    config:
      endpoint: "0.0.0.0:4317"
      protocols:
        grpc:
          enabled: true
        http:
          enabled: true

  # SignalTypeRouter - routes signals based on their type
  signal_router:
    kind: processor
    plugin_urn: "urn:otap:processor:signal_type_router"
    out_ports:
      traces_out:
        dispatch_strategy: round_robin
        destinations: [traces_batch_processor, traces_resource_processor]
      metrics_out:
        dispatch_strategy: broadcast
        destinations: [metrics_processor, metrics_analytics]
      logs_out:
        dispatch_strategy: random
        destinations: [logs_processor_1, logs_processor_2]
    config:
      drop_unknown_signals: false

  # Traces processing chain
  traces_batch_processor:
    kind: processor
    plugin_urn: "urn:otap:processor:batch"
    out_ports:
      default:
        dispatch_strategy: round_robin
        destinations: [traces_exporter]
    config:
      timeout: "1s"
      send_batch_size: 512

  traces_resource_processor:
    kind: processor
    plugin_urn: "urn:otap:processor:resource"
    out_ports:
      default:
        dispatch_strategy: round_robin
        destinations: [traces_exporter]
    config:
      attributes:
        service.name: "my-service"

  # Metrics processing
  metrics_processor:
    kind: processor
    plugin_urn: "urn:otap:processor:batch"
    out_ports:
      default:
        dispatch_strategy: round_robin
        destinations: [metrics_exporter]
    config:
      timeout: "5s"
      send_batch_size: 1024

  metrics_analytics:
    kind: processor
    plugin_urn: "urn:otap:processor:analytics"
    out_ports:
      default:
        dispatch_strategy: round_robin
        destinations: [analytics_exporter]

  # Logs processing
  logs_processor_1:
    kind: processor
    plugin_urn: "urn:otap:processor:batch"
    out_ports:
      default:
        dispatch_strategy: round_robin
        destinations: [logs_exporter]
    config:
      timeout: "2s"
      send_batch_size: 256

  logs_processor_2:
    kind: processor
    plugin_urn: "urn:otap:processor:filter"
    out_ports:
      default:
        dispatch_strategy: round_robin
        destinations: [logs_exporter]
    config:
      include:
        severity_text: ["ERROR", "WARN"]

  # Exporters
  traces_exporter:
    kind: exporter
    plugin_urn: "urn:otap:exporter:otlp"
    config:
      endpoint: "https://traces-backend.example.com:4317"
      compression: gzip

  metrics_exporter:
    kind: exporter
    plugin_urn: "urn:otap:exporter:prometheus"
    config:
      endpoint: "http://prometheus.example.com:9090/api/v1/write"

  analytics_exporter:
    kind: exporter
    plugin_urn: "urn:otap:exporter:analytics"
    config:
      endpoint: "https://analytics.example.com/metrics"

  logs_exporter:
    kind: exporter
    plugin_urn: "urn:otap:exporter:loki"
    config:
      endpoint: "https://loki.example.com/loki/api/v1/push"
"#;

    info!("   Complete pipeline YAML configuration:");
    for line in pipeline_yaml.lines() {
        info!("   {line}");
    }

    info!("");
    info!("   Key features of this configuration:");
    info!("   • SignalTypeRouter automatically routes signals by type");
    info!("   • Traces: Round-robin between batch and resource processors");
    info!("   • Metrics: Broadcast to both processing and analytics");
    info!("   • Logs: Random distribution between batch and filter processors");
    info!("   • Each signal type has dedicated processing chains");
    info!("   • Zero-copy routing - no data duplication during routing");
}

/// Explains the explicit configuration principles for SignalTypeRouter
fn show_explicit_configuration_principles() {
    info!("   The SignalTypeRouter follows an EXPLICIT CONFIGURATION approach:");
    info!("");
    info!("   1. NO IMPLICIT BEHAVIOR:");
    info!("      • Every routing decision must be explicitly specified");
    info!("      • No default routing or fallback behaviors");
    info!("      • All signal types require explicit output port configuration");
    info!("");
    info!("   2. COMPLETE NODE DEFINITIONS:");
    info!("      • All referenced destination nodes must be fully configured");
    info!("      • No partial or incomplete node specifications allowed");
    info!("      • Each node must have a complete 'config' section");
    info!("");
    info!("   3. PORT-SPECIFIC ROUTING:");
    info!("      • traces_out: Routes trace signals");
    info!("      • metrics_out: Routes metric signals");
    info!("      • logs_out: Routes log signals");
    info!("      • profiles_out: Routes profile signals");
    info!("");
    info!("   4. STRATEGY SPECIFICATION:");
    info!("      • broadcast: Send to ALL destinations (zero-copy)");
    info!("      • round_robin: Cycle through destinations for load balancing");
    info!("      • random: Randomly select destinations");
    info!("      • least_loaded: Route to least busy destination (future)");
    info!("");
    info!("   5. VALIDATION REQUIREMENTS:");
    info!("      • Configuration is validated before runtime");
    info!("      • All destination references must be resolvable");
    info!("      • Plugin URNs must map to available implementations");
    info!("      • Dispatch strategies must be supported by target nodes");
    info!("");
    info!("   Benefits of explicit configuration:");
    info!("   ✓ Predictable pipeline behavior");
    info!("   ✓ Clear routing intentions");
    info!("   ✓ Easier debugging and troubleshooting");
    info!("   ✓ Better validation and error detection");
    info!("   ✓ Self-documenting pipeline configurations");
}
