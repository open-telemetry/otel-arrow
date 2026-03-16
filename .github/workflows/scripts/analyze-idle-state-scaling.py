#!/usr/bin/env python3
"""
Analyze idle state test results to verify the linear memory scaling model.

This script reads the benchmark JSON files from idle state tests with different
core counts and:
1. Fits a linear regression: Memory = C + N * R
2. Validates the fit quality (R² value)
3. Reports the per-core overhead (R) and constant overhead (C)

The shared-nothing, thread-per-core architecture should exhibit linear memory
scaling where each additional core adds a fixed amount of memory overhead.

Usage:
    python analyze-idle-state-scaling.py <results_base_dir> [output_json_path]

Example:
    python analyze-idle-state-scaling.py tools/pipeline_perf_test/results memory-scaling.json
"""

import json
import re
import sys
from pathlib import Path
from typing import Optional


def extract_memory_mib(json_file: Path) -> Optional[float]:
    """Extract the idle_ram_mib_avg from a benchmark JSON file."""
    try:
        with open(json_file, 'r') as f:
            data = json.load(f)
        
        for entry in data:
            if entry.get("name") == "idle_ram_mib_avg":
                return entry.get("value")
        
        return None
    except (json.JSONDecodeError, IOError) as e:
        print(f"Warning: Could not read {json_file}: {e}", file=sys.stderr)
        return None


def find_result_file(result_dir: Path) -> tuple[Optional[Path], Optional[float]]:
    """
    Find the benchmark JSON file in an idle state test result directory.
    
    Returns (file_path, memory_mib) tuple.
    """
    gh_benchmark_dir = result_dir / "gh-actions-benchmark"
    if not gh_benchmark_dir.exists():
        return None, None
    
    json_files = list(gh_benchmark_dir.glob("*.json"))
    if not json_files:
        return None, None
    
    # Use the most recent file (filenames include timestamps)
    result_file = sorted(json_files)[-1]
    memory_mib = extract_memory_mib(result_file)
    
    if memory_mib is None:
        return None, None
    
    return result_file, memory_mib


def extract_core_count(dir_name: str) -> Optional[int]:
    """
    Extract the core count from a directory name.
    
    Handles:
    - idle_state_1core -> 1
    - idle_state_2cores -> 2
    - idle_state_4cores -> 4
    """
    match = re.search(r'idle_state_(\d+)cores?', dir_name)
    if match:
        return int(match.group(1))
    return None


def linear_regression(x_values: list[int], y_values: list[float]) -> tuple[float, float, float]:
    """
    Perform simple linear regression: y = C + R * x
    
    Returns (C, R, r_squared) tuple where:
    - C is the y-intercept (constant overhead)
    - R is the slope (per-core overhead)
    - r_squared is the coefficient of determination (fit quality)
    """
    n = len(x_values)
    if n < 2:
        return 0.0, 0.0, 0.0
    
    # Calculate means
    x_mean = sum(x_values) / n
    y_mean = sum(y_values) / n
    
    # Calculate slope (R) and intercept (C)
    numerator = sum((x - x_mean) * (y - y_mean) for x, y in zip(x_values, y_values))
    denominator = sum((x - x_mean) ** 2 for x in x_values)
    
    if denominator == 0:
        return y_mean, 0.0, 0.0
    
    R = numerator / denominator  # slope (per-core overhead)
    C = y_mean - R * x_mean       # intercept (constant overhead)
    
    # Calculate R² (coefficient of determination)
    y_pred = [C + R * x for x in x_values]
    ss_res = sum((y - yp) ** 2 for y, yp in zip(y_values, y_pred))
    ss_tot = sum((y - y_mean) ** 2 for y in y_values)
    
    r_squared = 1 - (ss_res / ss_tot) if ss_tot > 0 else 0.0
    
    return C, R, r_squared


def format_memory(value: float) -> str:
    """Format memory value with appropriate precision."""
    if value >= 100:
        return f"{value:.1f}"
    elif value >= 10:
        return f"{value:.2f}"
    else:
        return f"{value:.3f}"


def print_scaling_report(memory_data: dict[int, float], C: float, R: float, r_squared: float):
    """Print a formatted memory scaling report."""
    if not memory_data:
        print("No idle state test results found.")
        return
    
    print()
    print("=" * 80)
    print("IDLE STATE MEMORY SCALING ANALYSIS")
    print("=" * 80)
    print()
    print("Goal: Verify linear memory scaling (Memory = C + N × R)")
    print()
    print(f"  C (Constant Overhead):   {format_memory(C)} MiB")
    print(f"  R (Per-Core Overhead):   {format_memory(R)} MiB/core")
    print(f"  R² (Fit Quality):        {r_squared:.4f}")
    print()
    print("Formula: Memory (MiB) ≈ {:.1f} + {:.2f} × N".format(C, R))
    print()
    
    # Print table header
    print("-" * 80)
    print(f"{'Cores':<8} {'Actual (MiB)':<18} {'Predicted (MiB)':<18} {'Error (%)':<12} {'Status':<10}")
    print("-" * 80)
    
    # Sort by core count and print rows
    for cores in sorted(memory_data.keys()):
        actual = memory_data[cores]
        predicted = C + R * cores
        error_pct = abs(actual - predicted) / actual * 100 if actual > 0 else 0
        
        # Status indicator
        if error_pct <= 5:
            status = "✅"
        elif error_pct <= 10:
            status = "🟡"
        elif error_pct <= 20:
            status = "🟠"
        else:
            status = "🔴"
        
        print(f"{cores:<8} {format_memory(actual):<18} {format_memory(predicted):<18} {error_pct:.1f}%{'':<8} {status}")
    
    print("-" * 80)
    print()
    
    # Summary and interpretation
    print("SUMMARY:")
    print(f"  • Each additional core adds ~{format_memory(R)} MiB of memory overhead")
    print(f"  • Base memory footprint (shared infrastructure): ~{format_memory(C)} MiB")
    if len(memory_data) > 1:
        max_cores = max(memory_data.keys())
        min_cores = min(memory_data.keys())
        print(f"  • Memory range: {format_memory(memory_data[min_cores])} MiB ({min_cores} core) → {format_memory(memory_data[max_cores])} MiB ({max_cores} cores)")
    print()
    
    if r_squared >= 0.99:
        print("✅ EXCELLENT: Near-perfect linear fit (R² ≥ 0.99).")
        print("   Memory scaling follows the share-nothing model precisely.")
    elif r_squared >= 0.95:
        print("✅ GOOD: Strong linear fit (R² ≥ 0.95).")
        print("   Memory scaling is consistent with the thread-per-core architecture.")
    elif r_squared >= 0.85:
        print("🟡 ACCEPTABLE: Reasonable linear fit (R² ≥ 0.85).")
        print("   Some variance observed, but general trend is linear.")
    else:
        print("🔴 POOR: Weak linear fit (R² < 0.85).")
        print("   Memory scaling does not follow the expected linear model.")
        print("   Investigate potential memory leaks or non-linear allocations.")
    
    print()
    print("=" * 80)
    print()


def generate_benchmark_json(memory_data: dict[int, float], C: float, R: float, r_squared: float) -> list[dict]:
    """
    Generate benchmark JSON in the format expected by github-action-benchmark.
    """
    benchmark_data = []
    
    # Add the linear model parameters
    benchmark_data.append({
        "name": "idle_memory_constant_overhead_mib",
        "value": round(C, 2),
        "unit": "MiB",
        "extra": "Constant memory overhead (C in Memory = C + N*R)"
    })
    
    benchmark_data.append({
        "name": "idle_memory_per_core_overhead_mib",
        "value": round(R, 2),
        "unit": "MiB",
        "extra": "Per-core memory overhead (R in Memory = C + N*R)"
    })
    
    benchmark_data.append({
        "name": "idle_memory_r_squared",
        "value": round(r_squared, 4),
        "unit": "",
        "extra": "Linear fit quality (R²); 1.0 = perfect linear scaling"
    })
    
    # Add per-core-count memory readings
    for cores in sorted(memory_data.keys()):
        actual = memory_data[cores]
        predicted = C + R * cores
        error_pct = abs(actual - predicted) / actual * 100 if actual > 0 else 0
        
        benchmark_data.append({
            "name": f"idle_memory_{cores}core_mib",
            "value": round(actual, 2),
            "unit": "MiB",
            "extra": f"Idle memory at {cores} core(s); predicted={predicted:.1f} MiB, error={error_pct:.1f}%"
        })
    
    return benchmark_data


def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze-idle-state-scaling.py <results_base_dir> [output_json_path]", file=sys.stderr)
        print("Example: python analyze-idle-state-scaling.py tools/pipeline_perf_test/results memory-scaling.json", file=sys.stderr)
        sys.exit(1)
    
    results_base = Path(sys.argv[1])
    output_json_path = Path(sys.argv[2]) if len(sys.argv) > 2 else None
    
    if not results_base.exists():
        print(f"Error: Results directory not found: {results_base}", file=sys.stderr)
        sys.exit(1)
    
    # Find idle state result directories
    memory_data: dict[int, float] = {}
    
    # Look for directories matching idle_state_* pattern
    idle_dirs = list(results_base.glob("idle_state_*"))
    
    if not idle_dirs:
        print("No idle state test results found (looking for idle_state_* directories)", file=sys.stderr)
        sys.exit(0)
    
    print(f"Found {len(idle_dirs)} idle state result directories", file=sys.stderr)
    
    for entry in sorted(idle_dirs):
        # Extract core count
        core_count = extract_core_count(entry.name)
        if core_count is None:
            continue
        
        result_file, memory_mib = find_result_file(entry)
        if result_file is None or memory_mib is None:
            print(f"Warning: No valid result file found for {entry.name}", file=sys.stderr)
            continue
        
        memory_data[core_count] = memory_mib
        print(f"Found: {core_count} core(s) -> {format_memory(memory_mib)} MiB", file=sys.stderr)
    
    if len(memory_data) < 2:
        print("Error: Need at least 2 data points for linear regression", file=sys.stderr)
        sys.exit(1)
    
    # Perform linear regression
    x_values = list(memory_data.keys())
    y_values = [memory_data[x] for x in x_values]
    C, R, r_squared = linear_regression(x_values, y_values)
    
    # Print the report
    print_scaling_report(memory_data, C, R, r_squared)
    
    # Generate and save benchmark JSON
    benchmark_data = generate_benchmark_json(memory_data, C, R, r_squared)
    
    if output_json_path:
        with open(output_json_path, 'w') as f:
            json.dump(benchmark_data, f, indent=2)
        print(f"Benchmark JSON written to: {output_json_path}", file=sys.stderr)
    
    # Exit with warning if fit is poor
    if r_squared < 0.85:
        print("Warning: Poor linear fit detected! Memory scaling may not follow expected model.", file=sys.stderr)


if __name__ == "__main__":
    main()
