// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! TUI application state, models, and CLI recipe generation helpers.
//!
//! The app module contains the state machine that sits between terminal events,
//! admin API refreshes, and renderable panes. Its submodules separate raw data
//! models, selection/focus behavior, command-palette behavior, helper
//! formatting, and equivalent CLI recipe generation so interactive features can
//! evolve without coupling them to terminal drawing code.

#[path = "app/helpers.rs"]
mod helpers;
#[path = "app/model.rs"]
mod model;
#[path = "app/palette.rs"]
mod palette;
#[path = "app/recipes.rs"]
mod recipes;
#[path = "app/state.rs"]
mod state;

pub(crate) use model::*;

use crate::args::UiStartView;
use crate::troubleshoot::PipelineDescribeReport;
use humantime::format_duration;
use otap_df_admin_api::{engine, groups, pipelines, telemetry};
use otap_df_config::pipeline::PipelineConfig;
use otap_df_config::policy::{CoreAllocation, CoreAllocationStrategy};
use std::collections::BTreeMap;
use std::time::Duration;

use self::helpers::*;

#[cfg(test)]
#[path = "app/tests.rs"]
mod tests;
