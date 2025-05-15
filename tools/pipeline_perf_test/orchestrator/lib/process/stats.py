"""The stats module manages resource usage statistics for perf processes

Provides classes and utilities for tracking and aggregating resource usage statistics
of deployed processes during OpenTelemetry performance tests.

This module enables consistent collection and analysis of performance metrics such as
CPU and memory usage across different deployment environments (Docker, Kubernetes, etc.).

These tools are used by the monitoring subsystem in the orchestrator to collect meaningful
performance data and produce standardized test results.
"""
import re
import threading
from typing import Dict

class ProcessStatsAggregation:
    """Class for aggregation of resource stats"""
    min: float
    max: float
    total: float
    samples: int

    def __init__(self):
        self.min = None
        self.max = None
        self.total = 0.0
        self.samples = 0

    def add_sample(self, sample: float):
        """Add a sample to the aggregation.

        Args:
            sample: A float representing the value observed
        """
        self.samples += 1
        self.total += sample
        if not self.min or self.min > sample:
            self.min = sample
        if not self.max or self.max < sample:
            self.max = sample


class ProcessStats:
    """Class for tracking observed process resource utilization stats"""
    def __init__(self):
        self.lock = threading.Lock()
        self.cpu = ProcessStatsAggregation()
        self.mem = ProcessStatsAggregation()

    def add_sample(self, cpu_percent: float, mem_mib: float):
        """Add an observation of cpu and memory utilizations stats

        Args:
            cpu_percent: Percentage utilization of single cpu expressed as a float.
            mem_mb: Memory utilization for the process in MiB.
        """
        with self.lock:
            self.cpu.add_sample(cpu_percent)
            self.mem.add_sample(mem_mib)

    def get_summary_string(self, metric_prefix: str, delimiter: str ="/") -> str:
        """Helper method to format min/avg/max stats."""
        summary = self.get_summary()
        return delimiter.join([
            f"{summary.get(metric_prefix + '_min', 0.0):.2f}",
            f"{summary.get(metric_prefix + '_avg', 0.0):.2f}",
            f"{summary.get(metric_prefix + '_max', 0.0):.2f}"
        ])

    def get_summary(self) -> Dict[str, float]:
        """Get a summary of observed resource utilization stats"""
        with self.lock:
            if not self.cpu.samples or not self.mem.samples:
                return {}
            return {
                "cpu_min": self.cpu.min,
                "cpu_avg": self.cpu.total / self.cpu.samples,
                "cpu_max": self.cpu.max,
                "mem_min": self.mem.min,
                "mem_avg": self.mem.total / self.mem.samples,
                "mem_max": self.mem.max,
            }


def parse_mem_to_mib(mem_str: str) -> float:
    """Parse the string returned by docker stats to a float representing the number of MiB in use by the container."""
    units_to_mib = {
        "kib": 1 / 1024,                   # 1 KiB = 1/1024 MiB
        "kb": 1000 / 1024 / 1024,          # 1 KB = 1000 bytes convert to MiB
        "mb": 1000000 / 1024 / 1024,       # 1 MB = 1,000,000 bytes convert to MiB
        "mi": 1,                           # already MiB
        "mib": 1,                          # already MiB
        "gb": 1_000_000_000 / 1024 / 1024, # 1 GB = 1,000,000,000 bytes MiB
        "gib": 1024                        # 1 GiB = 1024 MiB
    }
    match = re.match(r"([0-9.]+)([a-zA-Z]+)", mem_str)
    if not match:
        return 0.0
    value, unit = match.groups()
    return float(value) * units_to_mib.get(unit.lower(), 1)
