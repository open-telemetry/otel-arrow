#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Run the existing PostgreSQL TLS fixture through contrib integration CI."""

import argparse
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tempfile
import unittest
import uuid


CI = Path(__file__).resolve().parent
WORKSPACE = CI.parents[5]
COMPOSE = CI.parent / "fixtures/tls/compose.yaml"
LIVE_TESTS = (
    "live_verified_tls_polling_modes",
    "live_factory_replay_and_restart",
    "live_transport_cancellation_and_reconnect",
)


def command(args, *, env, timeout, stdout=None):
    return subprocess.run(
        args, cwd=WORKSPACE, env=env, timeout=timeout, check=True,
        stdout=stdout, stderr=subprocess.STDOUT if stdout else None, text=True,
    )


def compose(project):
    return ["docker", "compose", "--project-name", project, "--file", str(COMPOSE)]


def cleanup(artifacts):
    state = artifacts / "fixture.json"
    if not state.exists():
        return
    fixture = json.loads(state.read_text(encoding="utf-8"))
    project = fixture["project"]
    directory = Path(fixture["directory"])
    if not re.fullmatch(r"otel-pg-ci-[0-9a-f]{32}", project):
        raise ValueError("Invalid owned PostgreSQL Compose project")
    if not directory.name.startswith("otel-pg-ci-"):
        raise ValueError("Invalid owned PostgreSQL fixture directory")
    environment = dict(
        os.environ, POSTGRESQL_FIXTURE_PORT="0",
        POSTGRESQL_FIXTURE_EXPORT_DIR=str(directory),
    )
    # Do not collect container logs/configuration: they can include credentials.
    command(compose(project) + ["down", "--volumes"], env=environment, timeout=75)
    for name in ("ca.pem", "username", "password"):
        (directory / name).unlink(missing_ok=True)
    directory.rmdir()
    state.unlink()


def live(artifacts):
    artifacts.mkdir(parents=True, exist_ok=True)
    if (artifacts / "fixture.json").exists():
        raise RuntimeError("Previous fixture state exists; run cleanup first")
    directory = Path(tempfile.mkdtemp(
        prefix="otel-pg-ci-", dir=os.environ.get("RUNNER_TEMP"),
    ))
    if directory.resolve().is_relative_to(artifacts.resolve()):
        directory.rmdir()
        raise ValueError("Fixture credentials must be outside uploaded artifacts")
    project = "otel-pg-ci-" + uuid.uuid4().hex
    state = artifacts / "fixture.json"
    try:
        state.write_text(
            json.dumps({"project": project, "directory": str(directory)}), encoding="utf-8",
        )
    except OSError as error:
        try:
            directory.rmdir()
            state.unlink(missing_ok=True)
        except OSError as cleanup_error:
            error.add_note(f"Failed to remove unprovisioned fixture state: {cleanup_error}")
        raise
    environment = dict(
        os.environ, POSTGRESQL_FIXTURE_PORT="0",
        POSTGRESQL_FIXTURE_EXPORT_DIR=str(directory),
    )
    try:
        print("::add-mask::postgres-admin", flush=True)
        print("::add-mask::otel-reader-password", flush=True)
        command(
            compose(project) + ["up", "--build", "--detach", "--wait", "--wait-timeout", "120"],
            env=environment, timeout=300,
        )
        binding = command(
            compose(project) + ["port", "postgres", "5432"],
            env=environment, timeout=30, stdout=subprocess.PIPE,
        ).stdout.strip()
        match = re.fullmatch(r"127\.0\.0\.1:([0-9]+)", binding)
        if not match or not 0 < int(match[1]) <= 65535:
            raise RuntimeError("Expected exactly one PostgreSQL loopback port")
        environment.update(
            OTEL_ARROW_POSTGRESQL_TEST_PORT=match[1],
            OTEL_ARROW_POSTGRESQL_TEST_CA_FILE=str(directory / "ca.pem"),
        )
        log_path = artifacts / "postgresql-receiver-test.log"
        with log_path.open("w", encoding="utf-8") as log:
            command([
                "cargo", "test", "--locked", "-p", "otel-arrow-dfe-contrib-nodes",
                "--no-default-features", "--features", "postgresql", "--lib",
                "postgresql_receiver::worker::live_tests", "--",
                "--ignored", "--nocapture", "--test-threads=1",
            ], env=environment, timeout=2100, stdout=log)
        output = log_path.read_text(encoding="utf-8")
        print(output, flush=True)
        for name in LIVE_TESTS:
            if not re.search(rf"test \S+::{name} \.\.\. ok", output):
                raise RuntimeError(f"Live PostgreSQL test did not pass: {name}")
        if f"test result: ok. {len(LIVE_TESTS)} passed; 0 failed; 0 ignored;" not in output:
            raise RuntimeError("Expected all live tests to execute, not an empty suite")
    finally:
        cleanup(artifacts)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("test", "cleanup"))
    args = parser.parse_args()

    def interrupted(signum, _frame):
        raise RuntimeError(f"PostgreSQL integration interrupted by signal {signum}")

    signal.signal(signal.SIGTERM, interrupted)
    artifacts = Path(os.environ["OTEL_ARROW_INTEGRATION_ARTIFACT_DIR"]).resolve()
    if args.operation == "cleanup":
        cleanup(artifacts)
    else:
        suite = unittest.defaultTestLoader.discover(str(CI), pattern="test_harness.py")
        if not unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful():
            raise RuntimeError("PostgreSQL harness checks failed")
        live(artifacts)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"PostgreSQL integration failed: {error}", file=sys.stderr)
        sys.exit(1)
