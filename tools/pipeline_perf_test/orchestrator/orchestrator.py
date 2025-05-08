import argparse
import os
import time
import subprocess
from datetime import datetime
from typing import Optional, Dict, Any, List

class TargetProcess:
    """Class to manage target processes like the OTEL collector"""

    def __init__(self, target_type: str, container_id: str, config_path: Optional[str] = None):
        self.target_type = target_type
        self.container_id = container_id
        self.config_path = config_path

    def shutdown(self) -> None:
        """Gracefully shutdown the target process"""
        if self.container_id:
            print(f"Stopping Docker container {self.container_id}...")
            try:
                subprocess.run(["docker", "stop", self.container_id], check=True, capture_output=True, text=True)
                print(f"Docker container {self.container_id} stopped")
            except subprocess.CalledProcessError as e:
                print(f"Error stopping Docker container: {e}")
                print(f"Error output: {e.stderr}")


def launch_target(target_type: str = "collector", config_path: Optional[str] = None,
                 env_vars: Optional[Dict[str, str]] = None,
                 image_location: str = "otel",
                 image_tag: str = "latest",
                 ports: Optional[Dict[str, str]] = None,
                 container_name: Optional[str] = None
                 ) -> TargetProcess:
    """
    Launch the target using Docker

    Args:
        target_type: Type of target to launch (e.g., 'collector')
        config_path: Path to configuration file for the target
        env_vars: Environment variables to set for the target process
        image_location: Docker image location (e.g., 'otel' or 'ghcr.io/username')
        image_tag: Docker image tag (e.g., 'latest', 'v1.0')

    Returns:
        TargetProcess: Object representing the launched process
    """
    # Determine the Docker image name based on target type
    if target_type.lower() == 'collector':
        image_name = f"{image_location}/opentelemetry-collector:{image_tag}"
        if ports is None:
            ports = {"4317": "4317"}  # Default OTLP gRPC port
        if container_name is None:
            container_name = "otel-collector"
    #else:
        # TODO: Add other targets
        # image_name = f"{image_location}/{target_type}:{image_tag}"

    # Construct the Docker command
    cmd = ["docker", "run", "--rm", "-d"]

    # Add container name if provided
    if container_name:
        cmd.extend(["--name", container_name])

    # Add port mappings
    if ports:
        for host_port, container_port in ports.items():
            cmd.extend(["-p", f"{host_port}:{container_port}"])

    # Add environment variables if provided
    if env_vars:
        for key, value in env_vars.items():
            cmd.extend(["-e", f"{key}={value}"])

    # Add config mount if provided
    if config_path:
        # Get absolute path to config file
        abs_config_path = os.path.abspath(config_path)
        config_dir = os.path.dirname(abs_config_path)
        config_filename = os.path.basename(abs_config_path)
        cmd.extend(["-v", f"{config_dir}:/etc/otel/config:ro"])

        # Add the config file argument to the container command
        cmd.extend([image_name, "--config", f"/etc/otel/config/{config_filename}"])
    else:
        cmd.append(image_name)

    # Run the Docker container
    print(f"Launching {target_type} using Docker image: {image_name}...")
    try:
        # Start the container and get its ID
        container_id = subprocess.check_output(cmd, text=True).strip()
        print(f"Docker container started with ID: {container_id}")

        # Return a TargetProcess object
        return TargetProcess(
            target_type=target_type,
            container_id=container_id,
            config_path=config_path
        )
    except subprocess.CalledProcessError as e:
        print(f"Error launching Docker container: {e}")
        print(f"Error output: {e.stderr if hasattr(e, 'stderr') else 'No error output'}")
        raise

def run_loadgen(duration: int) -> Dict[str, Any]:
    """Run the load generator and return the results"""
    print("Starting load generator...")

    # Run the load generator
    cmd = ["python3", "load_generator/loadgen.py", "--duration", str(duration)]

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        output = result.stdout

        # Parse the output to extract metrics
        metrics = {}
        for line in output.strip().split("\n"):
            if "Sent" in line:
                try:
                    sent = int(line.split("Sent ")[1].split(" logs")[0])
                    metrics["logs_sent"] = sent
                except (IndexError, ValueError):
                    pass
            if "Achieved rate" in line:
                try:
                    rate = float(line.split("Achieved rate: ")[1].split(" logs/second")[0])
                    metrics["logs_per_second"] = rate
                except (IndexError, ValueError):
                    pass

        print(f"Load generator completed. Sent {metrics.get('logs_sent', 'unknown')} logs at {metrics.get('logs_per_second', 'unknown')} logs/second")
        return metrics
    except subprocess.CalledProcessError as e:
        print(f"Error running load generator: {e}")
        print(f"Error output: {e.stderr}")
        return {"error": str(e)}

# example usage
# python3 orchestrator/orchestrator.py --collector-config system_under_test/otel-collector/collector-config.yaml
# python3 orchestrator/orchestrator.py --collector-config system_under_test/otel-collector/collector-config.yaml --duration 30
def main():
    parser = argparse.ArgumentParser(description="Orchestrate OTel pipeline perf test")
    parser.add_argument("--keep-resources", action="store_true", help="Don't delete resources after test. Useful for debugging.")
    parser.add_argument("--duration", type=int, default=10, help="Duration to perform perf test in seconds")
    parser.add_argument("--results-dir", type=str, default="./results", help="Directory to store test results")
    parser.add_argument("--collector-config", type=str, required=True, help="Path to OTEL collector configuration file")
    parser.add_argument("--image-location", type=str, default="otel", help="Docker image location (e.g., 'otel' or 'ghcr.io/username')")
    parser.add_argument("--image-tag", type=str, default="latest", help="Docker image tag (e.g., 'latest', 'v1.0')")
    args = parser.parse_args()

    # Create results directory
    os.makedirs(args.results_dir, exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    results_file = os.path.join(args.results_dir, f"perf_results_{timestamp}.txt")

    target_process = None
    try:
        print("\nRunning perf tests...")

        # Launch the target system under test - OTEL collector now
        target_process = launch_target(
            'collector',
            args.collector_config,
            image_location=args.image_location,
            image_tag=args.image_tag
        )
        # Give it a moment to initialize
        time.sleep(2)

        # Run the load generator
        metrics = run_loadgen(args.duration)

        # Write results to file
        with open(results_file, "w") as f:
            f.write(f"Performance test run at: {timestamp}\n")
            f.write(f"Test duration: {args.duration} seconds\n")
            f.write(f"Collector config: {args.collector_config}\n\n")
            f.write("Results:\n")
            for key, value in metrics.items():
                f.write(f"- {key}: {value}\n")

        print(f"Test completed. Results saved to {results_file}")

    finally:
        if not args.keep_resources:
            print("\nCleaning up...")
            # Shutdown the target process if it was launched
            if target_process:
                target_process.shutdown()
        else :
            print("Resources kept for debugging. Manual cleanup may be required.")


if __name__ == "__main__":
    main()
