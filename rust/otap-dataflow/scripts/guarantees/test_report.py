"""Regression checks for the gate's evidence and component-inventory decisions."""

import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import report


class GateTests(unittest.TestCase):
    # Scenario: a receiver starts declaring arrival context without a controller change.
    # Guarantees: a reviewer sees the cross-component live-update requirement.
    def test_receiver_declaration_affects_live_control(self):
        path = report.DFE / "crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs"
        rules = json.loads(report.MAP.read_text())
        self.assertIn("live", report.affected(path, "ContextDeclarationProvider::arriving()", rules))
        common = report.DFE / "crates/other/src/transport.rs"
        self.assertIn("live", report.affected(common, "ContextDeclarationProvider::arriving()", rules))

    # Scenario: report files contain a retry, a skipped case, and a same-named test in another crate.
    # Guarantees: a passing unrelated case cannot hide a missing or flaky guarantee.
    def test_junit_requires_exact_successful_case(self):
        with tempfile.TemporaryDirectory() as tmp:
            xml = Path(tmp) / "junit.xml"
            cases = '<testcase name="released_core_allocation_precedence" classname="other"/>'
            for outcome, expected in [("<skipped/>", False), ("<flakyFailure/>", False), ("", True)]:
                xml.write_text('<testsuite>' + cases + '<testcase name="released_core_allocation_precedence" '
                               'classname="otel-arrow-dfe-config::release_guarantees">' + outcome +
                               '</testcase></testsuite>')
                results = report.test_results(Path(tmp))
                self.assertEqual(results["release-precedence"], expected)
                self.assertFalse(results["retry-recovery"])

    # Scenario: a new factory enters the compiled inventory and claims an inherited gap.
    # Guarantees: even a new table row must supply evidence for all three operations.
    def test_new_component_cannot_inherit_untested(self):
        text = report.INVENTORY.read_text()
        old = [r for r in json.loads(text) if r["id"] != "urn:otel:processor:retry"]
        def base(_ref, path):
            return json.dumps(old) if path == report.INVENTORY else None
        with patch.object(report, "read_base", side_effect=base):
            with self.assertRaisesRegex(ValueError, "processor:retry add"):
                report.validate_support("base")

    # Scenario: an exception is proposed or edited in the same change that uses it.
    # Guarantees: only an unchanged decision on the base covering this specific area qualifies.
    def test_exception_has_to_precede_use(self):
        path = report.EXCEPTIONS / "sample.md"
        base = "# Decision\n\n## Scope\n\n- live\n"
        with patch.object(report, "read_base", return_value=None):
            self.assertFalse(report.approved_decision("base", path, "live"))
        with patch.object(report, "read_base", return_value=base), patch.object(
                Path, "is_file", return_value=True), patch.object(Path, "read_text", return_value=base):
            self.assertTrue(report.approved_decision("base", path, "live"))
            self.assertFalse(report.approved_decision("base", path, "security"))
            with patch.object(Path, "read_text", return_value=base + "new scope"):
                self.assertFalse(report.approved_decision("base", path, "live"))

    # Scenario: two operation entries claim to cover the same registered factory.
    # Guarantees: duplicate records cannot silently override the operator-facing evidence.
    def test_duplicate_support_row_rejected(self):
        row = "| `urn:test:exporter:x` | a | b | c | - |\n"
        with self.assertRaisesRegex(ValueError, "duplicate"):
            report.support_table(row + row)

    # Scenario: a contribution cites a continuous suite that exists but is never invoked by CI.
    # Guarantees: the benchmark claim requires an active workload or an explicit obstacle.
    def test_benchmark_needs_active_suite(self):
        with self.assertRaisesRegex(ValueError, "invoked by continuous CI"):
            report.validate_benchmark("urn:test:processor:x", {
                "workload": "tools/pipeline_perf_test/test_suites/integration/continuous/passthrough.yaml"})
        with self.assertRaisesRegex(ValueError, "continuous workload"):
            report.validate_benchmark("urn:test:processor:x", {})

    # Scenario: a seed test is offered for a different or unrelated changed path.
    # Guarantees: a real passing test cannot be reused as a blanket security assessment.
    def test_scope_and_evidence_must_match(self):
        path = Path("foo.rs")
        item = {"paths": [str(path)], "area": "security", "status": "tested",
                "source": "live-rollout", "reason": "Verify delivery."}
        with self.assertRaisesRegex(ValueError, "covers live"):
            report.evidence_record(item, Path("review.json"), {path}, {"live-rollout": True}, "base")
        item["area"] = "live"
        with self.assertRaisesRegex(ValueError, "changed paths"):
            report.evidence_record(item, Path("review.json"), set(), {"live-rollout": True}, "base")

    # Scenario: one case in a review fails even though the other has a valid result.
    # Guarantees: CI blocks the failure and still displays the independent passing evidence.
    def test_missing_evidence_preserves_other_observations(self):
        source = report.REVIEW / "change.json"
        changed = Path("src/foo.rs")
        common = {"paths": [str(changed)], "status": "tested", "reason": "Exercises the operation."}
        data = {"evidence": [dict(common, area="live", source="live-rollout"),
                             dict(common, area="behavior", source="retry-recovery")]}
        problems = []
        with patch.object(Path, "is_file", return_value=True), patch.object(
                Path, "read_text", return_value=json.dumps(data)), patch.object(
                report, "read_base", return_value=None):
            records, _ = report.evidence_records({source, changed}, {"retry-recovery": True}, "base", problems)
        self.assertEqual([r.area for r in records], ["behavior"])
        self.assertEqual(len(problems), 1)
        self.assertIn("live-rollout", problems[0])

    # Scenario: a release fixture is retired and its proposed replacement is also retired.
    # Guarantees: an intentional break must provide a verifiable, accepted migration target.
    def test_migration_requires_accepted_replacement_and_prior_decision(self):
        old = "v0.60.0/syslog-console.yaml"
        new = "v0.60.0/syslog-console.json"
        entry = {"replacement": new, "diagnostic": "unsupported", "decision": "pending.md"}
        with patch.object(report, "migrations", return_value={old: entry, new: entry}):
            with self.assertRaisesRegex(ValueError, "accepted replacement"):
                report.validate_migrations("base")
        with patch.object(report, "migrations", return_value={old: entry}):
            with self.assertRaisesRegex(ValueError, "preceding"):
                report.validate_migrations("base")
