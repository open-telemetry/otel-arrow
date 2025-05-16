"""
docker.py

Defines the `DockerProcess` class, a concrete implementation of `DeployedProcess` for managing
Docker-based components within a performance testing environment.

This class encapsulates the lifecycle and monitoring logic for processes running inside Docker
containers, such as load generators, backends, or telemetry collectors. It tracks resource
usage and enables clean shutdown of containers under test.

Key Features:
- Inherits from `DeployedProcess` and specializes it for Docker environments.
- Accepts a Docker `container_id` to identify and interact with the running container.
- Supports background monitoring using a thread and a stoppable event.
- Integrates with `ProcessStats` to record runtime performance metrics.

Intended to be used by the orchestration layer to deploy, monitor, and manage containerized
components.
"""
import threading
import time

import docker
from docker.errors import NotFound, APIError

from ..stats import ProcessStats
from .base import DeployedProcess

class DockerProcess(DeployedProcess):
    """Class to manage Docker processes like containers"""

    def __init__(self, container_id: str, client: docker.DockerClient, log_cli: bool=False):
        super().__init__("docker")
        self.container_id = container_id
        self.monitoring_thread = None
        self.stop_monitoring_event = threading.Event()
        self.log_cli = log_cli
        self._client = client

    def shutdown(self) -> None:
        """Gracefully shutdown and remove the Docker container"""
        if self.container_id:
            try:
                container = self._client.containers.get(self.container_id)

                print(f"Stopping Docker container: {self.container_id}")
                if self.log_cli:
                    print(f"CLI_COMMAND = docker stop -t 10 {self.container_id}")
                container.stop(timeout=10)  # default is 10 seconds

                print(f"Removing Docker container: {self.container_id}")
                if self.log_cli:
                    print(f"CLI_COMMAND = docker rm -f {self.container_id}")
                container.remove(force=True)
            except NotFound:
                print(f"Container {self.container_id} not found. It may have already been removed.")
            except APIError as e:
                print(f"Error stopping/removing Docker container: {e}")

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
            try:
                container = self._client.containers.get(container_id)
            except APIError as e:
                print(f"Could not retrieve container {container_id}: {e}")
                return

            while not stop_event.is_set():
                try:
                    stat_data = container.stats(stream=False)

                    # CPU usage calculation
                    cpu_stats = stat_data['cpu_stats']
                    precpu_stats = stat_data['precpu_stats']
                    cpu_delta = cpu_stats['cpu_usage']['total_usage'] - precpu_stats['cpu_usage']['total_usage']
                    system_delta = cpu_stats['system_cpu_usage'] - precpu_stats['system_cpu_usage']

                    cpu_usage = 0.0
                    if system_delta > 0.0 and cpu_delta > 0.0:
                        num_cpus = len(cpu_stats['cpu_usage'].get('percpu_usage', [])) or cpu_stats['online_cpus']
                        cpu_usage = (cpu_delta / system_delta) * num_cpus

                    # Memory usage in MiB
                    mem_usage = stat_data['memory_stats']['usage']
                    mem_mb = mem_usage / (1024 * 1024)

                    stats.add_sample(cpu_usage, mem_mb)
                    print(f"Monitored Container ({container_id[:12]}) Cur. CPU (#Cores): {cpu_usage:.2f} "
                          f"({stats.get_summary_string('cpu')}) Cur Mem (MiB): {mem_mb:.2f} "
                          f"({stats.get_summary_string('mem')})")

                    time.sleep(interval)

                except APIError as e:
                    print(f"Error collecting stats for container {container_id}: {e}")
                    break
                except Exception as e:
                    print(f"Unexpected error while monitoring {container_id}: {e}")
                    break

        print(f"Starting monitoring for Docker container: {self.container_id[:12]}")
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
        print(f"Stopping monitoring for Docker container: {self.container_id[:12]}")
        self.stop_monitoring_event.set()
        self.monitoring_thread.join()
