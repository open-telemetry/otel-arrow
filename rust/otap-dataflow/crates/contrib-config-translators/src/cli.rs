// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Development CLI for the vendor configuration translators.
//!
//! Reads a vendor configuration document and writes the equivalent OTAP dataflow pipeline YAML.
//! Intended for inspecting translator output against real payloads without going through the
//! test harness.
//!
//! Note this lives at `src/cli.rs` rather than the conventional `src/bin/`, because the
//! repository's top-level `.gitignore` matches `bin/` and would make the file invisible to git.
//!
//! ```text
//! cargo run -p otel-arrow-dfe-contrib-config-translators --bin config-translator -- \
//!   --input path/to/AMCSConfig.json
//! ```
//!
//! The generated file can then be handed straight to the engine:
//!
//! ```text
//! cargo run --bin df_engine -- --config generated.yaml --num-cores 1
//! ```

// This binary exists to write a configuration document to stdout and diagnostics to stderr, which
// is the one place in the workspace where printing to the console is the intended behaviour
// rather than stray debugging output.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use clap::Parser;
use otel_arrow_dfe_config::engine::OtelDataflowSpec;
use otel_arrow_dfe_contrib_config_translators::ConfigTranslator;
use otel_arrow_dfe_contrib_config_translators::amcs::{AMCS_DIALECT, AmcsTranslator};
use std::path::PathBuf;
use std::process::ExitCode;

/// Translate a vendor configuration document into an OTAP dataflow pipeline specification.
#[derive(Parser, Debug)]
#[command(name = "config-translator", about, long_about = None)]
struct Args {
    /// Path to the vendor configuration document to translate.
    #[arg(short, long)]
    input: PathBuf,

    /// Where to write the generated YAML. Defaults to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Configuration dialect to translate from.
    #[arg(short, long, default_value = AMCS_DIALECT)]
    dialect: String,

    /// Verify that the generated YAML parses back through the engine configuration loader.
    #[arg(long)]
    validate: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            // Diagnostics belong on stderr so stdout stays a clean YAML document.
            eprintln!("config-translator: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Translate the input document and emit the result.
fn run(args: &Args) -> Result<(), String> {
    if args.dialect != AMCS_DIALECT {
        return Err(format!(
            "unsupported dialect `{}`; only `{AMCS_DIALECT}` is currently available",
            args.dialect
        ));
    }

    let raw = std::fs::read_to_string(&args.input)
        .map_err(|e| format!("cannot read {}: {e}", args.input.display()))?;

    let yaml = AmcsTranslator::new()
        .translate_to_yaml(&raw)
        .map_err(|e| format!("translation failed: {e}"))?;

    if args.validate {
        let _ = OtelDataflowSpec::from_yaml(&yaml)
            .map_err(|e| format!("generated configuration was rejected by the engine: {e}"))?;
        eprintln!("config-translator: generated configuration parsed and validated successfully");
    }

    match &args.output {
        Some(path) => std::fs::write(path, &yaml)
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?,
        None => println!("{yaml}"),
    }

    Ok(())
}
