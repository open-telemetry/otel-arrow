// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Action execution helpers for mutating TUI workflows.
//!
//! This module is the boundary between interactive operator intent and admin
//! API mutations. It validates the selected action, submits reconfigure,
//! shutdown, and scaling requests through the SDK, and updates app state so the
//! UI can immediately show operation progress without duplicating mutation
//! logic in input handlers or rendering code.

use super::*;

/// Executes a confirmed TUI action and updates UI state to show the resulting operation.
pub(super) async fn execute_ui_action(
    client: &AdminClient,
    app: &mut AppState,
    _args: &UiArgs,
    action: UiAction,
) -> Result<(), CliError> {
    match action {
        UiAction::PipelineEditAndRedeploy { .. } => Err(CliError::invalid_usage(
            "edit and redeploy must be opened through the editor flow",
        )),
        UiAction::PipelineDeployDraft {
            group_id,
            pipeline_id,
        } => submit_pipeline_config_draft(client, app, &group_id, &pipeline_id).await,
        UiAction::PipelineScale {
            group_id,
            pipeline_id,
            target_cores,
        } => submit_pipeline_scale(client, app, &group_id, &pipeline_id, target_cores).await,
        UiAction::PipelineSetCoreCount { .. } => Err(CliError::invalid_usage(
            "set core count must be confirmed through the scale editor",
        )),
        UiAction::PipelineShutdown {
            group_id,
            pipeline_id,
        } => submit_pipeline_shutdown(client, app, &group_id, &pipeline_id).await,
        UiAction::GroupShutdown {
            group_id,
            pipelines,
        } => submit_group_shutdown(client, app, &group_id, &pipelines).await,
    }
}

async fn submit_pipeline_scale(
    client: &AdminClient,
    app: &mut AppState,
    group_id: &str,
    pipeline_id: &str,
    target_cores: usize,
) -> Result<(), CliError> {
    if target_cores == 0 {
        return Err(CliError::invalid_usage(
            "pipeline core count must be greater than 0",
        ));
    }

    let details = load_pipeline_details(client, app, group_id, pipeline_id).await?;
    submit_pipeline_reconfigure(
        client,
        app,
        group_id,
        pipeline_id,
        resize_pipeline_config(details.pipeline, target_cores)?,
    )
    .await
}

async fn submit_pipeline_config_draft(
    client: &AdminClient,
    app: &mut AppState,
    group_id: &str,
    pipeline_id: &str,
) -> Result<(), CliError> {
    let Some(draft) = app.pipelines.config_draft.as_ref() else {
        return Err(CliError::invalid_usage(
            "no staged pipeline config draft is available",
        ));
    };
    let Some(pipeline) = draft.parsed.clone() else {
        return Err(CliError::invalid_usage(
            "the staged pipeline config draft is not valid",
        ));
    };

    submit_pipeline_reconfigure(client, app, group_id, pipeline_id, pipeline).await?;
    app.pipelines.config_draft = None;
    Ok(())
}

async fn submit_pipeline_reconfigure(
    client: &AdminClient,
    app: &mut AppState,
    group_id: &str,
    pipeline_id: &str,
    pipeline: PipelineConfig,
) -> Result<(), CliError> {
    let request = pipelines::ReconfigureRequest {
        pipeline,
        step_timeout_secs: 60,
        drain_timeout_secs: 60,
    };
    let outcome = client
        .pipelines()
        .reconfigure(
            group_id,
            pipeline_id,
            &request,
            &ui_operation_options(false),
        )
        .await?;

    let status = match outcome {
        pipelines::ReconfigureOutcome::Accepted(status)
        | pipelines::ReconfigureOutcome::Completed(status)
        | pipelines::ReconfigureOutcome::Failed(status)
        | pipelines::ReconfigureOutcome::TimedOut(status) => status,
    };

    app.view = View::Pipelines;
    app.pipeline_selected = Some(format!("{group_id}:{pipeline_id}"));
    app.pipeline_tab = PipelineTab::Rollout;
    app.focus = FocusArea::Detail;
    app.reset_scroll();
    app.pipelines.active_rollout_id = Some(status.rollout_id);
    Ok(())
}

async fn submit_pipeline_shutdown(
    client: &AdminClient,
    app: &mut AppState,
    group_id: &str,
    pipeline_id: &str,
) -> Result<(), CliError> {
    let outcome = client
        .pipelines()
        .shutdown(group_id, pipeline_id, &ui_operation_options(false))
        .await?;

    let status = match outcome {
        pipelines::ShutdownOutcome::Accepted(status)
        | pipelines::ShutdownOutcome::Completed(status)
        | pipelines::ShutdownOutcome::Failed(status)
        | pipelines::ShutdownOutcome::TimedOut(status) => status,
    };

    app.view = View::Pipelines;
    app.pipeline_selected = Some(format!("{group_id}:{pipeline_id}"));
    app.pipeline_tab = PipelineTab::Shutdown;
    app.focus = FocusArea::Detail;
    app.reset_scroll();
    app.pipelines.active_shutdown_id = Some(status.shutdown_id);
    Ok(())
}

async fn submit_group_shutdown(
    client: &AdminClient,
    app: &mut AppState,
    group_id: &str,
    pipeline_keys: &[String],
) -> Result<(), CliError> {
    if pipeline_keys.is_empty() {
        return Err(CliError::invalid_usage(
            "the selected group has no pipelines to shut down",
        ));
    }

    let mut submission_errors = Vec::new();
    for key in pipeline_keys {
        let Some((pipeline_group_id, pipeline_id)) = key.split_once(':') else {
            submission_errors.push(format!("invalid pipeline key '{key}'"));
            continue;
        };
        match client
            .pipelines()
            .shutdown(pipeline_group_id, pipeline_id, &ui_operation_options(false))
            .await
        {
            Ok(
                pipelines::ShutdownOutcome::Accepted(_) | pipelines::ShutdownOutcome::Completed(_),
            ) => {}
            Ok(pipelines::ShutdownOutcome::Failed(status)) => {
                submission_errors.push(format!(
                    "{pipeline_group_id}/{pipeline_id}: shutdown '{}' ended in state {}",
                    status.shutdown_id, status.state
                ));
            }
            Ok(pipelines::ShutdownOutcome::TimedOut(status)) => {
                submission_errors.push(format!(
                    "{pipeline_group_id}/{pipeline_id}: shutdown '{}' timed out in state {}",
                    status.shutdown_id, status.state
                ));
            }
            Err(err) => {
                submission_errors.push(format!("{pipeline_group_id}/{pipeline_id}: {err}"));
            }
        }
    }

    app.view = View::Groups;
    app.group_selected = Some(group_id.to_string());
    app.group_tab = GroupTab::Shutdown;
    app.focus = FocusArea::Detail;
    app.reset_scroll();
    app.groups.active_shutdown = Some(app::ActiveGroupShutdown {
        group_id: group_id.to_string(),
        pipeline_count: pipeline_keys.len(),
        started_at: std::time::SystemTime::now(),
        wait_timeout: Duration::from_secs(60),
        submission_errors,
        request_count: pipeline_keys.len(),
    });
    Ok(())
}

/// Loads pipeline details, reusing the currently cached describe report when it matches.
pub(super) async fn load_pipeline_details(
    client: &AdminClient,
    app: &AppState,
    group_id: &str,
    pipeline_id: &str,
) -> Result<pipelines::PipelineDetails, CliError> {
    if let Some(describe) = app.pipelines.describe.as_ref()
        && describe.details.pipeline_group_id.as_ref() == group_id
        && describe.details.pipeline_id.as_ref() == pipeline_id
    {
        return Ok(describe.details.clone());
    }

    client
        .pipelines()
        .details(group_id, pipeline_id)
        .await?
        .ok_or_else(|| {
            CliError::not_found(format!("pipeline '{group_id}/{pipeline_id}' was not found"))
        })
}

fn resize_pipeline_config(
    pipeline: PipelineConfig,
    target_cores: usize,
) -> Result<PipelineConfig, CliError> {
    let mut value = serde_json::to_value(&pipeline).map_err(|err| {
        CliError::config(format!(
            "failed to serialize pipeline config for resize: {err}"
        ))
    })?;
    let root = value
        .as_object_mut()
        .ok_or_else(|| CliError::config("failed to edit pipeline config: root is not an object"))?;
    let policies = root
        .entry("policies".to_string())
        .or_insert_with(|| json!({}));
    let policies = policies.as_object_mut().ok_or_else(|| {
        CliError::config("failed to edit pipeline config: policies is not an object")
    })?;
    let resources = policies
        .entry("resources".to_string())
        .or_insert_with(|| json!({}));
    let resources = resources.as_object_mut().ok_or_else(|| {
        CliError::config("failed to edit pipeline config: resources is not an object")
    })?;
    let _ = resources.insert(
        "core_allocation".to_string(),
        json!({
            "type": "core_count",
            "count": target_cores,
        }),
    );
    serde_json::from_value(value).map_err(|err| {
        CliError::config(format!(
            "failed to rebuild pipeline config after resizing to {target_cores} cores: {err}"
        ))
    })
}

fn ui_operation_options(wait: bool) -> OperationOptions {
    OperationOptions {
        wait,
        timeout_secs: 60,
    }
}
