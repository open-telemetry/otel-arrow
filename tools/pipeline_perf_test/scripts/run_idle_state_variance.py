#!/usr/bin/env python3
"""
Run an idle-state scenario repeatedly and report idle memory variance.

Hosted CI runners are noisy, so one run is not a trustworthy datapoint. This
runner:

1. Records runner metadata (OS, CPU, memory, toolchain, run settings) to
   <output-dir>/runner-metadata.json.
2. Runs the orchestrator for the warm-up iterations and discards the results.
3. Runs the measured iterations, copying each run's benchmark JSON from
   --results-dir into <output-dir>/iter_NN.
4. Analyzes the samples with analyze_idle_state_variance and writes
   <output-dir>/variance.json.

When GITHUB_STEP_SUMMARY is set, the metadata table and the variance report are
appended to the job summary.

The iteration counts and CV threshold default to the WARMUP_ITERATIONS,
MEASURED_ITERATIONS, and MAX_CV_PERCENT environment variables, so a workflow
can tune them without changing the command.

Usage (relative paths are resolved against tools/pipeline_perf_test):
    python scripts/run_idle_state_variance.py \\
        --config test_suites/integration/continuous/idle-state-windows-docker.yaml \\
        --results-dir results/windows_idle_state_1core/gh-actions-benchmark \\
        --output-dir results/windows_idle_variance

Exits non-zero if an orchestrator run fails, a measured run produces no
benchmark JSON, or the analysis inputs are invalid. A noisy CV is reported but
does not fail the run.
"""

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Callable, Mapping, Optional

from analyze_idle_state_variance import (
    DEFAULT_MAX_CV_PERCENT,
    VarianceAnalysisError,
    VarianceResult,
    analyze,
    write_output,
)

PERF_TEST_DIR = Path(__file__).resolve().parent.parent
REPO_ROOT = PERF_TEST_DIR.parent.parent
ORCHESTRATOR_SCRIPT = "orchestrator/run_orchestrator.py"
METADATA_FILE = "runner-metadata.json"
VARIANCE_FILE = "variance.json"

RunCommand = Callable[[list[str], Path], int]


class RunnerError(Exception):
    """Raised when an iteration cannot produce a usable sample."""


def _run_subprocess(command: list[str], cwd: Path) -> int:
    return subprocess.run(command, cwd=cwd, check=False).returncode


def _command_output(command: list[str], cwd: Path) -> Optional[str]:
    try:
        completed = subprocess.run(
            command, cwd=cwd, check=False, capture_output=True, text=True
        )
    except OSError:
        return None
    if completed.returncode != 0:
        return None
    return completed.stdout.strip() or None


def _cpu_model() -> Optional[str]:
    if sys.platform == "win32":
        try:
            import winreg

            with winreg.OpenKey(
                winreg.HKEY_LOCAL_MACHINE,
                r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
            ) as key:
                return str(winreg.QueryValueEx(key, "ProcessorNameString")[0]).strip()
        except OSError:
            pass
    elif sys.platform.startswith("linux"):
        try:
            with open("/proc/cpuinfo", encoding="utf-8") as f:
                for line in f:
                    if line.startswith("model name"):
                        return line.split(":", 1)[1].strip()
        except OSError:
            pass
    return platform.processor() or None


def _physical_cores() -> Optional[int]:
    try:
        import psutil
    except ImportError:
        return None
    return psutil.cpu_count(logical=False)


def _total_memory_gb() -> Optional[float]:
    try:
        import psutil
    except ImportError:
        return None
    return round(psutil.virtual_memory().total / 1024**3, 2)


def collect_runner_metadata(
    settings: Mapping[str, object],
    env: Mapping[str, str] = os.environ,
    command_output: Optional[Callable[[list[str], Path], Optional[str]]] = None,
) -> dict[str, object]:
    """Describe the runner and run settings so results can be compared."""
    command_output = command_output or _command_output
    metadata: dict[str, object] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "commit_sha": env.get("GITHUB_SHA"),
        "ref": env.get("GITHUB_REF"),
        "runner_os": env.get("RUNNER_OS") or platform.system(),
        "runner_image": env.get("ImageOS"),
        "runner_image_version": env.get("ImageVersion"),
        "os_platform": platform.platform(),
        "os_version": platform.version(),
        "cpu_model": _cpu_model(),
        "logical_cores": os.cpu_count(),
        "physical_cores": _physical_cores(),
        "total_memory_gb": _total_memory_gb(),
        "rust_version": command_output(
            ["rustc", "--version"], REPO_ROOT / "rust" / "otap-dataflow"
        ),
        "python_version": platform.python_version(),
    }
    metadata.update(settings)
    return metadata


def format_metadata_markdown(metadata: Mapping[str, object]) -> str:
    rows = [f"| {key} | {value} |" for key, value in metadata.items()]
    return "\n".join(["### Runner metadata", "| Field | Value |", "|---|---|", *rows])


def format_variance_markdown(title: str, result: VarianceResult) -> str:
    return "\n".join([f"### {title}", "```", result.report, "```"])


def append_step_summary(text: str, env: Mapping[str, str] = os.environ) -> None:
    summary_path = env.get("GITHUB_STEP_SUMMARY")
    if not summary_path:
        return
    with open(summary_path, "a", encoding="utf-8") as f:
        f.write(text + "\n\n")


def _run_orchestrator(
    config: str, label: str, run_command: RunCommand, perf_test_dir: Path
) -> None:
    command = [sys.executable, ORCHESTRATOR_SCRIPT, "--config", config]
    exit_code = run_command(command, perf_test_dir)
    if exit_code != 0:
        raise RunnerError(f"{label} failed with exit code {exit_code}")


def run_iterations(
    config: str,
    results_dir: Path,
    output_dir: Path,
    warmup_iterations: int,
    measured_iterations: int,
    run_command: RunCommand = _run_subprocess,
    perf_test_dir: Path = PERF_TEST_DIR,
) -> None:
    """Run warm-up and measured iterations, saving each measured sample.

    Each measured run starts with an empty `results_dir`, so a run that writes
    nothing cannot reuse an earlier run's benchmark JSON.
    """
    for i in range(1, warmup_iterations + 1):
        print(f"=== Warmup iteration {i}/{warmup_iterations} (discarded) ===", flush=True)
        _run_orchestrator(config, f"Warmup iteration {i}", run_command, perf_test_dir)

    for i in range(1, measured_iterations + 1):
        print(f"=== Measured iteration {i}/{measured_iterations} ===", flush=True)
        if results_dir.exists():
            shutil.rmtree(results_dir)
        _run_orchestrator(config, f"Measured iteration {i}", run_command, perf_test_dir)

        json_files = sorted(results_dir.glob("*.json")) if results_dir.is_dir() else []
        if not json_files:
            raise RunnerError(
                f"Measured iteration {i} produced no benchmark JSON in {results_dir}"
            )
        iter_dir = output_dir / f"iter_{i:02d}"
        iter_dir.mkdir(parents=True)
        for json_file in json_files:
            shutil.copy2(json_file, iter_dir / json_file.name)


def _non_negative_int(value: str) -> int:
    number = int(value)
    if number < 0:
        raise argparse.ArgumentTypeError(f"must be >= 0, got {number}")
    return number


def _sample_count(value: str) -> int:
    number = int(value)
    if number < 2:
        raise argparse.ArgumentTypeError(
            f"at least 2 measured iterations are needed to estimate variance, got {number}"
        )
    return number


def _positive_float(value: str) -> float:
    number = float(value)
    if number <= 0:
        raise argparse.ArgumentTypeError(f"must be > 0, got {number}")
    return number


def parse_args(
    argv: Optional[list[str]] = None, env: Mapping[str, str] = os.environ
) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run an idle-state scenario repeatedly and report memory variance."
    )
    parser.add_argument("--config", required=True, help="Orchestrator test suite config.")
    parser.add_argument(
        "--results-dir",
        required=True,
        type=Path,
        help="Directory where the config writes its benchmark JSON.",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        type=Path,
        help="Directory for iteration samples, metadata, and the variance summary.",
    )
    # argparse applies `type` to string defaults, so invalid env values are
    # reported the same way as invalid flags.
    parser.add_argument(
        "--warmup-iterations",
        type=_non_negative_int,
        default=env.get("WARMUP_ITERATIONS") or "1",
        help="Discarded warm-up runs (default: $WARMUP_ITERATIONS or 1).",
    )
    parser.add_argument(
        "--measured-iterations",
        type=_sample_count,
        default=env.get("MEASURED_ITERATIONS") or "5",
        help="Measured runs (default: $MEASURED_ITERATIONS or 5).",
    )
    parser.add_argument(
        "--max-cv-percent",
        type=_positive_float,
        default=env.get("MAX_CV_PERCENT") or str(DEFAULT_MAX_CV_PERCENT),
        help="CV threshold for the stability verdict (default: $MAX_CV_PERCENT or 15).",
    )
    parser.add_argument(
        "--summary-title",
        default="Idle memory variance",
        help="Heading for the job summary section.",
    )
    return parser.parse_args(argv)


def _resolve(path: Path, base: Path) -> Path:
    return path if path.is_absolute() else base / path


def main(
    argv: Optional[list[str]] = None,
    env: Mapping[str, str] = os.environ,
    run_command: RunCommand = _run_subprocess,
    perf_test_dir: Path = PERF_TEST_DIR,
) -> int:
    args = parse_args(argv, env)
    results_dir = _resolve(args.results_dir, perf_test_dir)
    output_dir = _resolve(args.output_dir, perf_test_dir)

    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True)

    metadata = collect_runner_metadata(
        {
            "config": args.config,
            "warmup_iterations": args.warmup_iterations,
            "measured_iterations": args.measured_iterations,
            "max_cv_percent": args.max_cv_percent,
        },
        env,
    )
    metadata_json = json.dumps(metadata, indent=2)
    (output_dir / METADATA_FILE).write_text(metadata_json + "\n", encoding="utf-8")
    print(metadata_json, flush=True)
    append_step_summary(format_metadata_markdown(metadata), env)

    try:
        run_iterations(
            args.config,
            results_dir,
            output_dir,
            args.warmup_iterations,
            args.measured_iterations,
            run_command,
            perf_test_dir,
        )
        result = analyze(output_dir, args.max_cv_percent, args.measured_iterations)
    except (RunnerError, VarianceAnalysisError) as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 1

    print(result.report)
    write_output(result, output_dir / VARIANCE_FILE)
    append_step_summary(
        format_variance_markdown(
            f"{args.summary_title} (N={args.measured_iterations})", result
        ),
        env,
    )
    # Non-gating: a noisy result is reported but does not fail the job.
    return 0


if __name__ == "__main__":
    sys.exit(main())
