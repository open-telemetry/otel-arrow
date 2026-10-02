#!/bin/bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PLAN_SCRIPT="${SCRIPT_DIR}/plan-release.sh"
TEST_ROOT=$(mktemp -d)
trap 'rm -rf "$TEST_ROOT"' EXIT

assert_json() {
    local json="$1"
    local expression="$2"
    local expected="$3"
    local actual
    actual=$(jq -r "$expression" <<< "$json")
    if [ "$actual" != "$expected" ]; then
        echo "Assertion failed: ${expression}" >&2
        echo "Expected: ${expected}" >&2
        echo "Actual:   ${actual}" >&2
        exit 1
    fi
}

create_entry() {
    local file="$1"
    local change_type="$2"
    local component="$3"
    cat > "$file" <<EOF
change_type: ${change_type}
component: ${component}
note: Test entry.
issues: [1]
subtext:
EOF
}

setup_repository() {
    mkdir -p "$TEST_ROOT/go/.chloggen" "$TEST_ROOT/rust/otap-dataflow/.chloggen"
    git -C "$TEST_ROOT" init --quiet
    git -C "$TEST_ROOT" config user.name test
    git -C "$TEST_ROOT" config user.email test@example.com
    touch "$TEST_ROOT/go/.chloggen/TEMPLATE.yaml"
    touch "$TEST_ROOT/rust/otap-dataflow/.chloggen/TEMPLATE.yaml"
    git -C "$TEST_ROOT" add .
    git -C "$TEST_ROOT" commit --quiet -m initial
    git -C "$TEST_ROOT" tag go/v1.2.3
    git -C "$TEST_ROOT" tag rust/otap-dataflow/v0.58.0
}

# Scenario: A component has no pending changelog entries.
# Guarantees: The planner reports no release and does not calculate a version.
test_no_pending_entries() {
    local plan
    plan=$(cd "$TEST_ROOT" && "$PLAN_SCRIPT" go)
    assert_json "$plan" '.has_changes' 'false'
    assert_json "$plan" '.next_version' 'null'
}

# Scenario: Pending changes contain only bug fixes and dependency enhancements.
# Guarantees: The planner selects the next patch version.
test_patch_release() {
    local plan
    create_entry "$TEST_ROOT/go/.chloggen/fix.yaml" bug_fix all
    create_entry "$TEST_ROOT/go/.chloggen/dependencies.yaml" enhancement dependencies
    plan=$(cd "$TEST_ROOT" && "$PLAN_SCRIPT" go)
    assert_json "$plan" '.impact' 'patch'
    assert_json "$plan" '.entry_count' '2'
    assert_json "$plan" '.next_version' '1.2.4'
}

# Scenario: At least one pending entry adds a non-dependency enhancement.
# Guarantees: The planner promotes the release to the next minor version.
test_minor_release() {
    local plan
    create_entry "$TEST_ROOT/rust/otap-dataflow/.chloggen/feature.yaml" enhancement engine
    plan=$(cd "$TEST_ROOT" && "$PLAN_SCRIPT" rust)
    assert_json "$plan" '.impact' 'minor'
    assert_json "$plan" '.next_version' '0.59.0'
}

setup_repository
test_no_pending_entries
test_patch_release
test_minor_release

echo "plan-release tests passed"
