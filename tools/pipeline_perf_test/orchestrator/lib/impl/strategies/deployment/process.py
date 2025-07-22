"""
Process-Based Deployment Strategy Module.

This module defines a deployment strategy for running components as operating system
subprocesses. It provides configuration models, runtime tracking structures, and logic to
start and stop processes using Python's `subprocess` module.

Key Components:
- `ComponentProcessRuntime`: A Pydantic model that stores runtime metadata for a process,
  including its PID, `Popen` handle, and captured stdout/stderr logs.
- `ProcessDeploymentConfig`: A configuration model specifying the command to execute and
  optional environment variables.
- `ProcessDeployment`: A concrete deployment strategy that implements how to launch and
  terminate a process as part of the component lifecycle.

Features:
- Environment variable support via both `dict` and `list[str]` formats.
- Graceful process termination with a fallback to force kill if needed.
- Capture and optional debug logging of stdout and stderr output from the process.

This strategy is registered under the name `"process"` and integrates with the component
framework and step-level test execution context.

YAML Usage Example:
components:
  otel-collector:
    deployment:
      process:
        command: python -m ./load_generator/loadgen.py --serve
        environment: {}
"""
import os
import subprocess

from typing import ClassVar, Dict, List, Literal, Optional, Union

from pydantic import BaseModel, ConfigDict

from ....core.component.component import (
    Component,
)
from ....core.context.framework_element_contexts import StepContext
from ....core.strategies.deployment_strategy import (
    DeploymentStrategy,
    DeploymentStrategyConfig,
)
from ....runner.registry import deployment_registry, PluginMeta


STRATEGY_NAME = "process"


class ComponentProcessRuntime(BaseModel):
    """Base Model for component process runtime information."""

    type: ClassVar[Literal["component_process_runtime"]] = "component_process_runtime"
    pid: Optional[int] = None
    process: Optional[subprocess.Popen[bytes]] = None
    std_out_logs: Optional[list[str]] = None
    std_err_logs: Optional[list[str]] = None
    # Support Popen[bytes]
    model_config = ConfigDict(arbitrary_types_allowed=True)


@deployment_registry.register_config(STRATEGY_NAME)
class ProcessDeploymentConfig(DeploymentStrategyConfig):
    """
    Configuration model for deploying a component as a child process.

    Attributes:
        command (str): Command used to run the process.
        environment (Optional[Union[Dict[str, str], List[str]]]): Environment variables to
            set for the process, either as a dictionary of key-value pairs or a list of
            strings in 'KEY=VALUE' format.
    """

    command: str
    environment: Optional[Union[Dict[str, str], List[str]]] = None


@deployment_registry.register_class(STRATEGY_NAME)
class ProcessDeployment(DeploymentStrategy):
    """
    Deployment strategy to manage the lifecycle of components as a sub-process.

    This class handles starting and stopping processes based on the given
    deployment configuration.

    Methods:
        start(component: Component, ctx: StepContext):
            Starts a process in a thread for the specified component using the deployment
            configuration.

        stop(component: Component, ctx: StepContext):
            Stops and removes the process associated with the component, using
            process ID, and thread stored in the component runtime. Raises errors if process
            cannot be found or stopped.
    """

    type: ClassVar[Literal["process"]] = "process"
    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
components:
  otel-collector:
    deployment:
      process:
        command: python -m ./load_generator/loadgen.py --serve
        environment: {}
""",
    )

    def __init__(self, config: ProcessDeploymentConfig):
        """Initialize the strategy and specify default hooks to register."""
        self.config = config
        self.default_component_hooks = {}
        self.stop_event = None

    def start(self, component: Component, ctx: StepContext):
        """Start a process based on the provided configuration.

        Args:
            component: the component invoking this strategy.
            context: the current execution context.

        Raises:
            ValueError: on incompatible configuration value.
            TypeError: on incompatilbe configuration type.
        """

        logger = ctx.get_logger(__name__)
        logger.debug(f"Starting process for {component.name}")
        process_runtime: ComponentProcessRuntime = component.get_or_create_runtime(
            ComponentProcessRuntime.type, ComponentProcessRuntime
        )

        # Prepare environment
        env = os.environ.copy()
        if isinstance(self.config.environment, dict):
            env.update(self.config.environment)
        elif isinstance(self.config.environment, list):
            for item in self.config.environment:
                key, _, value = item.partition("=")
                if key and value:
                    env[key] = value

        # Start the subprocess
        logger.debug(f"Launching command: {self.config.command}")
        process = subprocess.Popen(
            self.config.command,
            shell=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
        )

        # Store the process object and pid in the runtime
        process_runtime.pid = process.pid

        # Store the process in the runtime
        process_runtime.process = process
        component.set_runtime_data(ComponentProcessRuntime.type, process_runtime)

        logger.debug(
            f"Process for {component.name} started with PID: {process_runtime.pid}"
        )

    def stop(self, component: Component, ctx: StepContext):
        """Stop a process running in a background thread.

        Args:
            component: the component invoking this strategy.
            ctx: the current execution context.

        Raises:
            RuntimeError: if process thread or stop event is missing.
        """
        logger = ctx.get_logger(__name__)
        runtime: ComponentProcessRuntime = component.get_or_create_runtime(
            ComponentProcessRuntime.type, ComponentProcessRuntime
        )

        if not runtime.process:
            logger.warning(f"No process found for component '{component.name}'")
            raise RuntimeError("No running process found for component.")

        logger.debug(f"Stopping process for {component.name}, with PID: {runtime.pid}")

        process = runtime.process
        try:
            # Attempt to terminate the process gracefully
            process.terminate()
            try:
                # Wait for process to terminate with a timeout
                process.wait(timeout=5)
                logger.info(f"Process for {component.name} terminated successfully.")
            except subprocess.TimeoutExpired:
                logger.warning(
                    f"Process for {component.name} did not terminate, killing it."
                )
                process.kill()  # Force kill if terminate fails

            stdout_logs, stderr_logs = process.communicate()
            args = ctx.get_suite().get_runtime("args")
            if stdout_logs:
                decoded = (
                    stdout_logs.decode("utf-8")
                    if isinstance(stdout_logs, bytes)
                    else str(stdout_logs)
                )
                runtime.std_out_logs = decoded.splitlines()
                if args.debug:
                    logger.debug("Process std out For %s:\n%s", component.name, decoded)
            if stderr_logs:
                decoded = (
                    stderr_logs.decode("utf-8")
                    if isinstance(stderr_logs, bytes)
                    else str(stderr_logs)
                )
                runtime.std_err_logs = decoded.splitlines()
                if args.debug:
                    logger.debug("Process std err For %s:\n%s", component.name, decoded)

        except Exception as e:
            logger.exception(f"Error stopping process for {component.name}: {e}")

        # Clear runtime info
        runtime.process = None
        runtime.pid = None
        component.set_runtime_data(ComponentProcessRuntime.type, runtime)
