"""Offline configuration, evidence and report checks for receiver -> local Perf."""

from datetime import datetime, timedelta, timezone
from io import BytesIO
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import duckdb
import yaml

from tools.comparison_dashboard import dashboard, kafka_syslog_receiver_only as local


DASHBOARD = Path(__file__).resolve().parents[1]
PERF_TEST = DASHBOARD.parent / "pipeline_perf_test"
REPORT = (PERF_TEST / "test_suites" / "comparison_dashboard" / "reports"
          / "report_logs_receiver_only.yaml")


class ReceiverOnlySuiteTests(unittest.TestCase):
    def setUp(self):
        previous = Path.cwd()
        os.chdir(DASHBOARD)
        self.addCleanup(os.chdir, previous)
        self.manifest = dashboard.load_manifest(Path("manifest.yaml"))
        self.suite, self.config = self.load(
            "dfe-logs-kafka-syslog-receiver-only.yaml"
        )

    @staticmethod
    def render(path, variables):
        return yaml.safe_load(
            dashboard.render_orchestrator_config(Path(path), variables)
        )

    def load(self, name):
        suite = dashboard.load_suite(Path("suites/dfe") / name)
        context = dashboard.build_template_context(
            suite, self.manifest, DASHBOARD / ".data/receiver-only-render-test", 20
        )
        return suite, self.render(suite["orchestrator_template"], context)

    def steps(self, test):
        spec = test["from_template"]
        return self.render(spec["path"], spec["variables"])["steps"]

    def engine(self, steps):
        hooks = next(s for s in steps if s["name"] == "Deploy Kafka Consumer")
        spec = hooks["hooks"]["run"]["pre"][0]["render_template"]
        return self.render(spec["template_path"], spec["variables"])

    # Scenario: All seven high rates render the registered orchestrator schema.
    # Guarantees: Nested steps/hooks parse offline after real plugin registration.
    def test_full_orchestrator_schema(self):
        sys.path.insert(0, str(PERF_TEST / "orchestrator"))
        self.addCleanup(sys.path.pop, 0)
        from lib.impl import actions, strategies  # noqa: F401
        from lib.runner.schema.loader import load_config_from_string
        from lib.impl.strategies.hooks.reporting.sql_report import SQLReportDetails

        config = load_config_from_string(yaml.safe_dump(self.config))
        self.assertEqual([t.name for t in config.tests],
                         ["100k", "200k", "300k", "400k", "600k", "800k", "1000k"])
        self.assertTrue(all(t.steps for t in config.tests))
        SQLReportDetails.model_validate(yaml.safe_load(REPORT.read_text()))

    # Scenario: Every rate renders the same isolated Kafka input and allocated core.
    # Guarantees: Only three containers and receiver -> local Perf are deployed.
    def test_topology_and_input(self):
        components = self.config["components"]
        self.assertEqual(set(components),
                         {"load-generator", "kafka-broker", "kafka-consumer"})
        images = {name: c["deployment"]["docker"]["image"]
                  for name, c in components.items()}
        self.assertEqual(images, {
            "load-generator": "load_generator:kafka-syslog",
            "kafka-broker": "apache/kafka:latest",
            "kafka-consumer": "df_engine:latest",
        })
        ports = {p["published"] for c in components.values()
                 for p in c["deployment"]["docker"]["ports"]}
        self.assertEqual(ports, {18085, 19094, 18088})
        guard = self.config["hooks"]["run"]["pre"][0]["run_command"]["command"]
        self.assertIn("backend-service", guard)
        for test in self.config["tests"]:
            with self.subTest(rate=test["name"]):
                steps = self.steps(test)
                consumer = self.engine(steps)
                pipeline = consumer["groups"]["default"]["pipelines"]["main"]
                nodes = pipeline["nodes"]
                self.assertEqual(set(nodes), {"receiver", "perf"})
                self.assertEqual(nodes["receiver"], {
                    "type": "urn:otel:receiver:kafka",
                    "config": {
                        "brokers": "kafka-broker:9092",
                        "group_id": "kafka-syslog-benchmark",
                        "client_id": "dfe-kafka-syslog-consumer",
                        "auto_offset_reset": "earliest",
                        "commit": {"mode": "auto", "interval_ms": 1000},
                        "session_timeout_ms": 30000,
                        "heartbeat_interval_ms": 3000,
                        "min_fetch_bytes": 1,
                        "max_fetch_bytes": 1048576,
                        "max_fetch_wait_ms": 500,
                        "max_partition_fetch_bytes": 1048576,
                        "isolation_level": "read_uncommitted",
                        "logs": {"topics": ["otel-syslog"], "encoding": "syslog"},
                    },
                })
                self.assertEqual(nodes["perf"]["type"], "urn:otel:exporter:perf")
                self.assertTrue(nodes["perf"]["policies"]["telemetry"]["item_counts"])
                self.assertEqual(pipeline["connections"],
                                 [{"from": "receiver", "to": "perf"}])
                self.assertEqual(
                    consumer["policies"]["resources"]["core_allocation"]["set"],
                    [{"start": 1, "end": 1}],
                )
                self.assertEqual(consumer["policies"]["telemetry"],
                                 {"runtime_metrics": "normal"})
                start = next(s for s in steps
                             if s["name"] == "Start Raw Syslog Producer")
                payload = start["hooks"]["run"]["pre"][0]["send_http_request"]
                self.assertEqual(payload["payload"], {
                    "load_type": "syslog", "syslog_transport": "kafka",
                    "syslog_format": "rfc5424", "syslog_content_type": "random",
                    "kafka_brokers": "kafka-broker:9092",
                    "kafka_topic": "otel-syslog",
                    "target_rate": int(test["name"][:-1]) * 1000,
                    "threads": 1, "batch_size": 100,
                    "body_size": 1024, "message_size": 1024,
                })
                rendered = yaml.safe_dump(steps)
                self.assertNotIn("backend-service", rendered)
                self.assertNotIn("18087", rendered)

    # Scenario: The producer stops before bounded drain and the admin exits on stop.
    # Guarantees: Final counters are saved pre-shutdown, never from a dead endpoint.
    def test_capture_order_timing_and_delivery_gate(self):
        for test in self.config["tests"]:
            steps = self.steps(test)
            names = [s["name"] for s in steps]
            ordered = [
                "Monitor All", "Start Raw Syslog Producer", "Warm Up",
                "Observe Load", "Stop and Flush Producer", "Bounded Consumer Drain",
                "Capture Final Local Perf Counters", "Stop Kafka Consumer",
                "Capture Kafka Record and Verify Delivery", "Stop Monitoring All",
                "Destroy All", "Run Report",
            ]
            self.assertEqual(sorted(ordered, key=names.index), ordered)
            for name, seconds in (("Warm Up", 10), ("Observe Load", 20),
                                  ("Bounded Consumer Drain", 10)):
                self.assertEqual(steps[names.index(name)]["action"]["wait"],
                                 {"delay_seconds": seconds})
            observe = steps[names.index("Observe Load")]["hooks"]["run"]
            for phase, label in (("pre", "observation-start"),
                                 ("post", "observation-stop")):
                self.assertIn(f"--phase {label}", yaml.safe_dump(observe[phase]))
            stop = steps[names.index("Stop Kafka Consumer")]["hooks"]["run"]["pre"]
            self.assertIn("timeout_secs=15", stop[0]["send_http_request"]["url"])
            self.assertEqual(len(stop), 1)
            capture = steps[names.index("Capture Final Local Perf Counters")]
            self.assertIn("--phase final", yaml.safe_dump(capture))
            verify = steps[names.index("Capture Kafka Record and Verify Delivery")]
            command = verify["hooks"]["run"]["pre"][-1]["run_command"]["command"]
            self.assertNotIn("\n", command)
            self.assertEqual("--expect-full-delivery" in command,
                             test["name"] == "1k")
            report = steps[names.index("Run Report")]["hooks"]["run"]
            self.assertEqual(report["pre"][0]["run_command"]["command"], command)
            inspect = steps[names.index("Deploy Load Generator")]
            self.assertIn("images.txt", yaml.safe_dump(inspect))

    # Scenario: The receiver-only comparison is built from a clean layer1 checkout.
    # Guarantees: Seven cases reference only the registered local Perf suite.
    def test_comparison_registration(self):
        comparison_path = Path("comparisons/kafka_receiver_syslog_receiver_only.yaml")
        self.assertIn(comparison_path.resolve(), self.manifest.comparison_files)
        comparison = yaml.safe_load(comparison_path.read_text())
        self.assertEqual([t["name"] for t in comparison["tests"]],
                         [t["name"] for t in self.config["tests"]])
        self.assertEqual([t["loadgen_rate"] for t in comparison["tests"]],
                         self.suite["variables"]["rates"])
        self.assertEqual(comparison["tests"][-1]["label"], "1M")
        self.assertEqual([s["short"] for s in comparison["suites"]], [
            "receiver -> local Perf",
        ])
        self.assertEqual(comparison["suites"][0]["slug"], self.suite["slug"])
        self.assertEqual(self.suite["meta"]["protocols"], ["syslog"])

    # Scenario: A future explicit 1k override requests a full-delivery smoke.
    # Guarantees: Only that override requires equality; default high rates do not.
    def test_explicit_low_rate_override_keeps_full_delivery_gate(self):
        suite = {**self.suite, "variables": {
            **self.suite["variables"], "rates": [1000],
        }}
        context = dashboard.build_template_context(
            suite, self.manifest, DASHBOARD / ".data/receiver-only-render-test", 20
        )
        config = self.render(suite["orchestrator_template"], context)
        self.assertEqual([t["name"] for t in config["tests"]], ["1k"])
        for test in config["tests"] + self.config["tests"]:
            steps = self.steps(test)
            report = next(s for s in steps if s["name"] == "Run Report")
            command = report["hooks"]["run"]["pre"][0]["run_command"]["command"]
            self.assertEqual("--expect-full-delivery" in command,
                             test["name"] == "1k")


class ReceiverOnlyDeliveryTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(dir=Path.cwd())
        self.addCleanup(directory.cleanup)
        self.output = Path(directory.name)

    @staticmethod
    def consumer(received):
        def sample(name, value, node, scope, extra=""):
            labels = ",".join(f'{k}="{v}"' for k, v in local.PIPELINE.items())
            return (f'{name}{{{labels},otel_scope_node_id="{node}",'
                    f'otel_scope_name="{scope}"{extra}}} {value}\n')

        result = sample(
            "items_total", received, "perf", "node.input",
            ',signal="logs",outcome="success"',
        )
        for name, value in (("records_received_total", received),
                            ("records_bytes_total", received * 1024)):
            result += sample(name, value, "receiver", "receiver.kafka.consumer")
        for name, value in (("started_total", received),
                            ("completed_total", received),
                            ("payload_size_bytes_total", received * 1024)):
            result += sample(name, value, "receiver", "receiver.kafka.messages",
                             ',signal="logs"')
        result += sample("group_lag", 0, "receiver", "receiver.kafka.consumer")
        return result

    def fixture(self, received=1000):
        for index, phase in enumerate(local.PHASES, 1):
            produced = 100 * index if phase != "final" else 1000
            delivered = 50 * index if phase != "final" else received
            (self.output / f"producer-{phase}.prom").write_text(
                f"logs_produced {produced}\nbytes_sent {produced * 1024}\n"
                "failed 0\nkafka_pending 0\nkafka_delivery_failed 0\n"
                "kafka_enqueue_failed 0\nkafka_flush_timeouts 0\n"
            )
            (self.output / f"consumer-{phase}.prom").write_text(
                self.consumer(delivered)
            )
            local.write_json(self.output / f"capture-{phase}.json", {
                "phase": phase,
                "captures": {
                    component: {
                        "url": url,
                        "started_at": f"2026-01-01T00:00:{index}0+00:00",
                        "finished_at": f"2026-01-01T00:00:{index}1+00:00",
                    } for component, url in local.ENDPOINTS.items()
                },
            })
        header = b"<134>1 2026-01-01T00:00:00Z host app - - - "
        (self.output / "kafka-record.txt").write_bytes(
            header + b"x" * (1024 - len(header)) + b"\n"
        )
        (self.output / "images.txt").write_text("\n".join(
            f"/{name} sha256:{'a' * 64}"
            for name in ("load-generator", "kafka-broker", "kafka-consumer")
        ) + "\n")
        (self.output / "kafka-consumer-config.rendered.yaml").write_text(
            "version: otel_dataflow/v1\n"
        )

    # Scenario: The smoke workload reaches local Perf in full before shutdown.
    # Guarantees: Exact count equality persists without any backend scrape or default.
    def test_complete_local_delivery(self):
        self.fixture()
        result = local.verify(self.output, True)
        self.assertEqual(result["local_perf_logs"], 1000)
        self.assertEqual(result["not_observed_at_local_perf"], 0)
        self.assertNotIn("backend_logs", result)
        self.assertFalse(result["snapshots"][-1]["decode_error_series_present"])
        self.assertEqual(json.loads(
            (self.output / "verified-delivery.json").read_text()
        ), result)
        self.assertEqual(result["snapshots"][0]["not_observed_at_local_perf"], 50)

    # Scenario: A saturated run has a bounded deficit even when group_lag reports zero.
    # Guarantees: High rates record the deficit; the full-delivery smoke rejects it.
    def test_bounded_deficit_not_drain_claim(self):
        self.fixture(900)
        result = local.verify(self.output)
        self.assertEqual(result["not_observed_at_local_perf"], 100)
        with self.assertRaisesRegex(ValueError, "Incomplete smoke delivery"):
            local.verify(self.output, True)
        self.assertFalse((self.output / "verified-delivery.json").exists())

    # Scenario: The real first run exported messages but emitted no item counters.
    # Guarantees: Positive Perf message counts never replace missing success items.
    def test_real_snapshot_without_items_remains_invalid(self):
        self.fixture()
        fixture = (Path(__file__).parent / "fixtures"
                   / "receiver-only-missing-items.prom")
        (self.output / "consumer-final.prom").write_bytes(fixture.read_bytes())
        samples = local.read_prometheus(fixture)
        self.assertEqual(local.count(
            local.pipeline_samples(samples, "perf"), "messages_total",
            otel_scope_name="exporter.exports", signal="logs", outcome="success",
        ), 1463355)
        with self.assertRaisesRegex(
            ValueError, "Missing or invalid counter items_total"
        ):
            local.verify(self.output)
        self.assertFalse((self.output / "verified-delivery.json").exists())

    # Scenario: Producer, local Perf or observed decoder error metrics are invalid.
    # Guarantees: Missing/nonfinite/fractional counters and wrong sinks cannot pass.
    def test_invalid_evidence_fails(self):
        changes = [
            ("producer-final.prom", "kafka_pending 0", "kafka_pending 1"),
            ("producer-final.prom", "kafka_delivery_failed 0",
             "kafka_delivery_failed 1"),
            ("producer-final.prom", "bytes_sent 1024000", "bytes_sent 1024"),
            ("producer-final.prom", "logs_produced 1000", "logs_produced nan"),
            ("consumer-final.prom", "} 1000\n", "} nan\n"),
            ("consumer-final.prom", "} 1000\n", "} 1001\n"),
            ("consumer-final.prom", "} 1000\n", "} -1\n"),
            ("consumer-final.prom", "} 1000\n", "} 999.5\n"),
            ("consumer-final.prom", 'otel_scope_node_id="perf"',
             'otel_scope_node_id="batch"'),
            ("consumer-final.prom", 'outcome="success"', 'outcome="failure"'),
            ("consumer-final.prom", 'otel_scope_core_id="1"', 'otel_scope_core_id="2"'),
            ("consumer-final.prom", "records_received_total", "missing_counter"),
            ("consumer-observation-start.prom", "} 50\n", "} 2000\n"),
            ("kafka-record.txt", "<134>1 ", "invalid "),
            ("images.txt", "/kafka-consumer", "/backend-service"),
        ]
        for filename, original, invalid in changes:
            with self.subTest(filename=filename, invalid=invalid):
                self.fixture()
                path = self.output / filename
                path.write_text(path.read_text().replace(original, invalid))
                with self.assertRaises(ValueError):
                    local.verify(self.output)
                self.assertFalse((self.output / "verified-delivery.json").exists())
        self.fixture()
        path = self.output / "consumer-final.prom"
        failure = self.consumer(1).splitlines()[1].replace(
            "records_received_total", "decode_errors_total"
        )
        path.write_text(path.read_text() + failure + "\n")
        with self.assertRaisesRegex(ValueError, "decode_errors_total"):
            local.verify(self.output)
        self.fixture()
        (self.output / "consumer-final.prom").unlink()
        with self.assertRaises(FileNotFoundError):
            local.verify(self.output)

    # Scenario: Live final snapshots must be captured from the still-running admin.
    # Guarantees: Bounded, non-resetting scrapes preserve raw bytes and capture times.
    def test_snapshot_capture(self):
        with patch.object(local, "urlopen", side_effect=[
            BytesIO(b"producer metric\n"), BytesIO(b"consumer metric\n")
        ]) as request:
            local.snapshot(self.output, "final")
        self.assertEqual(request.call_count, 2)
        self.assertEqual(request.call_args_list[1].args[0],
                         local.ENDPOINTS["consumer"])
        self.assertEqual(request.call_args_list[1].kwargs, {"timeout": 5})
        self.assertEqual((self.output / "consumer-final.prom").read_bytes(),
                         b"consumer metric\n")
        capture = json.loads((self.output / "capture-final.json").read_text())
        self.assertEqual(capture["phase"], "final")
        self.assertIn("pre-shutdown", capture["timing"])


class ReceiverOnlyReportTests(unittest.TestCase):
    def database(self):
        db = duckdb.connect()
        self.addCleanup(db.close)
        db.execute("CREATE TABLE metadata (Attribute VARCHAR)")
        db.execute('CREATE TABLE metadata_row ("test.name" VARCHAR, '
                   '"test.suite" VARCHAR, "test.start" VARCHAR)')
        db.execute("INSERT INTO metadata_row VALUES "
                   "('smoke','local-perf','2026-01-01T00:00:00Z')")
        db.execute('CREATE TABLE events ("attributes.test.name" VARCHAR, '
                   "name VARCHAR, timestamp BIGINT)")
        base = datetime(2026, 1, 1, tzinfo=timezone.utc)
        for name, second in (("observation_start", 1), ("observation_stop", 11)):
            db.execute("INSERT INTO events VALUES (?,?,?)",
                       ["smoke", name, int((base.timestamp() + second) * 1e9)])
        labels = (
            "component_name", "otel_scope_core_id", "otel_scope_name",
            "otel_scope_node_id", "signal", "outcome",
            "otel_scope_pipeline_group_id", "otel_scope_pipeline_id",
        )
        db.execute("CREATE TABLE metrics (timestamp TIMESTAMPTZ, metric_name VARCHAR, "
                   "value DOUBLE, " + ", ".join(
                       f'"metric_attributes.{label}" VARCHAR' for label in labels
                   ) + ")")
        for second in range(12):
            for name, value, component in (
                ("logs_produced", 1000 * second, "load-generator"),
                ("container.cpu.usage", 0.75, "kafka-consumer"),
                ("container.cpu.allocated", 1, "kafka-consumer"),
                ("container.memory.usage", 100 * 1048576, "kafka-consumer"),
            ):
                db.execute("INSERT INTO metrics VALUES (?,?,?,?,?,?,?,?,?,?,?)", [
                    base + timedelta(seconds=second), name, value, component,
                    None, None, None, None, None, None, None,
                ])
            for rate, component, core, node, outcome, group, pipeline in (
                (900, "kafka-consumer", "1", "perf", "success", "default", "main"),
                (99999, "backend-service", "2", "perf", "success", "default", "main"),
                (99999, "kafka-consumer", "1", "perf", "failure", "default", "main"),
                (99999, "kafka-consumer", "1", "batch", "success", "default", "main"),
                (99999, "kafka-consumer", "1", "perf", "success", "system", "other"),
            ):
                db.execute("INSERT INTO metrics VALUES (?,?,?,?,?,?,?,?,?,?,?)", [
                    base + timedelta(seconds=second), "items", rate * second,
                    component, core, "node.input", node, "logs", outcome, group,
                    pipeline,
                ])
        return db

    @staticmethod
    def execute_report(db):
        for query in yaml.safe_load(REPORT.read_text())["queries"]:
            db.execute(query["sql"])

    # Scenario: Other Perf endpoints/outcomes coexist with the measured local sink.
    # Guarantees: Summary and time series count only local successes and one-core CPU.
    def test_local_perf_rates_and_resources(self):
        db = self.database()
        self.execute_report(db)
        values = dict(db.execute(
            "SELECT name,value FROM gh_actions_benchmark"
        ).fetchall())
        self.assertEqual(values, {
            "logs_produced_rate": 1000, "logs_received_rate": 900,
            "cpu_percentage_normalized_avg": 75,
            "cpu_percentage_normalized_max": 75,
            "ram_mib_avg": 100, "ram_mib_max": 100, "test_duration": 10,
        })
        rates = db.execute("SELECT DISTINCT value FROM dashboard_timeseries "
                           "WHERE metric = 'logs_received_rate'").fetchall()
        self.assertEqual(rates, [(900,)])
        self.assertFalse(any("egress" in name or "dropped" in name for name in values))

    # Scenario: Report input is absent, nonfinite, reset or wrongly normalized.
    # Guarantees: The report fails instead of publishing zero-shaped successful data.
    def test_invalid_report_measurements_fail(self):
        for mutation in (
            "DELETE FROM metrics WHERE metric_name = 'items'",
            "UPDATE metrics SET value = 'nan' WHERE metric_name = 'logs_produced'",
            "UPDATE metrics SET value = NULL WHERE metric_name = 'logs_produced' "
            "AND timestamp = '2026-01-01T00:00:05Z'",
            "UPDATE metrics SET value = 0 WHERE metric_name = 'logs_produced' "
            "AND timestamp = '2026-01-01T00:00:05Z'",
            "UPDATE metrics SET value = 2 "
            "WHERE metric_name = 'container.cpu.allocated'",
            "DELETE FROM metrics WHERE metric_name = 'container.memory.usage'",
            "DELETE FROM events",
        ):
            with self.subTest(mutation=mutation):
                db = self.database()
                db.execute(mutation)
                with self.assertRaises(duckdb.Error):
                    self.execute_report(db)


if __name__ == "__main__":
    unittest.main()
