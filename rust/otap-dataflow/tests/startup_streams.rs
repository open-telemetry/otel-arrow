// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Verifies which standard stream the engine binary uses for startup output.
//!
//! The console output service guarantees that a `record_json` stdout stays
//! parseable, but the startup banner is written before any configuration has
//! claimed stdout, so it cannot go through that service. This test pins the
//! banner to stderr from outside the process, which is the only place the
//! distinction is observable.

use std::path::PathBuf;
use std::process::Command;

/// A line that only the startup banner produces.
const BANNER_MARKER: &str = "System Information:";

fn engine_config(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("configs")
        .join(name)
}

/// Scenario: the engine binary validates a configuration and exits.
/// Guarantees: the startup banner goes to stderr and stdout carries only the
/// result the operator asked for, so prose can never precede `record_json` output.
#[test]
fn startup_banner_goes_to_stderr_not_stdout() {
    let output = Command::new(env!("CARGO_BIN_EXE_df_engine"))
        .arg("--config")
        .arg(engine_config("syslog-console.yaml"))
        .arg("--validate-and-exit")
        .output()
        .expect("the engine binary starts");

    assert!(
        output.status.success(),
        "validation run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");

    assert!(
        stderr.contains(BANNER_MARKER),
        "the startup banner must be written to stderr, stderr was:\n{stderr}"
    );
    assert!(
        !stdout.contains(BANNER_MARKER),
        "the startup banner must never reach stdout, stdout was:\n{stdout}"
    );
    // The validation result is an explicitly requested answer, so it stays on stdout.
    assert!(
        stdout.contains("is valid."),
        "stdout must still carry the requested validation result, stdout was:\n{stdout}"
    );
}
