"""
Module defining a step action that introduces a delay during test execution.

This module provides a simple mechanism to pause a test step for a specified number of seconds,
which can be useful for timing control, waiting for external processes, or creating intentional
delays between test phases.

Typical YAML configuration example:

```yaml
tests:
  - name: Test Max Rate Logs
    steps:
    - name: Wait For Test
      action:
        wait:
          delay_seconds: 3
```

Classes:
    - WaitActionConfig: Configuration class that specifies the delay duration in seconds.
    - WaitAction: Step action that performs a blocking wait based on the configured delay.

The action is registered in the test step action registry under the name `"wait"`.

Usage:
    Include this action in a test step to introduce a time-based pause.

Example:
    delay_seconds: 2.5  # Waits for 2.5 seconds when the step is executed
"""
import time

from ...core.context import StepContext
from ...core.framework.step import StepActionConfig, StepAction
from ...runner.registry import test_step_action_registry, PluginMeta


ACTION_NAME = "wait"


@test_step_action_registry.register_config(ACTION_NAME)
class WaitActionConfig(StepActionConfig):
    """
    Configuration for the WaitAction.

    Attributes:
        delay_seconds (float): The number of seconds to pause during step execution.
            This should be a non-negative float representing the delay duration.
    """
    delay_seconds: float


@test_step_action_registry.register_class(ACTION_NAME)
class WaitAction(StepAction):
    """
    Step action that introduces a delay during test execution.

    This action simply waits for the duration specified in the configuration before proceeding.
    Useful for introducing timing gaps or waiting for external processes to settle.

    Attributes:
        config (WaitActionConfig): Configuration containing the delay duration in seconds.
    """
    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Test Max Rate Logs
    steps:
    - name: Wait For Test
      action:
        wait:
          delay_seconds: 3
"""
    )

    def __init__(self, config: WaitActionConfig):
        """
        Initializes the WaitAction with the given configuration.

        Args:
            config (WaitActionConfig): The configuration specifying how long to wait.
        """
        self.config = config

    def execute(self, _ctx: StepContext):
        """
        Executes the wait action by sleeping for the configured number of seconds.

        Args:
            _ctx (StepContext): The execution context for the step (unused in this action).
        """
        time.sleep(self.config.delay_seconds)
