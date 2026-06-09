// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared fetch helpers that normalize not-found handling across commands.
//!
//! Command runners use this module when several command families need the same
//! admin SDK lookup and the same CLI error semantics. Keeping these helpers in
//! one place avoids subtle differences in not-found messages for logs,
//! rollouts, shutdowns, pipeline status, and pipeline describe data.

use crate::error::CliError;
use crate::troubleshoot::{PipelineDescribeReport, describe_pipeline};
use otap_df_admin_api::telemetry::LogsQuery;
use otap_df_admin_api::{AdminClient, telemetry};

/// Fetch retained admin logs or return the current CLI not-found error.
pub(crate) async fn fetch_logs(
    client: &AdminClient,
    after: Option<u64>,
    limit: Option<usize>,
) -> Result<telemetry::LogsResponse, CliError> {
    let logs = client.telemetry().logs(&LogsQuery { after, limit }).await?;
    logs.ok_or_else(|| CliError::not_found("retained admin logs are not available on this engine"))
}

/// Fetch rollout status and normalize missing resources into a CLI not-found error.
pub(crate) async fn fetch_rollout(
    client: &AdminClient,
    pipeline_group_id: &str,
    pipeline_id: &str,
    rollout_id: &str,
) -> Result<otap_df_admin_api::pipelines::RolloutStatus, CliError> {
    client
        .pipelines()
        .rollout_status(pipeline_group_id, pipeline_id, rollout_id)
        .await?
        .ok_or_else(|| {
            CliError::not_found(format!(
                "rollout '{}' for pipeline '{}/{}' was not found",
                rollout_id, pipeline_group_id, pipeline_id
            ))
        })
}

/// Fetch shutdown status and normalize missing resources into a CLI not-found error.
pub(crate) async fn fetch_shutdown(
    client: &AdminClient,
    pipeline_group_id: &str,
    pipeline_id: &str,
    shutdown_id: &str,
) -> Result<otap_df_admin_api::pipelines::ShutdownStatus, CliError> {
    client
        .pipelines()
        .shutdown_status(pipeline_group_id, pipeline_id, shutdown_id)
        .await?
        .ok_or_else(|| {
            CliError::not_found(format!(
                "shutdown '{}' for pipeline '{}/{}' was not found",
                shutdown_id, pipeline_group_id, pipeline_id
            ))
        })
}

/// Fetch pipeline status and normalize a missing pipeline into a CLI not-found error.
pub(crate) async fn fetch_pipeline_status(
    client: &AdminClient,
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> Result<otap_df_admin_api::pipelines::Status, CliError> {
    client
        .pipelines()
        .status(pipeline_group_id, pipeline_id)
        .await?
        .ok_or_else(|| {
            CliError::not_found(format!(
                "pipeline '{}/{}' was not found",
                pipeline_group_id, pipeline_id
            ))
        })
}

/// Fetch the details, status, and probes that power the describe/diagnose/bundle flows.
pub(crate) async fn fetch_pipeline_describe(
    client: &AdminClient,
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> Result<PipelineDescribeReport, CliError> {
    let details = client
        .pipelines()
        .details(pipeline_group_id, pipeline_id)
        .await?;
    let Some(details) = details else {
        return Err(CliError::not_found(format!(
            "pipeline '{}/{}' was not found",
            pipeline_group_id, pipeline_id
        )));
    };
    let status = fetch_pipeline_status(client, pipeline_group_id, pipeline_id).await?;
    let livez = client
        .pipelines()
        .livez(pipeline_group_id, pipeline_id)
        .await?;
    let readyz = client
        .pipelines()
        .readyz(pipeline_group_id, pipeline_id)
        .await?;
    Ok(describe_pipeline(details, status, livez, readyz))
}
