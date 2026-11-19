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
from ..common.kubernetes import ComponentKubernetesRuntime, pod_identity


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
        label_selector: app=my-service,component=collector
        # Optional: use a specific kubeconfig and context.
        # kubeconfig: /path/to/kubeconfig
        # context: my-context
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
        pods = [pod_identity(pod) for pod in live_pods]
        component.set_runtime_data(
            ComponentKubernetesRuntime.type,
            ComponentKubernetesRuntime(
                namespace=self.config.namespace,
                label_selector=self.config.label_selector,
                context=self.config.context,
                kubeconfig=self.config.kubeconfig,
                pods=pods,
            ),
        )
        ctx.get_logger(__name__).info(
            "Observing %d existing Kubernetes pod(s) in %s",
            len(pods),
            self.config.namespace,
        )
