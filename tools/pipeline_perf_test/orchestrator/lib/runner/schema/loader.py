"""
loader.py

This module provides utility functions for loading a `TestSuiteConfig` from YAML sources.

Functions:
- load_config_from_file: Reads a YAML file from disk and parses it into a `TestSuiteConfig`.
- load_config_from_string: Parses a YAML string directly into a `TestSuiteConfig`.

These utilities support configuration-driven workflows, allowing test suites to be
defined declaratively in YAML and loaded at runtime using Pydantic model validation.
"""

import yaml
from pathlib import Path
from .test_config import TestSuiteConfig


def load_config_from_file(path: str | Path) -> TestSuiteConfig:
    """Loads and parses a TestSuiteConfig from a YAML file."""
    with open(path, "r", encoding="utf-8") as f:
        data = yaml.safe_load(f)
    return TestSuiteConfig.model_validate(data)


def load_config_from_string(yaml_str: str) -> TestSuiteConfig:
    """Loads and parses a TestSuiteConfig from a YAML string."""
    data = yaml.safe_load(yaml_str)
    return TestSuiteConfig.model_validate(data)
