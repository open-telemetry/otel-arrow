// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! # Internal Telemetry Receiver
//!
//! This receiver collects internal telemetry (traces, logs, metrics) from the OTAP-Dataflow
//! engine itself for self-diagnostics via the tracing integration layer.
//!
//! ## Architecture
//! ```text
//! tracing::info!(count = 42, error = ?e)
//!     ↓
//! OtlpTracingLayer (crates/telemetry/src/tracing_integration/subscriber.rs)
//!     ↓ Visit::record_i64(), record_debug(), etc. (captures full structure)
//!     ↓ Build TracingLogRecord with nested TracingAnyValue (Arrays, Maps)
//!     ↓ Encode to OTLP bytes using stateful encoder
//!     ↓ Send OTLP bytes to channel
//!     ↓
//! InternalTelemetryReceiver (this module)
//!     ↓ Receives OTLP bytes from channel
//!     ↓ Wraps as OtapPdata
//!     ↓ Injects into dataflow pipeline
//! ```
//!
//! ## Key Features
//! - **Structured Capture**: Full fidelity preservation of nested data via TracingAnyValue
//! - **OTLP Native**: Direct encoding to OTLP bytes without intermediate conversions
//! - **Pipeline Integration**: Routes internal telemetry through standard dataflow
//!
//! This receiver operates as a **shared** receiver (Send + Sync) to handle multithreaded
//! telemetry collection from across the application.

mod shared_receiver;

pub use shared_receiver::{Config, InternalTelemetryReceiver, get_otlp_bytes_sender};

use crate::OTAP_RECEIVER_FACTORIES;
use linkme::distributed_slice;
use otap_df_engine::ReceiverFactory;

/// URN for the internal telemetry receiver
pub const INTERNAL_TELEMETRY_RECEIVER_URN: &str = "urn:otap:receiver:internal-telemetry:v1";

/// Register the internal telemetry receiver factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static INTERNAL_TELEMETRY_RECEIVER_FACTORY: ReceiverFactory<crate::pdata::OtapPdata> =
    ReceiverFactory {
        name: INTERNAL_TELEMETRY_RECEIVER_URN,
        create: |_pipeline_ctx: otap_df_engine::context::PipelineContext,
                 node: otap_df_engine::node::NodeId,
                 node_config: std::sync::Arc<otap_df_config::node::NodeUserConfig>,
                 receiver_config: &otap_df_engine::config::ReceiverConfig| {
            let config: Config = serde_json::from_value(node_config.config.clone())
                .map_err(|e| otap_df_config::error::Error::InvalidConfiguration {
                    errors: vec![otap_df_config::error::Error::DeserializationError {
                        context: otap_df_config::error::Context::default(),
                        format: "JSON".to_string(),
                        details: format!("Failed to parse internal telemetry receiver config: {}", e),
                    }],
                })?;
            
            let receiver = InternalTelemetryReceiver::new(config);
            
            Ok(otap_df_engine::receiver::ReceiverWrapper::shared(
                receiver,
                node,
                node_config,
                receiver_config,
            ))
        },
    };
