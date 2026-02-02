// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Entry point to run pipeline validations manually instead of via tests.

use validation_test::pipeline::{PIPELINE_CONFIG_YAML, run_validation_tests};

#[tokio::main]
async fn main() {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| PIPELINE_CONFIG_YAML.to_string());

    if let Err(err) = run_validation_tests(Some(&config_path)).await {
        eprintln!("Pipeline validation run failed: {err}");
        std::process::exit(1);
    }
}
