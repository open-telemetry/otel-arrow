// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! External-editor integration for live pipeline config editing.
//!
//! This module handles the disruptive parts of interactive configuration
//! editing: suspending the terminal UI, launching the user's editor with a YAML
//! file, parsing and diffing the result, then staging a deployable draft. Keeping
//! that flow isolated protects the main event loop from terminal-mode mistakes
//! and makes live reconfiguration behavior easier to review.

use super::*;
use crate::BIN_NAME;

/// Opens the selected pipeline config in an external editor and stages the edited draft.
pub(super) async fn stage_pipeline_editor_draft(
    client: &AdminClient,
    session: &mut TerminalSession,
    tx: &mpsc::Sender<UiEvent>,
    stop: &mut Arc<AtomicBool>,
    event_thread: &mut Option<thread::JoinHandle<()>>,
    app: &mut AppState,
    group_id: &str,
    pipeline_id: &str,
) -> Result<(), CliError> {
    let details = load_pipeline_details(client, app, group_id, pipeline_id).await?;
    let original_yaml = serialize_pipeline_config_yaml(&details.pipeline)?;
    let initial_yaml = app
        .pipelines
        .config_draft
        .as_ref()
        .map(|draft| draft.edited_yaml.clone())
        .unwrap_or_else(|| original_yaml.clone());
    let edited_content = edit_pipeline_config_external(
        session,
        tx,
        stop,
        event_thread,
        &initial_yaml,
        group_id,
        pipeline_id,
    )?;

    match parse_pipeline_config_content(&edited_content, group_id, pipeline_id) {
        Ok(parsed) => {
            let edited_yaml = serialize_pipeline_config_yaml(&parsed)?;
            app.stage_pipeline_config_draft(
                original_yaml,
                edited_yaml.clone(),
                Some(parsed),
                String::new(),
                None,
            );
            if let Some(draft) = app.pipelines.config_draft.as_mut() {
                draft.diff = build_yaml_diff(&draft.original_yaml, &draft.edited_yaml);
            }
        }
        Err(err) => {
            app.stage_pipeline_config_draft(
                original_yaml,
                edited_content,
                None,
                String::new(),
                Some(err.to_string()),
            );
        }
    }

    app.view = View::Pipelines;
    app.pipeline_selected = Some(format!("{group_id}:{pipeline_id}"));
    app.pipeline_tab = PipelineTab::Config;
    app.focus = FocusArea::Detail;
    app.reset_scroll();
    Ok(())
}

fn edit_pipeline_config_external(
    session: &mut TerminalSession,
    tx: &mpsc::Sender<UiEvent>,
    stop: &mut Arc<AtomicBool>,
    event_thread: &mut Option<thread::JoinHandle<()>>,
    initial_yaml: &str,
    group_id: &str,
    pipeline_id: &str,
) -> Result<String, CliError> {
    stop_event_thread(stop, event_thread);
    session.suspend()?;

    let edit_result = run_editor_command(initial_yaml, group_id, pipeline_id);
    let resume_result = session.resume().map_err(CliError::from);
    restart_event_thread(stop, event_thread, tx);
    resume_result?;
    edit_result
}

fn run_editor_command(
    initial_yaml: &str,
    group_id: &str,
    pipeline_id: &str,
) -> Result<String, CliError> {
    let editor = resolve_editor_command()?;
    let file_prefix = format!("{BIN_NAME}-pipeline-");
    let mut file = Builder::new()
        .prefix(&file_prefix)
        .suffix(".yaml")
        .tempfile()
        .map_err(|err| {
            CliError::config(format!(
                "failed to create temporary pipeline config file: {err}"
            ))
        })?;
    Write::write_all(&mut file, initial_yaml.as_bytes()).map_err(|err| {
        CliError::config(format!(
            "failed to write temporary pipeline config file: {err}"
        ))
    })?;
    file.flush().map_err(|err| {
        CliError::config(format!(
            "failed to flush temporary pipeline config file: {err}"
        ))
    })?;

    let program = &editor[0];
    let status = Command::new(program)
        .args(&editor[1..])
        .arg(file.path())
        .status()
        .map_err(|err| CliError::config(format!("failed to launch editor '{program}': {err}")))?;
    if !status.success() {
        return Err(CliError::config(format!(
            "editor '{program}' exited unsuccessfully while editing {group_id}/{pipeline_id}: {status}"
        )));
    }

    std::fs::read_to_string(file.path()).map_err(|err| {
        CliError::config(format!(
            "failed to read edited pipeline config for '{group_id}/{pipeline_id}': {err}"
        ))
    })
}

fn resolve_editor_command() -> Result<Vec<String>, CliError> {
    let value = env::var("VISUAL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("EDITOR")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "vi".to_string());
    split_editor_command(&value)
}

/// Splits a `$VISUAL` or `$EDITOR` command string into executable arguments.
pub(super) fn split_editor_command(command: &str) -> Result<Vec<String>, CliError> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escape = false;

    for ch in command.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        match quote {
            Some('\'') => {
                if ch == '\'' {
                    quote = None;
                } else {
                    current.push(ch);
                }
            }
            Some('"') => match ch {
                '"' => quote = None,
                '\\' => escape = true,
                _ => current.push(ch),
            },
            None => match ch {
                '\'' | '"' => quote = Some(ch),
                '\\' => escape = true,
                ch if ch.is_whitespace() => {
                    if !current.is_empty() {
                        parts.push(std::mem::take(&mut current));
                    }
                }
                _ => current.push(ch),
            },
            Some(other) => unreachable!("unsupported quote state {other}"),
        }
    }

    if escape || quote.is_some() {
        return Err(CliError::config(format!(
            "failed to parse editor command '{command}'"
        )));
    }
    if !current.is_empty() {
        parts.push(current);
    }
    if parts.is_empty() {
        return Err(CliError::config("the configured editor command is empty"));
    }
    Ok(parts)
}

/// Builds a small inline diff for the TUI config draft preview.
pub(super) fn build_yaml_diff(original: &str, edited: &str) -> String {
    if original == edited {
        return "No effective config changes were detected.".to_string();
    }

    let original_lines = original.lines().collect::<Vec<_>>();
    let edited_lines = edited.lines().collect::<Vec<_>>();
    let comparison_size = original_lines.len().saturating_mul(edited_lines.len());
    if comparison_size > 250_000 {
        return format!(
            "--- current\n+++ edited\nDiff omitted because the files are too large for the inline TUI diff ({} x {} lines).",
            original_lines.len(),
            edited_lines.len()
        );
    }

    let lcs = longest_common_subsequence_lengths(&original_lines, &edited_lines);

    let mut lines = vec!["--- current".to_string(), "+++ edited".to_string()];
    let (mut i, mut j) = (0usize, 0usize);
    while i < original_lines.len() && j < edited_lines.len() {
        if original_lines[i] == edited_lines[j] {
            lines.push(format!("  {}", original_lines[i]));
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            lines.push(format!("- {}", original_lines[i]));
            i += 1;
        } else {
            lines.push(format!("+ {}", edited_lines[j]));
            j += 1;
        }
    }
    while i < original_lines.len() {
        lines.push(format!("- {}", original_lines[i]));
        i += 1;
    }
    while j < edited_lines.len() {
        lines.push(format!("+ {}", edited_lines[j]));
        j += 1;
    }

    lines.join("\n")
}

fn longest_common_subsequence_lengths<'a>(left: &[&'a str], right: &[&'a str]) -> Vec<Vec<usize>> {
    let mut lcs = vec![vec![0usize; right.len() + 1]; left.len() + 1];
    for i in (0..left.len()).rev() {
        for j in (0..right.len()).rev() {
            lcs[i][j] = if left[i] == right[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    lcs
}
