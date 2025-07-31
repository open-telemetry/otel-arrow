"""
Execution strategy module for the 'pipeline_perf_loadgen' load generator.

This strategy controls a pipeline performance load generator by sending HTTP requests
to a specified load generation service. It allows configuring parameters such as
the number of threads, request body size, attribute counts, and batch sizes to
simulate realistic load during test execution.

Classes:
    - PipelinePerfLoadgenConfig: Configuration model defining the load generation
      parameters and target service endpoint.
    - PipelinePerfLoadgenExecution: Execution strategy that starts and stops the
      load generator by invoking HTTP API endpoints.

Usage:
    This strategy is registered under the name 'pipeline_perf_loadgen' and can be
    referenced in component execution strategies.

Typical YAML Example:

```yaml
components:
  load-generator:
    execution:
      pipeline_perf_loadgen:
        endpoint:  "http://localhost:5001/"
        threads: 1
        target_rate: 10000
        body_size: 25
        num_attributes: 2
        attribute_value_size: 15
        batch_size: 10000
```

Raises:
    - Requests HTTPError if start or stop requests fail.
"""

from typing import Optional, ClassVar, Literal
from urllib.parse import urljoin

import requests

from ....core.component import Component
from ....core.context import StepContext
from ....core.strategies.execution_strategy import (
    ExecutionStrategy,
    ExecutionStrategyConfig,
)
from ....runner.registry import execution_registry, PluginMeta


STRATEGY_NAME = "pipeline_perf_loadgen"


@execution_registry.register_config(STRATEGY_NAME)
class PipelinePerfLoadgenConfig(ExecutionStrategyConfig):
    """
    Configuration schema for the pipeline performance load generator execution strategy.

    Attributes:
        endpoint (Optional[str]): Base URL of the load generator service. Defaults to
            "http://localhost:5001/".
        threads (Optional[int]): Number of concurrent threads to simulate. Defaults to 1.
        target_rate (Optional[int]): Number of messages / sec to target. Defaults to None.
        body_size (Optional[int]): Size of the request body payload. Defaults to 25.
        num_attributes (Optional[int]): Number of attributes included in each load event.
            Defaults to 2.
        attribute_value_size (Optional[int]): Size of each attribute's value. Defaults to 15.
        batch_size (Optional[int]): Number of events sent in each batch. Defaults to 10000.
        tcp_connection_per_thread(Optional[bool]): Use a dedicated tcp connection per-thread.
    """

    endpoint: Optional[str] = "http://localhost:5001/"
    threads: Optional[int] = 1
    target_rate: Optional[int] = None
    body_size: Optional[int] = 25
    num_attributes: Optional[int] = 2
    attribute_value_size: Optional[int] = 15
    batch_size: Optional[int] = 10000
    tcp_connection_per_thread: Optional[bool] = True


@execution_registry.register_class(STRATEGY_NAME)
class PipelinePerfLoadgenExecution(ExecutionStrategy):
    """
    Execution strategy implementation for controlling the pipeline performance load generator.

    This strategy starts and stops the load generator by issuing HTTP POST requests
    to designated 'start' and 'stop' endpoints.

    Attributes:
        type (ClassVar[Literal["pipeline_perf_loadgen"]]): Identifier for this strategy.
        config (PipelinePerfLoadgenConfig): Configuration instance with load parameters.
        default_hooks (dict): Placeholder for lifecycle hooks (empty by default).
        start_endpoint (str): Fully qualified URL for starting the load generator.
        stop_endpoint (str): Fully qualified URL for stopping the load generator.

    Methods:
        start(component, ctx): Sends a POST request to start load generation with configured parameters.
        stop(component, ctx): Sends a POST request to stop load generation.
    """

    type: ClassVar[Literal["pipeline_perf_loadgen"]] = "pipeline_perf_loadgen"
    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
components:
  load-generator:
    execution:
      pipeline_perf_loadgen:
        endpoint:  "http://localhost:5001/"
        threads: 1
        target_rate: 10000
        tcp_connection_per_thread: false
        body_size: 25
        num_attributes: 2
        attribute_value_size: 15
        batch_size: 10000
""",
    )

    def __init__(self, config: PipelinePerfLoadgenConfig):
        """
        Initializes the execution strategy with the provided configuration.

        Args:
            config (PipelinePerfLoadgenConfig): Configuration for load generation.
        """
        self.config = config
        self.default_hooks = {}
        self.start_endpoint = urljoin(config.endpoint, "start")
        self.stop_endpoint = urljoin(config.endpoint, "stop")

    def start(self, _component: Component, ctx: StepContext):
        """
        Starts the load generator by sending a POST request with load parameters.

        Args:
            _component (Component): The component instance (unused).
            ctx (StepContext): The current execution context for logging.

        Raises:
            requests.HTTPError: If the HTTP request to start the load generator fails.
        """
        logger = ctx.get_logger(__name__)
        resp = requests.post(
            self.start_endpoint,
            json={
                "threads": self.config.threads,
                "body_size": self.config.body_size,
                "target_rate": self.config.target_rate,
                "num_attributes": self.config.num_attributes,
                "attribute_value_size": self.config.attribute_value_size,
                "batch_size": self.config.batch_size,
                "tcp_connection_per_thread": self.config.tcp_connection_per_thread
            },
            timeout=60,
        )
        resp.raise_for_status()
        logger.debug(f"Got response from loadgen start: {resp.text}")

    def stop(self, _component: Component, ctx: StepContext):
        """
        Stops the load generator by sending a POST request to the stop endpoint.

        Args:
            _component (Component): The component instance (unused).
            ctx (StepContext): The current execution context for logging.

        Raises:
            requests.HTTPError: If the HTTP request to stop the load generator fails.
        """
        logger = ctx.get_logger(__name__)
        resp = requests.post(self.stop_endpoint, timeout=60)
        resp.raise_for_status()
        logger.debug(f"Got response from loadgen stop: {resp.text}")
