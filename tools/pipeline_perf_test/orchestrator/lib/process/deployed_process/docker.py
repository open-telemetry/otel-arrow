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
import subprocess
import threading
import time

from ..stats import ProcessStats, parse_mem_to_mib
from .base import DeployedProcess

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
