"""Bounded raw Syslog records over Kafka, independent of OTLP exporters."""

import math
import socket
import threading
import time


class BalancedKafkaRouter:
    """Serialize only enqueue attempts; queue-full retries do not consume slots."""

    def __init__(self, topics, partitions_per_topic):
        self.targets = tuple(
            (topic, partition)
            for topic in topics
            for partition in range(partitions_per_topic)
        )
        self._next = 0
        self._lock = threading.Lock()

    def produce(self, producer, payload, callback_factory):
        with self._lock:
            topic, partition = self.targets[self._next]
            producer.produce(
                topic, partition=partition, value=payload,
                on_delivery=callback_factory(len(payload), (topic, partition)),
            )
            self._next = (self._next + 1) % len(self.targets)


def producer_config(args):
    """Keep librdkafka's in-memory queue and delivery lifetime finite."""
    return {
        "bootstrap.servers": args["kafka_brokers"],
        "queue.buffering.max.messages": args["kafka_queue_max_messages"],
        "queue.buffering.max.kbytes": args["kafka_queue_max_kbytes"],
        "message.timeout.ms": args["kafka_message_timeout_ms"],
        "request.timeout.ms": args["kafka_request_timeout_ms"],
        "socket.timeout.ms": args["kafka_request_timeout_ms"],
        "socket.connection.setup.timeout.ms": args["kafka_request_timeout_ms"],
        "linger.ms": args["kafka_linger_ms"],
        "acks": args["kafka_acks"],
        "compression.type": args["kafka_compression"],
        # Delivery callbacks below report failures once, without native log spam.
        "log_level": 0,
    }


class KafkaSyslogWorker:
    """One producer per worker; only poll/flush dispatch delivery callbacks."""

    def __init__(self, loadgen, args, producer_factory=None):
        if producer_factory is None:
            from confluent_kafka import Producer

            producer_factory = Producer
        self.loadgen = loadgen
        self.args = args
        self.producer_factory = producer_factory
        self.producer = None

    def _delivered(self, size, target=None):
        def callback(error, _message):
            self.loadgen.record_kafka_delivery(size, error, target)
        return callback

    def send(self, payload):
        """Retry queue pressure, not records already accepted by librdkafka."""
        deadline = time.monotonic() + self.args["kafka_send_timeout"]
        while not self.loadgen.stop_event.is_set():
            try:
                if self.loadgen.kafka_router is None:
                    self.producer.produce(
                        self.args["kafka_topic"],
                        value=payload,
                        on_delivery=self._delivered(len(payload)),
                    )
                else:
                    self.loadgen.kafka_router.produce(
                        self.producer, payload, self._delivered
                    )
            except BufferError:
                self.loadgen.increment_metric("kafka_queue_full")
                self.producer.poll(0)
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    self.loadgen.update_metrics(failed=1, kafka_enqueue_failed=1)
                    raise TimeoutError("Kafka queue remained full until send deadline")
                self.loadgen.stop_event.wait(min(0.01, remaining))
                continue
            except Exception:
                self.loadgen.update_metrics(failed=1, kafka_enqueue_failed=1)
                raise
            self.loadgen.update_metrics(
                kafka_enqueued=1, kafka_pending=1, kafka_pending_bytes=len(payload)
            )
            return True
        return False

    def _wait_until(self, deadline):
        while not self.loadgen.stop_event.is_set():
            self.producer.poll(0)
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                return
            self.loadgen.stop_event.wait(min(0.05, remaining))

    def run(self, thread_id):
        args = self.args
        size = args["message_size"]
        # Reuse the existing generator, not its mixed CEF/non-CEF TCP/UDP pool.
        # Bound fixture memory independently of the scheduling batch size.
        pool = []
        for _ in range(min(256, max(1, 1024 * 1024 // size))):
            if self.loadgen.stop_event.is_set():
                return
            pool.append(self.loadgen.create_syslog_message(
                hostname=socket.gethostname(),
                message_body=args["message_body"],
                body_size=min(args["body_size"], size),
                header_type=args["syslog_format"],
                syslog_content_type=args["syslog_content_type"],
                message_size=size,
                line_delimited=False,
            ))
        if self.loadgen.stop_event.is_set():
            return
        self.producer = self.producer_factory(producer_config(args))
        rate = args["target_rate"]
        thread_rate = rate / args["threads"] if rate else None
        chunk = args["batch_size"]
        if thread_rate:
            # Large batches must not become multi-second startup bursts.
            chunk = min(chunk, max(1, math.ceil(thread_rate * 0.05)))
        index = 0
        try:
            if rate:
                self._wait_until(time.monotonic() + thread_id / rate)
            while not self.loadgen.stop_event.is_set():
                started = time.monotonic()
                for offset in range(chunk):
                    if not self.send(pool[index % len(pool)]):
                        break
                    index += 1
                    if offset % 64 == 0:
                        self.producer.poll(0)
                self.producer.poll(0)
                if thread_rate:
                    deadline = started + chunk / thread_rate
                    if time.monotonic() > deadline:
                        self.loadgen.increment_metric("late_batches")
                    # Each chunk starts a new schedule: no accumulated catch-up debt.
                    self._wait_until(deadline)
        finally:
            pending = self.producer.flush(args["kafka_flush_timeout"])
            if pending:
                self.loadgen.increment_metric("kafka_flush_timeouts")
                self.loadgen.record_error(
                    f"Kafka flush deadline expired with {pending} pending records"
                )
