#!/usr/bin/env node
// Perf Regression Watcher
//
// Compares the latest two entries of every docs/benchmarks/**/data.js file
// on the benchmarks branch and reports metrics whose |delta| exceeds a
// configurable threshold by opening or commenting on a single shared issue
// labelled "perf-regression".
//
// Inputs (env):
//   GH_TOKEN              - token with issues:write on REPO
//   REPO                  - "owner/name" where the issue is filed (and labels live)
//   BENCHMARKS_REPO       - "owner/name" of the repo hosting the benchmarks branch
//   BENCHMARKS_BRANCH     - branch holding the data.js files (e.g. "benchmarks")
//   DEFAULT_THRESHOLD_PCT - fallback threshold if config has none / file missing
//   DRY_RUN               - "true" prints findings without touching issues
//   WORKFLOW_RUN_URL      - link to the triggering workflow run (for issue body)
//   CONFIG_JSON           - perf-regression config as JSON (already YAML-parsed)
//
// Exits 0 on success even if regressions are found. Exits non-zero only on
// unexpected errors (network, parse). This keeps the workflow green so the
// signal channel is "did an issue get opened/commented", not "did CI fail".

const {
  GH_TOKEN,
  REPO,
  BENCHMARKS_REPO,
  BENCHMARKS_BRANCH,
  DEFAULT_THRESHOLD_PCT = "10",
  DRY_RUN = "false",
  WORKFLOW_RUN_URL = "",
  CONFIG_JSON = "{}",
  PATH_FILTER = "",
} = process.env;

const LABEL = "perf-regression";
const ISSUE_TITLE = "Nightly perf/size regression watcher: investigation needed";
const DEFAULT_THRESHOLD = Number(DEFAULT_THRESHOLD_PCT);
const DRY = DRY_RUN === "true";

if (!GH_TOKEN || !REPO || !BENCHMARKS_REPO || !BENCHMARKS_BRANCH) {
  console.error("Missing required env vars.");
  process.exit(2);
}

const config = (() => {
  try {
    const c = JSON.parse(CONFIG_JSON);
    return {
      defaultThreshold: Number(c.default_threshold_pct ?? DEFAULT_THRESHOLD),
      overrides: c.metric_overrides ?? [],
      ignored: new Set(c.ignored_paths ?? []),
      monitor: new Set(c.monitor ?? []),
    };
  } catch (e) {
    console.warn(`Config parse failed (${e.message}); using defaults.`);
    return {
      defaultThreshold: DEFAULT_THRESHOLD,
      overrides: [],
      ignored: new Set(),
      monitor: new Set(),
    };
  }
})();

async function ghApi(path, init = {}) {
  const res = await fetch(`https://api.github.com${path}`, {
    ...init,
    headers: {
      "Accept": "application/vnd.github+json",
      "Authorization": `Bearer ${GH_TOKEN}`,
      "User-Agent": "perf-regression-watcher",
      "X-GitHub-Api-Version": "2022-11-28",
      ...(init.headers || {}),
    },
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`GitHub API ${init.method || "GET"} ${path} -> ${res.status}: ${text}`);
  }
  return res.json();
}

async function fetchText(url) {
  const res = await fetch(url, {
    headers: { "User-Agent": "perf-regression-watcher" },
  });
  if (!res.ok) throw new Error(`fetch ${url} -> ${res.status}`);
  return res.text();
}

// Parse a github-action-benchmark data.js file:
//   window.BENCHMARK_DATA = { ... };
function parseDataJs(text) {
  const m = text.match(/=\s*({[\s\S]*})\s*;?\s*$/);
  if (!m) throw new Error("data.js: no JSON object found");
  return JSON.parse(m[1]);
}

function thresholdFor(path, metricName) {
  for (const o of config.overrides) {
    if (o.path !== path) continue;
    if (!o.metrics || o.metrics.includes(metricName)) {
      return Number(o.threshold_pct);
    }
  }
  return config.defaultThreshold;
}

// Returns array of finding objects for one data.js file.
function diffEntries(path, data) {
  const findings = [];
  for (const [suite, runs] of Object.entries(data.entries || {})) {
    if (runs.length < 2) continue;
    const curr = runs[runs.length - 1];
    const prev = runs[runs.length - 2];

    // Skip when the two compared runs are on the same commit — any delta is
    // pure run-to-run variance, not a regression we can attribute to a change.
    if (prev.commit?.id && curr.commit?.id && prev.commit.id === curr.commit.id) {
      console.log(
        `  ${path}: skipped (latest two runs on same commit ${curr.commit.id.slice(0, 7)})`,
      );
      continue;
    }

    // github-action-benchmark publishes one bench row per (scenario, metric)
    // but the `name` field is just the metric — scenario lives in `extra` as
    // "<Suite header>/<SCENARIO-ID> - <label>". Without disambiguating by
    // scenario, a single file like syslog-tcp (4 scenarios × 11 metrics) would
    // collapse to 11 series and "first occurrence" picks whichever scenario the
    // publisher happens to emit first — masking real per-scenario regressions.
    // Key everything by `${scenario}::${name}`. When `extra` is missing or
    // unparseable, scenario is '' and behavior matches the old single-series
    // collapse for that file.
    const keyOf = (b) => `${scenarioOf(b.extra)}::${b.name}`;
    const firstByKey = (benches) => {
      const out = {};
      let dupes = 0;
      for (const b of benches) {
        const k = keyOf(b);
        if (k in out) dupes++;
        else out[k] = b;
      }
      return { map: out, dupes };
    };
    const { map: currByKey, dupes: currDupes } = firstByKey(curr.benches);
    const { map: prevByKey } = firstByKey(prev.benches);
    if (currDupes > 0) {
      console.warn(
        `  ${path}: ${currDupes} duplicate (scenario,metric) pair(s) in latest run; using first occurrence`,
      );
    }

    for (const [k, c] of Object.entries(currByKey)) {
      const p = prevByKey[k];
      if (!p || p.value === 0) continue;
      const deltaPct = ((c.value - p.value) / p.value) * 100;
      const th = thresholdFor(path, c.name);
      if (Math.abs(deltaPct) < th) continue;
      findings.push({
        path,
        suite,
        scenario: scenarioOf(c.extra),
        metric: c.name,
        unit: c.unit || "",
        prevValue: p.value,
        currValue: c.value,
        prevCommit: prev.commit?.id,
        currCommit: curr.commit?.id,
        currCommitUrl: curr.commit?.url,
        deltaPct,
        threshold: th,
        arrow: deltaPct > 0 ? "📈" : "📉",
      });
    }
  }
  return findings;
}

// "docs/benchmarks/nightly/syslog-tcp/data.js" -> "nightly/syslog-tcp"
// "docs/benchmarks/binary-size/data.js"        -> "binary-size"
function shortName(path) {
  return path.replace(/^docs\/benchmarks\//, "").replace(/\/data\.js$/, "");
}

// github-action-benchmark stores per-row metadata in `extra` with the shape:
//   "<Suite header>/<SCENARIO-ID> - <metric label>"
// e.g. "Nightly - Syslog TCP/SYSLOG-TCP-3164-ATTR-OTLP - Log Throughput".
// We use the SCENARIO-ID to disambiguate sub-scenarios that share the same
// metric `name`. Returns '' when extra is missing or doesn't match the shape.
function scenarioOf(extra) {
  if (!extra || typeof extra !== "string") return "";
  const slash = extra.indexOf("/");
  if (slash < 0) return "";
  const tail = extra.slice(slash + 1);
  const dash = tail.lastIndexOf(" - ");
  return (dash < 0 ? tail : tail.slice(0, dash)).trim();
}

async function fetchCommitRange(prev, curr) {
  // Returns array of {sha, title, url} for commits in (prev, curr].
  try {
    const cmp = await ghApi(
      `/repos/${BENCHMARKS_REPO}/compare/${prev}...${curr}`,
    );
    return (cmp.commits || []).map((c) => ({
      sha: c.sha,
      title: (c.commit?.message || "").split("\n")[0],
      url: c.html_url,
    }));
  } catch (err) {
    console.warn(`  compare ${prev.slice(0, 7)}..${curr.slice(0, 7)} failed: ${err.message}`);
    return null;
  }
}

async function buildCommitRanges(findings) {
  const ranges = new Map();
  const pairs = new Set();
  for (const f of findings) {
    if (f.prevCommit && f.currCommit) pairs.add(`${f.prevCommit}..${f.currCommit}`);
  }
  for (const key of pairs) {
    const [prev, curr] = key.split("..");
    if (prev === curr) continue; // same commit, no range to fetch
    ranges.set(key, await fetchCommitRange(prev, curr));
  }
  return ranges;
}

function renderBody(findings, commitRanges = new Map()) {
  const lines = [];
  lines.push(
    `<!-- watcher:auto-generated. Do not edit the table headers; the watcher uses them to dedup. -->`,
  );
  lines.push("");
  lines.push(
    `The nightly perf-regression watcher flagged the following benchmark deltas against the previous nightly. Investigate whether each change is intentional and either close this issue or open follow-up issues per metric.`,
  );
  lines.push("");
  if (WORKFLOW_RUN_URL) lines.push(`Watcher run: ${WORKFLOW_RUN_URL}`);
  lines.push(
    `Dashboards: https://${BENCHMARKS_REPO.split("/")[0]}.github.io/${BENCHMARKS_REPO.split("/")[1]}/benchmarks/`,
  );
  lines.push("");
  lines.push("| | Benchmark | Scenario | Metric | Previous | Current | Δ | Threshold |");
  lines.push("|---|---|---|---|---|---|---|---|");
  for (const f of findings) {
    const unit = f.unit ? ` ${f.unit}` : "";
    const delta = `${f.deltaPct >= 0 ? "+" : ""}${f.deltaPct.toFixed(2)}%`;
    const benchUrl = `https://github.com/${BENCHMARKS_REPO}/blob/${BENCHMARKS_BRANCH}/${f.path}`;
    const scen = f.scenario ? `\`${f.scenario}\`` : "—";
    lines.push(
      `| ${f.arrow} | [\`${shortName(f.path)}\`](${benchUrl}) | ${scen} | \`${f.metric}\` | ${f.prevValue}${unit} | ${f.currValue}${unit} | ${delta} | ±${f.threshold}% |`,
    );
  }
  lines.push("");

  // Per-pair commit ranges. A pair appears once even if many benchmarks share it.
  const pairs = new Map();
  for (const f of findings) {
    if (!f.prevCommit || !f.currCommit) continue;
    const key = `${f.prevCommit}..${f.currCommit}`;
    if (!pairs.has(key)) {
      pairs.set(key, {
        prev: f.prevCommit,
        curr: f.currCommit,
        benchmarks: new Set(),
      });
    }
    pairs.get(key).benchmarks.add(shortName(f.path));
  }
  if (pairs.size > 0) {
    lines.push("### Commits in compared range");
    lines.push("");
    for (const { prev, curr, benchmarks } of pairs.values()) {
      const compareUrl = `https://github.com/${BENCHMARKS_REPO}/compare/${prev}...${curr}`;
      const benchList = [...benchmarks].sort().join(", ");
      lines.push(`**\`${prev.slice(0, 7)}\` → [\`${curr.slice(0, 7)}\`](${compareUrl})** (${benchList})`);
      lines.push("");
      const commits = commitRanges.get(`${prev}..${curr}`);
      if (commits === null) {
        lines.push(`_Failed to fetch commit range._`);
      } else if (!commits || commits.length === 0) {
        lines.push(`_No commits in range._`);
      } else {
        const cap = 25;
        for (const c of commits.slice(0, cap)) {
          lines.push(`- [\`${c.sha.slice(0, 7)}\`](${c.url}) ${c.title}`);
        }
        if (commits.length > cap) {
          lines.push(`- _… and ${commits.length - cap} more (see [compare](${compareUrl}))_`);
        }
      }
      lines.push("");
    }
  }

  const fingerprints = findings
    .map((f) => `${f.path}::${f.scenario || ""}::${f.metric}::${f.currCommit ?? "?"}`)
    .sort();
  lines.push(`<!-- fingerprints:${fingerprints.join(",")} -->`);
  return lines.join("\n");
}

async function findOpenIssue() {
  const issues = await ghApi(
    `/repos/${REPO}/issues?state=open&labels=${encodeURIComponent(LABEL)}&per_page=10`,
  );
  return issues[0] || null;
}

function bodyFingerprints(body) {
  const m = (body || "").match(/<!-- fingerprints:([^>]*)-->/);
  if (!m) return new Set();
  return new Set(
    m[1]
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean),
  );
}

async function getRecentFingerprints(issueNumber) {
  const fingerprints = new Set();
  const issue = await ghApi(`/repos/${REPO}/issues/${issueNumber}`);
  for (const fp of bodyFingerprints(issue.body)) fingerprints.add(fp);
  const comments = await ghApi(
    `/repos/${REPO}/issues/${issueNumber}/comments?per_page=100`,
  );
  for (const c of comments) {
    for (const fp of bodyFingerprints(c.body)) fingerprints.add(fp);
  }
  return fingerprints;
}

async function ensureLabel() {
  try {
    await ghApi(`/repos/${REPO}/labels/${encodeURIComponent(LABEL)}`);
  } catch {
    if (DRY) return;
    await ghApi(`/repos/${REPO}/labels`, {
      method: "POST",
      body: JSON.stringify({
        name: LABEL,
        color: "d93f0b",
        description: "Nightly perf/size benchmark regressed beyond configured threshold",
      }),
    });
  }
}

async function main() {
  // 1. Discover data.js files under docs/benchmarks/ on the benchmarks branch.
  const tree = await ghApi(
    `/repos/${BENCHMARKS_REPO}/git/trees/${BENCHMARKS_BRANCH}?recursive=1`,
  );
  const dataPaths = tree.tree
    .filter((n) => n.type === "blob")
    .map((n) => n.path)
    .filter(
      (p) =>
        p.startsWith("docs/benchmarks/") &&
        p.endsWith("/data.js") &&
        !config.ignored.has(p) &&
        // monitor allowlist (when non-empty) and ad-hoc PATH_FILTER both apply.
        (config.monitor.size === 0 || config.monitor.has(p)) &&
        (!PATH_FILTER || p.includes(PATH_FILTER)),
    );
  const scopeNotes = [];
  if (config.monitor.size > 0) scopeNotes.push(`monitor allowlist: ${config.monitor.size}`);
  if (PATH_FILTER) scopeNotes.push(`filter: "${PATH_FILTER}"`);
  console.log(
    `Discovered ${dataPaths.length} data.js files on ${BENCHMARKS_REPO}@${BENCHMARKS_BRANCH}${scopeNotes.length ? ` (${scopeNotes.join("; ")})` : ""}`,
  );

  // 2. Fetch + diff each.
  const allFindings = [];
  for (const path of dataPaths) {
    try {
      const raw = await fetchText(
        `https://raw.githubusercontent.com/${BENCHMARKS_REPO}/${BENCHMARKS_BRANCH}/${path}`,
      );
      const data = parseDataJs(raw);
      const f = diffEntries(path, data);
      console.log(`  ${path}: ${f.length} finding(s)`);
      allFindings.push(...f);
    } catch (e) {
      console.warn(`  ${path}: skipped (${e.message})`);
    }
  }

  if (allFindings.length === 0) {
    console.log("No regressions above threshold. Done.");
    return;
  }

  console.log(`\nTotal findings: ${allFindings.length}`);
  const commitRanges = await buildCommitRanges(allFindings);
  const body = renderBody(allFindings, commitRanges);
  console.log("\n--- Proposed issue body ---");
  console.log(body);
  console.log("--- End body ---\n");

  if (DRY) {
    console.log("DRY_RUN=true; not opening/commenting an issue.");
    return;
  }

  await ensureLabel();

  // 3. Find existing open issue.
  const existing = await findOpenIssue();
  const newFingerprints = new Set(
    allFindings.map((f) => `${f.path}::${f.metric}::${f.currCommit ?? "?"}`),
  );

  if (existing) {
    const seen = await getRecentFingerprints(existing.number);
    const novel = [...newFingerprints].filter((fp) => !seen.has(fp));
    if (novel.length === 0) {
      console.log(`Issue #${existing.number} already covers all current findings; skipping comment.`);
      return;
    }
    console.log(`Commenting on existing issue #${existing.number} with ${novel.length} new finding(s).`);
    const novelFindings = allFindings.filter((f) =>
      novel.includes(`${f.path}::${f.metric}::${f.currCommit ?? "?"}`),
    );
    const novelRanges = await buildCommitRanges(novelFindings);
    await ghApi(`/repos/${REPO}/issues/${existing.number}/comments`, {
      method: "POST",
      body: JSON.stringify({ body: renderBody(novelFindings, novelRanges) }),
    });
    return;
  }

  console.log("Opening new issue.");
  const issue = await ghApi(`/repos/${REPO}/issues`, {
    method: "POST",
    body: JSON.stringify({
      title: ISSUE_TITLE,
      body,
      labels: [LABEL],
    }),
  });
  console.log(`Opened: ${issue.html_url}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
