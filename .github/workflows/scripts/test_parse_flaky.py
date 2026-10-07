#!/usr/bin/env python3
"""Tests for flaky-test report history and formatting."""

import importlib.util
import tempfile
import unittest
from datetime import date
from pathlib import Path


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
