import os
import grpc
import time
import argparse
import random
import string
import threading
import concurrent.futures
from opentelemetry.proto.collector.logs.v1 import logs_service_pb2_grpc, logs_service_pb2
from opentelemetry.proto.logs.v1 import logs_pb2
from opentelemetry.proto.common.v1 import common_pb2

def generate_random_string(length):
    """Generate a random string of given length."""
    return ''.join(random.choice(string.ascii_letters + string.digits) for _ in range(length))

def create_log_record(body_size=25, num_attributes=2, attribute_value_size=15):
    log_message = generate_random_string(body_size)

    attributes = []
    for i in range(num_attributes):
        attr_key = f"attribute.{i+1}"
        attr_value = generate_random_string(attribute_value_size)
        attributes.append(
            common_pb2.KeyValue(
                key=attr_key,
                value=common_pb2.AnyValue(string_value=attr_value)
            )
        )

    return logs_pb2.LogRecord(
        time_unix_nano=int(time.time_ns()),
        severity_text="INFO",
        severity_number=9,
        body=common_pb2.AnyValue(string_value=log_message),
        attributes=attributes,
    )

def worker_thread(thread_id, args, end_time):
    """Worker thread function that sends logs to the collector."""
    endpoint = os.getenv("OTLP_ENDPOINT", "localhost:4317")
    channel = grpc.insecure_channel(endpoint)
    stub = logs_service_pb2_grpc.LogsServiceStub(channel)

    # Pre-create the batch once and reuse it
    log_batch = [create_log_record(
        body_size=args.body_size,
        num_attributes=args.num_attributes,
        attribute_value_size=args.attribute_value_size
    ) for _ in range(args.batch_size)]

    scope_logs = logs_pb2.ScopeLogs(log_records=log_batch)
    resource_logs = logs_pb2.ResourceLogs(
        scope_logs=[scope_logs]
    )
    request = logs_service_pb2.ExportLogsServiceRequest(
        resource_logs=[resource_logs]
    )

    bytes_sent = 0
    sent = 0
    failed = 0

    print(f"Thread {thread_id} started, sending logs to {endpoint}")
    while time.time() < end_time:
        try:
            stub.Export(request)
            sent += args.batch_size
            bytes_sent += request.ByteSize()
        except Exception as e:
            failed += args.batch_size
            print(f"Thread {thread_id}: Failed to send log batch: {e}")

    return sent, failed, bytes_sent

def main():
    parser = argparse.ArgumentParser(description="Loadgen for OTLP logs")
    parser.add_argument("--duration", type=int, default=15, help="Duration in seconds (default: 15)")
    parser.add_argument("--batch-size", type=int, default=5000, help="Number of logs per batch (default: 10000)")
    parser.add_argument("--threads", type=int, default=4, help="Number of worker threads (default: 8)")
    parser.add_argument("--body-size", type=int, default=25,
                        help="Size of log message body in characters (default: 25)")
    parser.add_argument("--num-attributes", type=int, default=2,
                        help="Number of attributes per log (default: 2)")
    parser.add_argument("--attribute-value-size", type=int, default=15,
                        help="Size of attribute values in characters (default: 15)")
    args = parser.parse_args()

    print(f"Starting load generator with configuration:")
    print(f"- Duration: {args.duration} seconds")
    print(f"- Batch size: {args.batch_size} logs")
    print(f"- Threads: {args.threads}")
    print(f"- Log body size: {args.body_size} characters")
    print(f"- Attributes per log: {args.num_attributes}")
    print(f"- Attribute value size: {args.attribute_value_size} characters")

    end_time = time.time() + args.duration

    # Create and start worker threads
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.threads) as executor:
        futures = [executor.submit(worker_thread, i, args, end_time) for i in range(args.threads)]

        # Wait for all threads to complete
        total_sent = 0
        total_failed = 0
        total_bytes_sent = 0
        for future in concurrent.futures.as_completed(futures):
            sent, failed, bytes_sent = future.result()
            total_sent += sent
            total_failed += failed
            total_bytes_sent += bytes_sent

    print(f"LOADGEN_LOGS_SENT: {total_sent}")
    print(f"LOADGEN_LOGS_FAILED: {total_failed}")
    print(f"LOADGEN_BYTES_SENT: {total_bytes_sent} bytes")

if __name__ == "__main__":
    main()
