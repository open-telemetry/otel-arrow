// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

/// Scenario: an author discovers the contract check from xtask help or runs its previous spelling.
/// Guarantees: the advertised command serves its own help and stale invocations fail with a migration hint.
#[test]
fn check_contracts_is_the_advertised_entry_point() {
    let xtask = env!("CARGO_BIN_EXE_xtask");
    for args in [vec!["help"], vec!["check-contracts", "--help"]] {
        let out = Command::new(xtask).args(args).output().unwrap();
        assert!(out.status.success());
        let help = String::from_utf8(out.stdout).unwrap();
        assert!(help.contains("check-contracts [--component"));
        assert!(!help.contains("guarantees [--component"));
    }
    let old = Command::new(xtask)
        .args(["guarantees", "--help"])
        .output()
        .unwrap();
    assert!(!old.status.success());
    assert!(
        String::from_utf8(old.stderr)
            .unwrap()
            .contains("cargo xtask check-contracts")
    );
}
