#!/usr/bin/env python3
"""Tests for flaky-test job links, report history, and formatting."""

import importlib.util
import io
import subprocess
import tempfile
import unittest
from contextlib import redirect_stderr
from datetime import date
from pathlib import Path
from unittest.mock import patch


MODULE_PATH = Path(__file__).with_name("parse_flaky.py")
SPEC = importlib.util.spec_from_file_location("parse_flaky", MODULE_PATH)
PARSE_FLAKY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PARSE_FLAKY)


def flaky_test(name):
    """Build a minimal currently observed flaky-test record."""
    return {
        "name": name,
        "reason": "Marked flaky by nextest (1x)",
        "pass_count": 1,
        "fail_count": 1,
        "flaky_direct": 1,
        "fail_messages": [],
        "affected_os": ["ubuntu-latest"],
        "all_os": ["ubuntu-latest"],
        "fail_artifacts": [],
        "fail_job_links": [],
    }


class FlakyJobLinkTests(unittest.TestCase):
    # Scenario: Required jobs use partition-only or legacy full matrices.
    # Guarantees: Required jobs resolve without breaking existing matrix jobs.
    def test_required_job_names_match_current_and_legacy_metadata(self):
        cases = [
            ("test_required_linux", "ubuntu-latest",
             "test_required_linux (2)"),
            ("test_required_windows", "windows-latest",
             "test_required_windows (2)"),
            ("test_required", "ubuntu-latest",
             "test_required (otap-dataflow, ubuntu-latest, 2)"),
            ("test_nonrequired", "macos-latest",
             "test_nonrequired (otap-dataflow, macos-latest, 2)"),
        ]
        for job, os_name, display_name in cases:
            with self.subTest(job=job):
                meta = {
                    "job": job,
                    "os": os_name,
                    "partition": "2",
                    "folder": "otap-dataflow",
                }
                jobs = {
                    ("123", display_name.replace("2)", "3)")): "wrong",
                    ("456", display_name): "wrong-run",
                    ("123", display_name): "https://example.test/job/2",
                }

                self.assertEqual(
                    PARSE_FLAKY._find_job_url(jobs, "123", meta),
                    "https://example.test/job/2",
                )

    # Scenario: Job lookup returns no match or fails despite artifact metadata.
    # Guarantees: A warning and one run link replace an empty Failed Jobs cell.
    def test_unresolved_artifacts_fall_back_to_run_links(self):
        current = flaky_test("crate::flaky")
        current["fail_artifacts"] = [
            ("123", "junit-xml-first"),
            ("123", "junit-xml-second"),
        ]
        metadata = {
            ("123", artifact): {"job": "renamed"}
            for artifact in ("junit-xml-first", "junit-xml-second")
        }
        outcomes = [
            subprocess.CompletedProcess([], 0, "", ""),
            subprocess.CalledProcessError(1, ["gh"], stderr="API denied"),
        ]
        for outcome in outcomes:
            with self.subTest(outcome=type(outcome).__name__):
                with patch.object(
                    PARSE_FLAKY.subprocess, "run",
                    return_value=outcome,
                    side_effect=outcome if isinstance(outcome, Exception) else None,
                ) as api, redirect_stderr(io.StringIO()) as warnings:
                    PARSE_FLAKY.lookup_job_urls(
                        [current], "owner/repo", metadata,
                    )

                self.assertTrue(api.call_args.kwargs["check"])
                self.assertIn("Warning:", warnings.getvalue())
                self.assertEqual(current["fail_job_links"], [
                    ("run #123",
                     "https://github.com/owner/repo/actions/runs/123"),
                ])


class FlakyHistoryTests(unittest.TestCase):
    # Scenario: Downloaded XML files are malformed or use unsupported roots.
    # Guarantees: No file is counted as successfully parsed JUnit input.
    def test_invalid_xml_does_not_count_as_parsed_junit(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            artifacts = Path(temp_dir)
            (artifacts / "malformed.xml").write_text("<testsuite>")
            (artifacts / "unsupported.xml").write_text("<report />")

            _, _, parsed_count = PARSE_FLAKY.parse_junit_files(artifacts)

        self.assertEqual(parsed_count, 0)

    # Scenario: A downloaded XML file has a supported JUnit root element.
    # Guarantees: The parser records that valid JUnit input was processed.
    def test_valid_junit_xml_counts_as_parsed(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            artifacts = Path(temp_dir)
            (artifacts / "junit.xml").write_text(
                '<testsuite><testcase name="passes" /></testsuite>'
            )

            _, _, parsed_count = PARSE_FLAKY.parse_junit_files(artifacts)

        self.assertEqual(parsed_count, 1)

    # Scenario: A report without a history marker is read during migration.
    # Guarantees: Existing table entries remain tracked from the migration day.
    def test_legacy_report_names_migrate_to_last_seen_history(self):
        body = "|  | <code>crate::legacy_test</code> | ubuntu-latest |"

        parsed = PARSE_FLAKY.parse_flaky_history(
            body,
            date(2026, 10, 2),
        )

        self.assertEqual(
            parsed,
            {"crate::legacy_test": date(2026, 10, 2)},
        )

    # Scenario: Tests cross the seven-day retention boundary.
    # Guarantees: Seven-day-old tests remain, while older tests age out.
    def test_retention_boundary(self):
        body = PARSE_FLAKY.format_issue_body(
            [],
            50,
            "",
            {
                "crate::recent_test": date(2026, 9, 25),
                "crate::old_test": date(2026, 9, 24),
            },
            date(2026, 10, 2),
            7,
        )

        self.assertIn(":hourglass_flowing_sand:", body)
        self.assertIn("| 2026-09-25 |", body)
        parsed = PARSE_FLAKY.parse_flaky_history(
            body,
            date(2026, 10, 2),
        )
        self.assertEqual(
            parsed,
            {"crate::recent_test": date(2026, 9, 25)},
        )

    # Scenario: A retained test is observed again in the current sample.
    # Guarantees: Its last-seen date advances to the actual flaky run date.
    def test_current_observation_refreshes_last_seen_date(self):
        current = flaky_test("crate::recurring_test")
        current["fail_artifacts"] = [
            ("123", "junit-xml-required-ubuntu-latest-1")
        ]
        sample_metadata = {
            "selected_runs": 1,
            "xml_count": 1,
            "oldest_run": "2026-10-01T23:00:00Z",
            "newest_run": "2026-10-01T23:00:00Z",
            "run_dates": {"123": "2026-10-01T23:00:00Z"},
        }
        body = PARSE_FLAKY.format_issue_body(
            [current],
            50,
            "",
            {"crate::recurring_test": date(2026, 9, 28)},
            date(2026, 10, 2),
            7,
            sample_metadata,
        )

        parsed = PARSE_FLAKY.parse_flaky_history(
            body,
            date(2026, 10, 2),
        )
        self.assertEqual(
            parsed,
            {"crate::recurring_test": date(2026, 10, 1)},
        )

    # Scenario: Current observations fill the visible table.
    # Guarantees: Omitted observations and retained tests remain in history.
    def test_hidden_history_is_not_limited_by_visible_rows(self):
        current = [
            flaky_test(f"crate::current_test_{index:02d}")
            for index in range(PARSE_FLAKY.MAX_REPORT_TESTS + 1)
        ]
        previous = {"crate::retained_test": date(2026, 10, 1)}

        body = PARSE_FLAKY.format_issue_body(
            current,
            50,
            "",
            previous,
            date(2026, 10, 5),
            7,
        )
        parsed = PARSE_FLAKY.parse_flaky_history(
            body,
            date(2026, 10, 5),
        )

        self.assertEqual(len(parsed), len(current) + 1)
        self.assertIn("crate::current_test_50", parsed)
        self.assertIn("crate::retained_test", parsed)
        self.assertIn("stored only in hidden history", body)

    # Scenario: Seven-day history exceeds its reserved issue-body budget.
    # Guarantees: Report generation fails instead of silently dropping tests.
    def test_history_marker_fails_closed_when_budget_is_exceeded(self):
        current = [
            flaky_test(f"crate::{index:03d}_" + ("x" * 120))
            for index in range(300)
        ]

        with self.assertRaisesRegex(
            ValueError,
            "history marker exceeds",
        ):
            PARSE_FLAKY.format_issue_body(
                current,
                50,
                "",
                {},
                date(2026, 10, 5),
                7,
            )


if __name__ == "__main__":
    unittest.main()
