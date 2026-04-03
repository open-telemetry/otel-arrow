// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A minimal custom collector built on top of the OTAP dataflow engine
//! used as a library.
//!
//! This example demonstrates how to embed the engine in a custom binary
//! using the `otap_df_controller::startup` helpers, without copying any
//! startup logic from the default `src/main.rs`.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example custom_collector -- --config configs/fake-perf.yaml
//! cargo run --example custom_collector -- --config configs/fake-perf.yaml --poll-status
//! ```

use std::path::PathBuf;

use clap::Parser;
use otap_df_config::engine::OtelDataflowSpec;
use otap_df_config::policy::{CoreAllocation, CoreRange};
// Side-effect imports: link the crates so their `linkme` distributed-slice
// registrations (component factories) are visible in `OTAP_PIPELINE_FACTORY`.
use otap_df_contrib_nodes as _;
use otap_df_controller::Controller;
use otap_df_controller::startup;
use otap_df_core_nodes as _;
use otap_df_otap::OTAP_PIPELINE_FACTORY;

/// A minimal custom OTAP dataflow collector.
///
/// Shows how the engine can be embedded as a library. All startup logic
/// (validation, CLI overrides, system info) comes from
/// `otap_df_controller::startup`.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Custom OTAP Collector (library mode example)"
)]
struct Args {
    /// Path to the engine configuration file (.yaml or .json)
    #[arg(short = 'c', long, value_name = "FILE")]
    config: PathBuf,

    /// Number of cores to use (0 for all available cores)
    #[arg(long)]
    num_cores: Option<usize>,

    /// Inclusive range of CPU core IDs to pin threads to (e.g. "0-3")
    #[arg(long, value_name = "RANGE", value_parser = parse_core_id_allocation, conflicts_with = "num_cores")]
    core_id_range: Option<CoreAllocation>,

    /// Address to bind the HTTP admin server to (e.g. "127.0.0.1:8080")
    #[arg(long)]
    http_admin_bind: Option<String>,

    /// Validate the configuration and exit without starting the engine
    #[arg(long)]
    validate_and_exit: bool,

    /// Periodically print pipeline health snapshots to stderr
    #[arg(long)]
    poll_status: bool,
}

fn parse_core_id_allocation(s: &str) -> Result<CoreAllocation, String> {
    Ok(CoreAllocation::CoreSet {
        set: s
            .split(',')
            .map(|part| {
                part.trim()
                    .parse::<usize>()
                    .map(|n| CoreRange { start: n, end: n })
                    .or_else(|_| {
                        let normalized = part.replace("..=", "-").replace("..", "-");
                        let mut parts = normalized.split('-');
                        let start = parts
                            .next()
                            .ok_or("missing start")?
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "invalid start".to_string())?;
                        let end = parts
                            .next()
                            .ok_or("missing end")?
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "invalid end".to_string())?;
                        Ok(CoreRange { start, end })
                    })
            })
            .collect::<Result<Vec<CoreRange>, String>>()?,
    })
}

fn memory_allocator_name() -> &'static str {
    if cfg!(feature = "mimalloc") {
        "mimalloc"
    } else if cfg!(all(feature = "jemalloc", not(windows))) {
        "jemalloc"
    } else {
        "system"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the TLS crypto provider before any network operations.
    otap_df_otap::crypto::install_crypto_provider()
        .map_err(|e| format!("Failed to install rustls crypto provider: {e}"))?;

    let args = Args::parse();

    // Print system diagnostics and registered components.
    println!(
        "{}",
        startup::system_info(&OTAP_PIPELINE_FACTORY, memory_allocator_name())
    );

    // Load and patch the configuration.
    let mut engine_cfg = OtelDataflowSpec::from_file(&args.config)?;
    startup::apply_cli_overrides(
        &mut engine_cfg,
        args.num_cores,
        args.core_id_range,
        args.http_admin_bind,
    );

    // Validate that every node URN maps to a registered component.
    startup::validate_engine_components(&engine_cfg, &OTAP_PIPELINE_FACTORY)?;

    if args.validate_and_exit {
        println!("Configuration '{}' is valid.", args.config.display());
        return Ok(());
    }

    // Run the engine, obtaining an ObservedStateHandle for in-process health checks.
    let poll_status = args.poll_status;
    let controller = Controller::new(&OTAP_PIPELINE_FACTORY);
    let result = controller.run_forever_with_observer(engine_cfg, |handle| {
        eprintln!("[observer] ObservedStateHandle obtained");
        if poll_status {
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    let snapshot = handle.snapshot();
                    for (key, status) in &snapshot {
                        eprintln!(
                            "[observer] pipeline {}:{} -> {:?}",
                            key.pipeline_group_id().as_ref(),
                            key.pipeline_id().as_ref(),
                            status
                        );
                    }
                }
            });
        }
    });
    match result {
        Ok(_) => {
            println!("Pipeline completed successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("Pipeline failed: {e}");
            std::process::exit(1);
        }
    }
}
