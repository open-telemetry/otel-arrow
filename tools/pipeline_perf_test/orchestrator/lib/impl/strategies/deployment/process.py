from pydantic import BaseModel
from typing import ClassVar, Literal, Optional


class ProcessRuntime(BaseModel):
    """Base Model for generic process runtime information."""

    type: ClassVar[Literal["process_runtime"]] = "process_runtime"
    logs: Optional[str] = None
