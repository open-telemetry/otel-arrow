"""Resource identities discovered for a caller-owned Kubernetes component."""

from typing import ClassVar, Dict, List, Literal, Optional

from pydantic import BaseModel, Field


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
    context: Optional[str] = None
    pods: List[KubernetesPodIdentity] = Field(min_length=1)
