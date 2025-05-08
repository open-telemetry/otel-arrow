import os
import grpc
import time
import argparse
from opentelemetry.proto.collector.trace.v1 import trace_service_pb2_grpc, trace_service_pb2
from opentelemetry.proto.trace.v1 import trace_pb2

def create_span():
    return trace_pb2.Span(
        name="test-span",
        span_id=bytes.fromhex("1122334455667788"),
        trace_id=bytes.fromhex("aabbccddeeff00112233445566778899"),
        start_time_unix_nano=int(time.time_ns()),
        end_time_unix_nano=int(time.time_ns() + 1000000),
    )

def main():
    parser = argparse.ArgumentParser(description="Loadgen for OTLP traces")
    parser.add_argument("--duration", type=int, default=15, help="Duration in seconds (default: 15)")
    args = parser.parse_args()

    endpoint = os.getenv("OTLP_ENDPOINT", "localhost:4317")
    channel = grpc.insecure_channel(endpoint)
    stub = trace_service_pb2_grpc.TraceServiceStub(channel)

    end_time = time.time() + args.duration
    start_time = time.time()

    sent = 0
    while time.time() < end_time:
        span = create_span()
        resource_span = trace_pb2.ResourceSpans(
            scope_spans=[trace_pb2.ScopeSpans(spans=[span])]
        )
        request = trace_service_pb2.ExportTraceServiceRequest(
            resource_spans=[resource_span]
        )
        try:
            stub.Export(request)
            sent += 1
        except Exception as e:
            print(f"Failed to send span: {e}")

    actual_duration = time.time() - start_time
    rate_achieved = sent / actual_duration

    print(f"Loadgen done. Sent {sent} spans over {actual_duration:.2f}s.")
    print(f"Achieved rate: {rate_achieved:.2f} spans/second")

if __name__ == "__main__":
    main()
