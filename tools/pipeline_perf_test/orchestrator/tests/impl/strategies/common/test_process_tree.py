"""Tests for the terminate_process_tree helper."""

import logging
import subprocess
import sys

import psutil

from lib.impl.strategies.common.process import terminate_process_tree

# A parent process that spawns a long-lived child, then sleeps itself. Both the
# parent and the child must be reaped by terminate_process_tree. The child PID is
# printed so the test can track the grandchild independently of the parent.
_PARENT_SCRIPT = (
    "import subprocess, sys, time;"
    "child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)']);"
    "print(child.pid, flush=True);"
    "time.sleep(60)"
)


def _wait_gone(pids, timeout=5.0):
    """Return True once none of the given PIDs correspond to live processes."""
    procs = []
    for pid in pids:
        try:
            procs.append(psutil.Process(pid))
        except psutil.NoSuchProcess:
            continue
    _, alive = psutil.wait_procs(procs, timeout=timeout)
    return not alive


# Scenario: terminate_process_tree is called on a parent that spawned a child.
# Guarantees: both the parent and its descendant are terminated, so no orphaned
# grandchild survives a tree kill.
def test_terminates_parent_and_child():
    parent = subprocess.Popen(
        [sys.executable, "-c", _PARENT_SCRIPT], stdout=subprocess.PIPE
    )
    try:
        # First line of stdout is the child's PID.
        child_pid = int(parent.stdout.readline().decode().strip())

        terminate_process_tree(parent.pid, logging.getLogger(__name__))

        assert _wait_gone([parent.pid, child_pid]), "parent or child still alive"
    finally:
        # Safety net so a failed assertion never leaks processes.
        for pid in (parent.pid,):
            try:
                psutil.Process(pid).kill()
            except psutil.NoSuchProcess:
                pass
        if parent.stdout:
            parent.stdout.close()
        parent.wait(timeout=5)


# Scenario: terminate_process_tree is called with a PID that is not running.
# Guarantees: the call is a no-op and does not raise, so teardown stays robust
# when a process has already exited.
def test_nonexistent_pid_is_noop():
    # Find a PID that is very unlikely to exist.
    missing = 2**31 - 1
    while psutil.pid_exists(missing):
        missing -= 1
    terminate_process_tree(missing, logging.getLogger(__name__))  # must not raise
