"""Validate pinned operator configuration using the candidate engine binary."""

import json
from pathlib import Path
import subprocess
import tempfile

from engine import from_args


FIXTURES = Path(__file__).resolve().parents[2] / "crates/config/tests/fixtures/releases"


# Scenario: validate release YAML, its JSON form, and a graph with an unknown node.
# Guarantees: release inputs load and an invalid graph fails for the missing reference.
def run(engine):
    for fixture in sorted(FIXTURES.glob("*/*")):
        subprocess.run(engine.command(fixture) + ["--validate-and-exit"],
                       check=True, timeout=15, stdout=subprocess.DEVNULL)
    with tempfile.TemporaryDirectory() as tmp:
        source = json.loads((FIXTURES / "v0.60.0/syslog-console.json").read_text())
        source["groups"]["default"]["pipelines"]["main"]["connections"][0]["to"] = "missing-node"
        invalid = Path(tmp) / "missing-node.json"
        invalid.write_text(json.dumps(source))
        result = subprocess.run(engine.command(invalid) + ["--validate-and-exit"],
                                capture_output=True, text=True, timeout=15)
        if result.returncode <= 0 or "missing-node" not in result.stderr:
            raise AssertionError(f"expected missing-node rejection: {result.returncode} {result.stderr}")


if __name__ == "__main__":
    run(from_args(__doc__))
