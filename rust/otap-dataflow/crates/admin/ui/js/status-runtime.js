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

function splitCamelCase(value) {
  return String(value || "")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .trim();
}

function normalizeEventType(rawType) {
  if (!rawType || typeof rawType !== "object") {
    return {
      kind: "unknown",
      name: "",
      label: "Unknown",
    };
  }

  if ("Request" in rawType) {
    const name = String(rawType.Request || "");
    return {
      kind: "request",
      name,
      label: splitCamelCase(name) || "Request",
    };
  }

  if ("Success" in rawType) {
    const name = String(rawType.Success || "");
    return {
      kind: "success",
      name,
      label: splitCamelCase(name) || "Success",
    };
  }

  if ("Error" in rawType) {
    const rawError = rawType.Error;
    let name = "";
    if (typeof rawError === "string") {
      name = rawError;
    } else if (rawError && typeof rawError === "object") {
      const [firstKey] = Object.keys(rawError);
      name = firstKey || "Error";
    }
    return {
      kind: "error",
      name,
      label: splitCamelCase(name) || "Error",
    };
  }

  return {
    kind: "unknown",
    name: "",
    label: "Unknown",
  };
}

function normalizeEngineEvent(rawEvent, coreId) {
  if (!rawEvent || typeof rawEvent !== "object") return null;
  const engineEvent =
    rawEvent.Engine && typeof rawEvent.Engine === "object"
      ? rawEvent.Engine
      : rawEvent;
  if (!engineEvent || typeof engineEvent !== "object" || !engineEvent.type) {
    return null;
  }

  const type = normalizeEventType(engineEvent.type);
  const time =
    engineEvent.time && Number.isFinite(new Date(engineEvent.time).getTime())
      ? new Date(engineEvent.time)
      : null;

  return {
    coreId: coreId == null ? null : String(coreId),
    time: engineEvent.time || null,
    timeMs: time ? time.getTime() : Number.NEGATIVE_INFINITY,
    typeKind: type.kind,
    typeName: type.name,
    typeLabel: type.label,
    message:
      engineEvent.message == null || engineEvent.message === ""
        ? null
        : String(engineEvent.message),
  };
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
    const recentEvents = [];
    const rawCores =
      rawPipelineStatus?.cores && typeof rawPipelineStatus.cores === "object"
        ? rawPipelineStatus.cores
        : {};
    Object.entries(rawCores).forEach(([coreId, coreStatus]) => {
      const rawRecentEvents = Array.isArray(coreStatus?.recentEvents)
        ? coreStatus.recentEvents
        : [];
      rawRecentEvents.forEach((rawEvent) => {
        const normalized = normalizeEngineEvent(rawEvent, coreId);
        if (normalized) {
          recentEvents.push(normalized);
        }
      });
    });
    recentEvents.sort((a, b) => b.timeMs - a.timeMs);
    const latestEvent = recentEvents[0] || null;

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
      latestEventSummary: latestEvent?.typeLabel || "-",
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
          recentEvents,
          latestEvent,
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
