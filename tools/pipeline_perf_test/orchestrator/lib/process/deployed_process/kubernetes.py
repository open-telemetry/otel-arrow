"""
kubernetes.py

Defines the `K8sDeployedResource` class, a concrete implementation of `DeployedProcess` for managing
components deployed in a Kubernetes cluster as part of a performance testing pipeline.

This class provides an abstraction for handling the deployment, monitoring, and shutdown of
Kubernetes  resources such as load generators, backends, and OpenTelemetry collectors.
It leverages Kubernetes manifests  to manage resource definitions and integrates with `ProcessStats`
for performance data collection.

Key Features:
- Inherits from `DeployedProcess` and specializes it for Kubernetes deployments.
- Accepts the deployment name, manifest file path, and target namespace for managing the resource.
- Intended to support monitoring and lifecycle management of a Kubernetes-deployed process.
- Integrates with orchestration logic to provide parity with Docker-based deployment workflows.

Used by the orchestration framework to manage and test components in Kubernetes environments.
"""
import subprocess
import threading
import time

from ..stats import ProcessStats, parse_mem_to_mib
from .base import DeployedProcess

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

        print(f"Starting monitoring for K8S deployment: {self.namespace}/{self.deployment_name}")
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
        print(f"Stopping monitoring for K8S deployment: {self.namespace}/{self.deployment_name}")
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

