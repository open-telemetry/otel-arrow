"""
base.py

Defines the abstract base class `DeployedProcess` for managing deployed system components
(e.g., backend services, load generators, or collectors) in a performance testing environment.

This class serves as the foundation for concrete implementations that manage specific deployment
targets or environments (e.g., Docker containers, Kubernetes pods). It encapsulates shared
functionality  such as process monitoring and statistics collection via the `ProcessStats` utility.

Key Components:
- `process_type`: A string identifier for the type of deployed process.
- `stats`: A `ProcessStats` object for tracking resource usage and performance metrics.
- `start_monitoring(interval)`: Begin monitoring the process at a specified time interval (to be
    implemented by subclasses).
- `stop_monitoring()`: Stop the monitoring process (to be implemented by subclasses).
- `shutdown()`: Gracefully shut down the deployed process (to be implemented by subclasses).

Intended to be subclassed with environment-specific logic for process lifecycle management.
"""
from ..stats import ProcessStats

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
