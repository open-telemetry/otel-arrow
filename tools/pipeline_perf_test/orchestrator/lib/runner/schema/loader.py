"""
loader.py

This module provides utility functions for loading a `SuiteConfig` from YAML sources.

Functions:
- load_config_from_file: Reads a YAML file from disk and parses it into a `SuiteConfig`.
- load_config_from_string: Parses a YAML string directly into a `SuiteConfig`.

These utilities support configuration-driven workflows, allowing test suites to be
defined declaratively in YAML and loaded at runtime using Pydantic model validation.
"""

from pathlib import Path

import yaml

from .framework_element_config import SuiteConfig


def load_config_from_file(path: str | Path) -> SuiteConfig:
    """Loads and parses a SuiteConfig from a YAML file."""
    with open(path, "r", encoding="utf-8") as f:
        data = yaml.safe_load(f)
    return SuiteConfig.model_validate(data)


def load_config_from_string(yaml_str: str) -> SuiteConfig:
    """Loads and parses a SuiteConfig from a YAML string."""
    data = yaml.safe_load(yaml_str)
    return SuiteConfig.model_validate(data)
