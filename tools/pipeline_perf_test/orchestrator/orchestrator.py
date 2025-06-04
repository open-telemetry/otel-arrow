r"""Orchestrator is the primary entrypoint for pipeline perf test benchmarking.

This script orchestrates performance testing for OpenTelemetry (OTel) pipelines by deploying test
environments  either using Docker or Kubernetes. It sets up and manages test resources, executes
a load generation workload, and captures performance results for analysis.

The main functionality is controlled via command-line arguments, allowing users to configure
test duration, resource management, deployment targets, and specific configurations or manifests
depending on the deployment mode.

Key Arguments:
- --duration: Duration of the performance test in seconds (default: 10).
- --keep-resources: Retain deployed resources after test completion (useful for debugging).
- --results-dir: Directory path to store performance test results.
- --deployment-target: Choose between 'docker' or 'kubernetes' (default: 'docker').
- --skip-backend-build / --skip-loadgen-build: Skip Docker builds for backend or load generator.
- Docker-specific:
    - --collector-config: Path to the OTEL configuration file (required for Docker deployment).
- Kubernetes-specific:
    - --k8s-namespace: Kubernetes namespace to use (default: 'default').
    - --k8s-collector-manifest / --k8s-backend-manifest / --k8s-loadgen-manifest: Paths to
            respective Kubernetes manifest YAML files (all required for Kubernetes deployment).

Usage Example:
    python3 orchestrator/orchestrator.py \
        --collector-config system_under_test/otel-collector/collector-config.yaml --duration 30

    python3 orchestrator/orchestrator.py --deployment-target kubernetes \
        --k8s-collector-manifest system_under_test/otel-collector/collector-manifest.yaml \
        --k8s-backend-manifest backend/backend-manifest.yaml \
        --k8s-loadgen-manifest load_generator/loadgen-manifest.yaml \
        --k8s-namespace perf-test-otel --duration 30

Raises errors when required configuration files are missing based on the selected deployment target.

Pre-requisites:
1. Create and activate a virtual environment:
   python3 -m venv venv
   source venv/bin/activate  # On Windows use: venv\Scripts\activate
2. Install dependencies:
   pip install -r orchestrator/requirements.txt
3. Run the orchestrator with Docker:
    python3 orchestrator/orchestrator.py \
        --collector-config system_under_test/otel-collector/collector-config.yaml --duration 30
4. Run with Kubernetes (currently requires kubernetes metrics-server):
    python3 orchestrator/orchestrator.py --deployment-target kubernetes \
        --k8s-collector-manifest system_under_test/otel-collector/collector-manifest.yaml \
        --k8s-backend-manifest backend/backend-manifest.yaml \
        --k8s-loadgen-manifest load_generator/loadgen-manifest.yaml \
        --k8s-namespace perf-test-otel --duration 30
    
    If the cluster does not have the metrics server installed, you will need to install it. Be aware that the script below also enables insecure TLS for dev purposes:

        # Apply metrics-server
        kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml

        # Patch with both required flags
        kubectl patch deployment metrics-server -n kube-system --type=json -p='[
        {"op":"add","path":"/spec/template/spec/containers/0/args/-","value":"--kubelet-insecure-tls"},
        {"op":"add","path":"/spec/template/spec/containers/0/args/-","value":"--kubelet-preferred-address-types=InternalIP"}
        ]'

        # Restart deployment
        kubectl rollout restart deployment metrics-server -n kube-system
"""
import argparse
import os
import subprocess
import time
from datetime import datetime

# This will get cleaned up when we refactor the test control flow, but for now just import lots of stuff.
import docker
from lib.process.utils.docker import VolumeMount, PortBinding, cleanup_docker_containers
from lib.process.utils.docker import build_docker_image, launch_container, get_docker_logs, create_docker_network, delete_docker_network
from lib.process.utils.kubernetes import create_k8s_namespace, deploy_kubernetes_resources, run_k8s_loadgen, setup_k8s_port_forwarding
from lib.report.report import get_report_string, get_benchmark_json, parse_logs_for_sent_count

def main():
    parser = argparse.ArgumentParser(description="Orchestrate OTel pipeline perf test")
    parser.add_argument("--duration", type=int, default=10, help="Duration to perform perf test in seconds")
    parser.add_argument("--keep-resources", action="store_true", help="Don't delete resources after test. Useful for debugging.")
    parser.add_argument("--results-dir", type=str, default="./results", help="Directory to store test results")
    parser.add_argument("--log-cli-commands", action="store_true", help="Log the equivalent docker / kubetcl cli commands executed. Useful for debugging.")

    # Deployment target choice
    parser.add_argument("--deployment-target", type=str, choices=["docker", "kubernetes"], default="docker",
                        help="Whether to deploy to Docker (default) or Kubernetes")

    parser.add_argument("--skip-backend-build", action="store_true", help="Skip building backend Docker image (for Docker/K8s deployment)")
    parser.add_argument("--skip-loadgen-build", action="store_true", help="Skip building loadgen Docker image (for Docker/K8s deployment)")

    # Docker-specific arguments
    parser.add_argument("--collector-config", type=str, help="Path to OTEL collector configuration file (for Docker deployment)")

    # Kubernetes-specific arguments
    parser.add_argument("--k8s-namespace", type=str, default="default", help="Kubernetes namespace for deployments")
    parser.add_argument("--k8s-collector-manifest", type=str, help="Path to collector Kubernetes manifest YAML")
    parser.add_argument("--k8s-backend-manifest", type=str, help="Path to backend Kubernetes manifest YAML")
    parser.add_argument("--k8s-loadgen-manifest", type=str, help="Path to load generator Kubernetes manifest YAML")

    args = parser.parse_args()

    # Validate arguments based on deployment target
    if args.deployment_target == "docker" and not args.collector_config:
        parser.error("--collector-config is required for Docker deployment")
    elif args.deployment_target == "kubernetes" and (not args.k8s_collector_manifest or not args.k8s_backend_manifest or not args.k8s_loadgen_manifest):
        parser.error("--k8s-collector-manifest, --k8s-backend-manifest, and --k8s-loadgen-manifest are required for Kubernetes deployment")

    # Create results directory
    os.makedirs(args.results_dir, exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    # Initialize resources and counters
    network_created = False
    backend_process = None
    target_process = None
    loadgen_process = None
    target_process_stats = None
    k8s_backend_resource = None
    k8s_collector_resource = None
    k8s_port_forward_process = None
    logs_failed_count = 0  # Initialize failed count to 0

    try:
        print("\nRunning perf tests...")

        # Initialize image names
        backend_image = "backend-service:latest"
        loadgen_image = "otel-loadgen:latest"

        docker_client = None
        if not args.skip_backend_build or not args.skip_loadgen_build:
            print("Building container images(s)...")
            docker_client = docker.from_env()
            # Build the backend Docker image if not skipped
            if not args.skip_backend_build:
                backend_image = build_docker_image(backend_image, "backend", docker_client, log_cli=args.log_cli_commands)
            else:
                print(f"Using existing backend image: {backend_image}")

            # Build the loadgen Docker image if not skipped
            if not args.skip_loadgen_build:
                loadgen_image = build_docker_image(loadgen_image, "load_generator", docker_client, log_cli=args.log_cli_commands)
            else:
                print(f"Using existing loadgen image: {loadgen_image}")

        if args.deployment_target == "docker":
            if not docker_client:
                docker_client = docker.from_env()
            # Clean up any existing containers with the same names we'll use
            cleanup_docker_containers(["backend-service", "otel-collector", "otel-loadgen"], docker_client, log_cli=args.log_cli_commands)

            # Docker deployment flow
            # Create a Docker network for inter-container communication
            network = "perf-test-network"
            network_created = create_docker_network(network, docker_client, log_cli=args.log_cli_commands)


            backend_ports = [
                PortBinding(container_port=5317, host_port=5317),
                PortBinding(container_port=5000, host_port=5000),
            ]
            # Launch the backend service as a Docker container
            backend_process = launch_container(
                image_name=backend_image,
                container_name="backend-service",
                client=docker_client,
                ports=backend_ports,
                network=network,
                log_cli=args.log_cli_commands
            )

            # Give it a moment to initialize
            time.sleep(2)

            # Prepare collector config mounting
            collector_cmd_args = []
            abs_config_path = os.path.abspath(args.collector_config)
            config_dir = os.path.dirname(abs_config_path)
            config_filename = os.path.basename(abs_config_path)
            collector_volumes = [VolumeMount(config_dir,"/etc/otel/config","ro")]
            collector_cmd_args = ["--config", f"/etc/otel/config/{config_filename}"]


            collector_ports = [
                PortBinding(container_port=4317, host_port=4317),
            ]
            # Launch the collector
            collector_image = "otel/opentelemetry-collector:latest"
            target_process = launch_container(
                image_name=collector_image,
                container_name="otel-collector",
                client=docker_client,
                ports=collector_ports,
                network=network,
                volume_mounts=collector_volumes,
                command_args=collector_cmd_args,
                log_cli=args.log_cli_commands
            )

            # Give it a moment to initialize
            time.sleep(2)

            # Run the load generator using Docker
            print("Starting load generator using Docker...")
            loadgen_env = {
                "OTLP_ENDPOINT": "otel-collector:4317"  # Use Docker network DNS name
            }
            loadgen_process = launch_container(
                image_name=loadgen_image,
                container_name="otel-loadgen",
                client=docker_client,
                network=network,
                environment=loadgen_env,
                command_args=["--duration", str(args.duration)],
                log_cli=args.log_cli_commands
            )

            # Start monitoring once the load generator is built and launched
            # Set statically to ensure ~10 samples for now
            target_process.start_monitoring(args.duration/10)

            # Wait for the loadgen container to finish (it runs for the specified duration)
            print(f"Waiting for load generator to finish (running for {args.duration}s)...")

            # Let the load run for the specified duration
            time.sleep(args.duration)
            # Stop monitoring immediately to avoid recording idle stats.
            # Eventually this should be based on active signaling that the test load is done rather than timers.
            target_process.stop_monitoring()
            # Add 5 seconds buffer
            time.sleep(5)
            target_process_stats = target_process.get_stats()

            # Get logs from the container (which should have finished by now)
            print("Getting logs from load generator container...")
            logs = get_docker_logs(loadgen_process.container_id, docker_client, log_cli=args.log_cli_commands)

            # Parse the output to extract logs sent count and failed count
            logs_sent_count, logs_failed_count = parse_logs_for_sent_count(logs)

            if logs_sent_count > 0:
                print(f"Load generator completed. Sent {logs_sent_count} logs, Failed {logs_failed_count} logs")
            else:
                logs_sent_count = -1
                logs_failed_count = 0

        else:
            # Create namespace if it doesn't exist
            if not create_k8s_namespace(args.k8s_namespace):
                print("Failed to create or confirm Kubernetes namespace. Exiting.")
                return 1

            # Deploy backend
            k8s_backend_resource = deploy_kubernetes_resources(
                args.k8s_backend_manifest,
                "backend",
                args.k8s_namespace
            )

            # Wait for backend to be ready
            k8s_backend_resource.wait_until_ready()

            # Set up port forwarding for the backend service metrics endpoint
            # This is necessary to access the backend service metrics from the orchestrator
            try:
                service_name = "backend-service"
                remote_port = 5000

                print(f"Setting up port forwarding for backend metrics: {service_name}:{remote_port}")
                k8s_port_forward_process = setup_k8s_port_forwarding(service_name, args.k8s_namespace, remote_port, remote_port)
                if k8s_port_forward_process:
                    print(f"Port forwarding established. Backend metrics available at http://localhost:{remote_port}/metrics")
                else:
                    print("Failed to set up port forwarding for backend metrics. Metrics collection may fail.")
            except Exception as e:
                print(f"Error setting up port forwarding: {e}")
                print("Continuing without port forwarding. Metrics collection may fail.")

            # Deploy collector
            k8s_collector_resource = deploy_kubernetes_resources(
                args.k8s_collector_manifest,
                "otel-collector",
                args.k8s_namespace
            )

            # Wait for collector to be ready
            k8s_collector_resource.wait_until_ready()

            # Run the load generator in Kubernetes
            logs_sent_count, logs_failed_count, _ = run_k8s_loadgen(
                args.k8s_loadgen_manifest,
                args.k8s_namespace,
                args.duration,
                k8s_collector_resource,
                loadgen_image
            )

            target_process_stats = k8s_collector_resource.get_stats()


        # Print results
        print(
            get_report_string(
                timestamp,
                args,
                logs_failed_count,
                logs_sent_count,
                target_process_stats
            )
        )

        # Write benchmark result to file in a JSON
        # format expected by GitHub Action Benchmark
        benchmark_file = os.path.join(args.results_dir, f"benchmark_{timestamp}.json")
        with open(benchmark_file, "w") as f:
            f.write(
            get_benchmark_json(
                timestamp,
                args,
                logs_failed_count,
                logs_sent_count,
                target_process_stats
            )
            )

        print(f"Test completed. Benchmark data saved to {benchmark_file}")

    finally:
        if not args.keep_resources:
            print("\nCleaning up...")
            if args.deployment_target == "docker":
                # Cleanup Docker resources
                if target_process:
                    target_process.shutdown()
                if backend_process:
                    backend_process.shutdown()
                if loadgen_process:
                    loadgen_process.shutdown()
                if network_created:
                    delete_docker_network(network, docker_client)
            else:
                # Cleanup Kubernetes resources
                # First terminate port forwarding if it's active
                if k8s_port_forward_process:
                    print("Terminating port forwarding...")
                    k8s_port_forward_process.terminate()
                    k8s_port_forward_process.wait(timeout=5)
                    print("Port forwarding terminated")

                # Then clean up other resources
                if k8s_collector_resource:
                    k8s_collector_resource.shutdown()
                if k8s_backend_resource:
                    k8s_backend_resource.shutdown()
                # The loadgen manifest is automatically cleaned up after the job completes
                try:
                    subprocess.run(["kubectl", "delete", "-f", args.k8s_loadgen_manifest, "-n", args.k8s_namespace, "--ignore-not-found=true"],
                                   check=True, capture_output=True)
                    print(f"Deleted load generator resources")
                except subprocess.CalledProcessError as e:
                    print(f"Error deleting load generator resources: {e}")

                # Delete the entire namespace if it's not the default namespace
                if args.k8s_namespace != "default":
                    print(f"Deleting Kubernetes namespace: {args.k8s_namespace}")
                    try:
                        subprocess.run(["kubectl", "delete", "namespace", args.k8s_namespace, "--ignore-not-found=true"],
                                      check=True, capture_output=True)
                        print(f"Deleted Kubernetes namespace: {args.k8s_namespace}")
                    except subprocess.CalledProcessError as e:
                        print(f"Error deleting Kubernetes namespace: {e}")
                        print(f"Error output: {e.stderr}")
        else:
            print("Resources kept for debugging. Manual cleanup may be required.")


if __name__ == "__main__":
    main()
