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
use std::collections::HashMap;
use std::time::Duration;
use testcontainers::core::IntoContainerPort;
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

/// Describes a Docker container to run alongside the validation scenario.
///
/// Use the builder methods to configure the image and environment variables.
/// Port mappings are handled automatically by the framework when generators,
/// captures, or pipeline nodes declare connections via
/// [`ContainerConnection`](crate::traffic::ContainerConnection).
///
/// # Example
///
/// let redis = ContainerConfig::new("redis", "7.2.4")
///     .env("REDIS_ARGS", "--save 60 1");
///
#[derive(Debug)]
pub struct ContainerConfig {
    /// Docker image name (e.g., `"redis"`, `"confluentinc/cp-kafka"`)
    pub(crate) image: String,
    /// Docker image tag (e.g., `"7.2.4"`, `"latest"`)
    pub(crate) tag: String,
    /// Environment variables to set on the container
    pub(crate) env_vars: Vec<(String, String)>,
    /// Optional entrypoint override
    pub(crate) entrypoint: Option<String>,
    /// Fixed host-to-container port mappings keyed by container port,
    /// with host port as the value. Populated by the framework during
    /// config wiring for container connections.
    pub(crate) mapped_ports: HashMap<u16, u16>,
}

impl ContainerConfig {
    /// Create a new container configuration for the given Docker image and tag.
    #[must_use]
    pub fn new(image: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            tag: tag.into(),
            env_vars: Vec::new(),
            entrypoint: None,
            mapped_ports: HashMap::new(),
        }
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
    /// Builds the Docker image, applies port mappings, environment variables,
    /// and entrypoint overrides, then starts the container. Returns a handle
    /// to the running container.
    pub(crate) async fn start(self) -> Result<ContainerAsync<GenericImage>, ValidationError> {
        let mut image = GenericImage::new(&self.image, &self.tag);

        if let Some(ep) = &self.entrypoint {
            image = image.with_entrypoint(ep);
        }

        image = image.with_wait_for(WaitFor::Nothing);

        // Convert to ContainerRequest for settings that consume the image.
        let mut request = testcontainers::core::ContainerRequest::from(image);

        // Apply host-to-container port mappings set during config wiring.
        for (&container_port, &host_port) in &self.mapped_ports {
            request = request.with_mapped_port(host_port, container_port.tcp());
        }

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
        assert!(config.env_vars.is_empty());
        assert!(config.entrypoint.is_none());
        assert!(config.mapped_ports.is_empty());
    }

    #[test]
    fn container_config_builder_chaining() {
        let config = ContainerConfig::new("kafka", "7.5.0")
            .env("FOO", "bar")
            .env("BAZ", "qux")
            .entrypoint("/bin/sh");

        assert_eq!(config.env_vars.len(), 2);
        assert_eq!(config.env_vars[0], ("FOO".into(), "bar".into()));
        assert_eq!(config.entrypoint, Some("/bin/sh".into()));
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
        let mut config = ContainerConfig::new("alpine", "3.20");
        // Simulate a mapped port as the framework would set during config wiring.
        let _ = config.mapped_ports.insert(8080, 8080);

        let container = config.start().await.expect("container should start");

        // Stop the container via testcontainers stop().
        container.stop().await.expect("container should stop");
    }
}
