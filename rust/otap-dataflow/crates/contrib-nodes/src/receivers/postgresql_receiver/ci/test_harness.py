#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Harness contract checks using fake external commands, not live-DB evidence."""

import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


CI_DIR = Path(__file__).resolve().parent
ROOT = CI_DIR.parents[7]
BASH = shutil.which("bash")
TEST_NAME = (
    "receivers::postgresql_receiver::worker::tls_tests::"
    "live_scalar_and_snapshot_native_execution"
)


def shell_path(path):
    if os.name == "nt":
        return subprocess.check_output(
            [BASH, "-c", 'cygpath -u "$1"', "bash", str(path)], text=True
        ).strip()
    return str(path)


class HarnessTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.artifacts = self.directory / "artifacts"
        self.artifacts.mkdir()
        self.calls = self.directory / "calls"
        self.stubs = self.directory / "stubs.sh"
        self.stubs.write_text(
            r"""
docker() {
  printf '%s\n' "$*" >> "$CALL_LOG"
  case " $* " in
    *" up "*)
      printf 'test CA\n' > "$POSTGRESQL_FIXTURE_EXPORT_DIR/ca.pem"
      printf 'otel_reader\n' > "$POSTGRESQL_FIXTURE_EXPORT_DIR/username"
      printf 'otel-reader-password\n' > "$POSTGRESQL_FIXTURE_EXPORT_DIR/password"
      return "${STARTUP_STATUS:-0}"
      ;;
    *" port "*) printf '127.0.0.1:49152\n' ;;
    *" ps "*) printf '[]\n' ;;
    *" down "*) return "${CLEANUP_STATUS:-0}" ;;
    *) echo "Unexpected Docker operation" >&2; return 99 ;;
  esac
}
cargo() {
  printf 'cargo %s\n' "$*" >> "$CALL_LOG"
  test "$OTEL_ARROW_POSTGRESQL_TEST_PORT" = 49152 || return 98
  test -s "$OTEL_ARROW_POSTGRESQL_TEST_CA_FILE" || return 98
  case " $* " in
    *" --list "*)
      if [[ "${MISSING_TEST:-0}" == 0 ]]; then
        printf '%s: test\n' "$TEST_NAME"
      fi
      ;;
    *)
      test "${TEST_STATUS:-0}" = 0 || return "$TEST_STATUS"
      printf 'test result: ok. %s passed; 0 failed; 0 ignored; 0 measured\n' "${PASS_COUNT:-1}"
      ;;
  esac
}
timeout() {
  printf 'timeout %s\n' "$*" >> "$CALL_LOG"
  shift 3
  "$@"
}
python3() {
  printf 'python3 %s\n' "$*" >> "$CALL_LOG"
  [[ "$1" == */ci/test_harness.py ]]
}
""",
            encoding="utf-8",
            newline="\n",
        )
        self.environment = {
            **os.environ,
            "BASH_ENV": shell_path(self.stubs),
            "CALL_LOG": shell_path(self.calls),
            "RUNNER_TEMP": shell_path(self.directory),
            "OTEL_ARROW_INTEGRATION_ARTIFACT_DIR": shell_path(self.artifacts),
            "TEST_NAME": TEST_NAME,
        }

    def run_harness(self, **overrides):
        result = subprocess.run(
            [BASH, shell_path(CI_DIR / "run.sh")],
            cwd=ROOT,
            env={**self.environment, **overrides},
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
        calls = self.calls.read_text(encoding="utf-8")
        self.assertIn(" down --volumes", calls)
        self.assertNotIn(" logs ", calls)
        self.assertNotIn(" config ", calls)
        return result, calls

    def assert_fixture_removed(self):
        self.assertFalse(list(self.directory.glob("otel-pg-ci.*")))
        self.assertFalse((self.artifacts / "fixture-directory").exists())
        self.assertFalse((self.artifacts / "compose-project").exists())
        for artifact in self.artifacts.iterdir():
            self.assertNotIn("otel-reader-password", artifact.read_text())
            self.assertNotIn("postgres-admin", artifact.read_text())

    # Scenario: The PostgreSQL receiver or shared lockfile changes.
    # Guarantees: Discovery selects the advisory Ubuntu manifest, not unrelated changes.
    def test_discovery(self):
        spec = importlib.util.spec_from_file_location(
            "discovery", ROOT / ".github/scripts/discover-contrib-integration-tests.py"
        )
        discovery = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(discovery)
        manifests = discovery.load_manifests()
        manifest_path = (CI_DIR / "integration-test.json").relative_to(ROOT).as_posix()
        for path in (
            manifest_path,
            manifest_path.replace("ci/integration-test.json", "worker.rs"),
            "rust/otap-dataflow/Cargo.lock",
        ):
            selected = discovery.select_manifests(manifests, [path], False)
            manifest = next(item for item in selected if item["id"] == "postgresql-receiver")
            self.assertEqual(manifest["runner"], "ubuntu-24.04")
            self.assertFalse(manifest["required"])
        self.assertEqual(discovery.select_manifests(manifests, ["unrelated.txt"], False), [])
        manifest = json.loads((CI_DIR / "integration-test.json").read_text())
        self.assertEqual(manifest["command"], ["bash", manifest_path.replace("integration-test.json", "run.sh")])

    # Scenario: The fixture starts and the selected native test passes.
    # Guarantees: Bounded commands execute the exact ignored test and remove only owned state.
    def test_success(self):
        result, calls = self.run_harness()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("--ignored --exact --nocapture", calls)
        self.assertIn("/ci/test_harness.py", calls)
        self.assertIn("--wait --wait-timeout 120", calls)
        self.assertIn("--signal=TERM --kill-after=15s 30m cargo test", calls)
        self.assertIn("--signal=TERM --kill-after=15s 2m cargo test", calls)
        self.assertRegex(calls, r"--project-name otel-pg-ci-[0-9a-f]{16} ")
        self.assert_fixture_removed()

    # Scenario: Provisioning fails after creating fixture state.
    # Guarantees: No test runs, the failure propagates, and owned resources are cleaned.
    def test_startup_failure(self):
        result, calls = self.run_harness(STARTUP_STATUS="17")
        self.assertEqual(result.returncode, 17, result.stderr)
        self.assertNotIn("cargo test", calls)
        self.assert_fixture_removed()

    # Scenario: A stale filter selects nothing or the harness reports zero executed tests.
    # Guarantees: Neither missing nor no-op live coverage can produce a successful CI result.
    def test_no_op_rejected(self):
        for override in ({"MISSING_TEST": "1"}, {"PASS_COUNT": "0"}):
            with self.subTest(override=override):
                result, _ = self.run_harness(**override)
                self.assertNotEqual(result.returncode, 0)
                self.assert_fixture_removed()

    # Scenario: The native test fails.
    # Guarantees: Its failure remains nonzero and does not prevent fixture cleanup.
    def test_test_failure(self):
        result, _ = self.run_harness(TEST_STATUS="19")
        self.assertEqual(result.returncode, 19, result.stderr)
        self.assert_fixture_removed()

    # Scenario: Docker refuses to remove the owned project after a passing test.
    # Guarantees: Cleanup failure fails the command and preserves state for a second cleanup.
    def test_cleanup_failure(self):
        result, _ = self.run_harness(CLEANUP_STATUS="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue((self.artifacts / "compose-project").is_file())
        cleanup = subprocess.run(
            [BASH, shell_path(CI_DIR / "cleanup.sh")],
            env=self.environment,
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
        self.assertEqual(cleanup.returncode, 0, cleanup.stderr)
        self.assert_fixture_removed()


if __name__ == "__main__":
    unittest.main()
