"""
Template expansion utilities for framework element configurations.

This module provides functions to support templating within the test framework.
Framework elements such as steps, scenarios, and suites can optionally reference
external template files via a `from_template` field. These templates are rendered
using Jinja2 with user-supplied variables and parsed as YAML configuration.
"""

import yaml
from pathlib import Path

import jinja2

from ...core.framework.element import TemplateReference


def load_template(path: str, variables: dict) -> dict:
    """
    Loads a template file, renders it with Jinja2, and returns parsed data.
    """
    template_str = Path(path).read_text(encoding="utf-8")
    rendered = jinja2.Template(template_str).render(**variables)
    return yaml.safe_load(rendered)


def expand_template_if_needed(data: dict) -> dict:
    """
    Checks element config for template key and expands if needed.
    """
    if not isinstance(data, dict):
        return data

    template_ref = data.get("from_template")
    if not template_ref:
        return data

    if isinstance(template_ref, dict):
        ref = TemplateReference(**template_ref)
    elif isinstance(template_ref, TemplateReference):
        ref = template_ref
    else:
        raise ValueError("Invalid template reference")

    loaded = load_template(ref.path, ref.variables)

    # Merge with outer config
    merged = {**loaded, **data}
    merged.pop("from_template", None)
    return merged
