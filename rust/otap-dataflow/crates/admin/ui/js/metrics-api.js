// Keep fallback endpoint lists stable and duplicate-free.
function dedupeUrls(urls) {
  const seen = new Set();
  return urls.filter((url) => {
    if (seen.has(url)) return false;
    seen.add(url);
    return true;
  });
}

// Build metrics endpoint candidates in priority order:
// 1) same-origin target-specific
// 2) same-origin generic
// 3) localhost:8085 target-specific
// 4) localhost:8085 generic
// This preserves compatibility across local/dev/proxy setups.
export function buildMetricsCandidates({ targetPath, query }) {
  const normalizedTarget = String(targetPath || "")
    .replace(/^\/+/, "")
    .replace(/\/+$/, "");

  const localTargetPrefix = normalizedTarget ? `/${normalizedTarget}` : "";
  const localPrefixes = [localTargetPrefix, ""];
  const localPaths = ["/telemetry/metrics", "/metrics"];
  const localUrls = localPrefixes.flatMap((prefix) =>
    localPaths.map((path) => `${prefix}${path}?${query}`)
  );

  const port8085Base = "http://localhost:8085";
  const port8085TargetPrefix = normalizedTarget
    ? `${port8085Base}/${normalizedTarget}`
    : port8085Base;
  const port8085Prefixes = [port8085TargetPrefix, port8085Base];
  const port8085Urls = port8085Prefixes.flatMap((prefix) =>
    localPaths.map((path) => `${prefix}${path}?${query}`)
  );

  return dedupeUrls([...localUrls, ...port8085Urls]);
}

// Probe candidate URLs until one returns a valid JSON payload.
// `resolvedUrl` is attempted first so the steady-state path is fast.
export async function fetchMetricsFromCandidates(urlCandidates, resolvedUrl) {
  const urls = resolvedUrl
    ? [resolvedUrl, ...urlCandidates.filter((url) => url !== resolvedUrl)]
    : urlCandidates;

  let lastError = new Error("Failed to load metrics from any configured endpoint.");
  for (const url of urls) {
    try {
      const resp = await fetch(url, { cache: "no-store" });
      if (!resp.ok) {
        throw new Error(`HTTP ${resp.status} ${resp.statusText}`);
      }
      const data = await resp.json();
      return { data, resolvedUrl: url };
    } catch (err) {
      const message = err?.message || String(err);
      lastError = new Error(`${url}: ${message}`);
    }
  }

  throw lastError;
}
