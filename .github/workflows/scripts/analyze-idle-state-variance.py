#!/usr/bin/env python3
"""
Aggregate repeated idle-state measurements into stable summary statistics.

Hosted CI runners are noisy, so a single datapoint is not trustworthy. This
script consumes the benchmark JSON produced by N repeated runs of the same
idle-state config and reports central-tendency and spread statistics
(median, p95, mean, stddev, coefficient of variation).

The coefficient of variation (CV = stddev / mean) is compared against a
threshold to give a *non-gating* stability signal: it reports whether the
measurements were stable enough to be meaningful, without failing the build.

Usage:
    python analyze-idle-state-variance.py <variance_dir> [output_json] [max_cv_percent]

Where:
    <variance_dir>   Directory containing per-iteration subdirectories
                     (iter_01, iter_02, ...), each with a benchmark JSON file
                     that includes an "idle_ram_mib_avg" entry.
    [output_json]    Optional path to write the aggregated summary JSON.
    [max_cv_percent] Optional CV threshold (percent) for the stability signal.
                     Defaults to 15.0. This is informational only.

Exit code is always 0 (non-gating) unless the inputs are structurally invalid
(e.g. no measurements found), which returns a non-zero code so the workflow
surfaces a genuine misconfiguration rather than silently passing.
"""

import json
import statistics
import sys
from pathlib import Path
from typing import Optional

METRIC_NAME = "idle_ram_mib_avg"
DEFAULT_MAX_CV_PERCENT = 15.0


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
    """Collect one idle memory sample per iteration subdirectory."""
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
        samples.append(float(value))
        print(f"Found: {iter_dir.name} -> {value:.2f} MiB", file=sys.stderr)
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


def print_report(stats: dict[str, float], max_cv_percent: float, stable: bool) -> None:
    """Print a concise, CI-summary-friendly report."""
    print()
    print("=" * 72)
    print("IDLE STATE VARIANCE ANALYSIS (repeated measurements)")
    print("=" * 72)
    print()
    print(f"  Metric:            {METRIC_NAME}")
    print(f"  Samples (N):       {int(stats['count'])}")
    print(f"  Median:            {stats['median']:.2f} MiB")
    print(f"  p95:               {stats['p95']:.2f} MiB")
    print(f"  Mean:              {stats['mean']:.2f} MiB")
    print(f"  Min / Max:         {stats['min']:.2f} / {stats['max']:.2f} MiB")
    print(f"  Std dev:           {stats['stddev']:.2f} MiB")
    print(f"  CV (stddev/mean):  {stats['cv_percent']:.2f}%  (threshold {max_cv_percent:.1f}%)")
    print()
    if stable:
        print(f"[STABLE] CV {stats['cv_percent']:.2f}% <= {max_cv_percent:.1f}% : measurements are")
        print("         consistent enough to be treated as meaningful.")
    else:
        print(f"[NOISY]  CV {stats['cv_percent']:.2f}% > {max_cv_percent:.1f}% : measurements are noisy;")
        print("         treat results as directional only (non-gating).")
    print("=" * 72)
    print()


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


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 1

    variance_dir = Path(sys.argv[1])
    output_json = Path(sys.argv[2]) if len(sys.argv) > 2 and sys.argv[2] else None
    max_cv_percent = float(sys.argv[3]) if len(sys.argv) > 3 and sys.argv[3] else DEFAULT_MAX_CV_PERCENT

    if not variance_dir.exists():
        print(f"Error: variance directory not found: {variance_dir}", file=sys.stderr)
        return 1

    samples = collect_samples(variance_dir)
    if not samples:
        print("Error: no idle-state samples collected", file=sys.stderr)
        return 1

    stats = summarize(samples)
    stable = stats["cv_percent"] <= max_cv_percent
    print_report(stats, max_cv_percent, stable)

    if output_json is not None:
        output_json.parent.mkdir(parents=True, exist_ok=True)
        with open(output_json, "w", encoding="utf-8") as f:
            json.dump(build_output(stats, max_cv_percent, stable), f, indent=2)
        print(f"Wrote summary JSON: {output_json}", file=sys.stderr)

    # Non-gating: a noisy result is reported but does not fail the job.
    return 0


if __name__ == "__main__":
    sys.exit(main())
