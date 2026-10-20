#!/usr/bin/env python3
"""Offline owner-coverage tests using only the Python standard library.

Run: python3 .github/workflows/scripts/test_report_owner_coverage.py
"""

import io
import json
import os
import tempfile
import unittest
import urllib.error
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest.mock import Mock, patch

import report_owner_coverage as reporter


class OwnerCoverageTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        nodes = "rust/otap-dataflow/crates/contrib-nodes/src"
        self.receiver = f"{nodes}/receivers/source"
        self.exporter = f"{nodes}/exporters/sink"
        self.extensions = "rust/otap-dataflow/crates/contrib-extensions"
        self.write_owners(self.receiver, ["Alice"])
        self.write_owners(self.exporter, ["Bob"])

    def write_owners(self, directory, owners):
        path = self.root / directory / "metadata.yaml"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            f"status:\n  codeowners:\n    active: [{', '.join(owners)}]\n",
            encoding="ascii",
        )
        return path

    def files(self, *directories):
        return [
            {"filename": f"{directory}/lib.rs", "status": "modified"}
            for directory in directories
        ]

    def pull(self, files, author="contributor"):
        return {
            "number": 42,
            "user": {"login": author},
            "changed_files": len(files),
            "state": "open",
            "base": {"ref": "main"},
            "head": {"sha": "test-head"},
        }

    def review(self, login, state="APPROVED", number=1):
        return {
            "id": number,
            "user": {"login": login},
            "state": state,
            "submitted_at": (
                None if state == "PENDING" else f"2026-01-01T00:00:{number:02d}Z"
            ),
        }

    def report(self, files, reviews=(), author="contributor"):
        with redirect_stdout(io.StringIO()):
            return reporter.owner_report(
                self.pull(files, author), files, list(reviews), self.root
            )

    # Scenario: An owner authors changes to their own and someone else's nodes.
    # Guarantees: Case-insensitive authorship covers only owned paths, not the whole PR.
    def test_owner_authorship_is_scoped_to_owned_paths(self):
        files = self.files(self.receiver)
        state, _, summary = self.report(files, author="ALICE")
        self.assertEqual(state, "success")
        self.assertIn("author @alice", summary)
        self.assertNotIn("approved by @alice", summary)
        self.assertEqual(self.report(files)[0], "failure")

        state, description, summary = self.report(
            self.files(self.receiver, self.exporter), author="ALICE"
        )
        self.assertEqual(state, "failure")
        self.assertEqual(description, "Owner coverage missing: exporters/sink")
        self.assertIn(f"`{self.receiver}`: owner coverage author @alice.", summary)
        self.assertIn(f"`{self.exporter}`: owner coverage missing.", summary)

    # Scenario: Owner and non-owner reviews accompany a mixed-ownership PR.
    # Guarantees: Only listed owners cover their paths; independent review is distinct from authorship.
    def test_owner_approval_supplies_missing_coverage(self):
        files = self.files(self.receiver, self.exporter)
        self.assertEqual(
            self.report(files, [self.review("outsider")], author="Alice")[0],
            "failure",
        )
        state, _, summary = self.report(
            files, [self.review("BOB")], author="Alice"
        )
        self.assertEqual(state, "success")
        self.assertIn("author @alice", summary)
        self.assertIn("approved by @bob", summary)
        self.assertEqual(
            reporter.effective_approvals([self.review("ALICE")], "alice"), set()
        )

    # Scenario: Reviews arrive out of order and an owner's decision changes.
    # Guarantees: Comments/pending reviews preserve approval; dismissal or changes requested remove it until reapproval.
    def test_latest_decisive_review_controls_coverage(self):
        files = self.files(self.exporter)
        approved = self.review("Bob")
        for state in ("COMMENTED", "PENDING", "DISMISSED", "CHANGES_REQUESTED"):
            with self.subTest(state=state):
                reviews = [self.review("BOB", state, 2), approved]
                expected = (
                    "success" if state in {"COMMENTED", "PENDING"} else "failure"
                )
                self.assertEqual(self.report(files, reviews)[0], expected)
                reviews.insert(0, self.review("bob", number=3))
                self.assertEqual(self.report(files, reviews)[0], "success")

    # Scenario: An extension inherits crate owners and later has its own metadata.
    # Guarantees: Inheritance covers the affected extension, but local owners take precedence.
    def test_inherited_ownership_and_local_override(self):
        extension = f"{self.extensions}/src/auth"
        self.write_owners(self.extensions, ["Alice"])
        files = self.files(extension)
        state, _, summary = self.report(files, author="ALICE")
        self.assertEqual(state, "success")
        self.assertIn(f"`{extension}`: owner coverage author @alice.", summary)

        self.write_owners(extension, ["Bob"])
        self.assertEqual(self.report(files, author="Alice")[0], "failure")
        self.assertEqual(self.report(files, [self.review("Bob")])[0], "success")

    # Scenario: Ownership is missing, malformed locally, empty, or lists only a team.
    # Guarantees: Unusable ownership stays an error and never falls back to a valid ancestor.
    def test_unavailable_ownership_is_an_error(self):
        extension = f"{self.extensions}/src/auth"
        self.write_owners(self.extensions, ["Alice"])
        local = self.write_owners(extension, ["Bob"])
        for content in (
            "status:\n  codeowners:\n    active: Alice\n",
            "status:\n  codeowners:\n    active: []\n",
            "status:\n  codeowners:\n    active: [org/team]\n",
        ):
            with self.subTest(metadata=content):
                local.write_text(content, encoding="ascii")
                state, description, summary = self.report(
                    self.files(extension), author="Alice"
                )
                self.assertEqual(state, "error")
                self.assertEqual(description, "Owner coverage unknown: extensions/auth")
                self.assertIn("ownership unavailable", summary)

        unknown = f"{self.receiver}-unknown"
        state, description, summary = self.report(
            self.files(self.exporter, unknown)
        )
        self.assertEqual(state, "error")
        self.assertEqual(description, "Owner coverage unknown: receivers/source-unknown")
        self.assertIn(f"`{self.exporter}`: owner coverage missing.", summary)

    # Scenario: A rename crosses owners while other changes touch only core paths.
    # Guarantees: Both rename paths need coverage; core-only changes do not need contrib ownership.
    def test_rename_and_core_path_scope(self):
        renamed = [{
            "filename": f"{self.exporter}/lib.rs",
            "previous_filename": f"{self.receiver}/lib.rs",
            "status": "renamed",
        }]
        self.assertEqual(self.report(renamed, author="Alice")[0], "failure")
        state, _, summary = self.report(
            renamed, [self.review("Bob")], author="Alice"
        )
        self.assertEqual(state, "success")
        self.assertIn(f"`{self.receiver}`", summary)
        self.assertIn(f"`{self.exporter}`", summary)

        core = self.files("rust/otap-dataflow/crates/engine/src")
        self.assertEqual(
            self.report(core)[:2], ("success", "No contrib components affected")
        )
        self.assertEqual(
            self.report(core + self.files(self.receiver), author="Alice")[0], "success"
        )

    # Scenario: Descriptions fit exactly, overflow, or contain a very long node name.
    # Guarantees: Status text stays within 140 characters without losing full paths in the summary.
    def test_description_limit_and_full_summary(self):
        for issue in ("missing", "unknown"):
            with self.subTest(issue=issue):
                prefix = f"Owner coverage {issue}: "
                name = "x" * (140 - len(prefix))
                self.assertEqual(
                    reporter.coverage_description(issue, [name]), prefix + name
                )
                self.assertEqual(
                    reporter.coverage_description(issue, [name + "x"]),
                    f"Owner coverage {issue} for 1 component(s)",
                )

        directory = f"{self.exporter}-" + "x" * 130
        self.write_owners(directory, ["Bob"])
        state, description, summary = self.report(self.files(directory))
        self.assertEqual(state, "failure")
        self.assertEqual(description, "Owner coverage missing for 1 component(s)")
        self.assertIn(f"`{directory}`", summary)

    # Scenario: GitHub's file list does not contain all files declared by the PR.
    # Guarantees: Partial inputs cannot produce a misleading successful coverage report.
    def test_incomplete_file_list_is_rejected(self):
        files = self.files(self.receiver, self.exporter)
        with self.assertRaisesRegex(ValueError, "complete PR file list"):
            reporter.owner_report(self.pull(files), files[:1], [], self.root)

    # Scenario: Reporting finds coverage, missing coverage, or an API failure.
    # Guarantees: Red coverage remains advisory; API failure publishes error and fails the run, with no real requests.
    def test_status_publication_and_api_failure(self):
        files = self.files(self.receiver)
        cases = (
            (self.report(files, author="Alice"), 0),
            (self.report(files), 0),
            (None, 1),
        )
        event = self.root / "event.json"
        event.write_text(json.dumps({"pull_request": {"number": 42}}))
        summary = self.root / "summary.md"
        environment = {
            "GITHUB_REPOSITORY": "open-telemetry/otel-arrow",
            "GITHUB_TOKEN": "offline-fixture",
            "GITHUB_EVENT_NAME": "pull_request_target",
            "GITHUB_EVENT_PATH": str(event),
            "GITHUB_SERVER_URL": "https://github.com",
            "GITHUB_RUN_ID": "123",
            "GITHUB_STEP_SUMMARY": str(summary),
        }
        for report, exit_code in cases:
            with self.subTest(report=report):
                client = Mock()
                client.request.return_value = self.pull(files)
                client.paginated.side_effect = (
                    [files, []] if report else urllib.error.URLError("API unavailable")
                )
                with (
                    patch.dict(os.environ, environment),
                    patch.object(reporter, "GitHubClient", return_value=client),
                    patch.object(reporter, "owner_report", return_value=report) as evaluate,
                    redirect_stdout(io.StringIO()),
                    redirect_stderr(io.StringIO()) as errors,
                ):
                    self.assertEqual(reporter.main(), exit_code)

                method, path, status = client.request.call_args.args
                self.assertEqual((method, path), ("POST", "/statuses/test-head"))
                self.assertEqual(status["context"], "contrib-owner-coverage")
                self.assertEqual(
                    status["target_url"],
                    "https://github.com/open-telemetry/otel-arrow/actions/runs/123",
                )
                if report:
                    evaluate.assert_called_once_with(
                        self.pull(files), files, [],
                        Path(reporter.__file__).resolve().parents[3],
                    )
                    self.assertEqual(status["state"], report[0])
                    self.assertEqual(status["description"], report[1])
                    self.assertEqual(summary.read_text(), report[2])
                else:
                    evaluate.assert_not_called()
                    self.assertEqual(status["state"], "error")
                    self.assertIn("::error::", errors.getvalue())
                    self.assertIn("could not be evaluated", summary.read_text())


if __name__ == "__main__":
    unittest.main()
