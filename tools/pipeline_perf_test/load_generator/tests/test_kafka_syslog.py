"""Raw Kafka framing, delivery accounting, backpressure, and run lifecycle."""

import os
import sys
import threading
import time
from unittest.mock import MagicMock

import pytest
from pydantic import ValidationError

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

import kafka_syslog  # noqa: E402
import loadgen as module  # noqa: E402
from kafka_syslog import KafkaSyslogWorker, producer_config  # noqa: E402
from loadgen import (  # noqa: E402
    LoadGenConfig, LoadGenerator, _syslog_header, _generate_message_pool,
)


class FakeProducer:
    def __init__(self):
        self.queue = []
        self.values = []
        self.topics = []
        self.partitions = []
        self.attempts = 0
        self.full_attempts = 0
        self.always_full = False
        self.poll_delivers = False
        self.flush_delivers = True
        self.delivery_error = None
        self.enqueue_error = None
        self.flush_error = None
        self.on_enqueue = lambda: None
        self.enqueued = threading.Event()
        self.flush_timeouts = []

    def produce(self, topic, *, value, on_delivery, **routing):
        self.attempts += 1
        if self.enqueue_error:
            raise self.enqueue_error
        if self.always_full or self.attempts <= self.full_attempts:
            raise BufferError("queue full")
        self.queue.append(on_delivery)
        self.values.append(value)
        self.topics.append(topic)
        self.partitions.append(routing.get("partition"))
        self.on_enqueue()
        self.enqueued.set()

    def deliver(self):
        pending, self.queue = self.queue, []
        for callback in pending:
            callback(self.delivery_error, None)

    def poll(self, _timeout):
        if self.poll_delivers:
            self.deliver()

    def flush(self, timeout):
        self.flush_timeouts.append(timeout)
        if self.flush_error:
            raise self.flush_error
        if self.flush_delivers:
            self.deliver()
        return len(self.queue)


def config(**overrides):
    fields = dict(
        load_type="syslog", syslog_transport="kafka", syslog_format="rfc5424",
        threads=1, batch_size=3, target_rate=100, kafka_flush_timeout=0.05,
    )
    fields.update(overrides)
    return LoadGenConfig(**fields)


def sender(fake=None, **overrides):
    fake = fake or FakeProducer()
    generator = LoadGenerator(kafka_producer_factory=lambda _args: fake)
    worker = KafkaSyslogWorker(generator, config(**overrides).model_dump())
    worker.producer = fake
    return generator, worker, fake


def await_exit(generator):
    generator.controller_thread.join(timeout=2)
    assert not generator.controller_thread.is_alive()


# Scenario: Records are enqueued but no broker delivery callback has run.
# Guarantees: Success counters advance only on callbacks, with exact bytes once.
def test_delivery_only_metrics():
    generator, worker, fake = sender()
    assert worker.send(b"first")
    assert worker.send("second-\u00e9".encode())
    before = generator.get_metrics()
    assert before["logs_produced"] == before["bytes_sent"] == 0
    assert before["kafka_pending"] == before["kafka_enqueued"] == 2
    fake.deliver()
    fake.deliver()
    after = generator.get_metrics()
    assert after["logs_produced"] == 2
    assert after["bytes_sent"] == after["logs_bytes_produced"] == 14
    assert after["kafka_pending"] == after["kafka_pending_bytes"] == 0
    assert after["failed"] == 0


# Scenario: Multiple accepted records receive terminal broker errors.
# Guarantees: Failures count per record, not as delivery; error details stay bounded.
def test_terminal_delivery_failure():
    generator, worker, fake = sender()
    for _ in range(12):
        worker.send(b"x")
    fake.delivery_error = "broker rejected record"
    fake.deliver()
    metrics = generator.get_metrics()
    assert metrics["kafka_delivery_failed"] == metrics["failed"] == 12
    assert metrics["logs_produced"] == metrics["bytes_sent"] == 0
    assert metrics["kafka_pending"] == 0
    assert generator.stop_event.is_set()
    assert generator.get_status()["status"] == "failed"
    assert len(generator.errors) == 8


# Scenario: librdkafka reports transient queue pressure before accepting a record.
# Guarantees: Retries neither duplicate records nor count as failure or delivery.
def test_queue_pressure_retries_once():
    generator, worker, fake = sender()
    fake.full_attempts = 2
    assert worker.send(b"one record")
    assert fake.attempts == 3
    assert fake.values == [b"one record"]
    assert generator.get_metrics()["kafka_queue_full"] == 2
    assert generator.get_metrics()["failed"] == 0
    assert generator.get_metrics()["logs_produced"] == 0
    fake.deliver()
    assert generator.get_metrics()["logs_produced"] == 1


# Scenario: The producer queue never has room within the configured deadline.
# Guarantees: Sending terminates promptly with one failure and no false delivery.
def test_queue_pressure_has_deadline():
    generator, worker, fake = sender(kafka_send_timeout=0.03)
    fake.always_full = True
    started = time.monotonic()
    with pytest.raises(TimeoutError, match="send deadline"):
        worker.send(b"record")
    assert time.monotonic() - started < 0.5
    metrics = generator.get_metrics()
    assert metrics["failed"] == metrics["kafka_enqueue_failed"] == 1
    assert metrics["logs_produced"] == metrics["kafka_pending"] == 0


# Scenario: Stop arrives while a record is waiting for queue capacity.
# Guarantees: Backpressure waits are interruptible without reporting false sends.
def test_stop_interrupts_queue_wait():
    generator, worker, fake = sender(kafka_send_timeout=60)
    fake.always_full = True
    thread = threading.Thread(target=worker.send, args=(b"record",))
    thread.start()
    time.sleep(0.03)
    generator.stop_event.set()
    thread.join(timeout=0.5)
    assert not thread.is_alive()
    assert generator.get_metrics()["kafka_pending"] == 0
    assert generator.get_metrics()["logs_produced"] == 0
    assert generator.get_metrics()["failed"] == 0


# Scenario: A Kafka run emits a chunk of ordinary RFC5424 benchmark records.
# Guarantees: Records are raw, plain-body, exact-size UTF-8 without framing or CEF.
def test_raw_records_and_successful_flush():
    generator, _, fake = sender()
    fake.on_enqueue = lambda: (
        generator.stop_event.set() if len(fake.values) == 3 else None
    )
    assert generator.start(config())[1] == 200
    await_exit(generator)
    snapshot, code = generator.stop()
    assert code == 200
    assert snapshot["status"] == "stopped"
    assert len(fake.values) == 3
    for value in fake.values:
        assert len(value) == 1024
        assert value.startswith(b"<134>1 ")
        assert b" loadgen - - - " in value
        assert b"CEF:" not in value
        assert b"\n" not in value
        assert len(value.decode("utf-8")) == 1024
    assert fake.topics == ["otel-syslog"] * 3
    assert fake.partitions == [None] * 3
    assert fake.flush_timeouts == [0.05]
    assert snapshot["metrics"]["logs_produced"] == 3
    assert snapshot["metrics"]["bytes_sent"] == 3072


# Scenario: RFC3164 or RFC5424 CEF is selected explicitly for Kafka.
# Guarantees: A single record retains the CEF header and has no appended newline.
@pytest.mark.parametrize("header_type", ["rfc3164", "rfc5424"])
def test_explicit_cef_records(header_type):
    generator = LoadGenerator()
    value = generator.create_syslog_message(
        hostname="host", header_type=header_type, syslog_content_type="cef",
        message_size=1024, line_delimited=False,
    )
    assert b"CEF:0|" in value
    assert len(value) == 1024
    assert b"\n" not in value


# Scenario: A UTF-8 body must be shortened at a multibyte character boundary.
# Guarantees: Raw Kafka records preserve valid UTF-8 and exact total byte size.
def test_utf8_size_and_framing():
    generator = LoadGenerator()
    header = _syslog_header("host", "rfc5424")
    size = len(header) + 4
    value = generator.create_syslog_message(
        hostname="host", header_type="rfc5424",
        message_body="\u00e9\u6f22\U0001f642z", message_size=size,
        line_delimited=False,
    )
    assert len(value) == size
    expected = "\u00e9  "
    assert value.decode("utf-8") == header + expected


# Scenario: A requested size cannot hold the header or hostname is not legal Syslog.
# Guarantees: The generator rejects malformed messages rather than truncating headers.
@pytest.mark.parametrize("overrides", [
    {"message_size": 4}, {"hostname": "bad host"},
    {"hostname": "h\u00f6st"}, {"hostname": "x" * 256},
    {"header_type": "invalid"},
    {
        "syslog_content_type": "cef", "message_size": 100,
        "line_delimited": False,
    },
])
def test_invalid_message_rejected(overrides):
    options = dict(hostname="host", header_type="rfc5424", message_size=1024,
                   line_delimited=False)
    options.update(overrides)
    with pytest.raises(ValueError):
        LoadGenerator().create_syslog_message(**options)


# Scenario: A worker drains unsuccessfully or its flush operation raises.
# Guarantees: Status and stop retain failure details and outstanding record counts.
@pytest.mark.parametrize("flush_error", [None, RuntimeError("flush exploded")])
def test_flush_failure_visible(flush_error):
    generator, _, fake = sender()
    fake.flush_delivers = False
    fake.flush_error = flush_error
    fake.on_enqueue = generator.stop_event.set
    generator.start(config())
    await_exit(generator)
    snapshot, code = generator.stop()
    assert code == 500
    assert snapshot["status"] == "failed"
    assert snapshot["running"] is False
    assert snapshot["errors"]
    assert snapshot["metrics"]["kafka_pending"] == 1
    assert snapshot["metrics"]["kafka_pending_bytes"] == 1024
    assert snapshot["metrics"]["logs_produced"] == 0
    assert snapshot["metrics"]["kafka_flush_timeouts"] == int(flush_error is None)


# Scenario: Broker delivery fails during poll rather than during shutdown.
# Guarantees: The controller reports failure even when flush eventually drains.
def test_async_delivery_failure_visible():
    generator, _, fake = sender()
    fake.poll_delivers = True
    fake.delivery_error = "message timeout"
    generator.start(config())
    await_exit(generator)
    snapshot, code = generator.stop()
    assert code == 500
    assert snapshot["metrics"]["kafka_delivery_failed"] == 1
    assert snapshot["metrics"]["kafka_pending"] == 0
    assert snapshot["metrics"]["logs_produced"] == 0


# Scenario: produce raises a non-buffer exception while the controller is running.
# Guarantees: Future exceptions are observed and surface through status and stop.
def test_worker_exception_visible():
    generator, _, fake = sender()
    fake.enqueue_error = RuntimeError("produce exploded")
    generator.start(config())
    await_exit(generator)
    snapshot, code = generator.stop()
    assert code == 500
    assert "produce exploded" in snapshot["errors"][0]
    assert snapshot["metrics"]["kafka_enqueue_failed"] == 1
    assert snapshot["metrics"]["kafka_enqueued"] == 0


# Scenario: A nominal chunk contains one hour of traffic at the requested rate.
# Guarantees: Stop interrupts pacing immediately and drains accepted records.
def test_stop_interrupts_rate_wait():
    generator, _, fake = sender()
    generator.start(config(target_rate=1, threads=1, batch_size=3600))
    assert fake.enqueued.wait(1)
    started = time.monotonic()
    snapshot, code = generator.stop()
    assert time.monotonic() - started < 0.5
    assert code == 200
    assert snapshot["metrics"]["kafka_pending"] == 0
    assert len(fake.values) == 1


# Scenario: Sending the first chunk is much slower than its rate budget.
# Guarantees: Later chunks restart pacing instead of repaying accumulated catch-up debt.
def test_rate_lag_does_not_catch_up(monkeypatch):
    generator, worker, fake = sender(target_rate=20, batch_size=1)
    clock = [0.0]
    sends = []

    def enqueued():
        sends.append(clock[0])
        if len(sends) == 1:
            clock[0] += 1
        if len(sends) == 3:
            generator.stop_event.set()

    fake.on_enqueue = enqueued
    worker.producer_factory = lambda _: fake
    monkeypatch.setattr(kafka_syslog.time, "monotonic", lambda: clock[0])
    monkeypatch.setattr(
        worker, "_wait_until",
        lambda deadline: clock.__setitem__(0, max(clock[0], deadline)),
    )
    worker.run(0)
    assert sends == pytest.approx([0.0, 1.0, 1.05])
    assert generator.get_metrics()["late_batches"] == 1


# Scenario: Four workers share a requested aggregate rate of 80 records/second.
# Guarantees: Each worker is paced at 20 records/second, not the aggregate rate.
def test_rate_is_aggregate(monkeypatch):
    generator, worker, fake = sender(target_rate=80, threads=4, batch_size=1)
    clock = [0.0]
    deadlines = []

    def wait_until(deadline):
        deadlines.append(deadline)
        clock[0] = deadline

    fake.on_enqueue = generator.stop_event.set
    worker.producer_factory = lambda _: fake
    monkeypatch.setattr(kafka_syslog.time, "monotonic", lambda: clock[0])
    monkeypatch.setattr(worker, "_wait_until", wait_until)
    worker.run(2)
    assert deadlines == pytest.approx([2 / 80, 2 / 80 + 1 / 20])


# Scenario: Two clients start a run concurrently.
# Guarantees: Only one controller wins and neither start resets another run's counters.
def test_concurrent_start_is_serialized(monkeypatch):
    generator = LoadGenerator()
    monkeypatch.setattr(
        generator, "syslog_kafka_worker_thread",
        lambda *_: generator.stop_event.wait(),
    )
    barrier = threading.Barrier(3)
    codes = []

    def start():
        barrier.wait()
        codes.append(generator.start(config())[1])

    threads = [threading.Thread(target=start) for _ in range(2)]
    for thread in threads:
        thread.start()
    barrier.wait()
    for thread in threads:
        thread.join(timeout=1)
    try:
        assert sorted(codes) == [200, 400]
    finally:
        generator.stop()


# Scenario: A worker does not finish within the bounded stop join.
# Guarantees: Stop reports failure and retains the controller to block restarts.
def test_stop_timeout_preserves_controller(monkeypatch):
    generator = LoadGenerator()
    release = threading.Event()
    entered = threading.Event()

    def blocked(*_):
        entered.set()
        release.wait()

    monkeypatch.setattr(generator, "syslog_kafka_worker_thread", blocked)
    generator.start(config(kafka_flush_timeout=0.01))
    assert entered.wait(1)
    try:
        started = time.monotonic()
        snapshot, code = generator.stop()
        assert time.monotonic() - started < 1.5
        assert code == 500
        assert snapshot["running"] is True
        assert generator.start(config())[1] == 400
        assert generator.stop_event.is_set()
    finally:
        release.set()
        await_exit(generator)
    monkeypatch.setattr(
        generator, "syslog_kafka_worker_thread",
        lambda *_: generator.stop_event.wait(),
    )
    assert generator.start(config())[1] == 200
    assert generator.get_status()["errors"] == []
    assert generator.stop()[1] == 200


# Scenario: Clients submit Kafka settings that violate queue, header, or timeout rules.
# Guarantees: Pydantic and HTTP reject invalid settings before starting workers.
@pytest.mark.parametrize("overrides", [
    {"load_type": "otlp"}, {"syslog_format": "none"},
    {"message_size": 1}, {"message_size": 2048, "kafka_queue_max_kbytes": 1},
    {"kafka_brokers": " "}, {"kafka_topic": "bad topic"}, {"kafka_topic": ".."},
    {"kafka_acks": "0"}, {"kafka_queue_max_messages": 0},
    {"kafka_send_timeout": 0}, {"kafka_flush_timeout": -1},
    {"kafka_send_timeout": float("inf")}, {"kafka_message_timeout_ms": 0},
    {"kafka_request_timeout_ms": 0}, {"kafka_request_timeout_ms": 999},
    {"kafka_compression": "invalid"},
    {"kafka_linger_ms": 10000},
])
def test_invalid_kafka_config_and_api(overrides, monkeypatch):
    monkeypatch.setattr(module, "loadgen", LoadGenerator())
    with pytest.raises(ValidationError):
        config(**overrides)
    fields = config().model_dump()
    fields.update(overrides)
    response = module.app.test_client().post("/start", json=fields)
    assert response.status_code == 400
    assert "error" in response.get_json()
    assert module.loadgen.get_status()["status"] == "idle"


# Scenario: Flask controls a Kafka run that fails asynchronously.
# Guarantees: Start remains compatible; status, stop, and Prometheus expose the failure.
def test_kafka_http_lifecycle(monkeypatch):
    generator, _, fake = sender()
    fake.enqueue_error = RuntimeError("API worker error")
    monkeypatch.setattr(module, "loadgen", generator)
    client = module.app.test_client()
    readiness = client.get("/status")
    assert readiness.status_code == 200
    assert readiness.get_json()["status"] == "idle"
    response = client.post("/start", json=config().model_dump())
    assert response.status_code == 200
    assert response.get_json()["status"] == "started"
    await_exit(generator)
    response = client.get("/status")
    assert response.status_code == 500
    assert response.get_json()["status"] == "failed"
    assert response.get_json()["running"] is False
    assert client.post("/stop").status_code == 500
    metrics = client.get("/metrics").get_data(as_text=True)
    assert "logs_produced 0\n" in metrics
    assert "kafka_enqueue_failed 1\n" in metrics


# Scenario: HTTP hooks check readiness and health across all lifecycle states.
# Guarantees: Only failed status returns HTTP 500; snapshots always include metrics.
@pytest.mark.parametrize("status, expected_code", [
    ("idle", 200), ("running", 200), ("stopping", 200),
    ("stopped", 200), ("failed", 500),
])
def test_status_http_state(status, expected_code, monkeypatch):
    generator = LoadGenerator()
    generator.status = status
    monkeypatch.setattr(module, "loadgen", generator)
    response = module.app.test_client().get("/status")
    assert response.status_code == expected_code
    snapshot = response.get_json()
    assert snapshot["status"] == status
    assert "running" in snapshot
    assert "errors" in snapshot
    assert "logs_bytes_produced" in snapshot["metrics"]


# Scenario: Standalone Kafka mode encounters an asynchronous worker failure.
# Guarantees: CLI settings reach the producer and the process reports a nonzero exit.
def test_kafka_cli_failure(monkeypatch, capsys):
    generator, _, fake = sender()
    fake.enqueue_error = RuntimeError("CLI worker error")
    monkeypatch.setattr(module, "loadgen", generator)
    monkeypatch.setattr(sys, "argv", [
        "loadgen.py", "--load-type", "syslog", "--syslog-transport", "kafka",
        "--syslog-format", "rfc5424", "--kafka-brokers", "broker:9092",
        "--kafka-topic", "topic", "--kafka-flush-timeout", "0.05",
        "--duration", "10", "--threads", "1",
    ])
    with pytest.raises(SystemExit) as exc:
        module.main()
    assert exc.value.code == 1
    assert generator.current_config["kafka_brokers"] == "broker:9092"
    assert generator.current_config["kafka_topic"] == "topic"
    assert "CLI worker error" in capsys.readouterr().err


# Scenario: Default producer options are translated to actual librdkafka keys.
# Guarantees: Queue bounds and deadlines are finite; compression is off, acks is 1.
def test_producer_defaults():
    args = config().model_dump()
    assert args["message_size"] == 1024
    options = producer_config(args)
    assert options["queue.buffering.max.messages"] == 10000
    assert options["queue.buffering.max.kbytes"] == 16384
    assert options["acks"] == "1"
    assert options["compression.type"] == "none"
    assert options["linger.ms"] == 5
    assert options["message.timeout.ms"] == 10000
    assert options["request.timeout.ms"] == options["socket.timeout.ms"] == 5000
    assert LoadGenConfig().message_size is None
    assert LoadGenConfig().syslog_transport == "udp"
    assert LoadGenConfig().syslog_format == "rfc3164"
    assert LoadGenConfig(
        load_type="syslog", syslog_transport="kafka"
    ).syslog_format == "rfc5424"


# Scenario: Legacy TCP and UDP workers send explicit static Syslog bodies.
# Guarantees: TCP keeps newline-delimited batches; UDP keeps separate datagrams.
@pytest.mark.parametrize("transport", ["tcp", "udp"])
def test_existing_syslog_transports(transport, monkeypatch):
    generator = LoadGenerator()
    sock = MagicMock()
    monkeypatch.setattr(module.socket, "socket", lambda *_: sock)

    def sendall(_value):
        generator.stop_event.set()

    def sendto(value, _address):
        generator.stop_event.set()
        return len(value)

    sock.sendall.side_effect = sendall
    sock.sendto.side_effect = sendto
    args = LoadGenConfig(
        load_type="syslog", syslog_transport=transport, syslog_format="rfc5424",
        message_body="legacy", message_size=128, batch_size=2, threads=1,
    ).model_dump()
    getattr(generator, f"syslog_{transport}_worker_thread")(0, args)
    if transport == "tcp":
        value = sock.sendall.call_args.args[0]
        assert value.count(b"\n") == 2
        assert len(value) == 256
    else:
        assert sock.sendto.call_count == 2
        for call in sock.sendto.call_args_list:
            assert len(call.args[0]) == 128
            assert call.args[0].endswith(b"\n")
    assert generator.get_metrics()["logs_produced"] == 2
    assert generator.get_metrics()["bytes_sent"] == 256
    assert generator.get_metrics()["kafka_enqueued"] == 0


# Scenario: Existing stream/datagram benchmarks select a small mixed message pool.
# Guarantees: TCP/UDP preserve their mixed bodies and newline framing at that size.
def test_legacy_pool_keeps_small_messages():
    pool = _generate_message_pool(16, "rfc5424", message_size=128)
    assert len(pool) == 16
    assert all(len(record) == 128 and record.endswith(b"\n") for record in pool)
    assert any(b" CEF:" in record for record in pool)
    assert any(b" NOT-CEF:" in record for record in pool)


# Scenario: Legacy callers request a size smaller than the header or no header.
# Guarantees: TCP/UDP retain header-plus-newline fallback and case-insensitive none.
def test_legacy_message_edge_cases():
    generator = LoadGenerator()
    value = generator.create_syslog_message(
        hostname="host", header_type="rfc5424", message_body="body", message_size=1,
    )
    assert value.startswith(b"<134>1 ")
    assert value.endswith(b" host loadgen - - - \n")
    assert generator.create_syslog_message(
        hostname="unused", header_type="NONE", message_body="body",
    ) == b"body\n"
