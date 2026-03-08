import {
  getPipelineSelectionKeyFromAttrs,
  normalizeAttributes,
} from "./pipeline-utils.js";

export function isDeltaCounterMetric(metric) {
  if (!metric) return false;
  if (String(metric.temporality || "").toLowerCase() === "delta") return true;
  if (metric.instrument === "delta_counter") return true;
  if (typeof metric.instrument === "string" && metric.instrument.includes("delta")) {
    return true;
  }
  const name = String(metric.name || "").toLowerCase();
  if (name.endsWith(".delta") || name.endsWith(".count")) return true;
  if (name.startsWith("signals.")) return true;
  return false;
}

function getAggregationMode(metric) {
  const instrument = String(metric.instrument || "").toLowerCase();
  const temporality = String(metric.temporality || "").toLowerCase();
  if (temporality === "delta" || temporality === "cumulative") return "sum";
  if (instrument.includes("counter") || instrument.includes("sum")) return "sum";
  if (instrument.includes("gauge")) return "avg";
  if (isDeltaCounterMetric(metric)) return "sum";
  return "avg";
}

export function aggregateMetricSets(metricSets) {
  const groups = new Map();
  metricSets.forEach((set) => {
    const attrs = normalizeAttributes(set.attributes || {});
    const mergedAttrs = { ...attrs };
    delete mergedAttrs["core.id"];
    delete mergedAttrs["numa.node.id"];
    delete mergedAttrs["thread.id"];
    const attrEntries = Object.entries(mergedAttrs).sort(([a], [b]) =>
      a.localeCompare(b)
    );
    const key = `${set.name}::${JSON.stringify(attrEntries)}`;
    let group = groups.get(key);
    if (!group) {
      group = {
        name: set.name,
        attributes: mergedAttrs,
        metrics: new Map(),
      };
      groups.set(key, group);
    }
    (set.metrics || []).forEach((metric) => {
      const metricName = metric.name;
      if (!metricName) return;
      const existing = group.metrics.get(metricName);
      if (typeof metric.value !== "number" || !Number.isFinite(metric.value)) {
        if (!existing) {
          group.metrics.set(metricName, {
            metric: { ...metric },
            sum: 0,
            count: 0,
            mode: getAggregationMode(metric),
            hasNumeric: false,
          });
        }
        return;
      }
      const mode = getAggregationMode(metric);
      const entry =
        existing ||
        {
          metric: { ...metric },
          sum: 0,
          count: 0,
          mode,
          hasNumeric: false,
        };
      entry.sum += metric.value;
      entry.count += 1;
      entry.mode = mode;
      entry.hasNumeric = true;
      group.metrics.set(metricName, entry);
    });
  });

  const aggregated = [];
  groups.forEach((group) => {
    const metrics = [];
    group.metrics.forEach((entry) => {
      if (!entry.hasNumeric) {
        metrics.push(entry.metric);
        return;
      }
      const value = entry.mode === "avg" ? entry.sum / (entry.count || 1) : entry.sum;
      metrics.push({ ...entry.metric, value });
    });
    aggregated.push({
      name: group.name,
      attributes: group.attributes,
      metrics,
    });
  });
  return aggregated;
}

export function filterMetricSets(
  metricSets,
  { selectedPipelineKey = null, selectedCoreId = null, coreAllId = "__all__" } = {}
) {
  const filtered = metricSets.filter((set) => {
    const attrs = normalizeAttributes(set.attributes || {});
    const pipelineKey = getPipelineSelectionKeyFromAttrs(attrs);
    const coreId = attrs["core.id"];
    if (selectedPipelineKey && pipelineKey !== selectedPipelineKey) {
      return false;
    }
    if (selectedCoreId && selectedCoreId !== coreAllId && coreId !== selectedCoreId) {
      return false;
    }
    return true;
  });
  if (selectedCoreId === coreAllId) {
    return aggregateMetricSets(filtered);
  }
  return filtered;
}

export function getDagMetricSets(
  metricSets,
  dagScope,
  { selectedPipelineKey = null, selectedCoreId = null, coreAllId = "__all__" } = {}
) {
  if (dagScope.mode !== "connected" || !(dagScope.pipelineKeys instanceof Set)) {
    return filterMetricSets(metricSets, {
      selectedPipelineKey,
      selectedCoreId,
      coreAllId,
    });
  }

  const scoped = metricSets.filter((set) => {
    const attrs = normalizeAttributes(set.attributes || {});
    const pipelineKey = getPipelineSelectionKeyFromAttrs(attrs);
    return pipelineKey && dagScope.pipelineKeys.has(pipelineKey);
  });
  return aggregateMetricSets(scoped);
}
