# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Compare the unchanged Filelog prototype and current decoder outside the workspace."""

import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import tomllib


PROTOTYPE = "86b4fb2e08cec44c3798241d440323d9ab949d22"
ORIGINAL = (
    "rust/otap-dataflow/crates/core-nodes/src/receivers/"
    "filelog_receiver/framing/decoder.rs"
)


def command_output(command, cwd):
    """Return command output, propagating failures rather than publishing partial results."""
    return subprocess.run(
        command, cwd=cwd, check=True, text=True, capture_output=True
    ).stdout.strip()


def locked_version(packages, name, prefix):
    """Select exactly one supported direct dependency version from the workspace lock."""
    matches = [
        package["version"]
        for package in packages
        if package["name"] == name and package["version"].startswith(prefix)
    ]
    if len(matches) != 1:
        raise ValueError(f"expected one locked {name} {prefix} version, got {matches}")
    return matches[0]


def summarize(criterion_root, output_file, runs):
    """Write median/min/max of independent run means, keeping fail latency visible."""
    files = sorted(criterion_root.glob("**/candidate-1/estimates.json"))
    if not files:
        raise ValueError(f"no candidate measurements in {criterion_root}")
    with output_file.open("w", newline="", encoding="utf-8") as output:
        writer = csv.writer(output, lineterminator="\n")
        writer.writerow(
            ["case", "implementation", "bytes", "median_ns", "min_ns", "max_ns", "MiB/s"]
        )
        for estimates_file in files:
            case_dir = estimates_file.parent.parent
            for implementation in ["prototype", "candidate"]:
                metadata = json.loads(
                    (case_dir / f"{implementation}-1" / "benchmark.json")
                    .read_text(encoding="utf-8")
                )
                source_bytes = metadata["throughput"]["Bytes"]
                means = [
                    json.loads(
                        (case_dir / f"{implementation}-{run}" / "estimates.json")
                        .read_text(encoding="utf-8")
                    )["mean"]["point_estimate"]
                    for run in range(1, runs + 1)
                ]
                median = statistics.median(means)
                writer.writerow(
                    [
                        metadata["full_id"],
                        implementation,
                        source_bytes,
                        f"{median:.3f}",
                        f"{min(means):.3f}",
                        f"{max(means):.3f}",
                        f"{source_bytes * 1e9 / median / (1024 * 1024):.3f}",
                    ]
                )


def main():
    """Build two small benchmark-only libraries with identical event consumers."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="new directory outside the repository")
    parser.add_argument("--prototype", default=PROTOTYPE, help="locally available Git commit")
    parser.add_argument("--runs", type=int, default=3, help="alternating run pairs (default: 3)")
    args = parser.parse_args()
    if args.runs < 1:
        parser.error("--runs must be positive")
    workspace = Path(__file__).resolve().parents[4]
    repository = workspace.parents[1]
    output = args.output.resolve()
    if output.exists() or output.is_relative_to(repository):
        parser.error("output must be a new directory outside the repository")
    prototype = command_output(
        ["git", "rev-parse", "--verify", "--end-of-options", f"{args.prototype}^{{commit}}"],
        repository,
    )
    original = subprocess.run(
        ["git", "show", f"{prototype}:{ORIGINAL}"],
        cwd=repository,
        check=True,
        capture_output=True,
    ).stdout
    candidate = (
        workspace / "crates/core-nodes/src/receivers/filelog_receiver/decoder.rs"
    ).read_bytes()
    cases = Path(__file__).with_name("cases.rs").read_bytes()
    packages = tomllib.loads((workspace / "Cargo.lock").read_text(encoding="utf-8"))["package"]
    error_version = locked_version(packages, "thiserror", "2.")
    criterion_version = locked_version(packages, "criterion", "0.8.")
    rustc = command_output(["rustc", "-Vv"], workspace)
    cargo = command_output(["cargo", "-V"], workspace)

    output.mkdir(parents=True)
    (output / "framing").mkdir()
    (output / "framing/decoder.rs").write_bytes(original)
    (output / "framing/mod.rs").write_text("pub mod decoder;\n", encoding="utf-8")
    (output / "candidate.rs").write_bytes(candidate)
    (output / "cases.rs").write_bytes(cases)
    (output / "rust-toolchain.toml").write_bytes((workspace / "rust-toolchain.toml").read_bytes())
    (output / "Cargo.toml").write_text(
        f"""[package]
name = "filelog-decode-measure"
version = "0.0.0"
edition = "2024"
publish = false
[lib]
path = "lib.rs"
[features]
candidate = []
[dependencies]
thiserror = "={error_version}"
[dev-dependencies]
criterion = "={criterion_version}"
[[bench]]
name = "decoder"
path = "main.rs"
harness = false
[profile.bench]
lto = "thin"
codegen-units = 1
""",
        encoding="utf-8",
    )
    (output / "lib.rs").write_text(
        """// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#[cfg(not(feature = "candidate"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Encoding { Utf8, Ascii, Utf16Le, Utf16Be, Raw }
#[cfg(not(feature = "candidate"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnDecodeError { PreserveRaw, Replace, Fail }
#[cfg(not(feature = "candidate"))]
pub mod framing;
#[cfg(not(feature = "candidate"))]
pub use framing::decoder;
#[cfg(feature = "candidate")]
#[path = "candidate.rs"]
pub mod decoder;
#[cfg(feature = "candidate")]
pub use decoder::{Encoding, OnDecodeError};
""",
        encoding="utf-8",
    )
    (output / "main.rs").write_text(
        """use criterion::{criterion_group, criterion_main};
use filelog_decode_measure::{Encoding, OnDecodeError, decoder};
mod cases;
criterion_group!(benches, cases::bench_decoder);
criterion_main!(benches);
""",
        encoding="utf-8",
    )
    (output / "metadata.json").write_text(
        json.dumps(
            {
                "prototype_commit": prototype,
                "candidate_sha256": hashlib.sha256(candidate).hexdigest(),
                "cases_sha256": hashlib.sha256(cases).hexdigest(),
                "rustc": rustc,
                "cargo": cargo,
                "platform": platform.platform(),
                "machine": platform.machine(),
                "logical_cpus": os.cpu_count(),
                "thiserror": error_version,
                "criterion": criterion_version,
                "runs": args.runs,
                "profile": "bench: opt-level=3, thin LTO, codegen-units=1",
            },
            indent=2,
        ) + "\n",
        encoding="utf-8",
    )
    env = dict(os.environ, CARGO_TARGET_DIR=str(output / "target"))
    for run in range(1, args.runs + 1):
        for implementation in ["prototype", "candidate"]:
            command = ["cargo", "bench", "--bench", "decoder"]
            if implementation == "candidate":
                command += ["--features", "candidate"]
            command += ["--", "--noplot", "--save-baseline", f"{implementation}-{run}"]
            with (output / f"{implementation}-{run}.log").open("w", encoding="utf-8") as log:
                subprocess.run(
                    command, cwd=output, env=env, check=True, stdout=log, stderr=subprocess.STDOUT
                )
            print(f"Completed {implementation} run {run}", flush=True)
    summarize(output / "target/criterion", output / "summary.csv", args.runs)
    print(f"Evidence retained in {output}")


if __name__ == "__main__":
    main()
