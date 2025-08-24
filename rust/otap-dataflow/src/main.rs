//! Create and run a multi-core pipeline

use clap::Parser;
use mimalloc_rust::*;
use otap_df_config::pipeline::PipelineConfig;
use otap_df_config::pipeline_group::Quota;
use otap_df_config::{PipelineGroupId, PipelineId};
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use std::path::PathBuf;

#[global_allocator]
static GLOBAL_MIMALLOC: GlobalMiMalloc = GlobalMiMalloc;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    after_help = system_info()
)]
struct Args {
    /// Path to the pipeline configuration file (.json, .yaml, or .yml)
    #[arg(short, long)]
    pipeline: PathBuf,

    /// Number of cores to use (0 for default)
    #[arg(long, default_value = "0")]
    num_cores: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // For now, we predefine pipeline group and pipeline IDs.
    // That will be replaced with a more dynamic approach in the future.
    let pipeline_group_id: PipelineGroupId = "default_pipeline_group".into();
    let pipeline_id: PipelineId = "default_pipeline".into();

    // Load pipeline configuration from file
    let pipeline_cfg = PipelineConfig::from_file(
        pipeline_group_id.clone(),
        pipeline_id.clone(),
        &args.pipeline,
    )?;

    // Create controller and start pipeline with multi-core support
    let controller = Controller::new(&OTAP_PIPELINE_FACTORY);
    let quota = Quota {
        num_cores: args.num_cores,
    };

    println!("Starting pipeline with {} cores", args.num_cores);

    let result = controller.run_forever(pipeline_group_id, pipeline_id, pipeline_cfg, quota);
    match result {
        Ok(_) => {
            println!("Pipeline run successfully");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{e}");
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
        "Examples:
  {} --pipeline configs/otlp-perf.yaml --num-cores 4
  {} -p configs/otlp-perf.yaml

System Information:
  Available CPU cores: {}
  Build mode: {}
  Default memory allocator: mimalloc

Available Plugin URNs:
  Receivers: {}
  Processors: {}
  Exporters: {}

Configuration files can be found in the configs/ directory.{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_NAME"),
        available_cores,
        build_mode,
        receivers_sorted.join(", "),
        processors_sorted.join(", "),
        exporters_sorted.join(", "),
        debug_warning
    )
}
