import pytest
from unittest.mock import MagicMock, patch
from jinja2 import TemplateError, exceptions

from lib.impl.strategies.hooks.render_template import (
    RenderTemplateHook,
    RenderTemplateConfig,
)


def test_render_template_hook_successful_render(tmp_path):
    # Setup paths
    template_content = "Hello, {{ name }}!"
    template_path = tmp_path / "template.j2"
    output_path = tmp_path / "output.txt"

    # Create a template file
    template_path.write_text(template_content)

    # Config with variables
    config = RenderTemplateConfig(
        template_path=str(template_path),
        output_path=str(output_path),
        variables={"name": "World"},
    )
    hook = RenderTemplateHook(config=config)

    # Mock context
    mock_ctx = MagicMock()
    mock_logger = MagicMock()
    mock_ctx.get_logger.return_value = mock_logger

    # Execute
    hook.execute(mock_ctx)

    # Assert file written with rendered content
    assert output_path.read_text() == "Hello, World!"

    # Logger checks
    mock_logger.debug.assert_any_call(
        f"Loading template: {template_path.name} from {template_path.parent}"
    )
    mock_logger.debug.assert_any_call(f"Rendering with variables: {config.variables}")
    mock_logger.info.assert_called_once_with(
        f"Rendered template saved to {str(output_path)}"
    )


def test_render_template_hook_template_not_found():
    # Config pointing to a nonexistent template file
    config = RenderTemplateConfig(
        template_path="/nonexistent/template.j2",
        output_path="/tmp/output.txt",
        variables={"foo": "bar"},
    )
    hook = RenderTemplateHook(config=config)

    # Mock context
    mock_ctx = MagicMock()
    mock_logger = MagicMock()
    mock_ctx.get_logger.return_value = mock_logger

    with pytest.raises(exceptions.TemplateNotFound):
        hook.execute(mock_ctx)

    mock_logger.error.assert_called()


@patch("jinja2.Environment.get_template")
def test_render_template_hook_rendering_error(mock_get_template):
    # Setup: simulate Jinja2 template rendering error
    mock_template = MagicMock()
    mock_template.render.side_effect = TemplateError("Rendering failed")
    mock_get_template.return_value = mock_template

    config = RenderTemplateConfig(
        template_path="/fake/path/template.j2",
        output_path="/fake/output.txt",
        variables={"foo": "bar"},
    )
    hook = RenderTemplateHook(config=config)

    mock_ctx = MagicMock()
    mock_logger = MagicMock()
    mock_ctx.get_logger.return_value = mock_logger

    with pytest.raises(TemplateError, match="Rendering failed"):
        hook.execute(mock_ctx)

    mock_logger.error.assert_called()
    assert any(
        "Template rendering failed" in call[0][0]
        for call in mock_logger.error.call_args_list
    )
