"""
Hook strategy module for executing shell commands.

This module defines the `run_command` hook, which allows arbitrary shell commands
to be executed during a pipeline's lifecycle. It can be used for setup, cleanup,
diagnostics, or any custom action required in a test or deployment step.

Classes:
    - RunCommandConfig: Configuration schema specifying the shell command to execute.
    - RunCommandHook: Hook strategy that runs the configured command when invoked.

Use case:
    Useful for triggering external tools, modifying environment state, or scripting
    ad hoc operations as part of a larger execution flow.

Warnings:
    This hook executes commands using `shell=True`, which can be a security risk if
    input is not properly sanitized. Use caution when including user-supplied strings.
"""

from ....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from ....core.context.base import BaseContext
from ....core.context import ComponentHookContext, FrameworkElementHookContext
from ....runner.registry import hook_registry, PluginMeta


@hook_registry.register_config("run_command")
class RunCommandConfig(HookStrategyConfig):
    """
    Configuration class for the 'run_command' hook.

    Attributes:
        command (str): The shell command to be executed by the hook.
    """

    command: str


@hook_registry.register_class("run_command")
class RunCommandHook(HookStrategy):
    """
    Hook strategy that runs a specified shell command.

    This class is responsible for executing a shell command defined in its configuration
    when the hook is triggered.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[
            FrameworkElementHookContext.__name__,
            ComponentHookContext.__name__,
        ],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Test Max Rate Logs
    steps:
      - name: Run a command then wait step
        action:
          wait:
            delay_seconds: 10
        hooks:
          run:
            pre:
              - run_command:
                  command: python somefile.py
""",
    )

    def __init__(self, config: RunCommandConfig):
        """
        Initialize the hook with its configuration.

        Args:
            config (RunCommandConfig): Configuration object containing the command to run.
        """
        self.config = config

    def execute(self, ctx: BaseContext):
        """
        Execute the configured shell command using the subprocess module.

        Args:
            ctx (BaseContext): The execution context, providing utilities like logging.

        Raises:
            subprocess.CalledProcessError: If the command returns a non-zero exit code.
        """
        import subprocess

        logger = ctx.get_logger(__name__)

        logger.debug(f"Running: {self.config.command}")
        # Execute the command with shell=True, and raise an exception if it fails
        subprocess.run([self.config.command], shell=True, check=True)
