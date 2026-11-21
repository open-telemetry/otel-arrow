"""Build the operator guarantee summary from the actual base-to-candidate diff.

Run from the repository root on the checked-out candidate. The support table
is both the operator document and the input checked against inventory.
"""

import argparse
from collections import defaultdict
from dataclasses import dataclass
import json
from pathlib import Path
import re
import subprocess
import sys
from xml.etree import ElementTree

from released import migrations

DFE = Path("rust/otap-dataflow")
DOCS = DFE / "docs/guarantees"
MAP = DOCS / "review-map.json"
SUPPORT = DOCS / "live-support.md"
INVENTORY = DFE / "components-baseline.json"
FIXTURES = DFE / "crates/config/tests/fixtures/releases"
GUIDES = {DFE / "docs/design-principles.md", DFE / "docs/ai/ai-assisted-pr-review.md"}
REVIEW = Path(".github/guarantee-reviews")
EXCEPTIONS = DOCS / "exceptions"
OPERATIONS = ("add", "change", "remove", "in-place")
STATUS = {"tested", "reviewed with rationale", "approved exception"}
RUST_CASES = {
    "release-precedence": "otel-arrow-dfe-config::release_guarantees/released_core_allocation_precedence",
    "retry-recovery": "otel-arrow-dfe-validation::core_node_liveness_tests/test_retry_pipeline_eventually_recovers_after_transient_nacks",
}


def git(*args):
    return subprocess.check_output(["git", "--literal-pathspecs", *map(str, args)])


def read_base(base, path):
    result = subprocess.run(["git", "show", f"{base}:{path}"], capture_output=True)
    return result.stdout.decode() if result.returncode == 0 else None


def tracked_diff(base):
    # Comparing two trees also handles queue batches, deletions and shallow PR merge refs.
    raw = git("diff", "--no-renames", "--name-only", "-z", base, "HEAD")
    return {Path(p.decode()) for p in raw.split(b"\0") if p}


def affected(path, lines, rules):
    name = str(path)
    if not (path.is_relative_to(DFE) or name.startswith((
            "tools/pipeline_perf_test/", "proto/", ".github/workflows/"))):
        return set()
    if path.name == "README.md" and re.search(r"/(receivers|processors|exporters)/", name):
        hits = {"configuration"}
        for key, pattern in (("live", "reconfigur|in-place"), ("performance", "benchmark|throughput"),
                             ("security", "credential|secret|authentication")):
            if re.search(pattern, lines, re.I):
                hits.add(key)
        return hits
    hits = set()
    for key, rule in rules.items():
        if re.search(rule["path"], name) or (path.is_relative_to(DFE) and rule.get("lines") and
                path.suffix in {".rs", ".toml", ".proto", ".yaml", ".yml"} and
                re.search(rule["lines"], lines)):
            hits.add(key)
    # Every new engine implementation receives an architectural review, including new directories.
    if path.suffix == ".rs" and not hits:
        hits.add("design")
    return hits


def inventory(text):
    return {r["id"] for r in json.loads(text) if r["category"] in {
        "Receiver", "Processor", "Exporter"}}


def inside(root, path):
    return path.resolve().is_relative_to(root.resolve())


def support_table(text):
    rows = {}
    for line in text.splitlines():
        if not line.startswith("| `urn:"):
            continue
        urn, *cells = [v.strip() for v in line.strip("|").split("|")]
        urn = urn.strip("`")
        if urn in rows or len(cells) != len(OPERATIONS):
            raise ValueError(f"duplicate or malformed support row for {urn}")
        rows[urn] = dict(zip(OPERATIONS, cells))
    return rows


def approved_decision(base, path, area):
    doc = Path(path)
    if doc.parent != EXCEPTIONS or doc.suffix != ".md":
        return False
    current = read_base(base, doc)
    if not current or not doc.is_file() or current != doc.read_text():
        return False
    scope = re.search(r"^## Scope\n(.*?)(?=^## |\Z)", current, re.M | re.S)
    return bool(scope and f"- {area}" in scope[1].splitlines())


def validate_support(base):
    text = SUPPORT.read_text()
    rows = support_table(text)
    ids = inventory(INVENTORY.read_text())
    if ids != rows.keys():
        raise ValueError(f"support table and component inventory differ: {sorted(ids ^ rows.keys())}")
    inherited = support_table(read_base(base, SUPPORT) or "")
    existing = inventory(read_base(base, INVENTORY) or "[]")
    refs = dict(re.findall(r"^\[([\w-]+)\]: (\S+)$", text, re.M))
    covered = 0
    for urn, ops in rows.items():
        for operation, cell in ops.items():
            if operation == "in-place" and cell == "-":
                if inherited.get(urn, {}).get(operation, "-") != "-":
                    raise ValueError(f"{urn} lost its in-place promise; record an exception")
                continue
            match = re.fullmatch(r"\[(scenario|manual|untested|exception)\]\[([\w-]+)\]", cell)
            if not match or match[2] not in refs:
                raise ValueError(f"{urn} {operation}: invalid evidence cell {cell}")
            kind, link = match[1], refs[match[2]]
            if kind == "untested":
                if (urn not in existing or link != "#coverage-backlog" or
                        inherited.get(urn, {}).get(operation, cell) != cell):
                    raise ValueError(f"{urn} {operation}: new or weakened promise needs evidence")
                continue
            target = (SUPPORT.parent / link.split("#")[0]).resolve()
            if not inside(Path.cwd(), target) or not target.is_file():
                raise ValueError(f"{urn} {operation}: missing local evidence at {link}")
            if kind == "scenario" and target.suffix.lower() not in {".rs", ".py", ".sh", ".yaml", ".yml"}:
                raise ValueError(f"{urn} {operation}: link an executable scenario")
            if kind == "exception" and not approved_decision(
                    base, target.relative_to(Path.cwd()), f"{urn} {operation}"):
                raise ValueError(f"{urn} {operation}: exception requires a decision already on base")
            if kind == "scenario" and operation != "in-place":
                covered += 1
    return covered, 3 * len(ids)


def check_decisions(paths):
    for path in paths:
        if path.parent != EXCEPTIONS or not path.exists():
            continue
        text = path.read_text()
        for heading in ("Scope", "Justification", "Operator procedure", "Migration and release",
                        "Verification", "Decision"):
            if not re.search(rf"^## {heading}\n\s*\n[^#\s]", text, re.M):
                raise ValueError(f"{path} needs a populated '{heading}' section")
        if not re.search(r"https://github\.com/open-telemetry/otel-arrow/(pull|issues)/[0-9]+", text):
            raise ValueError(f"{path} needs the maintainer decision link")


def validate_migrations(base):
    retired = migrations()
    for name, record in retired.items():
        replacement = record["replacement"]
        for key in (name, replacement):
            path = FIXTURES / key
            if (Path(key).is_absolute() or len(Path(key).parts) != 2 or
                    not inside(FIXTURES, path) or not path.is_file()):
                raise ValueError(f"invalid migration fixture {key}")
        if replacement in retired or not record["diagnostic"].strip():
            raise ValueError(f"{name}: supply an accepted replacement and a rejection diagnostic")
        if not approved_decision(base, record["decision"], f"configuration {name}"):
            raise ValueError(f"{name}: migration needs a preceding fixture-specific decision")


def test_results(folder):
    # A failed, skipped or flaky/retried case cannot supply a passing observation.
    seen = defaultdict(list)
    if folder:
        for report in folder.rglob("junit.xml"):
            for case in ElementTree.parse(report).iter("testcase"):
                case_id = case.get("classname", "") + "/" + case.get("name", "")
                seen[case_id].append(all(case.find(tag) is None for tag in (
                    "failure", "error", "skipped", "rerunFailure", "flakyFailure", "rerunError",
                    "flakyError")))
    result = {"junit:" + key: all(outcomes) for key, outcomes in seen.items()}
    for key, case_id in RUST_CASES.items():
        result[key] = result.get("junit:" + case_id, False)
    return result


@dataclass
class Evidence:
    area: str
    paths: set
    status: str
    reason: str
    record: Path
    source: str


def validate_benchmark(urn, coverage):
    if coverage.get("workload"):
        suite = Path(coverage["workload"])
        root = Path("tools/pipeline_perf_test")
        continuous = root / "test_suites/integration/continuous"
        workflow = Path(".github/workflows/pipeline-perf-test-continuous.yml").read_text()
        if (not inside(continuous, suite) or not suite.is_file() or
                str(suite.relative_to(root)) not in workflow):
            raise ValueError(f"{urn}: link a suite invoked by continuous CI")
    elif not coverage.get("obstacle", "").strip():
        raise ValueError(f"{urn}: describe the continuous workload or its obstacle")
    source = coverage["measurement"]
    if not (source.startswith("https://") or
            (inside(Path.cwd(), Path(source)) and Path(source).is_file())):
        raise ValueError(f"{urn}: link the workload measurement or reproducible alternative")


def evidence_record(item, path, paths, checks, base):
    state, reason, source = item["status"], item["reason"].strip(), item["source"]
    scope = {Path(name) for name in item["paths"]}
    if state not in STATUS or not reason or not scope or not scope <= paths:
        raise ValueError("invalid status, rationale or changed paths")
    if state == "tested":
        if not checks.get(source):
            raise ValueError(f"missing passing result for {source}")
        expected = {"released-inputs": "configuration", "release-precedence": "configuration",
                    "retry-recovery": "behavior", "live-rollout": "live"}
        if source in expected and item["area"] != expected[source]:
            raise ValueError(f"{source} covers {expected[source]}, not {item['area']}")
    elif state == "approved exception":
        if not approved_decision(base, source, item["area"]):
            raise ValueError("exception has no matching decision on the base")
    else:
        local = Path(source.split("#")[0])
        if not ((not local.is_absolute() and inside(Path.cwd(), local) and local.is_file()) or
                re.fullmatch(r"https://[^\s<>|]+", source)):
            raise ValueError(f"missing review evidence at {source}")
    return Evidence(item["area"], scope, state, reason, path, source)


def evidence_records(paths, checks, base, problems):
    records = []
    benchmarks = set()
    invalid = (OSError, ValueError, KeyError, TypeError, AttributeError)
    for path in sorted(paths):
        if path.parent != REVIEW or not path.is_file() or path.suffix != ".json":
            continue
        try:
            if read_base(base, path) is not None:
                raise ValueError("add a new assessment instead of editing a past one")
            data = json.loads(path.read_text())
            for urn, coverage in data.get("benchmarks", {}).items():
                validate_benchmark(urn, coverage)
                benchmarks.add(urn)
            for item in data["evidence"]:
                try:
                    records.append(evidence_record(item, path, paths, checks, base))
                except invalid as error:
                    problems.append(f"{path}: {error}")
        except invalid as error:
            problems.append(f"{path}: {error}")
    return records, benchmarks


def md(value):
    return str(value).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace(
        "|", "&#124;").replace("\n", " ")


def report(args):
    rules = json.loads(MAP.read_text())
    paths = tracked_diff(args.base)
    checks = test_results(args.junit)
    checks.update({"released-inputs": args.released == "success", "live-rollout": args.live == "success"})
    print("## Operator guarantees\n")
    coverage = None
    print(f"[Candidate test run]({args.run_url})\n")
    problems = []
    for key in (*RUST_CASES, "released-inputs", "live-rollout"):
        passed = checks[key]
        print(f"- **{'tested' if passed else 'evidence missing'}** {key}")
        if not passed:
            problems.append(f"Missing passing observation for {key}.")
    records, reviewed_ids = evidence_records(paths, checks, args.base, problems)
    try:
        new_ids = inventory(INVENTORY.read_text()) - inventory(read_base(args.base, INVENTORY) or "[]")
        if new_ids - reviewed_ids:
            problems.append("New components need benchmark coverage: " + ", ".join(sorted(new_ids - reviewed_ids)))
        linked, total = validate_support(args.base)
        coverage = {"scenario_linked_operations": linked, "registered_operations": total}
        print(f"\n{linked}/{total} component operations linked to executable scenarios.")
        check_decisions(paths)
        validate_migrations(args.base)
    except (ValueError, OSError, TypeError, KeyError) as error:
        problems.append(str(error))
    for path in sorted(paths):
        if path.is_relative_to(FIXTURES) and read_base(args.base, path) is not None:
            problems.append(f"Keep released input {path} intact and add a separate version or migration case.")
    if paths & GUIDES and MAP not in paths:
        problems.append("Update the review mapping when the review guide or design principles change.")
    impacts = defaultdict(set)
    for path in sorted(paths):
        if path.parent in {REVIEW, EXCEPTIONS}:
            continue
        lines = git("diff", "--no-ext-diff", "--no-renames", "-U0", args.base,
                    "HEAD", "--", path).decode(errors="replace")
        lines = "\n".join(s[1:] for s in lines.splitlines()
                          if s.startswith(("+", "-")) and not s.startswith(("+++", "---")))
        for area in affected(path, lines, rules):
            impacts[area].add(path)
    print("\n| Area | Outcome | Evidence |\n| --- | --- | --- |")
    for record in records:
        if record.area not in rules:
            problems.append(f"Unknown guarantee {record.area} in {record.record}.")
            continue
        impacts[record.area].update(record.paths)
    for area, scope in sorted(impacts.items()):
        for record in (r for r in records if r.area == area):
            scope.difference_update(record.paths)
            if record.status == "tested":
                link = args.run_url
            elif record.source.startswith("https://"):
                link = record.source
            else:
                link = f"{args.tree_url}/{record.source}"
            print(f"| {md(area)} | {md(record.status)} | {md(record.reason)} "
                  f"([evidence](<{link}>), [assessment](<{args.tree_url}/{record.record}>)) |")
        if scope:
            names = ", ".join(md(p) for p in sorted(scope)[:5])
            if len(scope) > 5:
                names += f" (and {len(scope) - 5} more)"
            print(f"| {area} | evidence missing | {names} |")
            problems.append(f"{area}: {rules[area]['evidence']}")
    print("\nUnaffected by the diff rules: " + (", ".join(sorted(rules.keys() - impacts.keys())) or "none"))
    if problems:
        print("\n### Required before merge\n")
        for problem in problems:
            print("- " + md(problem))
    if args.metrics:
        snapshot = {"commit": git("rev-parse", "HEAD").decode().strip(),
                    "seed_results": {key: checks[key] for key in (*RUST_CASES, "released-inputs", "live-rollout")},
                    "affected_areas": sorted(impacts), "coverage": coverage,
                    "blocking_findings": problems, "run": args.run_url}
        args.metrics.write_text(json.dumps(snapshot, indent=2) + "\n")
    return int(bool(problems))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True)
    parser.add_argument("--junit", type=Path)
    parser.add_argument("--metrics", type=Path)
    parser.add_argument("--released", default="missing")
    parser.add_argument("--live", default="missing")
    parser.add_argument("--run-url", default="https://github.com/open-telemetry/otel-arrow/actions")
    parser.add_argument("--tree-url", default="https://github.com/open-telemetry/otel-arrow/blob/main")
    sys.exit(report(parser.parse_args()))
