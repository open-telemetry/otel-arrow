#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Runner contract checks; mocks are not evidence of live PostgreSQL coverage."""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import run


class HarnessTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="otel-pg-ci-test-")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.artifacts = self.directory / "artifacts"
        self.environment = patch.dict(os.environ, RUNNER_TEMP=str(self.directory))
        self.environment.start()
        self.addCleanup(self.environment.stop)
        self.commands = []
        self.startup_error = self.test_error = self.cleanup_error = False
        self.no_tests = False
        self.mock_command = patch.object(run, "command", side_effect=self.command)
        self.mock_command.start()
        self.addCleanup(self.mock_command.stop)
        quiet = patch("builtins.print")
        quiet.start()
        self.addCleanup(quiet.stop)

    def command(self, args, *, env, timeout, stdout=None):
        self.commands.append(args)
        self.assertGreater(timeout, 0)
        if "up" in args:
            directory = Path(env["POSTGRESQL_FIXTURE_EXPORT_DIR"])
            self.assertFalse(directory.is_relative_to(self.artifacts))
            for name in ("ca.pem", "username", "password"):
                (directory / name).write_text("fixture", encoding="utf-8")
            if self.startup_error:
                raise subprocess.CalledProcessError(17, args)
        elif "port" in args:
            return subprocess.CompletedProcess(args, 0, stdout="127.0.0.1:49152\n")
        elif args[0] == "cargo":
            self.assertEqual(env["OTEL_ARROW_POSTGRESQL_TEST_PORT"], "49152")
            self.assertTrue(Path(env["OTEL_ARROW_POSTGRESQL_TEST_CA_FILE"]).is_file())
            self.assertIn("--ignored", args)
            self.assertNotIn("--list", args)
            if self.test_error:
                raise subprocess.CalledProcessError(19, args)
            if not self.no_tests:
                for name in run.LIVE_TESTS:
                    stdout.write(f"test receivers::postgresql_receiver::worker::live_tests::{name} ... ok\n")
                stdout.write(f"test result: ok. {len(run.LIVE_TESTS)} passed; 0 failed; 0 ignored;\n")
        elif "down" in args:
            if self.cleanup_error:
                raise subprocess.CalledProcessError(21, args)
        else:
            self.fail(f"Unexpected command: {args}")
        return subprocess.CompletedProcess(args, 0)

    def assert_cleaned(self):
        self.assertFalse((self.artifacts / "fixture.json").exists())
        self.assertEqual(list(self.directory.iterdir()), [self.artifacts])
        self.assertTrue(any("down" in args and "--volumes" in args for args in self.commands))

    # Scenario: The receiver or one of the live suite's shared dependencies changes.
    # Guarantees: Discovery selects the advisory PostgreSQL job, not unrelated files.
    def test_discovery(self):
        root = run.CI.parents[7]
        spec = importlib.util.spec_from_file_location(
            "discovery", root / ".github/scripts/discover-contrib-integration-tests.py",
        )
        discovery = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(discovery)
        manifests = discovery.load_manifests()
        for path in (
            "contrib-nodes/src/receivers/postgresql_receiver/worker.rs",
            "scraper/src/controller.rs", "engine/src/receiver.rs",
            "config/src/node.rs", "channel/src/lib.rs", "telemetry/src/registry.rs",
            "otap/src/pdata.rs", "pdata/src/lib.rs", "pdata-codec/src/lib.rs",
        ):
            selected = discovery.select_manifests(
                manifests, ["rust/otap-dataflow/crates/" + path], False,
            )
            manifest = next(item for item in selected if item["id"] == "postgresql-receiver")
            self.assertEqual(manifest["runner"], "ubuntu-24.04")
            self.assertFalse(manifest["required"])
        self.assertEqual(discovery.select_manifests(manifests, ["unrelated.txt"], False), [])
        manifest = json.loads((run.CI / "integration-test.json").read_text())
        self.assertEqual(manifest["command"][-1], "test")
        self.assertEqual(manifest["cleanup"][-1], "cleanup")

    # Scenario: Provisioning succeeds and all required live scenarios pass.
    # Guarantees: One Cargo suite execution is sufficient and all fixture state is removed.
    def test_success(self):
        run.live(self.artifacts)
        self.assertEqual(sum(args[0] == "cargo" for args in self.commands), 1)
        self.assert_cleaned()
        run.cleanup(self.artifacts)

    # Scenario: Persisting fixture ownership fails before any Docker command starts.
    # Guarantees: Empty owned directories and partial state are removed; the original error propagates.
    def test_state_write_failure(self):
        error = OSError("fixture state write failed")

        def fail_write(path, *_args, **_kwargs):
            path.write_bytes(b"{")
            raise error

        with patch.object(Path, "write_text", fail_write):
            with self.assertRaises(OSError) as raised:
                run.live(self.artifacts)
        self.assertIs(raised.exception, error)
        self.assertEqual(self.commands, [])
        self.assertFalse((self.artifacts / "fixture.json").exists())
        self.assertEqual(list(self.directory.iterdir()), [self.artifacts])

    # Scenario: Database initialization or native test execution fails.
    # Guarantees: The original failure propagates and owned resources are still removed.
    def test_failure_cleanup(self):
        for failure in ("startup_error", "test_error"):
            with self.subTest(failure=failure):
                setattr(self, failure, True)
                with self.assertRaises(subprocess.CalledProcessError):
                    run.live(self.artifacts)
                self.assert_cleaned()
                setattr(self, failure, False)

    # Scenario: A stale test filter exits successfully without executing the live suite.
    # Guarantees: Missing tests cannot yield successful live coverage.
    def test_no_op_rejected(self):
        self.no_tests = True
        with self.assertRaisesRegex(RuntimeError, "did not pass"):
            run.live(self.artifacts)
        self.assert_cleaned()

    # Scenario: Docker fails to tear down the owned project.
    # Guarantees: Cleanup fails visibly and retains ownership state for the runner's cleanup.
    def test_cleanup_failure(self):
        self.cleanup_error = True
        with self.assertRaises(subprocess.CalledProcessError):
            run.live(self.artifacts)
        self.assertTrue((self.artifacts / "fixture.json").is_file())
        self.cleanup_error = False
        run.cleanup(self.artifacts)
        self.assert_cleaned()


if __name__ == "__main__":
    unittest.main()
