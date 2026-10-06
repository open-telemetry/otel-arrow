const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

const appSource = fs.readFileSync(path.join(__dirname, "../shared/app.js"), "utf8");

function dashboard() {
  const chartCalls = [];
  const selector = {};
  const target = { innerHTML: "", querySelector: () => ({}) };
  const context = vm.createContext({
    window: {
      DATA_PATH: "../data/suite",
      COMPARISON_SLUG: "kafka",
      METRICS_META: { logs_received_rate: { label: "Received" } },
    },
    localStorage: { getItem: () => null },
    document: {
      getElementById: (id) => ({
        "comparison-chart": target,
        "metric-select": selector,
      })[id] || null,
      createElement: () => ({
        textContent: "",
        get innerHTML() {
          return this.textContent.replace(/&/g, "&amp;")
            .replace(/</g, "&lt;").replace(/>/g, "&gt;");
        },
      }),
    },
    console,
  });
  vm.runInContext(appSource, context);
  context.createBarChart = (...args) => {
    chartCalls.push(args);
    return {};
  };
  context.createDiagonalPattern = () => "missing-pattern";
  const comparison = {
    slug: "kafka",
    suites: [{ slug: "receiver" }],
    chart: { metrics: { default: "logs_received_rate" } },
  };
  return {
    target,
    chartCalls,
    comparison,
    render: (data) => context.renderComparisonChart(data, comparison, []),
    chartData: (data, tests) => context.buildComparisonChartData(
      data, comparison, tests, "logs_received_rate",
    ),
    rateLabel: vm.runInContext(
      'TIMESERIES_METRICS.find((metric) => metric.key === "logs_received_rate").label',
      context,
    ),
    series: vm.runInContext("TIMESERIES_METRICS", context),
  };
}

function suite(value) {
  return {
    receiver: {
      tests: [{
        name: "100k",
        metrics: [{ name: "logs_received_rate", value, unit: "logs/sec" }],
      }],
    },
  };
}

// Scenario: A comparison is opened before its suite has any published metrics.
// Guarantees: Missing suites and empty results show guidance instead of a null-label crash.
test("unpublished comparisons show an explicit empty state", () => {
  for (const data of [{}, { receiver: { tests: [] } }, suite(null)]) {
    const app = dashboard();
    app.render(data);
    assert.match(app.target.innerHTML, /No published metrics/);
    assert.match(app.target.innerHTML, /serve the site containing its results/);
    assert.doesNotMatch(app.target.innerHTML, /<canvas|<select/);
    assert.equal(app.chartCalls.length, 0);
  }
});

// Scenario: Filters exclude every suite from a comparison.
// Guarantees: The filter-specific message is retained, without drawing a misleading chart.
test("empty filters remain distinguishable from unpublished results", () => {
  const app = dashboard();
  app.comparison.suites = [];
  app.render(suite(100));
  assert.match(app.target.innerHTML, /No suites match the current filters/);
  assert.equal(app.chartCalls.length, 0);
});

// Scenario: Published values exist, but none are permitted by the comparison's metric list.
// Guarantees: An empty allowed metric intersection does not reach metric-label formatting.
test("unavailable allowed metrics show the empty state", () => {
  const app = dashboard();
  app.comparison.chart.metrics.allowed = ["unpublished_metric"];
  app.render(suite(100));
  assert.match(app.target.innerHTML, /No published metrics/);
  assert.equal(app.chartCalls.length, 0);
});

// Scenario: Available data returns after an empty selection, including a measured zero.
// Guarantees: The configured metric is restored and zero is not treated as missing data.
test("comparisons recover when metrics become available", () => {
  for (const value of [0, 123]) {
    const app = dashboard();
    app.render({});
    app.render(suite(value));
    assert.doesNotMatch(app.target.innerHTML, /No published metrics/);
    assert.match(app.target.innerHTML, /Received \(logs\/sec\)/);
    assert.match(app.target.innerHTML, /value="logs_received_rate" selected/);
    assert.equal(app.chartCalls.length, 1);
    assert.equal(app.chartCalls[0][4], "logs_received_rate");
  }
});

// Scenario: A configured default metric has no value, but a different metric is available.
// Guarantees: The existing fallback to the first available metric is preserved.
test("missing defaults still fall back to an available metric", () => {
  const app = dashboard();
  app.comparison.chart.metrics.default = "unpublished_metric";
  app.render(suite(100));
  assert.equal(app.chartCalls.length, 1);
  assert.equal(app.chartCalls[0][4], "logs_received_rate");
});

// Scenario: A comparison mixes a measured zero with null and unrun rate cases.
// Guarantees: Missing bars stay flagged as unavailable instead of measured zeros.
test("partial results distinguish missing values from zero", () => {
  const app = dashboard();
  const data = suite(0);
  data.receiver.tests.push({
    name: "200k", metrics: [{ name: "logs_received_rate", value: null }],
  });
  const chart = app.chartData(data, [
    { name: "100k" }, { name: "200k" }, { name: "300k" },
  ]);
  assert.deepEqual(Array.from(chart.datasets[0]._missing), [false, true, true]);
  assert.equal(chart.datasets[0].data[0], 0);
});

// Scenario: Log input throughput can be measured at local Perf or a backend.
// Guarantees: The common time-series label does not imply remote backend delivery.
test("received log time series uses an endpoint-neutral label", () => {
  assert.equal(dashboard().rateLabel, "Received Log Rate");
});

// Scenario: Multicore runs report aggregate CPU and separate per-core log rates.
// Guarantees: Each scaling series has its own label and matching scalar average.
test("scaling series preserve per-core identity and aggregate CPU", () => {
  const series = dashboard().series;
  for (const core of [1, 2, 3, 4]) {
    const key = `logs_received_rate_core${core}`;
    const metric = series.find((value) => value.key === key);
    assert.equal(metric.avg, key);
    assert.equal(metric.label, `Core ${core} Received Log Rate`);
  }
  const cpu = series.find((value) => value.key === "cpu_percentage_total");
  assert.equal(cpu.label, "CPU Total");
  assert.equal(cpu.avg, "cpu_percentage_total_avg");
});
