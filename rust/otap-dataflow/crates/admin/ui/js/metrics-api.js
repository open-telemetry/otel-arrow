// Keep fallback endpoint lists stable and duplicate-free.
function dedupeUrls(urls) {
  const seen = new Set();
  return urls.filter((url) => {
    if (seen.has(url)) return false;
    seen.add(url);
    return true;
  });
}

// Build same-origin metrics endpoint candidates.
// The embedded UI should read from its own admin origin.
export function buildMetricsCandidates({ query }) {
  const localPaths = ["/telemetry/metrics", "/metrics"];
  const localUrls = localPaths.map((path) => `${path}?${query}`);
  return dedupeUrls(localUrls);
}

// Probe candidate URLs until one returns a valid JSON payload.
// `resolvedUrl` is attempted first so the steady-state path is fast.
// Optional `fetchOptions` allows cancellation (AbortController signal).
export async function fetchMetricsFromCandidates(
  urlCandidates,
  resolvedUrl,
  fetchOptions = {}
) {
  const urls = resolvedUrl
    ? [resolvedUrl, ...urlCandidates.filter((url) => url !== resolvedUrl)]
    : urlCandidates;

  let lastError = new Error("Failed to load metrics from any configured endpoint.");
  for (const url of urls) {
    try {
      const resp = await fetch(url, { cache: "no-store", ...fetchOptions });
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
