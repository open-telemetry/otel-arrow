"""
Module: managed_component

Each `ManagedComponent` represents a managed entity in the system, such as a service or
application instance. The managed_component supports full lifecycle management (configure,
deploy, start, stop, destroy), implemented through strategy patterns.

Classes:
    ManagedComponentConfiguration: Base configuration model for the ManagedComponent class
    ManagedComponent: Represents a lifecycle-managed testbed component with pluggable
               deployment and monitoring strategies.
"""

from typing import Any, Optional, Dict

from pydantic import BaseModel, Field, model_validator

from ...core.component.component import (
    Component,
    HookableComponentPhase,
    ComponentPhase,
)
from ...core.errors.error_handler import OnErrorConfig
from ...core.context.framework_element_contexts import StepContext, ScenarioContext
from ...core.strategies.base import BaseStrategy
from ...core.strategies.execution_strategy import (
    ExecutionStrategy,
    ExecutionStrategyConfig,
)
from ...core.strategies.configuration_strategy import (
    ConfigurationStrategy,
    ConfigurationStrategyConfig,
)
from ...core.strategies.deployment_strategy import (
    DeploymentStrategy,
    DeploymentStrategyConfig,
)
from ...runner.wrappers import (
    DeploymentWrapper,
    MonitoringWrapper,
    ExecutionWrapper,
    ConfigurationWrapper,
)
from ...core.telemetry.framework_event import FrameworkEvent
from ...core.strategies.monitoring_strategy import MonitoringStrategy
from ...runner.registry import monitoring_registry
from ...runner.schema.hook_config import HooksConfig


class ManagedComponentConfiguration(BaseModel):
    """
    Configuration model for a managed component, capturing lifecycle hooks,
    deployment, monitoring, execution, and error handling strategies.

    Attributes:
        hooks (Dict[ComponentPhase, HooksConfig]): A mapping of component lifecycle phases
            (e.g., setup, teardown) to their associated hooks configuration.
        deployment (Optional[DeploymentWrapper]): Optional deployment strategy wrapper
            detailing how the component is deployed (e.g., Docker, Kubernetes).
        monitoring (Optional[Dict[str, MonitoringWrapper]]): Optional dictionary mapping
            monitoring strategy names to their configuration wrappers, supporting multiple
            monitoring strategies per component.
        configuration (Optional[ConfigurationWrapper]): Optional wrapper for general
            component configuration parameters.
        execution (Optional[ExecutionWrapper]): Optional wrapper for execution-related
            strategy or resource management configurations.
        on_error (Optional[OnErrorConfig]): Configuration defining error handling behavior,
            with a default empty configuration if not provided.

    Validators:
        parse_monitoring_section (classmethod): A Pydantic model validator (executed before
            validation) that processes the 'monitoring' section from raw input data. It:
              - Ensures monitoring strategies are recognized and valid.
              - Instantiates corresponding MonitoringWrapper objects with the proper
                strategy type and config.
              - Raises a ValueError if an unknown monitoring strategy is encountered.
    """

    hooks: Dict[ComponentPhase, HooksConfig] = Field(default_factory=dict)
    deployment: Optional[DeploymentWrapper] = None
    monitoring: Optional[Dict[str, MonitoringWrapper]] = None
    configuration: Optional[ConfigurationWrapper] = None
    execution: Optional[ExecutionWrapper] = None
    on_error: Optional[OnErrorConfig] = Field(default_factory=OnErrorConfig)

    @model_validator(mode="before")
    @classmethod
    def parse_monitoring_section(cls, data: Any) -> Any:
        """
        Pre-validation hook that parses the raw 'monitoring' section from input data,
        converting it into structured MonitoringWrapper instances.

        Args:
            data (Any): Raw input data dictionary for the model.

        Returns:
            Any: The updated data dictionary with the 'monitoring' key
                converted to a dictionary of MonitoringWrapper instances.

        Raises:
            ValueError: If an unknown monitoring strategy key is encountered.
        """
        monitoring = data.get("monitoring")
        if monitoring and isinstance(monitoring, dict):
            parsed = {}
            for strategy_type, config_data in monitoring.items():
                config_cls = monitoring_registry.config.get(strategy_type)
                if not config_cls:
                    raise ValueError(f"Unknown monitoring strategy: '{strategy_type}'")
                strategy_config = config_cls(**config_data)
                parsed[strategy_type] = MonitoringWrapper(
                    element_type=strategy_type, config=strategy_config
                )
            data["monitoring"] = parsed
        return data


class ManagedComponent(Component):
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
        configuration_strategy: ConfigurationStrategy,
        deployment_strategy: DeploymentStrategy,
        monitoring_strategy: MonitoringStrategy,
        execution_strategy: ExecutionStrategy,
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

        self.configuration: ConfigurationStrategy = configuration_strategy
        self.deployment: DeploymentStrategy = deployment_strategy
        self.execution_strategy: ExecutionStrategy = execution_strategy
        self.monitoring: MonitoringStrategy = monitoring_strategy

        if self.deployment and self.deployment.default_component_hooks:
            for hook_phase in self.deployment.default_component_hooks:
                for hook in self.deployment.default_component_hooks[hook_phase]:
                    self.add_hook(hook_phase, hook)

    def replace_strategy(self, strategy: BaseStrategy) -> bool:
        """Replace a strategy instance on the component.

        Args:
            strategy: BaseStrategy to replace into the component.
        Returns:
            true if successful, else false
        """
        if isinstance(strategy, ConfigurationStrategy):
            self.configuration = strategy
        elif isinstance(strategy, DeploymentStrategy):
            self.deployment = strategy
        elif isinstance(strategy, ExecutionStrategy):
            self.execution_strategy = strategy
        elif isinstance(strategy, MonitoringStrategy):
            self.monitoring = strategy
        else:
            return False
        return True

    def get_component_config(self) -> BaseModel:
        """Get the component's configuration model.

        Returns: a BaseModel describing the configuration of the component.
        """
        return self.component_config

    def get_deployment_config(self) -> DeploymentStrategyConfig:
        return self.component_config.deployment.config

    def get_configuration_config(self) -> ConfigurationStrategyConfig:
        return self.component_config.configuration.config

    def get_execution_config(self) -> ExecutionStrategyConfig:
        return self.component_config.execution.config

    def _configure(self, ctx: StepContext):
        """
        Apply configuration to the component using the configured strategy, if present.
        Executes lifecycle hooks before and after the configuration.
        """
        self._run_hooks(HookableComponentPhase.PRE_CONFIGURE, ctx)
        if self.configuration:
            ctx.record_event(
                FrameworkEvent.STRATEGY_START.namespaced(),
                **{"ctx.phase": "configuration", "ctx.component": self.name},
            )
            self._with_span(
                ctx, "configuration.start", lambda: self.configuration.start(self, ctx)
            )
            ctx.record_event(
                FrameworkEvent.STRATEGY_END.namespaced(),
                **{"ctx.phase": "configuration", "ctx.component": self.name},
            )
        self._run_hooks(HookableComponentPhase.POST_CONFIGURE, ctx)

    def _deploy(self, ctx: StepContext):
        """
        Deploy the component using the specified deployment strategy.
        Executes lifecycle hooks before and after deployment.
        """
        self._run_hooks(HookableComponentPhase.PRE_DEPLOY, ctx)
        if self.deployment:
            ctx.record_event(
                FrameworkEvent.STRATEGY_START.namespaced(),
                **{"ctx.phase": "deploy", "ctx.component": self.name},
            )
            self._with_span(
                ctx, "deployment.start", lambda: self.deployment.start(self, ctx)
            )
            ctx.record_event(
                FrameworkEvent.STRATEGY_END.namespaced(),
                **{"ctx.phase": "deploy", "ctx.component": self.name},
            )
        self._run_hooks(HookableComponentPhase.POST_DEPLOY, ctx)

    def _start(self, ctx: StepContext):
        """
        Start the deployed component using the execution strategy.
        Executes lifecycle hooks before and after starting.
        """
        self._run_hooks(HookableComponentPhase.PRE_START, ctx)
        if self.execution_strategy:
            ctx.record_event(
                FrameworkEvent.STRATEGY_START.namespaced(),
                **{"ctx.phase": "start", "ctx.component": self.name},
            )
            self._with_span(
                ctx,
                "execution_strategy.start",
                lambda: self.execution_strategy.start(self, ctx),
            )
            ctx.record_event(
                FrameworkEvent.STRATEGY_END.namespaced(),
                **{"ctx.phase": "start", "ctx.component": self.name},
            )
        self._run_hooks(HookableComponentPhase.POST_START, ctx)

    def _stop(self, ctx: StepContext):
        """
        Stop the running component using the execution strategy.
        Executes lifecycle hooks before and after stopping.
        """
        self._run_hooks(HookableComponentPhase.PRE_STOP, ctx)
        if self.execution_strategy:
            ctx.record_event(
                FrameworkEvent.STRATEGY_START.namespaced(),
                **{"ctx.phase": "stop", "ctx.component": self.name},
            )
            self.execution_strategy.stop(self, ctx)
            self._with_span(
                ctx,
                "execution_strategy.stop",
                lambda: self.execution_strategy.stop(self, ctx),
            )
            ctx.record_event(
                FrameworkEvent.STRATEGY_END.namespaced(),
                **{"ctx.phase": "stop", "ctx.component": self.name},
            )
        self._run_hooks(HookableComponentPhase.POST_STOP, ctx)

    def _destroy(self, ctx: StepContext):
        """
        Tear down and clean up the component using the deployment strategy.
        Executes lifecycle hooks before and after destruction.
        """
        self._run_hooks(HookableComponentPhase.PRE_DESTROY, ctx)
        if self.deployment:
            ctx.record_event(
                FrameworkEvent.STRATEGY_START.namespaced(),
                **{"ctx.phase": "destroy", "ctx.component": self.name},
            )
            self._with_span(
                ctx, "deployment.stop", lambda: self.deployment.stop(self, ctx)
            )
            ctx.record_event(
                FrameworkEvent.STRATEGY_END.namespaced(),
                **{"ctx.phase": "destroy", "ctx.component": self.name},
            )
        self._run_hooks(HookableComponentPhase.POST_DESTROY, ctx)

    def _start_monitoring(self, ctx: StepContext):
        """
        Start all monitoring strategies associated with the component.
        Executes lifecycle hooks before and after monitoring startup.
        """
        self._run_hooks(HookableComponentPhase.PRE_START_MONITORING, ctx)
        ctx.record_event(
            FrameworkEvent.STRATEGY_START.namespaced(),
            **{"ctx.phase": "start_monitoring", "ctx.component": self.name},
        )
        self._with_span(
            ctx, "monitoring.start", lambda: self.monitoring.start(self, ctx)
        )
        ctx.record_event(
            FrameworkEvent.STRATEGY_END.namespaced(),
            **{"ctx.phase": "start_monitoring", "ctx.component": self.name},
        )
        self._run_hooks(HookableComponentPhase.POST_START_MONITORING, ctx)

    def _stop_monitoring(self, ctx: StepContext):
        """
        Stop all monitoring strategies associated with the component.
        Executes lifecycle hooks before and after monitoring shutdown.
        """
        self._run_hooks(HookableComponentPhase.PRE_STOP_MONITORING, ctx)
        ctx.record_event(
            FrameworkEvent.STRATEGY_START.namespaced(),
            **{"ctx.phase": "stop_monitoring", "ctx.component": self.name},
        )
        self._with_span(ctx, "monitoring.stop", lambda: self.monitoring.stop(self, ctx))
        ctx.record_event(
            FrameworkEvent.STRATEGY_END.namespaced(),
            **{"ctx.phase": "stop_monitoring", "ctx.component": self.name},
        )
        self._run_hooks(HookableComponentPhase.POST_STOP_MONITORING, ctx)

    def _collect_monitoring_data(self, ctx: ScenarioContext) -> dict:
        """
        Collect and return monitoring data from all configured monitoring strategies.

        Returns:
            dict: Aggregated monitoring data.
        """
        ctx.record_event(
            FrameworkEvent.STRATEGY_START.namespaced(),
            **{"ctx.phase": "collect_monitoring_data", "ctx.component": self.name},
        )
        ret = self.monitoring.collect(self, ctx)
        ctx.record_event(
            FrameworkEvent.STRATEGY_END.namespaced(),
            **{"ctx.phase": "collect_monitoring_data", "ctx.component": self.name},
        )
        return ret
