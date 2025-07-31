"""
A log ingestion backend for use in pipeline performance testing.

This module does the following:
- Starts a gRPC server that listens for OTLP log export requests on port 5317.
- Implements a basic LogsServiceServicer to count the number of received log records.
- Starts a Flask server on port 5000 that exposes two endpoints:
    - `/metrics`: Returns the count of received logs in JSON format.
    - `/prom_metrics`: Returns the count in Prometheus-friendly plain text format.
- Handles graceful shutdown on SIGINT and SIGTERM signals.

Intended for testing or development purposes where a mock OTLP log collector is needed.
"""

import asyncio
import os
import signal
import sys
import grpc  # type: ignore
from flask import Flask, jsonify
from opentelemetry.proto.collector.logs.v1 import logs_service_pb2_grpc
from opentelemetry.proto.collector.logs.v1.logs_service_pb2 import (
    ExportLogsServiceResponse,
)

# Constants for ports
FLASK_PORT = int(os.getenv('FLASK_PORT', 5000))
GRPC_PORT = int(os.getenv('GRPC_PORT', 5317))

app = Flask(__name__)
received_logs = 0
received_logs_lock = asyncio.Lock()  # Async lock to guard the received_logs
grpc_server = None


def handle_signal(signal, frame):
    print("Received signal to terminate, stopping gRPC server.")
    if grpc_server:
        grpc_server.stop(0)
    sys.exit(0)


class FakeLogsExporter(logs_service_pb2_grpc.LogsServiceServicer):
    async def Export(self, request, context):
        global received_logs
        count = sum(
            len(ss.log_records) for rs in request.resource_logs for ss in rs.scope_logs
        )
        # Acquire the lock before modifying the global received_logs
        async with received_logs_lock:
            received_logs += count
            if received_logs % 10000 == 0:
                print(f"Total received logs: {received_logs}")
        return ExportLogsServiceResponse()


@app.route("/metrics")
async def metrics():
    async with received_logs_lock:
        print(f"Metrics endpoint called. Returning: {received_logs}")
        return jsonify({"received_logs": received_logs})


@app.route("/prom_metrics")
async def prom_metrics():
    async with received_logs_lock:
        print(f"Metrics endpoint called. Returning: {received_logs}")
        return f"received_logs {received_logs}"


async def start_flask():
    # Run Flask app asynchronously
    from threading import Thread

    flask_thread = Thread(
        target=lambda: app.run(host="0.0.0.0", port=FLASK_PORT, use_reloader=False)
    )
    flask_thread.daemon = True
    flask_thread.start()
    await asyncio.sleep(0)


async def serve():
    global grpc_server
    try:
        grpc_server = grpc.aio.server()
        logs_service_pb2_grpc.add_LogsServiceServicer_to_server(
            FakeLogsExporter(), grpc_server
        )
        grpc_server.add_insecure_port(f"[::]:{GRPC_PORT}")
        await grpc_server.start()
        print(f"Fake OTLP gRPC server started on port {GRPC_PORT}")
        await grpc_server.wait_for_termination()
    except Exception as e:
        print(f"Error starting gRPC server: {e}")
        raise


async def main():
    signal.signal(signal.SIGINT, handle_signal)
    signal.signal(signal.SIGTERM, handle_signal)

    # Start both Flask and gRPC servers
    flask_task = asyncio.create_task(start_flask())
    grpc_task = asyncio.create_task(serve())

    # Run both tasks concurrently
    await asyncio.gather(flask_task, grpc_task)


if __name__ == "__main__":
    asyncio.run(main())
