// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Contrib nodes (receiver, exporter, processor).

use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;
use otap_df_otap::pdata::OtapPdata;

/// Exporter implementations for contrib nodes.
pub mod exporters;

/// Processor implementations for contrib nodes.
pub mod processors;

/// Factory for Contrib-based pipeline components
#[pipeline_factory(CONTRIB, OtapPdata)]
pub static CONTRIB_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();