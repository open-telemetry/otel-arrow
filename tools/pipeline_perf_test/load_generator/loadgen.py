"""
OTLP Logs Load Generator

This module implements a configurable load generator for OpenTelemetry
Protocol (OTLP) logs. It supports generating batches of randomized log records
with customizable sizes and attributes, and sending them concurrently to an
OTLP collector endpoint over gRPC.

Features:
- Generates OTLP log records with random content for testing or benchmarking.
- Runs multiple worker threads to simulate concurrent load.
- Supports shared or dedicated TCP connection per-worker thread.
- Supports optional rate targeting for message throughput (or max achievable).
- Provides a Flask-based HTTP API to start, stop, and monitor the load
    generator.
- Can run either as a one-off command line tool or as a long-running server.
- Handles graceful shutdown on system signals.

Usage:
- As a command-line tool: run with desired parameters for batch size, threads,
    duration, etc.
- As a server: start with --serve flag and control load generation via HTTP
    endpoints (/start, /stop, /metrics).

Endpoints:
- POST /start: Start load generation with specified parameters in JSON.
- POST /stop: Stop the load generation.
- GET /metrics: Retrieve current load generation metrics (logs sent, failed,
    bytes sent).

Environment Variables:
- OTLP_ENDPOINT: Target OTLP gRPC endpoint (default: localhost:4317).
- SYSLOG_SERVER: Target syslog server hostname/IP (default: localhost).
- SYSLOG_PORT: Target syslog server port (default: 514).
"""

import argparse
import concurrent.futures
import os
import random
import signal
import socket
import string
import sys
import threading
import time
from datetime import datetime as dt, timezone
from typing import Optional

import grpc  # type: ignore
from flask import Flask, jsonify, request
from opentelemetry.proto.collector.logs.v1 import (
    logs_service_pb2_grpc,
    logs_service_pb2,
)
from opentelemetry.proto.logs.v1 import logs_pb2
from opentelemetry.proto.common.v1 import common_pb2
from pydantic import BaseModel, Field, field_validator, ValidationError


FLASK_PORT = 5001
LOG_SEVERITY_NUMBER = logs_pb2.SeverityNumber.SEVERITY_NUMBER_INFO
LOG_SEVERITY_TEXT = "INFO"


app = Flask(__name__)


class LoadGenConfig(BaseModel):
    body_size: int = Field(
        25, gt=0, description="Size of log message body in characters"
    )
    num_attributes: int = Field(2, gt=0, description="Number of attributes per log")
    attribute_value_size: int = Field(
        15, gt=0, description="Size of attribute values in characters"
    )
    batch_size: int = Field(5000, gt=0, description="Number of logs per batch")
    threads: int = Field(4, gt=0, description="Number of worker threads to run")
    target_rate: Optional[int] = Field(
        None, gt=0, description="Optional target messages per second"
    )
    tcp_connection_per_thread: bool = Field(
        True, description="Use a dedicated TCP connection per-thread"
    )
    load_type: str = Field(
        "otlp", description="Load generation type: 'otlp' or 'syslog'"
    )

    @field_validator(
        "body_size", "num_attributes", "attribute_value_size", "batch_size", "threads"
    )
    def must_be_positive(cls, v):
        """Ensure positive values for key config attributes."""
        if v <= 0:
            raise ValueError("must be a positive integer")
        return v

    @field_validator("load_type")
    def validate_load_type(cls, v):
        """Ensure load_type is either 'otlp' or 'syslog'."""
        if v.lower() not in ["otlp", "syslog"]:
            raise ValueError("load_type must be 'otlp' or 'syslog'")
        return v.lower()


class LoadGenerator:
    def __init__(self):
        self.controller_thread = None
        self.stop_event = threading.Event()
        self.current_config = {}
        self.lock = threading.Lock()
        self.metrics = {"sent": 0, "failed": 0, "bytes_sent": 0, "late_batches": 0}

    def generate_random_string(self, length: int) -> str:
        """
        Generate a random alphanumeric string of the specified length.
        """
        return "".join(
            random.choice(string.ascii_letters + string.digits) for _ in range(length)
        )

    def create_log_record(
        self,
        body_size: int = 25,
        num_attributes: int = 2,
        attribute_value_size: int = 15,
    ):
        """
        Create a single OTLP log record with random content.
        """
        log_message = self.generate_random_string(body_size)
        attributes = [
            common_pb2.KeyValue(
                key=f"attribute.{i+1}",
                value=common_pb2.AnyValue(
                    string_value=self.generate_random_string(attribute_value_size)
                ),
            )
            for i in range(num_attributes)
        ]
        return logs_pb2.LogRecord(
            time_unix_nano=int(time.time_ns()),
            severity_text=LOG_SEVERITY_TEXT,
            severity_number=LOG_SEVERITY_NUMBER,
            body=common_pb2.AnyValue(string_value=log_message),
            attributes=attributes,
        )

    def increment_metric(self, key: str, amount: int = 1) -> None:
        with self.lock:
            self.metrics[key] += amount

    def worker_thread(self, thread_id: int, args: dict) -> None:
        """
        Worker thread that sends batches of log records to an OTLP endpoint.
        """
        endpoint = os.getenv("OTLP_ENDPOINT", "localhost:4317")

        channel = None
        if args.get("tcp_connection_per_thread"):
            # This disables the default python grpc client behavior of shared global
            # subchannels per destination.
            channel = grpc.insecure_channel(
                endpoint, options=[("grpc.use_local_subchannel_pool", 1)]
            )
        else:
            channel = grpc.insecure_channel(endpoint)

        stub = logs_service_pb2_grpc.LogsServiceStub(channel)

        batch_size = args["batch_size"]
        thread_count = args["threads"]
        target_rate = args.get("target_rate")

        if target_rate:
            thread_rate = target_rate / thread_count
            batch_interval = batch_size / thread_rate
            print(
                f"Thread {thread_id} started with rate limit: {thread_rate} "
                f"logs/sec (interval: {batch_interval:.4f}s)"
            )
        else:
            batch_interval = None
            print(f"Thread {thread_id} started with no rate limit")

        log_batch = [
            self.create_log_record(
                body_size=args["body_size"],
                num_attributes=args["num_attributes"],
                attribute_value_size=args["attribute_value_size"],
            )
            for _ in range(batch_size)
        ]

        scope_logs = logs_pb2.ScopeLogs(log_records=log_batch)
        resource_logs = logs_pb2.ResourceLogs(scope_logs=[scope_logs])
        logs_request = logs_service_pb2.ExportLogsServiceRequest(
            resource_logs=[resource_logs]
        )

        next_send_time = time.perf_counter()
        while not self.stop_event.is_set():
            try:
                stub.Export(logs_request)
                self.increment_metric("sent", args["batch_size"])
                self.increment_metric("bytes_sent", logs_request.ByteSize())
            except Exception as e:
                print(f"Thread {thread_id}: Failed to send log batch: {e}")
                self.increment_metric("failed", args["batch_size"])

            # If we're targeting a specific rate we do additional calculations
            # to ensure we're not exceeding it via sleep. If we're not reaching
            # the target rate (e.g. we're sending without sleep and it's
            # still too slow), we increment a metric to inform observers.
            if batch_interval:
                now = time.perf_counter()
                sleep_time = next_send_time - now
                if sleep_time > 0:
                    time.sleep(sleep_time)
                elif now - next_send_time > batch_interval:
                    # More than 1 interval behind
                    self.increment_metric("late_batches")
                next_send_time += batch_interval

    def syslog_worker_thread(self, thread_id: int, args: dict) -> None:
        """
        Worker thread that sends syslog messages to a syslog server.
        """
        syslog_server = os.getenv("SYSLOG_SERVER", "localhost")
        syslog_port = int(os.getenv("SYSLOG_PORT", "514"))

        # Create TCP socket for syslog
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)
        sock.connect((syslog_server, syslog_port))

        batch_size = args["batch_size"]
        thread_count = args["threads"]
        target_rate = args.get("target_rate")

        if target_rate:
            thread_rate = target_rate / thread_count
            batch_interval = batch_size / thread_rate
            print(
                f"Thread {thread_id} started with rate limit: {thread_rate} "
                f"logs/sec (interval: {batch_interval:.4f}s)"
            )
        else:
            batch_interval = None
            print(f"Thread {thread_id} started with no rate limit")

        # Pre-generate syslog messages batch (similar to OTLP log_batch)
        syslog_batch = []
        for _ in range(batch_size):
            syslog_message = self.create_syslog_message(
                body_size=args["body_size"],
                num_attributes=args["num_attributes"],
                attribute_value_size=args["attribute_value_size"],
            )
            syslog_batch.append(syslog_message)

        # Combine all messages into a single buffer for efficient sending
        batch_buffer = b''.join(syslog_batch)
        batch_total_size = len(batch_buffer)

        next_send_time = time.perf_counter()
        while not self.stop_event.is_set():
            try:
                sock.sendall(batch_buffer)
                self.increment_metric("sent", args["batch_size"])
                self.increment_metric("bytes_sent", batch_total_size)
            except Exception as e:
                print(f"Thread {thread_id}: Failed to send syslog batch: {e}")
                self.increment_metric("failed", args["batch_size"])
                # Try to reconnect
                try:
                    sock.close()
                    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                    sock.settimeout(5)
                    sock.connect((syslog_server, syslog_port))
                except Exception as reconnect_error:
                    print(f"Thread {thread_id}: Reconnection failed: {reconnect_error}")
                    break

            # If we're targeting a specific rate we do additional calculations
            # to ensure we're not exceeding it via sleep. If we're not reaching
            # the target rate (e.g. we're sending without sleep and it's
            # still too slow), we increment a metric to inform observers.
            if batch_interval:
                now = time.perf_counter()
                sleep_time = next_send_time - now
                if sleep_time > 0:
                    time.sleep(sleep_time)
                elif now - next_send_time > batch_interval:
                    # More than 1 interval behind
                    self.increment_metric("late_batches")
                next_send_time += batch_interval
        
        sock.close()

    def create_syslog_message(
        self,
        body_size: int = 25,
        num_attributes: int = 2,
        attribute_value_size: int = 15,
    ) -> bytes:
        """
        Create a single syslog message with structure similar to OTLP log record.
        """
        hostname = socket.gethostname()
        
        # Pre-generate static parts of the message
        pri = "<134>"  # local0.info = 16*8+6 = 134
        tag = "loadgen"
        
        # Generate timestamp (RFC3164 format with space-padded day)
        utc_time = dt.now(timezone.utc)
        day = utc_time.day
        timestamp = utc_time.strftime(f"%b {day:2d} %H:%M:%S")
        
        # Create log message body (similar to OTLP body)
        log_message = self.generate_random_string(body_size)
        
        # Create attributes (similar to OTLP attributes)
        attributes = []
        for i in range(num_attributes):
            attr_value = self.generate_random_string(attribute_value_size)
            attributes.append(f"attr{i+1}={attr_value}")
        
        # Combine everything into syslog format
        attributes_str = " ".join(attributes) if attributes else ""
        message_content = f"{log_message} {attributes_str}".strip()
        
        syslog_message = f"{pri}{timestamp} {hostname} {tag}: {message_content}\n"
        return syslog_message.encode('utf-8')

    def run_loadgen(self, args_dict):
        """
        Start the load generation process by launching multiple worker threads.
        Chooses between OTLP and syslog workers based on configuration.
        """
        with self.lock:
            self.metrics.update({"sent": 0, "failed": 0, "bytes_sent": 0})

        # Determine which worker thread to use based on configuration
        load_type = args_dict.get("load_type", "otlp").lower()
        
        if load_type == "syslog":
            worker_func = self.syslog_worker_thread
            syslog_server = os.getenv("SYSLOG_SERVER", "localhost")
            syslog_port = os.getenv("SYSLOG_PORT", "514")
            print(f"Using syslog worker, target: {syslog_server}:{syslog_port}")
        else:
            worker_func = self.worker_thread
            endpoint = os.getenv("OTLP_ENDPOINT", "localhost:4317")
            print(f"Using OTLP worker, target: {endpoint}")

        with concurrent.futures.ThreadPoolExecutor(
            max_workers=args_dict.get("threads", 4)
        ) as executor:
            futures = [
                executor.submit(worker_func, i, args_dict)
                for i in range(args_dict.get("threads", 4))
            ]
            concurrent.futures.wait(futures)

        with self.lock:
            self.current_config["metrics"] = self.metrics.copy()

    def start(self, config: LoadGenConfig):
        """
        Start the load generator with the specified configuration.
        Returns a tuple of (response_dict, status_code).
        """

        if self.controller_thread and self.controller_thread.is_alive():
            return {"error": "Load generation already running"}, 400

        self.stop_event.clear()
        with self.lock:
            self.current_config.update(config)
            self.current_config["running"] = True
            self.current_config["metrics"] = {}

        self.controller_thread = threading.Thread(
            target=self.run_loadgen, args=(config.model_dump(),)
        )
        self.controller_thread.start()

        return {"status": "started"}, 200

    def stop(self):
        """
        Stop the currently running load generator.
        Returns a tuple of (response_dict, status_code).
        """
        self.stop_event.set()
        if self.controller_thread:
            self.controller_thread.join(timeout=10)
            self.controller_thread = None
        with self.lock:
            self.current_config["running"] = False
        return {"status": "stopped"}, 200

    def get_metrics(self):
        """
        Get a copy of the current metrics.
        """
        with self.lock:
            return self.metrics.copy()


# Create a global LoadGenerator instance for the Flask app to use
loadgen = LoadGenerator()


@app.route("/start", methods=["POST"])
def start():
    try:
        config_data = request.get_json()
        config = LoadGenConfig(**config_data)
    except ValidationError as e:
        return jsonify({"error": e.errors()}), 400
    except Exception:
        return jsonify({"error": "Invalid JSON or missing data"}), 400

    resp, code = loadgen.start(config)
    return jsonify(resp), code


@app.route("/stop", methods=["POST"])
def stop():
    resp, code = loadgen.stop()
    return jsonify(resp), code


@app.route("/metrics", methods=["GET"])
def metrics_endpoint():
    metrics = loadgen.get_metrics()
    lines = [f"{k} {v}" for k, v in metrics.items()]
    return "\n".join(lines), 200


def handle_signal(sig, frame):
    print(f"\nReceived signal {sig}, shutting down gracefully...")
    loadgen.stop()
    sys.exit(0)


def is_port_in_use(port, host="0.0.0.0"):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.settimeout(1)
        return s.connect_ex((host, port)) == 0

def main():
    def get_default_value(field_name: str):
        return LoadGenConfig.model_fields[field_name].default

    parser = argparse.ArgumentParser(description="Loadgen for OTLP logs")
    parser.add_argument(
        "--serve", action="store_true", help="Start the server (default: False)"
    )
    parser.add_argument(
        "--serve-port",
        type=int,
        default=FLASK_PORT,
        help=f"Server port to listen on (default {FLASK_PORT})",
    )
    parser.add_argument(
        "--duration", type=int, default=15, help="Duration in seconds (default: 15)"
    )
    parser.add_argument(
        "--body-size",
        type=int,
        default=get_default_value("body_size"),
        help=(
            "Size of log message body in characters "
            f"(default {get_default_value('body_size')})"
        ),
    )
    parser.add_argument(
        "--num-attributes",
        type=int,
        default=get_default_value("num_attributes"),
        help=(
            "Number of attributes per log "
            f"(default {get_default_value('num_attributes')})"
        ),
    )
    parser.add_argument(
        "--attribute-value-size",
        type=int,
        default=get_default_value("attribute_value_size"),
        help=(
            "Size of attribute values in characters "
            f"(default {get_default_value('attribute_value_size')})"
        ),
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=get_default_value("batch_size"),
        help=f"Number of logs per batch (default {get_default_value('batch_size')})",
    )
    parser.add_argument(
        "--threads",
        type=int,
        default=get_default_value("threads"),
        help=f"Number of worker threads (default {get_default_value('threads')})",
    )
    parser.add_argument(
        "--target-rate",
        type=int,
        default=get_default_value("target_rate"),
        help=(
            "Optional message rate to target "
            f"(default {get_default_value('target_rate')})"
        ),
    )
    parser.add_argument(
        "--tcp-connection-per-thread",
        type=bool,
        default=get_default_value("tcp_connection_per_thread"),
        help=(
            "Use a dedicated TCP connection per-thread (default "
            f"{get_default_value("tcp_connection_per_thread")})"
        ),
    )
    args = parser.parse_args()

    if args.serve:
        if is_port_in_use(FLASK_PORT):
            raise RuntimeError(f"Port {FLASK_PORT} is already in use.")
        app.run(host="0.0.0.0", port=args.serve_port)
        return

    print("Starting load generator with configuration:")
    print(f"- Duration: {args.duration} seconds")
    print(f"- Batch size: {args.batch_size} logs")
    print(f"- Threads: {args.threads}")
    print(f"- Target Rate: {args.target_rate}")
    print(f"- Log body size: {args.body_size} characters")
    print(f"- Attributes per log: {args.num_attributes}")
    print(f"- Attribute value size: {args.attribute_value_size} characters")

    config = LoadGenConfig(
        body_size=args.body_size,
        num_attributes=args.num_attributes,
        attribute_value_size=args.attribute_value_size,
        batch_size=args.batch_size,
        threads=args.threads,
        target_rate=args.target_rate,
    )

    loadgen.start(config=config)

    try:
        time.sleep(args.duration)
    except KeyboardInterrupt:
        print("Interrupted by user, stopping early...")

    loadgen.stop()

    print(f'LOADGEN_LOGS_SENT: {loadgen.metrics.get("sent", 0)}')
    print(f'LOADGEN_LOGS_FAILED: {loadgen.metrics.get("failed", 0)}')
    print(f'LOADGEN_BYTES_SENT: {loadgen.metrics.get("bytes_sent", 0)} bytes')


if __name__ == "__main__":
    signal.signal(signal.SIGINT, handle_signal)
    signal.signal(signal.SIGTERM, handle_signal)
    main()
