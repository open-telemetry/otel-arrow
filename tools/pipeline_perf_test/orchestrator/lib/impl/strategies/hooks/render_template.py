"""
Hook strategy module for rendering Jinja2 templates.

This module defines the `render_template` hook, which renders a Jinja2 template
from a specified source file to a destination file using provided variables.

Classes:
    - RenderTemplateConfig: Configuration schema for the template rendering.
    - RenderTemplateHook: Hook strategy that performs the Jinja2 template rendering.

Use case:
    Useful for generating config files, scripts, or data files during the test
    lifecycle using dynamic data and templates.
"""

from ....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from ....core.context.base import BaseContext
from ....core.context import ComponentHookContext, FrameworkElementHookContext
from ....runner.registry import hook_registry, PluginMeta
from typing import Dict
from jinja2 import Environment, FileSystemLoader, TemplateError
import os

HOOK_NAME = "render_template"


@hook_registry.register_config(HOOK_NAME)
class RenderTemplateConfig(HookStrategyConfig):
    """
    Configuration class for the 'render_template' hook.

    Attributes:
        template_path (str): Path to the Jinja2 template file.
        output_path (str): Path where the rendered file should be written.
        variables (Dict[str, Any]): Key-value pairs to use in rendering the template.
    """

    template_path: str
    output_path: str
    variables: Dict[str, str]


@hook_registry.register_class(HOOK_NAME)
class RenderTemplateHook(HookStrategy):
    """
    Hook strategy that renders a Jinja2 template using provided variables.

    This class is responsible for loading the template file, rendering it with
    the given variables, and saving the result to the output file.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[
            FrameworkElementHookContext.__name__,
            ComponentHookContext.__name__,
        ],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Test Config Generator
    steps:
      - name: Setup with Template
        action:
          noop: {}
        hooks:
          run:
            pre:
              - render_template:
                  template_path: templates/config.j2
                  output_path: /tmp/generated_config.yaml
                  variables:
                    env: staging
                    retries: "5"
""",
    )

    def __init__(self, config: RenderTemplateConfig):
        """
        Initialize the hook with its configuration.

        Args:
            config (RenderTemplateConfig): Configuration object with template path,
                                           output path, and variable dictionary.
        """
        self.config = config

    def execute(self, ctx: BaseContext):
        """
        Render the template and write it to the output path.

        Args:
            ctx (BaseContext): Execution context providing utilities like logging.

        Raises:
            FileNotFoundError: If the template file does not exist.
            TemplateError: If rendering fails due to template syntax or logic.
        """
        logger = ctx.get_logger(__name__)

        template_path = self.config.template_path
        output_path = self.config.output_path
        variables = self.config.variables

        template_dir = os.path.dirname(template_path)
        template_file = os.path.basename(template_path)

        logger.debug(f"Loading template: {template_file} from {template_dir}")
        logger.debug(f"Rendering with variables: {variables}")
        logger.debug(f"Output will be written to: {output_path}")

        try:
            env = Environment(loader=FileSystemLoader(template_dir))
            template = env.get_template(template_file)
            rendered = template.render(variables)

            os.makedirs(os.path.dirname(output_path), exist_ok=True)

            with open(output_path, "w") as f:
                f.write(rendered)

            logger.info(f"Rendered template saved to {output_path}")

        except FileNotFoundError as e:
            logger.error(f"Template file not found: {e}")
            raise
        except TemplateError as e:
            logger.error(f"Template rendering failed: {e}")
            raise
