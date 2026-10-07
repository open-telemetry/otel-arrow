"""Regression tests for the opt-in push-metrics config templates.

These tests render the real integration config templates through the
'render_template' hook (so the shared 'push_metrics_engine.yaml.j2' partial is
resolved via 'search_paths', exactly as the suites do) and assert the two
invariants the push migration depends on.
"""

import subprocess
from pathlib import Path
from unittest.mock import MagicMock

import yaml

from lib.impl.strategies.hooks.render_template import (
    RenderTemplateHook,
    RenderTemplateConfig,
)

# tests/impl/strategies/hooks/ -> orchestrator -> pipeline_perf_test
_PPT_ROOT = Path(__file__).resolve().parents[5]
_CFG = _PPT_ROOT / "test_suites/integration/templates/configs"
_COMMON = _CFG / "common"

# (template path relative to _CFG, base render variables, expected service.name)
_TEMPLATES = [
    ("loadgen/config.yaml.j2", {"loadgen_exporter_type": "otlp"}, "load-generator"),
    ("backend/config.yaml.j2", {"backend_receiver_type": "otap"}, "backend-service"),
    (
        "engine/batch_processor/otlp-batch-otlp.yaml.j2",
        {"backend_hostname": "backend-service"},
        "df-engine",
    ),
    (
        "engine/batch_processor/otap-batch-otap.yaml.j2",
        {"backend_hostname": "backend-service"},
        "df-engine",
    ),
]


def _render(template_path: Path, variables: dict, tmp_path: Path) -> str:
    output_path = tmp_path / "rendered.yaml"
    hook = RenderTemplateHook(
        config=RenderTemplateConfig(
            template_path=str(template_path),
            output_path=str(output_path),
            variables=variables,
            search_paths=[str(_COMMON)],
        )
    )
    ctx = MagicMock()
    ctx.get_logger.return_value = MagicMock()
    hook.execute(ctx)
    return output_path.read_text()


# Scenario: Each opt-in config template is rendered with the push endpoint unset,
#   both as it exists on disk and as committed on the main branch.
# Guarantees: The push opt-in block is inert when disabled - the rendered config
#   for scrape-based suites is byte-identical to the pre-push template, so those
#   suites are unaffected by the migration.
def test_push_off_render_matches_main(tmp_path):
    for rel, variables, _ in _TEMPLATES:
        template_path = _CFG / rel
        current = _render(template_path, variables, tmp_path)

        main_src = subprocess.check_output(
            ["git", "show", f"HEAD:tools/pipeline_perf_test/test_suites/integration/templates/configs/{rel}"],
            cwd=str(_PPT_ROOT),
        ).decode()
        main_template = tmp_path / "main_template.j2"
        main_template.write_text(main_src)
        main_rendered = _render(main_template, variables, tmp_path)

        assert current == main_rendered, f"push-off render drifted for {rel}"


# Scenario: Each opt-in config template is rendered with a push endpoint and test
#   name supplied.
# Guarantees: The engine self-identifies for push - it emits an 'engine' section
#   whose telemetry.resource carries the component's service.name and the test's
#   test.name, and whose observability pipeline exports internal metrics to the
#   configured OTLP endpoint.
def test_push_on_render_self_identifies(tmp_path):
    endpoint = "http://host.docker.internal:14317"
    for rel, variables, expected_service in _TEMPLATES:
        v = dict(variables)
        v["push_metrics_endpoint"] = endpoint
        v["push_test_name"] = "Logs-OTLP-BATCH-OTLP"
        doc = yaml.safe_load(_render(_CFG / rel, v, tmp_path))

        resource = doc["engine"]["telemetry"]["resource"]
        assert resource["service.name"] == expected_service, rel
        assert resource["test.name"] == "Logs-OTLP-BATCH-OTLP", rel

        nodes = doc["engine"]["observability"]["pipeline"]["nodes"]
        assert (
            nodes["metrics_otlp"]["config"]["grpc_endpoint"] == endpoint
        ), rel
