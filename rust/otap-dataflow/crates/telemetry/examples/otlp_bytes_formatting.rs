// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Example demonstrating the complete OTLP bytes encoding → decoding → formatting flow.
//!
//! This example shows:
//! 1. OtlpTracingLayer captures tracing events and encodes to OTLP bytes
//! 2. OTLP bytes are sent via channel to formatter
//! 3. OtlpBytesFormattingLayer decodes OTLP bytes and formats for human output
//!
//! This architecture:
//! - Removes dependency on opentelemetry crates for formatting
//! - Preserves complete structural fidelity (no data loss)
//! - Enables future async formatting in separate thread
//! - Supports colorized, customizable output

use otap_df_pdata::otlp::stateful_encoder::StatefulOtlpEncoder;
use otap_df_telemetry::tracing_integration::{OtlpTracingLayer, OtlpBytesFormattingLayer};
use std::sync::mpsc;
use std::thread;
use std::io;

fn main() {
    println!("=== OTLP Bytes Formatting Example ===\n");

    // Create channel for OTLP bytes
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(100);

    // Create OTLP encoding layer with stateful encoder
    // We need a new encoder for each invocation since it's stateful
    let resource_bytes = Vec::new(); // Empty resource for this example
    
    let tx_clone = tx.clone();
    let otlp_layer = OtlpTracingLayer::new(move |log_record| {
        // Create a new encoder for each log record
        let mut enc = StatefulOtlpEncoder::new(4096);
        let target = log_record.target();
        
        // Encode log record to OTLP bytes
        if let Err(e) = enc.encode_log_record(&log_record, &resource_bytes, target) {
            eprintln!("Failed to encode log record: {}", e);
            return;
        }
        
        // Flush encoder to get OTLP bytes
        let bytes = enc.flush();
        if let Err(e) = tx_clone.send(bytes.to_vec()) {
            eprintln!("Failed to send OTLP bytes: {}", e);
        }
    });

    // Create formatting layer for human-readable output
    let fmt_layer = OtlpBytesFormattingLayer::new(io::stdout)
        .with_ansi(true)
        .with_timestamp(false)  // Disable timestamp for cleaner example output
        .with_target(true);

    // Spawn formatter thread that processes OTLP bytes
    let formatter_thread = thread::spawn(move || {
        println!("Formatter thread started, waiting for OTLP bytes...\n");
        
        for otlp_bytes in rx {
            if let Err(e) = fmt_layer.format_otlp_bytes(&otlp_bytes) {
                eprintln!("Format error: {}", e);
            }
        }
        
        println!("\nFormatter thread finished");
    });

    // Install the OTLP tracing layer
    use tracing_subscriber::prelude::*;
    let subscriber = tracing_subscriber::registry().with(otlp_layer);
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set subscriber");

    println!("Starting to emit tracing events...\n");

    // Emit various tracing events with different data types
    tracing::info!(target: "app::server", port = 8080, host = "localhost", "Server starting");
    
    tracing::info!(
        target: "app::server",
        status = "ready",
        uptime_seconds = 5,
        "Server ready"
    );
    
    tracing::warn!(
        target: "app::database", 
        retry_count = 3,
        max_retries = 5,
        "Connection retry"
    );
    
    tracing::error!(
        target: "app::database",
        error = "connection timeout",
        timeout_ms = 5000,
        "Connection failed"
    );
    
    tracing::debug!(
        target: "app::cache",
        cache_size = 1024,
        hit_rate = 0.95,
        "Cache statistics"
    );

    // Give formatter thread time to process
    drop(tx);
    thread::sleep(std::time::Duration::from_millis(100));
    
    // Wait for formatter thread
    formatter_thread.join().expect("Formatter thread panicked");

    println!("\n=== Example Complete ===");
    println!("\nKey points:");
    println!("1. All events were encoded to OTLP bytes (preserving full structure)");
    println!("2. OTLP bytes were decoded and formatted for human output");
    println!("3. No opentelemetry SDK dependency required for formatting");
    println!("4. Full structural fidelity maintained through round-trip");
}
