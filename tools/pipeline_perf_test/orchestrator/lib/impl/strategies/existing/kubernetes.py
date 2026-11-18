"""Resolve Kubernetes pods through the read-only core API."""

from typing import Optional

from pydantic import Field

from ....core.component.component import Component
from ....core.context.framework_element_contexts import StepContext
from ....core.strategies.existing_strategy import (
    ExistingStrategy,
    ExistingStrategyConfig,
)
from ....runner.registry import existing_registry, PluginMeta
from ..common.kubernetes import ComponentKubernetesRuntime, KubernetesPodIdentity


@existing_registry.register_config("kubernetes")
class ExistingKubernetesConfig(ExistingStrategyConfig):
    """Select ready pods using a namespace and a nonempty label selector."""

    namespace: str = Field(min_length=1)
    label_selector: str = Field(min_length=1)
    context: Optional[str] = None
    kubeconfig: Optional[str] = None


@existing_registry.register_class("kubernetes")
class ExistingKubernetes(ExistingStrategy):
    """Resolve ready Kubernetes pods without changing cluster resources."""

    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
components:
  existing-service:
    existing:
      kubernetes:
        namespace: telemetry
        label_selector: app=my-service
    monitoring:
      prometheus:
        endpoint: http://127.0.0.1:8888/metrics
""",
    )

    def __init__(self, config: ExistingKubernetesConfig):
        self.config = config

    def resolve(self, component: Component, ctx: StepContext):
        from kubernetes import client, config

        with config.new_client_from_config(
            config_file=self.config.kubeconfig,
            context=self.config.context,
            persist_config=False,
        ) as api_client:
            result = client.CoreV1Api(api_client).list_namespaced_pod(
                namespace=self.config.namespace,
                label_selector=self.config.label_selector,
                _request_timeout=(5, 30),
            )
        live_pods = [
            pod
            for pod in result.items
            if pod.status.phase not in ("Succeeded", "Failed")
        ]
        if not live_pods:
            raise ValueError(
                f"No existing pods match {self.config.label_selector!r} "
                f"in namespace {self.config.namespace!r} after excluding terminal pods"
            )
        pods = []
        for pod in live_pods:
            name = pod.metadata.name
            if (
                pod.metadata.deletion_timestamp is not None
                or pod.status.phase != "Running"
                or not any(
                    condition.type == "Ready" and condition.status == "True"
                    for condition in (pod.status.conditions or [])
                )
            ):
                raise ValueError(f"Existing pod {name!r} is not running and ready")
            specs = pod.spec.containers + [
                container
                for container in (pod.spec.init_containers or [])
                if container.restart_policy == "Always"
            ]
            statuses = {
                container.name: container
                for container in (
                    (pod.status.container_statuses or [])
                    + (pod.status.init_container_statuses or [])
                )
            }
            containers = {}
            for spec in specs:
                status = statuses.get(spec.name)
                if (
                    status is None
                    or not status.ready
                    or not status.container_id
                    or status.state is None
                    or status.state.running is None
                ):
                    raise ValueError(
                        f"Existing pod {name!r} container {spec.name!r} "
                        "has no ready, running container identity"
                    )
                containers[spec.name] = status.container_id
            pods.append(
                KubernetesPodIdentity(
                    name=name,
                    uid=pod.metadata.uid,
                    node_name=pod.spec.node_name,
                    containers=containers,
                )
            )
        component.set_runtime_data(
            ComponentKubernetesRuntime.type,
            ComponentKubernetesRuntime(
                namespace=self.config.namespace, context=self.config.context, pods=pods
            ),
        )
        ctx.get_logger(__name__).info(
            "Observing %d existing Kubernetes pod(s) in %s",
            len(pods),
            self.config.namespace,
        )
