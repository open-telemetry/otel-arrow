"""Resource identities discovered for a caller-owned Kubernetes component."""

from typing import ClassVar, Dict, Iterable, List, Literal, Optional, TYPE_CHECKING

from pydantic import BaseModel, Field

if TYPE_CHECKING:
    from kubernetes.client import ApiClient, V1Pod


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


def validate_pod_identities(
    target: ComponentKubernetesRuntime, timeout: float, api_client: "ApiClient"
) -> None:
    """Revalidate the selected pods and running containers before endpoint access."""
    from kubernetes import client
    from kubernetes.client.exceptions import ApiException
    from kubernetes.config.config_exception import ConfigException
    from urllib3.exceptions import HTTPError

    try:
        current = client.CoreV1Api(api_client).list_namespaced_pod(
            namespace=target.namespace,
            label_selector=target.label_selector,
            _request_timeout=(min(5.0, timeout), timeout),
        )
    except (ApiException, ConfigException, HTTPError) as error:
        raise RuntimeError(f"Could not validate Kubernetes targets: {error}") from error
    identities = [pod_identity(pod) for pod in nonterminal_pods(current.items)]
    expected = {pod.name: pod for pod in target.pods}
    if (
        len(identities) != len(expected)
        or {pod.name: pod for pod in identities} != expected
    ):
        raise ValueError("Kubernetes pod/container identities changed; adopt again")


def nonterminal_pods(pods: Iterable["V1Pod"]) -> list["V1Pod"]:
    """Apply the same live-pod selection during adoption and monitoring."""
    return [pod for pod in pods if pod.status.phase not in ("Succeeded", "Failed")]


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
