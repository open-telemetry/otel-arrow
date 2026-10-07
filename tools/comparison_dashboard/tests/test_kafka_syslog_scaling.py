"""Offline matrix, group evidence and strict per-core capture regressions."""

from datetime import datetime, timedelta, timezone
import json
import math
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import yaml

from tools.comparison_dashboard import dashboard
from tools.comparison_dashboard import kafka_syslog_scaling as scaling


DASHBOARD = Path(__file__).resolve().parents[1]
PERF = DASHBOARD.parent / "pipeline_perf_test"


def topology(cores=4, topics=None, partitions=2):
    topics = topics or ["syslog-1", "syslog-2"]
    return {
        "topics": topics, "partitions_per_topic": partitions,
        "total_partitions": len(topics) * partitions, "cores": cores,
        "core_ids": list(range(1, cores + 1)), "group": "scaling",
        "aggregate_target_rate": 100000, "record_bytes": 1024,
        "client_ids": [f"dfe-kafka-syslog-core{i}" for i in range(1, cores + 1)],
    }


def members(config, modern=False):
    targets = sorted(scaling.validate_topology(
        config["topics"], config["partitions_per_topic"], config["cores"], 100000))
    header = ("GROUP CONSUMER-ID HOST CLIENT-ID #PARTITIONS "
              + ("CURRENT-ASSIGNMENT" if modern else "ASSIGNMENT"))
    result = [header]
    for core in config["core_ids"]:
        own = targets[core - 1::config["cores"]]
        by_topic = {}
        for topic, partition in own:
            by_topic.setdefault(topic, []).append(str(partition))
        assignment = (";".join(f"{t}:{','.join(p)}" for t, p in by_topic.items())
                      if modern else
                      ", ".join(f"{t}({','.join(p)})" for t, p in by_topic.items()))
        result.append(f"scaling member{core} /host dfe-kafka-syslog-core{core} "
                      f"{len(own)} {assignment}")
    return "\n".join(result) + "\n"


def offsets(config, committed=100):
    result = ["GROUP TOPIC PARTITION CURRENT-OFFSET LOG-END-OFFSET LAG "
              "CONSUMER-ID HOST CLIENT-ID"]
    for topic in config["topics"]:
        for partition in range(config["partitions_per_topic"]):
            current = "-" if committed is None else str(committed)
            end = 500 if committed is None else committed + 50
            lag = "-" if committed is None else "50"
            result.append(f"scaling {topic} {partition} {current} {end} {lag} "
                          "member /host client")
    return "\n".join(result) + "\n"


def metric(core, value=100, **extra):
    tags = {
        "otel_scope_pipeline_group_id": "default",
        "otel_scope_pipeline_id": f"core{core}", "otel_scope_core_id": str(core),
        "otel_scope_node_id": "perf", "otel_scope_name": "node.input",
        "signal": "logs", "outcome": "success", **extra,
    }
    return "items_total", tags, float(value)


def prom(sample):
    name, tags, value = sample
    labels = ",".join(f'{key}="{value}"' for key, value in tags.items())
    return f"{name}{{{labels}}} {value}\n"


class ScalingConfigTests(unittest.TestCase):
    def setUp(self):
        previous = Path.cwd()
        os.chdir(DASHBOARD)
        self.addCleanup(os.chdir, previous)
        self.manifest = dashboard.load_manifest(Path("manifest.yaml"))

    # Scenario: The approved nine topologies each run at two aggregate rates.
    # Guarantees: Eighteen bounded cases retain three components and independent cores.
    def test_matrix_schema_and_core_identity(self):
        sys.path.insert(0, str(PERF / "orchestrator"))
        self.addCleanup(sys.path.pop, 0)
        from lib.impl import actions, strategies  # noqa: F401
        from lib.runner.schema.loader import load_config_from_string
        actual = set()
        for cores, suffix in ((1, "1core"), (2, "2cores"), (4, "4cores")):
            suite = dashboard.load_suite(
                Path(f"suites/dfe/dfe-logs-kafka-syslog-scaling-{suffix}.yaml"))
            context = dashboard.build_template_context(
                suite, self.manifest, DASHBOARD / ".data/scaling-render-test", 20)
            text = dashboard.render_orchestrator_config(
                Path(suite["orchestrator_template"]), context)
            validated = load_config_from_string(text)
            self.assertTrue(all(test.steps for test in validated.tests))
            config = yaml.safe_load(text)
            self.assertEqual(set(config["components"]),
                             {"load-generator", "kafka-broker", "kafka-consumer"})
            for test in config["tests"]:
                spec = test["from_template"]
                steps = yaml.safe_load(dashboard.render_orchestrator_config(
                    Path(spec["path"]), spec["variables"]))["steps"]
                by_name = {step["name"]: step for step in steps}
                render = by_name["Deploy Kafka Consumer"]["hooks"]["run"]["pre"][0]
                render = render["render_template"]
                engine = yaml.safe_load(dashboard.render_orchestrator_config(
                    Path(render["template_path"]), render["variables"]))
                self.assertEqual(engine["policies"]["telemetry"],
                                 {"runtime_metrics": "normal"})
                pipelines = engine["groups"]["default"]["pipelines"]
                self.assertEqual(set(pipelines),
                                 {f"core{i}" for i in range(1, cores + 1)})
                for core in range(1, cores + 1):
                    pipeline = pipelines[f"core{core}"]
                    allocation = pipeline["policies"]["resources"]["core_allocation"]
                    self.assertEqual(allocation["set"], [{"start": core, "end": core}])
                    self.assertEqual(set(pipeline["nodes"]), {"receiver", "perf"})
                    self.assertEqual(pipeline["connections"],
                                     [{"from": "receiver", "to": "perf"}])
                    receiver = pipeline["nodes"]["receiver"]["config"]
                    self.assertEqual(receiver["client_id"],
                                     f"dfe-kafka-syslog-core{core}")
                    self.assertEqual(receiver["rebalance_strategy"], "round_robin")
                    self.assertTrue(pipeline["nodes"]["perf"]["policies"]
                                    ["telemetry"]["item_counts"])
                start = by_name["Start Raw Syslog Producer"]["hooks"]["run"]["pre"][0]
                payload = start["send_http_request"]["payload"]
                topics = payload["kafka_topics"]
                self.assertEqual(receiver["logs"]["topics"], topics)
                actual.add((cores, len(topics), payload["kafka_partitions_per_topic"],
                            payload["target_rate"]))
                self.assertNotIn("kafka_topic", payload)
                self.assertEqual(payload["message_size"], 1024)
                self.assertEqual(payload["threads"], 1)
                for name, seconds in (("Warm Up", 10), ("Observe Load", 20),
                                      ("Bounded Consumer Drain", 10)):
                    self.assertEqual(by_name[name]["action"]["wait"]["delay_seconds"],
                                     seconds)
                observe = by_name["Observe Load"]["hooks"]["run"]
                self.assertIn("group", observe["pre"][0]["run_command"]["command"])
                final = by_name["Capture Final Local Perf Counters"]["hooks"]["run"]
                commands = [hook["run_command"]["command"] for hook in final["pre"]]
                self.assertIn("snapshot", commands[0])
                self.assertIn("group", commands[1])
                self.assertNotIn("18087", yaml.safe_dump(steps))
        cells = ((1, 1, 1), (2, 1, 1), (4, 1, 1), (1, 1, 2), (1, 1, 4),
                 (2, 1, 2), (4, 1, 4), (4, 2, 2), (4, 4, 1))
        self.assertEqual(actual, {(*cell, rate) for cell in cells
                                  for rate in (100000, 300000)})

    # Scenario: Comparison columns include every measured topology at each rate.
    # Guarantees: Unconfigured combinations remain absent rather than fabricated runs.
    def test_comparison_case_coverage(self):
        comparison = yaml.safe_load(
            Path("comparisons/kafka_receiver_syslog_scaling.yaml").read_text())
        self.assertEqual(len(comparison["tests"]), 10)
        self.assertEqual(len(comparison["suites"]), 3)
        self.assertEqual({test["loadgen_rate"] for test in comparison["tests"]},
                         {100000, 300000})


class ScalingEvidenceTests(unittest.TestCase):
    # Scenario: Explicit topic names and bounded dimensions are required.
    # Guarantees: Invalid topology never silently becomes a single topic or core.
    def test_invalid_topology(self):
        for values in (([], 1, 1, 100), (["x", "x"], 1, 1, 100),
                       (["bad name"], 1, 1, 100), (["x"], True, 1, 100),
                       (["x"], 257, 1, 100), (["x"], 1, 0, 100),
                       (["x"], 1, 5, 100), (["x"], 1, 1, 0)):
            with self.subTest(values=values), self.assertRaises(ValueError):
                scaling.validate_topology(*values)

    # Scenario: Both supported Kafka output forms include active and idle members.
    # Guarantees: Clients map to real cores; idle rows need no assignment text.
    def test_group_members_active_and_idle(self):
        for config in (topology(), topology(topics=["syslog"], partitions=1)):
            for modern in (False, True):
                result = scaling.parse_members(members(config, modern), config)
                self.assertEqual([row["core_id"] for row in result], [1, 2, 3, 4])
                self.assertEqual(sum(len(row["partitions"]) for row in result),
                                 config["total_partitions"])

    # Scenario: A member is missing, duplicated, unknown or owns a repeated partition.
    # Guarantees: Configuration alone cannot stand in for complete real assignment.
    def test_invalid_assignment_coverage(self):
        config = topology()
        text = members(config)
        for broken in ("\n".join(text.splitlines()[:-1]),
                       text.replace("core4", "core3"),
                       text.replace("member4", "member3"),
                       text.replace("core4", "core5"),
                       text.replace("syslog-2(1)", "syslog-1(0)")):
            with self.subTest(text=broken), self.assertRaises(ValueError):
                scaling.parse_members(broken, config)

    # Scenario: Kafka reports unavailable commits or inconsistent/missing partitions.
    # Guarantees: Unknown offsets stay null; incomplete or false lag evidence fails.
    def test_partition_offsets(self):
        config = topology()
        self.assertTrue(all(row["committed_offset"] is None for row in
                            scaling.parse_offsets(offsets(config, None), config)))
        text = offsets(config)
        for broken in (text.replace("100 150 50", "100 150 49"),
                       "\n".join(text.splitlines()[:-1]),
                       text.replace("100 150 50", "-1 150 151")):
            with self.subTest(text=broken), self.assertRaises(ValueError):
                scaling.parse_offsets(broken, config)

    # Scenario: Four Perf counters coexist with missing, zero or malformed series.
    # Guarantees: Selection is exact by core/pipeline and missing never becomes zero.
    def test_per_core_counts(self):
        samples = [metric(core, 100 * core) for core in range(1, 5)]
        self.assertEqual([scaling.core_counts(samples, core)["local_perf_logs"]
                          for core in range(1, 5)], [100, 200, 300, 400])
        self.assertIsNone(scaling.core_counts([], 1)["local_perf_logs"])
        self.assertEqual(scaling.core_counts([metric(1, 0)], 1)["local_perf_logs"], 0)
        self.assertIsNone(scaling.core_counts(
            [metric(1, otel_scope_pipeline_id="main")], 1)["local_perf_logs"])
        for invalid in ([metric(1), metric(1)], [metric(1, -1)],
                        [metric(1, float("nan"))], [metric(1, 0.5)]):
            with self.subTest(samples=invalid), self.assertRaises(ValueError):
                scaling.core_counts(invalid, 1)

    def fixture(self, output, config):
        write = scaling.local.write_json
        write(output / "topology.json", config)
        beginning = datetime(2026, 10, 6, tzinfo=timezone.utc)

        def at(seconds):
            return (beginning + timedelta(seconds=seconds)).isoformat()

        for index, phase in enumerate(scaling.local.PHASES, 1):
            produced = index * 800
            producer = f"logs_produced {produced}\nbytes_sent {produced * 1024}\n"
            producer += "".join(f"{name} 0\n" for name in scaling.local.PRODUCER_ERRORS)
            for topic in config["topics"]:
                for partition in range(config["partitions_per_topic"]):
                    count = produced // config["total_partitions"]
                    labels = f'topic="{topic}",partition="{partition}"'
                    producer += (f"kafka_delivered_records{{{labels}}} {count}\n"
                                 f"kafka_delivered_bytes{{{labels}}} {count * 1024}\n")
            (output / f"producer-{phase}.prom").write_text(producer)
            (output / f"consumer-{phase}.prom").write_text(
                "".join(prom(metric(core, index * 100))
                        for core in config["core_ids"]))
            second = {1: 100, 2: 120, 3: 132}[index]
            write(output / f"capture-{phase}.json", {
                "phase": phase, "captures": {
                    component: {"url": url, "started_at": at(second),
                                "finished_at": at(second + 0.1)}
                    for component, url in scaling.local.ENDPOINTS.items()
                },
            })
        for phase, offset, second in (("before", 100, 98), ("final", 200, 133)):
            (output / f"group-{phase}-members.txt").write_text(members(config))
            (output / f"group-{phase}-offsets.txt").write_text(offsets(config, offset))
            write(output / f"group-{phase}.json",
                  {"started_at": at(second), "finished_at": at(second + 1)})
        (output / "images.txt").write_text("".join(
            f"/{name} sha256:{str(i) * 64}\n" for i, name in enumerate(
                ("load-generator", "kafka-broker", "kafka-consumer"), 1)))
        (output / "kafka-record.txt").write_bytes(b"<14>1 " + b"a" * 1018 + b"\n")
        (output / "kafka-consumer-config.rendered.yaml").write_text("fixture: true\n")
        write(output / "run-provenance.json", {"source_commit": "a" * 40})

    # Scenario: Complete balanced delivery and group evidence bracket live counters.
    # Guarantees: Core totals, committed progress and cutoff deficit are auditable.
    def test_verify_complete(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            self.fixture(output, topology())
            result = scaling.verify(output)
            self.assertEqual(result["status"], "verified")
            self.assertEqual(result["local_perf_logs"], 1200)
            self.assertEqual(result["not_observed_at_local_perf"], 1200)
            self.assertEqual([row["committed_progress"]
                              for row in result["partition_progress"]], [100] * 4)
            self.assertTrue(all(row["item_series_complete"]
                                for row in result["core_coverage"]))
            self.assertEqual(yaml.safe_load(
                (output / "scaling-evidence.yaml").read_text())["status"], "verified")

    # Scenario: Active or unassigned cores emit no successful item counter.
    # Guarantees: Coverage distinguishes those cases and totals stay explicitly null.
    def test_verify_missing_active_or_idle(self):
        for config, expected in ((topology(), "ACTIVE-core"),
                                 (topology(topics=["syslog"], partitions=1),
                                  "Unassigned core")):
            with tempfile.TemporaryDirectory() as directory:
                output = Path(directory)
                self.fixture(output, config)
                for phase in scaling.local.PHASES:
                    path = output / f"consumer-{phase}.prom"
                    path.write_text("".join(line + "\n"
                                            for line in path.read_text().splitlines()
                                            if 'otel_scope_core_id="4"' not in line))
                result = scaling.verify(output)
                self.assertEqual(result["status"], "flagged")
                self.assertIsNone(result["local_perf_logs"])
                self.assertTrue(any(expected in flag for flag in result["flags"]))

    # Scenario: Partition commits do not advance between captured boundaries.
    # Guarantees: The run is flagged rather than declared balanced active consumption.
    def test_verify_missing_partition_progress(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            self.fixture(output, topology())
            (output / "group-final-offsets.txt").write_text(offsets(topology(), 100))
            self.assertEqual(scaling.verify(output)["status"], "flagged")

    # Scenario: Timing, provenance, counters or assignment evidence is corrupted.
    # Guarantees: Verification deletes stale success and rejects the invalid evidence.
    def test_verify_rejects_corruption(self):
        changes = (
            ("consumer-final.prom", lambda text: text.replace("300.0", "1.0")),
            ("producer-final.prom", lambda text: text.replace(
                "kafka_pending 0", "kafka_pending 1")),
            ("group-before.json", lambda text: text.replace("00:01:39", "00:03:39")),
            ("run-provenance.json", lambda text: text.replace("a" * 40, "unknown")),
            ("producer-final.prom", lambda text: text.replace(" 600\n", " 601\n")),
            ("consumer-final.prom", lambda text: text.replace(
                'core_id="4"', 'core_id="5"')),
        )
        for filename, transform in changes:
            with self.subTest(file=filename), \
                    tempfile.TemporaryDirectory() as directory:
                output = Path(directory)
                self.fixture(output, topology())
                (output / "verified-scaling.json").write_text("{}")
                path = output / filename
                path.write_text(transform(path.read_text()))
                with self.assertRaises(ValueError):
                    scaling.verify(output)
                self.assertFalse((output / "verified-scaling.json").exists())

    # Scenario: The source tree is dirty at topic preparation.
    # Guarantees: No Kafka topic is created before immutable source provenance succeeds.
    def test_prepare_requires_committed_source(self):
        import subprocess
        with tempfile.TemporaryDirectory() as directory:
            error = subprocess.CalledProcessError(1, "git diff")
            with patch.object(scaling.subprocess, "run", side_effect=error):
                with patch.object(scaling, "kafka") as kafka:
                    with self.assertRaises(subprocess.CalledProcessError):
                        scaling.prepare(Path(directory), ["syslog"], 1, 1, 100, "g")
                    kafka.assert_not_called()


class RecordedScalingResultsTests(unittest.TestCase):
    # Scenario: The compact October 7 artifact represents the approved eighteen runs.
    # Guarantees: Every cell/rate, image identity and unchanged assignment is retained.
    def test_recorded_matrix_provenance_and_partition_coverage(self):
        result = json.loads(
            (DASHBOARD / "results/kafka_syslog_scaling_20261007.json").read_text())
        self.assertEqual(
            result["source_commit"], "a60b22791862c5e90a5752c668c7dba4bc306748")
        self.assertIsNone(result["engine_image_build_source_commit"])
        self.assertEqual(set(result["images"]),
                         {"load-generator", "kafka-broker", "kafka-consumer"})
        cells = ((1, 1, 1), (2, 1, 1), (4, 1, 1), (1, 1, 2), (1, 1, 4),
                 (2, 1, 2), (4, 1, 4), (4, 2, 2), (4, 4, 1))
        expected = {(*cell, rate) for cell in cells for rate in (100000, 300000)}
        actual = []
        for row in result["cases"]:
            config = row["topology"]
            actual.append((config["cores"], len(config["topics"]),
                           config["partitions_per_topic"],
                           config["aggregate_target_rate"]))
            self.assertTrue(row["assignment_unchanged_at_captures"])
            self.assertEqual(config["rebalance_strategy"], "round_robin")
            assigned = row["assigned_core_count_at_captures"]
            self.assertEqual(assigned["before"], assigned["final"])
            self.assertEqual(assigned["before"],
                             min(config["cores"], config["total_partitions"]))
            partitions = row["partition_progress"]
            self.assertEqual(len(partitions), config["total_partitions"])
            self.assertTrue(all(p["committed_progress"] > 0 for p in partitions))
            counts = [p["records"] for p in row["producer_partition_counts"]]
            self.assertEqual(sum(counts), row["final_producer_logs"])
            self.assertLessEqual(max(counts) - min(counts), 1)
        self.assertEqual(len(actual), 18)
        self.assertEqual(set(actual), expected)

    # Scenario: Idle controls have incomplete counts while other cores are measured.
    # Guarantees: All four controls stay NA, complete sums and CPU normalization agree.
    def test_recorded_null_coverage_and_aggregates(self):
        result = json.loads(
            (DASHBOARD / "results/kafka_syslog_scaling_20261007.json").read_text())
        flagged = 0
        for row in result["cases"]:
            metrics, cores = row["metrics"], row["topology"]["cores"]
            values = [metrics[f"logs_received_rate_core{i}"]
                      for i in range(1, cores + 1)]
            if None in values:
                flagged += 1
                self.assertEqual(row["status"], "flagged")
                self.assertIsNone(metrics["logs_received_rate"])
                self.assertIsNone(row["final_local_perf_logs"])
                self.assertIsNone(row["final_not_observed_at_local_perf"])
                self.assertTrue(all("Unassigned core" in flag for flag in row["flags"]))
            else:
                self.assertEqual(row["status"], "verified")
                self.assertTrue(
                    math.isclose(sum(values), metrics["logs_received_rate"]))
                self.assertEqual(row["final_not_observed_at_local_perf"],
                                 row["final_producer_logs"]
                                 - row["final_local_perf_logs"])
            self.assertTrue(math.isclose(metrics["cpu_percentage_total_avg"] / cores,
                                         metrics["cpu_percentage_normalized_avg"]))
            self.assertGreaterEqual(metrics["ram_mib_max"], metrics["ram_mib_avg"])
            self.assertFalse(any(row["decode_error_series_present"]))
        self.assertEqual(flagged, 4)


if __name__ == "__main__":
    unittest.main()
