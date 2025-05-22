"""
Module: managed_component

Each `ManagedComponent` represents a managed entity in the system, such as a service or
application instance. The managed_component supports full lifecycle management (configure,
deploy, start, stop, destroy), implemented through strategy patterns.

Classes:
    ManagedComponent: Represents a lifecycle-managed testbed component with pluggable
               deployment and monitoring strategies.
"""

from pydantic import BaseModel, Field
from typing import Any, Callable, List, Optional, Union, Literal

from ..strategies.monitoring.composite_monitoring_strategy import (
    CompositeMonitoringStrategy,
)
from ...core.component.runtime import ComponentRuntime
from ...core.component.lifecycle_component import (
    LifecycleComponent,
    HookableLifecyclePhase,
    LifecyclePhase,
)
from ..strategies.deployment.docker import DockerDeploymentConfig
from ...core.test_framework.test_context import TestStepContext, TestExecutionContext


LifecycleHookStrategy = Literal["append", "replace"]


class LifecycleHooks(BaseModel):
    pre: Optional[List[str]] = Field(
        default_factory=list, description="Commands to run before start/stop"
    )
    pre_strategy: LifecycleHookStrategy = "append"
    post: Optional[List[str]] = Field(
        default_factory=list, description="Commands to run after start/stop"
    )
    post_strategy: LifecycleHookStrategy = "append"


class ManagedComponentActioConfig(BaseModel):
    type: Literal["component_action"]
    target: str
    action: LifecyclePhase


class ManagedComponentConfiguration(BaseModel):
    """Base configuration model for the ManagedComponent class"""

    configure_hooks: Optional[LifecycleHooks] = Field(default_factory=LifecycleHooks)
    deploy_hooks: Optional[LifecycleHooks] = Field(default_factory=LifecycleHooks)
    start_hooks: Optional[LifecycleHooks] = Field(default_factory=LifecycleHooks)
    stop_hooks: Optional[LifecycleHooks] = Field(default_factory=LifecycleHooks)
    destroy_hooks: Optional[LifecycleHooks] = Field(default_factory=LifecycleHooks)
    start_monitoring_hooks: Optional[LifecycleHooks] = Field(
        default_factory=LifecycleHooks
    )
    stop_monitoring_hooks: Optional[LifecycleHooks] = Field(
        default_factory=LifecycleHooks
    )

    deployment: Optional[Union[DockerDeploymentConfig]]

    # placeholders
    configuration: Optional[str] = None
    execution: Optional[str] = None
    monitoring: Optional[List[str]] = []


class ManagedComponent(LifecycleComponent):
    """
    An orchestrated component that encapsulates deployment, configuration, and monitoring logic.

    Attributes:
        name (str): Name of the component.
        deployment: A deployment strategy instance responsible for managing deployment operations.
        monitoring (CompositeMonitoringStrategy): Composite strategy for managing monitoring implementations.
        config: Optional configuration strategy used to configure the component.
    """

    def __init__(
        self,
        name: str,
        config: ManagedComponentConfiguration,
    ):
        """
        Initialize the Component.

        Args:
            name (str): Identifier for the component.
            config_strategy: Configuration strategy applied during configure phase.
            deployment_strategy: Strategy object responsible for deploying the component.
            execution_strategy: Strategy object responsible for component logic execution.
            monitoring_strategies (List[MonitoringStrategy]): List of monitoring strategies to be applied.
        """
        super().__init__()
        self.name: str = name
        self.component_config: ManagedComponentConfiguration = config
        self.runtime: ComponentRuntime = ComponentRuntime()

        deployment_strategy = config.deployment.create() if config.deployment else None
        config_strategy = (
            config.configuration.create() if config.configuration else None
        )
        execution_strategy = config.execution.create() if config.execution else None
        monitoring_strategies = [m.create() for m in config.monitoring or []]
        self.configuration = config_strategy
        self.deployment = deployment_strategy
        self.execution_strategy = execution_strategy
        self.monitoring = CompositeMonitoringStrategy(monitoring_strategies)

        if self.deployment.default_hooks:
            for hook_phase in self.deployment.default_hooks:
                for hook in self.deployment.default_hooks[hook_phase]:
                    self.add_hook(hook_phase, hook)

    def get_or_create_runtime(self, namespace: str, factory: Callable[[], Any]) -> Any:
        """Get an existing runtime data structure or initialize a new one.

        Args:
            namespace: The namespace to get/create data for.
            factory: The initialization method if no namespace data exists.
        """
        return self.runtime.get_or_create(namespace, factory)

    def set_runtime_data(self, namespace: str, data: Any):
        """Set the data value on the component's runtime with the specified namespace.

        Args:
            namespace: The namespace to set the data value on.
            data: The data to set.
        """
        self.runtime.set(namespace, data)

    def get_component_config(self) -> BaseModel:
        """Get the component's configuration model.

        Returns: a BaseModel describing the configuration of the component.
        """
        return self.component_config

    def configure(self, ctx: TestStepContext):
        """
        Apply configuration to the component using the configured strategy, if present.
        Executes lifecycle hooks before and after the configuration.
        """
        self._run_hooks(HookableLifecyclePhase.PRE_CONFIGURE, ctx)
        if self.configuration:
            self.configuration.start(self, ctx)
        self._run_hooks(HookableLifecyclePhase.POST_CONFIGURE, ctx)

    def deploy(self, ctx: TestStepContext):
        """
        Deploy the component using the specified deployment strategy.
        Executes lifecycle hooks before and after deployment.
        """
        self._run_hooks(HookableLifecyclePhase.PRE_DEPLOY, ctx)
        if self.deployment:
            self.deployment.start(self, ctx)
        self._run_hooks(HookableLifecyclePhase.POST_DEPLOY, ctx)

    def start(self, ctx: TestStepContext):
        """
        Start the deployed component using the execution strategy.
        Executes lifecycle hooks before and after starting.
        """
        self._run_hooks(HookableLifecyclePhase.PRE_START, ctx)
        if self.execution_strategy:
            self.execution_strategy.start(self, ctx)
        self._run_hooks(HookableLifecyclePhase.POST_START, ctx)

    def stop(self, ctx: TestStepContext):
        """
        Stop the running component using the execution strategy.
        Executes lifecycle hooks before and after stopping.
        """
        self._run_hooks(HookableLifecyclePhase.PRE_STOP, ctx)
        if self.execution_strategy:
            self.execution_strategy.stop(self, ctx)
        self._run_hooks(HookableLifecyclePhase.POST_STOP, ctx)

    def destroy(self, ctx: TestStepContext):
        """
        Tear down and clean up the component using the deployment strategy.
        Executes lifecycle hooks before and after destruction.
        """
        self._run_hooks(HookableLifecyclePhase.PRE_DESTROY, ctx)
        if self.deployment:
            self.deployment.stop(self, ctx)
        self._run_hooks(HookableLifecyclePhase.POST_DESTROY, ctx)

    def start_monitoring(self, ctx: TestStepContext):
        """
        Start all monitoring strategies associated with the component.
        Executes lifecycle hooks before and after monitoring startup.
        """
        self._run_hooks(HookableLifecyclePhase.PRE_START_MONITORING, ctx)
        self.monitoring.start(self, ctx)
        self._run_hooks(HookableLifecyclePhase.POST_START_MONITORING, ctx)

    def stop_monitoring(self, ctx: TestStepContext):
        """
        Stop all monitoring strategies associated with the component.
        Executes lifecycle hooks before and after monitoring shutdown.
        """
        self._run_hooks(HookableLifecyclePhase.PRE_STOP_MONITORING, ctx)
        self.monitoring.stop(self, ctx)
        self._run_hooks(HookableLifecyclePhase.POST_STOP_MONITORING, ctx)

    def collect_monitoring_data(self, ctx: TestExecutionContext) -> dict:
        """
        Collect and return monitoring data from all configured monitoring strategies.

        Returns:
            dict: Aggregated monitoring data.
        """
        return self.monitoring.collect(self, ctx)
