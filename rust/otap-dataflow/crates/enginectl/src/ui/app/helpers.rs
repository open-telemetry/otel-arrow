// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared selection and formatting helpers for the app state.

use super::*;

impl AppState {
    pub(crate) fn selected_pipeline_scale_support(&self) -> Option<PipelineScaleSupport> {
        let describe = self.pipelines.describe.as_ref()?;
        let Some(policies) = describe.details.pipeline.policies() else {
            return Some(PipelineScaleSupport::Unsupported {
                reason: "Scaling in the TUI requires an explicit pipeline resources.core_allocation policy.".to_string(),
            });
        };
        let Some(resources) = policies.resources() else {
            return Some(PipelineScaleSupport::Unsupported {
                reason: "Scaling in the TUI requires an explicit pipeline resources.core_allocation policy.".to_string(),
            });
        };
        match &resources.core_allocation {
            CoreAllocation {
                strategy: CoreAllocationStrategy::CoreCount,
                count: Some(count),
                ..
            } => {
                if *count > 0 {
                    Some(PipelineScaleSupport::Supported {
                        current_cores: *count,
                    })
                } else {
                    Some(PipelineScaleSupport::Unsupported {
                        reason:
                            "Scaling in the TUI is disabled for core_count=0 because that means all available cores."
                                .to_string(),
                    })
                }
            }
            CoreAllocation {
                strategy: CoreAllocationStrategy::AllCores,
                ..
            } => Some(PipelineScaleSupport::Unsupported {
                reason:
                    "Scaling in the TUI is disabled for all_cores allocations. Use a full reconfigure instead."
                        .to_string(),
            }),
            CoreAllocation {
                strategy: CoreAllocationStrategy::CoreSet,
                ..
            } => Some(PipelineScaleSupport::Unsupported {
                reason:
                    "Scaling in the TUI is disabled for core_set allocations. Use a full reconfigure instead."
                        .to_string(),
            }),
            CoreAllocation {
                strategy: CoreAllocationStrategy::CoreCount,
                count: None,
                ..
            } => Some(PipelineScaleSupport::Unsupported {
                reason:
                    "Scaling in the TUI is disabled because the pipeline resources.core_allocation policy is invalid."
                        .to_string(),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PipelineScaleSupport {
    Supported { current_cores: usize },
    Unsupported { reason: String },
}

impl ActionMenuState {
    pub(super) fn new(
        title: impl Into<String>,
        target: impl Into<String>,
        entries: Vec<ActionMenuEntry>,
    ) -> Self {
        let selected = entries.iter().position(|entry| entry.enabled).unwrap_or(0);
        Self {
            title: title.into(),
            target: target.into(),
            entries,
            selected,
        }
    }
}

pub(super) fn empty_recipe(title: &str, description: &str) -> CommandRecipe {
    CommandRecipe {
        title: title.to_string(),
        description: description.to_string(),
        commands: Vec::new(),
        note: None,
    }
}

pub(super) fn command_line(label: &str, command: String) -> CommandLine {
    CommandLine {
        label: label.to_string(),
        command,
    }
}

pub(super) fn format_duration_arg(duration: Duration) -> String {
    format_duration(duration).to_string()
}

pub(super) fn shell_join(parts: Vec<String>) -> String {
    parts
        .into_iter()
        .map(|part| shell_quote(&part))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }

    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | ':' | '.' | '_' | '-' | '='))
    {
        return value.to_string();
    }

    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub(super) fn ensure_selected_key<T, F>(
    selected: Option<String>,
    items: &[T],
    map: F,
) -> Option<String>
where
    F: Fn(&T) -> String,
{
    if items.is_empty() {
        return None;
    }
    if let Some(selected) = selected {
        if items.iter().any(|item| map(item) == selected) {
            return Some(selected);
        }
    }
    Some(map(&items[0]))
}

pub(super) fn move_key_selection(selection: &mut Option<String>, keys: &[String], delta: isize) {
    if keys.is_empty() {
        *selection = None;
        return;
    }

    let current = selection
        .as_ref()
        .and_then(|selected| keys.iter().position(|key| key == selected))
        .unwrap_or(0);
    let next = wrap_index(current, keys.len(), delta);
    *selection = Some(keys[next].clone());
}

pub(super) fn select_key_edge(selection: &mut Option<String>, keys: &[String], end: bool) {
    if keys.is_empty() {
        *selection = None;
        return;
    }
    *selection = Some(if end {
        keys[keys.len() - 1].clone()
    } else {
        keys[0].clone()
    });
}

pub(super) fn update_selection(selection: &mut Option<String>, key: &str) -> bool {
    if selection.as_deref() == Some(key) {
        false
    } else {
        *selection = Some(key.to_string());
        true
    }
}

pub(super) fn wrap_index(current: usize, len: usize, delta: isize) -> usize {
    let len = len as isize;
    (((current as isize) + delta).rem_euclid(len)) as usize
}

pub(super) fn split_pipeline_key(key: &str) -> Option<(&str, &str)> {
    key.split_once(':')
}

pub(super) fn classify_pipeline_row(status: &pipelines::Status) -> (String, Tone) {
    if has_active_rollout(status) {
        return ("roll".to_string(), Tone::Accent);
    }
    let has_failure = status.cores.values().any(|core| {
        matches!(
            core.phase,
            pipelines::Phase::Failed(_) | pipelines::Phase::Rejected(_)
        )
    }) || status.instances.as_ref().is_some_and(|instances| {
        instances.iter().any(|instance| {
            matches!(
                instance.status.phase,
                pipelines::Phase::Failed(_) | pipelines::Phase::Rejected(_)
            )
        })
    });
    if has_failure {
        return ("fail".to_string(), Tone::Failure);
    }
    if pipeline_is_terminal(status) {
        return ("stop".to_string(), Tone::Muted);
    }
    if !pipeline_is_ready(status) || status.running_cores < status.total_cores {
        return ("warn".to_string(), Tone::Warning);
    }
    ("ok".to_string(), Tone::Success)
}

pub(super) fn combine_tones(left: Tone, right: Tone) -> Tone {
    use Tone::{Accent, Failure, Muted, Neutral, Success, Warning};
    match (left, right) {
        (Failure, _) | (_, Failure) => Failure,
        (Warning, _) | (_, Warning) => Warning,
        (Accent, _) | (_, Accent) => Accent,
        (Success, _) | (_, Success) => Success,
        (Muted, _) | (_, Muted) => Muted,
        _ => Neutral,
    }
}

pub(super) fn tone_badge(tone: Tone) -> &'static str {
    match tone {
        Tone::Accent => "roll",
        Tone::Success => "ok",
        Tone::Warning => "warn",
        Tone::Failure => "fail",
        Tone::Muted => "stop",
        Tone::Neutral => "info",
    }
}

pub(super) fn pipeline_is_ready(status: &pipelines::Status) -> bool {
    status.conditions.iter().any(|condition| {
        condition.kind == pipelines::ConditionKind::Ready
            && condition.status == pipelines::ConditionStatus::True
    })
}

pub(super) fn pipeline_is_terminal(status: &pipelines::Status) -> bool {
    let phases_terminal = if let Some(instances) = &status.instances {
        !instances.is_empty()
            && instances.iter().all(|instance| {
                matches!(
                    instance.status.phase,
                    pipelines::Phase::Stopped
                        | pipelines::Phase::Deleted
                        | pipelines::Phase::Failed(_)
                        | pipelines::Phase::Rejected(_)
                )
            })
    } else {
        !status.cores.is_empty()
            && status.cores.values().all(|core| {
                matches!(
                    core.phase,
                    pipelines::Phase::Stopped
                        | pipelines::Phase::Deleted
                        | pipelines::Phase::Failed(_)
                        | pipelines::Phase::Rejected(_)
                )
            })
    };
    phases_terminal && status.running_cores == 0
}

pub(super) fn has_active_rollout(status: &pipelines::Status) -> bool {
    status
        .rollout
        .as_ref()
        .is_some_and(|rollout| !rollout_is_terminal(rollout.state))
}

pub(super) fn active_rollout_id(status: &pipelines::Status) -> Option<String> {
    status
        .rollout
        .as_ref()
        .filter(|rollout| !rollout_is_terminal(rollout.state))
        .map(|rollout| rollout.rollout_id.clone())
}

pub(super) fn rollout_is_terminal(state: pipelines::PipelineRolloutState) -> bool {
    matches!(
        state,
        pipelines::PipelineRolloutState::Succeeded
            | pipelines::PipelineRolloutState::Failed
            | pipelines::PipelineRolloutState::RollbackFailed
    )
}
