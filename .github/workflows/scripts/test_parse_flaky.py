#!/usr/bin/env python3
"""Tests for flaky-test report history and formatting."""

import importlib.util
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


if __name__ == "__main__":
    unittest.main()
