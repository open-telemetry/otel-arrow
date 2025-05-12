import time
from concurrent import futures
import grpc
from flask import Flask, jsonify
from threading import Thread

from opentelemetry.proto.collector.logs.v1 import logs_service_pb2_grpc
from opentelemetry.proto.collector.logs.v1.logs_service_pb2 import ExportLogsServiceResponse

# Constants for ports
FLASK_PORT = 5000
GRPC_PORT = 5317

app = Flask(__name__)
received_logs = 0

class FakeLogsExporter(logs_service_pb2_grpc.LogsServiceServicer):
    def Export(self, request, context):
        print("Received logs")
        global received_logs
        count = sum(len(ss.log_records) for rs in request.resource_logs for ss in rs.scope_logs)
        received_logs += count
        print(f"Total received logs: {received_logs}")
        return ExportLogsServiceResponse()

@app.route("/metrics")
def metrics():
    print("Metrics endpoint called")
    return jsonify({"received_logs": received_logs})


def start_flask():
    app.run(host="0.0.0.0", port=FLASK_PORT)

def serve():
    try:
        server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
        logs_service_pb2_grpc.add_LogsServiceServicer_to_server(FakeLogsExporter(), server)
        server.add_insecure_port(f"[::]:{GRPC_PORT}")
        server.start()
        print(f"Fake OTLP gRPC server started on port {GRPC_PORT}")
        server.wait_for_termination()
    except Exception as e:
        print(f"Error starting gRPC server: {e}")
        raise

if __name__ == '__main__':
    Thread(target=start_flask).start()
    print("About to start gRPC server")
    serve()
