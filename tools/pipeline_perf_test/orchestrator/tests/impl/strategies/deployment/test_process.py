import subprocess
import unittest
from unittest.mock import MagicMock, patch
from lib.impl.strategies.deployment.process import (
    ProcessDeployment,
    ProcessDeploymentConfig,
)
import os


class TestProcessDeployment(unittest.TestCase):

    @patch("lib.impl.strategies.deployment.process.subprocess.Popen")
    @patch("lib.impl.strategies.deployment.process.Component.get_or_create_runtime")
    @patch("lib.impl.strategies.deployment.process.Component.set_runtime_data")
    @patch("lib.impl.strategies.deployment.process.StepContext.get_logger")
    def test_start_valid_process(
        self,
        mock_get_logger,
        mock_set_runtime_data,
        mock_get_or_create_runtime,
        mock_popen,
    ):
        # Setup mocks
        mock_logger = MagicMock()
        mock_get_logger.return_value = mock_logger

        # Mock the component runtime
        mock_component = MagicMock()
        mock_runtime = MagicMock()
        mock_component.get_or_create_runtime.return_value = mock_runtime

        # Configure Popen to mock a process
        mock_process = MagicMock()
        mock_process.pid = 1234
        mock_popen.return_value = mock_process

        # Sample valid configuration
        config = ProcessDeploymentConfig(
            command="echo 'hello world'", environment={"KEY": "VALUE"}
        )
        process_deployment = ProcessDeployment(config=config)

        # Call the start method
        process_deployment.start(mock_component, MagicMock())

        # Assertions
        mock_popen.assert_called_once_with(
            ["echo", "hello world"],
            shell=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env={**os.environ, **{"KEY": "VALUE"}},
        )

        mock_component.set_runtime_data.assert_called_once_with(
            "component_process_runtime", mock_runtime
        )
        self.assertEqual(mock_runtime.pid, 1234)

    @patch("lib.impl.strategies.deployment.process.subprocess.Popen")
    @patch("lib.impl.strategies.deployment.process.Component.get_or_create_runtime")
    @patch("lib.impl.strategies.deployment.process.Component.set_runtime_data")
    @patch("lib.impl.strategies.deployment.process.StepContext.get_logger")
    def test_start_invalid_command(
        self,
        mock_get_logger,
        mock_set_runtime_data,
        mock_get_or_create_runtime,
        mock_popen,
    ):
        # Setup mocks
        mock_logger = MagicMock()
        mock_get_logger.return_value = mock_logger

        # Mock the component runtime
        mock_component = MagicMock()
        mock_runtime = MagicMock()
        mock_component.get_or_create_runtime.return_value = mock_runtime
        mock_popen.side_effect = FileNotFoundError("Command not found")

        # Configure an invalid command
        config = ProcessDeploymentConfig(
            command="invalidcommand", environment={"KEY": "VALUE"}
        )
        process_deployment = ProcessDeployment(config=config)

        # Ensure the error is raised when start is called
        with self.assertRaises(FileNotFoundError):
            process_deployment.start(mock_component, MagicMock())

    # Scenario: stop() is called on a running process-based component.
    # Guarantees: the component's whole process tree is waited on / terminated via
    # wait_or_terminate_process_tree with the process's PID, so descendants are not
    # left orphaned.
    @patch("lib.impl.strategies.deployment.process.wait_or_terminate_process_tree")
    @patch("lib.impl.strategies.deployment.process.subprocess.Popen")
    @patch("lib.impl.strategies.deployment.process.Component.get_or_create_runtime")
    @patch("lib.impl.strategies.deployment.process.Component.set_runtime_data")
    @patch("lib.impl.strategies.deployment.process.StepContext.get_logger")
    def test_stop_process_terminates_tree(
        self,
        mock_get_logger,
        mock_set_runtime_data,
        mock_get_or_create_runtime,
        mock_popen,
        mock_wait_or_terminate_tree,
    ):
        # Setup mocks
        mock_logger = MagicMock()
        mock_get_logger.return_value = mock_logger

        # Mock the component runtime with a running process
        mock_component = MagicMock()
        mock_runtime = MagicMock()
        mock_process = MagicMock()
        mock_process.pid = 1234
        mock_runtime.process = mock_process
        mock_component.get_or_create_runtime.return_value = mock_runtime

        # Ensure the process tree is waited on / terminated
        process_deployment = ProcessDeployment(
            config=ProcessDeploymentConfig(command="echo 'hello world'")
        )
        process_deployment.stop(mock_component, MagicMock())

        # Assert the whole process tree was handled via the helper.
        mock_wait_or_terminate_tree.assert_called_once()
        self.assertEqual(mock_wait_or_terminate_tree.call_args.args[0], 1234)

    @patch("lib.impl.strategies.deployment.process.subprocess.Popen")
    @patch("lib.impl.strategies.deployment.process.Component.get_or_create_runtime")
    @patch("lib.impl.strategies.deployment.process.Component.set_runtime_data")
    @patch("lib.impl.strategies.deployment.process.StepContext.get_logger")
    def test_start_missing_environment(
        self,
        mock_get_logger,
        mock_set_runtime_data,
        mock_get_or_create_runtime,
        mock_popen,
    ):
        # Setup mocks
        mock_logger = MagicMock()
        mock_get_logger.return_value = mock_logger

        # Mock the component runtime
        mock_component = MagicMock()
        mock_runtime = MagicMock()
        mock_component.get_or_create_runtime.return_value = mock_runtime

        # Mock process creation
        mock_process = MagicMock()
        mock_process.pid = 1234
        mock_popen.return_value = mock_process

        # Configure ProcessDeploymentConfig with no environment
        config = ProcessDeploymentConfig(command="echo 'hello world'")
        process_deployment = ProcessDeployment(config=config)

        # Call start
        process_deployment.start(mock_component, MagicMock())

        # Check if Popen is called with an environment based on os.environ only (empty environment)
        mock_popen.assert_called_once_with(
            ["echo", "hello world"],
            shell=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=os.environ,
        )

    @patch("lib.impl.strategies.deployment.process.subprocess.Popen")
    @patch("lib.impl.strategies.deployment.process.Component.get_or_create_runtime")
    @patch("lib.impl.strategies.deployment.process.Component.set_runtime_data")
    @patch("lib.impl.strategies.deployment.process.StepContext.get_logger")
    def test_start_empty_environment(
        self,
        mock_get_logger,
        mock_set_runtime_data,
        mock_get_or_create_runtime,
        mock_popen,
    ):
        # Setup mocks
        mock_logger = MagicMock()
        mock_get_logger.return_value = mock_logger

        # Mock the component runtime
        mock_component = MagicMock()
        mock_runtime = MagicMock()
        mock_component.get_or_create_runtime.return_value = mock_runtime

        # Mock process creation
        mock_process = MagicMock()
        mock_process.pid = 1234
        mock_popen.return_value = mock_process

        # Configure ProcessDeploymentConfig with empty environment
        config = ProcessDeploymentConfig(command="echo 'hello world'", environment={})
        process_deployment = ProcessDeployment(config=config)

        # Call start
        process_deployment.start(mock_component, MagicMock())

        # Check if Popen is called with an environment based on os.environ only (empty environment)
        mock_popen.assert_called_once_with(
            ["echo", "hello world"],
            shell=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=os.environ,
        )

    # Scenario: stop() is called on a running process-based component.
    # Guarantees: the process is asked to stop gracefully (terminate) and its tree
    # is waited on / terminated, and its stdout/stderr are drained via communicate.
    @patch("lib.impl.strategies.deployment.process.wait_or_terminate_process_tree")
    @patch("lib.impl.strategies.deployment.process.subprocess.Popen")
    @patch("lib.impl.strategies.deployment.process.Component.get_or_create_runtime")
    @patch("lib.impl.strategies.deployment.process.Component.set_runtime_data")
    @patch("lib.impl.strategies.deployment.process.StepContext.get_logger")
    def test_stop_valid_process(
        self,
        mock_get_logger,
        mock_set_runtime_data,
        mock_get_or_create_runtime,
        mock_popen,
        mock_wait_or_terminate_tree,
    ):
        # Setup mocks
        mock_logger = MagicMock()
        mock_get_logger.return_value = mock_logger

        # Mock the component runtime with a running process
        mock_component = MagicMock()
        mock_runtime = MagicMock()
        mock_process = MagicMock()
        mock_process.pid = 1234
        mock_runtime.process = mock_process
        mock_component.get_or_create_runtime.return_value = mock_runtime

        # Simulate successful process termination
        mock_process.terminate.return_value = None

        # Call stop method
        process_deployment = ProcessDeployment(
            config=ProcessDeploymentConfig(command="echo 'hello world'")
        )
        process_deployment.stop(mock_component, MagicMock())

        # Check that terminate() was called and the tree was cleaned up.
        mock_process.terminate.assert_called_once()
        mock_wait_or_terminate_tree.assert_called_once()
        mock_process.communicate.assert_called_once()

    # Scenario: stop() delegates process-tree teardown to the shared helper.
    # Guarantees: wait_or_terminate_process_tree is invoked with the process PID so
    # the configured normal/graceful escalation (not just a direct-child kill) runs.
    @patch("lib.impl.strategies.deployment.process.wait_or_terminate_process_tree")
    @patch("lib.impl.strategies.deployment.process.subprocess.Popen")
    @patch("lib.impl.strategies.deployment.process.Component.get_or_create_runtime")
    @patch("lib.impl.strategies.deployment.process.Component.set_runtime_data")
    @patch("lib.impl.strategies.deployment.process.StepContext.get_logger")
    def test_stop_waits_on_tree_with_pid(
        self,
        mock_get_logger,
        mock_set_runtime_data,
        mock_get_or_create_runtime,
        mock_popen,
        mock_wait_or_terminate_tree,
    ):
        # Setup mocks
        mock_logger = MagicMock()
        mock_get_logger.return_value = mock_logger

        # Mock the component runtime with a running process
        mock_component = MagicMock()
        mock_runtime = MagicMock()
        mock_process = MagicMock()
        mock_process.pid = 1234
        mock_runtime.process = mock_process
        mock_component.get_or_create_runtime.return_value = mock_runtime

        # Call stop method
        process_deployment = ProcessDeployment(
            config=ProcessDeploymentConfig(command="echo 'hello world'")
        )
        process_deployment.stop(mock_component, MagicMock())

        # Assert the helper was called with the process PID.
        mock_wait_or_terminate_tree.assert_called_once()
        self.assertEqual(mock_wait_or_terminate_tree.call_args.args[0], 1234)

    @patch("lib.impl.strategies.deployment.process.subprocess.Popen")
    @patch("lib.impl.strategies.deployment.process.Component.get_or_create_runtime")
    @patch("lib.impl.strategies.deployment.process.Component.set_runtime_data")
    @patch("lib.impl.strategies.deployment.process.StepContext.get_logger")
    def test_stop_process_already_stopped(
        self,
        mock_get_logger,
        mock_set_runtime_data,
        mock_get_or_create_runtime,
        mock_popen,
    ):
        # Setup mocks
        mock_logger = MagicMock()
        mock_get_logger.return_value = mock_logger

        # Mock the component runtime with no process running
        mock_component = MagicMock()
        mock_runtime = MagicMock()
        mock_runtime.process = None  # No process running
        mock_component.get_or_create_runtime.return_value = mock_runtime

        # Call stop method
        process_deployment = ProcessDeployment(
            config=ProcessDeploymentConfig(command="echo 'hello world'")
        )

        # Ensure RuntimeError is raised when stopping with no process
        with self.assertRaises(RuntimeError):
            process_deployment.stop(mock_component, MagicMock())
