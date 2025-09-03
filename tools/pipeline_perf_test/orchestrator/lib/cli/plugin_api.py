from typing import Callable, List
import argparse

_argument_hooks: List[Callable[[argparse.ArgumentParser], None]] = []

_group_registry = {}


def get_or_create_arg_group(parser, group_name: str):
    """
    Ensures only one argument group is created per group name.
    """
    if group_name not in _group_registry:
        _group_registry[group_name] = parser.add_argument_group(group_name)
    return _group_registry[group_name]


def register_argument_hook(hook: Callable[[argparse.ArgumentParser], None]) -> None:
    """Allows plugins to register a function to modify the argument parser."""
    _argument_hooks.append(hook)


def apply_argument_hooks(parser: argparse.ArgumentParser) -> None:
    """Apply all registered argument hooks to a parser."""
    for hook in _argument_hooks:
        hook(parser)
