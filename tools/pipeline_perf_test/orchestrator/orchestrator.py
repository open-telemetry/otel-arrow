import argparse
import os
import time
import requests
import subprocess
from datetime import datetime
from typing import Optional, Dict, Any, List

class DockerProcess:
    """Class to manage Docker processes like containers"""

    def __init__(self, container_id: str):
        self.container_id = container_id

    def shutdown(self) -> None:
        """Gracefully shutdown the Docker container"""
        if self.container_id:
            try:
                subprocess.run(["docker", "stop", self.container_id], check=True, capture_output=True, text=True)
            except subprocess.CalledProcessError as e:
                print(f"Error stopping Docker container: {e}")
                print(f"Error output: {e.stderr}")


def launch_container(
    image_name: str,
    container_name: str,
    ports: Optional[Dict[str, str]] = None,
    network: Optional[str] = None,
    command_args: Optional[List[str]] = None,
    volumes: Optional[Dict[str, str]] = None
) -> DockerProcess:
    """
    Launch a Docker container

    Args:
        image_name: Docker image name with tag
        container_name: name for the container
        ports: Port mappings from host to container
        network: Docker network to connect to
        command_args: Additional command arguments to pass to the container
        volumes: Volume mounts from host to container

    Returns:
        DockerProcess: Object representing the launched container
    """
    print(f"Starting {container_name} service using Docker image: {image_name}...")

    # Construct the Docker command
    cmd = ["docker", "run", "--rm", "-d"]

    # Add container name
    cmd.extend(["--name", container_name])

    # Add network if specified
    if network:
        cmd.extend(["--network", network])

    # Add port mappings
    if ports:
        for host_port, container_port in ports.items():
            cmd.extend(["-p", f"{host_port}:{container_port}"])

    # Add volume mounts if provided
    if volumes:
        for host_path, container_path in volumes.items():
            cmd.extend(["-v", f"{host_path}:{container_path}"])

    # Add the image name
    cmd.append(image_name)

    # Add any additional command arguments
    if command_args:
        cmd.extend(command_args)

    # Run the Docker container
    try:
        # Start the container and get its ID
        container_id = subprocess.check_output(cmd, text=True).strip()
        print(f"Docker container started with ID: {container_id}")

        # Return a TargetProcess object
        return DockerProcess(
            container_id=container_id
        )
    except subprocess.CalledProcessError as e:
        print(f"Error launching Docker container: {e}")
        print(f"Error output: {e.stderr if hasattr(e, 'stderr') else 'No error output'}")
        raise

def run_loadgen(duration: int) -> (int, float):
    """Run the load generator and return the count of logs and duration"""
    print("Starting load generator...")

    # Record start time to calculate actual rate
    start_time = time.time()

    # Run the load generator
    cmd = ["python3", "load_generator/loadgen.py", "--duration", str(duration)]

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        output = result.stdout

        # Calculate actual duration
        actual_duration = time.time() - start_time

        # Parse the output to extract logs sent count
        logs_sent = 0
        for line in output.strip().split("\n"):
            if "LOADGEN_LOGS_SENT:" in line:
                try:
                    logs_sent = int(line.split("LOADGEN_LOGS_SENT:")[1].strip())
                except (IndexError, ValueError) as e:
                    print(f"Failed to parse logs sent count: {e}")
                    logs_sent = -1

        if logs_sent == 0:
            print(f"Could not find LOADGEN_LOGS_SENT in output: {output}")
        else:
            print(f"Load generator completed. Sent {logs_sent} logs in {actual_duration:.2f}s")

        return logs_sent, actual_duration
    except subprocess.CalledProcessError as e:
        print(f"Error running load generator: {e}")
        print(f"Error output: {e.stderr}")
        return -1, 0.0

def build_backend_image(backend_dir: str = "backend") -> str:
    """
    Build the backend Docker image locally

    Args:
        backend_dir: Directory containing the backend Dockerfile

    Returns:
        str: Name of the built image
    """
    image_name = "fake-backend:latest"

    print(f"Building backend Docker image '{image_name}'...")

    # Get the absolute path to the backend directory
    backend_path = os.path.abspath(backend_dir)

    # Build the Docker image
    try:
        cmd = ["docker", "build", "-t", image_name, backend_path]
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(f"Successfully built backend Docker image: {image_name}")
        return image_name
    except subprocess.CalledProcessError as e:
        print(f"Error building backend Docker image: {e}")
        print(f"Error output: {e.stderr}")
        raise

BACKEND_STATS_URL = "http://localhost:5000/metrics"
def get_backend_received_count() -> int:
    """
    Query the backend service for the count of received logs.

    Returns:
        int: The count of received logs, or -1 if the query fails.
    """
    print("\nQuerying backend for received count...")
    try:
        response = requests.get(BACKEND_STATS_URL)
        data = response.json()
        count = data.get("received_logs", -1)
        return count
    except Exception as e:
        print(f"Failed to query backend service: {e}")
        return -1

# Pre-requisites:
# 1. Create and activate a virtual environment:
#    python3 -m venv venv
#    source venv/bin/activate  # On Windows use: venv\Scripts\activate
# 2. Install dependencies:
#    pip install -r requirements.txt
# 3. Run the orchestrator:
#    python3 orchestrator/orchestrator.py --collector-config system_under_test/otel-collector/collector-config.yaml --duration 30
def main():
    parser = argparse.ArgumentParser(description="Orchestrate OTel pipeline perf test")
    parser.add_argument("--duration", type=int, default=10, help="Duration to perform perf test in seconds")
    parser.add_argument("--collector-config", type=str, required=True, help="Path to OTEL collector configuration file")
    parser.add_argument("--keep-resources", action="store_true", help="Don't delete resources after test. Useful for debugging.")
    parser.add_argument("--results-dir", type=str, default="./results", help="Directory to store test results")
    parser.add_argument("--skip-backend-build", action="store_true", help="Skip building backend Docker image (use existing image)")

    args = parser.parse_args()

    # Create results directory
    os.makedirs(args.results_dir, exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    results_file = os.path.join(args.results_dir, f"perf_results_{timestamp}.txt")

    # Create a Docker network for inter-container communication
    network = "perf-test-network"
    network_created = False
    try:
        subprocess.run(["docker", "network", "create", network], check=True, capture_output=True)
        print(f"Created Docker network: {network}")
        network_created = True
    except subprocess.CalledProcessError as e:
        if "already exists" not in str(e.stderr):
            print(f"Error creating network: {e}")
            print(f"Error output: {e.stderr}")
            raise
        print(f"Using existing Docker network: {network}")

    backend_process = None
    target_process = None

    try:
        print("\nRunning perf tests...")

        # Build the backend Docker image if not skipped
        backend_image = "fake-backend:latest"
        if not args.skip_backend_build:
            backend_image = build_backend_image("backend")

        # Launch the backend service as a Docker container
        backend_process = launch_container(
            image_name=backend_image,
            container_name="fake-backend",
            ports={"5317": "5317", "5000": "5000"},
            network=network
        )

        # Give it a moment to initialize
        time.sleep(2)

        # Prepare collector config mounting
        collector_volumes = {}
        collector_cmd_args = []
        abs_config_path = os.path.abspath(args.collector_config)
        config_dir = os.path.dirname(abs_config_path)
        config_filename = os.path.basename(abs_config_path)
        collector_volumes[config_dir] = "/etc/otel/config:ro"
        collector_cmd_args = ["--config", f"/etc/otel/config/{config_filename}"]

        # Launch the collector
        collector_image = f"otel/opentelemetry-collector:latest"
        target_process = launch_container(
            image_name=collector_image,
            container_name="otel-collector",
            ports={"4317": "4317"},
            network=network,
            volumes=collector_volumes,
            command_args=collector_cmd_args
        )

        # Give it a moment to initialize
        time.sleep(2)

        # Run the load generator
        logs_sent_count, duration = run_loadgen(args.duration)
        logs_received_backend_count = get_backend_received_count()

        # Calculate total logs lost
        total_logs_lost = logs_sent_count - logs_received_backend_count

        # Calculate logs sent rate
        logs_sent_rate = logs_sent_count / duration if duration > 0 else 0

        # Print results
        print(f"Total logs sent: {logs_sent_count}")
        print(f"Total logs backend reports as received: {logs_received_backend_count}")
        print(f"Duration: {duration:.2f} seconds")
        print(f"Logs sent rate: {logs_sent_rate:.2f} logs/second")
        print(f"Total logs lost: {total_logs_lost}")

        # Write results to file
        with open(results_file, "w") as f:
            f.write(f"Performance test run at: {timestamp}\n")
            f.write(f"Test duration: {args.duration} seconds\n")
            f.write(f"Collector config: {args.collector_config}\n\n")
            f.write("Results:\n")
            f.write(f"- Total logs sent: {logs_sent_count}\n")
            f.write(f"- Duration: {duration:.2f} seconds\n")
            f.write(f"- Logs sent rate: {logs_sent_rate:.2f} logs/second\n")
            f.write(f"- Total logs lost: {total_logs_lost}\n")

        print(f"Test completed. Results saved to {results_file}")

    finally:
        if not args.keep_resources:
            print("\nCleaning up...")
            # Shutdown the target process if it was launched
            if target_process:
                target_process.shutdown()
            if backend_process:
                backend_process.shutdown()
            if network_created:
                try:
                    subprocess.run(["docker", "network", "rm", network], check=True, capture_output=True)
                except subprocess.CalledProcessError as e:
                    print(f"Error removing Docker network: {e}")
        else:
            print("Resources kept for debugging. Manual cleanup may be required.")


if __name__ == "__main__":
    main()
