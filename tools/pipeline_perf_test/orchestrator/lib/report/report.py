"""Reporting module for pipeline perf test orchestrator.

Provides utilities for generating performance test reports and analyzing results
from OpenTelemetry pipeline tests.

This module includes functions to parse logs, retrieve metrics from the backend,
and format a summary report containing key performance indicators such as logs sent,
logs received, and resource usage.


Used by the orchestration layer to produce post-test summaries and assist in test
validation and debugging.
"""
from typing import Tuple

import requests

def get_report_string(
    timestamp,
    args,
    logs_failed_count,
    logs_sent_count,
    target_process_stats=None
) -> None:
    """Return a string representing a loadtest report

    Args:
        - timestamp: the start timestamp that the test run began at
        - args: the command line args
        - logs_failed_count: the number of failed log messages
        - logs_sent_count: the number of sent log messages
        - target_process_stats: CPU and Memory stats for the target process
    """

    logs_received_backend_count = get_backend_received_count("http://localhost:5000/metrics")

    # Calculate total logs lost (including those that failed at loadgen side and those lost in transit)
    # Logs that failed at loadgen side
    loadgen_failed = logs_failed_count

    # Logs lost in transit (difference between successful sends and received count)
    transit_lost = logs_sent_count - logs_received_backend_count

    # Total logs lost includes both loadgen failures and transit losses
    total_logs_lost = loadgen_failed + transit_lost

    # Calculate logs sent rate (based on attempted sends, not successful ones)
    total_logs_attempted = logs_sent_count + logs_failed_count
    logs_sent_rate = total_logs_attempted / args.duration if args.duration > 0 else 0

    # Format rate for human readability (K/sec or M/sec)
    if logs_sent_rate >= 1000000:
        formatted_rate = f"{logs_sent_rate/1000000:.2f}M/sec"
    elif logs_sent_rate >= 1000:
        formatted_rate = f"{logs_sent_rate/1000:.2f}K/sec"
    else:
        formatted_rate = f"{logs_sent_rate:.2f}/sec"

    # Calculate percentage of logs lost
    logs_lost_percentage = (total_logs_lost / total_logs_attempted * 100) if total_logs_attempted > 0 else 0

    lines = []
    lines.append(f"Performance test run at: {timestamp}")
    lines.append(f"Test duration: {args.duration} seconds")
    lines.append(f"Deployment target: {args.deployment_target}")

    if args.deployment_target == "docker":
        lines.append(f"Collector config: {args.collector_config}\n")
    else:
        lines.append("Kubernetes manifests:")
        lines.append(f"- Collector: {args.k8s_collector_manifest}")
        lines.append(f"- Backend: {args.k8s_backend_manifest}")
        lines.append(f"- LoadGen: {args.k8s_loadgen_manifest}\n")

    lines.append("Results:")
    lines.append(f"- Total logs attempted: {total_logs_attempted}")
    lines.append(f"- Logs successfully sent by loadgen: {logs_sent_count}")
    lines.append(f"- Logs failed at loadgen: {logs_failed_count}")
    lines.append(f"- Logs received by backend: {logs_received_backend_count}")
    lines.append(f"- Logs lost in transit: {transit_lost}")
    lines.append(f"- Duration: {args.duration:.2f} seconds")
    lines.append(f"- Logs attempt rate: {formatted_rate} ({logs_sent_rate:.2f} logs/second)")
    lines.append(f"- Total logs lost: {total_logs_lost} (failed at loadgen + lost in transit)")
    lines.append(f"- Percentage of logs lost: {logs_lost_percentage:.2f}%")

    if target_process_stats:
        lines.append(f"- CPU (#Cores) min/avg/max: {target_process_stats.get_summary_string('cpu')}")
        lines.append(f"- Memory (MiB) min/avg/max: {target_process_stats.get_summary_string('mem')}")

    return "\n".join(lines)


def parse_logs_for_sent_count(logs: str) -> Tuple[int, int]:
    """
    Parse logs to extract the LOADGEN_LOGS_SENT and LOADGEN_LOGS_FAILED counts

    Args:
        logs: Log output from the load generator

    Returns:
        Tuple[int, int]: Number of logs sent and failed, or (0, 0) if not found
    """
    logs_sent = 0
    logs_failed = 0
    if not logs:
        return logs_sent, logs_failed

    for line in logs.strip().split("\n"):
        if "LOADGEN_LOGS_SENT:" in line:
            try:
                logs_sent = int(line.split("LOADGEN_LOGS_SENT:")[1].strip())
            except (IndexError, ValueError) as e:
                print(f"Failed to parse logs sent count: {e}")
        elif "LOADGEN_LOGS_FAILED:" in line:
            try:
                logs_failed = int(line.split("LOADGEN_LOGS_FAILED:")[1].strip())
            except (IndexError, ValueError) as e:
                print(f"Failed to parse logs failed count: {e}")

    if logs_sent == 0:
        print(f"Could not find LOADGEN_LOGS_SENT in logs")
    if logs_failed == 0:
        print(f"Could not find LOADGEN_LLOADGEN_LOGS_FAILEDOGS_FAILED in logs")

    return logs_sent, logs_failed


def get_backend_received_count(url: str) -> int:
    """
    Query the backend service in Kubernetes for the count of received logs.

    Args:
        url: The URL of the backend service metrics endpoint.

    Returns:
        int: The count of received logs, or -1 if the query fails.
    """
    print(f"\nQuerying backend service at {url} for received count...")

    try:
        # Query the metrics endpoint
        response = requests.get(url, timeout=10)
        data = response.json()
        count = data.get("received_logs", -1)
        return count
    except Exception as e:
        print(f"Failed to query backend service: {e}")
        return -1
