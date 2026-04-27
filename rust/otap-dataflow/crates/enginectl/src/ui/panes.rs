// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pane assembly for pipeline, group, and engine detail views.
//!
//! Pane builders convert refreshed data and troubleshooting reports into
//! render-ready view models. They keep aggregation and presentation decisions,
//! such as which cards, tables, headers, and notes belong to each tab, separate
//! from the lower-level drawing routines in `ui::view`.

use super::*;

/// Build the default pipeline summary pane for the selected pipeline.
pub(super) fn build_pipeline_summary_pane(
    describe: &PipelineDescribeReport,
    header: DetailHeader,
) -> PipelineSummaryPane {
    PipelineSummaryPane {
        header: Some(header),
        stats: vec![
            card(
                "Running",
                format!(
                    "{}/{}",
                    describe.status.running_cores, describe.status.total_cores
                ),
                if describe.status.running_cores == describe.status.total_cores {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            card(
                "Generation",
                describe
                    .details
                    .active_generation
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                Tone::Accent,
            ),
            card(
                "Conditions",
                describe.status.conditions.len().to_string(),
                Tone::Muted,
            ),
            card(
                "Events",
                describe.recent_events.len().to_string(),
                if describe.recent_events.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Accent
                },
            ),
        ],
        conditions: condition_rows(&describe.status.conditions),
        cores: core_rows(&describe.status),
        events: event_rows(&describe.recent_events),
    }
}

/// Build the default group summary pane, including problem pipelines for quick triage.
pub(super) fn build_group_summary_pane(
    group_id: &str,
    describe: &GroupsDescribeReport,
    header: DetailHeader,
) -> GroupSummaryPane {
    let pipelines = pipeline_inventory_rows(&describe.status.pipelines, false);
    let problem_pipelines = pipelines
        .iter()
        .filter(|row| matches!(row.tone, Tone::Warning | Tone::Failure | Tone::Accent))
        .cloned()
        .collect::<Vec<_>>();

    GroupSummaryPane {
        header: Some(header),
        stats: vec![
            card(
                "Pipelines",
                describe.summary.total_pipelines.to_string(),
                Tone::Accent,
            ),
            card(
                "Running",
                describe.summary.running_pipelines.to_string(),
                Tone::Success,
            ),
            card(
                "Ready",
                describe.summary.ready_pipelines.to_string(),
                if describe.summary.ready_pipelines == describe.summary.total_pipelines {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            card(
                "Terminal",
                describe.summary.terminal_pipelines.to_string(),
                if describe.summary.terminal_pipelines == describe.summary.total_pipelines {
                    Tone::Muted
                } else {
                    Tone::Warning
                },
            ),
        ],
        problem_pipelines,
        pipelines,
        events: event_rows(
            &describe
                .recent_events
                .iter()
                .filter(|event| event.pipeline_group_id == group_id)
                .cloned()
                .collect::<Vec<_>>(),
        ),
    }
}

/// Build the default engine summary pane from global status and probe responses.
pub(super) fn build_engine_summary_pane(
    status: &engine::Status,
    livez: &engine::ProbeResponse,
    readyz: &engine::ProbeResponse,
    header: DetailHeader,
) -> EngineSummaryPane {
    let ready_pipelines = status
        .pipelines
        .values()
        .filter(|pipeline| pipeline_is_ready(pipeline))
        .count();
    let failing = readyz
        .failing
        .iter()
        .map(|failure| ProbeFailureRow {
            pipeline: failure.pipeline.clone(),
            condition: format!(
                "{:?}={:?}",
                failure.condition.kind, failure.condition.status
            )
            .to_ascii_lowercase(),
            message: failure.condition.message.clone().unwrap_or_default(),
            tone: Tone::Failure,
        })
        .collect::<Vec<_>>();

    EngineSummaryPane {
        header: Some(header),
        stats: vec![
            card(
                "Pipelines",
                status.pipelines.len().to_string(),
                Tone::Accent,
            ),
            card(
                "Ready",
                ready_pipelines.to_string(),
                probe_tone_engine(readyz.status),
            ),
            card(
                "Livez",
                format!("{:?}", livez.status).to_ascii_lowercase(),
                probe_tone_engine(livez.status),
            ),
            card(
                "Failing",
                failing.len().to_string(),
                if failing.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Failure
                },
            ),
        ],
        pipelines: pipeline_inventory_rows(&status.pipelines, true),
        failing,
    }
}

/// Build a metrics pane from a compact metrics response.
pub(super) fn build_metrics_pane(
    metrics: telemetry::CompactMetricsResponse,
    header: DetailHeader,
) -> MetricsPane {
    let rows = metric_rows(&metrics);
    MetricsPane {
        header: Some(add_header_chip(
            header,
            chip(
                "sets",
                metrics.metric_sets.len().to_string(),
                if metrics.metric_sets.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Accent
                },
            ),
        )),
        timestamp: Some(metrics.timestamp.clone()),
        rows,
        empty_message: "No compact metrics match the current scope.".to_string(),
    }
}

/// Build the rollout pane for the selected pipeline, or an empty pane when no rollout is active.
pub(super) fn build_rollout_pane(
    describe: &PipelineDescribeReport,
    rollout_status: Option<&pipelines::RolloutStatus>,
) -> OperationPane {
    let Some(status) = rollout_status else {
        return OperationPane {
            header: Some(add_header_chip(
                pipeline_header(describe),
                chip("rollout", "none", Tone::Muted),
            )),
            stats: Vec::new(),
            rows: Vec::new(),
            empty_message: "No active rollout for the selected pipeline.".to_string(),
        };
    };

    OperationPane {
        header: Some(DetailHeader {
            title: format!("Rollout {}", status.rollout_id),
            subtitle: Some(format!(
                "{}/{}",
                status.pipeline_group_id, status.pipeline_id
            )),
            chips: vec![
                chip(
                    "state",
                    format!("{:?}", status.state).to_ascii_lowercase(),
                    rollout_tone(status.state),
                ),
                chip("action", status.action.clone(), Tone::Accent),
                chip("target", status.target_generation.to_string(), Tone::Accent),
                chip(
                    "previous",
                    status
                        .previous_generation
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    Tone::Muted,
                ),
            ],
        }),
        stats: vec![
            card("Started", status.started_at.clone(), Tone::Muted),
            card("Updated", status.updated_at.clone(), Tone::Muted),
            card("Cores", status.cores.len().to_string(), Tone::Accent),
            card(
                "Failure",
                status
                    .failure_reason
                    .clone()
                    .unwrap_or_else(|| "none".to_string()),
                if status.failure_reason.is_some() {
                    Tone::Failure
                } else {
                    Tone::Muted
                },
            ),
        ],
        rows: status
            .cores
            .iter()
            .map(|core| OperationRow {
                core: core.core_id.to_string(),
                state: core.state.clone(),
                current_generation: core.target_generation.to_string(),
                previous_generation: core
                    .previous_generation
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                updated_at: core.updated_at.clone(),
                detail: core.detail.clone().unwrap_or_default(),
                tone: state_tone(&core.state),
            })
            .collect(),
        empty_message: "No rollout core state is available.".to_string(),
    }
}

/// Build the shutdown pane for the selected pipeline, or an empty pane when no shutdown is active.
pub(super) fn build_shutdown_pane(
    describe: &PipelineDescribeReport,
    shutdown_status: Option<&pipelines::ShutdownStatus>,
) -> OperationPane {
    let Some(status) = shutdown_status else {
        return OperationPane {
            header: Some(add_header_chip(
                pipeline_header(describe),
                chip("shutdown", "none", Tone::Muted),
            )),
            stats: Vec::new(),
            rows: Vec::new(),
            empty_message: "No active shutdown for the selected pipeline.".to_string(),
        };
    };

    OperationPane {
        header: Some(DetailHeader {
            title: format!("Shutdown {}", status.shutdown_id),
            subtitle: Some(format!(
                "{}/{}",
                status.pipeline_group_id, status.pipeline_id
            )),
            chips: vec![
                chip("state", status.state.clone(), state_tone(&status.state)),
                chip("cores", status.cores.len().to_string(), Tone::Accent),
            ],
        }),
        stats: vec![
            card("Started", status.started_at.clone(), Tone::Muted),
            card("Updated", status.updated_at.clone(), Tone::Muted),
            card(
                "Failure",
                status
                    .failure_reason
                    .clone()
                    .unwrap_or_else(|| "none".to_string()),
                if status.failure_reason.is_some() {
                    Tone::Failure
                } else {
                    Tone::Muted
                },
            ),
        ],
        rows: status
            .cores
            .iter()
            .map(|core| OperationRow {
                core: core.core_id.to_string(),
                state: core.state.clone(),
                current_generation: core.deployment_generation.to_string(),
                previous_generation: "n/a".to_string(),
                updated_at: core.updated_at.clone(),
                detail: core.detail.clone().unwrap_or_default(),
                tone: state_tone(&core.state),
            })
            .collect(),
        empty_message: "No shutdown core state is available.".to_string(),
    }
}

/// Build the client-side group shutdown tracking pane for the selected group.
pub(super) fn build_group_shutdown_pane(
    group_id: &str,
    status: &groups::Status,
    base_header: &DetailHeader,
    active_shutdown: Option<&app::ActiveGroupShutdown>,
) -> GroupShutdownPane {
    let tracker = active_shutdown.filter(|shutdown| shutdown.group_id == group_id);
    let snapshot = tracker.map(|shutdown| {
        group_shutdown_snapshot(
            groups::ShutdownStatus::Accepted,
            status,
            shutdown.started_at,
        )
    });

    let (state_label, state_tone, note) = match (tracker, snapshot.as_ref()) {
        (None, _) => (
            "idle".to_string(),
            Tone::Muted,
            Some(
                "No shutdown has been submitted from the UI for this group.".to_string(),
            ),
        ),
        (Some(shutdown), Some(_snapshot)) if !shutdown.submission_errors.is_empty() => (
            "failed".to_string(),
            Tone::Failure,
            Some(format!(
                "{} request errors: {}",
                shutdown.submission_errors.len(),
                shutdown.submission_errors.join(" | ")
            )),
        ),
        (Some(shutdown), Some(snapshot))
            if shutdown.started_at.elapsed().unwrap_or_default() >= shutdown.wait_timeout
                && !snapshot.all_terminal =>
        (
            "timed_out".to_string(),
            Tone::Failure,
            Some(format!(
                "The client-side group shutdown watch exceeded {}s.",
                shutdown.wait_timeout.as_secs()
            )),
        ),
        (_, Some(snapshot)) if snapshot.all_terminal => (
            "completed".to_string(),
            Tone::Success,
            Some(
                "Group shutdown in the UI is implemented client-side by submitting one pipeline shutdown per pipeline."
                    .to_string(),
            ),
        ),
        (Some(_), Some(_)) => (
            "running".to_string(),
            Tone::Warning,
            Some(
                "Group shutdown in the UI is implemented client-side by submitting one pipeline shutdown per pipeline."
                    .to_string(),
            ),
        ),
        _ => (
            "unknown".to_string(),
            Tone::Muted,
            Some("No current shutdown snapshot is available.".to_string()),
        ),
    };

    let mut header = base_header.clone();
    header.title = group_id.to_string();
    header.subtitle = Some("Group Shutdown".to_string());
    header.chips.push(chip("state", state_label, state_tone));
    if let Some(snapshot) = snapshot.as_ref() {
        header.chips.push(chip(
            "terminal",
            snapshot.terminal_pipelines.to_string(),
            Tone::Accent,
        ));
    }

    let rows = snapshot
        .as_ref()
        .map(|snapshot| {
            snapshot
                .pipelines
                .iter()
                .map(|pipeline| GroupShutdownRow {
                    pipeline: pipeline
                        .pipeline
                        .split_once(':')
                        .map_or_else(|| pipeline.pipeline.clone(), |(_, value)| value.to_string()),
                    running: format!("{}/{}", pipeline.running_cores, pipeline.total_cores),
                    terminal: bool_label(pipeline.terminal),
                    phases: pipeline.phases.join(", "),
                    tone: group_shutdown_tone(pipeline),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let stats = if let (Some(tracker), Some(snapshot)) = (tracker, snapshot.as_ref()) {
        vec![
            card(
                "Pipelines",
                tracker.pipeline_count.to_string(),
                Tone::Accent,
            ),
            card(
                "Terminal",
                format!(
                    "{}/{}",
                    snapshot.terminal_pipelines, snapshot.total_pipelines
                ),
                if snapshot.all_terminal {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            card("Elapsed", format!("{}ms", snapshot.elapsed_ms), Tone::Muted),
            card("Requests", tracker.request_count.to_string(), Tone::Muted),
        ]
    } else {
        Vec::new()
    };

    GroupShutdownPane {
        header: Some(header),
        stats,
        rows,
        empty_message: "No shutdown data is available for the selected group.".to_string(),
        note,
    }
}

/// Build a diagnosis pane by flattening findings and evidence into separate table models.
pub(super) fn build_diagnosis_pane(
    diagnosis: DiagnosisReport,
    header: DetailHeader,
) -> DiagnosisPane {
    let mut evidence = Vec::<EvidenceRow>::new();
    for finding in &diagnosis.findings {
        evidence.extend(finding.evidence.iter().map(evidence_row));
    }

    DiagnosisPane {
        header: Some(add_header_chip(
            header,
            chip(
                "status",
                format!("{:?}", diagnosis.status).to_ascii_lowercase(),
                diagnosis_tone(diagnosis.status),
            ),
        )),
        summary: diagnosis.summary.clone(),
        findings: diagnosis.findings.iter().map(finding_row).collect(),
        evidence,
        next_steps: diagnosis.recommended_next_steps,
    }
}

/// Build the pipeline configuration pane, choosing committed YAML, invalid draft, or deployable diff.
pub(super) fn build_config_pane(
    header: &DetailHeader,
    pipeline_group_id: &str,
    pipeline_id: &str,
    committed_yaml: &str,
    draft: Option<&app::PipelineConfigDraft>,
) -> ConfigPane {
    let mut preview_title = "Committed YAML".to_string();
    let mut preview = committed_yaml.to_string();
    let mut note = Some(
        "Press 'e' to edit in an external editor. A valid staged draft can be deployed with Enter. Press 'd' to discard a staged draft."
            .to_string(),
    );
    let mut stats = vec![
        card("Source", "committed".to_string(), Tone::Muted),
        card(
            "Lines",
            committed_yaml.lines().count().to_string(),
            Tone::Muted,
        ),
    ];

    if let Some(draft) = draft {
        if let Some(error) = &draft.error {
            preview_title = "Edited YAML".to_string();
            preview = draft.edited_yaml.clone();
            note = Some(format!(
                "The edited config for {pipeline_group_id}/{pipeline_id} could not be parsed: {error}"
            ));
            stats = vec![
                card("Source", "edited".to_string(), Tone::Warning),
                card("Status", "invalid".to_string(), Tone::Failure),
                card(
                    "Lines",
                    draft.edited_yaml.lines().count().to_string(),
                    Tone::Muted,
                ),
            ];
        } else if draft.is_deployable() {
            preview_title = "Canonical Diff".to_string();
            preview = draft.diff.clone();
            note = Some(
                "Review the staged config diff, press Enter to redeploy it, 'e' to reopen the editor, or 'd' to discard the draft."
                    .to_string(),
            );
            stats = vec![
                card("Source", "staged".to_string(), Tone::Accent),
                card("Status", "deployable".to_string(), Tone::Success),
                card(
                    "Edited lines",
                    draft.edited_yaml.lines().count().to_string(),
                    Tone::Muted,
                ),
            ];
        } else {
            preview_title = "Committed YAML".to_string();
            preview = committed_yaml.to_string();
            note = Some(
                "The edited config parsed successfully, but it does not change the canonical pipeline config."
                    .to_string(),
            );
            stats = vec![
                card("Source", "staged".to_string(), Tone::Accent),
                card("Status", "no-op".to_string(), Tone::Warning),
                card(
                    "Edited lines",
                    draft.edited_yaml.lines().count().to_string(),
                    Tone::Muted,
                ),
            ];
        }
    }

    ConfigPane {
        header: Some(add_header_chip(
            header.clone(),
            chip(
                "config",
                if draft.is_some() {
                    "draft"
                } else {
                    "committed"
                },
                if draft.as_ref().is_some_and(|draft| draft.is_deployable()) {
                    Tone::Accent
                } else {
                    Tone::Muted
                },
            ),
        )),
        stats,
        note,
        preview_title,
        preview,
    }
}

/// Build the support-bundle pane from collected metadata and a human preview.
pub(super) fn build_bundle_pane(
    title: &str,
    subtitle: Option<String>,
    metadata: &crate::troubleshoot::BundleMetadata,
    preview: String,
) -> BundlePane {
    BundlePane {
        header: Some(DetailHeader {
            title: title.to_string(),
            subtitle,
            chips: vec![
                chip(
                    "shape",
                    format!("{:?}", metadata.metrics_shape).to_ascii_lowercase(),
                    Tone::Accent,
                ),
                chip("logs", metadata.logs_limit.to_string(), Tone::Muted),
            ],
        }),
        stats: vec![
            card("Collected", metadata.collected_at.clone(), Tone::Muted),
            card("Preview bytes", preview.len().to_string(), Tone::Accent),
        ],
        preview,
    }
}

/// Build the standard pipeline detail header shared by pipeline panes.
pub(super) fn pipeline_header(describe: &PipelineDescribeReport) -> DetailHeader {
    DetailHeader {
        title: format!(
            "{}/{}",
            describe.details.pipeline_group_id, describe.details.pipeline_id
        ),
        subtitle: Some("Pipeline".to_string()),
        chips: vec![
            chip(
                "live",
                format!("{:?}", describe.livez.status).to_ascii_lowercase(),
                probe_tone(describe.livez.status),
            ),
            chip(
                "ready",
                format!("{:?}", describe.readyz.status).to_ascii_lowercase(),
                probe_tone(describe.readyz.status),
            ),
            chip(
                "running",
                format!(
                    "{}/{}",
                    describe.status.running_cores, describe.status.total_cores
                ),
                if describe.status.running_cores == describe.status.total_cores {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            chip(
                "generation",
                describe
                    .status
                    .active_generation
                    .or(describe.details.active_generation)
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                Tone::Accent,
            ),
            chip(
                "rollout",
                describe
                    .status
                    .rollout
                    .as_ref()
                    .map(|value| format!("{:?}", value.state).to_ascii_lowercase())
                    .unwrap_or_else(|| "none".to_string()),
                describe
                    .status
                    .rollout
                    .as_ref()
                    .map_or(Tone::Muted, |value| rollout_tone(value.state)),
            ),
        ],
    }
}

/// Build the standard group detail header shared by group panes.
pub(super) fn group_header(group_id: &str, describe: &GroupsDescribeReport) -> DetailHeader {
    DetailHeader {
        title: group_id.to_string(),
        subtitle: Some("Group".to_string()),
        chips: vec![
            chip(
                "pipelines",
                describe.summary.total_pipelines.to_string(),
                Tone::Accent,
            ),
            chip(
                "running",
                describe.summary.running_pipelines.to_string(),
                Tone::Success,
            ),
            chip(
                "ready",
                describe.summary.ready_pipelines.to_string(),
                if describe.summary.ready_pipelines == describe.summary.total_pipelines {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            chip(
                "terminal",
                describe.summary.terminal_pipelines.to_string(),
                if describe.summary.terminal_pipelines == describe.summary.total_pipelines {
                    Tone::Muted
                } else {
                    Tone::Warning
                },
            ),
        ],
    }
}

/// Build the standard engine detail header shared by engine panes.
pub(super) fn engine_header(
    status: &engine::Status,
    livez: &engine::ProbeResponse,
    readyz: &engine::ProbeResponse,
) -> DetailHeader {
    DetailHeader {
        title: "Engine".to_string(),
        subtitle: Some(status.generated_at.clone()),
        chips: vec![
            chip(
                "livez",
                format!("{:?}", livez.status).to_ascii_lowercase(),
                probe_tone_engine(livez.status),
            ),
            chip(
                "readyz",
                format!("{:?}", readyz.status).to_ascii_lowercase(),
                probe_tone_engine(readyz.status),
            ),
            chip(
                "pipelines",
                status.pipelines.len().to_string(),
                Tone::Accent,
            ),
            chip(
                "failing",
                readyz.failing.len().to_string(),
                if readyz.failing.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Failure
                },
            ),
        ],
    }
}

/// Append a status chip to an existing detail header.
pub(super) fn add_header_chip(mut header: DetailHeader, chip: StatusChip) -> DetailHeader {
    header.chips.push(chip);
    header
}

/// Construct a status chip from display parts.
pub(super) fn chip(label: impl Into<String>, value: impl Into<String>, tone: Tone) -> StatusChip {
    StatusChip {
        label: label.into(),
        value: value.into(),
        tone,
    }
}

/// Construct a summary card from display parts.
pub(super) fn card(label: impl Into<String>, value: impl Into<String>, tone: Tone) -> StatCard {
    StatCard {
        label: label.into(),
        value: value.into(),
        tone,
    }
}
