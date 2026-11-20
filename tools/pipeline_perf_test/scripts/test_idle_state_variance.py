#!/usr/bin/env python3
"""Offline tests: python3 tools/pipeline_perf_test/scripts/test_idle_state_variance.py."""

import contextlib
import io
import json
import math
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import analyze_idle_state_variance as variance
import run_idle_state_variance as runner

CONFIG = "test_suites/integration/continuous/idle-state-windows-docker.yaml"


def write_sample(variance_dir: Path, index: int, value: float) -> None:
    iter_dir = variance_dir / f"iter_{index:02d}"
    iter_dir.mkdir(parents=True)
    payload = [{"name": variance.METRIC_NAME, "value": value, "unit": "MiB"}]
    (iter_dir / "benchmark.json").write_text(json.dumps(payload), encoding="utf-8")


class FakeOrchestrator:
    """Stands in for run_orchestrator.py by writing one benchmark JSON per run.

    `values` holds the idle memory reported by each call, in order. A value of
    None simulates a run that exits cleanly but writes no benchmark JSON.
    """

    def __init__(self, results_dir: Path, values, exit_codes=None):
        self.results_dir = results_dir
        self.values = list(values)
        self.exit_codes = list(exit_codes or [])
        self.calls = []

    def __call__(self, command, cwd):
        index = len(self.calls)
        self.calls.append((command, cwd))
        if index < len(self.exit_codes) and self.exit_codes[index] != 0:
            return self.exit_codes[index]
        value = self.values[index]
        if value is not None:
            self.results_dir.mkdir(parents=True, exist_ok=True)
            payload = [{"name": variance.METRIC_NAME, "value": value, "unit": "MiB"}]
            (self.results_dir / f"run_{index:02d}.json").write_text(
                json.dumps(payload), encoding="utf-8"
            )
        return 0


class VarianceAnalysisTests(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        # collect_samples logs each sample to stderr; keep test output clean.
        stderr = contextlib.redirect_stderr(io.StringIO())
        stderr.__enter__()
        self.addCleanup(stderr.__exit__, None, None, None)

    def tearDown(self):
        self._tmp.cleanup()

    # Scenario: five consistent idle memory samples are analyzed.
    # Guarantees: the summary reports the exact count, mean, median, min, and max,
    # and a CV under the threshold is labeled stable in the report.
    def test_consistent_samples_are_summarized_and_marked_stable(self):
        for i, value in enumerate([100.0, 101.0, 99.0, 100.0, 100.0], start=1):
            write_sample(self.tmp, i, value)

        result = variance.analyze(self.tmp, max_cv_percent=15.0, expected_sample_count=5)

        self.assertEqual(result.stats["count"], 5)
        self.assertAlmostEqual(result.stats["mean"], 100.0)
        self.assertAlmostEqual(result.stats["median"], 100.0)
        self.assertEqual(result.stats["min"], 99.0)
        self.assertEqual(result.stats["max"], 101.0)
        self.assertTrue(result.stable)
        self.assertIn("[MEMORY STABLE]", result.report)

    # Scenario: samples spread widely enough to exceed the CV threshold.
    # Guarantees: the result is labeled noisy, and the CLI still exits 0 because
    # noise is informational rather than gating.
    def test_noisy_samples_are_reported_without_failing(self):
        for i, value in enumerate([50.0, 150.0, 100.0], start=1):
            write_sample(self.tmp, i, value)

        result = variance.analyze(self.tmp, max_cv_percent=15.0)
        with contextlib.redirect_stdout(io.StringIO()) as out:
            exit_code = variance.main([str(self.tmp), "", "15", "3"])

        self.assertFalse(result.stable)
        self.assertGreater(result.stats["cv_percent"], 15.0)
        self.assertEqual(exit_code, 0)
        self.assertIn("[MEMORY NOISY]", out.getvalue())

    # Scenario: fewer valid samples are found than the run expected.
    # Guarantees: analysis raises and the CLI exits non-zero, so a partially
    # failed run cannot be reported as a meaningful result.
    def test_sample_count_mismatch_is_rejected(self):
        for i, value in enumerate([100.0, 101.0, 99.0], start=1):
            write_sample(self.tmp, i, value)

        with self.assertRaisesRegex(variance.VarianceAnalysisError, "expected 5"):
            variance.analyze(self.tmp, expected_sample_count=5)
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(variance.main([str(self.tmp), "", "15", "5"]), 1)

    # Scenario: some iterations report zero, negative, NaN, or infinite memory.
    # Guarantees: only finite positive samples are counted, so invalid reports
    # cannot shrink the CV or satisfy the expected sample count.
    def test_invalid_samples_are_excluded(self):
        values = [100.0, 0.0, -5.0, math.nan, math.inf, 102.0]
        for i, value in enumerate(values, start=1):
            write_sample(self.tmp, i, value)

        with contextlib.redirect_stderr(io.StringIO()):
            samples = variance.collect_samples(self.tmp)
            with self.assertRaises(variance.VarianceAnalysisError):
                variance.analyze(self.tmp, expected_sample_count=len(values))

        self.assertEqual(samples, [100.0, 102.0])

    # Scenario: the variance directory is missing, or exists with no samples.
    # Guarantees: both cases raise instead of producing an empty summary.
    def test_missing_or_empty_directory_is_rejected(self):
        with self.assertRaisesRegex(variance.VarianceAnalysisError, "not found"):
            variance.analyze(self.tmp / "missing")
        with self.assertRaisesRegex(variance.VarianceAnalysisError, "no idle-state samples"):
            variance.analyze(self.tmp)

    # Scenario: percentiles are computed for multi-value and single-value inputs.
    # Guarantees: values between ranks are linearly interpolated, and a single
    # sample is its own percentile.
    def test_percentile_interpolates_between_ranks(self):
        self.assertAlmostEqual(variance.percentile([1.0, 2.0, 3.0, 4.0, 5.0], 95.0), 4.8)
        self.assertEqual(variance.percentile([10.0], 95.0), 10.0)

    # Scenario: the CLI is given an output path for the summary JSON.
    # Guarantees: the file contains the median, p95, mean, stddev, and CV entries
    # with rounded values, matching the uploaded artifact format.
    def test_cli_writes_benchmark_style_summary(self):
        for i, value in enumerate([100.0, 102.0], start=1):
            write_sample(self.tmp, i, value)
        output_json = self.tmp / "out" / "variance.json"

        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            exit_code = variance.main([str(self.tmp), str(output_json), "15", "2"])

        self.assertEqual(exit_code, 0)
        entries = {e["name"]: e for e in json.loads(output_json.read_text(encoding="utf-8"))}
        self.assertEqual(
            sorted(entries),
            [
                "idle_ram_mib_cv_percent",
                "idle_ram_mib_mean",
                "idle_ram_mib_median",
                "idle_ram_mib_p95",
                "idle_ram_mib_stddev",
            ],
        )
        self.assertEqual(entries["idle_ram_mib_median"]["value"], 101.0)
        self.assertEqual(entries["idle_ram_mib_stddev"]["value"], 1.41)


class RunIterationsTests(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.results_dir = self.tmp / "results" / "bench"
        self.output_dir = self.tmp / "variance"
        self.output_dir.mkdir()

    def tearDown(self):
        self._tmp.cleanup()

    # Scenario: one warm-up and three measured runs each report a distinct value.
    # Guarantees: only measured runs are saved, each into its own iter_NN
    # directory in order, and every run invokes the orchestrator with the config
    # from the perf test directory.
    def test_measured_runs_are_saved_and_warmups_discarded(self):
        orchestrator = FakeOrchestrator(self.results_dir, [999.0, 10.0, 20.0, 30.0])

        with contextlib.redirect_stdout(io.StringIO()):
            runner.run_iterations(
                CONFIG, self.results_dir, self.output_dir, 1, 3, orchestrator, self.tmp
            )

        with contextlib.redirect_stderr(io.StringIO()):
            samples = variance.collect_samples(self.output_dir)
        self.assertEqual(samples, [10.0, 20.0, 30.0])
        self.assertEqual(len(orchestrator.calls), 4)
        for command, cwd in orchestrator.calls:
            self.assertEqual(command[1:], [runner.ORCHESTRATOR_SCRIPT, "--config", CONFIG])
            self.assertEqual(cwd, self.tmp)

    # Scenario: a stale benchmark JSON exists and the next measured run writes none.
    # Guarantees: results are cleared before each measured run, so the run fails
    # instead of silently reusing the stale sample.
    def test_stale_results_are_not_reused(self):
        self.results_dir.mkdir(parents=True)
        (self.results_dir / "stale.json").write_text("[]", encoding="utf-8")
        orchestrator = FakeOrchestrator(self.results_dir, [None])

        with contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(runner.RunnerError, "Measured iteration 1 produced no"):
                runner.run_iterations(
                    CONFIG, self.results_dir, self.output_dir, 0, 2, orchestrator, self.tmp
                )

    # Scenario: the orchestrator exits non-zero during warm-up.
    # Guarantees: the run stops immediately with the failing iteration and exit
    # code, and no measured runs are attempted.
    def test_orchestrator_failure_stops_the_run(self):
        orchestrator = FakeOrchestrator(self.results_dir, [100.0, 100.0], exit_codes=[2])

        with contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(
                runner.RunnerError, "Warmup iteration 1 failed with exit code 2"
            ):
                runner.run_iterations(
                    CONFIG, self.results_dir, self.output_dir, 1, 2, orchestrator, self.tmp
                )
        self.assertEqual(len(orchestrator.calls), 1)


class RunnerMainTests(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.summary = self.tmp / "summary.md"
        self.results_dir = self.tmp / "results" / "bench"
        self.output_dir = self.tmp / "results" / "variance"
        self.args = [
            "--config", CONFIG,
            "--results-dir", "results/bench",
            "--output-dir", "results/variance",
            "--summary-title", "Windows idle memory variance (1 core)",
        ]
        self.env = {
            "WARMUP_ITERATIONS": "1",
            "MEASURED_ITERATIONS": "3",
            "MAX_CV_PERCENT": "10",
            "GITHUB_STEP_SUMMARY": str(self.summary),
            "GITHUB_SHA": "abc123",
        }
        patcher = patch.object(runner, "_command_output", return_value="rustc 1.0.0")
        patcher.start()
        self.addCleanup(patcher.stop)

    def tearDown(self):
        self._tmp.cleanup()

    def run_main(self, orchestrator):
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return runner.main(self.args, self.env, orchestrator, self.tmp)

    # Scenario: a full run with settings from environment variables succeeds.
    # Guarantees: relative paths resolve against the perf test directory, the
    # output directory holds metadata, samples, and the variance summary, and
    # the job summary gets both the metadata table and the variance report.
    def test_successful_run_writes_metadata_variance_and_summary(self):
        orchestrator = FakeOrchestrator(self.results_dir, [999.0, 100.0, 101.0, 99.0])

        self.assertEqual(self.run_main(orchestrator), 0)

        metadata = json.loads(
            (self.output_dir / runner.METADATA_FILE).read_text(encoding="utf-8")
        )
        self.assertEqual(metadata["commit_sha"], "abc123")
        self.assertEqual(metadata["rust_version"], "rustc 1.0.0")
        self.assertEqual(metadata["warmup_iterations"], 1)
        self.assertEqual(metadata["measured_iterations"], 3)
        self.assertEqual(metadata["max_cv_percent"], 10.0)
        self.assertTrue((self.output_dir / runner.VARIANCE_FILE).is_file())
        self.assertEqual(len(orchestrator.calls), 4)
        summary = self.summary.read_text(encoding="utf-8")
        self.assertIn("### Runner metadata", summary)
        self.assertIn("| commit_sha | abc123 |", summary)
        self.assertIn("### Windows idle memory variance (1 core) (N=3)", summary)
        self.assertIn("[MEMORY STABLE]", summary)

    # Scenario: a measured run fails after metadata has been recorded.
    # Guarantees: main exits non-zero and writes no variance summary, while the
    # runner metadata remains available for diagnosing the failure.
    def test_failed_run_exits_non_zero_without_variance(self):
        orchestrator = FakeOrchestrator(self.results_dir, [100.0, 100.0], exit_codes=[0, 3])

        self.assertEqual(self.run_main(orchestrator), 1)

        self.assertTrue((self.output_dir / runner.METADATA_FILE).is_file())
        self.assertFalse((self.output_dir / runner.VARIANCE_FILE).exists())

    # Scenario: an output directory from an earlier run contains extra samples.
    # Guarantees: the directory is cleared first, so old samples never inflate
    # the sample count or affect the statistics.
    def test_previous_output_is_cleared(self):
        write_sample(self.output_dir, 9, 500.0)
        orchestrator = FakeOrchestrator(self.results_dir, [999.0, 100.0, 101.0, 99.0])

        self.assertEqual(self.run_main(orchestrator), 0)

        self.assertFalse((self.output_dir / "iter_09").exists())
        entries = json.loads((self.output_dir / runner.VARIANCE_FILE).read_text(encoding="utf-8"))
        self.assertEqual(entries[0]["value"], 100.0)


class ParseArgsTests(unittest.TestCase):
    REQUIRED = ["--config", CONFIG, "--results-dir", "r", "--output-dir", "o"]

    # Scenario: iteration settings come from environment variables, and one is
    # also passed as a flag.
    # Guarantees: environment variables set the defaults and explicit flags
    # override them.
    def test_environment_sets_defaults_and_flags_override(self):
        env = {"WARMUP_ITERATIONS": "2", "MEASURED_ITERATIONS": "7", "MAX_CV_PERCENT": "12.5"}

        from_env = runner.parse_args(self.REQUIRED, env)
        overridden = runner.parse_args(self.REQUIRED + ["--measured-iterations", "3"], env)

        self.assertEqual(
            (from_env.warmup_iterations, from_env.measured_iterations, from_env.max_cv_percent),
            (2, 7, 12.5),
        )
        self.assertEqual(overridden.measured_iterations, 3)

    # Scenario: no iteration settings are provided by flags or environment.
    # Guarantees: the defaults are one warm-up, five measured runs, and a 15% CV
    # threshold, matching the workflow configuration.
    def test_defaults_without_environment(self):
        args = runner.parse_args(self.REQUIRED, {})

        self.assertEqual(
            (args.warmup_iterations, args.measured_iterations, args.max_cv_percent),
            (1, 5, 15.0),
        )

    # Scenario: settings that cannot produce a meaningful variance are supplied.
    # Guarantees: fewer than two measured runs, negative warm-ups, and a
    # non-positive CV threshold are rejected before any run starts.
    def test_invalid_settings_are_rejected(self):
        invalid = [
            (self.REQUIRED, {"MEASURED_ITERATIONS": "1"}),
            (self.REQUIRED + ["--warmup-iterations", "-1"], {}),
            (self.REQUIRED + ["--max-cv-percent", "0"], {}),
            (self.REQUIRED, {"MEASURED_ITERATIONS": "five"}),
        ]
        for argv, env in invalid:
            with self.subTest(argv=argv, env=env):
                with contextlib.redirect_stderr(io.StringIO()):
                    with self.assertRaises(SystemExit):
                        runner.parse_args(argv, env)


if __name__ == "__main__":
    unittest.main()
