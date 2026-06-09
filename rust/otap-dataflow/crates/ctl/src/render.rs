// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Human rendering and machine-output helpers for the CLI.
//!
//! Command modules should decide what data to fetch or mutate, not how that data
//! is shaped for people, scripts, or agents. This module gathers the rendering
//! adapters used by the CLI so human tables, event lines, structured JSON, NDJSON
//! streams, and bundle outputs stay consistent across command families.

#[path = "render/human.rs"]
mod human;
#[path = "render/output.rs"]
mod output;
#[path = "render/table.rs"]
mod table;

pub use human::{
    render_diagnosis, render_engine_probe, render_engine_status, render_event_line, render_events,
    render_group_shutdown_watch, render_groups_describe, render_groups_shutdown,
    render_groups_status, render_log_line, render_logs, render_metrics_compact,
    render_metrics_full, render_pipeline_describe, render_pipeline_details, render_pipeline_probe,
    render_pipeline_status, render_rollout_status, render_shutdown_status,
};
pub use output::{
    write_agent_output, write_bundle_output, write_event_output, write_human, write_log_event,
    write_mutation_output, write_read_output, write_stream_snapshot,
};

#[cfg(test)]
#[path = "render/tests.rs"]
mod tests;
