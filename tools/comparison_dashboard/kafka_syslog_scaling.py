"""Bounded receiver-only topology evidence; never substitute missing item counters."""

import argparse
from datetime import datetime
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess

import yaml

if __package__:
    from . import kafka_syslog_receiver_only as local
    from .kafka_syslog_metrics import read_prometheus
else:
    import kafka_syslog_receiver_only as local
    from kafka_syslog_metrics import read_prometheus


def validate_topology(topics, partitions, cores, target_rate):
    if (not isinstance(topics, list) or not topics
            or any(not isinstance(topic, str) or len(topic) > 249
                   or not re.fullmatch(r"[A-Za-z0-9_.-]+", topic)
                   or topic in (".", "..") for topic in topics)
            or len(set(topics)) != len(topics)):
        raise ValueError("Topics must be unique explicit Kafka names")
    if (type(partitions) is not int or partitions < 1
            or len(topics) * partitions > 256):
        raise ValueError("Require 1..256 total partitions")
    if type(cores) is not int or not 1 <= cores <= 4:
        raise ValueError("The short scaling suite supports 1..4 allocated cores")
    if type(target_rate) is not int or target_rate <= 0:
        raise ValueError("Aggregate target rate must be a positive integer")
    return {(topic, partition) for topic in topics for partition in range(partitions)}


def kafka(tool, *arguments):
    command = ["docker", "exec", "kafka-broker", f"/opt/kafka/bin/{tool}.sh",
               "--bootstrap-server", "localhost:9092", *arguments]
    try:
        return subprocess.run(command, check=True, capture_output=True, text=True,
                              timeout=30).stdout
    except subprocess.CalledProcessError as error:
        raise RuntimeError(
            f"Kafka command failed: {command}\n{error.stdout}\n{error.stderr}"
        ) from error


def prepare(output, topics, partitions, cores, target_rate, group):
    validate_topology(topics, partitions, cores, target_rate)
    if not re.fullmatch(r"[A-Za-z0-9_.-]+", group):
        raise ValueError("Invalid consumer group")
    output.mkdir(parents=True, exist_ok=True)
    root = Path(__file__).resolve().parents[2]
    subprocess.run(["git", "diff", "--quiet", "HEAD", "--"], cwd=root, check=True)
    commit = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    local.write_json(output / "run-provenance.json", {
        "source_commit": commit,
        "engine_build_source": "unknown; retained image pinned separately",
    })
    descriptions = []
    for topic in topics:
        kafka("kafka-topics", "--create", "--topic", topic,
              "--partitions", str(partitions), "--replication-factor", "1")
        description = kafka("kafka-topics", "--describe", "--topic", topic)
        if re.findall(r"PartitionCount:\s*(\d+)", description) != [str(partitions)]:
            raise ValueError(f"Unexpected topic description: {description}")
        descriptions.append(description)
    (output / "topic-description.txt").write_text("\n".join(descriptions))
    local.write_json(output / "topology.json", {
        "topics": topics, "partitions_per_topic": partitions,
        "total_partitions": len(topics) * partitions, "cores": cores,
        "core_ids": list(range(1, cores + 1)), "group": group,
        "aggregate_target_rate": target_rate, "record_bytes": 1024,
        "rebalance_strategy": "round_robin",
        "client_ids": [f"dfe-kafka-syslog-core{i}" for i in range(1, cores + 1)],
    })


def read_topology(output):
    config = json.loads((output / "topology.json").read_text())
    validate_topology(config["topics"], config["partitions_per_topic"],
                      config["cores"], config["aggregate_target_rate"])
    if (config["core_ids"] != list(range(1, config["cores"] + 1))
            or config["total_partitions"] != len(config["topics"])
            * config["partitions_per_topic"]
            or config["client_ids"] != [f"dfe-kafka-syslog-core{i}"
                                        for i in config["core_ids"]]
            or not re.fullmatch(r"[A-Za-z0-9_.-]+", config["group"])
            or config["record_bytes"] != 1024):
        raise ValueError("Inconsistent saved topology")
    return config


def rows_after_header(text, required):
    lines = [line.split() for line in text.splitlines() if line.strip()]
    for index, columns in enumerate(lines):
        if required <= set(columns):
            return columns, lines[index + 1:]
    raise ValueError(f"Kafka output has no expected header {required}: {text}")


def parse_members(text, config):
    columns, rows = rows_after_header(text, {"CONSUMER-ID", "CLIENT-ID", "#PARTITIONS"})
    expected = validate_topology(config["topics"], config["partitions_per_topic"],
                                 config["cores"], config["aggregate_target_rate"])
    members, assigned = [], []
    for row in rows:
        if len(row) <= max(columns.index(field) for field in
                           ("CONSUMER-ID", "CLIENT-ID", "#PARTITIONS")):
            raise ValueError(f"Incomplete member row: {row}")
        client = row[columns.index("CLIENT-ID")]
        if client not in config["client_ids"]:
            raise ValueError(f"Unexpected client: {client}")
        count = int(row[columns.index("#PARTITIONS")])
        if count < 0:
            raise ValueError("Negative partition count")
        if "CURRENT-ASSIGNMENT" in columns:
            value = (row[columns.index("CURRENT-ASSIGNMENT")]
                     if count else "")
            entries = ([entry.split(":", 1) for entry in value.split(";")]
                       if count else [])
        elif "ASSIGNMENT" in columns:
            value = " ".join(row[columns.index("ASSIGNMENT"):])
            entries = re.findall(r"([\w.-]+)\(([\d,\s]+)\)", value)
        else:
            raise ValueError("No supported assignment column")
        targets = [(topic, int(p.strip())) for topic, partitions in entries
                   for p in partitions.split(",")]
        if len(targets) != count:
            raise ValueError("Incomplete member assignment")
        assigned.extend(targets)
        members.append({
            "client_id": client, "consumer_id": row[columns.index("CONSUMER-ID")],
            "core_id": int(client.removeprefix("dfe-kafka-syslog-core")),
            "partitions": [{"topic": t, "partition": p} for t, p in sorted(targets)],
        })
    if (len(members) != config["cores"]
            or {m["client_id"] for m in members} != set(config["client_ids"])
            or len({m["consumer_id"] for m in members}) != config["cores"]
            or len(assigned) != len(expected) or set(assigned) != expected):
        raise ValueError("Group membership/assignment does not match configured cores")
    return sorted(members, key=lambda row: row["core_id"])


def parse_offsets(text, config):
    columns, rows = rows_after_header(
        text, {"TOPIC", "PARTITION", "CURRENT-OFFSET", "LOG-END-OFFSET", "LAG"}
    )
    expected = validate_topology(config["topics"], config["partitions_per_topic"],
                                 config["cores"], config["aggregate_target_rate"])
    offsets = []
    for row in rows:
        if len(row) < len(columns):
            raise ValueError(f"Incomplete offset row: {row}")
        data = dict(zip(columns, row))
        item = {"topic": data["TOPIC"], "partition": int(data["PARTITION"])}
        for field, label in (("committed_offset", "CURRENT-OFFSET"),
                             ("log_end_offset", "LOG-END-OFFSET"),
                             ("committed_lag", "LAG")):
            item[field] = None if data[label] == "-" else int(data[label])
            if item[field] is not None and item[field] < 0:
                raise ValueError("Negative offset or lag")
        offsets.append(item)
        current, end, lag = (item[field] for field in
                             ("committed_offset", "log_end_offset", "committed_lag"))
        if (None not in (current, end, lag) and end - current != lag):
            raise ValueError("Offset/lag evidence is inconsistent")
    if (len(offsets) != len(expected)
            or {(r["topic"], r["partition"]) for r in offsets} != expected):
        raise ValueError("Missing or duplicate partition offsets")
    return offsets


def group_snapshot(output, phase):
    if phase not in ("before", "final"):
        raise ValueError("Unknown group snapshot phase")
    config = read_topology(output)
    result = {"started_at": local.timestamp()}
    for suffix, arguments in (
        ("members", ("--members", "--verbose")), ("offsets", ()),
    ):
        text = kafka("kafka-consumer-groups", "--describe", "--group",
                     config["group"], *arguments)
        (output / f"group-{phase}-{suffix}.txt").write_text(text)
        result[suffix] = (parse_members(text, config) if suffix == "members"
                          else parse_offsets(text, config))
    result["finished_at"] = local.timestamp()
    local.write_json(output / f"group-{phase}.json", result)


def core_counts(samples, core):
    selected = [(name, tags, value) for name, tags, value in samples
                if tags.get("otel_scope_pipeline_group_id") == "default"
                and tags.get("otel_scope_pipeline_id") == f"core{core}"
                and tags.get("otel_scope_core_id") == str(core)]
    items = [(name, tags, value) for name, tags, value in selected
             if name == "items_total" and tags.get("otel_scope_node_id") == "perf"
             and tags.get("otel_scope_name") == "node.input"
             and tags.get("signal") == "logs" and tags.get("outcome") == "success"]
    if len(items) > 1:
        raise ValueError("Duplicate successful Perf item series for one core")
    for name, tags, value in selected:
        if not math.isfinite(value) or value < 0:
            raise ValueError(f"Invalid core {core} metric {name}")
        if (name.endswith("_total") and value
                and (tags.get("outcome") in ("failure", "refused", "error")
                     or re.search(r"(errors?|failures?|nacks?|dropped)_total$", name))):
            raise ValueError(f"Core {core} reported {name}: {value}")
    return {
        "core_id": core,
        "local_perf_logs": local.count(items, "items_total") if items else None,
        "decode_error_series_present": any(
            "decod" in name and re.search(r"error|fail", name)
            for name, _, _ in selected
        ),
    }


def counts(output, phase, config):
    producer = read_prometheus(output / f"producer-{phase}.prom")
    consumer = read_prometheus(output / f"consumer-{phase}.prom")
    for name in local.PRODUCER_ERRORS:
        value = local.count(producer, name)
        if value and (name != "kafka_pending" or phase == "final"):
            raise ValueError(f"Producer reported {name}: {value}")
    produced = local.count(producer, "logs_produced")
    if local.count(producer, "bytes_sent") != produced * 1024:
        raise ValueError("Expected one 1024-byte Syslog value per delivered record")
    per_core = [core_counts(consumer, core) for core in config["core_ids"]]
    actual = {(tags.get("otel_scope_pipeline_id"), tags.get("otel_scope_core_id"))
              for name, tags, _ in consumer
              if name == "items_total" and tags.get("otel_scope_node_id") == "perf"
              and tags.get("otel_scope_pipeline_group_id") == "default"
              and tags.get("signal") == "logs" and tags.get("outcome") == "success"}
    if not actual.issubset({(f"core{core}", str(core))
                           for core in config["core_ids"]}):
        raise ValueError("Unexpected core in local Perf telemetry")
    total = (sum(r["local_perf_logs"] for r in per_core)
             if all(r["local_perf_logs"] is not None for r in per_core) else None)
    return {"phase": phase, "producer_logs": produced, "per_core": per_core,
            "local_perf_logs": total}


def instant(value):
    parsed = datetime.fromisoformat(value)
    if parsed.utcoffset() is None:
        raise ValueError("Capture timestamps require explicit timezone")
    return parsed


def verify(output):
    result_path = output / "verified-scaling.json"
    result_path.unlink(missing_ok=True)
    (output / "scaling-evidence.yaml").unlink(missing_ok=True)
    config = read_topology(output)
    snapshots = [counts(output, phase, config) for phase in local.PHASES]
    timing = [json.loads((output / f"capture-{phase}.json").read_text())
              for phase in local.PHASES]
    for phase, capture in zip(local.PHASES, timing):
        if capture["phase"] != phase:
            raise ValueError("Incorrect capture phase")
        for component, url in local.ENDPOINTS.items():
            row = capture["captures"][component]
            start = instant(row["started_at"])
            stop = instant(row["finished_at"])
            if row["url"] != url or stop < start:
                raise ValueError("Invalid capture timing or endpoint")
    for before, after in zip(timing, timing[1:]):
        for component in local.ENDPOINTS:
            if (instant(before["captures"][component]["finished_at"])
                    > instant(after["captures"][component]["started_at"])):
                raise ValueError("Snapshot times out of order")
    for before, after in zip(snapshots, snapshots[1:]):
        if after["producer_logs"] < before["producer_logs"]:
            raise ValueError("Producer counter reset")
        for first, last in zip(before["per_core"], after["per_core"]):
            a, b = first["local_perf_logs"], last["local_perf_logs"]
            if a is not None and b is not None and b < a:
                raise ValueError("Per-core counter reset")
    final = snapshots[-1]
    produced, received = final["producer_logs"], final["local_perf_logs"]
    if produced <= 0 or (received is not None and not 0 < received <= produced):
        raise ValueError("Invalid final input/Perf total")
    producer = read_prometheus(output / "producer-final.prom")
    targets = validate_topology(config["topics"], config["partitions_per_topic"],
                                config["cores"], config["aggregate_target_rate"])
    rows = [(tags, value) for name, tags, value in producer
            if name == "kafka_delivered_records"]
    if (len(rows) != len(targets)
            or {(tags.get("topic"), int(tags["partition"])) for tags, _ in rows}
            != targets):
        raise ValueError("Missing or duplicate producer partition counters")
    distribution = []
    for topic, partition in sorted(targets):
        labels = {"topic": topic, "partition": str(partition)}
        records = local.count(producer, "kafka_delivered_records", **labels)
        if local.count(producer, "kafka_delivered_bytes", **labels) != records * 1024:
            raise ValueError("Invalid partition delivery bytes")
        distribution.append({
            "topic": topic, "partition": partition, "records": records,
        })
    values = [row["records"] for row in distribution]
    if sum(values) != produced or min(values) <= 0 or max(values) - min(values) > 1:
        raise ValueError("Producer traffic is not balanced across all partitions")
    groups = {}
    for phase in ("before", "final"):
        saved = json.loads((output / f"group-{phase}.json").read_text())
        if instant(saved["finished_at"]) < instant(saved["started_at"]):
            raise ValueError("Invalid group capture timing")
        saved["members"] = parse_members(
            (output / f"group-{phase}-members.txt").read_text(), config)
        saved["offsets"] = parse_offsets(
            (output / f"group-{phase}-offsets.txt").read_text(), config)
        groups[phase] = saved
    if (instant(groups["before"]["finished_at"])
            > min(instant(row["started_at"])
                  for row in timing[0]["captures"].values())
            or instant(groups["final"]["started_at"])
            < max(instant(row["finished_at"])
                  for row in timing[-1]["captures"].values())):
        raise ValueError("Group scans must stay outside the timed metric window")
    flags = []
    for component in local.ENDPOINTS:
        duration = (
            instant(timing[1]["captures"][component]["started_at"])
            - instant(timing[0]["captures"][component]["started_at"])
        ).total_seconds()
        if not 19.9 <= duration <= 25:
            flags.append(
                f"Unexpected observation capture span: {component} {duration}s")
    if groups["before"]["members"] != groups["final"]["members"]:
        flags.append("Group membership or partition assignment changed across captures")
    progress = []
    first = {(r["topic"], r["partition"]): r for r in groups["before"]["offsets"]}
    for row in groups["final"]["offsets"]:
        before = first[(row["topic"], row["partition"])]
        a, b = before["committed_offset"], row["committed_offset"]
        delta = b - a if a is not None and b is not None else None
        if delta is not None and delta < 0:
            raise ValueError("Committed offset regressed")
        progress.append({**row, "committed_progress": delta})
        if delta is None or delta == 0:
            flags.append(
                f"No proven committed progress: {row['topic']}:{row['partition']}")
    coverage = []
    for first, last in zip(groups["before"]["members"], groups["final"]["members"]):
        active = bool(first["partitions"] or last["partitions"])
        core = first["core_id"]
        values = [snapshot["per_core"][core - 1]["local_perf_logs"]
                  for snapshot in snapshots]
        coverage.append({
            "core_id": core,
            "assigned_at_either_capture": active,
            "item_series_complete": all(value is not None for value in values),
            "observed_item_delta": (values[1] - values[0]
                                    if None not in values[:2] else None),
        })
        if active and None in values:
            flags.append(f"Missing ACTIVE-core Perf telemetry: core{core}")
        elif not active and None in values:
            flags.append(f"Unassigned core{core} has no emitted Perf item series; "
                         "all-core aggregate remains unavailable")
    for snapshot in snapshots:
        for row in snapshot["per_core"]:
            if row["local_perf_logs"] is None:
                continue
            if row["local_perf_logs"] > snapshot["producer_logs"]:
                raise ValueError("Per-core Perf count exceeds producer total")
    sample = (output / "kafka-record.txt").read_bytes()
    if len(sample) != 1025 or re.match(rb"<\d{1,3}>1 ", sample) is None:
        raise ValueError("Invalid sampled raw Syslog record")
    if (not sample.endswith(b"\n") or b"\n" in sample[:-1]
            or b"\r" in sample[:-1] or b"\0" in sample):
        raise ValueError("Console sample lacks expected delimiter")
    sample[:-1].decode("utf-8", errors="strict")
    images = {}
    for line in (output / "images.txt").read_text().splitlines():
        match = re.fullmatch(
            r"/(load-generator|kafka-broker|kafka-consumer) (sha256:[0-9a-f]{64})",
            line)
        if not match or match[1] in images:
            raise ValueError("Invalid image identity evidence")
        images[match[1]] = match[2]
    if len(images) != 3:
        raise ValueError("Require exactly three image identities")
    provenance = json.loads((output / "run-provenance.json").read_text())
    if not re.fullmatch(r"[0-9a-f]{40}", provenance["source_commit"]):
        raise ValueError("Missing pinned source commit")
    config_hash = hashlib.sha256(
        (output / "kafka-consumer-config.rendered.yaml").read_bytes()).hexdigest()
    result = {
        "status": "flagged" if flags else "verified", "flags": flags,
        "topology": config, "images": images, "provenance": provenance,
        "consumer_config_sha256": config_hash, "snapshots": snapshots,
        "capture_timing": timing, "producer_partition_counts": distribution,
        "group_snapshots": groups, "partition_progress": progress,
        "core_coverage": coverage,
        "configured_core_count": config["cores"],
        "assigned_core_count_at_captures": {
            phase: sum(bool(row["partitions"]) for row in group["members"])
            for phase, group in groups.items()
        },
        "producer_logs": produced, "local_perf_logs": received,
        "not_observed_at_local_perf": (
            produced - received if received is not None else None),
        "note": (
            "Short characterization, not sustained capacity. Final Perf counters "
            "precede shutdown after the bounded producer-off drain. Missing cores "
            "remain null, not zero. Broker assignment captures bracket observation "
            "and drain; committed progress/lag are not end-to-end acknowledgements. "
            "Cutoff deficits "
            "are not proven loss. Absent decode-error series are unavailable."
        ),
    }
    local.write_json(result_path, result)
    (output / "scaling-evidence.yaml").write_text(
        yaml.safe_dump(result, sort_keys=False), encoding="utf-8")
    print(f"Scaling evidence {result['status']}: "
          f"{received}/{produced}; {len(flags)} flags")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "group", "snapshot", "verify"))
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--topics", type=json.loads)
    parser.add_argument("--partitions", type=int)
    parser.add_argument("--cores", type=int)
    parser.add_argument("--target-rate", type=int)
    parser.add_argument("--group")
    parser.add_argument("--phase")
    args = parser.parse_args()
    if args.action == "prepare":
        values = (args.topics, args.partitions, args.cores,
                  args.target_rate, args.group)
        if any(value is None for value in values):
            parser.error(
                "prepare requires topics, partitions, cores, target-rate, group")
        prepare(args.output_dir, args.topics, args.partitions, args.cores,
                args.target_rate, args.group)
    elif args.action == "group":
        group_snapshot(args.output_dir, args.phase)
    elif args.action == "snapshot":
        local.snapshot(args.output_dir, args.phase)
    else:
        verify(args.output_dir)


if __name__ == "__main__":
    main()
