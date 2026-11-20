"""Resolve Kubernetes pods through the read-only core API."""

from contextlib import ExitStack
from dataclasses import dataclass
from typing import Annotated, ClassVar, Optional, TYPE_CHECKING

from pydantic import Field

from ....core.component.component import Component, ComponentTarget
from ....core.context.framework_element_contexts import StepContext
from ....core.strategies.existing_strategy import (
    ExistingStrategy,
    ExistingStrategyConfig,
)
from ....runner.registry import existing_registry, PluginMeta
from ..common.kubernetes import (
    ComponentKubernetesRuntime,
    nonterminal_pods,
    pod_identity,
    validate_pod_identities,
)
from ..common.kubernetes_pod_proxy import (
    KubernetesPodProxyConfig,
    KubernetesPodProxyEndpoint,
)

if TYPE_CHECKING:
    from kubernetes.client import ApiClient


@dataclass
class KubernetesTargetAccess:
    """Component-owned access, independent of any monitoring strategy."""

    type: ClassVar[str] = "kubernetes_target_access"
    identity: Optional[ComponentKubernetesRuntime] = None
    api_client: Optional["ApiClient"] = None

    def get_client(self) -> "ApiClient":
        if self.api_client is None:
            raise RuntimeError(
                "Kubernetes target access has been released; adopt again"
            )
        return self.api_client

    def validate(self, timeout: float):
        if self.identity is None:
            raise RuntimeError("Kubernetes targets have not been discovered")
        validate_pod_identities(self.identity, timeout, self.get_client())

    def close(self):
        if self.api_client is not None:
            self.api_client.close()
            self.api_client = None


@existing_registry.register_config("kubernetes")
class ExistingKubernetesConfig(ExistingStrategyConfig):
    """Select ready pods and expose optional named endpoints through the API proxy."""

    namespace: str = Field(min_length=1)
    label_selector: str = Field(min_length=1)
    context: Optional[str] = None
    kubeconfig: Optional[str] = None
    endpoints: dict[Annotated[str, Field(min_length=1)], KubernetesPodProxyConfig] = (
        Field(default_factory=dict)
    )


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
        endpoints:
          metrics:
            port: 8888
            path: /metrics
    monitoring:
      prometheus:
        target_endpoint: metrics
""",
    )

    def __init__(self, config: ExistingKubernetesConfig):
        self.config = config

    def resolve(self, component: Component, ctx: StepContext):
        from kubernetes import client, config

        previous = component.runtime.get(KubernetesTargetAccess.type)
        if previous is not None:
            previous.close()
        component.set_targets([])
        access = KubernetesTargetAccess()
        component.set_runtime_data(KubernetesTargetAccess.type, access)
        ctx.get_suite().context.cleanup.callback(access.close)
        with ExitStack() as startup_cleanup:
            startup_cleanup.callback(access.close)
            configuration = client.Configuration()
            configuration.retries = 0
            access.api_client = config.new_client_from_config(
                config_file=self.config.kubeconfig,
                context=self.config.context,
                persist_config=False,
                client_configuration=configuration,
            )
            result = client.CoreV1Api(access.get_client()).list_namespaced_pod(
                namespace=self.config.namespace,
                label_selector=self.config.label_selector,
                _request_timeout=(5, 30),
            )
            live_pods = nonterminal_pods(result.items)
            if not live_pods:
                raise ValueError(
                    f"No existing pods match {self.config.label_selector!r} "
                    f"in namespace {self.config.namespace!r} "
                    "after excluding terminal pods"
                )
            pods = [pod_identity(pod) for pod in live_pods]
            identity = ComponentKubernetesRuntime(
                namespace=self.config.namespace,
                label_selector=self.config.label_selector,
                context=self.config.context,
                kubeconfig=self.config.kubeconfig,
                pods=pods,
            )
            access.identity = identity
            component.set_runtime_data(ComponentKubernetesRuntime.type, identity)
            targets = [
                ComponentTarget(
                    name=pod.name,
                    endpoints={
                        name: KubernetesPodProxyEndpoint(
                            access.get_client, identity.namespace, pod.name, endpoint
                        )
                        for name, endpoint in self.config.endpoints.items()
                    },
                    attributes={
                        "k8s.namespace.name": identity.namespace,
                        "k8s.pod.name": pod.name,
                        "k8s.pod.uid": pod.uid,
                        "k8s.node.name": pod.node_name,
                    },
                )
                for pod in pods
            ]
            component.set_targets(targets, access.validate)
            startup_cleanup.pop_all()
        ctx.get_logger(__name__).info(
            "Observing %d existing Kubernetes pod(s) in %s",
            len(pods),
            self.config.namespace,
        )

    def release(self, component: Component, ctx: StepContext):
        access = component.runtime.get(KubernetesTargetAccess.type)
        if access is not None:
            access.close()
