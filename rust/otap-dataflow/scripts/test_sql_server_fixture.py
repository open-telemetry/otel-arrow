#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Fixture and integration-runner tests; no database, Docker, or license acceptance."""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location(
    "sql_server_fixture", Path(__file__).with_name("sql-server-fixture.py")
)
FIXTURE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FIXTURE)

CI_SPEC = importlib.util.spec_from_file_location(
    "sql_server_ci",
    Path(__file__).resolve().parents[1] / "crates" / "contrib-nodes" / "src"
    / "receivers" / "sql_server_receiver" / "ci" / "run.py",
)
CI = importlib.util.module_from_spec(CI_SPEC)
CI_SPEC.loader.exec_module(CI)


class PrerequisiteTests(unittest.TestCase):
    # Scenario: The fixture permits non-TLS connections only through an explicit testing flag.
    # Guarantees: Default setup forces TLS, and the opt-in retains the certificate/TLS configuration.
    def test_unencrypted_fixture_mode_is_explicit(self):
        secure = FIXTURE.server_tls_configuration()
        testing = FIXTURE.server_tls_configuration(allow_unencrypted=True)
        self.assertIn("forceencryption = 1\n", secure)
        self.assertEqual(
            testing, secure.replace("forceencryption = 1\n", "forceencryption = 0\n")
        )
        self.assertIn("tlscert = ", testing)
        self.assertIn("tlskey = ", testing)

    # Scenario: OpenSSL is explicitly available on the user's PATH.
    # Guarantees: The user's selected executable wins over Git installation discovery.
    def test_path_executable_takes_precedence(self):
        with mock.patch.object(FIXTURE.shutil, "which", return_value="selected-openssl") as which:
            self.assertEqual(FIXTURE.find_executable("openssl"), "selected-openssl")
            which.assert_called_once_with("openssl")

    # Scenario: OpenSSL is absent from PATH but bundled in the Program Files Git installation.
    # Guarantees: Windows users can run the fixture without installing another OpenSSL package.
    def test_finds_git_openssl_without_path_entry(self):
        with tempfile.TemporaryDirectory() as temporary:
            executable = Path(temporary) / "Git" / "mingw64" / "bin" / "openssl.exe"
            executable.parent.mkdir(parents=True)
            executable.touch()
            with (
                mock.patch.object(FIXTURE.sys, "platform", "win32"),
                mock.patch.object(FIXTURE.shutil, "which", return_value=None),
                mock.patch.dict(os.environ, {"ProgramFiles": temporary}, clear=True),
            ):
                self.assertEqual(FIXTURE.find_executable("openssl"), str(executable))

    # Scenario: Git is installed outside the default Windows installation paths.
    # Guarantees: OpenSSL can be located relative to the discovered Git executable.
    def test_finds_openssl_relative_to_git(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            executable = root / "usr" / "bin" / "openssl.exe"
            executable.parent.mkdir(parents=True)
            executable.touch()
            with (
                mock.patch.object(FIXTURE.sys, "platform", "win32"),
                mock.patch.object(
                    FIXTURE.shutil, "which",
                    side_effect=lambda name: str(root / "cmd" / "git.exe") if name == "git" else None,
                ),
                mock.patch.dict(os.environ, {}, clear=True),
            ):
                self.assertEqual(FIXTURE.find_executable("openssl"), str(executable))

    # Scenario: OpenSSL is absent from both PATH and the supported installation locations.
    # Guarantees: Setup names the missing prerequisite before creating a partial fixture directory.
    def test_missing_openssl_fails_before_creating_directory(self):
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary) / "fixture"
            with (
                mock.patch.object(FIXTURE.sys, "platform", "linux"),
                mock.patch.object(FIXTURE.shutil, "which", return_value=None),
                mock.patch.object(FIXTURE.subprocess, "run") as run,
            ):
                with self.assertRaisesRegex(RuntimeError, "Required executable 'openssl'"):
                    FIXTURE.start(target)
                self.assertFalse(target.exists())
                run.assert_not_called()

    # Scenario: OpenSSL is available but Docker's executable is not.
    # Guarantees: Setup reports Docker explicitly without producing certificates or a directory.
    def test_missing_docker_fails_before_creating_directory(self):
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary) / "fixture"
            with (
                mock.patch.object(FIXTURE.shutil, "which", side_effect=lambda name: name if name == "openssl" else None),
                mock.patch.object(FIXTURE.subprocess, "run") as run,
            ):
                with self.assertRaisesRegex(RuntimeError, "Required executable 'docker'"):
                    FIXTURE.start(target)
                self.assertFalse(target.exists())
                run.assert_not_called()

    # Scenario: Docker is installed but its daemon is stopped or it uses Windows containers.
    # Guarantees: Setup reports an actionable Linux-container prerequisite before writing files.
    def test_unavailable_or_wrong_mode_docker_fails_preflight(self):
        for result in [
            subprocess.CompletedProcess(["docker"], 1, "", "daemon unavailable"),
            subprocess.CompletedProcess(["docker"], 0, "windows\n", ""),
        ]:
            with self.subTest(returncode=result.returncode, output=result.stdout):
                with tempfile.TemporaryDirectory() as temporary:
                    target = Path(temporary) / "fixture"
                    with (
                        mock.patch.object(FIXTURE.shutil, "which", side_effect=lambda name: name),
                        mock.patch.object(FIXTURE.subprocess, "run", return_value=result),
                    ):
                        with self.assertRaisesRegex(RuntimeError, "Linux-container mode"):
                            FIXTURE.start(target)
                        self.assertFalse(target.exists())

    # Scenario: A subprocess executable disappears between preflight and execution.
    # Guarantees: The error identifies the executable without including secret command arguments.
    def test_missing_subprocess_diagnostic_redacts_arguments(self):
        with mock.patch.object(FIXTURE.subprocess, "run", side_effect=FileNotFoundError):
            with self.assertRaises(RuntimeError) as raised:
                FIXTURE.run(["openssl", "SENSITIVE-argument"])
        self.assertIn("'openssl'", str(raised.exception))
        self.assertNotIn("SENSITIVE", str(raised.exception))


class CleanupTests(unittest.TestCase):
    def fixture(self, directory):
        (directory / "fixture.json").write_text(
            json.dumps({"container": "owned-container-id", "owner": "owner-label"}),
            encoding="utf-8",
        )
        for name in ("reader-password.txt", "setup.sql", "ca.key", "server.key",
                     "original.key", "replacement.key", "expired.key"):
            (directory / name).write_text("generated-secret", encoding="ascii")
        (directory / "unrelated.txt").write_text("keep", encoding="ascii")

    # Scenario: A manifest references a container without the fixture's ownership label.
    # Guarantees: Cleanup refuses before removing any container, secret, or unrelated file.
    def test_mismatched_owner_is_never_removed(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            with mock.patch.object(FIXTURE, "run", return_value="different-owner") as run:
                with self.assertRaisesRegex(RuntimeError, "not owned"):
                    FIXTURE.stop(directory)
                self.assertEqual(run.call_count, 1)
                self.assertEqual(run.call_args.args[0][1:3], ["inspect", "owned-container-id"])
            self.assertTrue((directory / "server.key").exists())
            self.assertTrue((directory / "unrelated.txt").exists())

    # Scenario: The owned fixture is stopped twice.
    # Guarantees: Only its container and named generated secrets are removed; repeat cleanup is inert.
    def test_owned_cleanup_is_scoped_and_idempotent(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            with mock.patch.object(FIXTURE, "run", side_effect=["owner-label", "diagnostics", ""]) as run:
                FIXTURE.stop(directory)
                FIXTURE.stop(directory)
                self.assertEqual(run.call_count, 3)
                self.assertEqual(run.call_args.args[0], ["docker", "rm", "--force", "owned-container-id"])
            for name in ("reader-password.txt", "setup.sql", "ca.key", "server.key",
                         "original.key", "replacement.key", "expired.key"):
                self.assertFalse((directory / name).exists())
            self.assertEqual((directory / "unrelated.txt").read_text(), "keep")
            self.assertEqual((directory / "server.log").read_text(), "diagnostics")

    # Scenario: Docker diagnostics fail after the fixture ownership check succeeds.
    # Guarantees: Owned cleanup still runs and deletes secrets, while the diagnostics error remains visible.
    def test_log_failure_does_not_prevent_owned_cleanup(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            with mock.patch.object(
                FIXTURE, "run", side_effect=["owner-label", RuntimeError("logs failed"), ""]
            ) as run:
                with self.assertRaisesRegex(RuntimeError, "logs failed"):
                    FIXTURE.stop(directory)
                self.assertEqual(run.call_args.args[0], ["docker", "rm", "--force", "owned-container-id"])
            self.assertFalse((directory / "server.key").exists())
            self.assertTrue(json.loads((directory / "fixture.json").read_text())["stopped"])

    # Scenario: Removing a confirmed owned container fails.
    # Guarantees: Its recovery metadata and credentials remain, and cleanup does not claim success.
    def test_failed_container_removal_preserves_recovery_state(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            with mock.patch.object(
                FIXTURE, "run", side_effect=["owner-label", "log", RuntimeError("remove failed")]
            ):
                with self.assertRaisesRegex(RuntimeError, "remove failed"):
                    FIXTURE.stop(directory)
            self.assertTrue((directory / "server.key").exists())
            self.assertNotIn("stopped", json.loads((directory / "fixture.json").read_text()))


class IntegrationRunnerTests(unittest.TestCase):
    # Scenario: CI launches the integration entry point without a live server in unit tests.
    # Guarantees: Fixture variables and the ignored-test selector reach Cargo in the correct workspace.
    def test_live_command_uses_fixture_environment(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture = root / "sql-server-fixture"
            fixture.mkdir()
            (fixture / "fixture.json").write_text(
                json.dumps({"env": {"OTAP_SQL_SERVER_PORT": "12345"}}), encoding="utf-8"
            )
            with (
                mock.patch.dict(os.environ, {
                    "RUNNER_TEMP": str(root),
                    "OTEL_ARROW_INTEGRATION_ARTIFACT_DIR": str(root / "artifacts"),
                }),
                mock.patch.object(CI.sys, "argv", ["run.py", "test"]),
                mock.patch.object(CI.subprocess, "run") as run,
                mock.patch.object(CI, "engine_trial") as trial,
            ):
                CI.main()
            self.assertEqual(run.call_count, 9)
            trial.assert_called_once_with(fixture, root / "artifacts")
            self.assertEqual(CI.SCRIPT.resolve(), Path(FIXTURE.__file__).resolve())
            command = run.call_args_list[2].args[0]
            self.assertIn("sql_server_receiver::live_tests", command)
            self.assertIn("--ignored", command)
            self.assertIn("--locked", command)
            self.assertIn("--test-threads=1", command)
            self.assertEqual(
                [call.args[0][-1] for call in run.call_args_list[3::2]],
                ["replacement", "expired", "valid"],
            )
            self.assertEqual(run.call_args.kwargs["env"]["OTAP_SQL_SERVER_PORT"], "12345")
            self.assertEqual(run.call_args.kwargs["cwd"], CI.WORKSPACE)
            self.assertTrue(run.call_args.kwargs["check"])

    # Scenario: CI fixture setup fails before producing usable environment metadata.
    # Guarantees: The failure propagates and no Cargo live command runs with absent fixture inputs.
    def test_failed_setup_prevents_live_tests(self):
        with tempfile.TemporaryDirectory() as temporary:
            with (
                mock.patch.dict(os.environ, {
                    "RUNNER_TEMP": temporary,
                    "OTEL_ARROW_INTEGRATION_ARTIFACT_DIR": str(Path(temporary) / "artifacts"),
                }),
                mock.patch.object(CI.sys, "argv", ["run.py", "test"]),
                mock.patch.object(
                    CI, "run", side_effect=[None, subprocess.CalledProcessError(1, "fixture")]
                ) as run,
            ):
                with self.assertRaises(subprocess.CalledProcessError):
                    CI.main()
                self.assertEqual(run.call_count, 2)

    # Scenario: A CI path mistake places the fixture inside the uploaded artifact directory.
    # Guarantees: Execution fails before provisioning certificates, credentials, or a container.
    def test_fixture_secrets_cannot_be_artifacts(self):
        with tempfile.TemporaryDirectory() as temporary:
            with (
                mock.patch.dict(os.environ, {
                    "RUNNER_TEMP": temporary,
                    "OTEL_ARROW_INTEGRATION_ARTIFACT_DIR": temporary,
                }),
                mock.patch.object(CI.sys, "argv", ["run.py", "test"]),
                mock.patch.object(CI, "run") as run,
            ):
                with self.assertRaisesRegex(RuntimeError, "outside the artifact"):
                    CI.main()
                run.assert_not_called()

    # Scenario: CI cleanup has both a generated server log and private fixture files.
    # Guarantees: The artifact directory receives only the server log, never fixture credentials.
    def test_cleanup_uploads_only_server_log(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture = root / "sql-server-fixture"
            fixture.mkdir()
            for name in ("fixture.json", "reader-password.txt", "server.key", "server.log"):
                (fixture / name).write_text("test", encoding="ascii")
            artifacts = root / "artifacts"
            with (
                mock.patch.dict(os.environ, {
                    "RUNNER_TEMP": str(root),
                    "OTEL_ARROW_INTEGRATION_ARTIFACT_DIR": str(artifacts),
                }),
                mock.patch.object(CI.sys, "argv", ["run.py", "cleanup"]),
                mock.patch.object(CI, "run") as run,
            ):
                CI.main()
                self.assertEqual(run.call_args.args[0][2], "stop")
            self.assertEqual([path.name for path in artifacts.iterdir()], ["server.log"])


class EngineTrialTests(unittest.TestCase):
    def fixture(self, directory):
        (directory / "fixture.json").write_text(json.dumps({"env": {}}), encoding="utf-8")

    # Scenario: The example engine emits fixture rows, installs its ACK cursor, and shuts down cleanly.
    # Guarantees: CI uses the exact YAML and records only a non-secret result outside the private fixture.
    def test_engine_trial_requires_rows_checkpoint_and_clean_exit(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            artifacts = directory / "artifacts"
            checkpoint = directory / "engine-checkpoints" / "source.checkpoint.1.json"
            checkpoint.parent.mkdir()
            checkpoint.write_text(json.dumps({"payload": {"cursor": {"tie_breaker": 3}}}),
                                  encoding="utf-8")
            process = mock.Mock()
            process.poll.side_effect = [None, 0]
            process.wait.return_value = 0

            def spawn(command, **options):
                self.assertEqual(command[1:3], ["--config", "configs/sql-server-console.yaml"])
                options["stdout"].write("hello\nthird\n")
                return process

            with (
                mock.patch.object(CI, "run") as run,
                mock.patch.object(CI.subprocess, "Popen", side_effect=spawn),
            ):
                CI.engine_trial(directory, artifacts)
                self.assertIn("--locked", run.call_args.args[0])
            process.send_signal.assert_called_once()
            process.terminate.assert_not_called()
            result = json.loads((artifacts / "engine-result.json").read_text())
            self.assertEqual(result["acknowledged_id"], 3)
            self.assertEqual([path.name for path in artifacts.iterdir()], ["engine-result.json"])

    # Scenario: The engine stays alive but never installs acknowledged progress.
    # Guarantees: CI reports failure and reaps only its own child, including a forced kill if necessary.
    def test_checkpoint_timeout_reaps_owned_process(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            process = mock.Mock()
            process.poll.return_value = None
            process.wait.side_effect = [subprocess.TimeoutExpired("engine", 5), 1]
            with (
                mock.patch.object(CI, "run"),
                mock.patch.object(CI.subprocess, "Popen", return_value=process),
                mock.patch.object(CI.time, "monotonic", side_effect=[0, 61]),
            ):
                with self.assertRaisesRegex(RuntimeError, "60 seconds"):
                    CI.engine_trial(directory, directory / "artifacts")
            process.terminate.assert_called_once()
            process.kill.assert_called_once()
            self.assertEqual(process.wait.call_count, 2)
            self.assertFalse((directory / "artifacts" / "engine-result.json").exists())

    # Scenario: A child exits before any acknowledged checkpoint is present.
    # Guarantees: Early engine failure never becomes a successful CI result or a leaked child process.
    def test_early_exit_cannot_pass_trial(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            process = mock.Mock()
            process.poll.return_value = 1
            with (
                mock.patch.object(CI, "run"),
                mock.patch.object(CI.subprocess, "Popen", return_value=process),
            ):
                with self.assertRaisesRegex(RuntimeError, "before installing"):
                    CI.engine_trial(directory, directory / "artifacts")
            process.terminate.assert_not_called()
            self.assertFalse((directory / "artifacts").exists())


class FaultTests(unittest.TestCase):
    def fixture(self, directory):
        state = {"container": "owned-id", "owner": "owner-label",
                 "env": {"OTAP_SQL_SERVER_CERTIFICATE": "valid"}}
        (directory / "fixture.json").write_text(json.dumps(state), encoding="utf-8")
        for name in ("server", "original", "replacement", "expired"):
            for suffix in ("crt", "key"):
                (directory / f"{name}.{suffix}").write_text(name, encoding="ascii")
        return state

    # Scenario: A certificate rotation or idle-disconnect request references an unowned container.
    # Guarantees: Ownership is checked before changing certificate files, restarting, or executing SQL.
    def test_faults_refuse_unowned_containers(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            for operation in (
                lambda: FIXTURE.rotate_certificate(directory, "expired"),
                lambda: FIXTURE.disconnect_readers(directory),
            ):
                with mock.patch.object(FIXTURE, "run", return_value="another-owner") as run:
                    with self.assertRaisesRegex(RuntimeError, "not owned"):
                        operation()
                    self.assertEqual(run.call_count, 1)
            self.assertEqual((directory / "server.crt").read_text(), "server")

    # Scenario: Rotation restarts the owned fixture and Docker assigns a new ephemeral host port.
    # Guarantees: Ready state records the actual port and certificate without changing global trust.
    def test_rotation_updates_state_after_owned_restart(self):
        for variant in ("replacement", "expired", "valid"):
            with self.subTest(variant=variant), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                state = self.fixture(directory)
                with (
                    mock.patch.object(
                        FIXTURE, "run", side_effect=["owner-label", "restarted", "127.0.0.1:53758"]
                    ) as run,
                    mock.patch.object(FIXTURE, "admin_query") as query,
                ):
                    FIXTURE.rotate_certificate(directory, variant)
                self.assertEqual(run.call_args_list[1].args[0], ["docker", "restart", "owned-id"])
                self.assertEqual(run.call_args.args[0], ["docker", "port", "owned-id", "1433/tcp"])
                query.assert_called_once()
                self.assertEqual(query.call_args.args[0]["container"], state["container"])
                self.assertEqual(query.call_args.args[1], "SELECT 1")
                self.assertEqual(query.call_args.kwargs, {"allow_expired": variant == "expired"})
                self.assertEqual((directory / "server.crt").read_text(),
                                 "original" if variant == "valid" else variant)
                state = json.loads((directory / "fixture.json").read_text())
                self.assertEqual(state["env"]["OTAP_SQL_SERVER_CERTIFICATE"], variant)
                self.assertEqual(state["env"]["OTAP_SQL_SERVER_PORT"], "53758")

    # Scenario: Docker reports an unsafe, ambiguous, absent, or out-of-range port mapping.
    # Guarantees: The fixture fails explicitly instead of publishing a misleading connection endpoint.
    def test_invalid_port_mappings_are_rejected(self):
        for mapping in ("", "0.0.0.0:1433", "127.0.0.1:0", "127.0.0.1:65536",
                        "127.0.0.1:1433\n[::]:1433", "127.0.0.1:not-a-port"):
            with self.subTest(mapping=mapping), mock.patch.object(FIXTURE, "run", return_value=mapping):
                with self.assertRaises((RuntimeError, ValueError)):
                    FIXTURE.published_port("docker", "owned-id")

    # Scenario: A restarted server is ready, but Docker cannot report its current host port.
    # Guarantees: The rotation reports failure and does not persist a falsely successful new state.
    def test_rotation_port_discovery_failure_preserves_manifest(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            before = (directory / "fixture.json").read_text()
            with (
                mock.patch.object(FIXTURE, "run", side_effect=[
                    "owner-label", "restarted", RuntimeError("port discovery failed"),
                ]),
                mock.patch.object(FIXTURE, "admin_query"),
            ):
                with self.assertRaisesRegex(RuntimeError, "port discovery failed"):
                    FIXTURE.rotate_certificate(directory, "replacement")
            self.assertEqual((directory / "fixture.json").read_text(), before)

    # Scenario: Rotation restarts the owned server but readiness never succeeds.
    # Guarantees: The failure remains visible and the manifest never advertises a successful rotation.
    def test_rotation_readiness_failure_is_not_success(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            with (
                mock.patch.object(FIXTURE, "run", side_effect=["owner-label", "restarted"]),
                mock.patch.object(FIXTURE, "admin_query", side_effect=RuntimeError("not ready")),
                mock.patch.object(FIXTURE.time, "monotonic", side_effect=[0, 121]),
            ):
                with self.assertRaisesRegex(RuntimeError, "120 seconds"):
                    FIXTURE.rotate_certificate(directory, "expired")
            self.assertEqual(json.loads((directory / "fixture.json").read_text())["env"]
                             ["OTAP_SQL_SERVER_CERTIFICATE"], "valid")

    # Scenario: A requested fault disconnects idle receiver sessions in the disposable database.
    # Guarantees: SQL restricts the target login/application/database and cannot select its administrator.
    def test_disconnect_is_scoped_to_fixture_readers(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.fixture(directory)
            with (
                mock.patch.object(FIXTURE, "run", return_value="owner-label"),
                mock.patch.object(FIXTURE, "admin_query") as query,
            ):
                FIXTURE.disconnect_readers(directory)
            sql = query.call_args.args[1]
            for predicate in (
                "login_name = N'otel_reader'", "program_name = N'otel-arrow-sql-server'",
                "database_id = DB_ID(N'otap_fixture')", "session_id <> @@SPID",
            ):
                self.assertIn(predicate, sql)

    # Scenario: Certificate rotation checks readiness after intentionally installing an expired leaf.
    # Guarantees: Only that explicit check bypasses trust; admin credentials never enter host command arguments.
    def test_admin_readiness_bypass_is_explicit(self):
        for expired in (False, True):
            with mock.patch.object(FIXTURE, "run") as run:
                FIXTURE.admin_query({"container": "owned-id"}, "SELECT 1", allow_expired=expired)
                command = run.call_args.args[0]
                self.assertEqual(command[:3], ["docker", "exec", "owned-id"])
                self.assertEqual(" -C" in command[5], expired)
                self.assertIn('$MSSQL_SA_PASSWORD', command[5])
                self.assertEqual(command[-1], "SELECT 1")


if __name__ == "__main__":
    unittest.main()
