from enum import Enum


class TestEvent(Enum):
    """Enum of Test Framework lifecycle events."""

    # === Suite-level Events ===
    SUITE_START = "suite_start"
    SUITE_END = "suite_end"

    # === Test-level Events ===
    TEST_START = "test_start"
    TEST_SUCCESS = "test_success"
    TEST_ERROR = "test_error"
    TEST_END = "test_end"

    # === Step-level Events ===
    STEP_START = "step_start"
    STEP_EXECUTE_START = "step_execute_start"
    STEP_EXECUTE_END = "step_execute_end"
    STEP_SUCCESS = "step_success"
    STEP_ERROR = "step_error"
    STEP_END = "step_end"

    # === Hooks ===
    HOOK_START = "hook_start"
    HOOK_END = "hook_end"

    # === Strategy Invocation ===
    STRATEGY_START = "strategy_start"
    STRATEGY_END = "strategy_end"

    # === Reporting ===
    REPORTING_START = "reporting_start"
    REPORTING_END = "reporting_end"

    def namespaced(self, namespace="test_framework"):
        """Return a string representation of the enum, prefixed with namespace"""
        return f"{namespace}.{self.value}"
