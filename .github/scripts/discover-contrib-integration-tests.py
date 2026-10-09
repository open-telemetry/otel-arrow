#!/usr/bin/env python3

import argparse
import fnmatch
import json
import os
import pathlib
import subprocess
import sys


REPOSITORY_ROOT = pathlib.Path(__file__).resolve().parents[2]
MANIFEST_PATTERN = (
    "rust/otap-dataflow/crates/contrib-nodes/src/**/ci/integration-test.json"
)
GLOBAL_PATHS = (
    ".github/scripts/discover-contrib-integration-tests.py",
    ".github/scripts/run-contrib-integration-test.py",
    ".github/workflows/contrib-integration.yml",
    ".github/workflows/rust-ci.yml",
    "rust/otap-dataflow/Cargo.lock",
    "rust/otap-dataflow/Cargo.toml",
    "rust/otap-dataflow/crates/contrib-nodes/Cargo.toml",
    "rust/otap-dataflow/crates/contrib-nodes/src/lib.rs",
    "rust/otap-dataflow/crates/contrib-nodes/src/receivers/mod.rs",
)
REQUIRED_FIELDS = {
    "id": str,
    "name": str,
    "runner": str,
    "platform": str,
    "timeout_minutes": int,
    "required": bool,
    "paths": list,
    "command": list,
}


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--base-sha")
    parser.add_argument("--head-sha")
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--required-only", action="store_true")
    return parser.parse_args()


def load_manifests():
    manifests = []
    identifiers = set()
    for path in sorted(REPOSITORY_ROOT.glob(MANIFEST_PATTERN)):
        relative_path = path.relative_to(REPOSITORY_ROOT).as_posix()
        with path.open(encoding="utf-8") as manifest_file:
            manifest = json.load(manifest_file)

        for field, expected_type in REQUIRED_FIELDS.items():
            if not isinstance(manifest.get(field), expected_type):
                raise ValueError(
                    f"{relative_path}: field '{field}' must be "
                    f"{expected_type.__name__}"
                )

        if manifest["id"] in identifiers:
            raise ValueError(f"duplicate integration test id: {manifest['id']}")
        identifiers.add(manifest["id"])

        if manifest["platform"] not in ("Linux", "Windows", "macOS"):
            raise ValueError(
                f"{relative_path}: unsupported platform '{manifest['platform']}'"
            )
        if manifest["timeout_minutes"] <= 0:
            raise ValueError(f"{relative_path}: timeout_minutes must be positive")
        if not manifest["paths"] or not all(
            isinstance(pattern, str) and pattern for pattern in manifest["paths"]
        ):
            raise ValueError(f"{relative_path}: paths must contain strings")
        if not manifest["command"] or not all(
            isinstance(argument, str) and argument
            for argument in manifest["command"]
        ):
            raise ValueError(f"{relative_path}: command must contain strings")
        cleanup = manifest.get("cleanup")
        if cleanup is not None and (
            not isinstance(cleanup, list)
            or not cleanup
            or not all(isinstance(argument, str) and argument for argument in cleanup)
        ):
            raise ValueError(f"{relative_path}: cleanup must contain strings")

        manifest["manifest"] = relative_path
        manifests.append(manifest)
    return manifests


def changed_paths(base_sha, head_sha):
    if not base_sha or not head_sha:
        raise ValueError("base and head SHAs are required unless --all is used")
    result = subprocess.run(
        ["git", "diff", "--name-only", f"{base_sha}...{head_sha}"],
        cwd=REPOSITORY_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return [line for line in result.stdout.splitlines() if line]


def matches(path, pattern):
    if pattern.endswith("/**"):
        return path.startswith(pattern[:-3])
    return fnmatch.fnmatchcase(path, pattern)


def select_manifests(manifests, paths, select_all):
    if select_all or any(
        matches(path, pattern) for path in paths for pattern in GLOBAL_PATHS
    ):
        return manifests

    selected = []
    for manifest in manifests:
        patterns = manifest["paths"]
        if manifest["manifest"] in paths or any(
            matches(path, pattern) for path in paths for pattern in patterns
        ):
            selected.append(manifest)
    return selected


def write_output(name, value):
    output_path = os.environ.get("GITHUB_OUTPUT")
    if output_path:
        with open(output_path, "a", encoding="utf-8") as output_file:
            output_file.write(f"{name}={value}\n")
    else:
        print(f"{name}={value}")


def main():
    args = parse_args()
    try:
        manifests = load_manifests()
        paths = [] if args.all else changed_paths(args.base_sha, args.head_sha)
        selected = select_manifests(manifests, paths, args.all)
        if args.required_only:
            selected = [manifest for manifest in selected if manifest["required"]]
    except (OSError, subprocess.CalledProcessError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    matrix = {
        "include": [
            {
                key: manifest[key]
                for key in (
                    "id",
                    "name",
                    "runner",
                    "platform",
                    "timeout_minutes",
                    "required",
                    "manifest",
                )
            }
            for manifest in selected
        ]
    }
    write_output("matrix", json.dumps(matrix, separators=(",", ":")))
    write_output("selected_count", str(len(selected)))

    if paths:
        print("Changed paths:")
        for path in paths:
            print(f"  {path}")
    print("Selected contrib integration tests:")
    for manifest in selected:
        requirement = "required" if manifest["required"] else "advisory"
        print(f"  {manifest['id']} ({manifest['runner']}, {requirement})")
    if not selected:
        print("  none")
    return 0


if __name__ == "__main__":
    sys.exit(main())
