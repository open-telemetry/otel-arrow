#!/usr/bin/env python3
"""Analyze a comparison_dashboard suite's published results for anomalies.

Anomaly definitions (per rate dir):
  - dropped_<signal>_percentage  > 5%
  - <signal>_received_rate       < 95% of nominal rate (parse from dir name)
  - <signal>_received_rate       > 105% of nominal rate

Usage: analyze.py <slug>
Exits 0 if no anomalies, 1 if any are reported.

Note: `dropped_<signal>_percentage` can be reported as `None` or a strongly
negative number under loadgen-backpressure; we tolerate `None` and ignore
negative values (these are reporting artifacts, not gaps).
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] if Path(__file__).resolve().parents[1].name == "comparison_dashboard" else None
# Allow override via env / cwd: assume cwd is comparison_dashboard if .site/data exists
import os
if ROOT is None:
    cwd = Path.cwd()
    if (cwd / ".site" / "data" / "suite").exists():
        ROOT = cwd
    else:
        # search upward
        for parent in cwd.parents:
            if (parent / ".site" / "data" / "suite").exists():
                ROOT = parent
                break
if ROOT is None:
    print("ERROR: cannot find .site/data/suite directory. Run from comparison_dashboard/.", file=sys.stderr)
    sys.exit(2)

DATA_DIR = ROOT / ".site" / "data" / "suite"
DROP_PCT_THRESHOLD = 5.0
RECEIVED_RATIO_LOW = 0.95
RECEIVED_RATIO_HIGH = 1.05


def parse_rate(test_name):
    m = re.match(r"^(\d+)k$", test_name)
    return int(m.group(1)) * 1000 if m else None


def metric_value(metrics, name):
    for m in metrics:
        if m.get("name") == name:
            v = m.get("value")
            return None if v is None else float(v)
    return None


def detect_signal(slug):
    if "_logs_" in slug: return "logs"
    if "_metrics_" in slug: return "metrics"
    if "_traces_" in slug: return "traces"
    return "unknown"


def analyze_suite(slug):
    suite_dir = DATA_DIR / slug
    if not suite_dir.exists():
        return [f"FATAL: suite dir not found: {suite_dir}"]
    signal = detect_signal(slug)
    issues = []
    rate_dirs = sorted([p for p in suite_dir.iterdir() if p.is_dir()],
                       key=lambda p: parse_rate(p.name) or 0)
    if not rate_dirs:
        return [f"FATAL: no rate dirs under {suite_dir}"]
    if signal == "logs":
        drop_name, recv_name = "dropped_logs_percentage", "logs_received_rate"
    elif signal == "metrics":
        drop_name, recv_name = "dropped_metrics_percentage", "metrics_received_rate"
    elif signal == "traces":
        drop_name, recv_name = "dropped_spans_percentage", "spans_received_rate"
    else:
        return [f"FATAL: unknown signal in {slug}"]
    for rd in rate_dirs:
        nominal = parse_rate(rd.name)
        if nominal is None:
            issues.append(f"{rd.name}: cannot parse rate"); continue
        mfile = rd / "metrics.json"
        if not mfile.exists():
            issues.append(f"{rd.name}: missing metrics.json"); continue
        metrics = json.load(open(mfile))
        drop = metric_value(metrics, drop_name)
        recv = metric_value(metrics, recv_name)
        if recv is None:
            issues.append(f"{rd.name}: missing {recv_name}"); continue
        if drop is not None and drop > DROP_PCT_THRESHOLD:
            issues.append(f"{rd.name}: {drop_name}={drop:.2f}% > {DROP_PCT_THRESHOLD}%")
        ratio = recv / nominal if nominal else None
        if ratio is not None:
            if ratio < RECEIVED_RATIO_LOW:
                issues.append(f"{rd.name}: {recv_name}={recv:.0f}/s = {ratio*100:.1f}% of nominal {nominal} (low)")
            elif ratio > RECEIVED_RATIO_HIGH:
                issues.append(f"{rd.name}: {recv_name}={recv:.0f}/s = {ratio*100:.1f}% of nominal {nominal} (high)")
    return issues


def main():
    if len(sys.argv) != 2:
        print("usage: analyze.py <slug>", file=sys.stderr); sys.exit(2)
    slug = sys.argv[1]
    issues = analyze_suite(slug)
    if not issues:
        print(f"[OK] {slug}: no anomalies"); sys.exit(0)
    print(f"[ANOMALIES] {slug}:")
    for i in issues: print(f"  - {i}")
    sys.exit(1)


if __name__ == "__main__":
    main()
