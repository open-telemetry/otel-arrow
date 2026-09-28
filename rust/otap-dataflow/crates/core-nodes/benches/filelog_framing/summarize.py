#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Summarize three Criterion baselines and a separate DHAT CSV run."""

import argparse
import csv
import json
import statistics
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("criterion", type=Path, help="Criterion output directory")
    parser.add_argument("allocations", type=Path, help="allocation harness CSV")
    args = parser.parse_args()
    with args.allocations.open(encoding="utf-8") as source:
        allocations = list(csv.DictReader(source))
    rows = []
    for allocation in allocations:
        name = allocation["case"]
        for chunk in (17, 131072):
            means = []
            for run in ("run1", "run2", "run3"):
                path = args.criterion / "filelog_framing" / name / str(chunk) / run / "estimates.json"
                with path.open(encoding="utf-8") as source:
                    means.append(json.load(source)["mean"]["point_estimate"])
            mean_ns = statistics.median(means)
            seconds = mean_ns / 1_000_000_000
            rows.append({
                "case": name,
                "chunk_bytes": chunk,
                "source_bytes": allocation["source_bytes"],
                "frames": allocation["frames"],
                "median_mean_ns": f"{mean_ns:.3f}",
                "min_mean_ns": f"{min(means):.3f}",
                "max_mean_ns": f"{max(means):.3f}",
                "source_mib_per_s": f"{int(allocation['source_bytes']) / seconds / (1024 * 1024):.3f}",
                "frames_per_s": f"{int(allocation['frames']) / seconds:.3f}",
                "allocations_per_frame": allocation["allocations_per_frame"] if chunk == 131072 else "",
                "peak_live_bytes": allocation["peak_live_bytes"] if chunk == 131072 else "",
            })
    if not rows:
        raise ValueError("allocation CSV contains no workloads")
    writer = csv.DictWriter(sys.stdout, fieldnames=list(rows[0]), lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)


if __name__ == "__main__":
    main()
