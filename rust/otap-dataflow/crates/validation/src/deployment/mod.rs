// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Run a candidate engine and observe its pipelines through the production admin client.
mod process;
pub mod syslog;
use anyhow::{Context, Result, bail, ensure};
use otel_arrow_dfe_admin_api::{
    AdminClient, AdminEndpoint, HttpAdminClientSettings,
    engine::ProbeStatus,
    operations::OperationOptions,
    pipelines::{PipelineDetails, PipelineRolloutState, ReconfigureOutcome, ReconfigureRequest},
};
pub use process::Engine;
use std::{path::Path, time::Duration};
use tokio::time::{sleep, timeout};

/// Poll a state observation until it succeeds, errors, or reaches the validation deadline.
pub async fn wait_for(label: &str, mut probe: impl AsyncFnMut() -> Result<bool>) -> Result<()> {
    timeout(Duration::from_secs(12), async {
        loop {
            if probe().await? {
                return Ok(());
            }
            sleep(Duration::from_millis(30)).await;
        }
    })
    .await
    .with_context(|| format!("timed out waiting for {label}"))?
}

/// Owns one deployed engine process until the session is dropped.
pub struct EngineSession {
    _process: process::Running,
    client: AdminClient,
    admin_url: String,
}

impl EngineSession {
    /// Start the candidate and wait for admin readiness and the requested running core counts.
    pub async fn start(
        engine: &Engine,
        config: &Path,
        env: &[(&str, String)],
        admin_url: String,
        pipelines: &[(&str, &str, usize)],
    ) -> Result<Self> {
        let client = AdminClient::builder()
            .http(HttpAdminClientSettings::new(AdminEndpoint::from_url(
                &admin_url,
            )?))
            .build()?;
        let mut process = engine.start(config, env)?;
        wait_for("engine and pipelines to become ready", async || {
            process.check()?;
            if !matches!(client.engine().readyz().await, Ok(probe) if probe.status == ProbeStatus::Ok) {
                return Ok(false);
            }
            for &(group, id, cores) in pipelines {
                if !matches!(client.pipelines().status(group, id).await, Ok(Some(s)) if s.running_cores == cores) {
                    return Ok(false);
                }
            }
            Ok(true)
        }).await?;
        Ok(Self {
            _process: process,
            client,
            admin_url,
        })
    }

    /// Use the typed admin endpoints of this candidate.
    #[must_use]
    pub fn client(&self) -> &AdminClient {
        &self.client
    }

    /// Base URL for surfaces without a typed admin endpoint.
    #[must_use]
    pub fn admin_url(&self) -> &str {
        &self.admin_url
    }

    /// Read the committed pipeline configuration and its active generation.
    pub async fn details(&self, group: &str, id: &str) -> Result<PipelineDetails> {
        self.client
            .pipelines()
            .details(group, id)
            .await?
            .with_context(|| format!("missing pipeline {group}/{id}"))
    }

    /// Apply a pipeline configuration and verify its committed topology. The caller
    /// can probe an independent pipeline between submission and rollout completion.
    pub async fn rollout(
        &self,
        group: &str,
        id: &str,
        request: &ReconfigureRequest,
        during: impl AsyncFnOnce() -> Result<()>,
    ) -> Result<()> {
        let outcome = self
            .client
            .pipelines()
            .reconfigure(group, id, request, &OperationOptions::default())
            .await?;
        during().await?;
        let status = match outcome {
            ReconfigureOutcome::Accepted(status) | ReconfigureOutcome::Completed(status) => status,
            other => bail!("{group}/{id} rollout: {other:?}"),
        };
        wait_for(&status.rollout_id, async || {
            let current = self
                .client
                .pipelines()
                .rollout_status(group, id, &status.rollout_id)
                .await?
                .context("missing rollout")?;
            match current.state {
                PipelineRolloutState::Succeeded => Ok(true),
                PipelineRolloutState::Failed | PipelineRolloutState::RollbackFailed => {
                    bail!("{group}/{id} rollout failed: {:?}", current.failure_reason)
                }
                _ => Ok(false),
            }
        })
        .await?;
        ensure!(
            self.details(group, id).await?.pipeline == request.pipeline,
            "{group}/{id}: committed pipeline differs from requested topology"
        );
        Ok(())
    }

    /// Reapply a pipeline and verify the no-op action, generation and committed topology.
    pub async fn assert_noop(
        &self,
        group: &str,
        id: &str,
        request: &ReconfigureRequest,
    ) -> Result<()> {
        let before = self.details(group, id).await?;
        let generation = before
            .active_generation
            .context("missing serving generation")?;
        let outcome = self
            .client
            .pipelines()
            .reconfigure(
                group,
                id,
                request,
                &OperationOptions {
                    wait: true,
                    timeout_secs: 10,
                },
            )
            .await?;
        ensure!(
            matches!(&outcome, ReconfigureOutcome::Completed(status)
            if status.action == "noop" && status.state == PipelineRolloutState::Succeeded),
            "reapplying the committed configuration should be a successful noop: {outcome:?}"
        );
        let after = self.details(group, id).await?;
        ensure!(
            after.active_generation == Some(generation) && after.pipeline == before.pipeline,
            "noop changed the committed configuration or serving generation"
        );
        Ok(())
    }
}
