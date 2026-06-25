"""Helpers for best-effort Linux perf support."""

import platform
import subprocess
import threading
from logging import LoggerAdapter
from shutil import which

_perf_install_attempted = False
_perf_install_lock = threading.Lock()


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

    return which("perf")