// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Equivalent CLI recipe generation, modal state helpers, and action-menu assembly.
//!
//! Recipes explain which non-interactive `dfctl` command corresponds to the
//! current TUI pane, making the UI a learning surface for automation. This
//! module also owns modal transitions that are closely tied to those actions,
//! such as opening action menus, staging config drafts, and preparing
//! confirmation or scale editors.

use super::*;

impl AppState {
    /// Builds the equivalent CLI command recipe for the active view and tab.
    pub(crate) fn current_command_recipe(&self) -> CommandRecipe {
        let mut recipe = match self.view {
            View::Pipelines => self.pipeline_command_recipe(),
            View::Groups => self.group_command_recipe(),
            View::Engine => self.engine_command_recipe(),
        };
        if self.command_context.sensitive_args_redacted {
            let redaction_note = "Sensitive command arguments are redacted in the TUI. Replace placeholders before running the command.";
            recipe.note = Some(match recipe.note {
                Some(note) => format!("{note} {redaction_note}"),
                None => redaction_note.to_string(),
            });
        }
        recipe
    }

    /// Opens the resource-list filter modal with the current query prefilled.
    pub(crate) fn start_filter_input(&mut self) {
        self.modal = UiModal::Filter;
        self.filter_input = self.filter_query.clone();
    }

    /// Applies the filter modal input to visible resource lists.
    pub(crate) fn apply_filter_input(&mut self) {
        self.filter_query = self.filter_input.trim().to_string();
        self.modal = UiModal::None;
        self.ensure_selection();
    }

    /// Closes the filter modal without changing the active filter.
    pub(crate) fn cancel_filter_input(&mut self) {
        self.modal = UiModal::None;
        self.filter_input.clear();
    }

    /// Clears the active resource-list filter and repairs selection.
    pub(crate) fn clear_filter(&mut self) {
        self.filter_query.clear();
        self.modal = UiModal::None;
        self.filter_input.clear();
        self.ensure_selection();
    }

    /// Resets the scroll offset of the detail pane.
    pub(crate) fn reset_scroll(&mut self) {
        self.detail_scroll = 0;
    }

    /// Returns true when the filter modal is active.
    pub(crate) fn is_filter_mode(&self) -> bool {
        matches!(self.modal, UiModal::Filter)
    }

    /// Returns true when the help overlay is active.
    pub(crate) fn show_help(&self) -> bool {
        matches!(self.modal, UiModal::Help)
    }

    /// Returns true when the equivalent-command overlay is active.
    pub(crate) fn show_command_overlay(&self) -> bool {
        matches!(self.modal, UiModal::Command)
    }

    /// Returns read-only action menu state when visible.
    pub(crate) fn action_menu(&self) -> Option<&ActionMenuState> {
        match &self.modal {
            UiModal::ActionMenu(menu) => Some(menu),
            _ => None,
        }
    }

    /// Returns read-only shutdown confirmation state when visible.
    pub(crate) fn shutdown_confirm(&self) -> Option<&ShutdownConfirmState> {
        match &self.modal {
            UiModal::ConfirmShutdown(confirm) => Some(confirm),
            _ => None,
        }
    }

    /// Returns read-only scale editor state when visible.
    pub(crate) fn scale_editor(&self) -> Option<&ScaleEditorState> {
        match &self.modal {
            UiModal::ScaleEditor(editor) => Some(editor),
            _ => None,
        }
    }

    /// Returns mutable action menu state when visible.
    pub(crate) fn action_menu_mut(&mut self) -> Option<&mut ActionMenuState> {
        match &mut self.modal {
            UiModal::ActionMenu(menu) => Some(menu),
            _ => None,
        }
    }

    /// Returns mutable scale editor state when visible.
    pub(crate) fn scale_editor_mut(&mut self) -> Option<&mut ScaleEditorState> {
        match &mut self.modal {
            UiModal::ScaleEditor(editor) => Some(editor),
            _ => None,
        }
    }

    /// Returns true when the current pipeline config draft can be deployed.
    pub(crate) fn has_deployable_pipeline_config_draft(&self) -> bool {
        self.pipelines
            .config_draft
            .as_ref()
            .is_some_and(PipelineConfigDraft::is_deployable)
    }

    /// Drops the currently staged pipeline config draft.
    pub(crate) fn discard_pipeline_config_draft(&mut self) {
        self.pipelines.config_draft = None;
    }

    /// Stores the current pipeline config draft and parse result.
    pub(crate) fn stage_pipeline_config_draft(
        &mut self,
        original_yaml: String,
        edited_yaml: String,
        parsed: Option<PipelineConfig>,
        diff: String,
        error: Option<String>,
    ) {
        self.pipelines.config_draft = Some(PipelineConfigDraft {
            original_yaml,
            edited_yaml,
            parsed,
            diff,
            error,
        });
    }

    /// Closes any active modal overlay.
    pub(crate) fn hide_modal(&mut self) {
        self.modal = UiModal::None;
    }

    /// Toggles the help overlay.
    pub(crate) fn toggle_help(&mut self) {
        self.modal = if self.show_help() {
            UiModal::None
        } else {
            UiModal::Help
        };
    }

    /// Toggles the equivalent-command overlay.
    pub(crate) fn toggle_command_overlay(&mut self) {
        self.modal = if self.show_command_overlay() {
            UiModal::None
        } else {
            UiModal::Command
        };
    }

    /// Opens the context action menu when actions are available.
    pub(crate) fn open_action_menu(&mut self) -> bool {
        let Some(menu) = self.build_action_menu() else {
            return false;
        };
        self.modal = UiModal::ActionMenu(menu);
        true
    }

    /// Opens the shutdown confirmation modal for a destructive action.
    pub(crate) fn open_shutdown_confirm(&mut self, action: UiAction) {
        let (title, prompt) = match &action {
            UiAction::PipelineShutdown {
                group_id,
                pipeline_id,
            } => (
                "Confirm Pipeline Shutdown".to_string(),
                format!("Submit shutdown for pipeline {group_id}/{pipeline_id}?"),
            ),
            UiAction::GroupShutdown {
                group_id,
                pipelines,
            } => (
                "Confirm Group Shutdown".to_string(),
                format!(
                    "Submit shutdown for all {} pipelines currently visible in group {group_id}?",
                    pipelines.len()
                ),
            ),
            _ => return,
        };
        self.modal = UiModal::ConfirmShutdown(ShutdownConfirmState {
            title,
            prompt,
            action,
        });
    }

    /// Opens the core-count editor for a pipeline scaling action.
    pub(crate) fn open_scale_editor(
        &mut self,
        group_id: String,
        pipeline_id: String,
        current_cores: usize,
    ) {
        self.modal = UiModal::ScaleEditor(ScaleEditorState {
            group_id,
            pipeline_id,
            current_cores,
            input: current_cores.to_string(),
        });
    }

    /// Returns all pipeline keys belonging to the selected group.
    pub(crate) fn selected_group_pipeline_keys(&self) -> Vec<String> {
        let Some(group_id) = self.selected_group_id() else {
            return Vec::new();
        };
        self.groups_status
            .as_ref()
            .map(|status| {
                status
                    .pipelines
                    .keys()
                    .filter(|key| {
                        key.split_once(':')
                            .is_some_and(|(group, _)| group == group_id)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    fn pipeline_command_recipe(&self) -> CommandRecipe {
        let Some((group_id, pipeline_id)) = self.selected_pipeline_target() else {
            return empty_recipe(
                "Equivalent CLI",
                "Wait for the first refresh and select a pipeline.",
            );
        };
        let selected_key = format!("{group_id}:{pipeline_id}");
        let pipeline_status = self
            .groups_status
            .as_ref()
            .and_then(|status| status.pipelines.get(&selected_key));

        match self.pipeline_tab {
            PipelineTab::Summary => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Describe pipeline {group_id}/{pipeline_id}."),
                commands: vec![command_line(
                    "describe",
                    self.command(vec![
                        "pipelines".to_string(),
                        "describe".to_string(),
                        group_id.clone(),
                        pipeline_id.clone(),
                    ]),
                )],
                note: None,
            },
            PipelineTab::Details => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Inspect detailed state for pipeline {group_id}/{pipeline_id}."),
                commands: vec![
                    command_line(
                        "describe",
                        self.command(vec![
                            "pipelines".to_string(),
                            "describe".to_string(),
                            group_id.clone(),
                            pipeline_id.clone(),
                        ]),
                    ),
                    command_line(
                        "status",
                        self.command(vec![
                            "pipelines".to_string(),
                            "status".to_string(),
                            group_id.clone(),
                            pipeline_id.clone(),
                        ]),
                    ),
                ],
                note: Some(
                    "The Details tab combines status, probes, and operation ids into one client-side view."
                        .to_string(),
                ),
            },
            PipelineTab::Config => {
                let mut commands = vec![command_line(
                    "get config",
                    self.command(vec![
                        "pipelines".to_string(),
                        "get".to_string(),
                        group_id.clone(),
                        pipeline_id.clone(),
                        "--output".to_string(),
                        "yaml".to_string(),
                    ]),
                )];
                let (description, note) = if self.has_deployable_pipeline_config_draft() {
                    commands.insert(
                        0,
                        command_line(
                            "reconfigure",
                            self.command(vec![
                                "pipelines".to_string(),
                                "reconfigure".to_string(),
                                group_id.clone(),
                                pipeline_id.clone(),
                                "--file".to_string(),
                                "<edited.yaml>".to_string(),
                                "--watch".to_string(),
                            ]),
                        ),
                    );
                    (
                        format!(
                            "Review the staged config draft for {group_id}/{pipeline_id} and redeploy it."
                        ),
                        Some(
                            "The UI keeps the edited YAML in memory. The CLI equivalent expects a file path."
                                .to_string(),
                        ),
                    )
                } else {
                    (
                        format!(
                            "Inspect the committed config for {group_id}/{pipeline_id}."
                        ),
                        Some(
                            "Use the action menu or press 'e' on this tab to open the config in an external editor."
                                .to_string(),
                        ),
                    )
                };
                CommandRecipe {
                    title: "Equivalent CLI".to_string(),
                    description,
                    commands,
                    note,
                }
            }
            PipelineTab::Events => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!(
                    "Watch normalized controller events for {group_id}/{pipeline_id}."
                ),
                commands: vec![command_line(
                    "watch events",
                    self.command(vec![
                        "pipelines".to_string(),
                        "events".to_string(),
                        "watch".to_string(),
                        group_id.clone(),
                        pipeline_id.clone(),
                        "--watch-interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                    ]),
                )],
                note: None,
            },
            PipelineTab::Logs => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Watch retained logs for {group_id}/{pipeline_id}."),
                commands: vec![command_line(
                    "watch logs",
                    self.command(vec![
                        "telemetry".to_string(),
                        "logs".to_string(),
                        "watch".to_string(),
                        "--tail".to_string(),
                        self.command_context.logs_tail.to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                        "--group".to_string(),
                        group_id.clone(),
                        "--pipeline".to_string(),
                        pipeline_id.clone(),
                    ]),
                )],
                note: None,
            },
            PipelineTab::Metrics => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Watch compact metrics for {group_id}/{pipeline_id}."),
                commands: vec![command_line(
                    "watch metrics",
                    self.command(vec![
                        "telemetry".to_string(),
                        "metrics".to_string(),
                        "watch".to_string(),
                        "--shape".to_string(),
                        "compact".to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                        "--group".to_string(),
                        group_id.clone(),
                        "--pipeline".to_string(),
                        pipeline_id.clone(),
                    ]),
                )],
                note: None,
            },
            PipelineTab::Rollout => {
                if let Some(rollout_id) = self
                    .pipelines
                    .active_rollout_id
                    .clone()
                    .or_else(|| pipeline_status.and_then(active_rollout_id))
                {
                    CommandRecipe {
                        title: "Equivalent CLI".to_string(),
                        description: format!(
                            "Watch active rollout {rollout_id} for {group_id}/{pipeline_id}."
                        ),
                        commands: vec![command_line(
                            "watch rollout",
                            self.command(vec![
                                "pipelines".to_string(),
                                "rollouts".to_string(),
                                "watch".to_string(),
                                group_id.clone(),
                                pipeline_id.clone(),
                                rollout_id.clone(),
                                "--interval".to_string(),
                                format_duration_arg(self.command_context.refresh_interval),
                            ]),
                        )],
                        note: None,
                    }
                } else {
                    CommandRecipe {
                        title: "Equivalent CLI".to_string(),
                        description: format!(
                            "No active rollout is visible for {group_id}/{pipeline_id}."
                        ),
                        commands: vec![command_line(
                            "watch rollout",
                            self.command(vec![
                                "pipelines".to_string(),
                                "rollouts".to_string(),
                                "watch".to_string(),
                                group_id.clone(),
                                pipeline_id.clone(),
                                "<rollout-id>".to_string(),
                                "--interval".to_string(),
                                format_duration_arg(self.command_context.refresh_interval),
                            ]),
                        )],
                        note: Some(
                            "This pane only has an exact CLI equivalent while a rollout is active."
                                .to_string(),
                        ),
                    }
                }
            }
            PipelineTab::Shutdown => {
                if let Some(shutdown_id) = self.pipelines.active_shutdown_id.clone() {
                    CommandRecipe {
                        title: "Equivalent CLI".to_string(),
                        description: format!(
                            "Watch shutdown {shutdown_id} for {group_id}/{pipeline_id}."
                        ),
                        commands: vec![command_line(
                            "watch shutdown",
                            self.command(vec![
                                "pipelines".to_string(),
                                "shutdowns".to_string(),
                                "watch".to_string(),
                                group_id.clone(),
                                pipeline_id.clone(),
                                shutdown_id,
                                "--interval".to_string(),
                                format_duration_arg(self.command_context.refresh_interval),
                            ]),
                        )],
                        note: None,
                    }
                } else {
                    CommandRecipe {
                        title: "Equivalent CLI".to_string(),
                        description: format!(
                            "No active shutdown is tracked for {group_id}/{pipeline_id}."
                        ),
                        commands: vec![command_line(
                            "watch shutdown",
                            self.command(vec![
                                "pipelines".to_string(),
                                "shutdowns".to_string(),
                                "watch".to_string(),
                                group_id.clone(),
                                pipeline_id.clone(),
                                "<shutdown-id>".to_string(),
                                "--interval".to_string(),
                                format_duration_arg(self.command_context.refresh_interval),
                            ]),
                        )],
                        note: Some(
                            "This pane has an exact CLI equivalent only after a shutdown has been submitted."
                                .to_string(),
                        ),
                    }
                }
            }
            PipelineTab::Diagnose => {
                let (label, command) = if let Some(rollout_id) = pipeline_status
                    .and_then(|status| status.rollout.as_ref())
                    .map(|rollout| rollout.rollout_id.clone())
                {
                    (
                        "diagnose rollout",
                        self.command(vec![
                            "pipelines".to_string(),
                            "diagnose".to_string(),
                            "rollout".to_string(),
                            group_id.clone(),
                            pipeline_id.clone(),
                            "--rollout-id".to_string(),
                            rollout_id,
                            "--logs-limit".to_string(),
                            self.command_context.logs_tail.to_string(),
                        ]),
                    )
                } else {
                    (
                        "diagnose shutdown",
                        self.command(vec![
                            "pipelines".to_string(),
                            "diagnose".to_string(),
                            "shutdown".to_string(),
                            group_id.clone(),
                            pipeline_id.clone(),
                            "--logs-limit".to_string(),
                            self.command_context.logs_tail.to_string(),
                        ]),
                    )
                };
                CommandRecipe {
                    title: "Equivalent CLI".to_string(),
                    description: format!("Run diagnosis for {group_id}/{pipeline_id}."),
                    commands: vec![command_line(label, command)],
                    note: None,
                }
            }
            PipelineTab::Bundle => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!(
                    "Export a troubleshooting bundle for {group_id}/{pipeline_id}."
                ),
                commands: vec![command_line(
                    "bundle",
                    self.command(vec![
                        "pipelines".to_string(),
                        "bundle".to_string(),
                        group_id.clone(),
                        pipeline_id.clone(),
                        "--logs-limit".to_string(),
                        self.command_context.logs_tail.to_string(),
                        "--metrics-shape".to_string(),
                        "compact".to_string(),
                        "--output".to_string(),
                        "json".to_string(),
                    ]),
                )],
                note: None,
            },
        }
    }

    fn group_command_recipe(&self) -> CommandRecipe {
        let Some(group_id) = self.selected_group_id() else {
            return empty_recipe(
                "Equivalent CLI",
                "Wait for the first refresh and select a group.",
            );
        };

        match self.group_tab {
            GroupTab::Summary => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!(
                    "Describe fleet-wide groups state while focusing on group {group_id}."
                ),
                commands: vec![command_line(
                    "describe groups",
                    self.command(vec!["groups".to_string(), "describe".to_string()]),
                )],
                note: Some(format!(
                    "The UI summary is scoped to group {group_id} client-side. The CLI command is fleet-wide."
                )),
            },
            GroupTab::Details => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Inspect detailed state for group {group_id}."),
                commands: vec![command_line(
                    "describe groups",
                    self.command(vec!["groups".to_string(), "describe".to_string()]),
                )],
                note: Some(format!(
                    "The Details tab is scoped to group {group_id} client-side. The CLI command is fleet-wide."
                )),
            },
            GroupTab::Events => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Watch normalized events for group {group_id}."),
                commands: vec![command_line(
                    "watch events",
                    self.command(vec![
                        "groups".to_string(),
                        "events".to_string(),
                        "watch".to_string(),
                        "--group".to_string(),
                        group_id.clone(),
                        "--watch-interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                    ]),
                )],
                note: None,
            },
            GroupTab::Logs => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Watch retained logs for group {group_id}."),
                commands: vec![command_line(
                    "watch logs",
                    self.command(vec![
                        "telemetry".to_string(),
                        "logs".to_string(),
                        "watch".to_string(),
                        "--tail".to_string(),
                        self.command_context.logs_tail.to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                        "--group".to_string(),
                        group_id.clone(),
                    ]),
                )],
                note: None,
            },
            GroupTab::Metrics => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Watch compact metrics for group {group_id}."),
                commands: vec![command_line(
                    "watch metrics",
                    self.command(vec![
                        "telemetry".to_string(),
                        "metrics".to_string(),
                        "watch".to_string(),
                        "--shape".to_string(),
                        "compact".to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                        "--group".to_string(),
                        group_id.clone(),
                    ]),
                )],
                note: None,
            },
            GroupTab::Shutdown => {
                let pipeline_keys = self.selected_group_pipeline_keys();
                let mut commands = Vec::new();
                for key in pipeline_keys {
                    if let Some((group, pipeline)) = split_pipeline_key(&key) {
                        commands.push(command_line(
                            &format!("shutdown {pipeline}"),
                            self.command(vec![
                                "pipelines".to_string(),
                                "shutdown".to_string(),
                                group.to_string(),
                                pipeline.to_string(),
                                "--watch".to_string(),
                            ]),
                        ));
                    }
                }
                CommandRecipe {
                    title: "Equivalent CLI".to_string(),
                    description: format!(
                        "Submit shutdowns for each pipeline currently visible in group {group_id}."
                    ),
                    commands,
                    note: Some(
                        "The UI implements group shutdown client-side because the current admin SDK does not expose a single group-scoped shutdown command."
                            .to_string(),
                    ),
                }
            }
            GroupTab::Diagnose => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!(
                    "Diagnose coordinated shutdown while focusing on group {group_id}."
                ),
                commands: vec![command_line(
                    "diagnose shutdown",
                    self.command(vec![
                        "groups".to_string(),
                        "diagnose".to_string(),
                        "shutdown".to_string(),
                        "--logs-limit".to_string(),
                        self.command_context.logs_tail.to_string(),
                    ]),
                )],
                note: Some(format!(
                    "The UI diagnosis is scoped to group {group_id} client-side. The CLI command is fleet-wide."
                )),
            },
            GroupTab::Bundle => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!(
                    "Export a fleet-wide troubleshooting bundle while focusing on group {group_id}."
                ),
                commands: vec![command_line(
                    "bundle",
                    self.command(vec![
                        "groups".to_string(),
                        "bundle".to_string(),
                        "--logs-limit".to_string(),
                        self.command_context.logs_tail.to_string(),
                        "--metrics-shape".to_string(),
                        "compact".to_string(),
                        "--output".to_string(),
                        "json".to_string(),
                    ]),
                )],
                note: Some(format!(
                    "The UI bundle preview is scoped to group {group_id} client-side. The CLI command is fleet-wide."
                )),
            },
        }
    }

    fn engine_command_recipe(&self) -> CommandRecipe {
        match self.engine_tab {
            EngineTab::Summary => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: "Inspect engine status and readiness.".to_string(),
                commands: vec![
                    command_line(
                        "status",
                        self.command(vec!["engine".to_string(), "status".to_string()]),
                    ),
                    command_line(
                        "livez",
                        self.command(vec!["engine".to_string(), "livez".to_string()]),
                    ),
                    command_line(
                        "readyz",
                        self.command(vec!["engine".to_string(), "readyz".to_string()]),
                    ),
                ],
                note: None,
            },
            EngineTab::Details => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: "Inspect detailed engine state and probes.".to_string(),
                commands: vec![
                    command_line(
                        "status",
                        self.command(vec!["engine".to_string(), "status".to_string()]),
                    ),
                    command_line(
                        "livez",
                        self.command(vec!["engine".to_string(), "livez".to_string()]),
                    ),
                    command_line(
                        "readyz",
                        self.command(vec!["engine".to_string(), "readyz".to_string()]),
                    ),
                    command_line(
                        "metrics",
                        self.command(vec![
                            "telemetry".to_string(),
                            "metrics".to_string(),
                            "get".to_string(),
                            "--shape".to_string(),
                            "compact".to_string(),
                        ]),
                    ),
                ],
                note: Some(
                    "The Details tab combines engine status, probes, and compact vitals client-side."
                        .to_string(),
                ),
            },
            EngineTab::Logs => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: "Watch retained engine logs.".to_string(),
                commands: vec![command_line(
                    "watch logs",
                    self.command(vec![
                        "telemetry".to_string(),
                        "logs".to_string(),
                        "watch".to_string(),
                        "--tail".to_string(),
                        self.command_context.logs_tail.to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                    ]),
                )],
                note: None,
            },
            EngineTab::Metrics => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: "Watch compact engine metrics.".to_string(),
                commands: vec![command_line(
                    "watch metrics",
                    self.command(vec![
                        "telemetry".to_string(),
                        "metrics".to_string(),
                        "watch".to_string(),
                        "--shape".to_string(),
                        "compact".to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                    ]),
                )],
                note: None,
            },
        }
    }

    fn command<S>(&self, args: impl IntoIterator<Item = S>) -> String
    where
        S: Into<String>,
    {
        let mut parts = self.command_context.prefix_args.clone();
        parts.extend(args.into_iter().map(Into::into));
        shell_join(parts)
    }

    /// Builds the context-sensitive action menu for the active selection.
    pub(super) fn build_action_menu(&self) -> Option<ActionMenuState> {
        match self.view {
            View::Pipelines => self.build_pipeline_action_menu(),
            View::Groups => self.build_group_action_menu(),
            View::Engine => None,
        }
    }

    fn build_pipeline_action_menu(&self) -> Option<ActionMenuState> {
        let (group_id, pipeline_id) = self.selected_pipeline_target()?;
        let mut entries = Vec::new();

        match self.selected_pipeline_scale_support() {
            Some(PipelineScaleSupport::Supported { current_cores }) => {
                entries.push(ActionMenuEntry {
                    label: "Scale up".to_string(),
                    detail: format!(
                        "Increase from {current_cores} to {} cores.",
                        current_cores + 1
                    ),
                    action: UiAction::PipelineScale {
                        group_id: group_id.clone(),
                        pipeline_id: pipeline_id.clone(),
                        target_cores: current_cores + 1,
                    },
                    enabled: true,
                });
                entries.push(ActionMenuEntry {
                    label: "Scale down".to_string(),
                    detail: if current_cores > 1 {
                        format!(
                            "Decrease from {current_cores} to {} cores.",
                            current_cores - 1
                        )
                    } else {
                        "Already at the minimum of 1 core.".to_string()
                    },
                    action: UiAction::PipelineScale {
                        group_id: group_id.clone(),
                        pipeline_id: pipeline_id.clone(),
                        target_cores: current_cores.saturating_sub(1),
                    },
                    enabled: current_cores > 1,
                });
                entries.push(ActionMenuEntry {
                    label: "Set core count…".to_string(),
                    detail: format!("Edit the explicit core_count starting from {current_cores}."),
                    action: UiAction::PipelineSetCoreCount {
                        group_id: group_id.clone(),
                        pipeline_id: pipeline_id.clone(),
                        current_cores,
                    },
                    enabled: true,
                });
            }
            Some(PipelineScaleSupport::Unsupported { reason }) => {
                for label in ["Scale up", "Scale down", "Set core count…"] {
                    entries.push(ActionMenuEntry {
                        label: label.to_string(),
                        detail: reason.clone(),
                        action: UiAction::PipelineSetCoreCount {
                            group_id: group_id.clone(),
                            pipeline_id: pipeline_id.clone(),
                            current_cores: 0,
                        },
                        enabled: false,
                    });
                }
            }
            None => {}
        }

        entries.push(ActionMenuEntry {
            label: "Edit and redeploy…".to_string(),
            detail: format!(
                "Open the current config for {group_id}/{pipeline_id} in an external editor."
            ),
            action: UiAction::PipelineEditAndRedeploy {
                group_id: group_id.clone(),
                pipeline_id: pipeline_id.clone(),
            },
            enabled: true,
        });

        entries.push(ActionMenuEntry {
            label: "Shutdown pipeline".to_string(),
            detail: format!("Submit shutdown for {group_id}/{pipeline_id}."),
            action: UiAction::PipelineShutdown {
                group_id: group_id.clone(),
                pipeline_id: pipeline_id.clone(),
            },
            enabled: true,
        });

        Some(ActionMenuState::new(
            "Pipeline Actions",
            format!("{group_id}/{pipeline_id}"),
            entries,
        ))
    }

    fn build_group_action_menu(&self) -> Option<ActionMenuState> {
        let group_id = self.selected_group_id()?;
        let pipelines = self.selected_group_pipeline_keys();
        if pipelines.is_empty() {
            return None;
        }

        Some(ActionMenuState::new(
            "Group Actions",
            group_id.clone(),
            vec![ActionMenuEntry {
                label: "Shutdown group".to_string(),
                detail: format!(
                    "Submit pipeline shutdowns for all {} pipelines currently visible in {group_id}.",
                    pipelines.len()
                ),
                action: UiAction::GroupShutdown {
                    group_id,
                    pipelines,
                },
                enabled: true,
            }],
        ))
    }
}
