#!/usr/bin/env bash
# ------------------------------------------------------------------------------
# setup.sh
#
# Developer setup for the performance dashboard tooling.
# Fetches the upstream orchestrator framework and creates a Python virtual
# environment with all required dependencies.
#
# Usage (from repo root):
#   bash tools/pipeline_perf_test/dashboard/setup.sh
# ------------------------------------------------------------------------------
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$REPO_ROOT"

echo "==> Fetching upstream perf tools..."
bash scripts/perf/fetch_perf_tools.sh

echo "==> Setting up Python environment..."
bash scripts/perf/setup_python_env.sh

UV_VENV_PATH=$(cat .uv_venv_path.txt)
echo ""
echo "============================================="
echo "Setup complete."
echo ""
echo "Activate the environment with:"
echo "  source ${UV_VENV_PATH}/bin/activate"
echo ""
echo "Then run a suite with:"
echo "  cd tools/pipeline_perf_test"
echo "  python dashboard/scripts/run.py dashboard/suites/<suite>.yaml"
echo "============================================="
