import os
import grpc
import time
import argparse
import threading
import concurrent.futures
from opentelemetry.proto.collector.logs.v1 import logs_service_pb2_grpc, logs_service_pb2
from opentelemetry.proto.logs.v1 import logs_pb2
from opentelemetry.proto.common.v1 import common_pb2

def create_log_record():
    # Using a hardcoded timestamp (May 14, 2025 12:00:00 UTC in nanoseconds)
    # This avoids the system call overhead of time.time_ns()
    hardcoded_time_ns = 1747065600000000000
    return logs_pb2.LogRecord(
        time_unix_nano=hardcoded_time_ns,
        severity_text="INFO",
        severity_number=9,
        body=common_pb2.AnyValue(string_value="This is a test log message"),
        attributes=[
            common_pb2.KeyValue(
                key="service.name",
                value=common_pb2.AnyValue(string_value="loadgen-service")
            ),
            common_pb2.KeyValue(
                key="event.id",
                value=common_pb2.AnyValue(string_value="1122334455667788")
            )
        ],
    )

def worker_thread(thread_id, args, end_time):
    """Worker thread function that sends logs to the collector."""
    endpoint = os.getenv("OTLP_ENDPOINT", "localhost:4317")
    channel = grpc.insecure_channel(endpoint)
    stub = logs_service_pb2_grpc.LogsServiceStub(channel)
    
    sent = 0
    failed = 0
    
    while time.time() < end_time:
        log_batch = [create_log_record() for _ in range(args.batch_size)]
        scope_logs = logs_pb2.ScopeLogs(log_records=log_batch)
        resource_logs = logs_pb2.ResourceLogs(
            scope_logs=[scope_logs]
        )
        request = logs_service_pb2.ExportLogsServiceRequest(
            resource_logs=[resource_logs]
        )
        try:
            stub.Export(request)
            sent += args.batch_size
        except Exception as e:
            failed += args.batch_size
            print(f"Thread {thread_id}: Failed to send log batch: {e}")
            
    return sent, failed

def main():
    parser = argparse.ArgumentParser(description="Loadgen for OTLP logs")
    parser.add_argument("--duration", type=int, default=15, help="Duration in seconds (default: 15)")
    parser.add_argument("--batch-size", type=int, default=10000, help="Number of logs per batch (default: 10000)")
    parser.add_argument("--threads", type=int, default=8, help="Number of worker threads (default: 8)")
    args = parser.parse_args()

    end_time = time.time() + args.duration
    
    # Create and start worker threads
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.threads) as executor:
        futures = [executor.submit(worker_thread, i, args, end_time) for i in range(args.threads)]
        
        # Wait for all threads to complete
        total_sent = 0
        total_failed = 0
        for future in concurrent.futures.as_completed(futures):
            sent, failed = future.result()
            total_sent += sent
            total_failed += failed
    
    print(f"LOADGEN_LOGS_SENT: {total_sent}")
    print(f"LOADGEN_LOGS_FAILED: {total_failed}")

if __name__ == "__main__":
    main()
