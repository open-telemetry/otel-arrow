from ...core.context import StepContext
from ...core.framework.step import StepActionConfig, StepAction
from ...runner.registry import step_action_registry, PluginMeta


ACTION_NAME = "no_op"


@step_action_registry.register_config(ACTION_NAME)
class NoOpActionConfig(StepActionConfig):
    """
    Configuration class for the No Op action.
    """

    pass


@step_action_registry.register_class(ACTION_NAME)
class NoOpAction(StepAction):
    """
    Step action that does nothing when execute is called.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Test Placeholder Step
    steps:
      - name: Do Nothing Step
        action:
          no_op: {}
""",
    )

    def __init__(self, config: NoOpActionConfig):
        self.config = config

    def execute(self, _ctx: StepContext):
        pass
