// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration test for the configurable CLI branding used by library embedders.
//!
//! This test exercises the public `run_with_terminal_and_diagnostics_branded`
//! entrypoint with a non-default [`Branding`] and asserts that a
//! machine-readable output envelope carries the embedder's `schemaVersion`
//! rather than the default `dfctl/v1`. It runs as a separate process-per-binary
//! integration test so the process-global branding does not affect other tests.

use clap::Parser;
use otap_df_ctl::{Branding, Cli, run_with_terminal_and_diagnostics_branded};

#[tokio::test]
async fn branded_entrypoint_overrides_schema_version() {
    // `commands --output agent-json` is a connection-free command that emits the
    // stable agent JSON envelope (including `schemaVersion`), so no admin server
    // is required.
    let cli = Cli::try_parse_from(["embedder", "commands", "--output", "agent-json"])
        .expect("parse commands invocation");

    let branding = Branding {
        bin_name: "embedder",
        schema_version: "embedder/v1",
    };

    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    run_with_terminal_and_diagnostics_branded(cli, &mut stdout, false, &mut stderr, branding)
        .await
        .expect("branded command run should succeed");

    let value: serde_json::Value =
        serde_json::from_slice(&stdout).expect("output should be valid JSON");
    assert_eq!(
        value
            .get("schemaVersion")
            .and_then(|v| v.as_str())
            .expect("schemaVersion present"),
        "embedder/v1",
        "embedder branding should override the default schemaVersion"
    );
}

#[tokio::test]
async fn branded_verbose_diagnostics_use_bin_name() {
    // `config view --verbose` reaches `write_diagnostics` without needing an
    // admin server (it only displays resolved connection settings). The branded
    // diagnostics line on stderr must carry the embedder's binary name.
    let cli = Cli::try_parse_from(["embedder", "--verbose", "config", "view"])
        .expect("parse config view invocation");

    let branding = Branding {
        bin_name: "embedder",
        schema_version: "embedder/v1",
    };

    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    run_with_terminal_and_diagnostics_branded(cli, &mut stdout, false, &mut stderr, branding)
        .await
        .expect("branded config view should succeed");

    let diagnostics = String::from_utf8(stderr).expect("stderr should be valid UTF-8");
    assert!(
        diagnostics.contains("embedder: target="),
        "verbose diagnostics should be prefixed with the embedder binary name, got: {diagnostics:?}"
    );
}
