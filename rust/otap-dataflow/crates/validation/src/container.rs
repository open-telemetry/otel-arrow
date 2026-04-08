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
use crate::template::render_jinja;
use minijinja::context;
use std::collections::HashMap;
use testcontainers::core::IntoContainerPort;
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

/// An environment variable whose value is a Jinja2 template resolved
/// after host port allocation. `{{ host_port }}` in the template is
/// replaced with the host port mapped to `internal_port`.
#[derive(Debug, Clone)]
pub(crate) struct TemplatedEnvVar {
    pub(crate) key: String,
    pub(crate) value_template: String,
    pub(crate) internal_port: u16,
}

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
    /// Templated environment variables to be resolved after port allocation.
    /// `None` when no templated env vars have been set.
    pub(crate) templated_env_vars: Option<Vec<TemplatedEnvVar>>,
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
            templated_env_vars: None,
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

    /// Set an environment variable whose value is a Jinja2 template.
    /// After port allocation, `{{ host_port }}` is replaced with the host
    /// port mapped to `internal_port`. If no connection maps that port,
    /// the framework auto-allocates one during config wiring.
    ///
    /// # Example
    ///
    /// ```ignore
    /// ContainerConfig::new("confluentinc/cp-kafka", "7.5.0")
    ///     .env_host_port(
    ///         "KAFKA_ADVERTISED_LISTENERS",
    ///         "PLAINTEXT://127.0.0.1:{{ host_port }}",
    ///         9092,
    ///     )
    /// ```
    #[must_use]
    pub fn env_host_port(
        mut self,
        key: impl Into<String>,
        value_template: impl Into<String>,
        internal_port: u16,
    ) -> Self {
        self.templated_env_vars
            .get_or_insert_with(Vec::new)
            .push(TemplatedEnvVar {
                key: key.into(),
                value_template: value_template.into(),
                internal_port,
            });
        self
    }

    /// Resolve all templated environment variables by rendering their
    /// Jinja2 templates with `host_port` set to the allocated host port
    /// from `mapped_ports`. Resolved values are appended to `env_vars`
    /// in FIFO order. After resolution, `templated_env_vars` is set to
    /// `None`.
    pub(crate) fn resolve_templated_env_vars(&mut self) -> Result<(), ValidationError> {
        let vars = match self.templated_env_vars.take() {
            Some(v) => v,
            None => return Ok(()),
        };
        for tev in vars {
            let &host_port = self.mapped_ports.get(&tev.internal_port).ok_or_else(|| {
                ValidationError::Config(format!(
                    "templated env var '{}' references unmapped internal port {}",
                    tev.key, tev.internal_port
                ))
            })?;
            let rendered = render_jinja(&tev.value_template, context! { host_port => host_port })?;
            self.env_vars.push((tev.key, rendered));
        }
        Ok(())
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
        assert!(config.templated_env_vars.is_none());
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

    #[test]
    fn env_host_port_stores_templated_var() {
        let config = ContainerConfig::new("kafka", "7.5.0").env_host_port(
            "KAFKA_ADVERTISED_LISTENERS",
            "PLAINTEXT://127.0.0.1:{{ host_port }}",
            9092,
        );
        let tevs = config.templated_env_vars.as_ref().expect("should be Some");
        assert_eq!(tevs.len(), 1);
        assert_eq!(tevs[0].key, "KAFKA_ADVERTISED_LISTENERS");
        assert_eq!(
            tevs[0].value_template,
            "PLAINTEXT://127.0.0.1:{{ host_port }}"
        );
        assert_eq!(tevs[0].internal_port, 9092);
    }

    #[test]
    fn env_host_port_multiple_preserves_order() {
        let config = ContainerConfig::new("kafka", "7.5.0")
            .env_host_port("FIRST", "{{ host_port }}", 9092)
            .env_host_port("SECOND", "{{ host_port }}", 8080);
        let tevs = config.templated_env_vars.as_ref().expect("should be Some");
        assert_eq!(tevs.len(), 2);
        assert_eq!(tevs[0].key, "FIRST");
        assert_eq!(tevs[0].internal_port, 9092);
        assert_eq!(tevs[1].key, "SECOND");
        assert_eq!(tevs[1].internal_port, 8080);
    }

    #[test]
    fn resolve_templated_env_vars_renders_and_consumes() {
        let mut config = ContainerConfig::new("kafka", "7.5.0").env_host_port(
            "LISTENERS",
            "PLAINTEXT://127.0.0.1:{{ host_port }}",
            9092,
        );
        // Simulate port allocation during wiring.
        let _ = config.mapped_ports.insert(9092, 54321);

        config
            .resolve_templated_env_vars()
            .expect("resolve should succeed");

        assert!(config.templated_env_vars.is_none());
        assert_eq!(config.env_vars.len(), 1);
        assert_eq!(config.env_vars[0].0, "LISTENERS");
        assert_eq!(config.env_vars[0].1, "PLAINTEXT://127.0.0.1:54321");
    }

    #[test]
    fn resolve_templated_env_vars_preserves_order() {
        let mut config = ContainerConfig::new("img", "tag")
            .env_host_port("FIRST", "a:{{ host_port }}", 9092)
            .env_host_port("SECOND", "b:{{ host_port }}", 8080);
        let _ = config.mapped_ports.insert(9092, 11111);
        let _ = config.mapped_ports.insert(8080, 22222);

        config
            .resolve_templated_env_vars()
            .expect("resolve should succeed");

        assert!(config.templated_env_vars.is_none());
        assert_eq!(config.env_vars.len(), 2);
        assert_eq!(config.env_vars[0], ("FIRST".into(), "a:11111".into()));
        assert_eq!(config.env_vars[1], ("SECOND".into(), "b:22222".into()));
    }

    #[test]
    fn resolve_templated_env_vars_unmapped_port_errors() {
        let mut config =
            ContainerConfig::new("img", "tag").env_host_port("MY_VAR", "{{ host_port }}", 5432);

        let err = config
            .resolve_templated_env_vars()
            .expect_err("should error on unmapped port");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("unmapped internal port"));
        assert!(err.to_string().contains("MY_VAR"));
    }

    #[test]
    fn resolve_templated_env_vars_none_is_noop() {
        let mut config = ContainerConfig::new("img", "tag");
        assert!(config.templated_env_vars.is_none());

        config
            .resolve_templated_env_vars()
            .expect("noop resolve should succeed");

        assert!(config.templated_env_vars.is_none());
        assert!(config.env_vars.is_empty());
    }
}
