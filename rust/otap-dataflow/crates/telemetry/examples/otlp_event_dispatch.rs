// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Example demonstrating OTLP bytes dispatched through the configured tracing subscriber.
//!
//! This example shows how OTLP bytes are decoded and dispatched as regular tracing events,
//! using whatever fmt layer configuration the application has set up. Users see no difference
//! in the console output - it looks exactly like regular tracing! logs.
//!
//! Architecture:
//! 1. Configure tracing_subscriber::fmt once (with colors, timestamps, etc.)
//! 2. Capture regular tracing events → encode to OTLP bytes (internal transport)
//! 3. Receive OTLP bytes → dispatch as tracing events (through configured fmt layer)
//! 4. Console output looks identical to regular tracing
//!
//! Run with:
//! ```sh
//! cargo run --example otlp_event_dispatch
//! ```

use otap_df_telemetry::tracing_integration::{
    OtlpTracingLayer,
    dispatch_otlp_bytes_as_events,
};
use otap_df_pdata::otlp::stateful_encoder::StatefulOtlpEncoder;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use tracing::{info, warn, error, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    println!("=== OTLP Event Dispatch Example ===\n");
    println!("This example shows OTLP bytes being dispatched through the configured fmt layer.");
    println!("Notice: The output looks EXACTLY like regular tracing! logs.\n");

    // Step 1: Create encoder and channel for OTLP bytes
    let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(4096)));
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    
    // Create the OTLP layer with encoder callback
    let encoder_clone = encoder.clone();
    let tx_clone = tx.clone();
    let otlp_layer = OtlpTracingLayer::new(move |log_record| {
        let mut enc = encoder_clone.lock().unwrap();
        let resource_bytes = Vec::new();
        
        // Encode the log record to OTLP bytes
        if let Ok(_) = enc.encode_log_record(&log_record, &resource_bytes, "otap::internal") {
            // Flush and send the bytes
            let otlp_bytes = enc.flush();
            let _ = tx_clone.send(otlp_bytes.to_vec());
        }
    });
    
    // Step 2: Configure tracing with both layers
    tracing_subscriber::registry()
        .with(otlp_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_level(true)
                .with_target(true)
                .with_thread_ids(false)
                .compact()
        )
        .with(tracing_subscriber::filter::LevelFilter::from_level(Level::INFO))
        .init();

    println!("✓ Configured tracing with fmt layer (colors, compact format)\n");

    // Step 3: Spawn a thread to receive OTLP bytes and dispatch them
    let dispatcher = thread::spawn(move || {
        println!("✓ Started dispatcher thread (receiving OTLP bytes)\n");
        
        for otlp_bytes in rx {
            // This is the magic: decode OTLP bytes and dispatch as tracing events
            // The configured fmt layer will format them exactly like regular tracing! logs
            if let Err(e) = dispatch_otlp_bytes_as_events(&otlp_bytes) {
                eprintln!("Failed to dispatch OTLP bytes: {:?}", e);
            }
        }
    });

    // Give the dispatcher time to start
    thread::sleep(Duration::from_millis(100));
    
    // Clean up the extra tx since we cloned it
    drop(tx);

    println!("--- Regular tracing events (captured → OTLP → dispatched → fmt) ---\n");

    // Step 3: Emit regular tracing events
    // These will be:
    // 1. Captured by OtlpTracingLayer
    // 2. Encoded to OTLP bytes
    // 3. Sent through channel
    // 4. Decoded by dispatcher
    // 5. Dispatched as tracing events
    // 6. Formatted by fmt layer (looks identical!)
    
    info!("Application started");
    
    info!(count = 42, name = "test", "Processing items");
    
    warn!(remaining = 10, "Low on resources");
    
    info!(
        request_id = "req-123",
        user = "alice",
        duration_ms = 125,
        "Request completed"
    );
    
    error!(error = "connection refused", retry_count = 3, "Failed to connect");
    
    info!("Application finished");

    println!("\n--- Done ---\n");
    println!("Notice: All events above were transported as OTLP bytes internally,");
    println!("but formatted using the configured fmt layer - no difference for users!");

    // Clean shutdown
    drop(dispatcher);
}
