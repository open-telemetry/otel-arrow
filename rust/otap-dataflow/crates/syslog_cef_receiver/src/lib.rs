// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]


use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;
use otap_df_otap::pdata::OtapPdata;
// Syslog CEF receiver implementation

/// Syslog CEF receiver module
pub mod syslog_cef_receiver;

/// Parser module for syslog message parsing
pub mod parser;

/// Arrow records encoder for syslog messages
pub mod arrow_records_encoder;


/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OtapPdata)]
pub static OTAP_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();
