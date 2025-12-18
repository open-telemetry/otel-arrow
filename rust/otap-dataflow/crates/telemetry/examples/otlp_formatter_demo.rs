// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Example demonstrating the configurable OtlpBytesFormattingLayer.
//!
//! This shows how to format OTLP bytes with tokio-like output, with options to:
//! - Enable/disable ANSI colors
//! - Enable/disable timestamps
//! - Enable/disable levels
//! - Enable/disable target (scope name)
//! - Enable/disable event_name
//!
//! Run with:
//! ```sh
//! cargo run --example otlp_formatter_demo -p otap-df-telemetry
//! ```

use otap_df_telemetry::tracing_integration::OtlpBytesFormattingLayer;
use otap_df_pdata::otlp::stateful_encoder::StatefulOtlpEncoder;
use std::sync::{Arc, Mutex};

fn main() {
    println!("=== OTLP Formatter Demo ===\n");

    // Demo 1: Full tokio-like format (default)
    println!("--- Demo 1: Full tokio-like format (with colors) ---");
    demo_with_config(true, true, true, true, true);

    println!("\n--- Demo 2: No colors ---");
    demo_with_config(false, true, true, true, true);

    println!("\n--- Demo 3: No timestamps ---");
    demo_with_config(true, false, true, true, true);

    println!("\n--- Demo 4: Minimal (message only) ---");
    demo_with_config(false, false, false, false, false);

    println!("\n--- Demo 5: No event names ---");
    demo_with_config(true, true, true, true, false);
}

fn demo_with_config(
    with_ansi: bool,
    with_timestamp: bool,
    with_level: bool,
    with_target: bool,
    with_event_name: bool,
) {
    // Create encoder
    let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(4096)));
    
    // Collect OTLP bytes
    let mut otlp_bytes_vec = Vec::new();
    
    // Actually emit tracing events and capture them
    use tracing_subscriber::prelude::*;
    use otap_df_telemetry::tracing_integration::OtlpTracingLayer;
    
    let encoder_clone = encoder.clone();
    let layer = OtlpTracingLayer::new(move |log_record| {
        let mut enc = encoder_clone.lock().unwrap();
        let resource_bytes = Vec::new();
        let _ = enc.encode_log_record(&log_record, &resource_bytes, "demo");
    });
    
    let subscriber = tracing_subscriber::registry().with(layer);
    
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(target: "app::server", port = 8080, "Server started");
        tracing::info!(target: "app::database", host = "localhost", "Connected to database");
        tracing::info!(target: "app::cache", key = 42, ttl = 300, "Cache hit");
    });
    
    // Collect the encoded bytes
    let mut enc = encoder.lock().unwrap();
    let bytes = enc.flush();
    if !bytes.is_empty() {
        otlp_bytes_vec.push(bytes.to_vec());
    }
    
    // Create formatter with specified config
    let formatter = OtlpBytesFormattingLayer::new(std::io::stdout)
        .with_ansi(with_ansi)
        .with_timestamp(with_timestamp)
        .with_level(with_level)
        .with_target(with_target)
        .with_event_name(with_event_name);
    
    // Format all the bytes
    for bytes in otlp_bytes_vec {
        let _ = formatter.format_otlp_bytes(&bytes);
    }
}
