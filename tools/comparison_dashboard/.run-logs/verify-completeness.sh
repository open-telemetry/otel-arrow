#!/bin/bash
# Walk every suite in .site/data/suite and verify each rate has metrics.json
# with the expected number of metrics (10 for fully-instrumented suites).
# Reports any rate that's missing or short.
#
# Usage: verify-completeness.sh [expected_count]   (default 10)
set -u
EXPECTED="${1:-10}"
PY="${PY:-/home/jakedern/repos/otel-arrow/.venv/bin/python}"
RATES=(100k 200k 300k 400k 600k 800k 1000k)
bad=0
total=0
for slug in $(ls .site/data/suite/ 2>/dev/null); do
    for r in "${RATES[@]}"; do
        total=$((total+1))
        f=".site/data/suite/$slug/$r/metrics.json"
        if [[ ! -f "$f" ]]; then
            echo "MISSING $slug/$r"
            bad=$((bad+1))
            continue
        fi
        n=$("$PY" -c "import json,sys; print(len(json.load(open('$f'))))" 2>/dev/null)
        if [[ "$n" != "$EXPECTED" ]]; then
            echo "SHORT   $slug/$r: $n metrics (expected $EXPECTED)"
            bad=$((bad+1))
        fi
    done
done
echo "---"
echo "Incomplete cells: $bad / $total"
exit $((bad > 0 ? 1 : 0))
