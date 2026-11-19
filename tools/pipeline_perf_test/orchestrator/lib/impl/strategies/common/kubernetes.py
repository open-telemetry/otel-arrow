"""Resource identities discovered for a caller-owned Kubernetes component."""

from typing import ClassVar, Dict, List, Literal, Optional, TYPE_CHECKING

from pydantic import BaseModel, Field

if TYPE_CHECKING:
    from kubernetes.client import V1Pod


class KubernetesPodIdentity(BaseModel):
    """A pod and its running regular/restartable-init containers."""

    name: str = Field(min_length=1)
    uid: str = Field(min_length=1)
    node_name: str = Field(min_length=1)
    containers: Dict[str, str] = Field(min_length=1)


class ComponentKubernetesRuntime(BaseModel):
    """Selected pod identities; no cluster resources are owned by the harness."""

    type: ClassVar[Literal["component_kubernetes_runtime"]] = (
        "component_kubernetes_runtime"
    )
    namespace: str
    label_selector: str
    context: Optional[str] = None
    kubeconfig: Optional[str] = None
    pods: List[KubernetesPodIdentity] = Field(min_length=1)


def pod_identity(pod: "V1Pod") -> KubernetesPodIdentity:
    """Validate a ready pod and identify its running containers and sidecars."""
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
    return KubernetesPodIdentity(
        name=name,
        uid=pod.metadata.uid,
        node_name=pod.spec.node_name,
        containers=containers,
    )
