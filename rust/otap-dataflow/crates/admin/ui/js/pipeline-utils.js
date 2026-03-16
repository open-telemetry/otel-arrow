// Shared pipeline/core attribute and selector key utilities.
// Centralizes normalization and key encoding so all UI modules use the same
// pipeline identity semantics.
const PIPELINE_KEY_SEPARATOR = "|";

// Convert incoming attribute payloads (plain or OTLP-like nested values) to
// flat strings.
export function normalizeAttributes(attrs) {
  const out = {};
  if (!attrs) return out;
  for (const [key, value] of Object.entries(attrs)) {
    if (value == null) continue;
    if (typeof value === "object") {
      const entry = Object.entries(value)[0];
      if (entry) out[key] = String(entry[1]);
    } else {
      out[key] = String(value);
    }
  }
  return out;
}

// Accept known attribute aliases to handle backend naming variations.
export function getPipelineGroupId(attrs) {
  return (
    attrs["pipeline.group.id"] ||
    attrs["pipeline_group_id"] ||
    attrs["otelcol.pipeline_group.id"] ||
    ""
  );
}

export function getPipelineId(attrs) {
  return (
    attrs["pipeline.id"] ||
    attrs["pipeline_id"] ||
    attrs["otelcol.pipeline.id"] ||
    ""
  );
}

// Use an encoded stable key so selectors can safely round-trip group/pipeline ids.
export function makePipelineSelectionKey(groupId, pipelineId) {
  return `${encodeURIComponent(groupId || "")}${PIPELINE_KEY_SEPARATOR}${encodeURIComponent(
    pipelineId || ""
  )}`;
}

export function getPipelineSelectionKeyFromAttrs(attrs) {
  const pipelineId = getPipelineId(attrs);
  if (!pipelineId) return null;
  const groupId = getPipelineGroupId(attrs);
  return makePipelineSelectionKey(groupId, pipelineId);
}

// Human-readable hierarchy label for optgroups (group/subgroup -> group › subgroup).
export function formatPipelineGroupLabel(groupId) {
  if (!groupId) return "(ungrouped)";
  const parts = String(groupId)
    .split("/")
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
  return parts.length ? parts.join(" › ") : String(groupId);
}

// Default to "all cores" when there is no valid explicit selection.
export function resolveSelectedCoreId(selectedCoreId, coreIds, allCoresId = "__all__") {
  const coreChoices = [allCoresId, ...coreIds];
  if (!selectedCoreId || !coreChoices.includes(selectedCoreId)) {
    return allCoresId;
  }
  return selectedCoreId;
}
