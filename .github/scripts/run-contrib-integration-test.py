#!/usr/bin/env python3

import argparse
import json
import os
import pathlib
import subprocess
import sys
import time


REPOSITORY_ROOT = pathlib.Path(__file__).resolve().parents[2]


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True)
    return parser.parse_args()


def run_command(command, environment):
    print(f"Running: {subprocess.list2cmdline(command)}", flush=True)
    return subprocess.run(
        command,
        cwd=REPOSITORY_ROOT,
        env=environment,
        check=False,
    ).returncode


def main():
    args = parse_args()
    manifest_path = (REPOSITORY_ROOT / args.manifest).resolve()
    if not manifest_path.is_relative_to(REPOSITORY_ROOT):
        print("error: manifest must be inside the repository", file=sys.stderr)
        return 1

    try:
        with manifest_path.open(encoding="utf-8") as manifest_file:
            manifest = json.load(manifest_file)
    except (OSError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    runner_os = os.environ.get("RUNNER_OS")
    if runner_os and runner_os != manifest["platform"]:
        print(
            f"error: test requires {manifest['platform']}, runner is {runner_os}",
            file=sys.stderr,
        )
        return 1

    artifact_dir = (
        REPOSITORY_ROOT / ".ci" / "contrib-integration" / manifest["id"]
    )
    artifact_dir.mkdir(parents=True, exist_ok=True)
    environment = os.environ.copy()
    environment["OTEL_ARROW_INTEGRATION_ARTIFACT_DIR"] = str(artifact_dir)

    started_at = time.time()
    command_result = 1
    cleanup_result = 0
    try:
        command_result = run_command(manifest["command"], environment)
    except OSError as error:
        print(f"error: failed to start integration command: {error}", file=sys.stderr)
    finally:
        if manifest.get("cleanup"):
            try:
                cleanup_result = run_command(manifest["cleanup"], environment)
            except OSError as error:
                print(f"error: failed to start cleanup command: {error}", file=sys.stderr)
                cleanup_result = 1

    result = {
        "id": manifest["id"],
        "command_exit_code": command_result,
        "cleanup_exit_code": cleanup_result,
        "duration_seconds": round(time.time() - started_at, 3),
    }
    with (artifact_dir / "result.json").open("w", encoding="utf-8") as result_file:
        json.dump(result, result_file, indent=2)
        result_file.write("\n")

    if command_result != 0:
        return command_result
    return cleanup_result


if __name__ == "__main__":
    sys.exit(main())
