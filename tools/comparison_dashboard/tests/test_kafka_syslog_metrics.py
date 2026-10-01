"""Shared saved-counter contract for Kafka benchmark evidence verifiers."""

from pathlib import Path
import tempfile
import unittest

from tools.comparison_dashboard.kafka_syslog_metrics import counter, read_prometheus


class SavedCounterTests(unittest.TestCase):
    # Scenario: Saved scrapes include comments, escaped labels and a timestamp.
    # Guarantees: Parsing preserves names, labels and values for endpoint filtering.
    def test_read_saved_scrape(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "metrics.prom"
            path.write_text(
                '# TYPE items_total counter\n\n'
                'items_total{node="perf",note="line\\n\\"quoted\\""} 42 1234\n'
            )
            self.assertEqual(read_prometheus(path), [
                ("items_total", {"node": "perf", "note": 'line\n"quoted"'}, 42),
            ])

    # Scenario: A saved scrape contains a malformed sample line.
    # Guarantees: Invalid evidence raises instead of being silently discarded.
    def test_invalid_scrape_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "metrics.prom"
            path.write_text("items_total{node=unfinished\n")
            with self.assertRaisesRegex(ValueError, "Invalid Prometheus sample"):
                read_prometheus(path)

    # Scenario: Several series share a counter name but represent different sinks.
    # Guarantees: Only matching labels contribute; a measured zero remains valid.
    def test_selected_counter_sum(self):
        samples = [
            ("items_total", {"node": "perf", "core": "1"}, 0),
            ("items_total", {"node": "perf", "core": "2"}, 42),
            ("items_total", {"node": "other"}, 1000),
        ]
        self.assertEqual(counter(samples, "items_total", node="perf"), 42)
        self.assertEqual(counter(samples, "items_total", node="perf", core="1"), 0)

    # Scenario: Required counter series are absent, negative or nonfinite.
    # Guarantees: No unavailable measurement is converted to zero or successful data.
    def test_required_counter_rejects_invalid_values(self):
        for value in (None, -1, float("nan"), float("inf")):
            with self.subTest(value=value):
                samples = [] if value is None else [("items_total", {}, value)]
                with self.assertRaisesRegex(ValueError, "Missing or invalid counter"):
                    counter(samples, "items_total")


if __name__ == "__main__":
    unittest.main()
