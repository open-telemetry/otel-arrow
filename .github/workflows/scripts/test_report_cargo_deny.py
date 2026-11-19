#!/usr/bin/env python3
"""Offline tests: python3 .github/workflows/scripts/test_report_cargo_deny.py."""

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from report_cargo_deny import main, report, scan_details


class CargoDenyIssueTests(unittest.TestCase):
    def setUp(self):
        self.client = Mock()
        self.client.paginated.return_value = []
        self.client.request.return_value = {"html_url": "https://example.test/issues/1"}
        self.run_url = "https://example.test/actions/runs/42"
        self.details = "## Findings\n\nAn affected dependency and its remediation."

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
        report(self.client, "otap-dataflow", "failure", self.run_url, self.details)
        method, path, body = self.client.request.call_args.args
        self.assertEqual((method, path), ("POST", "/issues"))
        self.assertEqual(
            body["title"], "Dependency Audit Failures: rust/otap-dataflow (automated)"
        )
        self.assertIn(self.run_url, body["body"])
        self.assertIn(self.details, body["body"])
        self.assertIn("<!-- cargo-deny: otap-dataflow -->", body["body"])
        self.assertEqual(body["labels"], ["area:security", "dependencies"])
        self.client.request.assert_called_once()

    # Scenario: A later audit fails while the workspace already has an open issue.
    # Guarantees: The report and required labels refresh without removing triage labels or duplicating issues.
    def test_repeated_failure_updates_existing_issue(self):
        self.client.paginated.return_value = [self.issue()]
        report(self.client, "otap-dataflow", "failure", self.run_url, self.details)
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
        report(self.client, "otap-dataflow", "success", self.run_url, self.details)
        method, path, body = self.client.request.call_args.args
        self.assertEqual((method, path), ("PATCH", "/issues/1"))
        self.assertEqual(body["state"], "closed")
        self.assertIn("Previous findings", body["body"])
        self.assertIn(self.run_url, body["body"])

    # Scenario: A clean workspace has no open tracking issue.
    # Guarantees: Successful audits do not create issues or perform writes.
    def test_success_without_issue_does_not_write(self):
        report(self.client, "contrib/data_engine", "success", self.run_url, self.details)
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
                    report(self.client, workspace, outcome, self.run_url, self.details)
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
            report(self.client, "otap-dataflow", "failure", self.run_url, self.details)
        self.client.request.assert_not_called()

    # Scenario: GitHub cannot list or update the tracking issue.
    # Guarantees: Reporting errors remain visible instead of pretending remediation is tracked.
    def test_api_errors_propagate(self):
        self.client.paginated.side_effect = RuntimeError("API unavailable")
        with self.assertRaisesRegex(RuntimeError, "API unavailable"):
            report(self.client, "otap-dataflow", "failure", self.run_url, self.details)
        self.client.request.assert_not_called()


class DiagnosticReportTests(unittest.TestCase):
    def setUp(self):
        self.context = {
            "run_url": "https://example.test/actions/runs/42",
            "ref": "feature/dependency-audit",
            "sha": "abc123",
            "scanned_at": "2026-11-19T16:00:00+00:00",
            "artifact_url": "https://example.test/actions/runs/42/artifacts/1",
        }
        self.advisory = {
            "type": "diagnostic",
            "fields": {
                "severity": "error",
                "code": "vulnerability",
                "message": "Example dependency can exhaust host memory",
                "advisory": {
                    "id": "RUSTSEC-2026-0001",
                    "cvss": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:H",
                },
                "notes": [
                    "Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0001",
                    "Solution: Upgrade to >=1.2.3 (try `cargo update -p example-crate`)",
                ],
                "graphs": [
                    {
                        "Krate": {"name": "example-crate", "version": "1.2.0"},
                        "parents": [
                            {"Krate": {"name": "example-host", "version": "0.1.0"}}
                        ],
                    }
                ],
                "labels": [
                    {
                        "line": 12,
                        "column": 1,
                        "span": "example-crate 1.2.0",
                        "message": "security vulnerability detected",
                    }
                ],
            },
        }

    def summary(self, errors=1, warnings=0):
        return {
            "type": "summary",
            "fields": {
                check: {
                    "errors": errors if check == "advisories" else 0,
                    "warnings": warnings if check == "licenses" else 0,
                }
                for check in ("advisories", "bans", "licenses", "sources")
            },
        }

    def render(self, *records, outcome="failure"):
        raw = "\n".join(json.dumps(record) for record in records)
        return scan_details(raw, outcome, self.context)

    # Scenario: A scan emits an advisory and a non-advisory policy warning.
    # Guarantees: Reports retain failure details and link to warnings without listing their diagnostics.
    def test_renders_advisory_and_policy_findings(self):
        policy = {
            "type": "diagnostic",
            "fields": {
                "severity": "warning",
                "code": "license-not-encountered",
                "message": "Unused license policy entry",
                "notes": ["Remove an obsolete allow-list entry if no longer needed."],
            },
        }
        body = self.render(policy, self.advisory, self.summary(warnings=1))
        for expected in (
            "| advisories | 1 |",
            "| licenses | 0 |",
            f"[Warnings omitted: 1; view full report]({self.context['artifact_url']})",
            "example-crate&#64;1.2.0",
            "[RUSTSEC-2026-0001](https://rustsec.org/advisories/RUSTSEC-2026-0001)",
            "Upgrade to &gt;=1.2.3",
            "&lt;- example-host&#64;0.1.0",
            "Line 12:1: example-crate 1.2.0",
            "CVSS vector: CVSS:3.1/",
            self.context["ref"],
            self.context["scanned_at"],
            self.context["artifact_url"],
            "https://example.test/commit/abc123",
        ):
            self.assertIn(expected, body)
        self.assertNotIn("| warning", body)
        self.assertNotIn("Unused license policy entry", body)
        self.assertNotIn("Remove an obsolete allow-list entry", body)

    # Scenario: Cargo-deny fails before finishing dependency checks.
    # Guarantees: Operational errors appear as incomplete scans, not as clean dependency audits.
    def test_reports_operational_failures(self):
        body = self.render({
            "type": "log",
            "fields": {"level": "ERROR", "message": "Unable to fetch advisory database"},
        })
        self.assertIn("Scan did not complete all checks", body)
        self.assertIn("Unable to fetch advisory database", body)
        self.assertIn("scan-error", body)

    # Scenario: A nominally successful process has missing, malformed, or failing diagnostics.
    # Guarantees: Incomplete or contradictory evidence cannot resolve a tracking issue.
    def test_rejects_unproven_success(self):
        for raw in (
            "",
            "not JSON",
            json.dumps(self.advisory),
            json.dumps(self.summary()),
            json.dumps(self.advisory) + "\n" + json.dumps(self.summary(errors=0)),
        ):
            with self.subTest(raw=raw):
                with self.assertRaises(ValueError):
                    scan_details(raw, "success", self.context)

    # Scenario: All checks pass, either cleanly or with a non-blocking policy warning.
    # Guarantees: Warnings do not prevent resolution and are linked only when present.
    def test_accepts_complete_warning_only_success(self):
        warning = {
            "type": "diagnostic",
            "fields": {
                "severity": "warning",
                "code": "license-not-encountered",
                "message": "Unused license policy entry",
            },
        }
        body = self.render(warning, self.summary(errors=0, warnings=1), outcome="success")
        self.assertNotIn("Unused license policy entry", body)
        self.assertIn(
            f"[Warnings omitted: 1; view full report]({self.context['artifact_url']})",
            body,
        )
        self.assertIn("No failing diagnostics were emitted", body)
        self.assertNotIn("Scan did not complete", body)
        clean_body = self.render(self.summary(errors=0), outcome="success")
        self.assertIn("No failing diagnostics were emitted", clean_body)
        self.assertNotIn("Scan did not complete", clean_body)
        self.assertNotIn("Warnings omitted", clean_body)

    # Scenario: Eight advisory failures coexist with 98 non-failing policy warnings.
    # Guarantees: Only the eight failures are detailed; warnings cannot inflate or crowd out failures.
    def test_warning_noise_is_excluded_from_failure_details(self):
        failures = [
            {
                "type": "diagnostic",
                "fields": dict(self.advisory["fields"], message=f"Advisory failure {index}"),
            }
            for index in range(8)
        ]
        warnings = [
            {
                "type": "diagnostic",
                "fields": {
                    "severity": "warning",
                    "code": "duplicate",
                    "message": f"Duplicate dependency warning {index}",
                },
            }
            for index in range(98)
        ]
        summary = self.summary(errors=8)
        for check, count in (("advisories", 2), ("bans", 88), ("licenses", 1), ("sources", 7)):
            summary["fields"][check]["warnings"] = count
        body = self.render(*warnings, *failures, summary)
        self.assertEqual(body.count("| error / vulnerability |"), 8)
        self.assertNotIn("Duplicate dependency warning", body)
        self.assertNotIn("Showing", body)
        self.assertIn(
            f"[Warnings omitted: 98; view full report]({self.context['artifact_url']})",
            body,
        )
        self.assertNotIn("| Warnings |", body)

    # Scenario: Diagnostics contain Markdown delimiters, HTML, and mention-like text.
    # Guarantees: Dependency-provided text cannot break tables, inject HTML, or mention teams.
    def test_escapes_untrusted_diagnostic_text(self):
        self.advisory["fields"]["message"] = "<script>alert(1)</script> | @team [click](bad)"
        body = self.render(self.advisory, self.summary())
        self.assertNotIn("<script>", body)
        self.assertNotIn("@team", body)
        self.assertNotIn("[click](bad)", body.split("<details>", 1)[0])
        self.assertIn("&#124;", body)
        self.assertIn("&#64;team", body)
        self.assertIn("&lt;script&gt;", body)

    # Scenario: Many findings contain unusually large, multibyte descriptions.
    # Guarantees: Issue content stays below GitHub's body limit with a visible full-report link.
    def test_bounds_large_reports(self):
        records = []
        for index in range(50):
            fields = dict(self.advisory["fields"])
            fields["message"] = f"Finding {index}: " + "\u00e9" * 10000
            fields["notes"] = ["Verbose details " * 10000]
            records.append({"type": "diagnostic", "fields": fields})
        body = self.render(*records, self.summary(errors=50))
        self.assertLess(len(body.encode("utf-8")), 60000)
        self.assertIn("Showing", body)
        self.assertIn("truncated; see full report", body)
        self.assertIn(self.context["artifact_url"], body)
        self.assertTrue(body.endswith("</details>"))

    # Scenario: The main entry point reads a truncated report for a successful scan.
    # Guarantees: Validation occurs before constructing a GitHub client or attempting issue writes.
    def test_main_does_not_touch_github_for_incomplete_success(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cargo-deny.jsonl"
            path.write_text(json.dumps(self.advisory), encoding="utf-8")
            environment = {
                "GITHUB_SERVER_URL": "https://example.test",
                "GITHUB_REPOSITORY": "example/repo",
                "GITHUB_RUN_ID": "42",
                "CHECK_OUTCOME": "success",
                "DIAGNOSTICS_PATH": str(path),
                "GITHUB_REF_NAME": "main",
                "GITHUB_SHA": "abc123",
                "REPORT_ARTIFACT_URL": self.context["artifact_url"],
            }
            with patch.dict(os.environ, environment), patch(
                "report_cargo_deny.GitHubClient"
            ) as client:
                with self.assertRaises(ValueError):
                    main()
                client.assert_not_called()


if __name__ == "__main__":
    unittest.main()
