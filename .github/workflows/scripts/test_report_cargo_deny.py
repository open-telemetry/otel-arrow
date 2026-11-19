#!/usr/bin/env python3
"""Offline tests: python3 .github/workflows/scripts/test_report_cargo_deny.py."""

import unittest
from unittest.mock import Mock

from report_cargo_deny import report


class CargoDenyIssueTests(unittest.TestCase):
    def setUp(self):
        self.client = Mock()
        self.client.paginated.return_value = []
        self.client.request.return_value = {"html_url": "https://example.test/issues/1"}
        self.run_url = "https://example.test/actions/runs/42"

    def issue(self, workspace="otap-dataflow", login="github-actions[bot]"):
        return {
            "number": 1,
            "html_url": "https://example.test/issues/1",
            "body": f"<!-- cargo-deny: {workspace} -->\nPrevious findings",
            "user": {"login": login},
            "labels": [{"name": "triaged"}],
        }

    # Scenario: A workspace fails without an existing automated tracking issue.
    # Guarantees: One labeled issue links to the run without touching other workspaces or PRs.
    def test_creates_issue_only_for_matching_workspace(self):
        pull = dict(self.issue(), pull_request={})
        self.client.paginated.return_value = [
            self.issue("contrib/data_engine"),
            self.issue(login="contributor"),
            pull,
        ]
        report(self.client, "otap-dataflow", "failure", self.run_url)
        method, path, body = self.client.request.call_args.args
        self.assertEqual((method, path), ("POST", "/issues"))
        self.assertEqual(
            body["title"], "Dependency Audit Failures: rust/otap-dataflow (automated)"
        )
        self.assertIn(self.run_url, body["body"])
        self.assertIn("<!-- cargo-deny: otap-dataflow -->", body["body"])
        self.assertEqual(body["labels"], ["area:security", "dependencies"])
        self.client.request.assert_called_once()

    # Scenario: A later audit fails while the workspace already has an open issue.
    # Guarantees: The report and required labels refresh without removing triage labels or duplicating issues.
    def test_repeated_failure_updates_existing_issue(self):
        self.client.paginated.return_value = [self.issue()]
        report(self.client, "otap-dataflow", "failure", self.run_url)
        method, path, body = self.client.request.call_args.args
        self.assertEqual((method, path), ("PATCH", "/issues/1"))
        self.assertEqual(
            body["title"], "Dependency Audit Failures: rust/otap-dataflow (automated)"
        )
        self.assertIn(self.run_url, body["body"])
        self.assertNotIn("state", body)
        self.assertEqual(body["labels"], ["area:security", "dependencies", "triaged"])
        self.client.request.assert_called_once()

    # Scenario: All dependency checks pass after a previous failure.
    # Guarantees: The open issue closes with a passing-run link and retains its previous report.
    def test_success_closes_issue(self):
        self.client.paginated.return_value = [self.issue()]
        report(self.client, "otap-dataflow", "success", self.run_url)
        method, path, body = self.client.request.call_args.args
        self.assertEqual((method, path), ("PATCH", "/issues/1"))
        self.assertEqual(body["state"], "closed")
        self.assertIn("Previous findings", body["body"])
        self.assertIn(self.run_url, body["body"])

    # Scenario: A clean workspace has no open tracking issue.
    # Guarantees: Successful audits do not create issues or perform writes.
    def test_success_without_issue_does_not_write(self):
        report(self.client, "contrib/data_engine", "success", self.run_url)
        self.client.request.assert_not_called()

    # Scenario: A scan is skipped, cancelled, or supplied an unknown workspace.
    # Guarantees: Incomplete or invalid checks cannot create or resolve issues.
    def test_invalid_input_does_not_write(self):
        for workspace, outcome in [
            ("otap-dataflow", "skipped"),
            ("otap-dataflow", "cancelled"),
            ("unknown", "success"),
        ]:
            with self.subTest(workspace=workspace, outcome=outcome):
                with self.assertRaises(ValueError):
                    report(self.client, workspace, outcome, self.run_url)
        self.client.request.assert_not_called()
        self.client.paginated.assert_not_called()

    # Scenario: More than one open issue already claims the same workspace.
    # Guarantees: Reporting fails visibly instead of updating an arbitrary issue.
    def test_duplicate_issues_are_rejected(self):
        self.client.paginated.return_value = [
            self.issue(),
            dict(self.issue(), number=2),
        ]
        with self.assertRaisesRegex(ValueError, "Multiple open"):
            report(self.client, "otap-dataflow", "failure", self.run_url)
        self.client.request.assert_not_called()

    # Scenario: GitHub cannot list or update the tracking issue.
    # Guarantees: Reporting errors remain visible instead of pretending remediation is tracked.
    def test_api_errors_propagate(self):
        self.client.paginated.side_effect = RuntimeError("API unavailable")
        with self.assertRaisesRegex(RuntimeError, "API unavailable"):
            report(self.client, "otap-dataflow", "failure", self.run_url)
        self.client.request.assert_not_called()


if __name__ == "__main__":
    unittest.main()
