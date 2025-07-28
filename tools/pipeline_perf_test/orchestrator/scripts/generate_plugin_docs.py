"""
Plugin Documentation Generator

This script generates markdown documentation for all plugin registries in the framework.
It scans registered strategy and action classes, extracts metadata and docstrings, and
renders the output using Jinja2 templates.

Output is written to the 'docs/plugins' directory, with one file per registry and an
index README summarizing all registered plugins.

Registries handled:
- Deployment strategies
- Monitoring strategies
- Configuration strategies
- Execution strategies
- Hook plugins
- Step actions
- Report formatters
- Report writers
"""

import inspect
import textwrap
from pathlib import Path
from typing import Type
from collections import defaultdict
from jinja2 import Environment, FileSystemLoader
from lib.runner import registry

from lib.impl import strategies  # Do not remove
from lib.impl import actions  # Do not remove

REGISTRIES = {
    "deployment_strategies": registry.deployment_registry,
    "monitoring_strategies": registry.monitoring_registry,
    "configuration_strategies": registry.configuration_registry,
    "execution_strategies": registry.execution_registry,
    "hook_strategies": registry.hook_registry,
    "step_actions": registry.step_action_registry,
    "report_formatters": registry.report_formatter_registry,
    "report_writers": registry.report_writer_registry,
}

OUTPUT_DIR = Path("docs/plugins")
REPORTS_OUTPUT_DIR = Path("docs/reports")
TEMPLATE_DIR = Path("scripts/templates")

env = Environment(loader=FileSystemLoader(TEMPLATE_DIR))
env.filters['wrap80'] = lambda text: textwrap.fill(text, width=80)


def extract_doc(obj: Type) -> str:
    """
    Extracts the docstring from a given class or type.

    Args:
        obj (Type): The class or object to extract the docstring from.

    Returns:
        str: The cleaned docstring, or a default fallback if not available.
    """
    return inspect.getdoc(obj) or "*No documentation available.*"


def write_report_docs():
    """
    Generates detailed report documentation for plugins that have `report_meta`.
    Outputs full metadata: docstring, YAML, aggregation info, etc.
    """
    REPORTS_OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    template = env.get_template("report.md.j2")

    for _registry_name, reg_obj in REGISTRIES.items():
        for type_name, cls in reg_obj.element.items():
            doc: registry.PluginMeta = getattr(cls, "PLUGIN_META", None)
            if not doc or not getattr(doc, "report_meta", None):
                continue

            config_cls = reg_obj.config.get(type_name)
            report_meta = doc.report_meta

            filtered_aggregations = [
                agg for agg in report_meta.supported_aggregations
                if agg.lower() != "none"
            ]
            filtered_outputs = {
                k: v for k, v in report_meta.sample_output.items()
                if k.lower() != "without aggregation" and k.lower() != "none"
            }

            output_path = REPORTS_OUTPUT_DIR / f"{type_name}.md"

            rendered = template.render(
                type_name=type_name,
                class_path=f"{cls.__module__}.{cls.__name__}",
                config_class=f"{config_cls.__module__}.{config_cls.__name__}" if config_cls else None,
                docstring=extract_doc(cls),
                example_yaml=(doc.yaml_example or "# No example provided").strip(),
                supported_contexts=doc.supported_contexts or [],
                installs_hooks=doc.installs_hooks or [],
                notes=doc.notes,
                supported_aggregations=filtered_aggregations,
                sample_outputs=filtered_outputs,
            )

            output_path.write_text(rendered)


def write_registry_docs(registry_name: str, reg_obj: registry.ElementRegistry) -> list[dict]:
    """
    Generates markdown documentation for a single plugin registry.

    - Extracts plugin metadata, docstrings, and example YAML.
    - Renders the content using a Jinja2 template.
    - Writes output to 'docs/plugins/{registry_name}.md'.

    Args:
        registry_name (str): Name of the registry (e.g., "execution_strategies").
        reg_obj (ElementRegistry): The registry object containing plugin classes.

    Returns:
        list[dict]: A list of summary row dictionaries for index aggregation.
    """
    output_path = OUTPUT_DIR / f"{registry_name}.md"
    output_path.parent.mkdir(parents=True, exist_ok=True)

    summary_rows = []
    detailed_sections = []

    for type_name, cls in reg_obj.element.items():
        doc: registry.PluginMeta = getattr(cls, "PLUGIN_META", None)
        if doc and getattr(doc, "report_meta", None):
            continue
        config_cls = reg_obj.config.get(type_name)

        docstring = extract_doc(cls)
        short_summary = docstring.split('.', maxsplit=1)[0]
        example_yaml = (doc.yaml_example if doc and doc.yaml_example else "# No example provided").strip()

        summary_rows.append({
            "registry": registry_name,
            "type_name": type_name,
            "module": cls.__module__,
            "class_name": cls.__name__,
            "config_name": config_cls.__name__ if config_cls else "-",
            "summary": short_summary
        })

        section = {
            "type_name": type_name,
            "class_path": f"{cls.__module__}.{cls.__name__}",
            "config_class": f"{cls.__module__}.{config_cls.__name__}" if config_cls else None,
            "supported_contexts": doc.supported_contexts if doc and doc.supported_contexts else [],
            "installs_hooks": doc.installs_hooks if doc and doc.installs_hooks else [],
            "cli_flags": defaultdict(list),
            "docstring": docstring,
            "notes": doc.notes if doc and doc.notes else None,
            "example_yaml": example_yaml,
        }

        if doc and doc.cli_flags:
            for flag in doc.cli_flags:
                section["cli_flags"][flag.group].append(flag)

        detailed_sections.append(section)

    template = env.get_template("registry.md.j2")
    rendered = template.render(
        registry_name=registry_name,
        summary_rows=summary_rows,
        detailed_sections=detailed_sections
    )

    output_path.write_text(rendered)
    return summary_rows


def write_index_with_summary(all_summaries: list[dict]):
    """
    Writes an index markdown file that summarizes all plugin registries.

    Groups plugins by registry and renders a top-level README using a template.

    Args:
        all_summaries (list[dict]): List of summary row entries from all registries.
    """
    index_path = OUTPUT_DIR / "README.md"
    registry_groups = defaultdict(list)
    for entry in all_summaries:
        registry_groups[entry["registry"]].append(entry)

    template = env.get_template("index.md.j2")
    rendered = template.render(
        registry_names=REGISTRIES.keys(),
        registry_groups=registry_groups
    )

    index_path.write_text(rendered)


def main():
    """
    Main entry point for the documentation generator.

    Iterates over all plugin registries, renders individual docs, and writes a global index.
    """
    all_summaries = []
    for name, reg in REGISTRIES.items():
        summaries = write_registry_docs(name, reg)
        all_summaries.extend(summaries)

    write_index_with_summary(all_summaries)
    write_report_docs()


if __name__ == "__main__":
    main()
