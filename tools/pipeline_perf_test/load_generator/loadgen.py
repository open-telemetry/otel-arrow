import os
import grpc
import time
import argparse
from opentelemetry.proto.collector.logs.v1 import logs_service_pb2_grpc, logs_service_pb2
from opentelemetry.proto.logs.v1 import logs_pb2
from opentelemetry.proto.common.v1 import common_pb2

def create_log_record():
    return logs_pb2.LogRecord(
        time_unix_nano=int(time.time_ns()),
        severity_text="INFO",
        severity_number=9,  # INFO in OTel severity number
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

def main():
    parser = argparse.ArgumentParser(description="Loadgen for OTLP logs")
    parser.add_argument("--duration", type=int, default=15, help="Duration in seconds (default: 15)")
    args = parser.parse_args()

    endpoint = os.getenv("OTLP_ENDPOINT", "localhost:4317")
    channel = grpc.insecure_channel(endpoint)
    stub = logs_service_pb2_grpc.LogsServiceStub(channel)

    end_time = time.time() + args.duration
    start_time = time.time()

    sent = 0
    while time.time() < end_time:
        log_record = create_log_record()
        scope_logs = logs_pb2.ScopeLogs(
            log_records=[log_record]
        )
        resource_logs = logs_pb2.ResourceLogs(
            scope_logs=[scope_logs]
        )
        request = logs_service_pb2.ExportLogsServiceRequest(
            resource_logs=[resource_logs]
        )
        try:
            stub.Export(request)
            sent += 1
        except Exception as e:
            print(f"Failed to send log: {e}")

    actual_duration = time.time() - start_time
    rate_achieved = sent / actual_duration

    print(f"Loadgen done. Sent {sent} logs over {actual_duration:.2f}s.")
    print(f"Achieved rate: {rate_achieved:.2f} logs/second")

if __name__ == "__main__":
    main()