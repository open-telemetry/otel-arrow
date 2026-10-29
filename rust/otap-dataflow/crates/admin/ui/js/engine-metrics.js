// Engine-level metrics summarization for top cards.
// Converts raw metric sets into stable aggregate values used by the UI.
import {
  getPipelineGroupId,
  getPipelineId,
  normalizeAttributes,
} from "./pipeline-utils.js";

// Aggregate engine-level signals and infer group/pipeline cardinality from attributes.
export function extractEngineSummary(metricSets, options = {}) {
  const skipAllZeroSnapshots = options.skipAllZeroSnapshots ?? true;
  const engineSets = metricSets.filter((ms) => ms.name === "engine");
  const pipelineSets = metricSets.filter((ms) => ms.name === "pipeline");
  const groupIds = new Set();
  const pipelineIds = new Set();
  const summary = {
    count: 0,
    cpuUtilSum: 0,
    cpuUtilCount: 0,
    memoryRssBytesSum: 0,
    memoryRssCount: 0,
    memoryCommitBytesSum: 0,
    memoryCommitCount: 0,
    groupCount: 0,
    pipelineCount: 0,
    coreCount: pipelineSets.length,
    uptimeSeconds: null,
  };

  metricSets.forEach((set) => {
    const attrs = normalizeAttributes(set.attributes || {});
    const groupId = getPipelineGroupId(attrs) || null;
    const pipelineId = getPipelineId(attrs) || null;
    if (groupId) groupIds.add(groupId);
    if (pipelineId) pipelineIds.add(pipelineId);
  });
  summary.groupCount = groupIds.size;
  summary.pipelineCount = pipelineIds.size;

  pipelineSets.forEach((set) => {
    (set.metrics || []).forEach((metric) => {
      if (
        metric.name !== "uptime" ||
        typeof metric.value !== "number" ||
        !Number.isFinite(metric.value)
      ) {
        return;
      }
      summary.uptimeSeconds =
        summary.uptimeSeconds == null
          ? metric.value
          : Math.max(summary.uptimeSeconds, metric.value);
    });
  });

  engineSets.forEach((set) => {
    let cpuUtil = null;
    let memoryRss = null;
    let memoryCommit = null;
    (set.metrics || []).forEach((metric) => {
      if (typeof metric.value !== "number" || !Number.isFinite(metric.value)) {
        return;
      }
      if (metric.name === "cpu.utilization") {
        cpuUtil = metric.value;
        return;
      }
      if (metric.name === "memory.rss") {
        memoryRss = metric.value;
      }
      if (metric.name === "memory.commit") {
        memoryCommit = metric.value;
      }
    });

    if (typeof memoryCommit === "number" && Number.isFinite(memoryCommit)) {
      summary.memoryCommitBytesSum += memoryCommit;
      summary.memoryCommitCount += 1;
    }

    const isAllZeroSnapshot =
      skipAllZeroSnapshots && cpuUtil === 0 && memoryRss === 0;
    if (isAllZeroSnapshot) {
      return;
    }

    summary.count += 1;
    if (typeof cpuUtil === "number" && Number.isFinite(cpuUtil)) {
      summary.cpuUtilSum += cpuUtil;
      summary.cpuUtilCount += 1;
    }
    if (typeof memoryRss === "number" && Number.isFinite(memoryRss)) {
      summary.memoryRssBytesSum += memoryRss;
      summary.memoryRssCount += 1;
    }
  });

  return summary;
}

// Convert summary aggregates into display-ready card values.
// Keeps previous CPU/memory values when configured and current sample has no usable data.
export function deriveEngineCardValues(summary, previous = {}, options = {}) {
  const holdLastValues = options.holdLastValues ?? true;
  let lastCpuUtilPercent = previous.lastCpuUtilPercent ?? null;
  let lastMemoryRssMiB = previous.lastMemoryRssMiB ?? null;
  let lastMemoryCommitMiB = previous.lastMemoryCommitMiB ?? null;
  let lastUptimeSeconds = previous.lastUptimeSeconds ?? null;

  if (!summary) {
    return {
      groupCount: 0,
      pipelineCount: 0,
      coreCount: 0,
      cpuUtilPercent: Number.isFinite(lastCpuUtilPercent) ? lastCpuUtilPercent : null,
      memoryRssMiB: Number.isFinite(lastMemoryRssMiB) ? lastMemoryRssMiB : null,
      memoryCommitMiB: Number.isFinite(lastMemoryCommitMiB) ? lastMemoryCommitMiB : null,
      uptimeSeconds: Number.isFinite(lastUptimeSeconds) ? lastUptimeSeconds : null,
      currentCpuUtilPercent: null,
      currentMemoryRssMiB: null,
      currentMemoryCommitMiB: null,
      currentUptimeSeconds: null,
      lastCpuUtilPercent: Number.isFinite(lastCpuUtilPercent) ? lastCpuUtilPercent : null,
      lastMemoryRssMiB: Number.isFinite(lastMemoryRssMiB) ? lastMemoryRssMiB : null,
      lastMemoryCommitMiB: Number.isFinite(lastMemoryCommitMiB) ? lastMemoryCommitMiB : null,
      lastUptimeSeconds: Number.isFinite(lastUptimeSeconds) ? lastUptimeSeconds : null,
    };
  }

  const avgCpuUtil =
    summary.cpuUtilCount > 0 ? summary.cpuUtilSum / summary.cpuUtilCount : null;
  const currentCpuUtilPercent = Number.isFinite(avgCpuUtil) ? avgCpuUtil * 100 : null;
  const avgMemoryRssBytes =
    summary.memoryRssCount > 0 ? summary.memoryRssBytesSum / summary.memoryRssCount : null;
  const currentMemoryRssMiB =
    Number.isFinite(avgMemoryRssBytes) && avgMemoryRssBytes >= 0
      ? avgMemoryRssBytes / (1024 * 1024)
      : null;
  const avgMemoryCommitBytes =
    summary.memoryCommitCount > 0
      ? summary.memoryCommitBytesSum / summary.memoryCommitCount
      : null;
  const currentMemoryCommitMiB =
    Number.isFinite(avgMemoryCommitBytes) && avgMemoryCommitBytes >= 0
      ? avgMemoryCommitBytes / (1024 * 1024)
      : null;
  const currentUptimeSeconds = Number.isFinite(summary.uptimeSeconds)
    ? summary.uptimeSeconds
    : null;

  let cpuUtilPercent = currentCpuUtilPercent;
  let memoryRssMiB = currentMemoryRssMiB;
  let memoryCommitMiB = currentMemoryCommitMiB;
  let uptimeSeconds = currentUptimeSeconds;

  if (Number.isFinite(currentCpuUtilPercent)) {
    lastCpuUtilPercent = currentCpuUtilPercent;
  } else if (holdLastValues && Number.isFinite(lastCpuUtilPercent)) {
    cpuUtilPercent = lastCpuUtilPercent;
  }

  if (Number.isFinite(currentMemoryRssMiB)) {
    lastMemoryRssMiB = currentMemoryRssMiB;
  } else if (holdLastValues && Number.isFinite(lastMemoryRssMiB)) {
    memoryRssMiB = lastMemoryRssMiB;
  }
  if (Number.isFinite(currentMemoryCommitMiB)) {
    lastMemoryCommitMiB = currentMemoryCommitMiB;
  } else if (holdLastValues && Number.isFinite(lastMemoryCommitMiB)) {
    memoryCommitMiB = lastMemoryCommitMiB;
  }

  if (Number.isFinite(currentUptimeSeconds)) {
    lastUptimeSeconds = currentUptimeSeconds;
  } else if (holdLastValues && Number.isFinite(lastUptimeSeconds)) {
    uptimeSeconds = lastUptimeSeconds;
  }

  return {
    groupCount: summary.groupCount || 0,
    pipelineCount: summary.pipelineCount || 0,
    coreCount: summary.coreCount || 0,
    cpuUtilPercent,
    memoryRssMiB,
    memoryCommitMiB,
    uptimeSeconds,
    currentCpuUtilPercent,
    currentMemoryRssMiB,
    currentMemoryCommitMiB,
    currentUptimeSeconds,
    lastCpuUtilPercent: Number.isFinite(lastCpuUtilPercent) ? lastCpuUtilPercent : null,
    lastMemoryRssMiB: Number.isFinite(lastMemoryRssMiB) ? lastMemoryRssMiB : null,
    lastMemoryCommitMiB: Number.isFinite(lastMemoryCommitMiB) ? lastMemoryCommitMiB : null,
    lastUptimeSeconds: Number.isFinite(lastUptimeSeconds) ? lastUptimeSeconds : null,
  };
}
