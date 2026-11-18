"""Read-only resource adoption and lifecycle ownership regressions."""

from unittest.mock import MagicMock, Mock, call, patch

from docker.errors import NotFound
from kubernetes import client
from kubernetes.client.exceptions import ApiException
from pydantic import ValidationError
import pytest

from lib.impl import strategies  # noqa: F401 (register built-in strategies)
from lib.core.context.component_hook_context import HookableComponentPhase
from lib.impl.actions.component_action import ComponentAction, ComponentActionConfig
from lib.impl.actions.multi_component_action import (
    MultiComponentAction,
    MultiComponentActionConfig,
)
from lib.impl.component.managed_component import ManagedComponentConfiguration
from lib.impl.strategies.common.docker import ComponentDockerRuntime
from lib.impl.strategies.common.kubernetes import ComponentKubernetesRuntime
from lib.impl.strategies.existing.docker import ExistingDockerConfig
from lib.impl.strategies.existing.kubernetes import ExistingKubernetesConfig
from lib.runner.factory import build_managed_component

DOCKER_MODULE = "lib.impl.strategies.existing.docker"
DOCKER_MONITOR_MODULE = "lib.impl.strategies.monitoring.docker_component"


def existing_component(existing, **kwargs):
    config = ManagedComponentConfiguration.model_validate(
        {"existing": existing, **kwargs}
    )
    return build_managed_component("existing-service", config)


def running_pod(name="worker-0"):
    def container_status(container_name):
        return client.V1ContainerStatus(
            name=container_name,
            image="example",
            image_id="sha256:example",
            ready=True,
            restart_count=0,
            container_id=f"containerd://{name}-{container_name}",
            state=client.V1ContainerState(running=client.V1ContainerStateRunning()),
        )

    return client.V1Pod(
        metadata=client.V1ObjectMeta(name=name, uid=f"uid-{name}"),
        spec=client.V1PodSpec(
            node_name="node-0",
            containers=[client.V1Container(name="worker")],
            init_containers=[
                client.V1Container(name="setup"),
                client.V1Container(name="sidecar", restart_policy="Always"),
            ],
        ),
        status=client.V1PodStatus(
            phase="Running",
            conditions=[client.V1PodCondition(type="Ready", status="True")],
            container_statuses=[container_status("worker")],
            init_container_statuses=[container_status("sidecar")],
        ),
    )


@pytest.fixture
def kubernetes_api():
    api = Mock(spec_set=["list_namespaced_pod"])
    api.list_namespaced_pod.return_value = client.V1PodList(
        items=[running_pod(), running_pod("worker-1")]
    )
    with (
        patch("kubernetes.config.new_client_from_config") as new_client,
        patch("kubernetes.client.CoreV1Api", return_value=api) as core_api,
    ):
        yield api, new_client, core_api


# Scenario: A component claims both harness deployment and caller ownership.
# Guarantees: Conflicting ownership is rejected before either backend runs.
def test_existing_and_deployment_are_mutually_exclusive():
    with pytest.raises(ValidationError, match="mutually exclusive"):
        ManagedComponentConfiguration.model_validate(
            {
                "existing": {"docker": {"container": "service"}},
                "deployment": {"docker": {"image": "example"}},
            }
        )


# Scenario: An existing-resource declaration has ambiguous or invalid selection.
# Guarantees: Invalid selectors/backend names fail explicitly instead of silently
# ignoring configuration or observing every resource in a namespace.
@pytest.mark.parametrize(
    "existing",
    [
        {},
        {"unknown": {}},
        {"docker": {"container": "service"}, "kubernetes": {}},
        {"docker": {"container": " "}},
        {"docker": {"container": "service", "image": "must-not-be-created"}},
        {"kubernetes": {"namespace": "telemetry", "label_selector": " "}},
        {"kubernetes": {"label_selector": "app=worker"}},
    ],
)
def test_invalid_existing_configuration_is_rejected(existing):
    with pytest.raises(ValidationError):
        existing_component(existing)


# Scenario: Valid existing-resource configuration is serialized and reloaded.
# Guarantees: The backend-specific selector and optional connection settings survive.
@pytest.mark.parametrize(
    "existing,config_type",
    [
        ({"docker": {"container": "service"}}, ExistingDockerConfig),
        (
            {"kubernetes": {"namespace": "telemetry", "label_selector": "app=worker"}},
            ExistingKubernetesConfig,
        ),
    ],
)
def test_existing_configuration_round_trips(existing, config_type):
    component = existing_component(existing)
    config = component.component_config
    restored = ManagedComponentConfiguration.model_validate(config.model_dump())
    assert restored == config
    assert isinstance(restored.existing.config, config_type)


# Scenario: A running Docker container is adopted, monitored, and torn down.
# Guarantees: Its resolved ID reaches the existing Docker monitor, with no image
# build, container creation, stop/removal, or network management hooks.
def test_existing_docker_reuses_monitoring_without_lifecycle_ownership():
    component = existing_component(
        {"docker": {"container": "service"}},
        monitoring={"docker_component": {"interval": 1}},
    )
    docker = MagicMock()
    container = docker.containers.get.return_value
    container.id = "resolved-container-id"
    container.status = "running"
    ctx = MagicMock()
    with (
        patch(f"{DOCKER_MODULE}.get_or_create_docker_client", return_value=docker),
        patch(
            f"{DOCKER_MONITOR_MODULE}.get_or_create_docker_client",
            return_value=docker,
        ),
        patch(f"{DOCKER_MONITOR_MODULE}.threading.Thread") as thread,
    ):
        thread.return_value.is_alive.return_value = False
        component.adopt(ctx)
        component.start_monitoring(ctx)
        component.stop_monitoring(ctx)
        component.destroy(ctx)

    assert docker.mock_calls == [call.containers.get("service")]
    runtime = component.get_or_create_runtime(
        ComponentDockerRuntime.type, ComponentDockerRuntime
    )
    assert runtime.container_id == "resolved-container-id"
    assert not runtime.network_created
    assert thread.call_args.kwargs["kwargs"]["container_id"] == runtime.container_id
    thread.return_value.start.assert_called_once()
    thread.return_value.join.assert_called_once()


# Scenario: Docker adoption selects a stopped or missing container.
# Guarantees: Adoption fails explicitly and never tries to start or recreate it.
@pytest.mark.parametrize("missing", [False, True])
def test_existing_docker_does_not_repair_unavailable_containers(missing):
    component = existing_component({"docker": {"container": "service"}})
    docker = MagicMock()
    if missing:
        docker.containers.get.side_effect = NotFound("missing")
    else:
        docker.containers.get.return_value.status = "exited"
    with (
        patch(f"{DOCKER_MODULE}.get_or_create_docker_client", return_value=docker),
        pytest.raises(NotFound if missing else ValueError),
    ):
        component.adopt(MagicMock())
    assert docker.mock_calls == [call.containers.get("service")]


# Scenario: A Kubernetes selector matches two ready pods with native sidecars.
# Guarantees: Adoption records pod/container identities using only a bounded
# read API, without persisting kubeconfig changes or touching cluster resources.
def test_existing_kubernetes_records_pods_and_sidecars_read_only(kubernetes_api):
    api, new_client, core_api = kubernetes_api
    component = existing_component(
        {
            "kubernetes": {
                "namespace": "telemetry",
                "label_selector": "app=worker",
                "context": "benchmark",
                "kubeconfig": "/caller/kubeconfig",
            }
        }
    )
    component.adopt(MagicMock())
    component.destroy(MagicMock())

    new_client.assert_called_once_with(
        config_file="/caller/kubeconfig", context="benchmark", persist_config=False
    )
    core_api.assert_called_once_with(new_client.return_value.__enter__.return_value)
    assert api.mock_calls == [
        call.list_namespaced_pod(
            namespace="telemetry",
            label_selector="app=worker",
            _request_timeout=(5, 30),
        )
    ]
    runtime = component.get_or_create_runtime(
        ComponentKubernetesRuntime.type, lambda: pytest.fail("No Kubernetes identity")
    )
    assert runtime.namespace == "telemetry"
    assert runtime.context == "benchmark"
    assert [(pod.name, pod.uid, pod.node_name) for pod in runtime.pods] == [
        ("worker-0", "uid-worker-0", "node-0"),
        ("worker-1", "uid-worker-1", "node-0"),
    ]
    assert runtime.pods[0].containers == {
        "worker": "containerd://worker-0-worker",
        "sidecar": "containerd://worker-0-sidecar",
    }


# Scenario: Kubernetes selection includes terminal pods alongside a ready pod.
# Guarantees: Only live pod identities are adopted, ignoring evicted/completed pods.
@pytest.mark.parametrize("phase", ["Failed", "Succeeded"])
def test_existing_kubernetes_skips_terminal_pods(kubernetes_api, phase):
    api, _, _ = kubernetes_api
    terminal_pod = running_pod("worker-old")
    terminal_pod.status = client.V1PodStatus(
        phase=phase, reason="Evicted" if phase == "Failed" else None
    )
    api.list_namespaced_pod.return_value.items = [terminal_pod, running_pod()]
    component = existing_component(
        {"kubernetes": {"namespace": "telemetry", "label_selector": "app=worker"}}
    )
    component.adopt(MagicMock())

    runtime = component.get_or_create_runtime(
        ComponentKubernetesRuntime.type, lambda: pytest.fail("No Kubernetes identity")
    )
    assert [pod.name for pod in runtime.pods] == ["worker-0"]


# Scenario: Kubernetes selection matches only pods in a terminal phase.
# Guarantees: Finished pods cannot produce a successful empty adoption.
@pytest.mark.parametrize("phases", [("Failed",), ("Succeeded",), ("Failed", "Succeeded")])
def test_existing_kubernetes_rejects_terminal_only_selection(kubernetes_api, phases):
    api, _, _ = kubernetes_api
    pods = [running_pod(f"worker-{index}") for index in range(len(phases))]
    for pod, phase in zip(pods, phases):
        pod.status.phase = phase
    api.list_namespaced_pod.return_value.items = pods
    component = existing_component(
        {"kubernetes": {"namespace": "telemetry", "label_selector": "app=worker"}}
    )
    with pytest.raises(ValueError, match="after excluding terminal pods"):
        component.adopt(MagicMock())


# Scenario: Kubernetes selection is empty or includes an unavailable live pod.
# Guarantees: Invalid observations fail synchronously before monitoring can use
# a partial identity set; the harness never tries to repair the deployment.
@pytest.mark.parametrize(
    "invalid", ["empty", "pending", "terminating", "not-ready", "missing-sidecar", "no-id"]
)
def test_existing_kubernetes_rejects_unavailable_pods(kubernetes_api, invalid):
    api, _, _ = kubernetes_api
    pod = running_pod()
    if invalid == "pending":
        pod.status.phase = "Pending"
    elif invalid == "terminating":
        from datetime import datetime, timezone

        pod.metadata.deletion_timestamp = datetime.now(timezone.utc)
    elif invalid == "not-ready":
        pod.status.conditions[0].status = "False"
    elif invalid == "missing-sidecar":
        pod.status.init_container_statuses = []
    elif invalid == "no-id":
        pod.status.container_statuses[0].container_id = None
    api.list_namespaced_pod.return_value.items = (
        [] if invalid == "empty" else [running_pod("worker-ready"), pod]
    )
    component = existing_component(
        {"kubernetes": {"namespace": "telemetry", "label_selector": "app=worker"}}
    )
    with pytest.raises(ValueError):
        component.adopt(MagicMock())
    assert api.list_namespaced_pod.call_count == 1


# Scenario: Kubernetes API access is forbidden or unavailable.
# Guarantees: The original API failure propagates, with no empty-success fallback.
def test_existing_kubernetes_propagates_api_errors(kubernetes_api):
    api, _, _ = kubernetes_api
    api.list_namespaced_pod.side_effect = ApiException(status=403, reason="Forbidden")
    component = existing_component(
        {"kubernetes": {"namespace": "telemetry", "label_selector": "app=worker"}}
    )
    with pytest.raises(ApiException, match="Forbidden"):
        component.adopt(MagicMock())


# Scenario: A dynamic strategy replacement attempts to change resource ownership.
# Guarantees: Neither an existing target nor a deployed component can silently
# gain the other's lifecycle behavior through replace_strategy.
@pytest.mark.parametrize("start_with_existing", [False, True])
def test_strategy_replacement_cannot_cross_ownership_modes(start_with_existing):
    existing = existing_component({"docker": {"container": "service"}})
    deployed = build_managed_component(
        "deployed",
        ManagedComponentConfiguration.model_validate(
            {"deployment": {"docker": {"image": "example"}}}
        ),
    )
    component, replacement = (
        (existing, deployed.deployment)
        if start_with_existing
        else (deployed, existing.existing)
    )
    with pytest.raises(ValueError, match="Cannot"):
        component.replace_strategy(replacement)


@pytest.mark.parametrize(
    "config",
    [{}, {"deployment": {"docker": {"image": "example"}}}],
)
def test_adopt_requires_existing_strategy(config):
    component = build_managed_component(
        "service", ManagedComponentConfiguration.model_validate(config)
    )
    with (
        patch.object(component, "_run_hooks") as hooks,
        pytest.raises(ValueError, match="requires an 'existing' strategy"),
    ):
        component.adopt(MagicMock())
    hooks.assert_not_called()


def test_deploy_does_not_adopt_existing_resources():
    component = existing_component({"docker": {"container": "service"}})
    with (
        patch.object(component.existing, "resolve") as resolve,
        patch.object(component, "_run_hooks") as hooks,
        pytest.raises(ValueError, match="Use the 'adopt' phase"),
    ):
        component.deploy(MagicMock())
    resolve.assert_not_called()
    hooks.assert_not_called()


def test_adopt_runs_its_own_hooks_around_resolution():
    component = existing_component(
        {"docker": {"container": "service"}},
        hooks={
            "adopt": {
                "pre": [{"record_event": {"name": "before_adopt"}}],
                "post": [{"record_event": {"name": "after_adopt"}}],
            }
        },
    )
    assert len(component._hooks[HookableComponentPhase.PRE_ADOPT]) == 1
    assert len(component._hooks[HookableComponentPhase.POST_ADOPT]) == 1
    order = []
    ctx = MagicMock()
    with (
        patch.object(
            component,
            "_run_hooks",
            side_effect=lambda phase, context: order.append(phase),
        ),
        patch.object(
            component.existing,
            "resolve",
            side_effect=lambda target, context: order.append("resolve"),
        ) as resolve,
    ):
        component.adopt(ctx)
    assert order == [
        HookableComponentPhase.PRE_ADOPT,
        "resolve",
        HookableComponentPhase.POST_ADOPT,
    ]
    resolve.assert_called_once_with(component, ctx)


@pytest.mark.parametrize(
    "action_type,config_type,targets",
    [
        (ComponentAction, ComponentActionConfig, {"target": "existing-service"}),
        (
            MultiComponentAction,
            MultiComponentActionConfig,
            {"targets": ["existing-service"]},
        ),
    ],
)
def test_adopt_phase_dispatches_to_resource_resolution(
    action_type, config_type, targets
):
    component = existing_component({"docker": {"container": "service"}})
    ctx = MagicMock()
    ctx.get_component_by_name.return_value = component
    action = action_type(config_type(phase="adopt", **targets))
    with patch.object(component.existing, "resolve") as resolve:
        action.execute(ctx)
    resolve.assert_called_once_with(component, ctx)
    ctx.set_step_component.assert_called_once_with(component)


@pytest.mark.parametrize("failure", ["omitted", "resolve", "post_hook"])
def test_monitoring_requires_successful_adoption(failure):
    component = existing_component({"docker": {"container": "service"}})
    ctx = MagicMock()

    def run_hooks(phase, _ctx):
        if failure == "post_hook" and phase == HookableComponentPhase.POST_ADOPT:
            raise ValueError("adoption failed")

    with (
        patch.object(component.existing, "resolve") as resolve,
        patch.object(component, "_run_hooks", side_effect=run_hooks) as hooks,
        patch.object(component.monitoring, "start") as start,
    ):
        if failure != "omitted":
            if failure == "resolve":
                resolve.side_effect = ValueError("adoption failed")
            with pytest.raises(ValueError, match="adoption failed"):
                component.adopt(ctx)
        hooks.reset_mock()
        with pytest.raises(ValueError, match="requires a successful 'adopt' phase"):
            component.start_monitoring(ctx)
    hooks.assert_not_called()
    start.assert_not_called()


@pytest.mark.parametrize("invalidate", ["failed_readoption", "replace_strategy"])
def test_monitoring_cannot_reuse_stale_adoption(invalidate):
    component = existing_component({"docker": {"container": "service"}})
    ctx = MagicMock()
    with patch.object(component.existing, "resolve"):
        component.adopt(ctx)

    if invalidate == "failed_readoption":
        with (
            patch.object(
                component.existing, "resolve", side_effect=ValueError("not found")
            ),
            pytest.raises(ValueError, match="not found"),
        ):
            component.adopt(ctx)
    else:
        replacement = existing_component({"docker": {"container": "other"}})
        component.replace_strategy(replacement.existing)

    with patch.object(component.monitoring, "start") as start:
        with pytest.raises(ValueError, match="requires a successful 'adopt' phase"):
            component.start_monitoring(ctx)
        start.assert_not_called()
        with patch.object(component.existing, "resolve"):
            component.adopt(ctx)
        component.start_monitoring(ctx)
        start.assert_called_once_with(component, ctx)
