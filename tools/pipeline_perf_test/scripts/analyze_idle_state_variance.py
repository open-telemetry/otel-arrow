#!/usr/bin/env python3
"""
Aggregate repeated idle-memory measurements into summary statistics.

Hosted CI runners are noisy, so a single datapoint is not trustworthy. This
script consumes the benchmark JSON produced by N repeated runs of the same
idle-state config and reports central-tendency and spread statistics
(median, p95, mean, stddev, coefficient of variation).

The coefficient of variation (CV = stddev / mean) is compared against a
threshold to give a *non-gating* memory-stability signal: it reports whether
the memory measurements were stable enough to be meaningful, without failing
the build. CPU stability is not analyzed.

run_idle_state_variance.py calls `analyze` after collecting the repeated
runs. The script can also be run on its own against an existing directory.

Usage:
    python analyze_idle_state_variance.py <variance_dir> [output_json] [max_cv_percent] [expected_sample_count]

Where:
    <variance_dir>   Directory containing per-iteration subdirectories
                     (iter_01, iter_02, ...), each with a benchmark JSON file
                     that includes an "idle_ram_mib_avg" entry.
    [output_json]    Optional path to write the aggregated summary JSON.
    [max_cv_percent] Optional CV threshold (percent) for the stability signal.
                     Defaults to 15.0. This is informational only.
    [expected_sample_count]
                     Optional exact number of valid iteration samples required.

Exit code is always 0 (non-gating) unless the inputs are structurally invalid
(e.g. no measurements found or fewer samples than expected), which returns a
non-zero code so the workflow surfaces a genuine misconfiguration rather than
silently passing.
"""

import json
import math
import statistics
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

METRIC_NAME = "idle_ram_mib_avg"
DEFAULT_MAX_CV_PERCENT = 15.0


class VarianceAnalysisError(Exception):
    """Raised when the inputs cannot produce a trustworthy variance result."""


@dataclass(frozen=True)
class VarianceResult:
    """Summary statistics and the memory-stability verdict for one analysis."""

    stats: dict[str, float]
    max_cv_percent: float
    stable: bool

    @property
    def report(self) -> str:
        return format_report(self.stats, self.max_cv_percent, self.stable)


def extract_metric(json_file: Path, metric: str = METRIC_NAME) -> Optional[float]:
    """Extract a named metric value from a benchmark JSON file."""
    try:
        with open(json_file, "r", encoding="utf-8") as f:
            data = json.load(f)
    except (json.JSONDecodeError, IOError) as exc:
        print(f"Warning: could not read {json_file}: {exc}", file=sys.stderr)
        return None

    for entry in data:
        if entry.get("name") == metric:
            return entry.get("value")
    return None


def collect_samples(variance_dir: Path) -> list[float]:
    """Collect one idle memory sample per iteration subdirectory.

    Rejects non-finite or non-positive values as invalid measurements.
    """
    samples: list[float] = []
    iter_dirs = sorted(variance_dir.glob("iter_*"))
    for iter_dir in iter_dirs:
        json_files = sorted(iter_dir.glob("*.json"))
        if not json_files:
            print(f"Warning: no JSON found in {iter_dir}", file=sys.stderr)
            continue
        # Most recent file (filenames include timestamps).
        value = extract_metric(json_files[-1])
        if value is None:
            print(f"Warning: no {METRIC_NAME} in {json_files[-1]}", file=sys.stderr)
            continue
        float_value = float(value)
        # Reject non-finite (NaN, inf) or non-positive values.
        if not math.isfinite(float_value):
            print(f"Warning: non-finite {METRIC_NAME} in {iter_dir.name}: {float_value}", file=sys.stderr)
            continue
        if float_value <= 0.0:
            print(f"Warning: non-positive {METRIC_NAME} in {iter_dir.name}: {float_value} MiB", file=sys.stderr)
            continue
        samples.append(float_value)
        print(f"Found: {iter_dir.name} -> {float_value:.2f} MiB", file=sys.stderr)
    return samples


def percentile(sorted_values: list[float], pct: float) -> float:
    """Linear-interpolation percentile (pct in [0, 100])."""
    if not sorted_values:
        return 0.0
    if len(sorted_values) == 1:
        return sorted_values[0]
    rank = (pct / 100.0) * (len(sorted_values) - 1)
    lower = int(rank)
    upper = min(lower + 1, len(sorted_values) - 1)
    frac = rank - lower
    return sorted_values[lower] + (sorted_values[upper] - sorted_values[lower]) * frac


def summarize(samples: list[float]) -> dict[str, float]:
    """Compute summary statistics for a list of samples."""
    ordered = sorted(samples)
    mean = statistics.fmean(ordered)
    stddev = statistics.stdev(ordered) if len(ordered) > 1 else 0.0
    cv_percent = (stddev / mean * 100.0) if mean else 0.0
    return {
        "count": len(ordered),
        "min": ordered[0],
        "max": ordered[-1],
        "mean": mean,
        "median": statistics.median(ordered),
        "p95": percentile(ordered, 95.0),
        "stddev": stddev,
        "cv_percent": cv_percent,
    }


def format_report(stats: dict[str, float], max_cv_percent: float, stable: bool) -> str:
    """Format a concise, CI-summary-friendly report."""
    lines = [
        "=" * 72,
        "IDLE MEMORY VARIANCE ANALYSIS (repeated measurements)",
        "=" * 72,
        "",
        f"  Metric:            {METRIC_NAME}",
        f"  Samples (N):       {int(stats['count'])}",
        f"  Median:            {stats['median']:.2f} MiB",
        f"  p95:               {stats['p95']:.2f} MiB",
        f"  Mean:              {stats['mean']:.2f} MiB",
        f"  Min / Max:         {stats['min']:.2f} / {stats['max']:.2f} MiB",
        f"  Std dev:           {stats['stddev']:.2f} MiB",
        f"  CV (stddev/mean):  {stats['cv_percent']:.2f}%  (threshold {max_cv_percent:.1f}%)",
        "",
    ]
    if stable:
        lines += [
            f"[MEMORY STABLE] CV {stats['cv_percent']:.2f}% <= {max_cv_percent:.1f}% : memory",
            "                measurements are consistent enough to be meaningful.",
        ]
    else:
        lines += [
            f"[MEMORY NOISY]  CV {stats['cv_percent']:.2f}% > {max_cv_percent:.1f}% : memory",
            "                measurements are noisy; treat results as directional only (non-gating).",
        ]
    lines.append("=" * 72)
    return "\n".join(lines)


def build_output(stats: dict[str, float], max_cv_percent: float, stable: bool) -> list[dict]:
    """Build a benchmark-style JSON payload for artifact upload."""
    return [
        {"name": "idle_ram_mib_median", "value": round(stats["median"], 2), "unit": "MiB",
         "extra": f"Median idle memory over {int(stats['count'])} runs"},
        {"name": "idle_ram_mib_p95", "value": round(stats["p95"], 2), "unit": "MiB",
         "extra": "95th percentile idle memory"},
        {"name": "idle_ram_mib_mean", "value": round(stats["mean"], 2), "unit": "MiB",
         "extra": "Mean idle memory"},
        {"name": "idle_ram_mib_stddev", "value": round(stats["stddev"], 2), "unit": "MiB",
         "extra": "Standard deviation of idle memory"},
        {"name": "idle_ram_mib_cv_percent", "value": round(stats["cv_percent"], 2), "unit": "%",
         "extra": f"Coefficient of variation; threshold={max_cv_percent}% stable={stable}"},
    ]


def analyze(
    variance_dir: Path,
    max_cv_percent: float = DEFAULT_MAX_CV_PERCENT,
    expected_sample_count: Optional[int] = None,
) -> VarianceResult:
    """Analyze the iteration samples under `variance_dir`.

    Raises VarianceAnalysisError when the directory is missing, holds no
    valid samples, or holds a different number of valid samples than
    `expected_sample_count`. A noisy result is not an error.
    """
    if not variance_dir.exists():
        raise VarianceAnalysisError(f"variance directory not found: {variance_dir}")

    samples = collect_samples(variance_dir)
    if not samples:
        raise VarianceAnalysisError("no idle-state samples collected")
    if expected_sample_count is not None and len(samples) != expected_sample_count:
        raise VarianceAnalysisError(
            f"expected {expected_sample_count} idle-state samples, collected {len(samples)}"
        )

    stats = summarize(samples)
    return VarianceResult(
        stats=stats,
        max_cv_percent=max_cv_percent,
        stable=stats["cv_percent"] <= max_cv_percent,
    )


def write_output(result: VarianceResult, output_json: Path) -> None:
    """Write the benchmark-style summary JSON for `result`."""
    output_json.parent.mkdir(parents=True, exist_ok=True)
    with open(output_json, "w", encoding="utf-8") as f:
        json.dump(build_output(result.stats, result.max_cv_percent, result.stable), f, indent=2)


def main(argv: Optional[list[str]] = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    if not args:
        print(__doc__, file=sys.stderr)
        return 1

    variance_dir = Path(args[0])
    output_json = Path(args[1]) if len(args) > 1 and args[1] else None
    max_cv_percent = float(args[2]) if len(args) > 2 and args[2] else DEFAULT_MAX_CV_PERCENT
    expected_sample_count = int(args[3]) if len(args) > 3 and args[3] else None

    try:
        result = analyze(variance_dir, max_cv_percent, expected_sample_count)
    except VarianceAnalysisError as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 1

    print(result.report)

    if output_json is not None:
        write_output(result, output_json)
        print(f"Wrote summary JSON: {output_json}", file=sys.stderr)

    # Non-gating: a noisy result is reported but does not fail the job.
    return 0


if __name__ == "__main__":
    sys.exit(main())
