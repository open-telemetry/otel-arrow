"""
Main entry point for the test orchestrator CLI.

This module is responsible for launching the orchestrator when run as a script.
It imports the main CLI logic from `lib.cli.main` and delegates execution to it.

Typical usage:
    python -m orchestrator/run_orchestrator.py \
        --config path/to/config.yaml [other options]

- Examples:

    python orchestrator/run_orchestrator.py \
        --config test_suites/collector_batch_comparison/test-suite-docker.yaml \
        --debug

"""

from lib.cli.main import main

if __name__ == "__main__":
    main()
