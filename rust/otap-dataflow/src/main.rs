//! Create and run a multi-core pipeline

use clap::Parser;
use mimalloc_rust::*;
use otap_df_config::pipeline::PipelineConfig;
use otap_df_config::pipeline_group::Quota;
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use std::fs;
use std::path::Path;

#[global_allocator]
static GLOBAL_MIMALLOC: GlobalMiMalloc = GlobalMiMalloc;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the JSON pipeline configuration file
    #[arg(short, long, default_value = "configs/otlp-otlp.json")]
    pipeline: String,

    /// Number of cores to use (0 for default)
    #[arg(long, default_value = "0")]
    num_cores: usize,
}

fn load_pipeline_config(file_path: &str) -> Result<PipelineConfig, Box<dyn std::error::Error>> {
    if Path::new(file_path).exists() {
        let config_content = fs::read_to_string(file_path)?;
        let pipeline_cfg: PipelineConfig = serde_json::from_str(&config_content)?;
        Ok(pipeline_cfg)
    } else {
        Err(format!("Pipeline configuration file '{file_path}' not found").into())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load pipeline configuration from file
    let pipeline_cfg = load_pipeline_config(&args.pipeline)?;

    // Create controller and start pipeline with multi-core support
    let controller = Controller::new(&OTAP_PIPELINE_FACTORY);
    let quota = Quota {
        num_cores: args.num_cores,
    };

    println!("Starting pipeline with {} cores", args.num_cores);

    let result = controller.run_forever(pipeline_cfg, quota);
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
