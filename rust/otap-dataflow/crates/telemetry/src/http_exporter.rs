// SPDX-License-Identifier: Apache-2.0

//! HTTP server for exposing telemetry endpoints.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;

use crate::config::HttpServerConfig;
use crate::registry::MetricsRegistryHandle;
use crate::semconv::SemConvRegistry;
use crate::attributes::AttributeValue;

/// Shared state for the HTTP server.
#[derive(Clone)]
struct AppState {
    metrics_registry: MetricsRegistryHandle,
}

/// JSON representation of aggregated metrics.
#[derive(Serialize)]
struct MetricsResponse {
    /// Timestamp when the metrics were collected.
    timestamp: String,
    /// List of metric sets with their values.
    metric_sets: Vec<MetricSetJson>,
}

/// JSON representation of a metric set.
#[derive(Serialize)]
struct MetricSetJson {
    /// Name of the metric set.
    name: String,
    /// Description of the metric set.
    description: String,
    /// Attributes associated with this metric set.
    attributes: HashMap<String, Value>,
    /// Individual metrics within this set.
    metrics: Vec<MetricJson>,
}

/// JSON representation of an individual metric.
#[derive(Serialize)]
struct MetricJson {
    /// Name of the metric.
    name: String,
    /// Description of the metric.
    description: String,
    /// Unit of measurement.
    unit: String,
    /// Current value.
    value: u64,
    /// Type of instrument (counter, gauge, histogram, etc.).
    instrument_type: String,
}

/// Starts the HTTP server with the given configuration and metrics registry.
pub async fn start_server(
    config: HttpServerConfig,
    metrics_registry: MetricsRegistryHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app_state = AppState { metrics_registry };

    let app = Router::new()
        .route("/semconv", get(get_semconv))
        .route("/metrics", get(get_metrics))
        .route("/health", get(health_check))
        .layer(ServiceBuilder::new())
        .with_state(app_state);

    let addr: SocketAddr = config.bind_address.parse()?;
    let listener = TcpListener::bind(&addr).await?;

    println!("Telemetry HTTP server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// Handler for the /semconv endpoint - returns semantic convention registry.
async fn get_semconv(
    State(state): State<AppState>,
) -> Result<Json<SemConvRegistry>, StatusCode> {
    let semconv = state.metrics_registry.generate_semconv_registry();
    Ok(Json(semconv))
}

/// Handler for the /metrics endpoint - returns aggregated metrics in JSON format.
async fn get_metrics(
    State(state): State<AppState>,
) -> Result<Json<MetricsResponse>, StatusCode> {
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Create a snapshot of current metrics without resetting them
    let metrics_snapshot = collect_metrics_snapshot(&state.metrics_registry);

    let response = MetricsResponse {
        timestamp,
        metric_sets: metrics_snapshot,
    };

    Ok(Json(response))
}

/// Handler for the /health endpoint - simple health check.
async fn health_check() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "telemetry-server"
    }))
}

/// Collects a snapshot of current metrics without resetting them.
fn collect_metrics_snapshot(registry: &MetricsRegistryHandle) -> Vec<MetricSetJson> {
    let mut metric_sets = Vec::new();

    registry.visit_current_metrics(|descriptor, attributes, metrics_iter| {
        let mut metrics = Vec::new();

        for (field, value) in metrics_iter {
            metrics.push(MetricJson {
                name: field.name.to_string(),
                description: field.brief.to_string(),
                unit: field.unit.to_string(),
                value,
                instrument_type: format!("{:?}", field.instrument),
            });
        }

        if !metrics.is_empty() {
            // Convert attributes to HashMap using the iterator
            let mut attrs_map = HashMap::new();
            for (key, value) in attributes.iter_attributes() {
                let json_value = attribute_value_to_json(value);
                let _ = attrs_map.insert(key.to_string(), json_value);
            }

            metric_sets.push(MetricSetJson {
                name: descriptor.name.to_string(),
                description: "".to_string(), // MetricsDescriptor doesn't have description field
                attributes: attrs_map,
                metrics,
            });
        }
    });

    metric_sets
}

/// Converts an AttributeValue to a JSON Value.
fn attribute_value_to_json(attr_value: &AttributeValue) -> Value {
    match attr_value {
        AttributeValue::String(s) => Value::String(s.clone()),
        AttributeValue::Int(i) => Value::Number((*i).into()),
        AttributeValue::UInt(u) => Value::Number((*u).into()),
        AttributeValue::Double(f) => {
            match serde_json::Number::from_f64(*f) {
                Some(num) => Value::Number(num),
                None => Value::Null, // Handle NaN or infinite values
            }
        }
        AttributeValue::Boolean(b) => Value::Bool(*b),
    }
}
