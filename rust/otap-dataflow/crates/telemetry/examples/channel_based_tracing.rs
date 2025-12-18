// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Example demonstrating channel-based tracing for normal operation.
//!
//! This shows how to:
//! 1. Initialize channel-based tracing (returns receiver)
//! 2. Spawn formatter task in a runtime (simulating admin runtime)
//! 3. Generate some tracing events from multiple threads
//! 4. Single-threaded formatting in the admin runtime
//!
//! Run with:
//! ```bash
//! cargo run --example channel_based_tracing
//! ```

use otap_df_telemetry::opentelemetry_client::logger_provider::LoggerProvider;
use std::time::Duration;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() {
    println!("=== Channel-Based Tracing Example ===\n");
    
    // In real usage: First initialize synchronous console logging for startup
    // LoggerProvider::init_default_console_tracing();
    // ... perform early initialization ...
    
    // Then transition to channel-based logging once admin runtime is ready
    // The transition message will be logged and you'll see it in the output
    println!("Step 1: Transitioning to channel-based logging...");
    let channel = LoggerProvider::init_default_channel_based_tracing(1000);
    let receiver = channel.into_receiver();
    
    println!("Step 2: Spawning formatter task in admin runtime...\n");
    
    // Spawn the formatter task in the "admin" runtime
    // In real code, this would be spawned in the actual admin runtime
    let formatter_task = tokio::spawn(async move {
        LoggerProvider::run_console_formatter_task(receiver).await;
        println!("\n✓ Formatter task exited (channel closed)");
    });
    
    // Give formatter task time to start
    println!("Step 3: Generating tracing events from multiple threads...\n");
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Simulate multiple threads generating tracing events
    // This demonstrates that the global subscriber can be called from anywhere
    let handles: Vec<_> = (0..3)
        .map(|i| {
            std::thread::spawn(move || {
                // Each thread generates some events
                for j in 0..5 {
                    info!(thread = i, iteration = j, "Thread event");
                    std::thread::sleep(Duration::from_millis(50));
                }
                
                if i == 1 {
                    warn!(thread = i, "Warning from thread");
                }
                
                if i == 2 {
                    error!(thread = i, "Error from thread");
                }
            })
        })
        .collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("\nStep 4: All worker threads completed, processing remaining events...\n");
    
    // Give the formatter task a moment to process remaining events
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Note: In real usage, the formatter task runs for the lifetime of the application.
    // The sender is held by the global tracing subscriber which never gets dropped.
    // When the application shuts down, you would explicitly drop/replace the subscriber
    // to close the channel, causing the formatter task to exit cleanly.
    
    // For this example, we abort since we can't easily drop the global subscriber
    formatter_task.abort();
    
    println!("✓ Example complete\n");
    println!("Note: The transition message was logged through the NEW channel-based");
    println!("      subscriber, demonstrating the seamless switchover from synchronous");
    println!("      to multi-threaded logging.");
}
