import argparse
import os
import re
import subprocess
import threading
import time
from datetime import datetime
from typing import Optional, Dict, List, Tuple

import requests


class ProcessStatsAggregation:
    """Class for aggregation of resource stats"""
    min: float
    max: float
    total: float
    samples: int

    def __init__(self):
        self.min = None
        self.max = None
        self.total = 0.0
        self.samples = 0

    def add_sample(self, sample: float):
        """Add a sample to the aggregation.

        Args:
            sample: A float representing the value observed
        """
        self.samples += 1
        self.total += sample
        if not self.min or self.min > sample:
            self.min = sample
        if not self.max or self.max < sample:
            self.max = sample


class ProcessStats:
    """Class for tracking observed process resource utilization stats"""
    def __init__(self):
        self.lock = threading.Lock()
        self.cpu = ProcessStatsAggregation()
        self.mem = ProcessStatsAggregation()

    def add_sample(self, cpu_percent: float, mem_mib: float):
        """Add an observation of cpu and memory utilizations stats

        Args:
            cpu_percent: Percentage utilization of single cpu expressed as a float.
            mem_mb: Memory utilization for the process in MiB.
        """
        with self.lock:
            self.cpu.add_sample(cpu_percent)
            self.mem.add_sample(mem_mib)

    def get_summary_string(self, metric_prefix: str, delimiter: str ="/") -> str:
        """Helper method to format min/avg/max stats."""
        summary = self.get_summary()
        return delimiter.join([
            f"{summary.get(metric_prefix + '_min', 0.0):.2f}",
            f"{summary.get(metric_prefix + '_avg', 0.0):.2f}",
            f"{summary.get(metric_prefix + '_max', 0.0):.2f}"
        ])

    def get_summary(self) -> Dict[str, float]:
        """Get a summary of observed resource utilization stats"""
        with self.lock:
            if not self.cpu.samples or not self.mem.samples:
                return {}
            return {
                "cpu_min": self.cpu.min,
                "cpu_avg": self.cpu.total / self.cpu.samples,
                "cpu_max": self.cpu.max,
                "mem_min": self.mem.min,
                "mem_avg": self.mem.total / self.mem.samples,
                "mem_max": self.mem.max,
            }


class DeployedProcess:
    """Base class for managing deployed processes"""

    def __init__(self, process_type: str):
        self.process_type = process_type
        self.stats = ProcessStats()

    def get_stats(self) -> ProcessStats:
        """Get the process stats object for the process"""
        return self.stats

    def start_monitoring(self, interval: float) -> None:
        """Initialize process monitoring"""
        pass

    def stop_monitoring(self) -> None:
        """Stop process monitoring"""
        pass

    def shutdown(self) -> None:
        """Gracefully shutdown the process"""
        pass


class DockerProcess(DeployedProcess):
    """Class to manage Docker processes like containers"""

    def __init__(self, container_id: str):
        super().__init__("docker")
        self.container_id = container_id
        self.monitoring_thread = None
        self.stop_monitoring_event = threading.Event()

    def shutdown(self) -> None:
        """Gracefully shutdown and remove the Docker container"""
        if self.container_id:
            try:
                # First stop the container if it's still running
                print(f"Stopping Docker container: {self.container_id}")
                stop_cmd = ["docker", "stop", self.container_id]
                subprocess.run(stop_cmd, check=True, capture_output=True, text=True)

                # Then remove the container
                print(f"Removing Docker container: {self.container_id}")
                rm_cmd = ["docker", "rm", "-f", self.container_id]
                subprocess.run(rm_cmd, check=True, capture_output=True, text=True)
            except subprocess.CalledProcessError as e:
                print(f"Error stopping/removing Docker container: {e}")
                print(f"Error output: {e.stderr}")

    def start_monitoring(self, interval: float) -> None:
        """
        Monitor Docker container's CPU and memory usage in a background thread using subprocess.

        Args:
            interval: Time in seconds between polling.
        """
        def monitor(container_id: str,
                    stats: ProcessStats,
                    stop_event: threading.Event,
                    interval: float = 1.0):
            while not stop_event.is_set():
                try:
                    cmd = ["docker", "stats", container_id, "--no-stream", "--format",
                        "{{.Container}} {{.CPUPerc}} {{.MemUsage}}"]
                    result = subprocess.check_output(cmd, text=True).strip()
                    if result:
                        parts = result.split()
                        # Example: ['11e99006dc11', '87.54%', '21.84MiB', '/', '7.662GiB']
                        cpu_str = parts[1]
                        cpu = float(cpu_str.strip('%')) / 100.0
                        mem_used_str = parts[2]
                        mem_mb = parse_mem_to_mib(mem_used_str)

                        stats.add_sample(cpu, mem_mb)
                        # Can add a flag to turn this on or off later
                        print(f"Monitored Container ({container_id[:12]}) Cur. CPU: {cpu:.2f} "
                              f"({stats.get_summary_string('cpu')}) Cur Mem: {mem_mb:.2f} "
                              f"({stats.get_summary_string('mem')})")
                        time.sleep(interval)
                except subprocess.CalledProcessError as e:
                    print(f"Error collecting stats for container {container_id}: {e}")
                    break
                except Exception as e:
                    print(f"Unexpected error while monitoring {container_id}: {e}")
                    break
        print(f"Starting montitoring for Docker container: {self.container_id[:12]}")
        monitor_args = {
            "container_id": self.container_id,
            "stats": self.stats,
            "stop_event": self.stop_monitoring_event,
            "interval": interval
        }
        self.monitoring_thread = threading.Thread(target=monitor, kwargs=monitor_args, daemon=True)
        self.monitoring_thread.start()

    def stop_monitoring(self) -> None:
        """Gracefully stop the container monitoring thread"""
        print(f"Stopping montitoring for Docker container: {self.container_id[:12]}")
        self.stop_monitoring_event.set()
        self.monitoring_thread.join()


class K8sDeployedResource(DeployedProcess):
    """Class to manage Kubernetes deployed resources"""

    def __init__(self, deployment_name: str, manifest_path: str, namespace: str = "default"):
        super().__init__("kubernetes")
        self.deployment_name = deployment_name
        self.manifest_path = manifest_path
        self.namespace = namespace
        self.monitoring_thread = None
        self.stop_monitoring_event = threading.Event()

    def shutdown(self) -> None:
        """Delete the Kubernetes resources defined in the manifest"""
        try:
            # Use kubectl delete to remove the resources defined in the manifest
            cmd = ["kubectl", "delete", "-f", self.manifest_path, "-n", self.namespace, "--ignore-not-found=true"]
            subprocess.run(cmd, check=True, capture_output=True, text=True)
            print(f"Deleted Kubernetes deployment '{self.deployment_name}' resources from {self.manifest_path}")
        except subprocess.CalledProcessError as e:
            print(f"Error deleting Kubernetes resources: {e}")
            print(f"Error output: {e.stderr}")

    def start_monitoring(self, interval: float) -> None:
        """
        Monitor a Kubernetes pod's CPU and memory usage in a background thread using subprocess.

        Args:
            interval: Time in seconds between polling.
        """
        def monitor(deployment_name: str,
                    namespace: str,
                    stats: ProcessStats,
                    stop_event: threading.Event,
                    interval: float = 1.0):
            # The approach below relies on kubernetes monitoring-server and kubelet metrics via
            # cAdvisor. These have an update interval on the order of 15 seconds, which is
            # fairly high for short lived tests. We will likely need to evaluate other approaches.
            # Possibly lowering housekeeping interval:
            # https://github.com/google/cadvisor/issues/2660 but that's also not a great solution.
            def get_pod_name():
                # Get the pod name associated with the deployment
                try:
                    cmd = [
                        "kubectl", "get", "pods", "-n", namespace,
                        "-l", f"app={deployment_name}",
                        "-o", "jsonpath={.items[0].metadata.name}"
                    ]
                    pod_name = subprocess.check_output(cmd, text=True).strip()
                    return pod_name
                except subprocess.CalledProcessError as e:
                    print(f"Failed to get pod for deployment {deployment_name}: {e}")
                    return None

            pod_name = get_pod_name()
            if not pod_name:
                print("Pod name could not be determined. Monitoring aborted.")
                return

            while not stop_event.is_set():
                try:
                    cmd = ["kubectl", "top", "pod", pod_name, "-n", namespace, "--no-headers"]
                    result = subprocess.check_output(cmd, text=True).strip()

                    if result:
                        # Example: mypod-abc123 12m 21Mi
                        parts = result.split()
                        cpu_str = parts[1]  # e.g., '12m'
                        mem_str = parts[2]  # e.g., '21Mi'

                        # Convert CPU and memory to standardized formats
                        if cpu_str.endswith('m'):
                            cpu = float(cpu_str.rstrip('m')) / 1000.0  # millicores to cores
                        else:
                            cpu = float(cpu_str)  # cores

                        mem_mib = parse_mem_to_mib(mem_str)

                        stats.add_sample(cpu, mem_mib)
                        print(f"Monitored Pod ({pod_name}) "
                              f"CPU: {cpu:.2f} ({stats.get_summary_string('cpu')}) "
                              f"MEM: {mem_mib:.2f} ({stats.get_summary_string('mem')})")
                except subprocess.CalledProcessError:
                    print(f"Error collecting stats for pod {pod_name}, "
                          "they may take up to 15 seconds to become available...")
                except Exception as e:
                    print(f"Unexpected error while monitoring pod {pod_name}: {e}")
                time.sleep(interval)

        print(f"Starting montitoring for K8S deployment: {self.namespace}/{self.deployment_name}")
        monitor_args = {
            "deployment_name": self.deployment_name,
            "namespace": self.namespace,
            "stats": self.stats,
            "stop_event": self.stop_monitoring_event,
            "interval": interval
        }
        self.monitoring_thread = threading.Thread(target=monitor, kwargs=monitor_args, daemon=True)
        self.monitoring_thread.start()

    def stop_monitoring(self) -> None:
        """Gracefully stop the container monitoring thread"""
        print(f"Stopping montitoring for K8S deployment: {self.namespace}/{self.deployment_name}")
        self.stop_monitoring_event.set()
        self.monitoring_thread.join()

    def wait_until_ready(self, timeout_sec: int = 60) -> bool:
        """
        Wait for a Kubernetes deployment to be ready

        Args:
            timeout_sec: Maximum time to wait in seconds

        Returns:
            bool: True if deployment is ready, False otherwise
        """
        print(f"Waiting for deployment '{self.deployment_name}' to be ready...")

        try:
            cmd = ["kubectl", "rollout", "status", f"deployment/{self.deployment_name}",
                "-n", self.namespace, f"--timeout={timeout_sec}s"]
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            print(f"Deployment '{self.deployment_name}' is ready")
            return True
        except subprocess.CalledProcessError as e:
            print(f"Error or timeout waiting for deployment: {e}")
            print(f"Error output: {e.stderr}")
            return False


def launch_container(
    image_name: str,
    container_name: str,
    ports: Optional[Dict[str, str]] = None,
    network: Optional[str] = None,
    command_args: Optional[List[str]] = None,
    volumes: Optional[Dict[str, str]] = None,
    environment: Optional[Dict[str, str]] = None
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
        environment: Environment variables to pass to the container

    Returns:
        DockerProcess: Object representing the launched container
    """
    print(f"Starting {container_name} service using Docker image: {image_name}...")

    # Construct the Docker command
    cmd = ["docker", "run", "-d"]

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

    # Add environment variables if provided
    if environment:
        for var_name, var_value in environment.items():
            cmd.extend(["-e", f"{var_name}={var_value}"])

    # Add the image name
    cmd.append(image_name)

    # Add any additional command arguments
    if command_args:
        cmd.extend(command_args)

    # Run the Docker container
    try:
        # Start the container and get its ID
        print(f"Running command: {' '.join(cmd)}")
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

def build_docker_image(image_name: str, dockerfile_dir: str) -> str:
    """
    Build a Docker image locally

    Args:
        image_name: Name and tag for the Docker image (e.g. "backend-service:latest")
        dockerfile_dir: Directory containing the Dockerfile

    Returns:
        str: Name of the built image
    """
    print(f"Building Docker image '{image_name}'...")

    # Get the absolute path to the directory
    dir_path = os.path.abspath(dockerfile_dir)

    # Build the Docker image
    try:
        cmd = ["docker", "build", "-t", image_name, dir_path]
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(f"Successfully built Docker image: {image_name}")
        return image_name
    except subprocess.CalledProcessError as e:
        print(f"Error building Docker image '{image_name}': {e}")
        print(f"Error output: {e.stderr}")
        raise

def deploy_kubernetes_resources(manifest_path: str, deployment_name: str, namespace: str = "default") -> K8sDeployedResource:
    """
    Deploy resources to Kubernetes using kubectl apply

    Args:
        manifest_path: Path to the Kubernetes YAML manifest file
        deployment_name: Name of the deployment to be created
        namespace: Kubernetes namespace to deploy to

    Returns:
        K8sDeployedResource: Object representing the deployed resources
    """
    print(f"Deploying '{deployment_name}' to Kubernetes using manifest: {manifest_path}...")

    try:
        # Apply the manifest to create the resources
        cmd = ["kubectl", "apply", "-f", manifest_path, "-n", namespace]
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(f"Successfully deployed '{deployment_name}' resources to Kubernetes")

        # Return a K8sDeployedResource object
        return K8sDeployedResource(
            deployment_name=deployment_name,
            manifest_path=manifest_path,
            namespace=namespace
        )
    except subprocess.CalledProcessError as e:
        print(f"Error deploying Kubernetes resources: {e}")
        print(f"Error output: {e.stderr}")
        raise

def get_docker_logs(container_id: str) -> str:
    """
    Get logs from a Docker container

    Args:
        container_id: ID or name of the Docker container

    Returns:
        str: Container logs or empty string if error
    """
    try:
        cmd = ["docker", "logs", container_id]
        logs = subprocess.check_output(cmd, text=True)
        return logs
    except subprocess.CalledProcessError as e:
        print(f"Error getting Docker container logs: {e}")
        print(f"Error output: {e.stderr if hasattr(e, 'stderr') else 'No error output'}")
        return ""

def get_k8s_logs(pod_selector: str, namespace: str = "default") -> str:
    """
    Get logs from a Kubernetes pod

    Args:
        pod_selector: Label selector to identify the pod (e.g., 'app=loadgen')
        namespace: Kubernetes namespace

    Returns:
        str: Pod logs or empty string if error
    """
    try:
        # First, get the pod name matching the selector
        cmd = ["kubectl", "get", "pods", "-l", pod_selector, "-n", namespace,
               "-o", "jsonpath={.items[0].metadata.name}"]
        pod_name = subprocess.check_output(cmd, text=True).strip()

        if not pod_name:
            print(f"No pod found with selector: {pod_selector}")
            return ""

        # Get logs from the pod
        cmd = ["kubectl", "logs", pod_name, "-n", namespace]
        logs = subprocess.check_output(cmd, text=True)
        return logs
    except subprocess.CalledProcessError as e:
        print(f"Error getting pod logs: {e}")
        print(f"Error output: {e.stderr if hasattr(e, 'stderr') else 'No error output'}")
        return ""

def parse_logs_for_sent_count(logs: str) -> Tuple[int, int]:
    """
    Parse logs to extract the LOADGEN_LOGS_SENT and LOADGEN_LOGS_FAILED counts

    Args:
        logs: Log output from the load generator

    Returns:
        Tuple[int, int]: Number of logs sent and failed, or (0, 0) if not found
    """
    logs_sent = 0
    logs_failed = 0
    if not logs:
        return logs_sent, logs_failed

    for line in logs.strip().split("\n"):
        if "LOADGEN_LOGS_SENT:" in line:
            try:
                logs_sent = int(line.split("LOADGEN_LOGS_SENT:")[1].strip())
            except (IndexError, ValueError) as e:
                print(f"Failed to parse logs sent count: {e}")
        elif "LOADGEN_LOGS_FAILED:" in line:
            try:
                logs_failed = int(line.split("LOADGEN_LOGS_FAILED:")[1].strip())
            except (IndexError, ValueError) as e:
                print(f"Failed to parse logs failed count: {e}")

    if logs_sent == 0:
        print(f"Could not find LOADGEN_LOGS_SENT in logs")
    if logs_failed == 0:
        print(f"Could not find LOADGEN_LLOADGEN_LOGS_FAILEDOGS_FAILED in logs")

    return logs_sent, logs_failed

def run_k8s_loadgen(loadgen_manifest: str, namespace: str, duration: int, k8s_collector_resource: K8sDeployedResource, skip_build: bool = False) -> Tuple[int, int, float]:
    """
    Deploy and run the load generator in Kubernetes and return the counts of logs and duration

    Args:
        loadgen_manifest: Path to the load generator Kubernetes manifest
        namespace: Kubernetes namespace
        duration: Test duration in seconds
        k8s_collector_resource: The collector resource to monitor
        skip_build: Skip building the loadgen image

    Returns:
        Tuple[int, int, float]: Count of logs sent, logs failed, and actual duration
    """
    print("Starting load generator in Kubernetes...")

    # Build the loadgen Docker image if not skipped
    if not skip_build:
        loadgen_image = "otel-loadgen:latest"
        loadgen_image = build_docker_image(loadgen_image, "load_generator")
        print(f"Built loadgen image: {loadgen_image}")

    # Modify the manifest to set the correct duration
    # Read the manifest
    with open(loadgen_manifest, 'r') as f:
        manifest_content = f.read()

    # Replace the {{DURATION}} placeholder with the actual duration
    temp_manifest = f"{loadgen_manifest}.tmp"
    updated_manifest = manifest_content.replace("{{DURATION}}", str(duration))

    print(f"Setting loadgen duration to {duration}s")

    # Write to a temporary file
    with open(temp_manifest, 'w') as f:
        f.write(updated_manifest)

    start_time = time.time()

    # Deploy load generator
    loadgen_resource = deploy_kubernetes_resources(temp_manifest, "otel-loadgen", namespace)

    # Wait for the load generator job to complete
    print(f"Waiting for loadgen job to complete (expected duration: {duration}s)...")

    # Job might take time to get created, so we'll wait a bit before polling
    time.sleep(5)

    # Start monitoring the collector resource, this should all get refactored to avoid
    # relying on sleep in favor of explicit ready / start / stop / error signals.
    k8s_collector_resource.start_monitoring(duration / 10)

    # Poll for job completion
    completed = False
    max_wait = duration + 30  # Add buffer time
    wait_start = time.time()

    while not completed and (time.time() - wait_start) < max_wait:
        try:
            cmd = ["kubectl", "get", "job", "otel-loadgen", "-n", namespace,
                  "-o", "jsonpath={.status.succeeded}"]
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            if result.stdout.strip() == "1":
                completed = True
                print("Load generator job completed successfully")
                break
            time.sleep(5)  # Check every 5 seconds
        except subprocess.CalledProcessError:
            time.sleep(5)  # Continue checking

    # High likelyhood that this will have accumulated samples from the idle collector after the job finished.
    # Better handling for component start/stop + test start/stop will be important in future PRs.
    k8s_collector_resource.stop_monitoring()

    if not completed:
        print("Warning: Load generator job didn't complete in the expected time, getting logs anyway")

    # Get logs from the loadgen pod
    logs = get_k8s_logs("app=loadgen", namespace)

    # Use the expected duration since the actual might be hard to determine precisely
    actual_duration = duration

    # Parse the output to extract logs sent count
    logs_sent, logs_failed = parse_logs_for_sent_count(logs)

    if logs_sent == 0:
        print(f"Could not find LOADGEN_LOGS_SENT in logs")
    else:
        print(f"Load generator completed. Sent {logs_sent} logs, Failed {logs_failed} logs in {actual_duration:.2f}s")

    # Clean up the temporary manifest
    try:
        os.remove(temp_manifest)
    except:
        pass

    return logs_sent, logs_failed, actual_duration

def parse_mem_to_mib(mem_str: str) -> float:
    """Parse the string returned by docker stats to a float representing the number of MiB in use by the container."""
    units_to_mib = {
        "kib": 1 / 1024,                   # 1 KiB = 1/1024 MiB
        "kb": 1000 / 1024 / 1024,          # 1 KB = 1000 bytes convert to MiB
        "mb": 1000000 / 1024 / 1024,       # 1 MB = 1,000,000 bytes convert to MiB
        "mi": 1,                           # already MiB
        "mib": 1,                          # already MiB
        "gb": 1_000_000_000 / 1024 / 1024, # 1 GB = 1,000,000,000 bytes MiB
        "gib": 1024                        # 1 GiB = 1024 MiB
    }
    match = re.match(r"([0-9.]+)([a-zA-Z]+)", mem_str)
    if not match:
        return 0.0
    value, unit = match.groups()
    return float(value) * units_to_mib.get(unit.lower(), 1)

def get_backend_received_count(url: str) -> int:
    """
    Query the backend service in Kubernetes for the count of received logs.

    Args:
        url: The URL of the backend service metrics endpoint.

    Returns:
        int: The count of received logs, or -1 if the query fails.
    """
    print(f"\nQuerying backend service at {url} for received count...")

    try:
        # Query the metrics endpoint
        response = requests.get(url, timeout=10)
        data = response.json()
        count = data.get("received_logs", -1)
        return count
    except Exception as e:
        print(f"Failed to query backend service: {e}")
        return -1

def setup_k8s_port_forwarding(service_name: str, namespace: str, local_port: int, remote_port: int) -> subprocess.Popen:
    """
    Set up port forwarding from a local port to a service in Kubernetes.

    Args:
        service_name: Name of the Kubernetes service to forward to
        namespace: Kubernetes namespace
        local_port: Local port to forward from
        remote_port: Remote port in the service to forward to

    Returns:
        subprocess.Popen: The process object for the port forwarding command
    """
    print(f"Setting up port forwarding from localhost:{local_port} to {service_name}:{remote_port} in namespace {namespace}...")

    # Build the kubectl port-forward command
    cmd = [
        "kubectl", "port-forward",
        f"service/{service_name}",
        f"{local_port}:{remote_port}",
        "-n", namespace
    ]

    # Start the port forwarding in a subprocess
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    # Give it a moment to establish the connection
    time.sleep(2)

    # Check if the process is still running
    if process.poll() is not None:
        # Process terminated prematurely
        stdout, stderr = process.communicate()
        print(f"Port forwarding failed to start: {stderr}")
        return None

    print(f"Port forwarding established successfully")
    return process

def create_k8s_namespace(namespace: str) -> bool:
    """
    Create a Kubernetes namespace if it doesn't exist

    Args:
        namespace: Name of the namespace to create

    Returns:
        bool: True if successful, False otherwise
    """
    print(f"Ensuring Kubernetes namespace exists: {namespace}")

    try:
        # Check if the namespace already exists
        check_cmd = ["kubectl", "get", "namespace", namespace, "--no-headers", "--ignore-not-found"]
        result = subprocess.run(check_cmd, check=True, capture_output=True, text=True)

        if namespace in result.stdout:
            print(f"Namespace {namespace} already exists")
            return True

        # Create the namespace
        create_cmd = ["kubectl", "create", "namespace", namespace]
        result = subprocess.run(create_cmd, check=True, capture_output=True, text=True)
        print(f"Created Kubernetes namespace: {namespace}")
        return True

    except subprocess.CalledProcessError as e:
        print(f"Error creating Kubernetes namespace: {e}")
        print(f"Error output: {e.stderr}")
        return False

def cleanup_docker_containers(container_names: List[str]) -> None:
    """
    Remove any existing Docker containers with the specified names

    Args:
        container_names: List of container names to clean up
    """
    print(f"Cleaning up any existing Docker containers with names: {container_names}")

    for name in container_names:
        try:
            # Check if the container exists
            inspect_cmd = ["docker", "ps", "-a", "-q", "-f", f"name={name}"]
            container_id = subprocess.run(inspect_cmd, check=True, capture_output=True, text=True).stdout.strip()

            if container_id:
                # Container exists, remove it forcefully
                print(f"Removing existing container: {name} (ID: {container_id})")
                rm_cmd = ["docker", "rm", "-f", name]
                subprocess.run(rm_cmd, check=True, capture_output=True, text=True)
        except subprocess.CalledProcessError as e:
            print(f"Error checking/removing container '{name}': {e}")
            # Continue with other containers

# Pre-requisites:
# 1. Create and activate a virtual environment:
#    python3 -m venv venv
#    source venv/bin/activate  # On Windows use: venv\Scripts\activate
# 2. Install dependencies:
#    pip install -r orchestrator/requirements.txt
# 3. Run the orchestrator with Docker:
#    python3 orchestrator/orchestrator.py --collector-config system_under_test/otel-collector/collector-config.yaml --duration 30
# 4. Run with Kubernetes (currently requires kubernetes metrics-server):
#    python3 orchestrator/orchestrator.py --deployment-target kubernetes --k8s-collector-manifest system_under_test/otel-collector/collector-manifest.yaml --k8s-backend-manifest backend/backend-manifest.yaml --k8s-loadgen-manifest load_generator/loadgen-manifest.yaml --k8s-namespace perf-test-otel --duration 30
def main():
    parser = argparse.ArgumentParser(description="Orchestrate OTel pipeline perf test")
    parser.add_argument("--duration", type=int, default=10, help="Duration to perform perf test in seconds")
    parser.add_argument("--keep-resources", action="store_true", help="Don't delete resources after test. Useful for debugging.")
    parser.add_argument("--results-dir", type=str, default="./results", help="Directory to store test results")

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
    results_file = os.path.join(args.results_dir, f"perf_results_{timestamp}.txt")

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

        if args.deployment_target == "docker":
            # Clean up any existing containers with the same names we'll use
            cleanup_docker_containers(["backend-service", "otel-collector", "otel-loadgen"])

            # Docker deployment flow
            # Create a Docker network for inter-container communication
            network = "perf-test-network"
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

            # Build the backend Docker image if not skipped
            backend_image = "backend-service:latest"
            if not args.skip_backend_build:
                backend_image = build_docker_image(backend_image, "backend")
            else:
                print(f"Using existing backend image: {backend_image}")

            # Launch the backend service as a Docker container
            backend_process = launch_container(
                image_name=backend_image,
                container_name="backend-service",
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
            collector_image = "otel/opentelemetry-collector:latest"
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

            # Build the loadgen Docker image if not skipped
            loadgen_image = "otel-loadgen:latest"
            if not args.skip_loadgen_build:
                loadgen_image = build_docker_image(loadgen_image, "load_generator")

            # Run the load generator using Docker
            print("Starting load generator using Docker...")
            loadgen_env = {
                "OTLP_ENDPOINT": "otel-collector:4317"  # Use Docker network DNS name
            }
            loadgen_process = launch_container(
                image_name=loadgen_image,
                container_name="otel-loadgen",
                network=network,
                environment=loadgen_env,
                command_args=["--duration", str(args.duration)]
            )

            # Start monitoring once the load generator is built and launched
            # Set statically to ensure ~10 samples for now
            target_process.start_monitoring(args.duration/10)

            # Wait for the loadgen container to finish (it runs for the specified duration)
            print(f"Waiting for load generator to finish (running for {args.duration}s)...")

            try:
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
                logs = get_docker_logs(loadgen_process.container_id)

                # Parse the output to extract logs sent count and failed count
                logs_sent_count, logs_failed_count = parse_logs_for_sent_count(logs)

                if logs_sent_count > 0:
                    print(f"Load generator completed. Sent {logs_sent_count} logs, Failed {logs_failed_count} logs")

                # Use the specified duration as the actual duration
                duration = args.duration
            except subprocess.CalledProcessError as e:
                print(f"Error getting logs from load generator container: {e}")
                logs_sent_count = -1
                logs_failed_count = 0
                duration = args.duration

            logs_received_backend_count = get_backend_received_count("http://localhost:5000/metrics")
        else:
            # Build the backend Docker image if not skipped
            backend_image = "backend-service:latest"
            if not args.skip_backend_build:
                backend_image = build_docker_image(backend_image, "backend")
            else:
                print(f"Using existing backend image: {backend_image}")

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
            logs_sent_count, logs_failed_count, duration = run_k8s_loadgen(
                args.k8s_loadgen_manifest,
                args.k8s_namespace,
                args.duration,
                k8s_collector_resource,
                args.skip_loadgen_build
            )

            target_process_stats = k8s_collector_resource.get_stats()

            # Query backend for received count (using port forwarding set up earlier)
            logs_received_backend_count = get_backend_received_count("http://localhost:5000/metrics")

        # Calculate total logs lost (including those that failed at loadgen side and those lost in transit)
        # Logs that failed at loadgen side
        loadgen_failed = logs_failed_count

        # Logs lost in transit (difference between successful sends and received count)
        transit_lost = logs_sent_count - logs_received_backend_count

        # Total logs lost includes both loadgen failures and transit losses
        total_logs_lost = loadgen_failed + transit_lost

        # Calculate logs sent rate (based on attempted sends, not successful ones)
        total_logs_attempted = logs_sent_count + logs_failed_count
        logs_sent_rate = total_logs_attempted / duration if duration > 0 else 0

        # Format rate for human readability (K/sec or M/sec)
        if logs_sent_rate >= 1000000:
            formatted_rate = f"{logs_sent_rate/1000000:.2f}M/sec"
        elif logs_sent_rate >= 1000:
            formatted_rate = f"{logs_sent_rate/1000:.2f}K/sec"
        else:
            formatted_rate = f"{logs_sent_rate:.2f}/sec"

        # Calculate percentage of logs lost
        logs_lost_percentage = (total_logs_lost / total_logs_attempted * 100) if total_logs_attempted > 0 else 0

        # Print results
        print(f"Total logs attempted: {total_logs_attempted}")
        print(f"Logs successfully sent by loadgen: {logs_sent_count}")
        print(f"Logs failed at loadgen: {logs_failed_count}")
        print(f"Logs received by backend: {logs_received_backend_count}")
        print(f"Logs lost in transit: {transit_lost}")
        print(f"Duration: {duration:.2f} seconds")
        print(f"Logs attempt rate: {formatted_rate} ({logs_sent_rate:.2f} logs/second)")
        print(f"Total logs lost: {total_logs_lost} ({logs_lost_percentage:.2f}% of attempted logs)")
        if target_process_stats:
            print(f"CPU min/avg/max: {target_process_stats.get_summary_string('cpu')}")
            print(f"Memory min/avg/max: {target_process_stats.get_summary_string('mem')}")

        # Write results to file
        with open(results_file, "w") as f:
            f.write(f"Performance test run at: {timestamp}\n")
            f.write(f"Test duration: {args.duration} seconds\n")
            f.write(f"Deployment target: {args.deployment_target}\n")
            if args.deployment_target == "docker":
                f.write(f"Collector config: {args.collector_config}\n\n")
            else:
                f.write(f"Kubernetes manifests:\n")
                f.write(f"- Collector: {args.k8s_collector_manifest}\n")
                f.write(f"- Backend: {args.k8s_backend_manifest}\n")
                f.write(f"- LoadGen: {args.k8s_loadgen_manifest}\n\n")
            f.write("Results:\n")
            f.write(f"- Total logs attempted: {total_logs_attempted}\n")
            f.write(f"- Logs successfully sent by loadgen: {logs_sent_count}\n")
            f.write(f"- Logs failed at loadgen: {logs_failed_count}\n")
            f.write(f"- Logs received by backend: {logs_received_backend_count}\n")
            f.write(f"- Logs lost in transit: {transit_lost}\n")
            f.write(f"- Duration: {duration:.2f} seconds\n")
            f.write(f"- Logs attempt rate: {formatted_rate} ({logs_sent_rate:.2f} logs/second)\n")
            f.write(f"- Total logs lost: {total_logs_lost} (failed at loadgen + lost in transit)\n")
            f.write(f"- Percentage of logs lost: {logs_lost_percentage:.2f}%\n")
            if target_process_stats:
                f.write(f"- CPU min/avg/max: {target_process_stats.get_summary_string('cpu')}\n")
                f.write(f"- Memory min/avg/max: {target_process_stats.get_summary_string('mem')}\n")

        print(f"Test completed. Results saved to {results_file}")

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
                    try:
                        subprocess.run(["docker", "network", "rm", "perf-test-network"], check=True, capture_output=True)
                    except subprocess.CalledProcessError as e:
                        print(f"Error removing Docker network: {e}")
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
