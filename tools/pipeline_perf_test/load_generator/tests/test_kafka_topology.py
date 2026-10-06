"""Explicit routing is balanced over successful enqueues, not attempts."""

import concurrent.futures
from collections import Counter
import sys
import threading

import pytest
from pydantic import ValidationError

from test_kafka_syslog import FakeProducer, await_exit, config
import kafka_syslog
import loadgen as module
from kafka_syslog import BalancedKafkaRouter, KafkaSyslogWorker
from loadgen import LoadGenConfig, LoadGenerator, _prometheus_label


def topology_fields(topic_count=2, partitions=4, **overrides):
    fields = dict(
        load_type="syslog", syslog_transport="kafka",
        kafka_topics=[f"syslog-{i}" for i in range(topic_count)],
        kafka_partitions_per_topic=partitions,
        threads=4, batch_size=1000, target_rate=400,
        message_size=1024, kafka_flush_timeout=0.05,
    )
    fields.update(overrides)
    return fields


def configure_sender(fields):
    configuration = LoadGenConfig(**fields)
    generator = LoadGenerator()
    # Direct send tests do not launch a controller; initialize its run state.
    generator.kafka_router = BalancedKafkaRouter(
        configuration.kafka_topics, configuration.kafka_partitions_per_topic
    )
    generator.kafka_target_metrics = {
        target: {"records": 0, "bytes": 0}
        for target in generator.kafka_router.targets
    }
    return generator, configuration.model_dump()


def worker_for(generator, args, fake):
    worker = KafkaSyslogWorker(generator, args, lambda _: fake)
    worker.producer = fake
    return worker


def target_counters(metrics, unit):
    return {
        key: value for key, value in metrics.items()
        if key.startswith(f"kafka_delivered_{unit}{{")
    }


# Scenario: Concurrent senders drive each topic and partition combination.
# Guarantees: Enqueues differ by at most one per target; delivery sums match totals.
@pytest.mark.parametrize("topic_count, partitions", [
    (1, 1), (1, 2), (1, 4), (1, 8), (2, 4), (4, 2), (8, 1),
])
def test_balanced_concurrent_topologies(topic_count, partitions):
    generator, args = configure_sender(topology_fields(topic_count, partitions))
    targets = generator.kafka_router.targets
    sequence = []
    sequence_lock = threading.Lock()

    class TrackingProducer(FakeProducer):
        def produce(self, topic, **kwargs):
            super().produce(topic, **kwargs)
            with sequence_lock:
                sequence.append((topic, kwargs["partition"]))

    producers = [TrackingProducer() for _ in range(4)]
    barrier = threading.Barrier(len(producers))
    payload = generator.create_syslog_message(
        "host", header_type="rfc5424", message_size=1024, line_delimited=False
    )
    completed = threading.Event()

    def send(fake):
        worker = worker_for(generator, args, fake)
        barrier.wait()
        for _ in range(67):
            assert worker.send(payload)
            fake.deliver()

    def observe():
        while not completed.is_set():
            snapshot = generator.get_metrics()
            assert sum(target_counters(snapshot, "records").values()) == (
                snapshot["logs_produced"]
            )
            assert sum(target_counters(snapshot, "bytes").values()) == (
                snapshot["logs_bytes_produced"]
            )
            status = generator.get_status()["metrics"]
            assert sum(target_counters(status, "records").values()) == (
                status["logs_produced"]
            )
            completed.wait(0.0001)

    with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
        observer = executor.submit(observe)
        try:
            list(executor.map(send, producers))
        finally:
            completed.set()
        observer.result()
    assert sequence == [targets[i % len(targets)] for i in range(268)]
    counts = Counter(sequence)
    assert set(counts) == set(targets)
    assert max(counts.values()) - min(counts.values()) <= 1
    metrics = generator.get_metrics()
    assert metrics["logs_produced"] == metrics["kafka_enqueued"] == 268
    assert metrics["bytes_sent"] == metrics["logs_bytes_produced"] == 268 * 1024
    assert metrics["kafka_pending"] == metrics["kafka_pending_bytes"] == 0
    assert all(fake.values == [payload] * 67 for fake in producers)
    for (topic, partition), count in counts.items():
        labels = f'topic="{topic}",partition="{partition}"'
        assert metrics[f"kafka_delivered_records{{{labels}}}"] == count
        assert metrics[f"kafka_delivered_bytes{{{labels}}}"] == count * 1024


# Scenario: A queue-full worker yields to another worker before retrying.
# Guarantees: Only accepted records advance shared routing; retries skip no target.
def test_retry_does_not_consume_target():
    generator, args = configure_sender(topology_fields())
    blocked = FakeProducer()
    blocked.full_attempts = 1
    other = FakeProducer()
    worker = worker_for(generator, args, blocked)
    other_worker = worker_for(generator, args, other)
    blocked.poll = lambda _: other_worker.send(b"x" * 1024)
    assert worker.send(b"x" * 1024)
    assert other.topics == blocked.topics == ["syslog-0"]
    assert other.partitions == [0]
    assert blocked.partitions == [1]
    assert generator.get_metrics()["logs_produced"] == 0
    assert sum(target_counters(generator.get_metrics(), "records").values()) == 0
    other.deliver()
    blocked.deliver()
    assert generator.get_metrics()["kafka_queue_full"] == 1
    assert generator.get_metrics()["logs_produced"] == 2
    assert generator.get_metrics()["failed"] == 0
    assert sum(target_counters(generator.get_metrics(), "records").values()) == 2


# Scenario: A balanced enqueue fails terminally or times out under queue pressure.
# Guarantees: The failed attempt advances neither the cursor nor delivery counters.
@pytest.mark.parametrize("queue_full", [False, True])
def test_failed_enqueue_does_not_consume_target(queue_full):
    generator, args = configure_sender(
        topology_fields(kafka_send_timeout=0.02)
    )
    fake = FakeProducer()
    fake.always_full = queue_full
    if not queue_full:
        fake.enqueue_error = RuntimeError("enqueue rejected")
    worker = worker_for(generator, args, fake)
    with pytest.raises((RuntimeError, TimeoutError)):
        worker.send(b"x" * 1024)
    fake.always_full = False
    fake.enqueue_error = None
    assert worker.send(b"x" * 1024)
    assert fake.topics == ["syslog-0"]
    assert fake.partitions == [0]
    fake.deliver()
    metrics = generator.get_metrics()
    assert metrics["kafka_enqueue_failed"] == 1
    assert metrics["logs_produced"] == 1
    assert sum(target_counters(metrics, "records").values()) == 1


# Scenario: Stop is requested while an explicit-route producer queue is full.
# Guarantees: The wait terminates without consuming a route or counting delivery.
def test_stop_interrupts_routed_queue_wait():
    generator, args = configure_sender(topology_fields(kafka_send_timeout=60))
    fake = FakeProducer()
    fake.always_full = True
    fake.poll = lambda _: generator.stop_event.set()
    worker = worker_for(generator, args, fake)
    assert worker.send(b"record") is False
    assert fake.attempts == 1
    metrics = generator.get_metrics()
    assert metrics["kafka_enqueued"] == metrics["kafka_pending"] == 0
    assert metrics["failed"] == metrics["logs_produced"] == 0
    assert sum(target_counters(metrics, "records").values()) == 0
    generator.stop_event.clear()
    fake.always_full = False
    assert worker.send(b"record")
    assert fake.partitions == [0]


# Scenario: A broker rejects one routed record but acknowledges the others.
# Guarantees: Per-target counters include only broker-confirmed success.
def test_per_target_delivery_failures():
    generator, args = configure_sender(topology_fields(1, 2))
    fake = FakeProducer()
    worker = worker_for(generator, args, fake)
    for _ in range(4):
        assert worker.send(b"x" * 1024)
    fake.queue.pop(0)("delivery timeout", None)
    fake.deliver()
    metrics = generator.get_metrics()
    assert metrics["kafka_delivery_failed"] == metrics["failed"] == 1
    assert metrics["kafka_enqueued"] == 4
    assert metrics["logs_produced"] == 3
    assert metrics["bytes_sent"] == 3 * 1024
    assert metrics['kafka_delivered_records{topic="syslog-0",partition="0"}'] == 1
    assert metrics['kafka_delivered_records{topic="syslog-0",partition="1"}'] == 2
    assert sum(target_counters(metrics, "records").values()) == 3
    assert sum(target_counters(metrics, "bytes").values()) == 3072
    assert generator.get_status()["status"] == "failed"


# Scenario: Explicit routing contains malformed, incomplete or ambiguous options.
# Guarantees: Model and API reject them before any producer starts.
@pytest.mark.parametrize("overrides", [
    {"kafka_topics": None},
    {"kafka_partitions_per_topic": None},
    {"kafka_topics": []},
    {"kafka_topics": ["duplicate", "duplicate"]},
    {"kafka_topics": [""]},
    {"kafka_topics": ["bad topic"]},
    {"kafka_topics": ["."]},
    {"kafka_topics": [".."]},
    {"kafka_topics": ['quote"']},
    {"kafka_topics": ["line\nbreak"]},
    {"kafka_topics": ["back\\slash"]},
    {"kafka_topics": ["non-ascii-\u00e9"]},
    {"kafka_topics": ["x" * 250]},
    {"kafka_topics": [True]},
    {"kafka_topics": [1]},
    {"kafka_topics": [None]},
    {"kafka_topics": "not-a-list"},
    {"kafka_topics": [f"topic-{i}" for i in range(257)]},
    {"kafka_partitions_per_topic": 0},
    {"kafka_partitions_per_topic": -1},
    {"kafka_partitions_per_topic": True},
    {"kafka_partitions_per_topic": False},
    {"kafka_partitions_per_topic": 1.5},
    {"kafka_partitions_per_topic": 2.0},
    {"kafka_partitions_per_topic": "4"},
    {"kafka_partitions_per_topic": 257},
    {"kafka_partitions_per_topic": 129},
    {"kafka_topic": "otel-syslog"},
    {"kafka_topic": "another-topic"},
    {"syslog_transport": "tcp"},
    {"syslog_transport": "udp"},
    {"load_type": "otlp"},
])
def test_topology_config_rejected(overrides, monkeypatch):
    fields = topology_fields(**overrides)
    generator = LoadGenerator()
    monkeypatch.setattr(module, "loadgen", generator)
    with pytest.raises(ValidationError):
        LoadGenConfig(**fields)
    response = module.app.test_client().post("/start", json=fields)
    assert response.status_code == 400
    assert generator.get_status()["status"] == "idle"
    assert generator.controller_thread is None


# Scenario: One topology field is omitted entirely rather than explicitly null.
# Guarantees: Defaults cannot silently select a partial explicit topology.
@pytest.mark.parametrize("missing", [
    "kafka_topics", "kafka_partitions_per_topic",
])
def test_topology_requires_both_fields(missing, monkeypatch):
    fields = topology_fields()
    del fields[missing]
    generator = LoadGenerator()
    monkeypatch.setattr(module, "loadgen", generator)
    with pytest.raises(ValidationError, match="must be set together"):
        LoadGenConfig(**fields)
    assert module.app.test_client().post("/start", json=fields).status_code == 400
    assert generator.controller_thread is None


# Scenario: A topology uses the exact supported cardinality boundary.
# Guarantees: 256 targets are accepted without changing the aggregate rate.
@pytest.mark.parametrize("topic_count, partitions", [(1, 256), (256, 1), (16, 16)])
def test_topology_cardinality_boundary(topic_count, partitions):
    fields = topology_fields(topic_count, partitions)
    configuration = LoadGenConfig(**fields)
    router = BalancedKafkaRouter(
        configuration.kafka_topics, configuration.kafka_partitions_per_topic
    )
    assert len(router.targets) == 256
    assert configuration.target_rate == 400


# Scenario: Legal topic names include mixed case, punctuation and maximum length.
# Guarantees: The configured topic order is preserved without sorting or rewriting.
def test_topic_names_and_list_order():
    topics = ["z" * 249, "Az_1.-9", "a"]
    generator, args = configure_sender(topology_fields(kafka_topics=topics))
    fake = FakeProducer()
    worker = worker_for(generator, args, fake)
    for _ in range(12):
        assert worker.send(b"raw record")
    assert fake.topics == [topic for topic in topics for _ in range(4)]
    assert fake.partitions == list(range(4)) * 3


# Scenario: API clients run an explicit topology then return to the legacy mode.
# Guarantees: Labels are drained atomically and reset per run, not kept stale.
def test_topology_http_lifecycle_and_reset(monkeypatch):
    fake = FakeProducer()
    generator = LoadGenerator(kafka_producer_factory=lambda _: fake)
    fake.on_enqueue = lambda: (
        generator.stop_event.set() if len(fake.values) == 8 else None
    )
    monkeypatch.setattr(module, "loadgen", generator)
    client = module.app.test_client()
    fields = topology_fields(4, 2, threads=1, target_rate=0)
    response = client.post("/start", json=fields)
    assert response.status_code == 200
    await_exit(generator)
    assert client.get("/status").status_code == 200
    response = client.post("/stop")
    assert response.status_code == 200
    snapshot = response.get_json()["metrics"]
    assert snapshot["logs_produced"] == 8
    assert len(target_counters(snapshot, "records")) == 8
    assert all(value == 1 for value in target_counters(snapshot, "records").values())
    assert all(value == 1024 for value in target_counters(snapshot, "bytes").values())
    text = client.get("/metrics").get_data(as_text=True)
    for key, value in snapshot.items():
        assert f"{key} {value}" in text.splitlines()

    monkeypatch.setattr(
        generator, "syslog_kafka_worker_thread",
        lambda *_: generator.stop_event.wait(),
    )
    assert generator.start(config())[1] == 200
    assert generator.kafka_router is None
    assert not target_counters(generator.get_metrics(), "records")
    assert generator.get_metrics()["logs_produced"] == 0
    assert generator.stop()[1] == 200


# Scenario: No broker callback arrives for an explicitly routed accepted record.
# Guarantees: All configured targets stay visible at zero, including unused ones.
def test_pending_targets_are_zero_initialized(monkeypatch):
    fake = FakeProducer()
    fake.flush_delivers = False
    generator = LoadGenerator(kafka_producer_factory=lambda _: fake)
    fake.on_enqueue = generator.stop_event.set
    monkeypatch.setattr(module, "loadgen", generator)
    assert generator.start(LoadGenConfig(**topology_fields(2, 4, threads=1)))[1] == 200
    await_exit(generator)
    snapshot = generator.get_metrics()
    client = module.app.test_client()
    text = client.get("/metrics").get_data(as_text=True).splitlines()
    for unit in ("records", "bytes"):
        counters = target_counters(snapshot, unit)
        assert len(counters) == 8
        assert set(counters.values()) == {0}
        for topic in ("syslog-0", "syslog-1"):
            for partition in range(4):
                labels = f'topic="{topic}",partition="{partition}"'
                assert f"kafka_delivered_{unit}{{{labels}}} 0" in text
    assert snapshot["kafka_pending"] == 1
    assert snapshot["logs_produced"] == snapshot["logs_bytes_produced"] == 0
    assert client.get("/status").status_code == 500


# Scenario: A run is started but every sender is still waiting to enqueue.
# Guarantees: Zero delivery series exist on /metrics before any producer callback.
def test_metrics_exist_before_first_enqueue(monkeypatch):
    generator = LoadGenerator()
    monkeypatch.setattr(
        generator, "syslog_kafka_worker_thread",
        lambda *_: generator.stop_event.wait(),
    )
    monkeypatch.setattr(module, "loadgen", generator)
    client = module.app.test_client()
    assert client.post("/start", json=topology_fields()).status_code == 200
    try:
        snapshot = client.get("/status").get_json()["metrics"]
        lines = client.get("/metrics").get_data(as_text=True).splitlines()
        for unit in ("records", "bytes"):
            counters = target_counters(snapshot, unit)
            assert len(counters) == 8
            assert all(value == 0 for value in counters.values())
            assert all(f"{key} 0" in lines for key in counters)
        assert snapshot["kafka_enqueued"] == snapshot["kafka_pending"] == 0
    finally:
        assert client.post("/stop").status_code == 200


# Scenario: The same server runs two distinct explicit topologies consecutively.
# Guarantees: New runs restart the cursor and remove all previous target labels.
def test_topology_restart_resets_labels_and_cursor():
    producers = []
    generator = LoadGenerator()

    def factory(_):
        fake = FakeProducer()
        fake.on_enqueue = generator.stop_event.set
        producers.append(fake)
        return fake

    generator.kafka_producer_factory = factory
    for topic in ["first-topic", "second-topic"]:
        fields = topology_fields(
            kafka_topics=[topic], kafka_partitions_per_topic=2, threads=1
        )
        assert generator.start(LoadGenConfig(**fields))[1] == 200
        await_exit(generator)
        assert generator.stop()[1] == 200
        metrics = generator.get_metrics()
        assert metrics["logs_produced"] == 1
        assert len(target_counters(metrics, "records")) == 2
        assert all(
            f'topic="{topic}"' in key
            for key in target_counters(metrics, "records")
        )
        assert producers[-1].topics == [topic]
        assert producers[-1].partitions == [0]


# Scenario: CLI users select multiple topics and an explicit partition count.
# Guarantees: CLI values reach the router without the default singular-topic conflict.
def test_topology_cli(monkeypatch):
    fake = FakeProducer()
    generator = LoadGenerator(kafka_producer_factory=lambda _: fake)
    fake.on_enqueue = generator.stop_event.set
    monkeypatch.setattr(module, "loadgen", generator)
    monkeypatch.setattr(sys, "argv", [
        "loadgen.py", "--load-type", "syslog", "--syslog-transport", "kafka",
        "--kafka-topics", "syslog-a", "syslog-b",
        "--kafka-partitions-per-topic", "4", "--threads", "1", "--duration", "10",
    ])
    module.main()
    assert generator.current_config["kafka_topics"] == ["syslog-a", "syslog-b"]
    assert generator.current_config["kafka_partitions_per_topic"] == 4
    assert fake.topics == ["syslog-a"]
    assert fake.partitions == [0]


# Scenario: CLI topology options are incomplete or include the singular topic.
# Guarantees: Invalid CLI combinations fail before a controller is started.
@pytest.mark.parametrize("options", [
    ["--kafka-topics", "syslog-a"],
    ["--kafka-partitions-per-topic", "4"],
    [
        "--kafka-topics", "syslog-a", "--kafka-partitions-per-topic", "4",
        "--kafka-topic", "otel-syslog",
    ],
])
def test_topology_cli_rejects_ambiguous_options(options, monkeypatch):
    generator = LoadGenerator()
    monkeypatch.setattr(module, "loadgen", generator)
    monkeypatch.setattr(sys, "argv", [
        "loadgen.py", "--load-type", "syslog", "--syslog-transport", "kafka",
        *options,
    ])
    with pytest.raises(ValidationError):
        module.main()
    assert generator.controller_thread is None


# Scenario: A legacy Kafka client supplies neither topology field.
# Guarantees: No partition is set and no per-target labels appear.
def test_omitted_topology_preserves_partitioner():
    generator = LoadGenerator()
    fake = FakeProducer()
    args = config(kafka_topic="legacy").model_dump()
    worker = worker_for(generator, args, fake)
    assert worker.send(b"one raw record")
    fake.deliver()
    assert fake.topics == ["legacy"]
    assert fake.partitions == [None]
    assert not target_counters(generator.get_metrics(), "records")
    assert generator.get_metrics()["logs_produced"] == 1


# Scenario: Worker counts and topic/partition layouts change at one total rate.
# Guarantees: Workers retain equal pacing shares and bounded chunks for every layout.
@pytest.mark.parametrize("topic_count, partitions", [
    (1, 1), (1, 8), (2, 4), (4, 2), (8, 1),
])
@pytest.mark.parametrize("threads", [1, 4])
def test_topology_preserves_aggregate_rate(
    topic_count, partitions, threads, monkeypatch
):
    generator, args = configure_sender(
        topology_fields(topic_count, partitions, threads=threads)
    )
    chunk = int(args["target_rate"] / threads * 0.05)
    for thread_id in range(threads):
        generator.stop_event.clear()
        fake = FakeProducer()
        fake.on_enqueue = lambda: (
            generator.stop_event.set() if len(fake.values) == chunk else None
        )
        worker = worker_for(generator, args, fake)
        clock = [0.0]
        deadlines = []

        def wait_until(deadline):
            deadlines.append(deadline)
            clock[0] = deadline

        monkeypatch.setattr(kafka_syslog.time, "monotonic", lambda: clock[0])
        monkeypatch.setattr(worker, "_wait_until", wait_until)
        worker.run(thread_id)
        assert deadlines == pytest.approx([
            thread_id / args["target_rate"],
            thread_id / args["target_rate"] + 0.05,
        ])
        assert len(fake.values) == chunk
        assert fake.flush_timeouts == [args["kafka_flush_timeout"]]
        assert all(
            len(value) == 1024 and value.startswith(b"<134>1 ") and b"\n" not in value
            for value in fake.values
        )
    snapshot = generator.get_metrics()
    assert snapshot["logs_produced"] / 0.05 == args["target_rate"]
    assert snapshot["kafka_pending"] == 0


# Scenario: Prometheus labels contain characters with special text-format meaning.
# Guarantees: Backslashes, quotes and newlines are escaped, never injected as syntax.
def test_prometheus_label_escaping():
    assert _prometheus_label('topic\\with"line\nbreak') == (
        'topic\\\\with\\"line\\nbreak'
    )
