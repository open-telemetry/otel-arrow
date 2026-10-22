#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Run the SQL Server live suite using the shared contrib integration runner."""

import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time


WORKSPACE = Path(__file__).resolve().parents[6]
SCRIPT = Path(__file__).resolve().with_name("sql-server-fixture.py")


def run(command, *, env=None):
    subprocess.run(command, cwd=WORKSPACE, env=env, check=True)

def fixture_environment(directory):
    fixture = json.loads((directory / "fixture.json").read_text(encoding="utf-8"))
    return dict(os.environ, **fixture["env"], CARGO_PROFILE_TEST_DEBUG="0")


def engine_trial(directory, artifacts):
    run([
        "cargo", "build", "--locked", "--no-default-features",
        "--features", "sql-server,core-nodes,crypto-ring", "--bin", "df_engine",
    ])
    environment = fixture_environment(directory)
    environment.update({
        key.removeprefix("OTAP_"): value
        for key, value in environment.copy().items() if key.startswith("OTAP_SQL_SERVER_")
    })
    checkpoint = directory / "engine-checkpoints"
    environment["SQL_SERVER_STATE_DIR"] = str(checkpoint)
    target = Path(environment.get("CARGO_TARGET_DIR", WORKSPACE / "target"))
    binary = target / "debug" / ("df_engine.exe" if sys.platform == "win32" else "df_engine")
    log_path = directory / "engine.log"
    with log_path.open("w", encoding="utf-8") as log:
        process = subprocess.Popen([
            str(binary), "--config", "configs/sql-server-console.yaml",
            "--http-admin-bind", "127.0.0.1:0",
        ], cwd=WORKSPACE, env=environment, stdout=log, stderr=subprocess.STDOUT,
            creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if sys.platform == "win32" else 0)
        try:
            deadline = time.monotonic() + 60
            while True:
                if process.poll() is not None:
                    raise RuntimeError("Engine exited before installing the expected checkpoint")
                cursors = [
                    json.loads(path.read_text(encoding="utf-8"))["payload"]["cursor"]
                    for path in checkpoint.rglob("*.checkpoint.*.json")
                ]
                if any(cursor.get("tie_breaker") == 3 for cursor in cursors):
                    break
                if time.monotonic() >= deadline:
                    raise RuntimeError("Engine did not acknowledge fixture rows within 60 seconds")
                time.sleep(0.1)
            process.send_signal(signal.CTRL_BREAK_EVENT if sys.platform == "win32" else signal.SIGINT)
            if process.wait(timeout=15) != 0:
                raise RuntimeError("Engine did not shut down cleanly")
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
    output = log_path.read_text(encoding="utf-8")
    if "hello" not in output or "third" not in output:
        raise RuntimeError("Console did not emit the fixture payloads")
    artifacts.mkdir(parents=True, exist_ok=True)
    (artifacts / "engine-result.json").write_text(
        json.dumps({"example": "sql-server-console.yaml", "acknowledged_id": 3,
                    "console_rows_observed": True, "clean_shutdown": True}), encoding="utf-8"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["test", "cleanup"])
    args = parser.parse_args()
    # Fixtures and secrets must never live beneath the uploaded artifact directory.
    directory = Path(os.environ["RUNNER_TEMP"]) / "sql-server-fixture"
    artifacts = Path(os.environ["OTEL_ARROW_INTEGRATION_ARTIFACT_DIR"])
    if directory.resolve().is_relative_to(artifacts.resolve()):
        raise RuntimeError("Fixture secrets must be outside the artifact directory")
    if args.operation == "cleanup":
        try:
            if (directory / "fixture.json").is_file():
                run([sys.executable, str(SCRIPT), "stop", "--directory", str(directory)])
        finally:
            log = directory / "server.log"
            if log.is_file():
                artifacts.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(log, artifacts / "server.log")
        return

    run([
        sys.executable, "-B", "-m", "unittest", "discover",
        "-s", str(SCRIPT.parent), "-p", "test_sql_server_fixture.py",
    ])
    run([
        sys.executable, str(SCRIPT), "start", "--directory", str(directory),
        "--allow-unencrypted",
    ])
    run([
        "cargo", "test", "--locked", "-p", "otel-arrow-dfe-contrib-nodes",
        "--no-default-features", "--features", "sql-server", "--lib",
        "sql_server_receiver::live_tests", "--", "--ignored", "--nocapture", "--test-threads=1",
    ], env=fixture_environment(directory))
    for variant in ("replacement", "expired", "valid"):
        run([
            sys.executable, str(SCRIPT), "rotate-certificate",
            "--directory", str(directory), "--certificate", variant,
        ])
        run([
            "cargo", "test", "--locked", "-p", "otel-arrow-dfe-contrib-nodes",
            "--no-default-features", "--features", "sql-server", "--lib",
            "sql_server_receiver::live_tests::live_certificate_rotation_state",
            "--", "--ignored", "--nocapture", "--test-threads=1",
        ], env=fixture_environment(directory))
    engine_trial(directory, artifacts)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"SQL Server integration failed: {error}", file=sys.stderr)
        sys.exit(1)
