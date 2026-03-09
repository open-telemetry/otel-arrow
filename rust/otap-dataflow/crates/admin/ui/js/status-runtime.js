// Runtime status normalization and health-condition summarization.
// Adapts /status payload shapes into stable pipeline-keyed data for selector
// decorations and the runtime overlay table.
import { makePipelineSelectionKey } from "./pipeline-utils.js";

const ACCEPTED_BENIGN_FALSE_REASONS = new Set([
  "Pending",
  "StartRequested",
  "Deleting",
  "ForceDeleting",
  "Deleted",
  "NoPipelineRuntime",
]);

function normalizeStatusCondition(rawCondition) {
  if (!rawCondition || typeof rawCondition !== "object") return null;
  const rawKind = rawCondition.kind ?? rawCondition.type;
  return {
    kind: rawKind == null ? "" : String(rawKind),
    status: rawCondition.status == null ? "" : String(rawCondition.status),
    reason: rawCondition.reason == null ? "" : String(rawCondition.reason),
    message: rawCondition.message == null ? "" : String(rawCondition.message),
  };
}

function findStatusCondition(conditions, kind) {
  if (!Array.isArray(conditions)) return null;
  for (const rawCondition of conditions) {
    const condition = normalizeStatusCondition(rawCondition);
    if (!condition) continue;
    if (condition.kind === kind) {
      return condition;
    }
  }
  return null;
}

function isAcceptedConditionFailure(condition) {
  if (!condition) return false;
  if (condition.status === "True") return false;
  if (condition.status === "Unknown") {
    return condition.reason !== "NoPipelineRuntime";
  }
  if (condition.status === "False") {
    return !ACCEPTED_BENIGN_FALSE_REASONS.has(condition.reason || "");
  }
  return false;
}

function classifyPipelineStatus(rawPipelineStatus) {
  const accepted = findStatusCondition(rawPipelineStatus?.conditions, "Accepted");
  const ready = findStatusCondition(rawPipelineStatus?.conditions, "Ready");
  const totalCoresRaw = Number(
    rawPipelineStatus?.totalCores ?? rawPipelineStatus?.total_cores ?? 0
  );
  const runningCoresRaw = Number(
    rawPipelineStatus?.runningCores ?? rawPipelineStatus?.running_cores ?? 0
  );
  const totalCores =
    Number.isFinite(totalCoresRaw) && totalCoresRaw > 0 ? Math.floor(totalCoresRaw) : 0;
  const runningCores =
    Number.isFinite(runningCoresRaw) && runningCoresRaw >= 0
      ? Math.floor(runningCoresRaw)
      : 0;

  let state = "unknown";
  let summary = "Unknown";
  if (isAcceptedConditionFailure(accepted)) {
    state = "down";
    summary = "Rejected";
  } else if (ready?.status === "False") {
    if (ready.reason === "NoActiveCores" || ready.reason === "NoPipelineRuntime") {
      state = "unknown";
      summary = "Pending";
    } else {
      state = "down";
      summary = "Not ready";
    }
  } else if (accepted?.status === "True" && ready?.status === "True") {
    state = "up";
    summary = "Ready";
  } else if (accepted?.status === "False") {
    state = "unknown";
    summary = "Pending";
  } else if (accepted?.status === "Unknown" || ready?.status === "Unknown") {
    state = "unknown";
    summary = "Pending";
  }

  const details = [];
  if (totalCores > 0) {
    details.push(`${runningCores}/${totalCores} cores running`);
  } else {
    details.push("no runtime cores");
  }
  if (accepted) {
    details.push(`Accepted=${accepted.status}${accepted.reason ? ` (${accepted.reason})` : ""}`);
  }
  if (ready) {
    details.push(`Ready=${ready.status}${ready.reason ? ` (${ready.reason})` : ""}`);
  }
  if (ready?.message) {
    details.push(ready.message);
  } else if (accepted?.message) {
    details.push(accepted.message);
  }

  return {
    state,
    summary,
    totalCores,
    runningCores,
    details,
    accepted,
    ready,
  };
}

export function getStatusSeverity(state) {
  if (state === "down") return 2;
  if (state === "unknown") return 1;
  return 0;
}

// Some engine payloads expose "group:pipeline", some expose keys with additional colons.
// Keep both first- and last-split variants so selector mapping remains robust.
function parseStatusPipelineSelectionKeys(rawPipelineKey) {
  const keys = new Set();
  if (typeof rawPipelineKey !== "string" || rawPipelineKey.length === 0) {
    return keys;
  }

  const firstSep = rawPipelineKey.indexOf(":");
  if (firstSep < 0) {
    return keys;
  }
  keys.add(
    makePipelineSelectionKey(
      rawPipelineKey.slice(0, firstSep),
      rawPipelineKey.slice(firstSep + 1)
    )
  );

  const lastSep = rawPipelineKey.lastIndexOf(":");
  if (lastSep > firstSep) {
    keys.add(
      makePipelineSelectionKey(
        rawPipelineKey.slice(0, lastSep),
        rawPipelineKey.slice(lastSep + 1)
      )
    );
  }
  return keys;
}

// Normalize /status payload into a stable snapshot consumed by selector and runtime overlay UIs.
export function buildStatusSnapshot(statusPayload, checkedAt = Date.now()) {
  const pipelineStatuses =
    statusPayload?.pipelines && typeof statusPayload.pipelines === "object"
      ? statusPayload.pipelines
      : {};
  const byPipelineKey = new Map();
  const rows = [];
  let total = 0;
  let up = 0;
  let down = 0;
  let unknown = 0;

  for (const [rawPipelineKey, rawPipelineStatus] of Object.entries(pipelineStatuses)) {
    const classified = classifyPipelineStatus(rawPipelineStatus);
    total += 1;
    if (classified.state === "up") {
      up += 1;
    } else if (classified.state === "down") {
      down += 1;
    } else {
      unknown += 1;
    }

    rows.push({
      rawPipelineKey,
      acceptedStatus: classified.accepted?.status || "Unknown",
      readyStatus: classified.ready?.status || "Unknown",
      runningCores: classified.runningCores,
      totalCores: classified.totalCores,
      topReason:
        (classified.accepted &&
          classified.accepted.status !== "True" &&
          (classified.accepted.reason || classified.accepted.message)) ||
        (classified.ready &&
          classified.ready.status !== "True" &&
          (classified.ready.reason || classified.ready.message)) ||
        "-",
      summary: classified.summary,
      state: classified.state,
    });

    const selectionKeys = parseStatusPipelineSelectionKeys(rawPipelineKey);
    selectionKeys.forEach((selectionKey) => {
      const previous = byPipelineKey.get(selectionKey);
      if (
        !previous ||
        getStatusSeverity(classified.state) > getStatusSeverity(previous.state)
      ) {
        byPipelineKey.set(selectionKey, {
          rawPipelineKey,
          state: classified.state,
          summary: classified.summary,
          totalCores: classified.totalCores,
          runningCores: classified.runningCores,
          details: classified.details,
        });
      }
    });
  }

  return {
    generatedAt: statusPayload?.generatedAt || statusPayload?.generated_at || null,
    checkedAt,
    total,
    up,
    down,
    unknown,
    rows,
    byPipelineKey,
  };
}
