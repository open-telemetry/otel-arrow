#!/bin/bash

set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <go|rust>" >&2
    exit 2
fi

TARGET="$1"

case "$TARGET" in
    go)
        TAG_PATTERN='go/v[0-9]*.[0-9]*.[0-9]*'
        TAG_PREFIX='go/v'
        ENTRIES_DIR='go/.chloggen'
        ;;
    rust)
        TAG_PATTERN='rust/otap-dataflow/v[0-9]*.[0-9]*.[0-9]*'
        TAG_PREFIX='rust/otap-dataflow/v'
        ENTRIES_DIR='rust/otap-dataflow/.chloggen'
        ;;
    *)
        echo "Error: target must be 'go' or 'rust'." >&2
        exit 2
        ;;
esac

LAST_TAG=$(git tag --list "$TAG_PATTERN" --sort=-version:refname | head -n1 || true)
if [ -z "$LAST_TAG" ]; then
    LAST_VERSION='0.0.0'
else
    LAST_VERSION=${LAST_TAG#"$TAG_PREFIX"}
fi

if [[ ! "$LAST_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: latest ${TARGET} tag has invalid version '${LAST_VERSION}'." >&2
    exit 1
fi

ENTRY_COUNT=0
IMPACT='patch'

shopt -s nullglob
for entry in "$ENTRIES_DIR"/*.yaml "$ENTRIES_DIR"/*.yml; do
    case "$(basename "$entry")" in
        TEMPLATE.yaml | TEMPLATE.yml | config.yaml | config.yml)
            continue
            ;;
    esac

    CHANGE_TYPE=$(sed -n 's/^change_type:[[:space:]]*\([^[:space:]#]*\).*$/\1/p' "$entry" | head -n1)
    COMPONENT=$(sed -n 's/^component:[[:space:]]*\([^[:space:]#]*\).*$/\1/p' "$entry" | head -n1)

    if [ -z "$CHANGE_TYPE" ] || [ -z "$COMPONENT" ]; then
        echo "Error: ${entry} must define change_type and component." >&2
        exit 1
    fi

    ENTRY_COUNT=$((ENTRY_COUNT + 1))
    case "$CHANGE_TYPE" in
        bug_fix)
            ;;
        enhancement)
            if [ "$COMPONENT" != 'dependencies' ]; then
                IMPACT='minor'
            fi
            ;;
        breaking | deprecation | new_component)
            IMPACT='minor'
            ;;
        *)
            echo "Error: ${entry} has unsupported change_type '${CHANGE_TYPE}'." >&2
            exit 1
            ;;
    esac
done
shopt -u nullglob

if [ "$ENTRY_COUNT" -eq 0 ]; then
    jq -n \
        --arg target "$TARGET" \
        --arg last_tag "$LAST_TAG" \
        --arg last_version "$LAST_VERSION" \
        '{
            target: $target,
            has_changes: false,
            impact: null,
            entry_count: 0,
            last_tag: $last_tag,
            last_version: $last_version,
            next_version: null
        }'
    exit 0
fi

IFS='.' read -r MAJOR MINOR PATCH <<< "$LAST_VERSION"
if [ "$IMPACT" = 'minor' ]; then
    MINOR=$((MINOR + 1))
    PATCH=0
else
    PATCH=$((PATCH + 1))
fi
NEXT_VERSION="${MAJOR}.${MINOR}.${PATCH}"

jq -n \
    --arg target "$TARGET" \
    --arg impact "$IMPACT" \
    --arg last_tag "$LAST_TAG" \
    --arg last_version "$LAST_VERSION" \
    --arg next_version "$NEXT_VERSION" \
    --argjson entry_count "$ENTRY_COUNT" \
    '{
        target: $target,
        has_changes: true,
        impact: $impact,
        entry_count: $entry_count,
        last_tag: $last_tag,
        last_version: $last_version,
        next_version: $next_version
    }'
