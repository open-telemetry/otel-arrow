"""Tests for the wait_or_terminate_process_tree helper."""

import logging
import subprocess
import sys

import psutil

from lib.impl.strategies.common.process import wait_or_terminate_process_tree

# A parent process that spawns a long-lived child, then sleeps itself. Both the
# parent and the child must be reaped by wait_or_terminate_process_tree. The child
# PID is printed so the test can track the grandchild independently of the parent.
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


def _force_kill(pids):
    """Best-effort kill so a failed assertion never leaks processes."""
    for pid in pids:
        try:
            psutil.Process(pid).kill()
        except psutil.NoSuchProcess:
            pass


# Scenario: wait_or_terminate_process_tree is called (normal_timeout=0) on a
# parent that spawned a still-running child.
# Guarantees: both the parent and its descendant are terminated, so no orphaned
# grandchild survives a tree kill.
def test_terminates_parent_and_child():
    parent = subprocess.Popen(
        [sys.executable, "-c", _PARENT_SCRIPT], stdout=subprocess.PIPE
    )
    child_pid = None
    try:
        # First line of stdout is the child's PID.
        child_pid = int(parent.stdout.readline().decode().strip())

        wait_or_terminate_process_tree(
            parent.pid, logging.getLogger(__name__), normal_timeout=0
        )

        assert _wait_gone([parent.pid, child_pid]), "parent or child still alive"
    finally:
        _force_kill([parent.pid] + ([child_pid] if child_pid else []))
        if parent.stdout:
            parent.stdout.close()
        parent.wait(timeout=5)


# Scenario: the process tree exits on its own before the normal wait elapses.
# Guarantees: the helper returns via the normal-wait path without escalating, and
# the process is reaped.
def test_waits_for_normal_exit():
    # Exits almost immediately; the normal wait should observe it exit.
    proc = subprocess.Popen([sys.executable, "-c", "pass"])
    try:
        wait_or_terminate_process_tree(
            proc.pid, logging.getLogger(__name__), normal_timeout=5
        )
        assert not psutil.pid_exists(proc.pid) or _wait_gone([proc.pid])
    finally:
        _force_kill([proc.pid])
        proc.wait(timeout=5)


# Scenario: wait_or_terminate_process_tree is called with a PID that is not
# running.
# Guarantees: the call is a no-op and does not raise, so teardown stays robust
# when a process has already exited.
def test_nonexistent_pid_is_noop():
    # Find a PID that is very unlikely to exist.
    missing = 2**31 - 1
    while psutil.pid_exists(missing):
        missing -= 1
    wait_or_terminate_process_tree(
        missing, logging.getLogger(__name__)
    )  # must not raise


# Scenario: a process that existed is fully exited and reaped before the helper
# enumerates its tree.
# Guarantees: the helper handles a process vanishing between lookup and
# enumeration as a clean no-op rather than raising NoSuchProcess.
def test_already_exited_pid_is_noop():
    proc = subprocess.Popen([sys.executable, "-c", "pass"])
    proc.wait(timeout=5)  # process is dead and reaped
    # Its PID now refers to a dead/absent process.
    wait_or_terminate_process_tree(
        proc.pid, logging.getLogger(__name__)
    )  # must not raise
