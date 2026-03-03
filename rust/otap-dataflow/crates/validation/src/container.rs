// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Docker container configuration for validation scenarios.
//!
//! [`ContainerConfig`] lets tests declare Docker containers that run alongside
//! the validation pipeline. The containers start before the pipeline group and
//! stop after it shuts down.
//!
//! The underlying `testcontainers` crate is fully abstracted — users only
//! interact with plain Rust types defined here.

use crate::error::ValidationError;
use testcontainers::core::IntoContainerPort;
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

/// Describes a Docker container to run alongside the validation scenario.
///
/// Use the builder methods to configure the image, exposed ports, and
/// environment variables.
///
/// # Example
///
/// ```ignore
/// let redis = ContainerConfig::new("redis", "7.2.4")
///     .expose_tcp(6379)
///     .env("REDIS_ARGS", "--save 60 1");
/// ```
pub struct ContainerConfig {
    /// Docker image name (e.g., `"redis"`, `"confluentinc/cp-kafka"`)
    pub(crate) image: String,
    /// Docker image tag (e.g., `"7.2.4"`, `"latest"`)
    pub(crate) tag: String,
    /// TCP ports to expose from the container
    pub(crate) exposed_tcp_ports: Vec<u16>,
    /// Environment variables to set on the container
    pub(crate) env_vars: Vec<(String, String)>,
    /// Optional entrypoint override
    pub(crate) entrypoint: Option<String>,
}

impl ContainerConfig {
    /// Create a new container configuration for the given Docker image and tag.
    #[must_use]
    pub fn new(image: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            tag: tag.into(),
            exposed_tcp_ports: Vec::new(),
            env_vars: Vec::new(),
            entrypoint: None,
        }
    }

    /// Expose a TCP port from the container.
    #[must_use]
    pub fn expose_tcp(mut self, port: u16) -> Self {
        self.exposed_tcp_ports.push(port);
        self
    }

    /// Set an environment variable on the container.
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push((key.into(), value.into()));
        self
    }

    /// Override the container's entrypoint.
    #[must_use]
    pub fn entrypoint(mut self, entrypoint: impl Into<String>) -> Self {
        self.entrypoint = Some(entrypoint.into());
        self
    }

    /// Start the container described by this configuration.
    ///
    /// Builds the Docker image, applies exposed ports, environment variables,
    /// and entrypoint overrides, then starts the container. Returns a handle
    /// to the running container.
    pub(crate) async fn start(self) -> Result<ContainerAsync<GenericImage>, ValidationError> {
        let mut image = GenericImage::new(&self.image, &self.tag);

        for port in &self.exposed_tcp_ports {
            image = image.with_exposed_port(port.tcp());
        }

        if let Some(ep) = &self.entrypoint {
            image = image.with_entrypoint(ep);
        }

        image = image.with_wait_for(WaitFor::Nothing);

        // Apply environment variables via ImageExt (consumes into ContainerRequest)
        let mut request = testcontainers::core::ContainerRequest::from(image);
        for (key, value) in &self.env_vars {
            request = request.with_env_var(key, value);
        }

        request.start().await.map_err(|e| {
            ValidationError::Container(format!(
                "failed to start container {}:{}: {e}",
                self.image, self.tag
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_config_builder_defaults() {
        let config = ContainerConfig::new("redis", "7.2.4");
        assert_eq!(config.image, "redis");
        assert_eq!(config.tag, "7.2.4");
        assert!(config.exposed_tcp_ports.is_empty());
        assert!(config.env_vars.is_empty());
        assert!(config.entrypoint.is_none());
    }

    #[test]
    fn container_config_builder_chaining() {
        let config = ContainerConfig::new("kafka", "7.5.0")
            .expose_tcp(9092)
            .expose_tcp(9093)
            .env("FOO", "bar")
            .env("BAZ", "qux")
            .entrypoint("/bin/sh");

        assert_eq!(config.exposed_tcp_ports, vec![9092, 9093]);
        assert_eq!(config.env_vars.len(), 2);
        assert_eq!(config.env_vars[0], ("FOO".into(), "bar".into()));
        assert_eq!(config.entrypoint, Some("/bin/sh".into()));
    }

    #[test]
    fn expose_tcp_single_port() {
        let config = ContainerConfig::new("img", "tag").expose_tcp(8080);
        assert_eq!(config.exposed_tcp_ports, vec![8080]);
    }

    #[test]
    fn env_single_var() {
        let config = ContainerConfig::new("img", "tag").env("KEY", "VALUE");
        assert_eq!(config.env_vars.len(), 1);
        assert_eq!(config.env_vars[0], ("KEY".into(), "VALUE".into()));
    }

    #[test]
    fn entrypoint_last_call_wins() {
        let config = ContainerConfig::new("img", "tag")
            .entrypoint("/bin/sh")
            .entrypoint("/bin/bash");
        assert_eq!(config.entrypoint, Some("/bin/bash".into()));
    }

    #[tokio::test]
    #[ignore] // Requires Docker daemon running on the host.
    async fn start_and_stop_container() {
        let config = ContainerConfig::new("alpine", "3.20").expose_tcp(8080);

        let container = config.start().await.expect("container should start");

        // Verify the container is running by resolving the mapped host port.
        let host_port = container
            .get_host_port_ipv4(8080.tcp())
            .await
            .expect("should resolve host port");
        assert_ne!(host_port, 0);

        // Stop the container via testcontainers stop().
        container.stop().await.expect("container should stop");
    }
}
