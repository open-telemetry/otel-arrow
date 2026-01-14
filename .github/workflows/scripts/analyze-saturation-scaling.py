#!/usr/bin/env python3
"""
Analyze saturation/scaling test results to compute scaling efficiency.

This script reads the benchmark JSON files from saturation tests with different
core counts and computes:
1. A summary table with throughput for each core setting
2. Scaling efficiency metrics (1.0 = perfect linear scaling)

Usage:
    python analyze-saturation-scaling.py <results_base_dir>

Example:
    python analyze-saturation-scaling.py tools/pipeline_perf_test/results
"""

import json
import re
import sys
from pathlib import Path
from typing import Optional


def extract_throughput(json_file: Path) -> Optional[float]:
    """Extract the logs_received_rate (throughput) from a benchmark JSON file."""
    try:
        with open(json_file, 'r') as f:
            data = json.load(f)
        
        for entry in data:
            if entry.get("name") == "logs_received_rate":
                return entry.get("value")
        
        return None
    except (json.JSONDecodeError, IOError) as e:
        print(f"Warning: Could not read {json_file}: {e}", file=sys.stderr)
        return None


def find_result_file(result_dir: Path) -> tuple[Optional[Path], Optional[float]]:
    """
    Find the benchmark JSON file in a saturation test result directory.
    
    In CI, there should be exactly one result file per core count.
    Returns (file_path, throughput) tuple.
    """
    gh_benchmark_dir = result_dir / "gh-actions-benchmark"
    if not gh_benchmark_dir.exists():
        return None, None
    
    json_files = list(gh_benchmark_dir.glob("*.json"))
    if not json_files:
        return None, None
    
    # In CI, there should be exactly one file. If multiple exist (e.g., local testing),
    # use the most recent one (filenames include timestamps).
    result_file = sorted(json_files)[-1]
    throughput = extract_throughput(result_file)
    
    if throughput is None:
        return None, None
    
    return result_file, throughput


def extract_core_count(dir_name: str) -> Optional[int]:
    """Extract the core count from a directory name like 'continuous_saturation_4core'."""
    match = re.search(r'continuous_saturation_(\d+)core', dir_name)
    if match:
        return int(match.group(1))
    return None


def compute_scaling_efficiency(throughputs: dict[int, float]) -> dict[int, float]:
    """
    Compute scaling efficiency for each core count.
    
    Scaling efficiency = (actual_throughput / baseline_throughput) / core_count
    
    Where baseline is the 1-core throughput. A value of 1.0 means perfect
    linear scaling (shared-nothing architecture working ideally).
    
    Returns a dict mapping core_count -> efficiency
    """
    if 1 not in throughputs:
        # Can't compute without a baseline
        return {}
    
    baseline = throughputs[1]
    if baseline <= 0:
        return {}
    
    efficiencies = {}
    for cores, throughput in throughputs.items():
        expected_throughput = baseline * cores
        efficiency = throughput / expected_throughput if expected_throughput > 0 else 0
        efficiencies[cores] = efficiency
    
    return efficiencies


def format_number(value: float) -> str:
    """Format a number with appropriate precision and thousands separators."""
    if value >= 1000:
        return f"{value:,.0f}"
    elif value >= 1:
        return f"{value:.2f}"
    else:
        return f"{value:.4f}"


def print_scaling_report(throughputs: dict[int, float], efficiencies: dict[int, float]):
    """Print a formatted scaling report."""
    if not throughputs:
        print("No saturation test results found.")
        return
    
    baseline = throughputs.get(1, 0)
    
    print()
    print("=" * 80)
    print("SATURATION/SCALING TEST RESULTS - SCALING ANALYSIS")
    print("=" * 80)
    print()
    print("Goal: Verify shared-nothing architecture with linear CPU scaling")
    print(f"Baseline (1 core): {format_number(baseline)} logs/sec")
    print()
    
    # Print table header
    print("-" * 80)
    print(f"{'Cores':<8} {'Throughput (logs/sec)':<25} {'Expected (linear)':<20} {'Scaling Efficiency':<15}")
    print("-" * 80)
    
    # Sort by core count and print rows
    for cores in sorted(throughputs.keys()):
        throughput = throughputs[cores]
        expected = baseline * cores if baseline > 0 else 0
        efficiency = efficiencies.get(cores, 0)
        
        # Color coding hint via emoji
        if efficiency >= 0.95:
            status = "✅"  # Excellent
        elif efficiency >= 0.85:
            status = "🟡"  # Good
        elif efficiency >= 0.70:
            status = "🟠"  # Acceptable
        else:
            status = "🔴"  # Poor
        
        print(f"{cores:<8} {format_number(throughput):<25} {format_number(expected):<20} {efficiency:.2%} {status}")
    
    print("-" * 80)
    print()
    
    # Summary statistics
    if len(efficiencies) > 1:
        avg_efficiency = sum(efficiencies.values()) / len(efficiencies)
        min_efficiency = min(efficiencies.values())
        max_cores = max(throughputs.keys())
        max_throughput = throughputs[max_cores]
        
        print("SUMMARY:")
        print(f"  • Average Scaling Efficiency: {avg_efficiency:.2%}")
        print(f"  • Minimum Scaling Efficiency: {min_efficiency:.2%}")
        print(f"  • Maximum Throughput ({max_cores} cores): {format_number(max_throughput)} logs/sec")
        print(f"  • Speedup ({max_cores} cores vs 1 core): {max_throughput/baseline:.1f}x" if baseline > 0 else "")
        print()
        
        if avg_efficiency >= 0.90:
            print("✅ EXCELLENT: The engine demonstrates near-perfect linear scaling.")
            print("   This validates the shared-nothing architecture design.")
        elif avg_efficiency >= 0.80:
            print("🟡 GOOD: The engine scales well with additional cores.")
            print("   Minor overhead observed, but architecture is mostly shared-nothing.")
        elif avg_efficiency >= 0.65:
            print("🟠 ACCEPTABLE: The engine shows reasonable scaling.")
            print("   Some contention or overhead present.")
        else:
            print("🔴 POOR: Scaling efficiency is below expectations.")
            print("   Investigate potential bottlenecks or shared resources.")
    
    print()
    print("=" * 80)
    print()


def generate_benchmark_json(throughputs: dict[int, float], efficiencies: dict[int, float]) -> list[dict]:
    """
    Generate benchmark JSON in the format expected by github-action-benchmark.
    
    Outputs scaling efficiency (0-1) where 1.0 = perfect linear scaling.
    Use with "customBiggerIsBetter" since higher efficiency is better.
    """
    benchmark_data = []
    
    if not efficiencies:
        return benchmark_data
    
    # Add per-core-count scaling efficiency metrics
    for cores in sorted(efficiencies.keys()):
        if cores == 1:
            continue  # Skip 1 core (baseline is always 100% efficient)
        
        efficiency = efficiencies[cores]
        
        benchmark_data.append({
            "name": f"scaling_efficiency_{cores}_cores",
            "value": round(efficiency, 4),
            "unit": "",
            "extra": f"Scaling efficiency at {cores} cores (1.0 = perfect linear scaling)"
        })
    
    # Add average scaling efficiency (excluding 1-core baseline)
    multi_core_efficiencies = [e for c, e in efficiencies.items() if c > 1]
    if multi_core_efficiencies:
        avg_efficiency = sum(multi_core_efficiencies) / len(multi_core_efficiencies)
        
        benchmark_data.append({
            "name": "scaling_efficiency_avg",
            "value": round(avg_efficiency, 4),
            "unit": "",
            "extra": "Average scaling efficiency across all multi-core tests (1.0 = perfect)"
        })
    
    return benchmark_data


def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze-saturation-scaling.py <results_base_dir> [output_json_path]", file=sys.stderr)
        print("Example: python analyze-saturation-scaling.py tools/pipeline_perf_test/results scaling-metrics.json", file=sys.stderr)
        sys.exit(1)
    
    results_base = Path(sys.argv[1])
    output_json_path = Path(sys.argv[2]) if len(sys.argv) > 2 else None
    
    if not results_base.exists():
        print(f"Error: Results directory not found: {results_base}", file=sys.stderr)
        sys.exit(1)
    
    # Find saturation test result directories using specific glob pattern
    # These directories are named: continuous_saturation_1core, continuous_saturation_2core, etc.
    throughputs: dict[int, float] = {}
    
    saturation_dirs = list(results_base.glob("continuous_saturation_*core"))
    if not saturation_dirs:
        print("No saturation test results found (looking for continuous_saturation_*core directories)", file=sys.stderr)
        sys.exit(0)
    
    for entry in sorted(saturation_dirs):
        core_count = extract_core_count(entry.name)
        if core_count is None:
            continue
        
        result_file, throughput = find_result_file(entry)
        if result_file is None or throughput is None:
            print(f"Warning: No valid result file found for {entry.name}", file=sys.stderr)
            continue
        
        throughputs[core_count] = throughput
        print(f"Found: {core_count} core(s) -> {format_number(throughput)} logs/sec", file=sys.stderr)
    
    # Compute scaling efficiency
    efficiencies = compute_scaling_efficiency(throughputs)
    
    # Print the report
    print_scaling_report(throughputs, efficiencies)
    
    # Generate and save benchmark JSON if output path provided
    if output_json_path:
        benchmark_data = generate_benchmark_json(throughputs, efficiencies)
        with open(output_json_path, 'w') as f:
            json.dump(benchmark_data, f, indent=2)
        print(f"Benchmark JSON written to: {output_json_path}", file=sys.stderr)
    
    # Exit with non-zero if scaling is poor (for CI alerting)
    if efficiencies:
        avg_efficiency = sum(efficiencies.values()) / len(efficiencies)
        if avg_efficiency < 0.50:
            print("Warning: Very poor scaling efficiency detected!", file=sys.stderr)
            # Don't fail the build, just warn
            # sys.exit(1)


if __name__ == "__main__":
    main()
