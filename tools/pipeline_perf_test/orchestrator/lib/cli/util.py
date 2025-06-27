"""
Module: util.py

Misc utility functions used by the test framework cli.
"""

import subprocess


def get_git_info():
    """
    Retrieves basic Git repository metadata for the current working directory.

    Returns:
        dict: A dictionary containing:
            - 'code.sha' (str): The full Git commit SHA of the current HEAD.
            - 'code.dirty' (bool): True if there are uncommitted changes in the working directory.
            - 'code.branch' (str): The current branch name.

    If the Git command fails (e.g., not a Git repository or Git is unavailable),
    the function returns default values:
        - 'code.sha': "unknown"
        - 'code.dirty': False
        - 'code.branch': "unknown"

    Notes:
        - Suppresses all stderr output from Git commands.
        - Strips all outputs of trailing newlines or whitespace.
        - Safe to use in environments where Git may not be available.
    """

    def run_git_cmd(cmd):
        return subprocess.check_output(cmd, stderr=subprocess.DEVNULL).decode().strip()

    try:
        sha = run_git_cmd(["git", "rev-parse", "HEAD"])
        dirty = bool(run_git_cmd(["git", "status", "--porcelain"]))
        branch = run_git_cmd(["git", "rev-parse", "--abbrev-ref", "HEAD"])
        return {
            "code.sha": sha,
            "code.dirty": dirty,
            "code.branch": branch,
        }
    except subprocess.CalledProcessError:
        return {"code.sha": "unknown", "code.dirty": False, "code.branch": "unknown"}
