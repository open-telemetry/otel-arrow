"""
Hook strategy to print Docker container logs for all components.

This hook collects and prints container logs from all components that have
been deployed via Docker. It's designed to run at the end of test execution
to provide a consolidated view of all container output.
"""

from typing import Optional

from .....core.context.base import BaseContext, ExecutionStatus
from .....core.context import StepContext
from .....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from .....runner.registry import hook_registry, PluginMeta
from ...common.docker import (
    ComponentDockerRuntime,
)


@hook_registry.register_config("print_container_logs")
class PrintContainerLogsConfig(HookStrategyConfig):
    """
    Configuration for the 'print_container_logs' hook.

    Attributes:
        components (Optional[list[str]]): List of component names to print logs for.
            If None, prints logs for all components with Docker runtime data.
        separator (Optional[str]): Separator line to use between container logs.
            Defaults to a line of equals signs.
    """

    components: Optional[list[str]] = None
    separator: Optional[str] = "=" * 80


@hook_registry.register_class("print_container_logs")
class PrintContainerLogs(HookStrategy):
    """
    Hook strategy to print Docker container logs for all deployed components.

    This hook iterates through all components in the test suite, retrieves their
    stored container logs (if any), and prints them in an organized manner. This
    provides a consolidated view of all container output at the end of test execution.

    Features:
        - Prints logs for all components or a specified subset
        - Clearly separates logs by component name
        - Handles cases where no logs are available
        - Non-blocking: always returns SUCCESS status

    Typical usage:
        Add as a post-hook after test reports to print all container diagnostics:

        ```yaml
        - name: Run Report
          action:
            wait:
              delay_seconds: 0
          hooks:
            run:
              post:
                - sql_report: ...
                - print_container_logs: {}
        ```
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
  - name: Print Container Logs
    action:
      wait:
        delay_seconds: 0
    hooks:
      run:
        post:
          - print_container_logs: {}
          # Or specify specific components:
          # - print_container_logs:
          #     components: [backend-service, df-engine]
""",
    )

    def __init__(self, config: PrintContainerLogsConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        # Get all components from the suite
        suite = ctx.get_suite()
        components = suite.components

        # Filter components if specified in config
        if self.config.components:
            components = {
                name: comp
                for name, comp in components.items()
                if name in self.config.components
            }

        # Collect all logs first
        logs_output = []
        logs_output.append(f"\n{self.config.separator}")
        logs_output.append("DOCKER CONTAINER LOGS")
        logs_output.append(f"{self.config.separator}")

        printed_any = False

        # Iterate through components and collect their logs
        for component_name, component in components.items():
            # Try to get Docker runtime data
            runtime = component.runtime.get(ComponentDockerRuntime.type)

            if runtime and isinstance(runtime, ComponentDockerRuntime):
                if runtime.container_logs:
                    printed_any = True
                    logs_output.append(f"\n{self.config.separator}")
                    logs_output.append(f"Component: {component_name}")
                    logs_output.append(f"Container ID: {runtime.container_id}")
                    logs_output.append(f"{self.config.separator}")

                    # Add the actual logs
                    logs_output.extend(runtime.container_logs)

        if not printed_any:
            logs_output.append("\nNo container logs found.")

        logs_output.append(f"\n{self.config.separator}")
        logs_output.append("END OF CONTAINER LOGS")
        logs_output.append(f"{self.config.separator}\n")

        # Print everything as a single block
        print("\n".join(logs_output))

        return ExecutionStatus.SUCCESS
