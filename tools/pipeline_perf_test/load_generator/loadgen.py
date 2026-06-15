"""
OTLP & Syslog Logs Load Generator

This module implements a configurable load generator for OpenTelemetry
Protocol (OTLP) logs and syslog messages. It supports generating batches of
randomized log records or CEF-formatted syslog messages with customizable
sizes and attributes, and sending them concurrently to an OTLP collector
endpoint over gRPC or to a syslog server over TCP/UDP.

Features:
- Generates OTLP log records with random content for testing or benchmarking.
- Generates syslog messages (RFC 3164/5424) with random or CEF payloads.
- Configurable syslog server, port, transport (TCP/UDP), and message format.
- Supports target message size with automatic padding/truncation.
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

Examples:
  Standalone OTLP load generation:
    python load_generator/loadgen.py --load-type otlp --duration 30 --threads 4 --batch-size 1000

  Standalone syslog UDP load generation:
    python load_generator/loadgen.py --load-type syslog --syslog-server 0.0.0.0 --syslog-port 5140 --duration 2

  Standalone syslog TCP load generation:
    python load_generator/loadgen.py --load-type syslog --syslog-server 0.0.0.0 --syslog-port 514 --syslog-transport tcp --duration 2

  Standalone syslog CEF load generation:
    python load_generator/loadgen.py --load-type syslog --syslog-content-type cef --syslog-server 0.0.0.0 --syslog-port 5140 --duration 2

  Server mode for API control:
    python load_generator/loadgen.py --serve
    # Then control via HTTP:
    # curl -X POST http://localhost:5001/start -H "Content-Type: application/json" -d '{"load_type": "syslog", "batch_size": 1000, "threads": 2, "syslog_server": "0.0.0.0", "syslog_port": 5140}'
    # curl -X POST http://localhost:5001/stop
    # curl http://localhost:5001/metrics

Endpoints:
- POST /start: Start load generation with specified parameters in JSON.
- POST /stop: Stop the load generation.
- GET /metrics: Retrieve current load generation metrics (logs sent, failed,
    bytes sent).

Environment Variables:
- OTLP_ENDPOINT: Target OTLP gRPC endpoint (default: localhost:4317).
- SYSLOG_SERVER: Fallback syslog server if --syslog-server not set (default: localhost).
- SYSLOG_PORT: Fallback syslog port if --syslog-port not set (default: 514).
- SYSLOG_TRANSPORT: Fallback transport if --syslog-transport not set (default: udp).
- SYSLOG_FORMAT: Fallback syslog header format if --syslog-format not set (default: rfc3164).
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

# CEF (Common Event Format) template for realistic syslog load generation
CEF_TEMPLATE = (
    'CEF:0|PaloAltoNetworks|PAN-OS|9.1.8|SSH2 Login Attempt(31914)'
    '|SSH2 Login Attempt(31914)|1|act=alert '
    'actionflags=0x2000000000000000 app=ssh cat=any cn1=67640598 '
    'cn2=1207111110 cnt=1 cs1=THREAT cs2=vulnerability cs3=Tap_Allow '
    'cs5= cs6= destinationTranslatedAddress=0.0.0.0 '
    'destinationTranslatedPort=0 deviceExternalId=0120010106097 '
    'deviceInboundInterface=ethernet1/3 '
    'deviceOutboundInterface=ethernet1/3 dntdom=Tap domeid=vsys1 '
    'dpt=22 dst=172.21.166.15 dstloc=172.16.0.0-172.31.255.255 '
    'duid= duser= dvchost=PA-820 dvcpid=12337 '
    'end=Jun 23 2021 20:36:07 GMT fileHash= fileId=0 filePath= '
    'fileType= flags=0x80002000 fname= '
    'logset=InfoCIC-LogForwarding msg=informational '
    'outcome=client-to-server proto=tcp request="" '
    'requestClientApplication= requestContext= requestMethod= '
    'rt=Jun 23 2021 20:36:07 GMT sntdom=Tap '
    'sourceTranslatedAddress=0.0.0.0 sourceTranslatedPort=0 '
    'spt=44840 src=172.21.76.92 '
    'srcloc=172.16.0.0-172.31.255.255 suid= suser= '
    'PanOSThreatCategory=brute-force PanOSParentSessionID=0 '
    'PanOSParentStartTime= PanOSContentVer=AppThreat-8348-6427 '
    'PanOSTunnelID=0 PanOSTunnelType=N/A'
)

# Pool of syslog message templates with placeholder fields for entropy.
# Each template represents a different type of network/security event.
# Placeholders: {src_ip}, {dst_ip}, {src_port}, {dst_port}, {session_id},
#               {user_id}, {hash}, {count}, {severity}
_SYSLOG_MESSAGE_TEMPLATES = [
    'CEF:0|PaloAltoNetworks|PAN-OS|9.1.8|SSH2 Login Attempt(31914)|SSH2 Login Attempt|{severity}|act=alert app=ssh src={src_ip} dst={dst_ip} spt={src_port} dpt=22 proto=TCP cnt={count} cs1=THREAT cs2=vulnerability dvchost=FW-{user_id} deviceExternalId={session_id}',
    'CEF:0|Fortinet|FortiGate|7.2.3|0419016384|ssl-login-fail|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt=443 proto=TCP act=blocked user=admin session_id={session_id} reason=invalid_cert cn1={count}',
    'CEF:0|CrowdStrike|Falcon|6.42|DetectionSummaryEvent|Malware Detected|{severity}|src={src_ip} dst={dst_ip} fname=/tmp/payload_{hash}.bin fileHash={hash} act=quarantine cs1=ML_Detection uid={user_id} cnt={count}',
    'CEF:0|Cisco|ASA|9.16|106023|ACL Deny|{severity}|src={src_ip} spt={src_port} dst={dst_ip} dpt={dst_port} proto=TCP act=deny cs1=outside_access_in cn1={count} reason=acl-drop session={session_id}',
    'CEF:0|Zscaler|NSS|6.1|WebFilter|URL Blocked|{severity}|src={src_ip} dst={dst_ip} dpt=443 request=https://malware.example.com/{hash} act=blocked cs1=Malware cs2=cloud_app uid={user_id} cnt={count}',
    'NOT-CEF:0|Security|threatmanager|1.0|{count}|connection anomaly|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt={dst_port} proto=TCP act=alert cs1Label=ThreatType cs1=Anomaly cs2Label=SessionID cs2={session_id} uid={user_id}',
    'NOT-CEF:0|Firewall|fw-cluster|2.3|{count}|rate limit exceeded|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt={dst_port} proto=UDP act=throttle cs1Label=RuleID cs1=RL-{hash} cs2Label=BurstCount cs2={count}',
    'NOT-CEF:0|IDS|suricata|6.0|{count}|ET SCAN potential scan|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt={dst_port} proto=TCP act=alert cs1Label=Signature cs1=ET-{hash} cs2Label=Category cs2=attempted-recon',
    'NOT-CEF:0|Proxy|squid|5.7|{count}|CONNECT tunnel|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt={dst_port} proto=TCP act=allow cs1Label=Method cs1=CONNECT cs2Label=URL cs2=api.service-{user_id}.internal:{dst_port}',
    'NOT-CEF:0|LoadBalancer|haproxy|2.8|{count}|backend timeout|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt={dst_port} proto=TCP act=503 cs1Label=Backend cs1=app-pool-{hash} cs2Label=Duration cs2={count}ms',
    'NOT-CEF:0|WAF|modsecurity|3.0|{count}|SQL injection attempt|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt=443 proto=TCP act=deny cs1Label=RuleID cs1=942100 cs2Label=Payload cs2=OR 1=1--{hash}',
    'NOT-CEF:0|DNS|unbound|1.17|{count}|NXDOMAIN response|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt=53 proto=UDP act=nxdomain cs1Label=Query cs1={hash}.malware-c2.net cs2Label=Type cs2=A',
    'NOT-CEF:0|VPN|wireguard|1.0|{count}|handshake complete|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt=51820 proto=UDP act=allow cs1Label=PeerID cs1=peer-{user_id} cs2Label=Interface cs2=wg0',
    'NOT-CEF:0|Auth|sshd|8.9|{count}|publickey accepted|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt=22 proto=TCP act=accept cs1Label=User cs1=deploy-{user_id} cs2Label=KeyFingerprint cs2=SHA256:{hash}',
    'NOT-CEF:0|Mail|postfix|3.7|{count}|message delivered|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt=25 proto=TCP act=sent cs1Label=QueueID cs1={session_id} cs2Label=Recipient cs2=user-{user_id}@corp.example.com',
    'NOT-CEF:0|Storage|minio|2023|{count}|object uploaded|{severity}|src={src_ip} dst={dst_ip} spt={src_port} dpt=9000 proto=TCP act=PUT cs1Label=Bucket cs1=data-lake-{user_id} cs2Label=Key cs2=exports/{hash}.parquet',
]

# Hostnames to rotate across pool messages for resource-level diversity
_HOSTNAME_POOL = [
    'web-prod-01', 'web-prod-02', 'web-prod-03', 'api-prod-01',
    'api-prod-02', 'db-prod-01', 'cache-prod-01', 'worker-01',
    'worker-02', 'gateway-01', 'monitor-01', 'batch-proc-01',
]

# Severity/facility combinations (PRI values) for syslog diversity
_PRI_POOL = [
    '<134>',  # local0.info
    '<131>',  # local0.err
    '<132>',  # local0.crit
    '<133>',  # local0.warning
    '<142>',  # local1.info
    '<139>',  # local1.err
    '<150>',  # local2.info
    '<147>',  # local2.err
    '<86>',   # auth.info
    '<84>',   # auth.warning
]


def _generate_message_pool(
    pool_size: int,
    header_type: str,
    message_size: Optional[int] = None,
) -> list[bytes]:
    """Pre-generate a pool of diverse syslog messages with randomized fields."""
    pool = []
    for i in range(pool_size):
        template = _SYSLOG_MESSAGE_TEMPLATES[i % len(_SYSLOG_MESSAGE_TEMPLATES)]
        hostname = _HOSTNAME_POOL[i % len(_HOSTNAME_POOL)]
        pri = _PRI_POOL[i % len(_PRI_POOL)]

        # Randomize field values for each pool entry
        msg = template.format(
            src_ip=f"{random.randint(10,172)}.{random.randint(0,255)}.{random.randint(0,255)}.{random.randint(1,254)}",
            dst_ip=f"{random.randint(10,172)}.{random.randint(0,255)}.{random.randint(0,255)}.{random.randint(1,254)}",
            src_port=random.randint(1024, 65535),
            dst_port=random.randint(80, 9999),
            session_id=f"sess-{random.randint(100000, 999999):06d}",
            user_id=f"u{random.randint(1000, 9999)}",
            hash=f"{random.getrandbits(64):016x}",
            count=random.randint(1, 5000),
            severity=random.randint(1, 10),
        )

        # Build syslog header
        tag = "loadgen"
        if header_type == "rfc3164":
            utc_time = dt.now(timezone.utc)
            day = utc_time.day
            timestamp = utc_time.strftime(f"%b {day:2d} %H:%M:%S")
            header = f"{pri}{timestamp} {hostname} {tag}: "
        elif header_type == "rfc5424":
            timestamp = dt.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
            header = f"{pri}1 {timestamp} {hostname} {tag} - - - "
        elif header_type.lower() == "none":
            header = ""
        else:
            raise ValueError(f"Unknown header_type: {header_type!r}")

        syslog_message = f"{header}{msg}\n"

        # Pad or truncate to target message_size if specified
        if message_size is not None:
            current_size = len(syslog_message.encode("utf-8"))
            if current_size < message_size:
                padding_needed = message_size - current_size
                syslog_message = f"{header}{msg}{' ' * padding_needed}\n"
            elif current_size > message_size:
                header_bytes = len(header.encode("utf-8"))
                available = message_size - header_bytes - 1
                if available > 0:
                    syslog_message = f"{header}{msg[:available]}\n"
                else:
                    syslog_message = f"{header}\n"

        pool.append(syslog_message.encode("utf-8"))

    return pool


app = Flask(__name__)


class LoadGenConfig(BaseModel):
    body_size: int = Field(
        25, gt=0, description="Size of log message body in characters"
    )
    message_body: Optional[str] = Field(
        None, description="Optional static message body content (overrides body_size)"
    )
    num_attributes: int = Field(2, gt=0, description="Number of attributes per log")
    attribute_value_size: int = Field(
        15, gt=0, description="Size of attribute values in characters"
    )
    batch_size: int = Field(5000, gt=0, description="Number of logs per batch")
    threads: int = Field(4, gt=0, description="Number of worker threads to run")
    target_rate: Optional[int] = Field(
        None, ge=0, description="Optional target messages per second (0 = no limit)"
    )
    tcp_connection_per_thread: bool = Field(
        True, description="Use a dedicated TCP connection per-thread"
    )
    load_type: str = Field(
        "otlp", description="Load generation type: 'otlp' or 'syslog'"
    )
    syslog_server: str = Field(
        default_factory=lambda: os.getenv("SYSLOG_SERVER", "localhost"),
        description="Syslog server address",
    )
    syslog_port: int = Field(
        default_factory=lambda: int(os.getenv("SYSLOG_PORT", "514")),
        gt=0, le=65535, description="Syslog server port",
    )
    syslog_transport: str = Field(
        default_factory=lambda: os.getenv("SYSLOG_TRANSPORT", "udp"),
        description="Syslog transport protocol: 'tcp' or 'udp'",
    )
    syslog_format: str = Field(
        "rfc3164",
        description="Syslog header format: 'rfc3164', 'rfc5424', or 'none'",
    )
    syslog_content_type: str = Field(
        "random", description="Syslog message content type: 'random' or 'cef'"
    )
    message_size: Optional[int] = Field(
        None, gt=0, description="Target total message size in bytes (pads to fit)"
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

    @field_validator("syslog_transport")
    def validate_syslog_transport(cls, v):
        """Ensure syslog_transport is either 'tcp' or 'udp'."""
        if v.lower() not in ["tcp", "udp"]:
            raise ValueError("syslog_transport must be 'tcp' or 'udp'")
        return v.lower()

    @field_validator("syslog_format")
    def validate_syslog_format(cls, v):
        """Ensure syslog_format is valid."""
        if v.lower() not in ["rfc3164", "rfc5424", "none"]:
            raise ValueError("syslog_format must be 'rfc3164', 'rfc5424', or 'none'")
        return v.lower()

    @field_validator("syslog_content_type")
    def validate_syslog_content_type(cls, v):
        """Ensure syslog_content_type is valid."""
        if v.lower() not in ["random", "cef"]:
            raise ValueError("syslog_content_type must be 'random' or 'cef'")
        return v.lower()


class _BatchMetricsAccumulator:
    """Per-thread accumulator for worker metrics.

    Worker threads record send-loop progress here without taking the shared
    metrics lock on every batch. ``flush()`` moves the accumulated deltas into
    the LoadGenerator's shared metrics dict (with one lock acquisition) and
    resets the local counters. This keeps lock contention low while still
    publishing during the observation window (rather than only on thread exit).
    """

    __slots__ = ("_loadgen", "sent", "failed", "bytes_sent", "late_batches")

    def __init__(self, loadgen: "LoadGenerator") -> None:
        self._loadgen = loadgen
        self.sent = 0
        self.failed = 0
        self.bytes_sent = 0
        self.late_batches = 0

    def flush(self) -> None:
        if not (self.sent or self.failed or self.bytes_sent or self.late_batches):
            return
        updates = {}
        if self.sent:
            updates["logs_produced"] = self.sent
            updates["bytes_sent"] = self.bytes_sent
        if self.failed:
            updates["failed"] = self.failed
        if self.late_batches:
            updates["late_batches"] = self.late_batches
        self._loadgen.update_metrics(**updates)
        self.sent = 0
        self.failed = 0
        self.bytes_sent = 0
        self.late_batches = 0


class LoadGenerator:
    def __init__(self):
        self.controller_thread = None
        self.stop_event = threading.Event()
        self.current_config = {}
        self.lock = threading.Lock()
        self.metrics = {"logs_produced": 0, "failed": 0, "bytes_sent": 0, "late_batches": 0}

    def generate_random_string(self, length: int) -> str:
        """
        Generate a random alphanumeric string of the specified length.
        """
        return "".join(
            random.choice(string.ascii_letters + string.digits) for _ in range(length)
        )

    def create_otlp_log_record(
        self,
        message_body: Optional[str] = None,
        body_size: int = 25,
        num_attributes: int = 2,
        attribute_value_size: int = 15,
    ):
        """
        Create a single OTLP log record.
        """
        if message_body is not None:
            log_message = message_body
        else:
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

    def update_metrics(self, **updates) -> None:
        """Update multiple metrics in a single lock acquisition."""
        with self.lock:
            for key, amount in updates.items():
                if key in self.metrics:
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
            self.create_otlp_log_record(
                message_body=args["message_body"],
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

        # Accumulate metrics locally between flushes to keep lock contention low
        # while still publishing during the observation window (not only on exit).
        acc = _BatchMetricsAccumulator(self)

        next_send_time = time.perf_counter()
        while not self.stop_event.is_set():
            try:
                stub.Export(logs_request)
                acc.sent += args["batch_size"]
                acc.bytes_sent += logs_request.ByteSize()
            except Exception as e:
                print(f"Thread {thread_id}: Failed to send log batch: {e}")
                acc.failed += args["batch_size"]

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
                    acc.late_batches += 1
                next_send_time += batch_interval

            # Publish metrics for this iteration so observers see progress
            # during the run, not only after the thread exits.
            acc.flush()

        # Final flush of any residual counters
        acc.flush()

    def syslog_tcp_worker_thread(self, thread_id: int, args: dict) -> None:
        """
        Worker thread that sends syslog messages to a syslog server via TCP.
        """
        syslog_server = args.get("syslog_server", os.getenv("SYSLOG_SERVER", "localhost"))
        syslog_port = int(args.get("syslog_port", os.getenv("SYSLOG_PORT", "514")))
        syslog_format = args.get("syslog_format", os.getenv("SYSLOG_FORMAT", "rfc3164"))

        print(f"Thread {thread_id}: Using TCP transport to syslog server {syslog_server}:{syslog_port}")

        # Create TCP socket for syslog
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)

        try:
            sock.connect((syslog_server, syslog_port))
            print(f"Thread {thread_id}: TCP connection established to {syslog_server}:{syslog_port}")
        except Exception as e:
            print(f"Thread {thread_id}: TCP connection failed to {syslog_server}:{syslog_port}: {e}")
            sock.close()
            return

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

        hostname = socket.gethostname()

        # Build message pool or use legacy static batch
        message_body = args.get("message_body")
        if message_body is not None:
            # Legacy mode: explicit message_body sends identical messages
            syslog_batch = []
            for _ in range(batch_size):
                syslog_message = self.create_syslog_message(
                    hostname=hostname,
                    message_body=message_body,
                    body_size=args["body_size"],
                    header_type=syslog_format,
                    syslog_content_type=args.get("syslog_content_type", "random"),
                    message_size=args.get("message_size"),
                )
                syslog_batch.append(syslog_message)
            batch_buffers = [b''.join(syslog_batch)]
        else:
            # Pool mode: diverse messages with structured entropy
            pool = _generate_message_pool(
                pool_size=256,
                header_type=syslog_format,
                message_size=args.get("message_size"),
            )
            # Pre-build multiple batch buffers with different shuffles
            num_batch_variants = 8
            batch_buffers = []
            for _ in range(num_batch_variants):
                batch = [random.choice(pool) for _ in range(batch_size)]
                batch_buffers.append(b''.join(batch))

        batch_idx = 0

        # Accumulate metrics locally between flushes to keep lock contention low
        # while still publishing during the observation window (not only on exit).
        acc = _BatchMetricsAccumulator(self)

        next_send_time = time.perf_counter()
        while not self.stop_event.is_set():
            try:
                buf = batch_buffers[batch_idx]
                sock.sendall(buf)
                batch_idx = (batch_idx + 1) % len(batch_buffers)
                acc.sent += args["batch_size"]
                acc.bytes_sent += len(buf)
            except Exception as e:
                print(f"Thread {thread_id}: Failed to send syslog batch: {e}")
                acc.failed += args["batch_size"]
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
                    acc.late_batches += 1
                next_send_time += batch_interval

            # Publish metrics for this iteration so observers see progress
            # during the run, not only after the thread exits.
            acc.flush()

        # Final flush of any residual counters
        acc.flush()

        sock.close()

    def syslog_udp_worker_thread(self, thread_id: int, args: dict) -> None:
        """
        Worker thread that sends syslog messages to a syslog server via UDP.
        """
        syslog_server = args.get("syslog_server", os.getenv("SYSLOG_SERVER", "localhost"))
        syslog_port = int(args.get("syslog_port", os.getenv("SYSLOG_PORT", "514")))
        syslog_format = args.get("syslog_format", os.getenv("SYSLOG_FORMAT", "rfc3164"))

        print(f"Thread {thread_id}: Using UDP transport to syslog server {syslog_server}:{syslog_port}")

        # Create UDP socket for syslog
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        # TODO: We need to find the right values
        # sock.setsockopt(socket.SOL_SOCKET, socket.SO_SNDBUF, 1024*1024)
        sock.setblocking(False)
        recv_buf = sock.getsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF)
        send_buf = sock.getsockopt(socket.SOL_SOCKET, socket.SO_SNDBUF)
        print(f"Thread {thread_id}: UDP Send buffer: {send_buf} bytes, Recv buffer: {recv_buf} bytes")

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

        hostname = socket.gethostname()

        # Build message pool or use legacy static batch
        message_body = args.get("message_body")
        if message_body is not None:
            # Legacy mode: explicit message_body sends identical messages
            syslog_batch = []
            for _ in range(batch_size):
                syslog_message = self.create_syslog_message(
                    hostname=hostname,
                    message_body=message_body,
                    body_size=args["body_size"],
                    header_type=syslog_format,
                    syslog_content_type=args.get("syslog_content_type", "random"),
                    message_size=args.get("message_size"),
                )
                syslog_batch.append(syslog_message)
            message_batches = [syslog_batch]
        else:
            # Pool mode: diverse messages with structured entropy
            pool = _generate_message_pool(
                pool_size=256,
                header_type=syslog_format,
                message_size=args.get("message_size"),
            )
            num_batch_variants = 8
            message_batches = []
            for _ in range(num_batch_variants):
                batch = [random.choice(pool) for _ in range(batch_size)]
                message_batches.append(batch)

        batch_idx = 0

        # Accumulate metrics locally between flushes to keep lock contention low
        # while still publishing during the observation window (not only on exit).
        # We accumulate over each UDP batch (inner for-loop) and flush once per batch.
        acc = _BatchMetricsAccumulator(self)
        total_failed = 0  # cumulative for first-N error-print throttling only

        next_send_time = time.perf_counter()
        while not self.stop_event.is_set():
            # UDP: Send individual messages instead of single batch of messages
            current_batch = message_batches[batch_idx]
            batch_idx = (batch_idx + 1) % len(message_batches)
            for message in current_batch:
                try:
                    bytes_sent = sock.sendto(message, (syslog_server, syslog_port))
                    acc.sent += 1
                    acc.bytes_sent += bytes_sent
                except Exception as e:
                    acc.failed += 1
                    total_failed += 1
                    # Only print first few errors to avoid spam
                    if total_failed <= 3:
                        print(f"Thread {thread_id}: Failed to send syslog message via UDP: {e}")

            # Rate limiting logic
            if batch_interval:
                now = time.perf_counter()
                sleep_time = next_send_time - now
                if sleep_time > 0:
                    time.sleep(sleep_time)
                elif now - next_send_time > batch_interval:
                    # More than 1 interval behind
                    acc.late_batches += 1
                next_send_time += batch_interval

            # Publish metrics for this iteration so observers see progress
            # during the run, not only after the thread exits.
            acc.flush()

        # Final flush of any residual counters
        acc.flush()

        sock.close()
        print(f"Thread {thread_id}: Syslog UDP worker exiting")

    def create_syslog_message(
        self,
        hostname: str,
        message_body: Optional[str] = None,
        body_size: int = 25,
        header_type: str = "rfc3164",  # can be "rfc3164", "rfc5424", or "none"
        syslog_content_type: str = "random",  # can be "random" or "cef"
        message_size: Optional[int] = None,  # target total message size in bytes
    ) -> bytes:
        """
        Create a single syslog message.

        Content is determined by priority:
        1. syslog_content_type="cef" -> uses CEF template
        2. message_body (if provided) -> uses static body
        3. Otherwise -> random string of body_size length

        If message_size is set, the message is padded or truncated to that size.
        """
        # Pre-generate static parts of the message
        pri = "<134>"  # local0.info = 16*8+6 = 134
        tag = "loadgen"

        # Determine message content
        if syslog_content_type == "cef":
            log_message = CEF_TEMPLATE
        elif message_body is not None:
            log_message = message_body
        else:
            log_message = self.generate_random_string(body_size)

        # Header generation
        if header_type == "rfc3164":
            # Example: <134>Oct 15 14:32:01 hostname loadgen: message
            utc_time = dt.now(timezone.utc)
            day = utc_time.day
            timestamp = utc_time.strftime(f"%b {day:2d} %H:%M:%S")
            header = f"{pri}{timestamp} {hostname} {tag}: "
        elif header_type == "rfc5424":
            # Example: <134>2025-10-15T14:32:01Z hostname appname - - - message
            timestamp = dt.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
            header = f"{pri}1 {timestamp} {hostname} {tag} - - - "
        elif header_type.lower() == "none":
            header = ""
        else:
            raise ValueError(
                "Invalid header_type. Must be 'rfc3164', 'rfc5424', or 'none'."
            )

        syslog_message = f"{header}{log_message}\n"

        # Pad or truncate to target message_size if specified
        if message_size is not None:
            current_size = len(syslog_message.encode("utf-8"))
            if current_size < message_size:
                # Pad with spaces before the trailing newline
                padding_needed = message_size - current_size
                syslog_message = f"{header}{log_message}{' ' * padding_needed}\n"
            elif current_size > message_size:
                # Truncate the message body to fit, preserving header + newline
                header_bytes = len(header.encode("utf-8"))
                available = message_size - header_bytes - 1  # -1 for newline
                if available > 0:
                    syslog_message = f"{header}{log_message[:available]}\n"
                else:
                    syslog_message = f"{header}\n"

        return syslog_message.encode("utf-8")

    def run_loadgen(self, args_dict):
        """
        Start the load generation process by launching multiple worker threads.
        Chooses between OTLP and syslog workers based on configuration.
        """
        with self.lock:
            self.metrics.update({"logs_produced": 0, "failed": 0, "bytes_sent": 0})

        # Determine which worker thread to use based on configuration
        load_type = args_dict.get("load_type", "otlp").lower()

        if load_type == "syslog":
            syslog_transport = args_dict.get(
                "syslog_transport", os.getenv("SYSLOG_TRANSPORT", "udp")
            ).lower()

            if syslog_transport not in ["tcp", "udp"]:
                print(f"Invalid syslog_transport '{syslog_transport}', using 'udp'")
                syslog_transport = "udp"

            if syslog_transport == "udp":
                worker_func = self.syslog_udp_worker_thread
            else:
                worker_func = self.syslog_tcp_worker_thread
        else:
            worker_func = self.worker_thread

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
    """Return True if ``port`` cannot be bound on ``host``.

    Tests bindability (the property we actually care about before calling
    ``app.run``) rather than reachability. ``connect_ex`` only sees loopback
    listeners and would miss a process bound to the same wildcard address on
    a different interface. Using ``bind`` with ``SO_REUSEADDR`` disabled gives
    a true "is this address+port currently claimed?" answer.
    """
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        try:
            s.bind((host, port))
        except OSError:
            return True
    return False


def main():
    def get_default_value(field_name: str):
        field = LoadGenConfig.model_fields[field_name]
        if field.default_factory is not None:
            return field.default_factory()
        return field.default

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
        "--body-message",
        type=str,
        default=get_default_value("message_body"),
        help=(
            "Optional static message body to send "
            f"(default {get_default_value('message_body')})"
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
            f"{get_default_value('tcp_connection_per_thread')})"
        ),
    )
    parser.add_argument(
        "--load-type",
        type=str,
        default=get_default_value("load_type"),
        help=(
            "Load generation type: 'otlp' or 'syslog' (default "
            f"{get_default_value('load_type')})"
        ),
    )
    parser.add_argument(
        "--syslog-server",
        type=str,
        default=os.getenv("SYSLOG_SERVER", get_default_value("syslog_server")),
        help=(
            "Syslog server address "
            f"(default {get_default_value('syslog_server')}, env: SYSLOG_SERVER)"
        ),
    )
    parser.add_argument(
        "--syslog-port",
        type=int,
        default=int(os.getenv("SYSLOG_PORT", str(get_default_value("syslog_port")))),
        help=(
            "Syslog server port "
            f"(default {get_default_value('syslog_port')}, env: SYSLOG_PORT)"
        ),
    )
    parser.add_argument(
        "--syslog-transport",
        type=str,
        default=os.getenv("SYSLOG_TRANSPORT", get_default_value("syslog_transport")),
        choices=["tcp", "udp"],
        help=(
            "Syslog transport protocol "
            f"(default {get_default_value('syslog_transport')}, env: SYSLOG_TRANSPORT)"
        ),
    )
    parser.add_argument(
        "--syslog-format",
        type=str,
        default=os.getenv("SYSLOG_FORMAT", get_default_value("syslog_format")),
        choices=["rfc3164", "rfc5424", "none"],
        help=(
            "Syslog header format "
            f"(default {get_default_value('syslog_format')}, env: SYSLOG_FORMAT)"
        ),
    )
    parser.add_argument(
        "--syslog-content-type",
        type=str,
        default=get_default_value("syslog_content_type"),
        choices=["random", "cef"],
        help=(
            "Syslog message content type "
            f"(default {get_default_value('syslog_content_type')})"
        ),
    )
    parser.add_argument(
        "--message-size",
        type=int,
        default=get_default_value("message_size"),
        help=(
            "Target total message size in bytes "
            f"(default {get_default_value('message_size')})"
        ),
    )
    args = parser.parse_args()

    if args.serve:
        if is_port_in_use(args.serve_port):
            raise RuntimeError(f"Port {args.serve_port} is already in use.")
        app.run(host="0.0.0.0", port=args.serve_port)
        return

    print("Starting load generator with configuration:")
    print(f"- Duration: {args.duration} seconds")
    print(f"- Load type: {args.load_type}")
    print(f"- Batch size: {args.batch_size} logs")
    print(f"- Threads: {args.threads}")
    print(f"- Target Rate: {args.target_rate}")
    print(f"- Log body size: {args.body_size} characters")
    print(f"- Log body message: {args.body_message}")
    print(f"- Attributes per log: {args.num_attributes}")
    print(f"- Attribute value size: {args.attribute_value_size} characters")
    if args.load_type == "syslog":
        print(f"- Syslog server: {args.syslog_server}:{args.syslog_port}")
        print(f"- Syslog transport: {args.syslog_transport}")
        print(f"- Syslog format: {args.syslog_format}")
        print(f"- Syslog content type: {args.syslog_content_type}")
    if args.message_size:
        print(f"- Target message size: {args.message_size} bytes")

    config = LoadGenConfig(
        body_size=args.body_size,
        num_attributes=args.num_attributes,
        attribute_value_size=args.attribute_value_size,
        message_body=args.body_message,
        batch_size=args.batch_size,
        threads=args.threads,
        target_rate=args.target_rate,
        load_type=args.load_type,
        syslog_server=args.syslog_server,
        syslog_port=args.syslog_port,
        syslog_transport=args.syslog_transport,
        syslog_format=args.syslog_format,
        syslog_content_type=args.syslog_content_type,
        message_size=args.message_size,
    )

    start_time = time.time()
    loadgen.start(config=config)

    try:
        time.sleep(args.duration)
    except KeyboardInterrupt:
        print("Interrupted by user, stopping early...")

    loadgen.stop()
    elapsed = time.time() - start_time

    logs_sent = loadgen.metrics.get("logs_produced", 0)
    logs_per_sec = logs_sent / elapsed if elapsed > 0 else 0
    print(f'LOADGEN_LOGS_SENT: {logs_sent}')
    print(f'LOADGEN_LOGS_FAILED: {loadgen.metrics.get("failed", 0)}')
    print(f'LOADGEN_BYTES_SENT: {loadgen.metrics.get("bytes_sent", 0)} bytes')
    print(f'LOADGEN_LOGS_SENT/SEC: {logs_per_sec:.2f}')


if __name__ == "__main__":
    signal.signal(signal.SIGINT, handle_signal)
    signal.signal(signal.SIGTERM, handle_signal)
    main()
