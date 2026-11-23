// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

/// Scenario: an author discovers the conformance command from xtask help or runs a previous spelling.
/// Guarantees: the advertised command serves its own help and stale invocations fail with a migration hint.
#[test]
fn conformance_is_the_advertised_entry_point() {
    let xtask = env!("CARGO_BIN_EXE_xtask");
    for args in [vec!["help"], vec!["conformance", "--help"]] {
        let out = Command::new(xtask).args(args).output().unwrap();
        assert!(out.status.success());
        let help = String::from_utf8(out.stdout).unwrap();
        let usage = help
            .lines()
            .find(|line| line.contains("conformance ["))
            .unwrap();
        assert!(usage.contains("--component FULL_URN"));
        assert!(usage.contains("--check embedded-ui"));
        assert!(!help.contains("guarantees [--component"));
        assert!(!help.contains("check-contracts [--component"));
    }
    for previous_name in ["guarantees", "check-contracts"] {
        let old = Command::new(xtask)
            .args([previous_name, "--help"])
            .output()
            .unwrap();
        assert!(!old.status.success());
        assert!(
            String::from_utf8(old.stderr)
                .unwrap()
                .contains("cargo xtask conformance")
        );
    }
}
