// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Provider for Prometheus exporter configuration.

use std::net::SocketAddr;

use axum::{Router, extract::State, http::StatusCode, response::Response, routing::get};
use opentelemetry_sdk::metrics::MeterProviderBuilder;
use otap_df_config::pipeline::service::telemetry::metrics::readers::pull::PrometheusExporterConfig;
use prometheus::{Encoder, Registry, TextEncoder};

use crate::error::Error;

/// Provider for Prometheus exporter configuration.
pub struct PrometheusExporterProvider {}

impl PrometheusExporterProvider {
    /// Configure the Prometheus exporter for the given MeterProviderBuilder.
    pub fn configure_exporter(
        mut sdk_meter_builder: MeterProviderBuilder,
        prometheus_config: &PrometheusExporterConfig,
        runtime: Option<tokio::runtime::Runtime>,
    ) -> Result<(MeterProviderBuilder, Option<tokio::runtime::Runtime>), Error> {
        Self::validate_config(prometheus_config)?;

        let registry = Registry::new();

        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()
            .map_err(|e| Error::ConfigurationError(e.to_string()))?;

        sdk_meter_builder = sdk_meter_builder.with_reader(exporter);

        // If there is a tokio runtime already, use it. Otherwise, create a new one.
        let mut tokio_runtime = match runtime {
            Some(rt) => rt,
            None => tokio::runtime::Runtime::new()
                .map_err(|e| Error::ConfigurationError(e.to_string()))?,
        };

        tokio_runtime =
            Self::start_async_prometheus_server(registry, tokio_runtime, prometheus_config)?;

        Ok((sdk_meter_builder, Some(tokio_runtime)))
    }

    fn validate_config(prometheus_config: &PrometheusExporterConfig) -> Result<(), Error> {
        let endpoint = format!("{}:{}", prometheus_config.host, prometheus_config.port);
        let _parsed_endpoint = endpoint.parse::<SocketAddr>().map_err(|e| {
            Error::ConfigurationError(format!("Invalid Prometheus bind address: {}", e))
        })?;

        let path = &prometheus_config.path;
        if !path.starts_with('/') {
            return Err(Error::ConfigurationError(
                "Prometheus metrics path must start with '/'".to_string(),
            ));
        }

        Ok(())
    }

    fn start_async_prometheus_server(
        registry: Registry,
        runtime: tokio::runtime::Runtime,
        prometheus_config: &PrometheusExporterConfig,
    ) -> Result<tokio::runtime::Runtime, Error> {
        let endpoint = format!("{}:{}", prometheus_config.host, prometheus_config.port);
        let path = prometheus_config.path.clone();
        let _server_handle = runtime.spawn(async move {
            Self::start_prometheus_server(registry, &endpoint, &path)
                .await
                .map_err(|e| {
                    Error::ConfigurationError(format!("Failed to start Prometheus server: {}", e))
                })
        });
        Ok(runtime)
    }

    async fn start_prometheus_server(
        registry: Registry,
        endpoint: &str,
        path: &str,
    ) -> Result<(), Error> {
        let addr: SocketAddr = endpoint.parse().map_err(|e| {
            Error::ConfigurationError(format!("Invalid Prometheus bind address: {}", e))
        })?;

        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            Error::ConfigurationError(format!("Failed to bind Prometheus server: {}", e))
        })?;

        let app = Router::new()
            .merge(Self::routes(path))
            .with_state(registry.clone());

        axum::serve(listener, app)
            .await
            .map_err(|e| Error::ConfigurationError(format!("Prometheus server failed: {}", e)))?;

        Ok(())
    }

    /// Define the routes for the Prometheus exporter.
    fn routes(path: &str) -> Router<Registry> {
        Router::new().route(path, get(Self::get_metrics))
    }

    /// Handler for the metrics endpoint. Ex: `/metrics`.
    async fn get_metrics(State(registry): State<Registry>) -> Result<Response, StatusCode> {
        let encoder = TextEncoder::new();
        let metric_families = registry.gather();
        let mut buffer = Vec::new();
        encoder
            .encode(&metric_families, &mut buffer)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
            .body(buffer.into())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_exporter_provider_configure_exporter() {
        let prometheus_config = PrometheusExporterConfig {
            host: "0.0.0.0".to_string(),
            port: 9090,
            path: "/metrics".to_string(),
        };
        let sdk_meter_builder = MeterProviderBuilder::default();
        let result = PrometheusExporterProvider::configure_exporter(
            sdk_meter_builder,
            &prometheus_config,
            None,
        );
        match result {
            Ok((_, Some(tokio_runtime))) => {
                tokio_runtime.shutdown_background();
            }
            _ => panic!("Failed to configure Prometheus exporter"),
        }
    }

    #[test]
    fn test_prometheus_invalid_host_config() {
        let prometheus_config = PrometheusExporterConfig {
            host: "invalid_host".to_string(),
            port: 9090,
            path: "/metrics".to_string(),
        };
        let result = PrometheusExporterProvider::validate_config(&prometheus_config);
        match result {
            Err(Error::ConfigurationError(err)) => {
                assert!(err.contains("Invalid Prometheus bind address"));
            }
            _ => panic!("Expected ConfigurationError for invalid host"),
        }
    }

    #[test]
    fn test_prometheus_invalid_path_config() {
        let prometheus_config = PrometheusExporterConfig {
            host: "0.0.0.0".to_string(),
            port: 9090,
            path: "invalid/path/for/prometheus".to_string(),
        };
        let result = PrometheusExporterProvider::validate_config(&prometheus_config);
        match result {
            Err(Error::ConfigurationError(err)) => {
                assert!(err.contains("must start with '/'"));
            }
            _ => panic!("Expected ConfigurationError for invalid path"),
        }
    }

    #[tokio::test]
    async fn test_prometheus_get_metrics_handler() {
        let registry = Registry::new();
        registry
            .register(Box::new(
                prometheus::Counter::new("test_counter", "A test counter").unwrap(),
            ))
            .unwrap();
        let response = PrometheusExporterProvider::get_metrics(State(registry)).await;
        match response {
            Ok(resp) => {
                assert_eq!(resp.status(), StatusCode::OK);
                let body = resp.into_body();
                let bytes = axum::body::to_bytes(body, usize::MAX)
                    .await
                    .expect("Failed to read body bytes");
                let body_str = String::from_utf8(bytes.to_vec()).expect("Body is not valid UTF-8");
                assert!(body_str.contains("HELP test_counter A test counter"));
            }
            Err(_) => panic!("Failed to get metrics"),
        }
    }

    #[tokio::test]
    async fn test_routes() {
        use axum::body::Body;
        use axum::http::{Method, Request};
        use tower::ServiceExt; // For oneshot

        let registry = Registry::new();
        registry
            .register(Box::new(
                prometheus::Counter::new("route_test_counter", "A counter for route testing")
                    .unwrap(),
            ))
            .unwrap();

        let app = PrometheusExporterProvider::routes("/metrics").with_state(registry);

        // Test the /metrics endpoint
        let request = Request::builder()
            .method(Method::GET)
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .expect("Failed to read body");
        let body_str = String::from_utf8(bytes.to_vec()).expect("Body is not valid UTF-8");

        assert!(body_str.contains("route_test_counter"));
    }
}
