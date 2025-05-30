"""
hook_config.py

This module defines the configuration schema for managing lifecycle hooks
attached to components or test phases in the framework.

Key Elements:
- HooksConfig: A Pydantic model representing pre- and post-phase hooks that can
  be applied to a component or test. Each phase supports a list of hook wrappers
  and an associated strategy for how hooks should be added (e.g., append or replace).

Types:
- HookAddStrategy: A literal type defining allowed strategies for modifying the
  existing hook list ("append" or "replace").

Usage:
- Use `HooksConfig` as part of higher-level configurations (e.g., test steps,
  test definitions, components) to declaratively define lifecycle hooks.
- Hooks are wrapped using `HookWrapper` to defer instantiation until build time.

This configuration model enables flexible and declarative control over how hooks
are injected into different phases of execution.
"""

from typing import List, Literal

from pydantic import BaseModel, Field

from ..wrappers import HookWrapper


HookAddStrategy = Literal["append", "replace"]


class HooksConfig(BaseModel):
    """Base configuration model that specifies a hook to set on a component or test"""

    pre: List[HookWrapper] = Field(default_factory=list)
    pre_strategy: HookAddStrategy = "append"
    post: List[HookWrapper] = Field(default_factory=list)
    post_strategy: HookAddStrategy = "append"
