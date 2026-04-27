// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for app-state selection, command recipes, and palette behavior.
//!
//! These tests document the user-facing guarantees of the state layer without
//! requiring terminal rendering: filters preserve useful selections, recipes
//! point to executable CLI commands, pipeline badges reflect operational state,
//! and the command palette chooses the expected action for the active context.

use super::*;

use otap_df_admin_api::pipelines::Status as PipelineStatus;
use serde_json::json;
use std::time::Duration;

fn pipeline_status(
    running_cores: usize,
    total_cores: usize,
    ready: bool,
    with_rollout: bool,
) -> PipelineStatus {
    let conditions = if ready {
        vec![json!({
            "type": "Ready",
            "status": "True",
            "reason": "Ready",
            "message": "ok"
        })]
    } else {
        Vec::new()
    };
    let mut value = json!({
        "conditions": conditions.clone(),
        "totalCores": total_cores,
        "runningCores": running_cores,
        "cores": {
            "0": {
                "phase": if running_cores > 0 { "running" } else { "stopped" },
                "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                "conditions": conditions,
                "deletePending": false
            }
        }
    });
    if with_rollout {
        value["rollout"] = json!({
            "rolloutId": "rollout-0",
            "state": "running",
            "targetGeneration": 1,
            "startedAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:01Z"
        });
    }
    serde_json::from_value(value).expect("status fixture should deserialize")
}

fn pipeline_status_with_rollout_state(state: &str) -> PipelineStatus {
    let mut value = json!({
        "conditions": [{
            "type": "Ready",
            "status": "True",
            "reason": "Ready",
            "message": "ok"
        }],
        "totalCores": 2,
        "runningCores": 2,
        "activeGeneration": 0,
        "cores": {
            "0": {
                "phase": "running",
                "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                "conditions": [],
                "deletePending": false
            },
            "1": {
                "phase": "running",
                "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                "conditions": [],
                "deletePending": false
            }
        },
        "rollout": {
            "rolloutId": "rollout-0",
            "state": state,
            "targetGeneration": 1,
            "startedAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:01Z"
        }
    });
    if state == "succeeded" {
        value["rollout"]["failureReason"] = serde_json::Value::Null;
    }
    serde_json::from_value(value).expect("status fixture should deserialize")
}

/// Scenario: the pipelines list is filtered by a substring that matches
/// only one pipeline key.
/// Guarantees: the filtered list keeps only matching pipelines and
/// preserves the semantic status badge for the remaining row.
#[test]
fn pipeline_items_apply_filter() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.groups_status = Some(groups::Status {
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        pipelines: BTreeMap::from([
            (
                "tenant-a:ingest".to_string(),
                pipeline_status(1, 1, true, false),
            ),
            (
                "tenant-b:export".to_string(),
                pipeline_status(1, 1, true, false),
            ),
        ]),
    });
    app.filter_query = "tenant-a".to_string();

    let items = app.pipeline_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].key, "tenant-a:ingest");
    assert_eq!(items[0].status_badge, "ok");
}

/// Scenario: multiple pipelines from the same group contribute different
/// running and readiness states.
/// Guarantees: the group aggregate row reports the combined counts and the
/// correct warning badge.
#[test]
fn group_items_aggregate_counts() {
    let mut app = AppState::new(UiStartView::Groups, true, 200);
    app.groups_status = Some(groups::Status {
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        pipelines: BTreeMap::from([
            (
                "tenant-a:ingest".to_string(),
                pipeline_status(1, 1, true, false),
            ),
            (
                "tenant-a:transform".to_string(),
                pipeline_status(0, 1, false, false),
            ),
        ]),
    });

    let items = app.group_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].group_id, "tenant-a");
    assert_eq!(items[0].pipelines, 2);
    assert_eq!(items[0].running, 1);
    assert_eq!(items[0].ready, 1);
    assert_eq!(items[0].terminal, 1);
    assert_eq!(items[0].status_badge, "warn");
}

/// Scenario: applying a new filter would remove the currently selected
/// pipeline from view.
/// Guarantees: selection is repaired to the first remaining pipeline in the
/// filtered result set.
#[test]
fn filter_apply_updates_selection() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.groups_status = Some(groups::Status {
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        pipelines: BTreeMap::from([
            (
                "tenant-a:ingest".to_string(),
                pipeline_status(1, 1, true, false),
            ),
            (
                "tenant-b:export".to_string(),
                pipeline_status(1, 1, true, true),
            ),
        ]),
    });
    app.pipeline_selected = Some("tenant-b:export".to_string());
    app.filter_input = "tenant-a".to_string();

    app.apply_filter_input();

    assert_eq!(app.pipeline_selected.as_deref(), Some("tenant-a:ingest"));
}

/// Scenario: a pipeline has an active non-terminal rollout.
/// Guarantees: the pipeline list shows the rollout badge and accent tone
/// rather than an ordinary healthy state.
#[test]
fn rollout_pipeline_items_show_roll_badge() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.groups_status = Some(groups::Status {
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        pipelines: BTreeMap::from([(
            "tenant-a:ingest".to_string(),
            pipeline_status(1, 1, true, true),
        )]),
    });

    let items = app.pipeline_items();
    assert_eq!(items[0].status_badge, "roll");
    assert_eq!(items[0].tone, Tone::Accent);
}

/// Scenario: rollout metadata remains attached after a rollout already
/// reached its terminal succeeded state.
/// Guarantees: the pipeline list falls back to the steady-state success
/// badge while still surfacing the terminal rollout state text.
#[test]
fn terminal_rollout_pipeline_items_fall_back_to_ok_badge() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.groups_status = Some(groups::Status {
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        pipelines: BTreeMap::from([(
            "tenant-a:ingest".to_string(),
            pipeline_status_with_rollout_state("succeeded"),
        )]),
    });

    let items = app.pipeline_items();
    assert_eq!(items[0].status_badge, "ok");
    assert_eq!(items[0].tone, Tone::Success);
    assert_eq!(items[0].rollout, "succeeded");
}

/// Scenario: the rollout detail recipe is requested for a pipeline whose
/// latest rollout is already terminal.
/// Guarantees: the generated recipe asks for an explicit rollout id instead
/// of pretending an active rollout is still available.
#[test]
fn rollout_recipe_ignores_terminal_rollout_summary() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.pipeline_selected = Some("tenant-a:ingest".to_string());
    app.pipeline_tab = PipelineTab::Rollout;
    app.groups_status = Some(groups::Status {
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        pipelines: BTreeMap::from([(
            "tenant-a:ingest".to_string(),
            pipeline_status_with_rollout_state("succeeded"),
        )]),
    });

    let recipe = app.current_command_recipe();

    assert!(recipe.description.contains("No active rollout is visible"));
    assert!(recipe.commands[0].command.contains("<rollout-id>"));
}

/// Scenario: the logs pane generates an equivalent CLI recipe while a
/// non-default command context is configured.
/// Guarantees: the recipe includes the selected target scope and the global
/// prefix arguments needed to reproduce the pane from a shell.
#[test]
fn pipeline_logs_recipe_uses_selected_target_and_context() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.pipeline_selected = Some("tenant-a:ingest".to_string());
    app.pipeline_tab = PipelineTab::Logs;
    app.set_command_context(UiCommandContext {
        target_url: "https://admin.example.com:8443/engine-a".to_string(),
        prefix_args: vec![
            "dfctl".to_string(),
            "--url".to_string(),
            "https://admin.example.com:8443/engine-a".to_string(),
            "--color".to_string(),
            "always".to_string(),
        ],
        sensitive_args_redacted: false,
        refresh_interval: Duration::from_secs(5),
        logs_tail: 250,
    });

    let recipe = app.current_command_recipe();

    assert_eq!(recipe.commands.len(), 1);
    assert!(recipe.commands[0].command.contains(
        "telemetry logs watch --tail 250 --interval 5s --group tenant-a --pipeline ingest"
    ));
    assert!(recipe.commands[0].command.contains("--color always"));
}

/// Scenario: the config pane recipe is requested before any staged draft is
/// present.
/// Guarantees: the recipe points operators to the get command and explains
/// that editing flows through the external editor path.
#[test]
fn pipeline_config_recipe_points_to_get_and_editor_flow() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.pipeline_selected = Some("tenant-a:ingest".to_string());
    app.pipeline_tab = PipelineTab::Config;

    let recipe = app.current_command_recipe();

    assert_eq!(recipe.commands.len(), 1);
    assert_eq!(
        recipe.commands[0].command,
        "dfctl --url http://127.0.0.1:8085 pipelines get tenant-a ingest --output yaml"
    );
    assert!(
        recipe
            .note
            .expect("config recipe should include note")
            .contains("external editor")
    );
}

/// Scenario: the pipeline Details tab generates an equivalent CLI recipe.
/// Guarantees: the recipe includes both the rich describe command and the
/// lower-level status command that back the object-inspection view.
#[test]
fn pipeline_details_recipe_includes_describe_and_status() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.pipeline_selected = Some("tenant-a:ingest".to_string());
    app.pipeline_tab = PipelineTab::Details;

    let recipe = app.current_command_recipe();

    assert_eq!(recipe.commands.len(), 2);
    assert_eq!(
        recipe.commands[0].command,
        "dfctl --url http://127.0.0.1:8085 pipelines describe tenant-a ingest"
    );
    assert_eq!(
        recipe.commands[1].command,
        "dfctl --url http://127.0.0.1:8085 pipelines status tenant-a ingest"
    );
}

/// Scenario: the groups summary pane generates an equivalent CLI recipe for
/// a selected group.
/// Guarantees: the recipe calls out that the UI scope is client-side while
/// the underlying CLI command remains fleet-wide.
#[test]
fn group_summary_recipe_mentions_client_side_scope() {
    let mut app = AppState::new(UiStartView::Groups, true, 200);
    app.group_selected = Some("tenant-a".to_string());

    let recipe = app.current_command_recipe();

    assert_eq!(
        recipe.commands[0].command,
        "dfctl --url http://127.0.0.1:8085 groups describe"
    );
    assert!(
        recipe
            .note
            .expect("group summary recipe should include note")
            .contains("client-side")
    );
}
