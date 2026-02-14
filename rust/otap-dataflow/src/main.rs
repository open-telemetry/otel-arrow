// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Create and run a multi-core pipeline

use clap::Parser;
use otap_df_config::engine::OtelDataflowSpec;
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use std::path::PathBuf;
use sysinfo::System;

#[cfg(all(
    not(windows),
    feature = "jemalloc",
    feature = "mimalloc",
    not(any(test, doc)),
    not(clippy)
))]
compile_error!(
    "Features `jemalloc` and `mimalloc` are mutually exclusive. \
     To build with mimalloc, use: cargo build --release --no-default-features --features mimalloc"
);

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(all(not(windows), feature = "jemalloc", not(feature = "mimalloc")))]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(all(not(windows), feature = "jemalloc", not(feature = "mimalloc")))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    after_help = system_info(),
    after_long_help = concat!(
        "EXAMPLES:\n",
        "  ", env!("CARGO_BIN_NAME"), " --config  config.yaml\n",
    )
)]
struct Args {
    /// Path to the engine configuration file (.json, .yaml, or .yml)
    #[arg(short = 'c', long, value_name = "FILE")]
    config: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize rustls crypto provider (required for rustls 0.23+)
    // We use ring as the default provider
    #[cfg(feature = "experimental-tls")]
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| format!("Failed to install rustls crypto provider: {e:?}"))?;

    let Args { config } = Args::parse();

    println!("{}", system_info());

    let controller = Controller::new(&OTAP_PIPELINE_FACTORY);
    let engine_cfg = OtelDataflowSpec::from_file(config)?;
    let result = controller.run_forever(engine_cfg);
    match result {
        Ok(_) => {
            println!("Pipeline run successfully");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Pipeline failed to run: {e}");
            std::process::exit(1);
        }
    }
}

fn system_info() -> String {
    // Your custom logic here - this could read files, check system state, etc.
    let available_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let build_mode = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    let memory_allocator = if cfg!(feature = "mimalloc") {
        "mimalloc"
    } else if cfg!(all(feature = "jemalloc", not(windows))) {
        "jemalloc"
    } else {
        "system"
    };

    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_memory_gb = sys.total_memory() as f64 / 1_073_741_824.0;
    let available_memory_gb = sys.available_memory() as f64 / 1_073_741_824.0;

    let debug_warning = if cfg!(debug_assertions) {
        "\n\n⚠️  WARNING: This binary was compiled in debug mode.
   Debug builds are NOT recommended for production, benchmarks, or performance testing.
   Use 'cargo build --release' for optimal performance."
    } else {
        ""
    };

    // Get available OTAP plugins
    let receivers: Vec<&str> = OTAP_PIPELINE_FACTORY
        .get_receiver_factory_map()
        .keys()
        .copied()
        .collect();
    let processors: Vec<&str> = OTAP_PIPELINE_FACTORY
        .get_processor_factory_map()
        .keys()
        .copied()
        .collect();
    let exporters: Vec<&str> = OTAP_PIPELINE_FACTORY
        .get_exporter_factory_map()
        .keys()
        .copied()
        .collect();

    let mut receivers_sorted = receivers;
    let mut processors_sorted = processors;
    let mut exporters_sorted = exporters;
    receivers_sorted.sort();
    processors_sorted.sort();
    exporters_sorted.sort();

    format!(
        "System Information:
  Available CPU cores: {}
  Available memory: {:.2} GB / {:.2} GB
  Build mode: {}
  Memory allocator: {}

Available Plugin URNs:
  Receivers: {}
  Processors: {}
  Exporters: {}

Configuration files can be found in the configs/ directory.{}",
        available_cores,
        available_memory_gb,
        total_memory_gb,
        build_mode,
        memory_allocator,
        receivers_sorted.join(", "),
        processors_sorted.join(", "),
        exporters_sorted.join(", "),
        debug_warning
    )
}
