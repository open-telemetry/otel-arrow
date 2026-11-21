"""Validate pinned operator configuration using the candidate engine binary."""

import json
from pathlib import Path
import subprocess
import tempfile

from engine import from_args


FIXTURES = Path(__file__).resolve().parents[2] / "crates/config/tests/fixtures/releases"


def migrations():
    manifest = Path(__file__).resolve().parents[2] / "docs/guarantees/release-migrations.json"
    return json.loads(manifest.read_text()) if manifest.exists() else {}


def validate(engine, fixture, diagnostic=None):
    result = subprocess.run(engine.command(fixture) + ["--validate-and-exit"],
                            capture_output=True, text=True, timeout=15)
    if diagnostic is None:
        if result.returncode != 0:
            raise RuntimeError(f"{fixture.name} failed validation: {result.stderr}")
    elif result.returncode <= 0 or diagnostic not in result.stderr:
        raise AssertionError(f"{fixture.name}: expected rejection containing {diagnostic!r}, "
                             f"got {result.returncode}: {result.stderr}")


# Scenario: validate release YAML, its JSON form, and a graph with an unknown node.
# Guarantees: release inputs load and an invalid graph fails for the missing reference.
def run(engine):
    retired = migrations()
    for fixture in sorted(FIXTURES.glob("*/*")):
        migration = retired.get(fixture.relative_to(FIXTURES).as_posix())
        validate(engine, fixture, migration["diagnostic"] if migration else None)
    with tempfile.TemporaryDirectory() as tmp:
        source = json.loads((FIXTURES / "v0.60.0/syslog-console.json").read_text())
        source["groups"]["default"]["pipelines"]["main"]["connections"][0]["to"] = "missing-node"
        invalid = Path(tmp) / "missing-node.json"
        invalid.write_text(json.dumps(source))
        validate(engine, invalid, "missing-node")


if __name__ == "__main__":
    run(from_args(__doc__))
