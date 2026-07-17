"""Helpers for best-effort Linux perf support."""

import platform
import subprocess
import threading
from logging import LoggerAdapter
from shutil import which

_perf_install_attempted = False
_perf_install_lock = threading.Lock()

_sudo_checked = False
_sudo_available = False
_sudo_lock = threading.Lock()


def passwordless_sudo_available() -> bool:
    """Return True if `sudo -n` can run a command without prompting (cached).

    The result is cached for the lifetime of the process because sudo
    availability does not change during a test run.
    """

    global _sudo_checked, _sudo_available
    with _sudo_lock:
        if _sudo_checked:
            return _sudo_available
        _sudo_checked = True

        if platform.system() != "Linux" or which("sudo") is None:
            _sudo_available = False
            return _sudo_available

        try:
            result = subprocess.run(
                ["sudo", "-n", "true"],
                check=False,
                capture_output=True,
                text=True,
            )
            _sudo_available = result.returncode == 0
        except Exception:
            _sudo_available = False
        return _sudo_available


def perf_privilege_prefix() -> list[str]:
    """Return the command prefix used to grant perf access to counters.

    Reading hardware counters and attaching to processes owned by other users
    (for example the root-owned main process of a Docker container) generally
    requires elevated privileges or a permissive `perf_event_paranoid` setting.
    When passwordless sudo is available we prefix perf invocations with
    `sudo -n`; otherwise we return an empty prefix and rely on the caller's own
    privileges.
    """

    if passwordless_sudo_available():
        return ["sudo", "-n"]
    return []


def read_perf_event_paranoid() -> str | None:
    """Return the value of /proc/sys/kernel/perf_event_paranoid, if readable."""

    try:
        with open("/proc/sys/kernel/perf_event_paranoid", encoding="utf-8") as fh:
            return fh.read().strip()
    except OSError:
        return None


def ensure_perf_binary(logger: LoggerAdapter) -> str | None:
    """Return the perf binary path, attempting a one-time noninteractive install.

    The install path is best-effort and only runs when all of the following are true:
    - running on Linux,
    - `perf` is not already on PATH,
    - passwordless `sudo` is available,
    - `apt-get` is available.

    This keeps local runs from hanging on sudo prompts while still allowing CI to
    self-heal if the runner image omits perf.
    """

    perf_bin = which("perf")
    if perf_bin:
        return perf_bin

    if platform.system() != "Linux":
        return None

    global _perf_install_attempted
    with _perf_install_lock:
        if _perf_install_attempted:
            return which("perf")
        _perf_install_attempted = True

    if which("apt-get") is None or which("sudo") is None:
        logger.debug("perf is unavailable and apt-get/sudo were not found")
        return None

    sudo_check = subprocess.run(
        ["sudo", "-n", "true"],
        check=False,
        capture_output=True,
        text=True,
    )
    if sudo_check.returncode != 0:
        logger.debug("perf is unavailable and passwordless sudo is not configured")
        return None

    install_command = [
        "sudo",
        "-n",
        "apt-get",
        "update",
    ]
    update_result = subprocess.run(
        install_command,
        check=False,
        capture_output=True,
        text=True,
    )
    if update_result.returncode != 0:
        logger.debug("apt-get update failed while trying to install perf")
        return None

    install_result = subprocess.run(
        [
            "sudo",
            "-n",
            "apt-get",
            "install",
            "-y",
            "linux-tools-common",
            "linux-tools-generic",
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if install_result.returncode != 0:
        logger.debug("apt-get install failed while trying to install perf")
        return None

    # The Ubuntu `perf` wrapper execs a kernel-version-specific binary
    # (`perf_$(uname -r)`), which lives in `linux-tools-$(uname -r)`. The
    # generic meta-package may not match the running kernel (common on
    # bare-metal cloud hosts), so best-effort install the exact package too.
    kernel_release = platform.release()
    kernel_tools_result = subprocess.run(
        [
            "sudo",
            "-n",
            "apt-get",
            "install",
            "-y",
            f"linux-tools-{kernel_release}",
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if kernel_tools_result.returncode != 0:
        logger.debug(
            "kernel-specific perf tools (linux-tools-%s) unavailable; "
            "relying on generic perf tools",
            kernel_release,
        )

    return which("perf")
