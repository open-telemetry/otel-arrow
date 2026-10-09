"""Tests for the run_command hook."""

import logging
import subprocess
import sys

import psutil
import pytest
from pydantic import ValidationError

from lib.impl.strategies.hooks.run_command import RunCommandConfig, RunCommandHook


class DummyContext:
    def get_logger(self, name):
        return logging.getLogger(name)


# Scenario: a run_command config is created with a non-positive timeout.
# Guarantees: zero and negative timeouts are rejected so the bound cannot be
# disabled or made to fire immediately.
@pytest.mark.parametrize("bad_timeout", [0, -1, -0.5])
def test_rejects_non_positive_timeout(bad_timeout):
    with pytest.raises(ValidationError):
        RunCommandConfig(command="echo hi", timeout=bad_timeout)


# Scenario: a run_command config is created with a valid positive timeout.
# Guarantees: a positive timeout is accepted and stored.
def test_accepts_positive_timeout():
    assert RunCommandConfig(command="echo hi", timeout=5.0).timeout == 5.0


def _run(command, timeout=30.0):
    hook = RunCommandHook(RunCommandConfig(command=command, timeout=timeout))
    hook.execute(DummyContext())


# Scenario: run_command executes a command that exits successfully.
# Guarantees: a zero-exit command completes without raising.
def test_successful_command():
    _run(f'{sys.executable} -c "import sys; sys.exit(0)"')


# Scenario: run_command executes a command that exits non-zero.
# Guarantees: the hook preserves check=True semantics by raising
# CalledProcessError, so a failing command fails the step.
def test_nonzero_exit_raises():
    with pytest.raises(subprocess.CalledProcessError):
        _run(f'{sys.executable} -c "import sys; sys.exit(3)"')


# Scenario: run_command times out while the shell command has spawned a
# long-lived child process.
# Guarantees: TimeoutExpired is raised and the spawned descendant is terminated,
# so a hung command cannot leave orphaned processes behind.
def test_timeout_kills_descendants(tmp_path):
    # The command spawns a child that sleeps, writes the child's PID to a file,
    # then sleeps itself so the hook's timeout fires.
    pid_file = tmp_path / "child.pid"
    script = (
        "import subprocess, sys, time, pathlib;"
        "c = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)']);"
        f"pathlib.Path(r'{pid_file}').write_text(str(c.pid));"
        "time.sleep(60)"
    )
    command = f'{sys.executable} -c "{script}"'

    child_pid = None
    try:
        with pytest.raises(subprocess.TimeoutExpired):
            _run(command, timeout=1.0)

        child_pid = int(pid_file.read_text().strip())
        try:
            child = psutil.Process(child_pid)
        except psutil.NoSuchProcess:
            return  # Already gone - descendant was reaped.
        _, alive = psutil.wait_procs([child], timeout=5.0)
        assert not alive, "spawned descendant was not terminated"
    finally:
        # Safety net: never leak the spawned child if anything above fails.
        if child_pid is not None:
            try:
                psutil.Process(child_pid).kill()
            except psutil.NoSuchProcess:
                pass
