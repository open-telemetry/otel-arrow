"""Capture and verify pre-shutdown Syslog Kafka receiver -> local Perf evidence."""

import argparse
from datetime import datetime, timezone
import json
import math
from pathlib import Path
import re
from urllib.request import urlopen

if __package__:
    from .kafka_syslog_metrics import counter, read_prometheus
else:
    from kafka_syslog_metrics import counter, read_prometheus


PHASES = ("observation-start", "observation-stop", "final")
ENDPOINTS = {
    "producer": "http://localhost:18085/metrics",
    "consumer": ("http://localhost:18088/api/v1/telemetry/metrics"
                 "?format=prometheus&reset=false"),
}
PIPELINE = {
    "otel_scope_pipeline_group_id": "default",
    "otel_scope_pipeline_id": "main",
    "otel_scope_core_id": "1",
}
PRODUCER_ERRORS = (
    "failed", "kafka_pending", "kafka_delivery_failed",
    "kafka_enqueue_failed", "kafka_flush_timeouts",
)


def timestamp():
    return datetime.now(timezone.utc).isoformat()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n",
                    encoding="utf-8")


def snapshot(output, phase):
    """Scrape live endpoints once, without resetting cumulative counters."""
    if phase not in PHASES:
        raise ValueError(f"Unknown capture phase: {phase}")
    output.mkdir(parents=True, exist_ok=True)
    captures = {}
    for component, url in ENDPOINTS.items():
        started = timestamp()
        with urlopen(url, timeout=5) as response:
            content = response.read()
        (output / f"{component}-{phase}.prom").write_bytes(content)
        captures[component] = {
            "url": url, "started_at": started, "finished_at": timestamp(),
        }
    write_json(output / f"capture-{phase}.json", {
        "phase": phase, "captures": captures,
        "timing": "pre-shutdown; final is after the bounded producer-off drain",
    })


def count(samples, name, **labels):
    value = counter(samples, name, **labels)
    if not value.is_integer():
        raise ValueError(f"Non-integral record/byte counter {name}")
    return int(value)


def pipeline_samples(samples, node):
    return [
        (name, tags, value) for name, tags, value in samples
        if tags.get("otel_scope_node_id") == node
        and all(tags.get(key) == value for key, value in PIPELINE.items())
    ]


def read_counts(output, phase):
    producer = read_prometheus(output / f"producer-{phase}.prom")
    consumer = read_prometheus(output / f"consumer-{phase}.prom")
    received = count(
        pipeline_samples(consumer, "perf"), "items_total",
        otel_scope_name="node.input", signal="logs", outcome="success",
    )
    receiver = pipeline_samples(consumer, "receiver")
    required = (
        ("records_received_total", "receiver.kafka.consumer", {}),
        ("records_bytes_total", "receiver.kafka.consumer", {}),
        ("started_total", "receiver.kafka.messages", {"signal": "logs"}),
        ("completed_total", "receiver.kafka.messages", {"signal": "logs"}),
        ("payload_size_bytes_total", "receiver.kafka.messages", {"signal": "logs"}),
    )
    for name, scope, labels in required:
        count(receiver, name, otel_scope_name=scope, **labels)
    receiver_metrics = []
    for name, tags, value in receiver:
        scope = tags.get("otel_scope_name", "")
        if not scope.startswith("receiver.kafka."):
            continue
        if not math.isfinite(value) or value < 0:
            raise ValueError(f"Invalid receiver metric {name}: {value}")
        receiver_metrics.append({
            "name": name, "scope": scope, "signal": tags.get("signal"),
            "outcome": tags.get("outcome"), "value": value,
        })
    for name, tags, value in receiver + pipeline_samples(consumer, "perf"):
        if (name.endswith("_total")
                and (tags.get("outcome") in ("failure", "refused", "error")
                     or re.search(r"(errors?|failures?|nacks?|dropped)_total$", name))):
            if not math.isfinite(value) or value != 0:
                raise ValueError(f"Receiver/local Perf reported {name}: {value}")
    health = {name: count(producer, name) for name in PRODUCER_ERRORS}
    # Pending delivery is normal during observation, but not after /stop flushes.
    for name, value in health.items():
        if value and (name != "kafka_pending" or phase == "final"):
            raise ValueError(f"Producer reported {name}: {value}")
    produced = count(producer, "logs_produced")
    return {
        "phase": phase,
        "producer_logs": produced,
        "producer_bytes": count(producer, "bytes_sent"),
        "producer_health": health,
        "local_perf_logs": received,
        "not_observed_at_local_perf": produced - received,
        "receiver_metrics": receiver_metrics,
        "decode_error_series_present": any(
            "decod" in row["name"] and re.search(r"error|fail", row["name"])
            for row in receiver_metrics
        ),
    }


def verify(output, expect_full_delivery=False):
    result_path = output / "verified-delivery.json"
    result_path.unlink(missing_ok=True)
    samples = [read_counts(output, phase) for phase in PHASES]
    captures = [
        json.loads((output / f"capture-{phase}.json").read_text())
        for phase in PHASES
    ]
    for phase, capture in zip(PHASES, captures):
        if capture["phase"] != phase:
            raise ValueError("Snapshot phase does not match evidence filename")
        for component, url in ENDPOINTS.items():
            timing = capture["captures"][component]
            if timing["url"] != url:
                raise ValueError("Snapshot endpoint does not match local topology")
            start = datetime.fromisoformat(timing["started_at"])
            stop = datetime.fromisoformat(timing["finished_at"])
            if start.utcoffset() is None or stop.utcoffset() is None or stop < start:
                raise ValueError("Invalid snapshot timestamps")
    for before, after in zip(samples, samples[1:]):
        for field in ("producer_logs", "producer_bytes", "local_perf_logs"):
            if after[field] < before[field]:
                raise ValueError(f"Counter reset across snapshots: {field}")
    for before, after in zip(captures, captures[1:]):
        for component in ENDPOINTS:
            if (datetime.fromisoformat(before["captures"][component]["finished_at"])
                    > datetime.fromisoformat(
                        after["captures"][component]["started_at"])):
                raise ValueError("Snapshot capture times are out of order")
    final = samples[-1]
    produced, received = final["producer_logs"], final["local_perf_logs"]
    if produced <= 0 or final["producer_bytes"] != produced * 1024:
        raise ValueError("Producer record count/size does not match raw Syslog")
    if not 0 < received <= produced:
        raise ValueError("Local Perf count is empty or exceeds broker-confirmed logs")
    if expect_full_delivery and received != produced:
        raise ValueError(f"Incomplete smoke delivery: sent={produced}, got={received}")
    record = (output / "kafka-record.txt").read_bytes()
    if (len(record) != 1025 or not record.endswith(b"\n")
            or re.match(rb"<\d{1,3}>1 ", record) is None):
        raise ValueError("Kafka sample is not one 1024-byte RFC 5424 value")
    images = {}
    for line in (output / "images.txt").read_text().splitlines():
        match = re.fullmatch(
            r"/(load-generator|kafka-broker|kafka-consumer) (sha256:[0-9a-f]{64})",
            line,
        )
        if not match or match[1] in images:
            raise ValueError("Invalid or duplicate container image evidence")
        images[match[1]] = match[2]
    if len(images) != 3:
        raise ValueError("Expected image evidence for exactly three containers")
    if not (output / "kafka-consumer-config.rendered.yaml").read_text().strip():
        raise ValueError("Missing rendered consumer config evidence")
    result = {
        "endpoint": "receiver -> local Perf",
        "producer_logs": produced,
        "local_perf_logs": received,
        "not_observed_at_local_perf": produced - received,
        "full_delivery_required": expect_full_delivery,
        "images": images,
        "snapshots": samples,
        "capture_timing": captures,
        "note": (
            "Final local Perf success counters are captured while the admin listener "
            "is alive, after the bounded producer-off drain and BEFORE shutdown. "
            "They exclude progress during shutdown, unlike historical post-consumer-"
            "shutdown backend totals. Observation snapshots are sequential, not "
            "atomic; producer-minus-Perf is a signed backlog proxy, not proven loss. "
            "group_lag=0 is not drain evidence. Missing decode-error series are "
            "unavailable, not zero. Equality is count-level, not identity/dedup proof."
        ),
    }
    write_json(result_path, result)
    print(f"Verified local Perf: {received}/{produced} broker-confirmed logs.")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    capture = commands.add_parser("snapshot")
    capture.add_argument("--output-dir", type=Path, required=True)
    capture.add_argument("--phase", choices=PHASES, required=True)
    check = commands.add_parser("verify")
    check.add_argument("--output-dir", type=Path, required=True)
    check.add_argument("--expect-full-delivery", action="store_true")
    args = parser.parse_args()
    if args.command == "snapshot":
        snapshot(args.output_dir, args.phase)
    else:
        verify(args.output_dir, args.expect_full_delivery)


if __name__ == "__main__":
    main()
