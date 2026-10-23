#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Runner contract checks; mocks are not evidence of live PostgreSQL coverage."""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import Mock, call, patch

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

    # Scenario: Compose waits for a fixture that has finished initialization.
    # Guarantees: Both readiness probes require TCP, which the temporary init server never opens.
    def test_tcp_readiness(self):
        healthcheck = run.COMPOSE.read_text(encoding="utf-8").split("healthcheck:", 1)[1]
        self.assertIn("pg_isready -h 127.0.0.1", healthcheck)
        self.assertIn('PGPASSWORD="$${POSTGRES_PASSWORD}" psql -h 127.0.0.1', healthcheck)
        self.assertIn("SELECT count(*) FROM public.receiver_events", healthcheck)

    # Scenario: The owned command succeeds, fails, times out, or is interrupted.
    # Guarantees: Every result waits for process-group cleanup before returning or propagating the error.
    def test_command_cleanup_order(self):
        self.mock_command.stop()
        for outcome in ("success", "failure", "timeout", "interrupt"):
            with self.subTest(outcome=outcome):
                process = Mock()
                process.returncode = 17 if outcome == "failure" else 0
                error = {
                    "timeout": subprocess.TimeoutExpired(["owned"], 30),
                    "interrupt": KeyboardInterrupt(),
                }.get(outcome)
                process.communicate.side_effect = error
                process.communicate.return_value = ("output", None)
                with patch.object(run.os, "name", "posix"), \
                        patch.object(run.subprocess, "Popen", return_value=process) as launch, \
                        patch.object(run, "stop_group") as stop:
                    if outcome == "success":
                        self.assertEqual(run.command(
                            ["owned"], env={}, timeout=30,
                        ).stdout, "output")
                    else:
                        expected = type(error) if error else subprocess.CalledProcessError
                        with self.assertRaises(expected):
                            run.command(["owned"], env={}, timeout=30)
                    self.assertTrue(launch.call_args.kwargs["start_new_session"])
                    stop.assert_called_once_with(process)

    # Scenario: An owned group ignores TERM, or has already exited before cleanup.
    # Guarantees: Cleanup escalates only for that group and always joins the direct child.
    def test_group_escalation(self):
        process = Mock(pid=12345)
        with patch.object(run.os, "killpg", create=True) as kill, \
                patch.object(run.signal, "SIGKILL", 9, create=True), \
                patch.object(run.time, "monotonic", side_effect=[0, 6]):
            run.stop_group(process)
        self.assertEqual(kill.call_args_list, [
            call(12345, run.signal.SIGTERM), call(12345, 0),
            call(12345, 9),
        ])
        process.wait.assert_called_once_with(timeout=5)
        process.reset_mock()
        with patch.object(run.os, "killpg", create=True, side_effect=ProcessLookupError):
            run.stop_group(process)
        process.wait.assert_called_once_with(timeout=5)


@unittest.skipUnless(sys.platform == "linux", "real owned process groups require the Linux CI runner")
class ProcessTreeTests(unittest.TestCase):
    def exercise(self, outcome, ignore_term=False):
        with tempfile.TemporaryDirectory(prefix="otel-pg-process-test-") as directory:
            root = Path(directory)
            script = r"""
import os, pathlib, signal, subprocess, sys, time
root = pathlib.Path(sys.argv[1])
ignore = sys.argv[3] == "1"
if ignore:
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
if sys.argv[2] == "child":
    (root / "ready.tmp").write_text(str(os.getpid()))
    (root / "ready.tmp").rename(root / "ready")
    while True:
        signal.pause()
child = subprocess.Popen(
    [sys.executable, __file__, str(root), "child", sys.argv[3]],
    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
)
def terminate(_signum, _frame):
    child.wait(timeout=10)
    sys.exit(0)
if not ignore:
    signal.signal(signal.SIGTERM, terminate)
while not (root / "go").exists():
    time.sleep(0.01)
if sys.argv[4] == "failure":
    sys.exit(17)
while True:
    signal.pause()
"""
            path = root / "tree.py"
            path.write_text(script, encoding="utf-8")
            original_popen = subprocess.Popen
            captured = []

            def launch(*args, **kwargs):
                process = original_popen(*args, **kwargs)
                captured.append(process)
                communicate = process.communicate

                def at_ready(timeout):
                    deadline = time.monotonic() + 30
                    while not (root / "ready").exists():
                        if time.monotonic() >= deadline or process.poll() is not None:
                            self.fail("owned child did not signal readiness")
                        time.sleep(0.01)
                    (root / "go").touch()
                    if outcome == "interrupt":
                        raise KeyboardInterrupt()
                    return communicate(timeout=0 if outcome == "timeout" else timeout)

                process.communicate = at_ready
                return process

            expected = {
                "timeout": subprocess.TimeoutExpired,
                "interrupt": KeyboardInterrupt,
                "failure": subprocess.CalledProcessError,
            }[outcome]
            try:
                with patch.object(run.subprocess, "Popen", side_effect=launch):
                    with self.assertRaises(expected):
                        run.command(
                            [sys.executable, str(path), str(root), "parent",
                             str(int(ignore_term)), outcome],
                            env=os.environ.copy(), timeout=30,
                        )
                self.assertIsNotNone(captured[0].poll(), "direct child reaped")
                pid = int((root / "ready").read_text())
                status = Path(f"/proc/{pid}/stat")
                deadline = time.monotonic() + 5
                while status.exists():
                    try:
                        if status.read_text().split(") ", 1)[1][0] == "Z":
                            break
                    except FileNotFoundError:
                        break
                    if time.monotonic() >= deadline:
                        self.fail("grandchild must not remain alive")
                    time.sleep(0.01)
            finally:
                if captured:
                    run.stop_group(captured[0])

    # Scenario: A command times out after its grandchild signals readiness.
    # Guarantees: The real command and descendant are terminated and the direct child is joined.
    def test_timeout_tree(self):
        self.exercise("timeout")

    # Scenario: An interrupt arrives while an owned command tree is running.
    # Guarantees: KeyboardInterrupt propagates only after the real process tree is stopped.
    def test_interrupted_tree(self):
        self.exercise("interrupt")

    # Scenario: A failed direct child leaves its grandchild running.
    # Guarantees: Failure cleanup terminates descendants even though their parent has exited.
    def test_failed_parent_tree(self):
        self.exercise("failure")

    # Scenario: Both command and grandchild ignore graceful termination.
    # Guarantees: Bounded escalation kills the full owned group rather than leaving live descendants.
    def test_timeout_escalates(self):
        self.exercise("timeout", ignore_term=True)


if __name__ == "__main__":
    unittest.main()
