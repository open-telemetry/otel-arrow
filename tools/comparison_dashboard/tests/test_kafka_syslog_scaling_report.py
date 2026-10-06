"""Offline DuckDB/schema checks for scalable receiver -> local Perf."""

from datetime import datetime, timedelta, timezone
from pathlib import Path
import sys
import unittest

import duckdb
import yaml


PERF_TEST = Path(__file__).resolve().parents[2] / "pipeline_perf_test"
REPORT = (PERF_TEST / "test_suites" / "comparison_dashboard" / "reports"
          / "report_logs_receiver_scaling.yaml")
BASE = datetime(2026, 1, 1, tzinfo=timezone.utc)
CORE2 = ('metric_name = \'items\' AND '
         '"metric_attributes.otel_scope_core_id" = \'2\'')


class ReceiverScalingReportTests(unittest.TestCase):
    def database(self, cores=2):
        """Use the receiver-only test's in-memory telemetry schema."""
        db = duckdb.connect()
        self.addCleanup(db.close)
        db.execute("CREATE TABLE metadata (Attribute VARCHAR)")
        db.execute('CREATE TABLE metadata_row ("test.name" VARCHAR, '
                   '"test.suite" VARCHAR, "test.start" VARCHAR)')
        db.execute("INSERT INTO metadata_row VALUES "
                   "('smoke','scaling-perf','2026-01-01T00:00:00Z')")
        db.execute('CREATE TABLE events ("attributes.test.name" VARCHAR, '
                   "name VARCHAR, timestamp BIGINT)")
        for name, second in (
            ("observation_start", 10), ("observation_stop", 30),
        ):
            db.execute("INSERT INTO events VALUES (?,?,?)",
                       ["smoke", name, int((BASE.timestamp() + second) * 1e9)])
        labels = (
            "component_name", "otel_scope_core_id", "otel_scope_name",
            "otel_scope_node_id", "signal", "outcome",
            "otel_scope_pipeline_group_id", "otel_scope_pipeline_id",
        )
        db.execute("CREATE TABLE metrics (timestamp TIMESTAMPTZ, "
                   "metric_name VARCHAR, "
                   "value DOUBLE, " + ", ".join(
                       f'"metric_attributes.{label}" VARCHAR'
                       for label in labels
                   ) + ")")
        rows = []
        for second in range(9, 32):
            for name, value, component in (
                ("logs_produced", 10000 * second, "load-generator"),
                ("container.cpu.usage", 0.75 * cores, "kafka-consumer"),
                ("container.cpu.allocated", cores, "kafka-consumer"),
                ("container.memory.usage", 100 * 1048576, "kafka-consumer"),
            ):
                rows.append([
                    BASE + timedelta(seconds=second), name, value,
                    component, None, None, None, None, None, None, None,
                ])
            for core in range(1, cores + 1):
                rows.append([
                    BASE + timedelta(seconds=second), "items",
                    100 * core * second,
                    "kafka-consumer", str(core), "node.input", "perf", "logs",
                    "success", "default", f"core{core}",
                ])
        db.executemany(
            "INSERT INTO metrics VALUES (?,?,?,?,?,?,?,?,?,?,?)", rows
        )
        return db

    @staticmethod
    def execute_report(db):
        for query in yaml.safe_load(REPORT.read_text())["queries"]:
            db.execute(query["sql"])

    @staticmethod
    def values(db):
        return dict(db.execute(
            "SELECT name, value FROM gh_actions_benchmark"
        ).fetchall())

    def assert_invalid(self, mutation, cores=2):
        db = self.database(cores)
        db.execute(mutation)
        with self.assertRaises(duckdb.Error):
            self.execute_report(db)

    # Scenario: The scaling report is loaded by the existing SQL report plugin.
    # Guarantees: Schema and result tables need no external fixture files.
    def test_report_schema_and_result_tables(self):
        sys.path.insert(0, str(PERF_TEST / "orchestrator"))
        self.addCleanup(sys.path.pop, 0)
        from lib.impl.strategies.hooks.reporting.sql_report import (
            SQLReportDetails,
        )

        details = SQLReportDetails.model_validate(
            yaml.safe_load(REPORT.read_text())
        )
        self.assertFalse(details.load_tables)
        db = self.database()
        self.execute_report(db)
        for result in details.result_tables:
            self.assertTrue(
                db.execute(f"SELECT * FROM {result.name}").description
            )
            self.assertFalse(result.finite_columns)
        self.assertIn("core_summary", [r.name for r in details.result_tables])
        self.assertIn("consumer_allocation",
                      [r.name for r in details.result_tables])

    # Scenario: One, two or four individually labeled local Perf pipelines run.
    # Guarantees: Rates sum every core; CPU, RAM and metadata retain units.
    def test_one_two_four_core_rates_resources_and_timeseries(self):
        for cores in (1, 2, 4):
            with self.subTest(cores=cores):
                db = self.database(cores)
                self.execute_report(db)
                total = 100 * cores * (cores + 1) / 2
                expected = {
                    "logs_produced_rate": 10000, "logs_received_rate": total,
                    "cpu_percentage_total_avg": 75 * cores,
                    "cpu_percentage_total_max": 75 * cores,
                    "cpu_percentage_normalized_avg": 75,
                    "cpu_percentage_normalized_max": 75,
                    "ram_mib_avg": 100, "ram_mib_max": 100,
                    "test_duration": 20,
                }
                expected.update({
                    f"logs_received_rate_core{core}": core * 100
                    for core in range(1, cores + 1)
                })
                self.assertEqual(self.values(db), expected)
                self.assertEqual(db.execute(
                    "SELECT allocated_cores FROM consumer_allocation"
                ).fetchall(), [(cores,)])
                summary = db.execute(
                    "SELECT core_id, pipeline_id, successful_items_delta, "
                    "logs_received_rate, measurement_available, "
                    "measurement_status "
                    "FROM core_summary ORDER BY core_id"
                ).fetchall()
                self.assertEqual(summary, [
                    (core, f"core{core}", 2000 * core, 100 * core,
                     True, "available")
                    for core in range(1, cores + 1)
                ])
                points = dict(db.execute(
                    "SELECT DISTINCT metric, value FROM dashboard_timeseries"
                ).fetchall())
                self.assertEqual(points, {
                    "logs_produced_rate": 10000, "logs_received_rate": total,
                    "cpu_percentage_total": 75 * cores,
                    "cpu_percentage_normalized": 75, "ram_mib": 100,
                    **{f"logs_received_rate_core{core}": 100 * core
                       for core in range(1, cores + 1)},
                })
                self.assertEqual(db.execute(
                    "SELECT MIN(t), MAX(t) FROM dashboard_timeseries"
                ).fetchone(), (0, 20))
                self.assertFalse(any("network" in name or "dropped" in name
                                     or "loss" in name for name in expected))

    # Scenario: An idle core exports no series or only one observation sample.
    # Guarantees: Its rate and the total are NULL; other cores stay measurable.
    def test_missing_or_insufficient_idle_core_is_unavailable_not_zero(self):
        for predicate, status in (
            ("true", "missing_core_counter"),
            ("timestamp <> '2026-01-01T00:00:20Z'",
             "insufficient_core_samples"),
        ):
            with self.subTest(status=status):
                db = self.database()
                db.execute(
                    f"DELETE FROM metrics WHERE {CORE2} AND ({predicate})"
                )
                self.execute_report(db)
                values = self.values(db)
                self.assertIsNone(values["logs_received_rate_core2"])
                self.assertIsNone(values["logs_received_rate"])
                self.assertEqual(values["logs_received_rate_core1"], 100)
                self.assertEqual(db.execute(
                    "SELECT measurement_available, measurement_status "
                    "FROM core_summary WHERE core_id = 2"
                ).fetchone(), (False, status))
                self.assertEqual(db.execute(
                    "SELECT measurement_available, measurement_status "
                    "FROM endpoint_summary WHERE metric = 'logs_received_rate'"
                ).fetchone(), (False, "unavailable_core_measurement"))
                self.assertEqual(db.execute(
                    "SELECT COUNT(value) FROM dashboard_timeseries "
                    "WHERE metric = 'logs_received_rate'"
                ).fetchone(), (0,))
                self.assertIn("logs_received_rate_core2", {
                    row[0] for row in db.execute(
                        "SELECT DISTINCT metric FROM dashboard_timeseries"
                    ).fetchall()
                })

    # Scenario: No allocated core exports the successful item counter.
    # Guarantees: All core and aggregate rates are explicitly unavailable.
    def test_all_core_counters_missing(self):
        db = self.database(4)
        db.execute("DELETE FROM metrics WHERE metric_name = 'items'")
        self.execute_report(db)
        self.assertEqual(db.execute(
            "SELECT COUNT(*), COUNT(logs_received_rate), "
            "COUNT(*) FILTER (WHERE measurement_available) FROM core_summary"
        ).fetchone(), (4, 0, 0))
        self.assertIsNone(self.values(db)["logs_received_rate"])
        self.assertEqual(db.execute(
            "SELECT metric, value FROM dashboard_timeseries "
            "WHERE metric LIKE 'logs_received_rate%' ORDER BY metric"
        ).fetchall(), [("logs_received_rate", None)] + [
            (f"logs_received_rate_core{core}", None) for core in range(1, 5)
        ])

    # Scenario: An idle core exports a constant counter, including zero.
    # Guarantees: Observed no-progress is zero, not an unavailable aggregate.
    def test_observed_zero_is_zero(self):
        for counter in (0, 1234):
            with self.subTest(counter=counter):
                db = self.database()
                db.execute(
                    f"UPDATE metrics SET value = {counter} WHERE {CORE2}"
                )
                self.execute_report(db)
                self.assertEqual(
                    self.values(db)["logs_received_rate_core2"], 0
                )
                self.assertEqual(self.values(db)["logs_received_rate"], 100)
                self.assertEqual(db.execute(
                    "SELECT measurement_available FROM core_summary "
                    "WHERE core_id = 2"
                ).fetchone(), (True,))

    # Scenario: Other node/scope/outcome/signal/group/core labels coexist.
    # Guarantees: Only exact default/coreN and core_id=N successes count.
    def test_wrong_core_pipeline_and_endpoint_labels_do_not_count(self):
        mutations = {
            "component_name": ("backend-service",),
            "otel_scope_core_id": ("0", "3", "01"),
            "otel_scope_pipeline_id": ("main", "core2", "core01"),
            "otel_scope_pipeline_group_id": ("system",),
            "otel_scope_node_id": ("receiver", "batch"),
            "otel_scope_name": ("node.output",),
            "signal": ("metrics",),
            "outcome": ("failure",),
        }
        for label, replacements in mutations.items():
            for replacement in replacements:
                with self.subTest(label=label, replacement=replacement):
                    db = self.database()
                    db.execute(
                        "INSERT INTO metrics SELECT * REPLACE ("
                        f"'{replacement}' AS \"metric_attributes.{label}\", "
                        "'nan'::DOUBLE AS value) FROM metrics "
                        "WHERE metric_name = 'items' AND "
                        '"metric_attributes.otel_scope_core_id" = \'1\''
                    )
                    self.execute_report(db)
                    self.assertEqual(
                        self.values(db)["logs_received_rate"], 300
                    )

    # Scenario: Wrong pipeline/core pairing is the only series for a core.
    # Guarantees: Unrelated successes cannot hide an unobserved idle core.
    def test_wrong_pairing_does_not_fill_missing_core(self):
        db = self.database()
        db.execute('UPDATE metrics SET '
                   '"metric_attributes.otel_scope_pipeline_id" '
                   f"= 'core1' WHERE {CORE2}")
        self.execute_report(db)
        self.assertIsNone(self.values(db)["logs_received_rate_core2"])
        self.assertIsNone(self.values(db)["logs_received_rate"])

    # Scenario: One core misses an interior scrape but neither endpoint.
    # Guarantees: Scalars sum valid deltas; incomplete plots have NULL points.
    def test_incomplete_timeseries_never_publishes_partial_aggregate(self):
        db = self.database()
        db.execute(f"DELETE FROM metrics WHERE {CORE2} "
                   "AND timestamp = '2026-01-01T00:00:20Z'")
        self.execute_report(db)
        self.assertEqual(self.values(db)["logs_received_rate"], 300)
        self.assertEqual(db.execute(
            "SELECT t, value FROM dashboard_timeseries "
            "WHERE metric = 'logs_received_rate' "
            "AND t BETWEEN 9 AND 12 ORDER BY t"
        ).fetchall(), [(9, 300), (10, None), (11, None), (12, 300)])

    # Scenario: Core observation series start and end at different scrapes.
    # Guarantees: Scalar core deltas use a shared interval before summing.
    def test_scalar_core_rates_use_common_interval(self):
        db = self.database()
        db.execute(
            f"DELETE FROM metrics WHERE {CORE2} AND "
            "timestamp IN ('2026-01-01T00:00:10Z', '2026-01-01T00:00:30Z')"
        )
        db.execute("UPDATE metrics SET value = value + 1000 "
                   "WHERE metric_name = 'items' AND "
                   '"metric_attributes.otel_scope_core_id" = \'1\' '
                   "AND timestamp >= '2026-01-01T00:00:30Z'")
        self.execute_report(db)
        self.assertEqual(self.values(db)["logs_received_rate"], 300)
        self.assertEqual(db.execute(
            "SELECT DISTINCT EXTRACT(SECOND FROM first_ts), "
            "EXTRACT(SECOND FROM last_ts) FROM core_summary"
        ).fetchall(), [(11, 29)])
        self.assertEqual(db.execute(
            "SELECT successful_items_delta FROM core_summary ORDER BY core_id"
        ).fetchall(), [(1800,), (3600,)])

    # Scenario: Observed core series have no two common scrape timestamps.
    # Guarantees: The report flags unavailable rates, never mixing intervals.
    def test_disjoint_core_scrapes_are_unavailable(self):
        db = self.database()
        db.execute("DELETE FROM metrics WHERE metric_name = 'items' AND "
                   "EXTRACT(SECOND FROM timestamp) % 2 = "
                   'CAST("metric_attributes.otel_scope_core_id" '
                   'AS INTEGER) % 2')
        self.execute_report(db)
        self.assertIsNone(self.values(db)["logs_received_rate"])
        self.assertEqual(db.execute(
            "SELECT DISTINCT measurement_status FROM core_summary"
        ).fetchall(), [("no_common_core_interval",)])
        self.assertEqual(db.execute(
            "SELECT COUNT(value) FROM dashboard_timeseries "
            "WHERE metric = 'logs_received_rate'"
        ).fetchone(), (0,))

    # Scenario: An existing producer or core sample has an invalid value.
    # Guarantees: Negative/nonfinite/fractional/NULL counters fail the report.
    def test_invalid_counter_values_fail(self):
        for endpoint in ("metric_name = 'logs_produced'", CORE2):
            for value in ("-1", "'nan'", "'inf'", "'-inf'", "1.5", "NULL"):
                with self.subTest(endpoint=endpoint, value=value):
                    self.assert_invalid(
                        f"UPDATE metrics SET value = {value} WHERE {endpoint} "
                        "AND timestamp = '2026-01-01T00:00:20Z'"
                    )

    # Scenario: A core resets while another increases enough to hide the reset.
    # Guarantees: Per-core validation rejects resets before aggregation.
    def test_cross_core_increase_cannot_hide_reset(self):
        self.assert_invalid(
            "UPDATE metrics SET value = CASE "
            'WHEN "metric_attributes.otel_scope_core_id" = \'1\' '
            "THEN value + 100000 ELSE 0 END WHERE metric_name = 'items' "
            "AND timestamp >= '2026-01-01T00:00:20Z'"
        )

    # Scenario: Producer or core counters reset on observation entry or later.
    # Guarantees: Warmup predecessors and every observed delta detect resets.
    def test_counter_resets_fail(self):
        for endpoint in ("metric_name = 'logs_produced'", CORE2):
            for second in (10, 20, 30):
                with self.subTest(endpoint=endpoint, second=second):
                    self.assert_invalid(
                        f"UPDATE metrics SET value = 0 WHERE {endpoint} "
                        f"AND timestamp = '2026-01-01T00:00:{second}Z'"
                    )

    # Scenario: A producer or expected core series has duplicate timestamps.
    # Guarantees: Even identical duplicates fail instead of double-counting.
    def test_duplicate_counter_samples_fail(self):
        for endpoint in ("metric_name = 'logs_produced'", CORE2):
            with self.subTest(endpoint=endpoint):
                self.assert_invalid(
                    "INSERT INTO metrics SELECT * FROM metrics "
                    f"WHERE {endpoint} "
                    "AND timestamp = '2026-01-01T00:00:20Z'"
                )

    # Scenario: Producer or resource series are absent or have only one sample.
    # Guarantees: Missing producer, CPU and memory fail, not produce zeros.
    def test_missing_or_insufficient_producer_and_resources_fail(self):
        for metric in ("logs_produced", "container.cpu.usage",
                       "container.memory.usage"):
            for predicate in ("true", "timestamp <> '2026-01-01T00:00:20Z'"):
                with self.subTest(metric=metric, predicate=predicate):
                    self.assert_invalid(
                        f"DELETE FROM metrics WHERE metric_name = '{metric}' "
                        f"AND ({predicate})"
                    )

    # Scenario: Allocation is missing, unstable or not an integer in 1..4.
    # Guarantees: Expected cores and CPU normalization use one valid integer N.
    def test_invalid_allocations_fail(self):
        self.assert_invalid("DELETE FROM metrics "
                            "WHERE metric_name = 'container.cpu.allocated'")
        for value in ("0", "-1", "5", "1.5", "'nan'", "'inf'", "NULL", "4"):
            with self.subTest(value=value):
                self.assert_invalid(
                    f"UPDATE metrics SET value = {value} "
                    "WHERE metric_name = 'container.cpu.allocated' "
                    "AND timestamp = '2026-01-01T00:00:20Z'"
                )

    # Scenario: CPU or RAM has negative, nonfinite, NULL or duplicated samples.
    # Guarantees: Resource summaries and plots reject invalid measurements.
    def test_invalid_resource_samples_fail(self):
        for metric in ("container.cpu.usage", "container.memory.usage"):
            for value in ("-1", "'nan'", "'inf'", "NULL"):
                with self.subTest(metric=metric, value=value):
                    self.assert_invalid(
                        f"UPDATE metrics SET value = {value} "
                        f"WHERE metric_name = '{metric}' "
                        "AND timestamp = '2026-01-01T00:00:20Z'"
                    )
            self.assert_invalid(
                "INSERT INTO metrics SELECT * FROM metrics "
                f"WHERE metric_name = '{metric}' "
                "AND timestamp = '2026-01-01T00:00:20Z'"
            )

    # Scenario: Consumer CPU and RAM vary through the observation window.
    # Guarantees: Average and peak are separate and use the correct units.
    def test_resource_average_and_peak_are_distinct(self):
        db = self.database(4)
        db.execute("UPDATE metrics SET value = 4 "
                   "WHERE metric_name = 'container.cpu.usage' "
                   "AND timestamp = '2026-01-01T00:00:20Z'")
        db.execute("UPDATE metrics SET value = 200 * 1048576 "
                   "WHERE metric_name = 'container.memory.usage' "
                   "AND timestamp = '2026-01-01T00:00:20Z'")
        self.execute_report(db)
        values = self.values(db)
        self.assertAlmostEqual(values["cpu_percentage_total_avg"], 6400 / 21)
        self.assertEqual(values["cpu_percentage_total_max"], 400)
        self.assertAlmostEqual(values["cpu_percentage_normalized_avg"],
                               1600 / 21)
        self.assertEqual(values["cpu_percentage_normalized_max"], 100)
        self.assertAlmostEqual(values["ram_mib_avg"], 2200 / 21)
        self.assertEqual(values["ram_mib_max"], 200)

    # Scenario: Observation events are absent, duplicated or out of order.
    # Guarantees: The report rejects ambiguous or invalid observation windows.
    def test_invalid_observation_window_fails(self):
        for mutation in (
            "DELETE FROM events",
            "INSERT INTO events SELECT * FROM events "
            "WHERE name = 'observation_start'",
            "UPDATE events SET timestamp = 0 WHERE name = 'observation_stop'",
        ):
            with self.subTest(mutation=mutation):
                self.assert_invalid(mutation)

    # Scenario: Counters change during bounded drain after observation ends.
    # Guarantees: Post-observation values never enter measured rates or plots.
    def test_post_observation_samples_do_not_enter_rates(self):
        db = self.database()
        db.execute("UPDATE metrics SET value = 0 WHERE metric_name = 'items' "
                   "AND timestamp > '2026-01-01T00:00:30Z'")
        self.execute_report(db)
        self.assertEqual(self.values(db)["logs_received_rate"], 300)
        self.assertEqual(db.execute(
            "SELECT MAX(t) FROM dashboard_timeseries"
        ).fetchone(), (20,))


if __name__ == "__main__":
    unittest.main()
