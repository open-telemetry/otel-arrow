// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Example demonstrating tokio-tracing integration with OTLP encoding.
//!
//! This example shows how tracing events are:
//! 1. Captured by OtlpTracingLayer
//! 2. Converted to TracingLogRecord (implements LogRecordView)
//! 3. Encoded to OTLP format using StatefulOtlpEncoder
//! 4. Automatically batched by scope (using tracing target as InstrumentationScope.name)

use otap_df_pdata::otlp::stateful_encoder::StatefulOtlpEncoder;
use otap_df_telemetry::tracing_integration::OtlpTracingLayer;
use tracing_subscriber::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    // Create stateful encoder (wrapped in Arc<Mutex<>> for interior mutability)
    let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(64 * 1024)));
    let encoder_clone = encoder.clone();
    
    // Pre-encode resource once (empty for this example)
    let resource_bytes: Vec<u8> = vec![];

    // Create a layer that encodes tracing events to OTLP
    let layer = OtlpTracingLayer::new(move |log_record| {
        // Extract scope name from tracing metadata target (module path)
        let scope_name = log_record.target();
        
        // Encode the log record with automatic batching by scope
        // The encoder will:
        // - Compare scope names
        // - Close previous scope if different
        // - Open new scope with InstrumentationScope.name = scope_name
        // - Append log record to the current scope
        if let Ok(mut enc) = encoder_clone.lock() {
            if let Err(e) = enc.encode_log_record(&log_record, &resource_bytes, scope_name) {
                eprintln!("Failed to encode log record: {}", e);
            }
        }
    });

    // Install the tracing subscriber
    let subscriber = tracing_subscriber::registry().with(layer);
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set subscriber");

    // Emit some tracing events from different modules/scopes
    // Each will use its target (module path) as the InstrumentationScope name
    
    tracing::info!(target: "app::server", port = 8080, "Server starting");
    tracing::info!(target: "app::server", "Server ready");
    
    tracing::warn!(target: "app::database", retry_count = 3, "Connection retry");
    tracing::error!(target: "app::database", "Connection failed");
    
    tracing::debug!(target: "app::cache", hit_rate = 0.95, "Cache stats");
    
    // Different third-party library
    tracing::info!(target: "third_party::lib", version = "2.0", "Library initialized");

    // Flush and get the encoded OTLP bytes
    let otlp_bytes = encoder.lock().unwrap().flush();
    
    println!("Tracing events emitted and encoded to OTLP format");
    println!("Scope names used: app::server, app::database, app::cache, third_party::lib");
    println!("Each scope change triggers automatic batching in the encoder");
    println!("Total OTLP bytes generated: {} bytes", otlp_bytes.len());
}
